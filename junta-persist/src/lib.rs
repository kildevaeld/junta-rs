#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
// // ShareCloneMap
// #![allow(where_clauses_object_safety)]

//! A set of middleware for sharing data between requests in the Iron
//! framework.

use junta::prelude::*;
use junta_middleware::{Middleware, Next, NextFuture};
use junta_service::plugins::{Extensible, Plugin};
use junta_service::*;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};
// use valse::{AfterMiddleware, BeforeMiddleware, IronResult, Request, Response};

/// The type that can be returned by `eval` to indicate error.
#[derive(Clone, Debug)]
pub enum PersistentError {
    /// The value was not found.
    NotFound,
}

impl Error for PersistentError {
    fn description(&self) -> &str {
        match *self {
            PersistentError::NotFound => "Value not found in extensions.",
        }
    }
}

impl fmt::Display for PersistentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

impl From<PersistentError> for JuntaError {
    fn from(error: PersistentError) -> JuntaError {
        //ValseError::new(error, StatusCode::INTERNAL_SERVER_ERROR)
        JuntaError::new(JuntaErrorKind::Error(error.to_string()))
    }
}

/// Helper trait for overloading the constructors of `Read`/`Write`/`State`.
/// This is an implementation detail, and should not be used for any other
/// purpose.
///
/// For example, this trait lets you construct a `Read<T>` from either a `T` or
/// an `Arc<T>`.
pub trait PersistentInto<T> {
    /// Convert `self` into a value of type `T`.
    fn persistent_into(self) -> T;
}

impl<T> PersistentInto<T> for T {
    fn persistent_into(self) -> T {
        self
    }
}

impl<T> PersistentInto<Arc<T>> for T {
    fn persistent_into(self) -> Arc<T> {
        Arc::new(self)
    }
}

impl<T> PersistentInto<Arc<Mutex<T>>> for T {
    fn persistent_into(self) -> Arc<Mutex<T>> {
        Arc::new(Mutex::new(self))
    }
}

impl<T> PersistentInto<Arc<RwLock<T>>> for T {
    fn persistent_into(self) -> Arc<RwLock<T>> {
        Arc::new(RwLock::new(self))
    }
}

/// Middleware for data that persists between requests with read and write capabilities.
///
/// The data is stored behind a `RwLock`, so multiple read locks
/// can be taken out concurrently.
///
/// If most threads need to take out a write lock, you may want to
/// consider `Write`, which stores the data behind a `Mutex`, which
/// has a faster locking speed.
///
/// `State` can be linked as `BeforeMiddleware` to add data to the `Request`
/// extensions and it can be linked as an `AfterMiddleware` to add data to
/// the `Response` extensions.
///
/// `State` also implements `Plugin`, so the data stored within can be
/// accessed through `request.get::<State<P>>()` as an `Arc<RwLock<P::Value>>`.
pub struct State<P: Key> {
    data: Arc<RwLock<P::Value>>,
}

/// Middleware for data that persists between Requests with read-only capabilities.
///
/// The data is stored behind an Arc, so multiple threads can have
/// concurrent, non-blocking access.
///
/// `Read` can be linked as `BeforeMiddleware` to add data to the `Request`
/// extensions and it can be linked as an `AfterMiddleware` to add data to
/// the `Response` extensions.
///
/// `Read` also implements `Plugin`, so the data stored within can be
/// accessed through `request.get::<Read<P>>()` as an `Arc<P::Value>`.
pub struct Read<P: Key> {
    data: Arc<P::Value>,
}

/// Middleware for data that persists between Requests for data which mostly
/// needs to be written instead of read.
///
/// The data is stored behind a `Mutex`, so only one request at a time can
/// access the data. This is more performant than `State` in the case where
/// most uses of the data require a write lock.
///
/// `Write` can be linked as `BeforeMiddleware` to add data to the `Request`
/// extensions and it can be linked as an `AfterMiddleware` to add data to
/// the `Response` extensions.
///
/// `Write` also implements `Plugin`, so the data stored within can be
/// accessed through `request.get::<Write<P>>()` as an `Arc<Mutex<P::Value>>`.
pub struct Write<P: Key> {
    data: Arc<Mutex<P::Value>>,
}

impl<P: Key> Clone for Read<P>
where
    P::Value: Send + Sync,
{
    fn clone(&self) -> Read<P> {
        Read {
            data: self.data.clone(),
        }
    }
}

impl<P: Key> Clone for State<P>
where
    P::Value: Send + Sync,
{
    fn clone(&self) -> State<P> {
        State {
            data: self.data.clone(),
        }
    }
}

impl<P: Key> Clone for Write<P>
where
    P::Value: Send + Sync,
{
    fn clone(&self) -> Write<P> {
        Write {
            data: self.data.clone(),
        }
    }
}

impl<P: Key> Key for State<P>
where
    P::Value: 'static,
{
    type Value = Arc<RwLock<P::Value>>;
}

impl<P: Key> Key for Read<P>
where
    P::Value: 'static,
{
    type Value = Arc<P::Value>;
}

impl<P: Key> Key for Write<P>
where
    P::Value: 'static,
{
    type Value = Arc<Mutex<P::Value>>;
}

impl<P: Key> Plugin<Context> for State<P>
where
    P::Value: Send + Sync,
{
    type Error = PersistentError;
    fn eval(req: &mut Context) -> Result<Arc<RwLock<P::Value>>, PersistentError> {
        req.extensions()
            .get::<State<P>>()
            .cloned()
            .ok_or(PersistentError::NotFound)
    }
}

impl<P: Key> Plugin<Context> for Read<P>
where
    P::Value: Send + Sync,
{
    type Error = PersistentError;
    fn eval(req: &mut Context) -> Result<Arc<P::Value>, PersistentError> {
        req.extensions()
            .get::<Read<P>>()
            .cloned()
            .ok_or(PersistentError::NotFound)
    }
}

impl<P: Key> Plugin<Context> for Write<P>
where
    P::Value: Send + Sync,
{
    type Error = PersistentError;
    fn eval(req: &mut Context) -> Result<Arc<Mutex<P::Value>>, PersistentError> {
        req.extensions()
            .get::<Write<P>>()
            .cloned()
            .ok_or(PersistentError::NotFound)
    }
}

impl<P: Key> Middleware for State<P>
where
    P::Value: Send + Sync,
{
    type Future = NextFuture;
    fn execute(&self, mut req: Context, next: Next) -> Self::Future {
        req.extensions_mut().insert::<State<P>>(self.data.clone());
        next.execute(req)
    }
}

impl<P: Key> Middleware for Read<P>
where
    P::Value: Send + Sync,
{
    type Future = NextFuture;
    fn execute(&self, mut req: Context, next: Next) -> Self::Future {
        req.extensions_mut().insert::<Read<P>>(self.data.clone());
        next.execute(req)
    }
}

impl<P: Key> Middleware for Write<P>
where
    P::Value: Send + Sync,
{
    type Future = NextFuture;
    fn execute(&self, mut req: Context, next: Next) -> Self::Future {
        req.extensions_mut().insert::<Write<P>>(self.data.clone());
        next.execute(req)
    }
}

impl<P: Key> State<P>
where
    P::Value: Send + Sync,
{
    /// Construct a new `State` middleware
    ///
    /// The data is initialized with the passed-in value.
    pub fn middleware<T>(start: T) -> State<P>
    where
        T: PersistentInto<Arc<RwLock<P::Value>>>,
    {
        State {
            data: start.persistent_into(),
        }
    }
}

impl<P: Key> Read<P>
where
    P::Value: Send + Sync,
{
    /// Construct a new `Read` middleware
    ///
    /// The data is initialized with the passed-in value.
    pub fn middleware<T>(start: T) -> Read<P>
    where
        T: PersistentInto<Arc<P::Value>>,
    {
        Read {
            data: start.persistent_into(),
        }
    }
}

impl<P: Key> Write<P>
where
    P::Value: Send + Sync,
{
    /// Construct a new `Write` middleware
    ///
    /// The data is initialized with the passed-in value.
    pub fn middleware<T>(start: T) -> Write<P>
    where
        T: PersistentInto<Arc<Mutex<P::Value>>>,
    {
        Write {
            data: start.persistent_into(),
        }
    }
}
