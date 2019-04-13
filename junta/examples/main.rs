use futures::future;
use futures::prelude::*;
use junta::prelude::*;
use junta_service::prelude::*;
use plugins::*;
use slog::{Drain, Logger};
use std::sync::Arc;
use std::time::Duration;
use tokio::prelude::FutureExt;

// struct TestHandler {}

// impl Handler for TestHandler {
//     type Future = future::FutureResult<(), JuntaError>;
//     fn handle(&self, client: &Arc<Client>, event: ClientEvent) -> Self::Future {
//         println!("client {:?}, event: {:?}", client, event);
//         future::ok(())
//     }
// }

struct Store {}

impl Key for Store {
    type Value = Store;
}

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = Logger::root(drain, slog::o! {});

    let mut runtime = tokio::runtime::Builder::new().build().unwrap();

    let fut = Server::bind("127.0.0.1:2794")
        .unwrap()
        .logger(logger)
        .serve(
            runtime.executor(),
            // check_fn(
            //     |ctx: &Context<ClientEvent>| {
            //         let text = match ctx.message() {
            //             ClientEvent::Message(MessageContent::Text(text)) => text,
            //             _ => return false,
            //         };
            //         return text == "greeting";
            //     },
            //     service_fn(|ctx: Context<ClientEvent>| {
            //         ctx.client().send(MessageContent::Text("Hello".to_string()))
            //     }),
            // )
            // .or(service_fn(|ctx: Context<ClientEvent>| {
            //     ctx.client()
            //         .send(MessageContent::Text("Hello 2".to_string()))
            // })),
            service_fn(|ctx: Context<ClientEvent>| {
                if ctx.message().is_message() {}
                //ctx.client().close()
                // ctx.client()
                //     .send(MessageContent::Text("Hello 2".to_string()))
                Ok(())
            }),
        )
        .unwrap();

    runtime.block_on(fut).unwrap();
}
