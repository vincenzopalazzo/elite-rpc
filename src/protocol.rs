//! Contrct definition for the protocol
//!
//! Author: Vincenzo Palazzo <vincenzopalazzo@member.fsf.org>
use serde::Serialize;

/// Type of Encoding.
///
/// FIXME: It is not supporting all the type of encoding
/// but some of them.
pub enum Encoding {
    UTF8,
}

/// Contract definition of the protocol.
pub trait Protocol: Clone {
    type InnerType: Serialize;

    /// Create a new instance of the protocol.
    fn new() -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Build the Request for this protocol.
    fn to_request(
        &self,
        method: &str,
        request: &Self::InnerType,
    ) -> anyhow::Result<Self::InnerType>;

    /// Build the response fom the response
    fn from_from_request(
        &self,
        content: &[u8],
        encoding: Option<Encoding>,
    ) -> anyhow::Result<Self::InnerType>;
}
