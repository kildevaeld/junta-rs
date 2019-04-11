use super::client::ClientEvent;
use super::context::Context;
use super::error::{JuntaError, JuntaErrorKind};
use super::handler::Handler;
use super::utils::*;

pub struct HandlerChain<S1, S2> {
    s1: S1,
    s2: S2,
}

impl<S1, S2> HandlerChain<S1, S2> {
    pub fn new(s1: S1, s2: S2) -> HandlerChain<S1, S2> {
        HandlerChain { s1, s2 }
    }
}

impl<S1, S2> Handler for HandlerChain<S1, S2>
where
    S1: Handler,
    S2: Handler,
{
    type Future = OneOfTreeFuture<
        (),
        JuntaError,
        S1::Future,
        S2::Future,
        futures::future::FutureResult<(), JuntaError>,
    >;

    fn handle(&self, ctx: Context<ClientEvent>) -> Self::Future {
        let fut = if self.s1.check(&ctx) {
            OneOfTree::Future1(self.s1.handle(ctx))
        } else if self.s2.check(&ctx) {
            OneOfTree::Future2(self.s2.handle(ctx))
        } else {
            OneOfTree::Future3(futures::future::err(
                JuntaErrorKind::Error("invalid request".to_string()).into(),
            ))
        };
        OneOfTreeFuture::new(fut)
    }

    #[inline]
    fn check(&self, ctx: &Context<ClientEvent>) -> bool {
        self.s1.check(ctx) || self.s2.check(ctx)
    }
}

pub trait HandlerExt: Handler + Sized {
    fn or<S: Handler>(self, service: S) -> HandlerChain<Self, S> {
        HandlerChain::new(self, service)
    }
}

impl<T> HandlerExt for T where T: Handler {}
