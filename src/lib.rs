//! Type Safe JSON RPC like client with different
//! transport layer.
//!
//! Author: Vincenzo Palazzo <vincenzopalazzo@member.fsf.org>
use std::marker::PhantomData;

pub use serde_json as json;

pub mod protocol;
pub mod transport;

use crate::protocol::Protocol;
use crate::transport::Transport;

pub struct EliteRPC<T: Transport<P>, P: Protocol> {
    transport: T,
    phantom: PhantomData<P>,
}

impl<P: Protocol, T: Transport<P>> EliteRPC<T, P> {
    pub fn new(info: &str) -> anyhow::Result<Self> {
        let protocol = P::new()?;
        let transport = T::new(info, protocol)?;
        Ok(Self {
            transport,
            phantom: PhantomData {},
        })
    }

    pub fn call(&self, method: &str, request: &P::InnerType) -> anyhow::Result<P::InnerType> {
        self.transport.call(method, request)
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::Protocol;
    use crate::transport::Transport;
    use crate::EliteRPC;

    pub struct MockTransport;
    #[derive(Clone)]
    pub struct MockProtocol;

    impl Transport<MockProtocol> for MockTransport {
        fn new(_: &str, _: MockProtocol) -> anyhow::Result<Self>
        where
            Self: Sized,
        {
            Ok(Self)
        }

        fn call(
            &self,
            _: &str,
            _: &<MockProtocol as Protocol>::InnerType,
        ) -> anyhow::Result<<MockProtocol as Protocol>::InnerType> {
            Ok(())
        }
    }

    impl Protocol for MockProtocol {
        type InnerType = ();

        fn from_from_request(
            &self,
            _: &[u8],
            _: Option<crate::protocol::Encoding>,
        ) -> anyhow::Result<Self::InnerType> {
            Ok(())
        }

        fn new() -> anyhow::Result<Self>
        where
            Self: Sized,
        {
            Ok(MockProtocol)
        }

        fn to_request(&self, _: &str, _: &Self::InnerType) -> anyhow::Result<Self::InnerType> {
            Ok(())
        }
    }

    #[test]
    pub fn test_init_example() -> anyhow::Result<()> {
        let rpc = EliteRPC::<MockTransport, MockProtocol>::new("")?;
        let response = rpc.call("foo", &())?;
        assert_eq!(response, ());
        Ok(())
    }
}
