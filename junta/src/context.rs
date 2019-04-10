use super::client::Client;
use super::client::ClientEvent;
use super::server::MessageContent;
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

    pub fn with_message<S>(self, message: S) -> Context<S> {
        Context {
            client: self.client,
            extensions: self.extensions,
            message,
            binary: self.binary,
        }
    }

    pub fn client(&self) -> &Arc<Client> {
        &self.client
    }
}
