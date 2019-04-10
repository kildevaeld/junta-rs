use futures::prelude::*;
use junta::prelude::*;
use serde::Serialize;
use websocket::OwnedMessage;

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
        let fut = match serde_json::to_string(data)
            .map_err(|e| JuntaError::new(JuntaErrorKind::Unknown))
        {
            Ok(s) => OneOfTwo::Future1(self.send(MessageContent::Text(s))),
            Err(e) => OneOfTwo::Future2(futures::future::err(e)),
        };

        Box::new(OneOfTwoFuture::new(fut))
    }
    fn send_binary<S: Serialize>(
        &self,
        data: &S,
    ) -> Box<Future<Item = (), Error = JuntaError> + Send + 'static> {
        let fut =
            match serde_cbor::to_vec(data).map_err(|e| JuntaError::new(JuntaErrorKind::Unknown)) {
                Ok(s) => OneOfTwo::Future1(self.send(MessageContent::Binary(s))),
                Err(e) => OneOfTwo::Future2(futures::future::err(e)),
            };

        Box::new(OneOfTwoFuture::new(fut))
    }
}
