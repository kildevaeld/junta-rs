use super::error::*;
use future_ext::*;
use futures::prelude::*;

pub trait Service {
    type Input;
    type Output;
    type Error;

    type Future: Future<Item = Self::Output, Error = Self::Error> + Send + 'static;

    fn call(&self, input: Self::Input) -> Self::Future;

    fn should_call(&self, _intput: &Self::Input) -> bool {
        true
    }
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

impl<T> IntoService for T
where
    T: Service,
{
    type Input = T::Input;
    type Output = T::Output;
    type Error = T::Error;
    type Future = T::Future;
    type Service = T;
    fn into_service(self) -> Self::Service {
        self
    }
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

pub fn service_fn<F, I, U>(service: F) -> ServiceFn<F, I>
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

pub struct CheckService<F, S> {
    check: F,
    service: S,
}

impl<F, S> Service for CheckService<F, S>
where
    S: Service,
    F: Fn(&S::Input) -> bool,
    <S as Service>::Error: From<ServiceError> + Send + 'static,
    <S as Service>::Output: Send + 'static,
{
    type Input = S::Input;
    type Output = S::Output;
    type Error = S::Error;
    type Future = OneOfTwoFuture<
        Self::Output,
        Self::Error,
        S::Future,
        futures::future::FutureResult<Self::Output, Self::Error>,
    >;

    fn call(&self, input: Self::Input) -> Self::Future {
        let fut = if (self.check)(&input) {
            OneOfTwo::First(self.service.call(input))
        } else {
            OneOfTwo::Second(futures::future::err(Self::Error::from(
                ServiceError::InvalidRequest,
            )))
        };
        OneOfTwoFuture::new(fut)
    }

    fn should_call(&self, input: &Self::Input) -> bool {
        (self.check)(input)
    }
}

impl<F, S> CheckService<F, S> {
    pub fn new(check: F, service: S) -> CheckService<F, S> {
        CheckService { check, service }
    }
}

pub fn check_fn<F, S>(check: F, service: S) -> CheckService<F, S> {
    CheckService::new(check, service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    #[test]
    fn test_service() {
        let service = service_fn(|_ctx| Result::<i32, String>::Ok(200));
        let fut = service.call("Hello, World");
        tokio::run(fut.map(|_o| assert_eq!(0, 200)).map_err(|e| panic!(e)));
    }
}
