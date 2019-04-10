use futures::Future;
use junta::prelude::*;
use junta_service::plugins::{Extensible, Pluggable};
use junta_service::*;
use serde::{Deserialize, Serialize};
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

    pub fn decode<'a, D: Deserialize<'a>>(&'a self) -> JuntaResult<D> {
        self.ctx.decode()
    }

    pub fn send<S: Serialize>(&self, data: &S) -> impl Future<Item = (), Error = JuntaError> {
        self.ctx.send(data)
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
