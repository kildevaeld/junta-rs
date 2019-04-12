use super::chain::ProtocolChain;
//use super::context::ProtocolContext;
use super::event::*;
use super::middleware::{ProtocolChainHandler, ProtocolHandable};
use super::response_protocol::ResponseProtocol;
use futures::prelude::*;
// use futures::sync::mpsc::{channel as multi_channel, Receiver, Sender};
use futures::sync::oneshot::{channel, Receiver as OneReceiver, Sender as OneSender};
use junta::prelude::*;
use junta_persist::{PersistMiddleware, State};
use junta_service::prelude::*;
use serde_cbor::Value;
// use std::sync::{Arc, RwLock};

pub struct Listener {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) sender: OneSender<JuntaResult<Value>>,
}

impl Listener {
    pub fn new(id: usize, name: String) -> (Listener, OneReceiver<JuntaResult<Value>>) {
        let (sx, rx) = channel();
        (
            Listener {
                id,
                name,
                sender: sx,
            },
            rx,
        )
    }
}

pub struct ListenerList;

impl Key for ListenerList {
    type Value = Vec<Listener>;
}

// pub struct Subscriber {
//     pub(crate) id: i32,
//     pub(crate) name: String,
//     pub(crate) sender: Sender<Value>,
// }

// impl Subscriber {
//     pub fn new(id: i32, name: String) -> (Subscriber, Receiver<Value>) {
//         let (sx, rx) = multi_channel(5);
//         (
//             Subscriber {
//                 id,
//                 name,
//                 sender: sx,
//             },
//             rx,
//         )
//     }
// }

// pub type SubscriberList = Arc<RwLock<Vec<Listener>>>;

pub trait Protocol: Sync + Send {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn execute(&self, ctx: ChildContext<ClientEvent, Event>) -> Self::Future;
    fn check(&self, _ctx: &BorrowedContext<ClientEvent, Event>) -> bool {
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

pub struct ProtocolService<S: Protocol> {
    service: ProtocolChainHandler<
        PersistMiddleware<
            ListenerList,
            State<ListenerList>,
            ChildContext<ClientEvent, Event>,
            (),
            JuntaError,
        >,
        ProtocolChain<ResponseProtocol, S>,
    >,
}

impl<S> ProtocolService<S>
where
    S: Protocol + 'static,
    <S as Protocol>::Future: 'static + Send,
{
    pub fn new(service: S) -> ProtocolService<S> {
        let resp = ResponseProtocol {};
        let middleware = State::<ListenerList>::middleware(Vec::new());
        let service = middleware.then_protocol(ProtocolChain::new(resp, service));

        ProtocolService { service }
    }
}

impl<S: Sync + Send + 'static> Service for ProtocolService<S>
where
    S: Protocol,
    <S as Protocol>::Future: 'static + Send,
{
    type Input = Context<ClientEvent>;
    type Output = ();
    type Error = JuntaError;
    type Future = OneOfTwoFuture<
        (),
        JuntaError,
        <ProtocolChainHandler<
            PersistMiddleware<
                ListenerList,
                State<ListenerList>,
                ChildContext<ClientEvent, Event>,
                (),
                JuntaError,
            >,
            ProtocolChain<ResponseProtocol, S>,
        > as Protocol>::Future,
        futures::future::FutureResult<(), JuntaError>,
    >;

    fn call(&self, ctx: Context<ClientEvent>) -> Self::Future {
        let fut = match ctx.message() {
            ClientEvent::Message(_) => {
                let event = ctx.decode::<Event>().unwrap();
                OneOfTwo::Future1(self.service.execute(ChildContext::new(ctx, event)))
            }
            _ => OneOfTwo::Future2(futures::future::ok(())),
        };
        OneOfTwoFuture::new(fut)
    }

    fn should_call(&self, ctx: &Context<ClientEvent>) -> bool {
        let event = match ctx.decode::<Event>() {
            Ok(event) => event,
            Err(_) => return false,
        };
        let borrow = BorrowedContext::new(ctx, &event);
        self.service.check(&borrow)
    }
}
