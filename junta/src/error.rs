use junta_service::error::ServiceError;
use std::error::Error;
use std::fmt;
use std::io;
use std::result::Result;
use websocket::WebSocketError;

pub type JuntaResult<T> = Result<T, JuntaError>;

#[cfg(feature = "encoding")]
#[derive(Debug)]
pub enum EncodingError {
    Binary(serde_cbor::error::Error),
    Text(serde_json::error::Error),
}

#[derive(Debug)]
pub enum JuntaErrorKind {
    Unknown(String),
    Send,
    Io(io::Error),
    NotFound,
    MissingOption(String),
    InvalidAddress,
    Service(ServiceError),
    Error(Box<Error + Sync + Send + 'static>),
    #[cfg(feature = "encoding")]
    Encoding(EncodingError),
    Transport(WebSocketError),
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
        JuntaError::new(JuntaErrorKind::Transport(kind))
    }
}

impl From<io::Error> for JuntaError {
    fn from(error: io::Error) -> JuntaError {
        JuntaError::new(JuntaErrorKind::Io(error))
    }
}

impl From<ServiceError> for JuntaError {
    fn from(error: ServiceError) -> JuntaError {
        JuntaError::new(JuntaErrorKind::Service(error))
    }
}

#[cfg(feature = "encoding")]
impl From<serde_json::Error> for JuntaError {
    fn from(error: serde_json::Error) -> JuntaError {
        JuntaError::new(JuntaErrorKind::Encoding(EncodingError::Text(error)))
    }
}

#[cfg(feature = "encoding")]
impl From<serde_cbor::error::Error> for JuntaError {
    fn from(error: serde_cbor::error::Error) -> JuntaError {
        JuntaError::new(JuntaErrorKind::Encoding(EncodingError::Binary(error)))
    }
}
