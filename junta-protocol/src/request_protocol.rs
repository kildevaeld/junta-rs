// use super::context::Context;
use super::event::*;
use super::protocol::{Protocol, ProtocolService};
use erased_serde::Serialize as ESerialize;
use futures::prelude::*;
use junta::prelude::*;
use junta_service::prelude::*;
use serde_cbor::Value;

pub trait RequestProtocolService {
    type Future: Future<Item = Box<dyn ESerialize>, Error = JuntaError>;
    fn execute(&self, ctx: Context<Value>) -> Self::Future;
}

pub struct RequestProtocolServiceFn<F> {
    inner: F,
}

impl<F> RequestProtocolServiceFn<F> {
    pub fn into_protocol<S: AsRef<str>>(self, name: S) -> RequestProtocol<Self> {
        RequestProtocol::new(name, self)
    }
}

impl<F, U, I> RequestProtocolService for RequestProtocolServiceFn<F>
where
    F: Fn(Context<Value>) -> U,
    U: IntoFuture<Item = I, Error = JuntaError>,
    <U as IntoFuture>::Future: 'static + Send,
    I: serde::Serialize + 'static,
{
    type Future = Box<Future<Item = Box<ESerialize>, Error = JuntaError> + Send + 'static>;
    fn execute(&self, ctx: Context<Value>) -> Self::Future {
        let out = (self.inner)(ctx);
        Box::new(
            out.into_future()
                .map(|o| Box::new(o) as Box<dyn ESerialize>),
        )
    }
}

pub fn protocol_req_fn<S: AsRef<str>, F: Send + Sync, U, I: serde::Serialize + 'static>(
    name: S,
    func: F,
) -> RequestProtocol<RequestProtocolServiceFn<F>>
where
    F: Fn(Context<Value>) -> U,
    U: IntoFuture<Item = I, Error = JuntaError> + Send,
    <U as IntoFuture>::Future: Send + 'static,
{
    RequestProtocol::new(name, RequestProtocolServiceFn { inner: func })
}

pub struct RequestProtocol<S> {
    service: S,
    name: String,
}

impl<S> RequestProtocol<S> {
    pub fn new<N: AsRef<str>>(name: N, service: S) -> RequestProtocol<S> {
        RequestProtocol {
            service,
            name: name.as_ref().to_string(),
        }
    }
}

impl<S: Sync + Send + 'static> Protocol for RequestProtocol<S>
where
    S: RequestProtocolService,
    <S as RequestProtocolService>::Future: 'static + Send,
{
    type Future = Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    fn execute(&self, ctx: ChildContext<ClientEvent, Event>) -> Self::Future {
        let out = match ctx.message().event_type.clone() {
            EventType::Req(name, req) => {
                let id = ctx.message().id;
                let name = name.to_string();
                let binary = ctx.binary();
                let client = ctx.client().clone();
                OneOfTwo::Future1(
                    self.service
                        .execute(ctx.into_parent().with_message(req).0)
                        .and_then(move |ret| {
                            let value = serde_cbor::to_value(ret).unwrap();
                            if binary {
                                client.send_binary(&Event::new(id, EventType::Res(name, value)))
                            } else {
                                client.send_text(&Event::new(id, EventType::Res(name, value)))
                            }
                        })
                        .map(|_| ()),
                )
            }
            _ => OneOfTwo::Future2(futures::future::err(
                JuntaErrorKind::Unknown("invalid request".to_string()).into(),
            )),
        };
        Box::new(OneOfTwoFuture::new(out))
    }

    fn check(&self, ctx: &BorrowedContext<ClientEvent, Event>) -> bool {
        match &ctx.message().event_type {
            EventType::Req(name, _) => name == self.name.as_str(),
            _ => false,
        }
    }
}

impl<S: Send + Sync + 'static> IntoService for RequestProtocol<S>
where
    S: RequestProtocolService,
    <S as RequestProtocolService>::Future: 'static + Send,
{
    type Input = Context<ClientEvent>;
    type Output = ();
    type Error = JuntaError;
    type Future = <ProtocolService<Self> as Service>::Future; //Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    type Service = ProtocolService<Self>;
    fn into_service(self) -> Self::Service {
        ProtocolService::new(self)
    }
}
