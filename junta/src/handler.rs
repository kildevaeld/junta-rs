use super::client::{Client, ClientEvent};
use super::context::Context;
use super::error::JuntaError;
use futures::prelude::*;
use std::sync::Arc;

pub trait Handler: Send + Sync {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn handle(self: &Self, ctx: Context<ClientEvent>) -> Self::Future;
    fn check(&self, _ctx: &Context<ClientEvent>) -> bool {
        true
    }
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

pub struct HandleFn<F> {
    inner: F,
}

impl<F: Send + Sync, U> Handler for HandleFn<F>
where
    F: Fn(Context<ClientEvent>) -> U,
    U: IntoFuture<Item = (), Error = JuntaError>,
    <U as IntoFuture>::Future: Send + 'static,
{
    type Future = U::Future;
    fn handle(&self, ctx: Context<ClientEvent>) -> Self::Future {
        (self.inner)(ctx).into_future()
    }
}

pub fn handle_fn<F, U>(service: F) -> HandleFn<F>
where
    F: Fn(Context<ClientEvent>) -> U,
    U: IntoFuture<Item = (), Error = JuntaError>,
    <U as IntoFuture>::Future: Send + 'static,
{
    HandleFn { inner: service }
}
