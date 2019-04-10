use futures::prelude::*;
use futures::sync::oneshot::{channel, Receiver, Sender};
use junta::prelude::*;
use junta_service::*;
use std::error::Error;
use std::fmt;

pub struct Next {
    ret: Receiver<JuntaResult<()>>,
    send: Sender<Context>,
}

impl Next {
    pub(crate) fn new() -> (Next, Sender<JuntaResult<()>>, Receiver<Context>) {
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

    pub fn execute(self, req: Context) -> NextFuture {
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

pub trait Middleware {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn execute(&self, req: Context, next: Next) -> Self::Future;
}

pub trait IntoMiddleware {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    type Middleware: Middleware<Future = Self::Future>;
    fn into_middleware(self) -> Self::Middleware;
}

impl<T> IntoMiddleware for T
where
    T: Middleware,
{
    type Future = T::Future;
    type Middleware = T;
    fn into_middleware(self) -> Self::Middleware {
        self
    }
}

impl<T> Middleware for std::sync::Arc<T>
where
    T: Middleware,
{
    type Future = T::Future;
    fn execute(&self, req: Context, next: Next) -> Self::Future {
        self.as_ref().execute(req, next)
    }
}

pub struct MiddlewareFn<F> {
    inner: F,
}

impl<F, U> Middleware for MiddlewareFn<F>
where
    F: (Fn(Context, Next) -> U) + Send,
    U: IntoFuture<Item = (), Error = JuntaError>,
    <U as IntoFuture>::Future: Send + 'static,
{
    type Future = U::Future;

    fn execute(&self, req: Context, next: Next) -> Self::Future {
        (self.inner)(req, next).into_future()
    }
}

pub fn middleware_fn<F, U>(f: F) -> MiddlewareFn<F>
where
    F: (Fn(Context, Next) -> U) + Send,
    U: IntoFuture<Item = (), Error = JuntaError> + Send + 'static,
{
    MiddlewareFn { inner: f }
}
