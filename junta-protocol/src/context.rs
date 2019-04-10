use super::event::*;
use super::protocol::{Listener, ListenerList, Subscriber, SubscriberList};
use atomic_counter::AtomicCounter;
use futures::future;
use futures::sync::oneshot::channel;
use futures::Future;
use junta::prelude::*;
use junta_service::plugins::{Extensible, Pluggable};
use junta_service::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use typemap::ShareMap;

pub struct ProtocolContext<I> {
    pub(crate) ctx: Context,
    pub(crate) data: I,
    pub(crate) listeners: ListenerList,
    pub(crate) subscribers: SubscriberList,
    pub(crate) counter: Arc<atomic_counter::RelaxedCounter>,
}

impl<I> ProtocolContext<I> {
    pub fn ctx(&self) -> &Context {
        &self.ctx
    }
    pub fn data(&self) -> &I {
        &self.data
    }

    pub fn send<S: Serialize>(&self, data: &S) -> impl Future<Item = (), Error = JuntaError> {
        self.ctx.send(data)
    }

    pub fn with_data<S>(self, data: S) -> ProtocolContext<S> {
        ProtocolContext {
            ctx: self.ctx,
            data: data,
            listeners: self.listeners,
            subscribers: self.subscribers,
            counter: self.counter,
        }
    }

    pub fn request<Str: AsRef<str>, S: Serialize, D: DeserializeOwned>(
        &self,
        name: Str,
        data: &S,
    ) -> impl Future<Item = D, Error = JuntaError> {
        let id = self.counter.inc() as i32;
        let name = name.as_ref().to_string();

        let (listener, rx) = Listener::new(id, name.clone());

        {
            self.listeners.write().unwrap().push(listener);
        }

        let value = serde_cbor::to_value(data).unwrap();

        let event = Event::new(id, EventType::Req(name, value));

        self.ctx
            .send(&event)
            .map_err(|e| JuntaError::new(JuntaErrorKind::Unknown))
            .and_then(|_| {
                rx.map_err(|e| JuntaError::new(JuntaErrorKind::Unknown))
                    .and_then(|resp| match resp {
                        Ok(m) => match serde_cbor::from_value(m) {
                            Ok(m) => future::ok(m),
                            Err(e) => future::err(JuntaErrorKind::Error(e.to_string()).into()),
                        },
                        Err(e) => future::err(e),
                    })
            })
    }
}

impl ProtocolContext<serde_cbor::Value> {
    pub fn decode<D: DeserializeOwned>(&self) -> JuntaResult<D> {
        serde_cbor::from_value(self.data().clone())
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
    }
}

impl<I> Extensible for ProtocolContext<I> {
    fn extensions(&self) -> &ShareMap {
        &self.ctx.extensions()
    }

    fn extensions_mut(&mut self) -> &mut ShareMap {
        self.ctx.extensions_mut()
    }
}

impl<I> Pluggable for ProtocolContext<I> {}
