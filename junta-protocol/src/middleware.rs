use super::event::Event;
use super::protocol::{IntoProtocol, Protocol};
use futures::prelude::*;
use futures::sync::oneshot::{Receiver, Sender};
use junta::prelude::*;
use junta_middleware::{IntoMiddleware, Middleware, Next};
use std::sync::Arc;
// use junta_service::*;

#[derive(Clone)]
pub struct ProtocolMiddlewareChain<S, F> {
    s: Arc<S>,
    f: Arc<F>,
}

impl<S: Middleware<I>, F, I: Send + 'static> Middleware<I> for ProtocolMiddlewareChain<S, F>
where
    S: Middleware<I> + Send + Sync + 'static,
    F: Middleware<I> + Send + Sync + 'static,
    <F as Middleware<I>>::Future: Sync,
    //<S as Middleware>::Item: Send,
{
    //type Item = S::Item;
    type Future = ProtocolMiddlewareChainFuture<S::Future>;
    fn call(&self, req: I, next: Next<I>) -> Self::Future {
        let (n, sx, rx) = Next::new();
        let f = self.f.clone();
        let fut = rx
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
            .and_then(move |req| f.call(req, next));

        let fut2 = self.s.call(req, n);
        ProtocolMiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }
}

pub trait ProtocolMiddlewareChainable<I>: Sized {
    fn stack<M: IntoMiddleware<I>>(self, other: M) -> ProtocolMiddlewareChain<Self, M::Middleware>;
}

impl<T, I> ProtocolMiddlewareChainable<I> for T
where
    T: Middleware<I>,
{
    fn stack<M: IntoMiddleware<I>>(self, other: M) -> ProtocolMiddlewareChain<Self, M::Middleware> {
        ProtocolMiddlewareChain {
            s: Arc::new(self),
            f: Arc::new(other.into_middleware()),
        }
    }
}

pub trait ProtocolHandable: Sized + Middleware<ChildContext<ClientEvent, Event>> {
    fn proto<M: IntoProtocol>(self, handler: M) -> ProtocolChainHandler<Self, M::Protocol> {
        ProtocolChainHandler {
            s: Arc::new(self),
            f: Arc::new(handler.into_protocol()),
        }
    }
    //fn to_handler(self) -> ProtocolChainHandler<Self, NotFoundHandler>;
}

impl<T> ProtocolHandable for T where T: Middleware<ChildContext<ClientEvent, Event>> {}

pub struct ProtocolMiddlewareChainFuture<F: Future> {
    s: Option<Box<Future<Item = (), Error = JuntaError> + Send>>,
    f: F,
    sx: Option<Sender<JuntaResult<()>>>,
}

impl<F: Future> ProtocolMiddlewareChainFuture<F> {
    pub fn new(
        s: Box<Future<Item = (), Error = JuntaError> + Send>,
        f: F,
        sx: Sender<JuntaResult<()>>,
    ) -> ProtocolMiddlewareChainFuture<F> {
        ProtocolMiddlewareChainFuture {
            s: Some(s),
            f,
            sx: Some(sx),
        }
    }
}

impl<F: Future> Future for ProtocolMiddlewareChainFuture<F>
where
    F: Future<Item = (), Error = JuntaError>,
{
    type Item = ();
    type Error = JuntaError;

    fn poll(self: &mut Self) -> Poll<Self::Item, Self::Error> {
        match &mut self.s {
            Some(fut) => match fut.poll() {
                Ok(Async::NotReady) => {}
                Ok(Async::Ready(m)) => {
                    match self.sx.take().unwrap().send(Ok(m)) {
                        Err(_) => panic!("send channel closed"),
                        _ => {}
                    };
                    self.s = None;
                }
                Err(e) => return Err(e),
            },
            None => {}
        };

        self.f.poll()
    }
}

pub struct ProtocolChainHandler<S, F>
where
    S: Middleware<ChildContext<ClientEvent, Event>> + Sync + Send,
    F: Protocol + Sync,
{
    s: Arc<S>,
    f: Arc<F>,
}

impl<S, F> Protocol for ProtocolChainHandler<S, F>
where
    S: Middleware<ChildContext<ClientEvent, Event>> + Sync + Send + 'static,
    F: Protocol + Sync + Send + 'static,
    <F as Protocol>::Future: Send + 'static,
{
    type Future = ProtocolMiddlewareChainFuture<S::Future>;
    fn execute(&self, req: ChildContext<ClientEvent, Event>) -> Self::Future {
        let (n, sx, rx) = Next::<ChildContext<ClientEvent, Event>>::new();
        let f = self.f.clone();

        let fut = rx
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
            .and_then(move |req| f.execute(req));

        let fut2 = self.s.call(req, n);
        ProtocolMiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }

    fn check(&self, req: &BorrowedContext<ClientEvent, Event>) -> bool {
        self.f.check(req)
    }
}
