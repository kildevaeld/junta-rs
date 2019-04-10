use super::error::{JuntaError, JuntaErrorKind};
use super::server::{Broadcast, MessageContent, Server};
use futures::prelude::*;
use futures::sink::Sink;
use futures::stream::{SplitSink, SplitStream};
use futures::sync::mpsc::{Receiver, Sender};
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use uuid::Uuid;
use websocket::r#async::MessageCodec;
use websocket::{CloseData, OwnedMessage};

macro_rules! poll_stream {
    ($stream: expr, $sender: expr) => {
        match $stream.poll() {
            Ok(Async::Ready(None)) => {
                println!("stream poll ready none");
                return Ok(Async::Ready(()));
            }
            Ok(Async::Ready(Some(msg))) => {
                //println!("got some {:?}", msg);
                $sender.start_send(msg); //.unwrap();
                                         // $sender = $sender.clone().send(msg).wait();

                Some(())
            }
            Ok(Async::NotReady) => None,
            Err(_) => return Err(JuntaErrorKind::Unknown.into()),
        }
    };
}

#[derive(Clone, PartialEq, Debug)]
pub enum ClientEvent {
    Connect,
    Message(MessageContent),
    Close(Option<CloseData>),
}

impl ClientEvent {
    fn from(msg: OwnedMessage) -> ClientEvent {
        match msg {
            OwnedMessage::Binary(bs) => ClientEvent::Message(MessageContent::Binary(bs)),
            OwnedMessage::Text(text) => ClientEvent::Message(MessageContent::Text(text)),
            OwnedMessage::Close(close) => ClientEvent::Close(close),
            _ => unreachable!("invalid"),
        }
    }
}

//#[derive(Clone)]
pub struct Client {
    pub(crate) id: Uuid,
    pub(crate) sender: Sender<OwnedMessage>,
    pub(crate) server: Arc<
        Broadcast<Future = futures::future::FutureResult<(), JuntaError>> + Send + Sync + 'static,
    >,
    pub(crate) address: SocketAddr,
}

impl Client {
    pub fn send(&self, msg: MessageContent) -> impl Future<Item = (), Error = JuntaError> {
        self.sender
            .clone()
            .send(msg.to_message())
            .map(|_| ())
            .map_err(|_| JuntaErrorKind::Send.into())
    }

    pub fn close(&self) -> impl Future<Item = (), Error = JuntaError> {
        self.sender
            .clone()
            .send(OwnedMessage::Close(None))
            .map(|_| ())
            .map_err(|_| JuntaErrorKind::Send.into())
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn broadcast(
        &self,
        msg: MessageContent,
    ) -> Box<Future<Item = (), Error = JuntaError> + Send + 'static> {
        Box::new(self.server.broadcast(self, msg))
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Client) -> bool {
        self.id == other.id
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        println!("drop that motherfucker");
    }
}

pub struct ClientFuture {
    id: Uuid,
    sink: SplitSink<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
    stream: SplitStream<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
    sender: Sender<OwnedMessage>,
    recv: Receiver<OwnedMessage>,
}

impl ClientFuture {
    pub fn new(
        id: uuid::Uuid,
        sink: SplitSink<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
        stream: SplitStream<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
        sender: Sender<OwnedMessage>,
        recv: Receiver<OwnedMessage>,
    ) -> ClientFuture {
        ClientFuture {
            id,
            sink,
            stream,
            sender,
            recv,
        }
    }
}

impl Future for ClientFuture {
    type Item = ();
    type Error = JuntaError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match poll_stream!(self.recv, self.sink) {
            Some(_) => {
                poll_stream!(self.recv, self.sink);
                ()
            }
            None => (),
        };

        match poll_stream!(self.stream, self.sender) {
            Some(_) => {
                poll_stream!(self.stream, self.sender);
                ()
            }
            None => (),
        };

        // match self.sender.poll_complete() {
        //     Ok(Async::NotReady) => (),
        //     Ok(Async::Ready(_)) => {
        //         self.sender.poll_complete();
        //         // println!("sender poll ready");
        //         //return Ok(Async::Ready(()));
        //         ()
        //     }
        //     Err(_) => return Err(JuntaErrorKind::Unknown.into()),
        // };

        match self.sink.poll_complete() {
            Ok(Async::NotReady) => (),
            Ok(Async::Ready(_)) => {
                self.sink.poll_complete();
                // println!("sink poll ready");
                //return Ok(Async::Ready(()));
                ()
            }
            Err(_) => return Err(JuntaErrorKind::Unknown.into()),
        };

        Ok(Async::NotReady)
    }
}