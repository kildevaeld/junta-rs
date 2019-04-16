use super::error::{JuntaError, JuntaErrorKind};
use super::server::{Broadcast, MessageContent};
use atomic_counter::AtomicCounter;
use future_ext::*;
use futures::prelude::*;
use futures::sink::Sink;
use futures::stream::{SplitSink, SplitStream};
use futures::sync::mpsc::{Receiver, Sender};
use futures::sync::oneshot::{Receiver as OneReceiver, Sender as OneSender};
use junta_service::error::ServiceError;
use junta_service::prelude::*;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use tokio::prelude::*;
use uuid::Uuid;
use websocket::r#async::MessageCodec;
use websocket::{CloseData, OwnedMessage};

macro_rules! poll_stream {
    ($stream: expr, $sender: expr) => {
        match $stream.poll() {
            Ok(Async::Ready(None)) => {
                return Ok(Async::Ready(()));
            }
            Ok(Async::Ready(Some(msg))) => {
                //println!("got some {:?}", msg);
                $sender.start_send(msg); //.unwrap();
                                         // $sender = $sender.clone().send(msg).wait();

                Some(())
            }
            Ok(Async::NotReady) => None,
            Err(e) => return Err(JuntaErrorKind::Service(ServiceError::ReceiverClosed).into()),
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
    pub fn is_connect(&self) -> bool {
        match self {
            ClientEvent::Connect => true,
            _ => false,
        }
    }

    pub fn is_message(&self) -> bool {
        match self {
            ClientEvent::Message(_) => true,
            _ => false,
        }
    }

    pub fn is_close(&self) -> bool {
        match self {
            ClientEvent::Close(_) => true,
            _ => false,
        }
    }
}

// impl ClientEvent {
//     fn from(msg: OwnedMessage) -> ClientEvent {
//         match msg {
//             OwnedMessage::Binary(bs) => ClientEvent::Message(MessageContent::Binary(bs)),
//             OwnedMessage::Text(text) => ClientEvent::Message(MessageContent::Text(text)),
//             OwnedMessage::Close(close) => ClientEvent::Close(close),
//             _ => unreachable!("invalid"),
//         }
//     }
// }

//#[derive(Clone)]
pub struct Client {
    pub(crate) id: Uuid,
    pub(crate) sender: Sender<OwnedMessage>,
    pub(crate) server: Arc<
        Broadcast<Future = futures::future::FutureResult<(), JuntaError>> + Send + Sync + 'static,
    >,
    pub(crate) address: SocketAddr,
    pub(crate) counter: Arc<atomic_counter::RelaxedCounter>,
    pub(crate) logger: slog::Logger,
    pub(crate) close: Mutex<Option<OneSender<()>>>,
}

impl Client {
    pub fn send(&self, msg: MessageContent) -> impl Future<Item = (), Error = JuntaError> {
        debug!(self.logger, "sending message {:?}", msg);

        // self.sender
        //     .clone()
        //     .send(msg.to_message())
        //     .map(|_| ())
        //     .map_err(|_| JuntaErrorKind::Send.into())

        match self.sender.clone().start_send(msg.to_message()) {
            Ok(_) => future::ok(()),
            Err(e) => future::err(JuntaErrorKind::Error(Box::new(e)).into()),
        }
    }

    pub fn close(&self) -> impl Future<Item = (), Error = JuntaError> {
        let fut = if self.close.lock().unwrap().is_none() {
            futures::future::ok(())
        } else {
            let sender = self.close.lock().unwrap().take().unwrap();
            if self.sender.is_closed() {
                futures::future::ok(())
            } else {
                match self
                    .sender
                    .clone()
                    .start_send(OwnedMessage::Close(Some(CloseData::new(
                        1000,
                        "NORMAL".to_string(),
                    )))) {
                    Ok(_) => future::ok(()),
                    Err(e) => future::err(JuntaErrorKind::Error(Box::new(e)).into()),
                }
            }

            // self.sender
            //     .clone()
            //     .send(OwnedMessage::Close(Some(CloseData::new(
            //         1000,
            //         "NORMAL".to_string(),
            //     ))))
            //     .map(|_| {
            //         sender.send(()).unwrap();
            //         ()
            //     })
            //     .map_err(|_| JuntaErrorKind::Send.into()),

            // sender.send(());
            // OneOfTwo::Future2(futures::future::ok(()))
        };

        fut
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn broadcast(
        &self,
        msg: MessageContent,
    ) -> Box<Future<Item = (), Error = JuntaError> + Send + 'static> {
        debug!(self.logger, "broadcast message");
        Box::new(self.server.broadcast(self, msg))
    }

    pub fn next_seq(&self) -> usize {
        self.counter.inc()
    }

    pub fn logger(&self) -> &slog::Logger {
        &self.logger
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
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

// impl Drop for Client {
//     fn drop(&mut self) {
//         println!("drop that motherfucker");
//     }
// }

pub struct ClientFuture {
    //id: Uuid,
    sink: SplitSink<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
    stream: SplitStream<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
    sender: Sender<OwnedMessage>,
    recv: Receiver<OwnedMessage>,
    close: OneReceiver<()>,
    closed: bool,
}

impl ClientFuture {
    pub fn new(
        //id: uuid::Uuid,
        sink: SplitSink<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
        stream: SplitStream<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
        sender: Sender<OwnedMessage>,
        recv: Receiver<OwnedMessage>,
        close: OneReceiver<()>,
    ) -> ClientFuture {
        ClientFuture {
            //id,
            sink,
            stream,
            sender,
            recv,
            close,
            closed: false,
        }
    }
}

impl Future for ClientFuture {
    type Item = ();
    type Error = JuntaError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.closed {
            return Ok(Async::Ready(()));
        }
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

        match self.sender.poll_complete() {
            Ok(Async::NotReady) => (),
            Ok(Async::Ready(_)) => {
                self.sender.poll_complete();
                // println!("sender poll ready");
                //return Ok(Async::Ready(()));
                ()
            }
            Err(e) => {
                //println!("sender poll error");
                return Err(JuntaErrorKind::Error(Box::new(e)).into());
            }
        };

        match self.sink.poll_complete() {
            Ok(Async::NotReady) => (),
            Ok(Async::Ready(_)) => {
                self.sink.poll_complete();

                //return Ok(Async::Ready(()));
                ()
            }
            Err(e) => return Err(JuntaErrorKind::Transport(e).into()),
        };

        match self.close.poll() {
            Ok(Async::NotReady) => (),
            Ok(Async::Ready(_)) => {
                //println!("closing the channel");
                self.sink.close();
                self.closed = true;
            }
            Err(e) => {
                //self.sink.close();
                //<self.sink.close();

                //println!("cerror polling close");
                return Ok(Async::Ready(()));
                //return Err(JuntaErrorKind::Error(Box::new(e)).into());
            }
        };

        Ok(Async::NotReady)
    }
}
