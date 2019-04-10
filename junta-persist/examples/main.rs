use futures::future;
use futures::prelude::*;
use junta::prelude::*;
use junta_middleware::*;
use junta_persist::*;
use junta_service::plugins::*;
use junta_service::*;
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

    let m = Read::<Store>::middleware(Store {});

    let m = m
        .stack(middleware_fn(|ctx, next| {
            println!("Middleware");
            next.execute(ctx)
        }))
        .then(service_fn(|mut ctx| {
            let store = ctx.get::<Read<Store>>()?;
            println!("Hello");
            Ok(())
        })); //.stack(middleware_fn(|ctx, next| Ok(())));

    let fut = Server::bind("127.0.0.1:2794")
        .unwrap()
        .logger(logger)
        .serve(runtime.executor(), ServiceHandler::new(m))
        .unwrap();

    runtime.block_on(fut).unwrap();
}
