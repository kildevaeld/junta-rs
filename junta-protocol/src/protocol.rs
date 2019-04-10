use super::chain::ProtocolChain;
use super::context::ProtocolContext;
use super::event::*;
use super::response_protocol::ResponseProtocol;
use futures::prelude::*;
use futures::sync::mpsc::{channel as multi_channel, Receiver, Sender};
use futures::sync::oneshot::{channel, Receiver as OneReceiver, Sender as OneSender};
use junta::prelude::*;
use junta_service::*;
use serde_cbor::Value;
use std::sync::{Arc, RwLock};

pub struct Listener {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) sender: OneSender<JuntaResult<Value>>,
}

impl Listener {
    pub fn new(id: i32, name: String) -> (Listener, OneReceiver<JuntaResult<Value>>) {
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

pub type ListenerList = Arc<RwLock<Vec<Listener>>>;

pub struct Subscriber {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) sender: Sender<Value>,
}

impl Subscriber {
    pub fn new(id: i32, name: String) -> (Subscriber, Receiver<Value>) {
        let (sx, rx) = multi_channel(5);
        (
            Subscriber {
                id,
                name,
                sender: sx,
            },
            rx,
        )
    }
}

pub type SubscriberList = Arc<RwLock<Vec<Listener>>>;

pub trait Protocol {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn execute(&self, ctx: ProtocolContext<Event>) -> Self::Future;
    fn check(&self, ctx: &Context, event: &Event) -> bool {
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
    service: ProtocolChain<ResponseProtocol, S>,
    listeners: ListenerList,
    subscribers: SubscriberList,
    counter: Arc<atomic_counter::RelaxedCounter>,
}

impl<S> ProtocolService<S>
where
    S: Protocol,
    <S as Protocol>::Future: 'static + Send,
{
    pub fn new(service: S) -> ProtocolService<S> {
        let listeners = Arc::new(RwLock::new(Vec::new()));
        let resp = ResponseProtocol {
            listeners: listeners.clone(),
        };
        let subscribers = Arc::new(RwLock::new(Vec::new()));
        let service = ProtocolChain::new(resp, service);
        ProtocolService {
            service,
            listeners,
            subscribers,
            counter: Arc::new(atomic_counter::RelaxedCounter::new(1)),
        }
    }
}

impl<S> Service for ProtocolService<S>
where
    S: Protocol,
    <S as Protocol>::Future: 'static + Send,
{
    type Future = <ProtocolChain<ResponseProtocol, S> as Protocol>::Future;

    fn execute(&self, ctx: Context) -> Self::Future {
        let event = ctx.decode::<Event>().unwrap();
        let ctx = ProtocolContext {
            ctx,
            data: event,
            listeners: self.listeners.clone(),
            subscribers: self.subscribers.clone(),
            counter: self.counter.clone(),
        };
        self.service.execute(ctx)
    }

    fn check(&self, ctx: &Context) -> bool {
        let event = match ctx.decode::<Event>() {
            Ok(event) => event,
            Err(_) => return false,
        };

        self.service.check(ctx, &event)
    }
}
