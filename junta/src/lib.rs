#[macro_use]
extern crate slog;

mod client;
#[cfg(feature = "encoding")]
mod client_ext;
mod context;
mod error;
pub mod plugins;
mod server;
//mod utils;

pub mod prelude {
    pub use super::client::*;
    #[cfg(feature = "encoding")]
    pub use super::client_ext::*;
    pub use super::context::*;
    pub use super::error::*;
    pub use super::plugins;
    pub use super::server::*;
    pub use typemap::Key;
}
