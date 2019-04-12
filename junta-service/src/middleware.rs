use super::error::ServiceError;
use super::service::*;
use futures::prelude::*;
use futures::sync::oneshot::{channel, Receiver, Sender};
use std::sync::Arc;

pub struct Next<I, O, E> {
    ret: Receiver<Result<O, E>>,
    send: Sender<I>,
}

impl<I, O, E> Next<I, O, E> {
    pub fn new() -> (Next<I, O, E>, Sender<Result<O, E>>, Receiver<I>) {
        let (sx1, rx1) = channel();
        let (sx2, rx2) = channel();
        (
            Next {
                ret: rx1,
                send: sx2,
            },
            sx1,
            rx2,
        )
    }

    pub fn call(self, req: I) -> NextFuture<O, E> {
        if self.send.send(req).is_err() {
            NextFuture { inner: None }
        } else {
            NextFuture {
                inner: Some(self.ret),
            }
        }
    }
}

pub struct NextFuture<O, E> {
    pub(crate) inner: Option<Receiver<Result<O, E>>>,
}

impl<O, E> NextFuture<O, E> {
    pub fn new(chan: Receiver<Result<O, E>>) -> NextFuture<O, E> {
        NextFuture { inner: Some(chan) }
    }
}

impl<O, E: From<ServiceError>> Future for NextFuture<O, E> {
    type Item = O;
    type Error = E;

    fn poll(self: &mut Self) -> Poll<Self::Item, Self::Error> {
        match &mut self.inner {
            None => Err(E::from(ServiceError::NullFuture)),
            Some(m) => match m.poll() {
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Ok(Async::Ready(Ok(res))) => Ok(Async::Ready(res)),
                Ok(Async::Ready(Err(e))) => Err(e),
                Err(_) => Err(E::from(ServiceError::ReceiverClosed)),
            },
        }
    }
}

pub trait Middleware {
    type Input;

    type Output;

    type Error;

    type Future: Future<Item = Self::Output, Error = Self::Error>;

    fn call(
        &self,
        input: Self::Input,
        next: Next<Self::Input, Self::Output, Self::Error>,
    ) -> Self::Future;
}

pub trait IntoMiddleware {
    type Input;
    type Output;
    type Error;

    type Future: Future<Item = Self::Output, Error = Self::Error> + Send + 'static;
    type Middleware: Middleware<
        Input = Self::Input,
        Output = Self::Output,
        Error = Self::Error,
        Future = Self::Future,
    >;
    fn into_middleware(self) -> Self::Middleware;
}

impl<T> IntoMiddleware for T
where
    T: Middleware,
    <T as Middleware>::Future: Send + 'static,
{
    type Input = T::Input;
    type Output = T::Output;
    type Error = T::Error;
    type Future = T::Future;
    type Middleware = T;

    fn into_middleware(self) -> Self::Middleware {
        self
    }
}

pub trait ThenService: Middleware + Sized + Send + Sync {
    fn then<S: IntoService<Input = Self::Input>>(
        self,
        service: S,
    ) -> ThenServiceHandler<Self, S::Service>
    where
        S::Service: Sync;
}

impl<T> ThenService for T
where
    T: Middleware + Sync + Send,
{
    fn then<S: IntoService<Input = Self::Input>>(
        self,
        service: S,
    ) -> ThenServiceHandler<Self, S::Service>
    where
        S::Service: Sync,
    {
        ThenServiceHandler {
            s: self,
            f: Arc::new(service.into_service()),
        }
    }
}

pub struct ThenServiceHandler<S, F>
where
    S: Middleware + Sync + Send,
    F: Service + Sync,
{
    s: S,
    f: Arc<F>,
}

impl<S, F> Service for ThenServiceHandler<S, F>
where
    S: Middleware + Sync + Send + 'static,
    <S as Middleware>::Input: Send,
    <S as Middleware>::Output: Send,
    <S as Middleware>::Error: From<ServiceError> + Send,
    F: Service<
            Input = <S as Middleware>::Input,
            Output = <S as Middleware>::Output,
            Error = <S as Middleware>::Error,
        > + Sync
        + Send
        + 'static,
    <F as Service>::Future: Send + 'static,
    <S as Middleware>::Future: Send + 'static,
{
    type Input = S::Input;
    type Output = S::Output;
    type Error = S::Error;
    type Future = MiddlewareChainFuture<S::Future, S::Input, S::Output, S::Error>;
    fn call(&self, req: S::Input) -> Self::Future {
        let (n, sx, rx) = Next::<S::Input, S::Output, S::Error>::new();
        let f = self.f.clone();

        let fut = rx
            .map_err(|_| S::Error::from(ServiceError::ReceiverClosed))
            .and_then(move |req| f.call(req));

        let fut2 = self.s.call(req, n);
        MiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }

    // fn check(&self, req: &Context<ClientEvent>) -> bool {
    //     self.f.check(req)
    // }
}

pub struct MiddlewareChainFuture<F: Future, I, O, E> {
    s: Option<Box<Future<Item = O, Error = E> + Send>>,
    f: F,
    sx: Option<Sender<Result<O, E>>>,
    _i: std::marker::PhantomData<I>,
}

impl<F: Future, I, O, E> MiddlewareChainFuture<F, I, O, E> {
    pub fn new(
        s: Box<Future<Item = O, Error = E> + Send>,
        f: F,
        sx: Sender<Result<O, E>>,
    ) -> MiddlewareChainFuture<F, I, O, E> {
        MiddlewareChainFuture {
            s: Some(s),
            f,
            sx: Some(sx),
            _i: std::marker::PhantomData,
        }
    }
}

impl<F: Future, I, O, E> Future for MiddlewareChainFuture<F, I, O, E>
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

use std::marker::PhantomData;

pub struct MiddlewareFn<F, I, O, E> {
    inner: F,
    _i: PhantomData<I>,
    _o: PhantomData<O>,
    _e: PhantomData<E>,
}

impl<F, I, O, E, U> Middleware for MiddlewareFn<F, I, O, E>
where
    F: Fn(I, Next<I, O, E>) -> U,
    U: IntoFuture<Item = O, Error = E>,
    <U as IntoFuture>::Future: Send + 'static,
{
    type Input = I;
    type Output = O;
    type Error = E;
    type Future = U::Future;

    fn call(&self, input: I, next: Next<Self::Input, Self::Output, Self::Error>) -> Self::Future {
        (self.inner)(input, next).into_future()
    }
}

pub fn middleware_fn<F, I, U>(
    service: F,
) -> impl Middleware<Input = I, Output = U::Item, Error = U::Error, Future = U::Future>
where
    F: Fn(I, Next<I, U::Item, U::Error>) -> U,
    U: IntoFuture,
    <U as IntoFuture>::Future: Send + 'static,
{
    MiddlewareFn {
        inner: service,
        _i: PhantomData,
        _o: PhantomData,
        _e: PhantomData,
    }
}
