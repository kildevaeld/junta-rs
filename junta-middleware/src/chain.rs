use super::middleware::{IntoMiddleware, Middleware, Next};
use futures::prelude::*;
use futures::sync::oneshot::{Receiver, Sender};
use std::sync::Arc;

use junta::prelude::*;
use junta_service::*;

#[derive(Clone)]
pub struct MiddlewareChain<S, F> {
    s: Arc<S>,
    f: Arc<F>,
}

impl<S, F> Middleware for MiddlewareChain<S, F>
where
    S: Middleware + Send + Sync + 'static,
    F: Middleware + Send + Sync + 'static,
    <F as Middleware>::Future: Sync,
{
    type Future = MiddlewareChainFuture<S::Future>;
    fn execute(&self, req: Context, next: Next) -> Self::Future {
        let (n, sx, rx) = Next::new();
        let f = self.f.clone();
        let fut = rx
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
            .and_then(move |req| f.execute(req, next));

        let fut2 = self.s.execute(req, n);
        MiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }
}

pub trait MiddlewareChainable: Sized {
    fn stack<M: IntoMiddleware>(self, other: M) -> MiddlewareChain<Self, M::Middleware>;
}

impl<T> MiddlewareChainable for T
where
    T: Middleware,
{
    fn stack<M: IntoMiddleware>(self, other: M) -> MiddlewareChain<Self, M::Middleware> {
        MiddlewareChain {
            s: Arc::new(self),
            f: Arc::new(other.into_middleware()),
        }
    }
}

pub trait Handable: Sized + Middleware + Send + Sync {
    fn then<M: IntoService>(self, handler: M) -> ChainHandler<Self, M::Service>
    where
        <M as IntoService>::Service: Sync,
    {
        ChainHandler {
            s: Arc::new(self),
            f: Arc::new(handler.into_service()),
        }
    }
    //fn to_handler(self) -> ChainHandler<Self, NotFoundHandler>;
}

impl<T> Handable for T
where
    T: Middleware + Send + Sync,
{
    // fn then<M: IntoService>(self, handler: M) -> ChainHandler<Self, M::Service>
    // where
    //     <M as IntoService>::Service: Sync;
    // {
    //     ChainHandler {
    //         s: Arc::new(self),
    //         f: Arc::new(handler.into_service()),
    //     }
    // }

    // fn to_handler(self) -> ChainHandler<Self, NotFoundHandler>
    // where
    //     <Self as Middleware>::Future: Send,
    // {
    //     ChainHandler {
    //         s: Arc::new(self),
    //         f: Arc::new(NotFoundHandler),
    //     }
    // }
}

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
    S: Middleware + Sync + Send,
    F: Service + Sync,
{
    s: Arc<S>,
    f: Arc<F>,
}

impl<S, F> Service for ChainHandler<S, F>
where
    S: Middleware + Sync + Send + 'static,
    F: Service + Sync + Send + 'static,
    <F as Service>::Future: Send + 'static,
{
    type Future = MiddlewareChainFuture<S::Future>;
    fn execute(&self, req: Context) -> Self::Future {
        let (n, sx, rx) = Next::new();
        let f = self.f.clone();

        let fut = rx
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
            .and_then(move |req| f.execute(req));

        let fut2 = self.s.execute(req, n);
        MiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }

    fn check(&self, req: &Context) -> bool {
        self.f.check(req)
    }
}

impl<S, F> IntoHandler for ChainHandler<S, F>
where
    S: Middleware + Sync + Send + 'static,
    F: Service + Sync + Send + 'static,
    <F as Service>::Future: Send + 'static,
{
    type Future = <ServiceHandler<Self> as Handler>::Future;
    type Handler = ServiceHandler<Self>;
    fn into_handler(self) -> Self::Handler {
        ServiceHandler::new(self)
    }
}
