use super::context::Context;
use futures::prelude::*;
use junta::prelude::*;
use std::sync::Arc;

pub trait Service {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    fn execute(&self, ctx: Context) -> Self::Future;
    fn check(&self, _ctx: &Context) -> bool {
        true
    }
}

pub trait IntoService {
    type Future: Future<Item = (), Error = JuntaError> + Send + 'static;
    type Service: Service<Future = Self::Future>;
    fn into_service(self) -> Self::Service;
}

impl<S> IntoService for S
where
    S: Service,
{
    type Future = S::Future;
    type Service = S;
    fn into_service(self) -> Self::Service {
        self
    }
}

pub struct ServiceFn<F> {
    inner: F,
}

impl<F, U> Service for ServiceFn<F>
where
    F: Fn(Context) -> U,
    U: IntoFuture<Item = (), Error = JuntaError>,
    <U as IntoFuture>::Future: Send + 'static,
{
    type Future = U::Future;
    fn execute(&self, ctx: Context) -> Self::Future {
        (self.inner)(ctx).into_future()
    }
}

impl<F, U> IntoHandler for ServiceFn<F>
where
    F: (Fn(Context) -> U) + Send + Sync + 'static,
    U: IntoFuture<Item = (), Error = JuntaError>,
    <U as IntoFuture>::Future: Send + 'static,
{
    type Future = <ServiceHandler<Self> as Handler>::Future;
    type Handler = ServiceHandler<Self>;
    fn into_handler(self) -> Self::Handler {
        ServiceHandler::new(self)
    }
}

pub fn service_fn<F, U>(service: F) -> ServiceFn<F>
where
    F: Fn(Context) -> U,
    U: IntoFuture<Item = (), Error = JuntaError>,
    <U as IntoFuture>::Future: Send + 'static,
{
    ServiceFn { inner: service }
}

pub struct ServiceHandler<S: Service> {
    service: S,
}

impl<S: Service> ServiceHandler<S> {
    pub fn new(service: S) -> ServiceHandler<S> {
        ServiceHandler { service }
    }
}

impl<S: Service + Send + Sync> Handler for ServiceHandler<S> {
    type Future =
        OneOfTwoFuture<(), JuntaError, S::Future, futures::future::FutureResult<(), JuntaError>>;
    fn handle(self: &Self, client: &Arc<Client>, event: ClientEvent) -> Self::Future {
        let fut = match event {
            ClientEvent::Close(_) => OneOfTwo::Future2(futures::future::ok(())),
            ClientEvent::Connect => OneOfTwo::Future2(futures::future::ok(())),
            ClientEvent::Message(msg) => {
                let ctx = Context::new(client.clone(), msg);

                if self.service.check(&ctx) {
                    OneOfTwo::Future1(self.service.execute(ctx))
                } else {
                    OneOfTwo::Future2(futures::future::ok(()))
                }
            }
        };

        OneOfTwoFuture::new(fut)
    }
}

pub struct ServiceChain<S1, S2> {
    s1: S1,
    s2: S2,
}

impl<S1, S2> ServiceChain<S1, S2> {
    pub fn new(s1: S1, s2: S2) -> ServiceChain<S1, S2> {
        ServiceChain { s1, s2 }
    }
}

impl<S1, S2> Service for ServiceChain<S1, S2>
where
    S1: Service,
    S2: Service,
{
    type Future = OneOfTreeFuture<
        (),
        JuntaError,
        S1::Future,
        S2::Future,
        futures::future::FutureResult<(), JuntaError>,
    >;

    fn execute(&self, ctx: Context) -> Self::Future {
        let fut = if self.s1.check(&ctx) {
            OneOfTree::Future1(self.s1.execute(ctx))
        } else if self.s2.check(&ctx) {
            OneOfTree::Future2(self.s2.execute(ctx))
        } else {
            OneOfTree::Future3(futures::future::err(
                JuntaErrorKind::Error("invalid request".to_string()).into(),
            ))
        };
        OneOfTreeFuture::new(fut)
    }

    #[inline]
    fn check(&self, ctx: &Context) -> bool {
        self.s1.check(ctx) || self.s2.check(ctx)
    }
}

impl<S1, S2> IntoHandler for ServiceChain<S1, S2>
where
    S1: Service + Send + Sync + 'static,
    S2: Service + Send + Sync + 'static,
{
    type Future = <ServiceHandler<Self> as Handler>::Future;
    type Handler = ServiceHandler<Self>;
    fn into_handler(self) -> Self::Handler {
        ServiceHandler::new(self)
    }
}

pub trait ServiceExt: Service + Sized {
    fn or<S: Service>(self, service: S) -> ServiceChain<Self, S> {
        ServiceChain::new(self, service)
    }
}

impl<T> ServiceExt for T where T: Service {}
