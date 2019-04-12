use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ServiceError {
    ReceiverClosed,
    NullFuture,
    InvalidRequest,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServiceError")
    }
}
