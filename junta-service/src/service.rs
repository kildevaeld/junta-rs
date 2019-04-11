use futures::prelude::*;

pub trait Service {
    type Input;
    type Output;
    type Error;

    type Future: Future<Item = Self::Output, Error = Self::Error> + Send + 'static;

    fn call(&self, input: Self::Input) -> Self::Future;
}

pub trait IntoService {
    type Input;
    type Output;
    type Error;
    type Future: Future<Item = Self::Output, Error = Self::Error> + Send + 'static;
    type Service: Service<
        Input = Self::Input,
        Output = Self::Output,
        Error = Self::Error,
        Future = Self::Future,
    >;

    fn into_service(self) -> Self::Service;
}

use std::marker::PhantomData;

pub struct ServiceFn<F, I> {
    inner: F,
    _i: PhantomData<I>,
}

impl<F, I, U> Service for ServiceFn<F, I>
where
    F: Fn(I) -> U,
    U: IntoFuture,
    <U as IntoFuture>::Future: Send + 'static,
{
    type Input = I;
    type Output = U::Item;
    type Error = U::Error;
    type Future = U::Future;

    fn call(&self, input: I) -> Self::Future {
        (self.inner)(input).into_future()
    }
}

pub fn service_fn<F, I, U>(
    service: F,
) -> impl Service<Input = I, Output = U::Item, Error = U::Error, Future = U::Future>
where
    F: Fn(I) -> U,
    U: IntoFuture,
    <U as IntoFuture>::Future: Send + 'static,
{
    ServiceFn {
        inner: service,
        _i: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    #[test]
    fn test_service() {
        let service = service_fn(|ctx| Result::<i32, String>::Ok(200));
        let fut = service.call("Hello, World");
        tokio::run(fut.map(|o| assert_eq!(0, 200)).map_err(|e| panic!(e)));
    }
}
