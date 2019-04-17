use super::pipe_chain::Pipe;
use super::service::{IntoService, Service};
use super::service_chain::ServiceChain;
use std::sync::Arc;

pub trait ServiceExt: Service + Sized {
    fn or<S: IntoService<Input = Self::Input, Output = Self::Output, Error = Self::Error>>(
        self,
        service: S,
    ) -> ServiceChain<Self, S::Service> {
        ServiceChain::new(self, service.into_service())
    }

    fn pipe<S: IntoService<Error = Self::Error>>(self, service: S) -> Pipe<Self, S::Service> {
        Pipe {
            s1: self,
            s2: Arc::new(service.into_service()),
        }
    }
}

impl<T> ServiceExt for T where T: Service {}

#[cfg(test)]
mod tests {
    use super::super::service::*;
    use super::*;
    use futures::prelude::*;

    #[test]
    fn test_service_pipe() {
        let s = service_fn(|input: &str| Ok::<_, ()>(2000i32))
            .pipe(service_fn(|input: i32| Ok(input + 2)));

        tokio::run(s.call("Hello, World").map(|out| assert_eq!(out, 2002)))
    }

}
