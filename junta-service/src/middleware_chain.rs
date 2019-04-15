use super::error::ServiceError;
use super::middleware::{IntoMiddleware, Middleware, Next};
use super::service::*;
use futures::prelude::*;
use futures::sync::oneshot::Sender;
use std::sync::Arc;

#[derive(Clone)]
pub struct MiddlewareChain<S1, S2> {
    s1: Arc<S1>,
    s2: Arc<S2>,
}

impl<S1: Send + Sync + 'static, S2: Send + Sync + 'static> Middleware for MiddlewareChain<S1, S2>
where
    S1: Middleware,
    <S1 as Middleware>::Input: Send + 'static,
    <S1 as Middleware>::Output: Send + 'static,
    <S1 as Middleware>::Error: Send + 'static + From<ServiceError>,
    S2: Middleware<
        Input = <S1 as Middleware>::Input,
        Output = <S1 as Middleware>::Output,
        Error = <S1 as Middleware>::Error,
    >,
    <S2 as Middleware>::Future: Send + 'static, //<S as Middleware>::Item: Send
{
    //type Item = S::Item;
    type Input = S1::Input;
    type Output = S1::Output;
    type Error = S1::Error;
    type Future = MiddlewareChainFuture<S1::Future, Self::Output, Self::Error>;
    fn call(
        &self,
        req: Self::Input,
        next: Next<Self::Input, Self::Output, Self::Error>,
    ) -> Self::Future {
        let (n, sx, rx) = Next::new();
        let f = self.s2.clone();
        let fut = rx
            .map_err(|_| Self::Error::from(ServiceError::ReceiverClosed))
            .and_then(move |req| f.call(req, next));

        let fut2 = self.s1.call(req, n);
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
            s1: Arc::new(self),
            s2: Arc::new(other.into_middleware()),
        }
    }
}

pub struct MiddlewareChainFuture<F: Future, O, E> {
    s: Option<Box<Future<Item = O, Error = E> + Send>>,
    f: F,
    sx: Option<Sender<Result<O, E>>>,
}

impl<F: Future, O, E> MiddlewareChainFuture<F, O, E> {
    pub fn new(
        s: Box<Future<Item = O, Error = E> + Send>,
        f: F,
        sx: Sender<Result<O, E>>,
    ) -> MiddlewareChainFuture<F, O, E> {
        MiddlewareChainFuture {
            s: Some(s),
            f,
            sx: Some(sx),
        }
    }
}

impl<F: Future, O, E> Future for MiddlewareChainFuture<F, O, E>
where
    F: Future<Item = O, Error = E>,
{
    type Item = O;
    type Error = E;

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
    <S as Middleware>::Error: Send + 'static + From<ServiceError>,
    <S as Middleware>::Input: Send + 'static,
    <S as Middleware>::Output: Send + 'static,
    F: Service<
            Input = <S as Middleware>::Input,
            Output = <S as Middleware>::Output,
            Error = <S as Middleware>::Error,
        > + Sync
        + Send
        + 'static,
    <S as Middleware>::Future: Send + 'static,
    <F as Service>::Future: Send + 'static,
{
    type Input = S::Input;
    type Output = S::Output;
    type Error = S::Error;
    type Future = MiddlewareChainFuture<S::Future, Self::Output, Self::Error>;

    fn call(&self, req: Self::Input) -> Self::Future {
        let (n, sx, rx) = Next::new();
        let f = self.f.clone();

        let fut = rx
            .map_err(|_| Self::Error::from(ServiceError::ReceiverClosed))
            .and_then(move |req| f.call(req));

        let fut2 = self.s.call(req, n);
        MiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }

    fn should_call(&self, req: &Self::Input) -> bool {
        self.f.should_call(req)
    }
}
