use super::event::Event;
use super::protocol::*;
use futures::prelude::*;
use futures::sync::oneshot::Sender;
use junta::prelude::*;
use junta_service::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProtocolMiddlewareChain<S1, S2> {
    s1: Arc<S1>,
    s2: Arc<S2>,
}

impl<S1: Send + Sync + 'static, S2: Send + Sync + 'static> Middleware
    for ProtocolMiddlewareChain<S1, S2>
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
    <S2 as Middleware>::Future: Send + 'static,
{
    //type Item = S::Item;
    type Input = S1::Input;
    type Output = S1::Output;
    type Error = S1::Error;
    type Future = ProtocolMiddlewareChainFuture<S1::Future, Self::Output, Self::Error>;
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
        ProtocolMiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }
}

pub trait ProtocolMiddlewareChainable: Sized {
    fn stack<M: IntoMiddleware>(self, other: M) -> ProtocolMiddlewareChain<Self, M::Middleware>;
}

impl<T> ProtocolMiddlewareChainable for T
where
    T: Middleware,
{
    fn stack<M: IntoMiddleware>(self, other: M) -> ProtocolMiddlewareChain<Self, M::Middleware> {
        ProtocolMiddlewareChain {
            s1: Arc::new(self),
            s2: Arc::new(other.into_middleware()),
        }
    }
}

pub trait ProtocolHandable: Sized + Middleware + Send + Sync {
    fn then_protocol<M: IntoProtocol>(self, handler: M) -> ProtocolChainHandler<Self, M::Protocol>
    where
        <M as IntoProtocol>::Protocol: Send + Sync,
    {
        ProtocolChainHandler {
            s: Arc::new(self),
            f: Arc::new(handler.into_protocol()),
        }
    }
}

impl<T> ProtocolHandable for T where
    T: Middleware<Input = ChildContext<ClientEvent, Event>, Output = (), Error = JuntaError>
        + Sync
        + Send
{
}

pub struct ProtocolMiddlewareChainFuture<F: Future, O, E> {
    s: Option<Box<Future<Item = O, Error = E> + Send>>,
    f: F,
    sx: Option<Sender<Result<O, E>>>,
}

impl<F: Future, O, E> ProtocolMiddlewareChainFuture<F, O, E> {
    pub fn new(
        s: Box<Future<Item = O, Error = E> + Send>,
        f: F,
        sx: Sender<Result<O, E>>,
    ) -> ProtocolMiddlewareChainFuture<F, O, E> {
        ProtocolMiddlewareChainFuture {
            s: Some(s),
            f,
            sx: Some(sx),
        }
    }
}

impl<F: Future, O, E> Future for ProtocolMiddlewareChainFuture<F, O, E>
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

pub struct ProtocolChainHandler<S, F>
where
    S: Middleware + Sync + Send,
    F: Protocol + Sync,
{
    s: Arc<S>,
    f: Arc<F>,
}

impl<S, F> Protocol for ProtocolChainHandler<S, F>
where
    S: Middleware<Input = ChildContext<ClientEvent, Event>, Output = (), Error = JuntaError>
        + Sync
        + Send
        + 'static,

    F: Protocol + Sync + Send + 'static,
    <S as Middleware>::Future: Send + 'static,
    <F as Protocol>::Future: Send + 'static,
{
    type Future = ProtocolMiddlewareChainFuture<S::Future, (), JuntaError>;

    fn execute(&self, req: ChildContext<ClientEvent, Event>) -> Self::Future {
        let (n, sx, rx) = Next::new();
        let f = self.f.clone();

        let fut = rx
            .map_err(|_| JuntaError::from(ServiceError::ReceiverClosed))
            .and_then(move |req| f.execute(req));

        let fut2 = self.s.call(req, n);
        ProtocolMiddlewareChainFuture::new(Box::new(fut), fut2, sx)
    }

    fn check(&self, ctx: &BorrowedContext<ClientEvent, Event>) -> bool {
        self.f.check(ctx)
    }
}
