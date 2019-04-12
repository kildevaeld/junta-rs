use super::event::*;
use super::protocol::ListenerList;
use super::protocol::Protocol;
use junta::prelude::*;
use junta_persist::*;
use plugins::*;

pub struct ResponseProtocol;

impl Protocol for ResponseProtocol {
    type Future = futures::future::FutureResult<(), JuntaError>; // Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    fn execute(&self, mut ctx: ChildContext<ClientEvent, Event>) -> Self::Future {
        let (name, data) = match ctx.message().clone().event_type {
            EventType::Res(name, data) => (name, data),
            _ => {
                return futures::future::err(
                    JuntaErrorKind::Unknown("invalid response".to_string()).into(),
                );
            }
        };

        let listeners = ctx.get::<State<ListenerList>>().unwrap();
        let index = listeners
            .read()
            .unwrap()
            .iter()
            .position(|m| m.id == ctx.message().id && m.name.as_str() == name);
        let fut = if index.is_none() {
            futures::future::err(JuntaErrorKind::Unknown("invalid response".to_string()).into())
        } else {
            let index = index.unwrap();
            let listener = listeners.write().unwrap().remove(index);
            match listener.sender.send(Ok(data)) {
                Ok(_) => {}
                Err(_) => {
                    error!(
                        ctx.client().logger(),
                        "could not trigger response. channel was closed"
                    );
                }
            }
            futures::future::ok(())
        };

        fut
    }

    fn check(&self, ctx: &BorrowedContext<ClientEvent, Event>) -> bool {
        let name = match &ctx.message().event_type {
            EventType::Res(name, _) => name,
            _ => return false,
        };
        let listeners = match ctx.extensions().get::<State<ListenerList>>() {
            Some(l) => l.read().unwrap(),
            None => {
                return false;
            }
        };

        listeners
            .iter()
            .find(|m| m.id == ctx.message().id && m.name.as_str() == name)
            .is_some()
    }
}
