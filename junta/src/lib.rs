#[macro_use]
extern crate slog;

mod chain;
mod client;
#[cfg(feature = "encoding")]
mod client_ext;
mod context;
mod error;
mod handler;
pub mod plugins;
mod server;
mod utils;

pub mod prelude {
    pub use super::chain::*;
    pub use super::client::*;
    #[cfg(feature = "encoding")]
    pub use super::client_ext::*;
    pub use super::context::*;
    pub use super::error::*;
    pub use super::handler::*;
    pub use super::plugins;
    pub use super::server::*;
    pub use super::utils::*;
    pub use typemap::Key;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
