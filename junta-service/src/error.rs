use std::error::Error;
use std::fmt;
pub enum ServiceError {
    ReceiverClosed,
    NullFuture,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServiceError")
    }
}
