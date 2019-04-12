#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;

pub mod chain;
//pub mod context;
mod context_ext;
pub mod event;
mod middleware;
pub mod protocol;
pub mod protocol_ext;
pub mod request_protocol;
pub mod response_protocol;

pub mod prelude {
    pub use super::chain::*;
    pub use super::context_ext::*;
    pub use super::event::*;
    pub use super::middleware::*;
    pub use super::protocol::*;
    pub use super::protocol_ext::*;
    pub use super::request_protocol::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
