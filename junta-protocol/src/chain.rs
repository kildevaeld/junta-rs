use super::event::*;
use super::protocol::{Protocol, ProtocolService};
use future_ext::*;
use junta::prelude::*;
use junta_service::prelude::*;
pub struct ProtocolChain<S1, S2> {
    s1: S1,
    s2: S2,
}

impl<S1, S2> ProtocolChain<S1, S2> {
    pub fn new(s1: S1, s2: S2) -> ProtocolChain<S1, S2> {
        ProtocolChain { s1, s2 }
    }
}

impl<S1, S2> Protocol for ProtocolChain<S1, S2>
where
    S1: Protocol,
    S2: Protocol,
{
    type Future = OneOfTreeFuture<
        (),
        JuntaError,
        S1::Future,
        S2::Future,
        futures::future::FutureResult<(), JuntaError>,
    >;

    fn execute(&self, ctx: ChildContext<ClientEvent, Event>) -> Self::Future {
        let borrow = BorrowedContext::new(ctx.parent(), ctx.message());
        let fut = if self.s1.check(&borrow) {
            OneOfTree::First(self.s1.execute(ctx))
        } else if self.s2.check(&borrow) {
            OneOfTree::Second(self.s2.execute(ctx))
        } else {
            OneOfTree::Third(futures::future::err(
                JuntaErrorKind::Unknown("invalid request".to_string()).into(),
            ))
        };
        OneOfTreeFuture::new(fut)
    }

    #[inline]
    fn check(&self, ctx: &BorrowedContext<ClientEvent, Event>) -> bool {
        self.s1.check(ctx) || self.s2.check(ctx)
    }
}

impl<S1, S2> IntoService for ProtocolChain<S1, S2>
where
    S1: Protocol + 'static,
    S2: Protocol + 'static,
{
    type Input = Context<ClientEvent>;
    type Output = ();
    type Error = JuntaError;
    type Future = <ProtocolService<Self> as Service>::Future; //Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    type Service = ProtocolService<Self>;
    fn into_service(self) -> Self::Service {
        ProtocolService::new(self)
    }
}
