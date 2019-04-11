use super::client::Client;
use super::client::ClientEvent;
#[cfg(feature = "encoding")]
use super::error::*;
use super::plugins::{Extensible, Pluggable};
use super::server::MessageContent;
#[cfg(feature = "encoding")]
use super::utils::*;
#[cfg(feature = "encoding")]
use futures::prelude::*;
use std::sync::Arc;
use typemap::{ShareMap, TypeMap};

pub struct Context<I> {
    client: Arc<Client>,
    extensions: ShareMap,
    message: I,
    binary: bool,
}

impl<I> Context<I> {
    pub fn new(client: Arc<Client>, message: ClientEvent) -> Context<ClientEvent> {
        let binary = match message {
            ClientEvent::Message(MessageContent::Binary(_)) => true,
            _ => false,
        };

        Context {
            client,
            message,
            extensions: TypeMap::custom(),
            binary,
        }
    }

    pub fn message(&self) -> &I {
        &self.message
    }

    pub fn into_message(self) -> I {
        self.message
    }

    pub fn with_message<S>(self, message: S) -> (Context<S>, I) {
        let value = self.message;
        (
            Context {
                client: self.client,
                extensions: self.extensions,
                message,
                binary: self.binary,
            },
            value,
        )
    }

    // pub fn borrow(&self, message: S) -> BorrowedContext<

    pub fn client(&self) -> &Arc<Client> {
        &self.client
    }

    #[cfg(feature = "encoding")]
    pub fn send<S: serde::Serialize>(
        &self,
        data: &S,
    ) -> impl Future<Item = (), Error = JuntaError> {
        let ret = if self.binary {
            self.encode_binary(data)
        } else {
            self.encode_text(data)
        };

        let fut = match ret {
            Ok(msg) => OneOfTwo::Future1(self.client.send(msg)),
            Err(e) => OneOfTwo::Future2(futures::future::err(e)),
        };

        OneOfTwoFuture::new(fut)
    }

    #[cfg(feature = "encoding")]
    pub fn encode_binary<S: serde::Serialize>(&self, data: &S) -> JuntaResult<MessageContent> {
        Ok(MessageContent::Binary(
            serde_cbor::to_vec(data).map_err(|_e| JuntaError::new(JuntaErrorKind::Unknown))?,
        ))
    }

    #[cfg(feature = "encoding")]
    pub fn encode_text<S: serde::Serialize>(&self, data: &S) -> JuntaResult<MessageContent> {
        Ok(MessageContent::Text(
            serde_json::to_string(data).map_err(|_e| JuntaError::new(JuntaErrorKind::Unknown))?,
        ))
    }

    pub fn binary(&self) -> bool {
        self.binary
    }
}

impl Context<ClientEvent> {
    #[cfg(feature = "encoding")]
    pub fn decode<'a, D: serde::de::Deserialize<'a>>(&'a self) -> JuntaResult<D> {
        match self.message() {
            ClientEvent::Message(MessageContent::Binary(slice)) => {
                serde_cbor::from_slice(slice.as_slice())
                    .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string())))
            }
            ClientEvent::Message(MessageContent::Text(text)) => serde_json::from_str(&text)
                .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string()))),
            _ => Err(JuntaErrorKind::Error("invalid".to_string()).into()),
        }
    }
}

impl Context<MessageContent> {
    #[cfg(feature = "encoding")]
    pub fn decode<'a, D: serde::de::Deserialize<'a>>(&'a self) -> JuntaResult<D> {
        match self.message() {
            MessageContent::Binary(slice) => serde_cbor::from_slice(slice.as_slice())
                .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string()))),
            MessageContent::Text(text) => serde_json::from_str(&text)
                .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string()))),
        }
    }
}

#[cfg(feature = "encoding")]
impl Context<serde_cbor::Value> {
    pub fn decode<D: serde::de::DeserializeOwned>(&self) -> JuntaResult<D> {
        serde_cbor::from_value(self.message().clone())
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
    }
}

#[cfg(feature = "encoding")]
impl Context<serde_json::Value> {
    pub fn decode<D: serde::de::DeserializeOwned>(&self) -> JuntaResult<D> {
        serde_json::from_value(self.message().clone())
            .map_err(|_| JuntaError::new(JuntaErrorKind::Unknown))
    }
}

impl<I> Extensible for Context<I> {
    fn extensions(&self) -> &ShareMap {
        &self.extensions
    }

    fn extensions_mut(&mut self) -> &mut ShareMap {
        &mut self.extensions
    }
}

impl<I> Pluggable for Context<I> {}

pub struct BorrowedContext<'a, I, M> {
    ctx: &'a Context<I>,
    msg: &'a M,
}

impl<'a, I, M> BorrowedContext<'a, I, M> {
    pub fn new(ctx: &'a Context<I>, msg: &'a M) -> BorrowedContext<'a, I, M> {
        BorrowedContext { ctx, msg }
    }

    pub fn message(&self) -> &M {
        self.msg
    }

    pub fn client(&self) -> &Arc<Client> {
        self.ctx.client()
    }

    #[cfg(feature = "encoding")]
    pub fn send<S: serde::Serialize>(
        &self,
        data: &S,
    ) -> impl Future<Item = (), Error = JuntaError> {
        self.ctx.send(data)
    }

    #[cfg(feature = "encoding")]
    pub fn encode_binary<S: serde::Serialize>(&self, data: &S) -> JuntaResult<MessageContent> {
        self.ctx.encode_binary(data)
    }

    #[cfg(feature = "encoding")]
    pub fn encode_text<S: serde::Serialize>(&self, data: &S) -> JuntaResult<MessageContent> {
        self.ctx.encode_text(data)
    }

    pub fn binary(&self) -> bool {
        self.ctx.binary
    }

    pub fn extensions(&self) -> &ShareMap {
        self.ctx.extensions()
    }
}

impl<'a, I> BorrowedContext<'a, I, ClientEvent> {
    #[cfg(feature = "encoding")]
    pub fn decode<'de, D: serde::de::Deserialize<'de>>(&'de self) -> JuntaResult<D> {
        match self.message() {
            ClientEvent::Message(MessageContent::Binary(slice)) => {
                serde_cbor::from_slice(slice.as_slice())
                    .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string())))
            }
            ClientEvent::Message(MessageContent::Text(text)) => serde_json::from_str(&text)
                .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string()))),
            _ => Err(JuntaErrorKind::Error("invalid".to_string()).into()),
        }
    }
}

pub struct ChildContext<I, M> {
    ctx: Context<I>,
    msg: M,
}

impl<I, M> ChildContext<I, M> {
    pub fn new(ctx: Context<I>, msg: M) -> ChildContext<I, M> {
        ChildContext { ctx, msg }
    }

    pub fn parent(&self) -> &Context<I> {
        &self.ctx
    }

    pub fn into_parent(self) -> Context<I> {
        self.ctx
    }

    pub fn message(&self) -> &M {
        &self.msg
    }

    pub fn client(&self) -> &Arc<Client> {
        self.ctx.client()
    }

    #[cfg(feature = "encoding")]
    pub fn send<S: serde::Serialize>(
        &self,
        data: &S,
    ) -> impl Future<Item = (), Error = JuntaError> {
        self.ctx.send(data)
    }

    #[cfg(feature = "encoding")]
    pub fn encode_binary<S: serde::Serialize>(&self, data: &S) -> JuntaResult<MessageContent> {
        self.ctx.encode_binary(data)
    }

    #[cfg(feature = "encoding")]
    pub fn encode_text<S: serde::Serialize>(&self, data: &S) -> JuntaResult<MessageContent> {
        self.ctx.encode_text(data)
    }

    pub fn binary(&self) -> bool {
        self.ctx.binary
    }
}

impl<I> ChildContext<I, ClientEvent> {
    #[cfg(feature = "encoding")]
    pub fn decode<'de, D: serde::de::Deserialize<'de>>(&'de self) -> JuntaResult<D> {
        match self.message() {
            ClientEvent::Message(MessageContent::Binary(slice)) => {
                serde_cbor::from_slice(slice.as_slice())
                    .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string())))
            }
            ClientEvent::Message(MessageContent::Text(text)) => serde_json::from_str(&text)
                .map_err(|e| JuntaError::new(JuntaErrorKind::Error(e.to_string()))),
            _ => Err(JuntaErrorKind::Error("invalid".to_string()).into()),
        }
    }
}

impl<I, M> Extensible for ChildContext<I, M> {
    fn extensions(&self) -> &ShareMap {
        &self.ctx.extensions
    }

    fn extensions_mut(&mut self) -> &mut ShareMap {
        &mut self.ctx.extensions
    }
}

impl<I, M> Pluggable for ChildContext<I, M> {}
