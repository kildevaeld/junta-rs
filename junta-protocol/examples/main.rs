use futures::future;
use futures::prelude::*;
use junta::prelude::*;
use junta_protocol::prelude::*;
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

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = Logger::root(drain, slog::o! {});

    let mut runtime = tokio::runtime::Builder::new().build().unwrap();

    let service = protocol_req_fn("greeting", |mut ctx| {
        let m = format!("Hello, World {:?}", ctx.decode::<String>().unwrap());
        ctx.client().broadcast(MessageContent::Text(format!(
            "hello from {}",
            ctx.client().id()
        )));

        let client = ctx.client().clone();

        ctx.request("greeting", &()).and_then(move |req: String| {
            println!("did receive {}", req);
            ctx.client().broadcast(MessageContent::Text(format!(
                "hello from again {}",
                ctx.client().id()
            )));
            Ok(m)
        })

        //Ok(m)

        //Ok(m)
    })
    .or(protocol_req_fn("greeting2", |value| {
        let m = format!("Hello, World 2 {:?}", value.message().clone());
        Ok(m)
    }));
    //.into_handler();

    let fut = Server::bind("127.0.0.1:2794")
        .unwrap()
        .logger(logger)
        .serve(runtime.executor(), service)
        .unwrap();

    runtime.block_on(fut).unwrap();
}
