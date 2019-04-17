use super::service::Service;
use futures::prelude::*;
use std::sync::Arc;

pub struct Pipe<S1, S2> {
    pub(crate) s1: S1,
    pub(crate) s2: Arc<S2>,
}

impl<S1, S2> Service for Pipe<S1, S2>
where
    S1: Service,
    S2: Service<Input = <S1 as Service>::Output, Error = <S1 as Service>::Error>
        + 'static
        + Send
        + Sync,
    <S1 as Service>::Error: Send + 'static,
    <S2 as Service>::Future: Send + 'static,
{
    type Input = S1::Input;
    type Error = S1::Error;
    type Output = S2::Output;
    type Future = Box<Future<Item = Self::Output, Error = Self::Error> + Send + 'static>;
    fn call(&self, input: Self::Input) -> Self::Future {
        let s2 = self.s2.clone();
        Box::new(self.s1.call(input).and_then(move |ret| s2.call(ret)))
    }
}
