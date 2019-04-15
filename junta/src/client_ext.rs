use super::client::Client;
use super::error::*;
use super::server::MessageContent;
use future_ext::*;
use futures::prelude::*;
use serde::Serialize;

pub trait ClientExt {
    fn send_text<S: Serialize>(
        &self,
        data: &S,
    ) -> Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    fn send_binary<S: Serialize>(
        &self,
        data: &S,
    ) -> Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
}

impl ClientExt for Client {
    fn send_text<S: Serialize>(
        &self,
        data: &S,
    ) -> Box<Future<Item = (), Error = JuntaError> + Send + 'static> {
        let fut = match serde_json::to_string(data) {
            Ok(s) => OneOfTwo::First(self.send(MessageContent::Text(s))),
            Err(e) => OneOfTwo::Second(futures::future::err(e.into())),
        };

        Box::new(OneOfTwoFuture::new(fut))
    }
    fn send_binary<S: Serialize>(
        &self,
        data: &S,
    ) -> Box<Future<Item = (), Error = JuntaError> + Send + 'static> {
        let fut = match serde_cbor::to_vec(data) {
            Ok(s) => OneOfTwo::First(self.send(MessageContent::Binary(s))),
            Err(e) => OneOfTwo::Second(futures::future::err(e.into())),
        };

        Box::new(OneOfTwoFuture::new(fut))
    }
}
