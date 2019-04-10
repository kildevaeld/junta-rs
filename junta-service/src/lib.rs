mod client_ext;
mod context;
pub mod plugins;
mod service;

pub use client_ext::*;
pub use context::*;
pub use service::*;
pub use typemap::Key;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
