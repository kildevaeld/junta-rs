use super::middleware::{IntoMiddleware, Middleware, Next};
use futures::prelude::*;
use futures::sync::oneshot::{Receiver, Sender};
use std::sync::Arc;

use junta::prelude::*;
// use junta_service::*;

#[derive(Clone)]
pub struct MiddlewareChain<S, F> {
    s: Arc<S>,
    f: Arc<F>,
}

impl<S: Middleware<I>, F, I: Send + 'static> Middleware<I> for MiddlewareChain<S, F>
where
    S: Middleware<I> + Send + Sync + 'static,
    F: Middleware<I> + Send + Sync + 'static,
    <F as Middleware<I>>::Future: Sync,
    //<S as Middleware>::Item: Send,
{
    //type Item = S::Item;
    type Future = MiddlewareChainFuture<S::Future>;
    fn call(&self, req: I, next: Next<I>) -> Self::Future {
        let (n, sx, rx) = Next::new();
        let f = self.f.clone();
        let fut = rx
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
            .and_then(move |req| f.call(req, next));

        let fut2 = self.s.call(req, n);
        MiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }
}

pub trait MiddlewareChainable<I>: Sized {
    fn stack<M: IntoMiddleware<I>>(self, other: M) -> MiddlewareChain<Self, M::Middleware>;
}

impl<T, I> MiddlewareChainable<I> for T
where
    T: Middleware<I>,
{
    fn stack<M: IntoMiddleware<I>>(self, other: M) -> MiddlewareChain<Self, M::Middleware> {
        MiddlewareChain {
            s: Arc::new(self),
            f: Arc::new(other.into_middleware()),
        }
    }
}

pub trait Handable: Sized + Middleware<Context<ClientEvent>> {
    fn then<M: IntoHandler>(self, handler: M) -> ChainHandler<Self, M::Handler> {
        ChainHandler {
            s: Arc::new(self),
            f: Arc::new(handler.into_handler()),
        }
    }
    //fn to_handler(self) -> ChainHandler<Self, NotFoundHandler>;
}

impl<T> Handable for T where T: Middleware<Context<ClientEvent>> {}

pub struct MiddlewareChainFuture<F: Future> {
    s: Option<Box<Future<Item = (), Error = JuntaError> + Send>>,
    f: F,
    sx: Option<Sender<JuntaResult<()>>>,
}

impl<F: Future> MiddlewareChainFuture<F> {
    pub fn new(
        s: Box<Future<Item = (), Error = JuntaError> + Send>,
        f: F,
        sx: Sender<JuntaResult<()>>,
    ) -> MiddlewareChainFuture<F> {
        MiddlewareChainFuture {
            s: Some(s),
            f,
            sx: Some(sx),
        }
    }
}

impl<F: Future> Future for MiddlewareChainFuture<F>
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

pub struct ChainHandler<S, F>
where
    S: Middleware<Context<ClientEvent>> + Sync + Send,
    F: Handler + Sync,
{
    s: Arc<S>,
    f: Arc<F>,
}

impl<S, F> Handler for ChainHandler<S, F>
where
    S: Middleware<Context<ClientEvent>> + Sync + Send + 'static,
    F: Handler + Sync + Send + 'static,
    <F as Handler>::Future: Send + 'static,
{
    type Future = MiddlewareChainFuture<S::Future>;
    fn handle(&self, req: Context<ClientEvent>) -> Self::Future {
        let (n, sx, rx) = Next::<Context<ClientEvent>>::new();
        let f = self.f.clone();

        let fut = rx
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
            .and_then(move |req| f.handle(req));

        let fut2 = self.s.call(req, n);
        MiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }

    // fn check(&self, req: &Context<ClientEvent>) -> bool {
    //     self.f.check(req)
    // }
}

