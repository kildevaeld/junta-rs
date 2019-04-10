use super::chain::ProtocolChain;
use super::protocol::{IntoProtocol, Protocol};

pub trait ProtocolExt: Protocol + Sized {
    fn or<S: IntoProtocol>(self, protocol: S) -> ProtocolChain<Self, S::Protocol> {
        ProtocolChain::new(self, protocol.into_protocol())
    }
}

impl<T> ProtocolExt for T where T: Protocol {}
