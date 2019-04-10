#[macro_use]
extern crate slog;

mod client;
mod context;
mod error;
mod handler;
mod plugins;
mod server;
mod utils;

pub mod prelude {
    pub use super::client::*;
    pub use super::error::*;
    pub use super::handler::*;
    pub use super::server::*;
    pub use super::utils::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
