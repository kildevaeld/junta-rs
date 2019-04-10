use super::event::*;
use super::protocol::{Protocol, ProtocolService};
use junta::prelude::*;
use junta_service::*;

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

    fn execute(&self, ctx: Context, event: Event) -> Self::Future {
        let fut = if self.s1.check(&ctx, &event) {
            OneOfTree::Future1(self.s1.execute(ctx, event))
        } else if self.s2.check(&ctx, &event) {
            OneOfTree::Future2(self.s2.execute(ctx, event))
        } else {
            OneOfTree::Future3(futures::future::err(
                JuntaErrorKind::Error("invalid request".to_string()).into(),
            ))
        };
        OneOfTreeFuture::new(fut)
    }

    #[inline]
    fn check(&self, ctx: &Context, event: &Event) -> bool {
        self.s1.check(ctx, event) || self.s2.check(ctx, event)
    }
}

impl<S1, S2> IntoService for ProtocolChain<S1, S2>
where
    S1: Protocol,
    S2: Protocol,
{
    type Future = <ProtocolService<Self> as Service>::Future; //Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    type Service = ProtocolService<Self>;
    fn into_service(self) -> Self::Service {
        ProtocolService::new(self)
    }
}
