use super::error::ServiceError;
use super::service::Service;
use super::utils::*;

pub struct ServiceChain<S1, S2> {
    s1: S1,
    s2: S2,
}

impl<S1, S2> ServiceChain<S1, S2> {
    pub fn new(s1: S1, s2: S2) -> ServiceChain<S1, S2> {
        ServiceChain { s1, s2 }
    }
}

impl<S1, S2> Service for ServiceChain<S1, S2>
where
    S1: Service,
    <S1 as Service>::Output: Send + 'static,
    <S1 as Service>::Error: Send + 'static + From<ServiceError>,
    S2: Service<
        Input = <S1 as Service>::Input,
        Output = <S1 as Service>::Output,
        Error = <S1 as Service>::Error,
    >,
{
    type Input = S1::Input;
    type Output = S1::Output;
    type Error = S1::Error;

    type Future = OneOfTreeFuture<
        Self::Output,
        Self::Error,
        S1::Future,
        S2::Future,
        futures::future::FutureResult<Self::Output, Self::Error>,
    >;

    fn call(&self, ctx: Self::Input) -> Self::Future {
        let fut = if self.s1.should_call(&ctx) {
            OneOfTree::Future1(self.s1.call(ctx))
        } else if self.s2.should_call(&ctx) {
            OneOfTree::Future2(self.s2.call(ctx))
        } else {
            OneOfTree::Future3(futures::future::err(Self::Error::from(
                ServiceError::InvalidRequest,
            )))
        };
        OneOfTreeFuture::new(fut)
    }

    #[inline]
    fn should_call(&self, ctx: &Self::Input) -> bool {
        self.s1.should_call(ctx) || self.s2.should_call(ctx)
    }
}

pub trait ServiceExt: Service + Sized {
    fn or<S: Service<Input = Self::Input, Output = Self::Output, Error = Self::Error>>(
        self,
        service: S,
    ) -> ServiceChain<Self, S> {
        ServiceChain::new(self, service)
    }
}

impl<T> ServiceExt for T where T: Service {}
