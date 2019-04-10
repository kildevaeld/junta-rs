use super::context::ProtocolContext;
use super::event::*;
use super::protocol::ListenerList;
use super::protocol::Protocol;
use futures::prelude::*;
use junta::prelude::*;
use junta_service::*;

pub struct ResponseProtocol {
    pub(crate) listeners: ListenerList,
}

impl Protocol for ResponseProtocol {
    type Future = Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    fn execute(&self, ctx: ProtocolContext<Event>) -> Self::Future {
        let (name, data) = match &ctx.data().event_type {
            EventType::Res(name, data) => (name, data),
            _ => {
                return Box::new(futures::future::err(JuntaErrorKind::Unknown.into()));
            }
        };
        let mut listeners = self.listeners.write().unwrap();
        let index = listeners
            .iter()
            .position(|m| m.id == ctx.data().id && m.name.as_str() == name);
        let fut = if index.is_none() {
            futures::future::err(JuntaErrorKind::Unknown.into())
        } else {
            let index = index.unwrap();
            let listener = listeners.remove(index);
            listener.sender.send(Ok(data.clone()));
            futures::future::ok(())
        };

        Box::new(fut)
    }

    fn check(&self, _ctx: &Context, event: &Event) -> bool {
        let name = match &event.event_type {
            EventType::Res(name, _) => name,
            _ => return false,
        };
        let listeners = self.listeners.read().unwrap();
        listeners
            .iter()
            .find(|m| m.id == event.id && m.name.as_str() == name)
            .is_some()
    }
}
