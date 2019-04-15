use super::client::{Client, ClientEvent, ClientFuture};
use super::context::Context;
use super::error::{JuntaError, JuntaErrorKind, JuntaResult};
use future_ext::{OneOfFour, OneOfFourFuture, OneOfTwo, OneOfTwoFuture};
use futures::prelude::*;
use futures::sync::oneshot::channel;
use junta_service::prelude::*;
use slog::{Discard, Logger};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use tokio::reactor::Handle;
use tokio::runtime::TaskExecutor;
use uuid::Uuid;
use websocket::message::OwnedMessage;
use websocket::r#async::Server as WSServer;
use websocket::server::InvalidConnection;
use websocket::WebSocketError;

pub type ClientList = Arc<RwLock<HashMap<Uuid, Arc<Client>>>>;

pub trait Broadcast {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn send_all(&self, msg: MessageContent) -> Self::Future;
    fn broadcast(&self, client: &Client, msg: MessageContent) -> Self::Future;
    fn client(&self, id: &Uuid) -> Option<Arc<Client>>;
}

struct Broadcaster {
    clients: ClientList,
    executor: TaskExecutor,
}

impl Broadcast for Broadcaster {
    type Future = futures::future::FutureResult<(), JuntaError>; // Box<Future<Item = (), Error = JuntaError> + Send + 'static>;
    fn send_all(&self, msg: MessageContent) -> Self::Future {
        let clients = self.clients.read().unwrap();
        let mut promises = Vec::new();
        for (_, v) in clients.iter() {
            promises.push(v.send(msg.clone()))
        }
        self.executor.spawn(
            futures::future::join_all(promises)
                .map(|_| ())
                .map_err(|_| ()),
        );

        futures::future::ok(())
        // Box::new(futures::future::join_all(promises).map(|_| ()))
    }

    fn broadcast(&self, client: &Client, msg: MessageContent) -> Self::Future {
        let clients = self.clients.read().unwrap();
        let mut promises = Vec::new();
        for (_, v) in clients.iter() {
            if v.as_ref() == client {
                continue;
            }
            promises.push(v.send(msg.clone()))
        }
        self.executor.spawn(
            futures::future::join_all(promises)
                .map(|_| ())
                .map_err(|_| ()),
        );

        futures::future::ok(())
    }

    fn client(&self, id: &Uuid) -> Option<Arc<Client>> {
        self.clients.read().unwrap().get(id).map(|c| c.clone())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum MessageContent {
    Text(String),
    Binary(Vec<u8>),
}

impl MessageContent {
    pub(crate) fn to_message(self) -> OwnedMessage {
        match self {
            MessageContent::Text(text) => OwnedMessage::Text(text),
            MessageContent::Binary(bs) => OwnedMessage::Binary(bs),
        }
    }
}

pub struct ServerBuilder {
    addr: SocketAddr,
    logger: Logger,
    // executor: TaskExecutor,
}

impl ServerBuilder {
    pub fn logger(mut self, logger: Logger) -> Self {
        self.logger = logger;
        self
    }

    pub fn serve<H>(self, executor: TaskExecutor, handler: H) -> JuntaResult<Server>
    where
        H: IntoService<Input = Context<ClientEvent>, Output = (), Error = JuntaError>,
        <H as IntoService>::Service: 'static + Send + Sync,
    {
        let clients = Arc::new(RwLock::new(HashMap::new()));
        Ok(Server {
            inner: ServerHandler::new(
                executor,
                Arc::new(handler.into_service()),
                clients,
                self.logger,
                self.addr,
            )?,
        })
    }
}

pub struct Server {
    inner: ServerHandler,
}

impl Server {
    pub fn bind<S: ToSocketAddrs>(addr: S) -> JuntaResult<ServerBuilder> {
        Ok(ServerBuilder {
            addr: addr.to_socket_addrs()?.nth(0).unwrap(),
            logger: Logger::root(Discard, o! {}),
        })
    }
}

impl Future for Server {
    type Item = ();
    type Error = JuntaError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}

struct ServerHandler {
    inner: Box<Future<Item = (), Error = JuntaError> + Send>,
}

impl ServerHandler {
    pub fn new<H: Service + 'static>(
        executor: TaskExecutor,
        handler: Arc<H>,
        clients: ClientList,
        logger: Logger,
        addr: SocketAddr,
    ) -> JuntaResult<ServerHandler>
    where
        H: Service<Input = Context<ClientEvent>, Output = (), Error = JuntaError>
            + 'static
            + Send
            + Sync,
    {
        let counter = Arc::new(atomic_counter::RelaxedCounter::new(1));

        let work = WSServer::bind(addr, &Handle::default())?
            .incoming()
            .map_err(|InvalidConnection { error, .. }| {
                JuntaError::new(JuntaErrorKind::Transport(WebSocketError::from(error)))
            })
            .from_err::<JuntaError>()
            .for_each(move |(upgrade, addr)| {
                let fut = if !upgrade.protocols().iter().any(|s| s == "rust-websocket") {
                    executor.spawn(upgrade.reject().map(|_| ()).map_err(|_| ()));
                    OneOfTwo::First(futures::future::ok(()))
                } else {
                    let t = executor.clone();

                    let clients = clients.clone();
                    let logger = logger.clone();
                    let handler = handler.clone();
                    let counter = counter.clone();

                    OneOfTwo::Second(
                        upgrade
                            .use_protocol("rust-websocket")
                            .accept()
                            .map_err(|e| JuntaError::new(JuntaErrorKind::Transport(e)))
                            .and_then(move |(client, _)| {
                                ServerHandler::connect(
                                    clients, logger, t, handler, client, addr, counter,
                                )
                            }),
                    )
                };

                OneOfTwoFuture::new(fut)
            });
        Ok(ServerHandler {
            inner: Box::new(work),
        })
    }

    fn connect<H: Service + 'static>(
        clients: ClientList,
        logger: Logger,
        executor: TaskExecutor,
        handler: Arc<H>,
        client: tokio::codec::Framed<
            tokio::net::TcpStream,
            websocket::r#async::MessageCodec<websocket::OwnedMessage>,
        >,
        addr: SocketAddr,
        counter: Arc<atomic_counter::RelaxedCounter>,
    ) -> impl Future<Item = (), Error = JuntaError>
    where
        H: Service<Input = Context<ClientEvent>, Output = (), Error = JuntaError>
            + 'static
            + Send
            + Sync,
    {
        let id = Uuid::new_v4();

        let logger = logger.new(slog::o! {
            "client" => id.to_string(),
            "address" => format!("{:?}",client.get_ref().peer_addr().unwrap())
        });

        info!(logger, "client connected");

        let (sink, stream) = client.split();

        let (sx, rx) = futures::sync::mpsc::channel(20);
        let (sx1, rx1) = futures::sync::mpsc::channel(20);
        let (sx2, rx2) = futures::sync::oneshot::channel();

        let client = Arc::new(Client {
            id: id.clone(),
            sender: sx1,
            server: Arc::new(Broadcaster {
                clients: clients.clone(),
                executor: executor.clone(),
            }),
            address: addr,
            counter: counter,
            logger: logger.clone(),
            close: Mutex::new(Some(sx2)),
        });

        let (cloned_client, cloned_list, cloned_handler) =
            (client.clone(), clients.clone(), handler.clone());

        let cl = client.clone();
        //let this2 = this.clone();

        clients.write().unwrap().insert(id.clone(), client);
        let exec = executor.clone();
        let logger = logger.clone();
        let v = handler
            .call(Context::<ClientEvent>::new(
                cl.clone(),
                ClientEvent::Connect,
            ))
            .and_then(|_| {
                rx.map_err(|_| JuntaError::from(ServiceError::ReceiverClosed))
                    .for_each(move |msg| {
                        let cl = cl.clone();
                        let fut = match msg {
                            OwnedMessage::Close(close_data) => {
                                clients.write().unwrap().remove(&cl.id);
                                debug!(logger, "client sent close message");
                                let client = cl.clone();
                                let logger = logger.clone();
                                let out = handler
                                    .call(Context::<ClientEvent>::new(
                                        cl,
                                        ClientEvent::Close(close_data),
                                    ))
                                    .and_then(move |_| {
                                        debug!(logger, "sending close to client");
                                        client.close()
                                    });

                                OneOfFour::First(out)
                            }
                            OwnedMessage::Ping(ping) => {
                                debug!(logger, "client sent ping");
                                OneOfFour::Second(
                                    cl.sender
                                        .clone()
                                        .send(OwnedMessage::Pong(ping))
                                        .map(|_| ())
                                        .map_err(|_| JuntaErrorKind::Send.into()),
                                )
                            }
                            OwnedMessage::Pong(_) => OneOfFour::Third(futures::future::ok(())),
                            OwnedMessage::Binary(data) => {
                                debug!(logger, "client sent binary message");
                                OneOfFour::Fourth(handler.call(Context::<ClientEvent>::new(
                                    cl,
                                    ClientEvent::Message(MessageContent::Binary(data)),
                                )))
                            }
                            OwnedMessage::Text(data) => {
                                debug!(logger, "client sent text message");
                                OneOfFour::Fourth(handler.call(Context::<ClientEvent>::new(
                                    cl,
                                    ClientEvent::Message(MessageContent::Text(data)),
                                )))
                            }
                        };

                        exec.spawn(
                            OneOfFourFuture::new(fut)
                                // .or_else(|e| {
                                //     println!("TOP-error {}", e);
                                //     Ok(())
                                // })
                                .map_err(|e: JuntaError| {
                                    println!("error {}", e);
                                    ()
                                }),
                        );
                        futures::future::ok(())
                        // OneOfFourFuture::new(fut).map_err(|e: JuntaError| {
                        //     //println!("error {}", e);
                        //     e
                        // })
                    })
            });

        let clogger = cloned_client.logger().clone();
        let fut = ClientFuture::new(sink, stream, sx, rx1, rx2);
        let client = cloned_client.clone();
        executor.spawn(
            v.join(fut)
                .and_then(move |_| {
                    let logger = cloned_client.logger().clone();
                    let elogger = logger.clone();
                    cloned_list.write().unwrap().remove(cloned_client.id());
                    let client = cloned_client.clone();
                    cloned_handler
                        .call(Context::<ClientEvent>::new(
                            cloned_client,
                            ClientEvent::Close(None),
                        ))
                        .map(move |_| {
                            info!(client.logger(), "client closed");
                            ()
                        })
                        .map_err(move |e| {
                            error!(elogger, "client closed with error"; "error" => e.to_string());
                            e
                        })
                })
                .map_err(move |e| {
                    error!(client.logger(), "client finished with error {}", e);
                    ()
                }),
        );
        futures::future::ok(())
    }
}

impl Future for ServerHandler {
    type Item = ();
    type Error = JuntaError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}
