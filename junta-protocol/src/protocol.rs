use super::event::*;
use futures::prelude::*;
use junta::prelude::*;
use junta_service::*;
use serde::Deserialize;
pub trait Protocol {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn execute(&self, ctx: Context, event: Event) -> Self::Future;
    fn check(&self, _ctx: &Context, _event: &Event) -> bool {
        true
    }
}

pub trait IntoProtocol {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    type Protocol: Protocol<Future = Self::Future>;
    fn into_protocol(self) -> Self::Protocol;
}

impl<S> IntoProtocol for S
where
    S: Protocol,
{
    type Future = S::Future;
    type Protocol = S;
    fn into_protocol(self) -> Self::Protocol {
        self
    }
}

pub struct ProtocolService<S> {
    service: S,
}

impl<S> ProtocolService<S>
where
    S: Protocol,
    <S as Protocol>::Future: 'static + Send,
{
    pub fn new(service: S) -> ProtocolService<S> {
        ProtocolService { service }
    }
}

impl<S> Service for ProtocolService<S>
where
    S: Protocol,
    <S as Protocol>::Future: 'static + Send,
{
    type Future = S::Future;
    fn execute(&self, ctx: Context) -> Self::Future {
        let event = ctx.decode::<Event>().unwrap();
        self.service.execute(ctx, event)
    }

    fn check(&self, ctx: &Context) -> bool {
        // let m = Event::new(
        //     1,
        //     EventType::Req(
        //         "Test".to_string(),
        //         serde_cbor::Value::String("Hello world".to_string()),
        //     ),
        // );
        // let json = serde_json::to_string_pretty(&m).unwrap();
        // println!("{}", json);

        let event = match ctx.decode::<Event>() {
            Ok(event) => event,
            Err(_) => return false,
        };
        self.service.check(ctx, &event)
    }
}
