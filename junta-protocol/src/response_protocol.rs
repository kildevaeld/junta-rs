use super::event::*;
use super::protocol::ListenerList;
use super::protocol::Protocol;
use futures::prelude::*;
use junta::prelude::*;
use junta_persist::*;
use plugins::*;

pub struct ResponseProtocol {
    //pub(crate) listeners: ListenerList,
}

impl Protocol for ResponseProtocol {
    type Future = Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    fn execute(&self, mut ctx: ChildContext<ClientEvent, Event>) -> Self::Future {
        let (name, data) = match ctx.message().clone().event_type {
            EventType::Res(name, data) => (name, data),
            _ => {
                return Box::new(futures::future::err(JuntaErrorKind::Unknown.into()));
            }
        };

        let listeners = ctx.get::<State<ListenerList>>().unwrap(); //.write().unwrap();
        let index = listeners
            .read()
            .unwrap()
            .iter()
            .position(|m| m.id == ctx.message().id && m.name.as_str() == name);
        let fut = if index.is_none() {
            futures::future::err(JuntaErrorKind::Unknown.into())
        } else {
            let index = index.unwrap();
            let listener = listeners.write().unwrap().remove(index);
            listener.sender.send(Ok(data.clone()));
            futures::future::ok(())
        };

        Box::new(fut)
    }

    fn check(&self, ctx: &BorrowedContext<ClientEvent, Event>) -> bool {
        let name = match &ctx.message().event_type {
            EventType::Res(name, _) => name,
            _ => return false,
        };
        let listeners = match ctx.extensions().get::<State<ListenerList>>() {
            Some(l) => l.read().unwrap(),
            None => {
                println!("could not find extensions");
                return false;
            }
        };
        //let listeners = ctx.get::<ListenerList>().unwrap().read().unwrap();
        listeners
            .iter()
            .find(|m| m.id == ctx.message().id && m.name.as_str() == name)
            .is_some()
    }
}
