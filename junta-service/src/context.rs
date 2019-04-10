use super::plugins::{Extensible, Pluggable};
use futures::prelude::*;
use junta::prelude::*;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::sync::Arc;
use typemap::{ShareMap, TypeMap};

pub struct Context {
    pub(crate) client: Arc<Client>,
    pub(crate) event: MessageContent,
    pub(crate) extensions: ShareMap,
}

impl Context {
    pub fn new(client: Arc<Client>, event: MessageContent) -> Context {
        Context {
            client,
            event,
            extensions: TypeMap::custom(),
        }
    }

    pub fn send<S: Serialize>(&self, data: &S) -> impl Future<Item = (), Error = JuntaError> {
        let ret = match self.event {
            MessageContent::Binary(_) => self.encode_binary(data),
            MessageContent::Text(_) => self.encode_text(data),
        };

        let fut = match ret {
            Ok(msg) => OneOfTwo::Future1(self.client.send(msg)),
            Err(e) => OneOfTwo::Future2(futures::future::err(e)),
        };

        OneOfTwoFuture::new(fut)
    }

    pub fn decode<'a, D: Deserialize<'a>>(&'a self) -> JuntaResult<D> {
        match &self.event {
            MessageContent::Binary(slice) => serde_cbor::from_slice(slice.as_slice())
                .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string()))),
            MessageContent::Text(text) => serde_json::from_str(&text)
                .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string()))),
        }
    }

    pub fn raw(&self) -> &MessageContent {
        &self.event
    }

    pub fn client(&self) -> &Arc<Client> {
        &self.client
    }

    pub fn encode_binary<S: Serialize>(&self, data: &S) -> JuntaResult<MessageContent> {
        Ok(MessageContent::Binary(
            serde_cbor::to_vec(data).map_err(|_e| JuntaError::new(JuntaErrorKind::Unknown))?,
        ))
    }

    pub fn encode_text<S: Serialize>(&self, data: &S) -> JuntaResult<MessageContent> {
        Ok(MessageContent::Text(
            serde_json::to_string(data).map_err(|_e| JuntaError::new(JuntaErrorKind::Unknown))?,
        ))
    }

    pub fn is_binary(&self) -> bool {
        match self.event {
            MessageContent::Binary(_) => true,
            _ => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self.event {
            MessageContent::Text(_) => true,
            _ => false,
        }
    }
}

impl Extensible for Context {
    fn extensions(&self) -> &ShareMap {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut ShareMap {
        &mut self.extensions
    }
}

impl Pluggable for Context {}
