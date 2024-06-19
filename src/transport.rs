//! Transport trait for JSON RPC like protocol.
use crate::protocol::Protocol;

pub mod curl;

/// Communication Transport Layer contract.
pub trait Transport<P: Protocol> {
    fn new(info: &str, protocol: P) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Perform the call action with the transport layer implemented.
    fn call(&self, method: &str, request: &P::InnerType) -> anyhow::Result<P::InnerType>;
}
