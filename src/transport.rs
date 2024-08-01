//! Transport trait for JSON RPC like protocol.
use crate::protocol::Protocol;

#[cfg(feature = "curl")]
pub mod curl;

/// Transport Method that it is used by the
/// method to build the request.
pub enum TransportMethod {
    /// The Http post method, the string inside is the
    /// url or any other information that can be useful
    Post(String),
    /// The http get method, the string inside is the
    /// url or any other information that can be useful
    Get(String),
    /// A custom transport method, e.g: Unix Socket. The
    /// first string is an simple identifier, and the second is
    /// a url or a request identifier.
    Custom(String, String),
}

/// Communication Transport Layer contract.
pub trait Transport<P: Protocol> {
    fn new(info: &str, protocol: P) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Perform the call action with the transport layer implemented.
    fn call(&self, method: TransportMethod, request: &P::InnerType)
        -> anyhow::Result<P::InnerType>;
}
