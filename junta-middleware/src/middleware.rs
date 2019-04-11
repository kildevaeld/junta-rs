use futures::prelude::*;
use futures::sync::oneshot::{channel, Receiver, Sender};
use junta::prelude::*;
//use junta_service::*;
use std::error::Error;
use std::fmt;

pub struct Next<I> {
    ret: Receiver<JuntaResult<()>>,
    send: Sender<I>,
}

impl<I> Next<I> {
    pub fn new() -> (Next<I>, Sender<JuntaResult<()>>, Receiver<I>) {
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

    pub fn execute(self, req: I) -> NextFuture {
        if self.send.send(req).is_err() {
            NextFuture { inner: None }
        } else {
            NextFuture {
                inner: Some(self.ret),
            }
        }
    }
}

pub struct NextFuture {
    pub(crate) inner: Option<Receiver<JuntaResult<()>>>,
}

impl NextFuture {
    pub fn new(chan: Receiver<JuntaResult<()>>) -> NextFuture {
        NextFuture { inner: Some(chan) }
    }
}

impl Future for NextFuture {
    type Item = ();
    type Error = JuntaError;

    fn poll(self: &mut Self) -> Poll<Self::Item, Self::Error> {
        match &mut self.inner {
            None => Err(JuntaError::new(JuntaErrorKind::Error(
                "next future error".to_string(),
            ))),
            Some(m) => match m.poll() {
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Ok(Async::Ready(Ok(res))) => Ok(Async::Ready(res)),
                Ok(Async::Ready(Err(e))) => Err(e),
                Err(e) => Err(JuntaErrorKind::Error(e.to_string()).into()),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChannelErr;

impl fmt::Display for ChannelErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Internal Server Error.")
    }
}

impl Error for ChannelErr {
    fn description(&self) -> &str {
        "Internal Server Error"
    }
}

pub trait Middleware<I>: Send + Sync {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn call(&self, req: I, next: Next<I>) -> Self::Future;
}

pub trait IntoMiddleware<I> {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    type Middleware: Middleware<I, Future = Self::Future>;
    fn into_middleware(self) -> Self::Middleware;
}

impl<T, I> IntoMiddleware<I> for T
where
    T: Middleware<I>,
{
    type Future = T::Future;
    type Middleware = T;
    fn into_middleware(self) -> Self::Middleware {
        self
    }
}

impl<T, I> Middleware<I> for std::sync::Arc<T>
where
    T: Middleware<I>,
{
    //type Item = T::Item;
    type Future = T::Future;
    fn call(&self, req: I, next: Next<I>) -> Self::Future {
        self.as_ref().call(req, next)
    }
}

pub struct MiddlewareFn<F> {
    inner: F,
    // _u: std::marker::PhantomData<U>,
    // _i: std::marker::PhantomData<I>,
}

impl<F: Send + Sync, U, I> Middleware<I> for MiddlewareFn<F>
where
    F: Fn(I, Next<I>) -> U,
    U: IntoFuture<Item = (), Error = JuntaError>,
    <U as IntoFuture>::Future: Send + 'static,
{
    type Future = U::Future;
    // type Item = Context<ClientEvent>;

    fn call(&self, req: I, next: Next<I>) -> Self::Future {
        (self.inner)(req, next).into_future()
    }
}

pub fn middleware_fn<F, U, I>(f: F) -> impl Middleware<I, Future = U::Future>
//MiddlewareFn<F>
where
    F: (Fn(I, Next<I>) -> U) + Send + Sync,
    U: IntoFuture<Item = (), Error = JuntaError> + Send + 'static,
    <U as IntoFuture>::Future: Send + 'static,
{
    MiddlewareFn {
        inner: f,
        // _u: std::marker::PhantomData,
        // _i: std::marker::PhantomData,
    }
}
