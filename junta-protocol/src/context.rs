use futures::Future;
use junta::prelude::*;
use junta_service::plugins::{Extensible, Pluggable};
use junta_service::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use typemap::ShareMap;

pub struct ProtocolContext<I> {
    pub(crate) ctx: Context,
    pub(crate) data: I,
}

impl<I> ProtocolContext<I> {
    pub fn ctx(&self) -> &Context {
        &self.ctx
    }
    pub fn data(&self) -> &I {
        &self.data
    }

    pub fn send<S: Serialize>(&self, data: &S) -> impl Future<Item = (), Error = JuntaError> {
        self.ctx.send(data)
    }
}

impl ProtocolContext<serde_cbor::Value> {
    pub fn decode<D: DeserializeOwned>(&self) -> JuntaResult<D> {
        serde_cbor::from_value(self.data().clone())
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
    }
}

impl<I> Extensible for ProtocolContext<I> {
    fn extensions(&self) -> &ShareMap {
        &self.ctx.extensions()
    }

    fn extensions_mut(&mut self) -> &mut ShareMap {
        self.ctx.extensions_mut()
    }
}

impl<I> Pluggable for ProtocolContext<I> {}
