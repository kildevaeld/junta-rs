use super::event::*;
use super::protocol::{Listener, ListenerList};
use futures::future;
use futures::prelude::*;
use junta::prelude::*;
use junta_persist::State;
use junta_service::error::ServiceError;
use plugins::*;
use serde::{de::DeserializeOwned, Serialize};

pub trait ContextExt {
    fn request<Str: AsRef<str>, S: Serialize, D: DeserializeOwned + Send + 'static>(
        &mut self,
        name: Str,
        msg: &S,
    ) -> Box<Future<Item = D, Error = JuntaError> + Send + 'static>;
}

impl<I: 'static> ContextExt for Context<I> {
    fn request<Str: AsRef<str>, S: Serialize, D: DeserializeOwned + Send + 'static>(
        &mut self,
        name: Str,
        data: &S,
    ) -> Box<Future<Item = D, Error = JuntaError> + Send + 'static> {
        let id = self.client().next_seq();
        let name = name.as_ref().to_string();

        let listeners = self.get::<State<ListenerList>>().unwrap();

        let (listener, rx) = Listener::new(id, name.clone());

        {
            listeners.write().unwrap().push(listener);
        }

        let value = serde_cbor::to_value(data).unwrap();

        let event = Event::new(id, EventType::Req(name, value));

        let fut = self
            .send(&event)
            //.map_err(|e| JuntaError::new(JuntaErrorKind::Unknown))
            .and_then(|_| {
                rx.map_err(|_| JuntaError::from(ServiceError::ReceiverClosed))
                    .and_then(|resp| match resp {
                        Ok(m) => match serde_cbor::from_value(m) {
                            Ok(m) => future::ok(m),
                            Err(e) => future::err(JuntaError::from(e)),
                        },
                        Err(e) => future::err(e),
                    })
            });

        Box::new(fut)
    }
}
