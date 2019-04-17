use super::pipe_chain::Pipe;
use super::service::Service;
use super::service_chain::ServiceChain;
use std::sync::Arc;

pub trait ServiceExt: Service + Sized {
    fn or<S: Service<Input = Self::Input, Output = Self::Output, Error = Self::Error>>(
        self,
        service: S,
    ) -> ServiceChain<Self, S> {
        ServiceChain::new(self, service)
    }

    fn pipe<S: Service<Error = Self::Error>>(self, service: S) -> Pipe<Self, S> {
        Pipe {
            s1: self,
            s2: Arc::new(service),
        }
    }
}

impl<T> ServiceExt for T where T: Service {}
