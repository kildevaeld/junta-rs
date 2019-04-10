#[macro_use]
extern crate serde_derive;

pub mod chain;
pub mod context;
pub mod event;
pub mod protocol;
pub mod protocol_ext;
pub mod request_protocol;
pub mod response_protocol;

pub mod prelude {
    pub use super::chain::*;
    pub use super::event::*;
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
