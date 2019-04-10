use super::client::{Client, ClientEvent};
use super::error::JuntaError;
use futures::prelude::*;
use std::sync::Arc;


pub trait Handler: Send + Sync {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn handle(self: &Self, client: &Arc<Client>, event: ClientEvent) -> Self::Future;
}

pub trait IntoHandler {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    type Handler: Handler<Future = Self::Future> + 'static;
    fn into_handler(self) -> Self::Handler;
}

impl<T> IntoHandler for T
where
    T: Handler + 'static,
{
    type Future = T::Future;
    type Handler = T;
    fn into_handler(self) -> Self::Handler {
        self
    }
}
