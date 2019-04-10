use std::error::Error;
use std::fmt;
use std::io;
use std::result::Result;
use websocket::WebSocketError;

pub type JuntaResult<T> = Result<T, JuntaError>;

#[derive(Debug)]
pub enum JuntaErrorKind {
    Unknown,
    WebSocket(WebSocketError),
    Send,
    Io(io::Error),
    NotFound,
    MissingOption(String),
    InvalidAddress,
    Error(String)
}

#[derive(Debug)]
pub struct JuntaError {
    kind: JuntaErrorKind,
}

impl JuntaError {
    pub fn new(kind: JuntaErrorKind) -> JuntaError {
        JuntaError { kind }
    }

    pub fn kind(&self) -> &JuntaErrorKind {
        &self.kind
    }
}

impl fmt::Display for JuntaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JuntaError({:?})", self.kind)
    }
}

impl Error for JuntaError {}

impl From<JuntaErrorKind> for JuntaError {
    fn from(kind: JuntaErrorKind) -> JuntaError {
        JuntaError::new(kind)
    }
}

impl From<WebSocketError> for JuntaError {
    fn from(kind: WebSocketError) -> JuntaError {
        JuntaError::new(JuntaErrorKind::WebSocket(kind))
    }
}

impl From<io::Error> for JuntaError {
    fn from(error: io::Error) -> JuntaError {
        JuntaError::new(JuntaErrorKind::Io(error))
    }
}
