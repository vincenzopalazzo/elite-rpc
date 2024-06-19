//! Curl Transport Layer.
//!
//! Author <vincenzopalazzo@member.fsf.org>
use curl::easy::Easy;
use curl::Error;
use serde_json as json;

use crate::protocol::Protocol;
use crate::transport::Transport;

pub struct HttpTransport<P: Protocol> {
    base_url: String,
    protocol: P,
}

impl<P: Protocol> HttpTransport<P> {
    pub fn build(prefix: &str, host: &str, port: u64, protocol: P) -> anyhow::Result<Self> {
        Self::new(&format!("{prefix}://{host}:{port}"), protocol)
    }

    pub fn client(&self, addons: &str) -> anyhow::Result<Easy> {
        let mut easy = Easy::new();
        easy.url(&format!("{}/{addons}", self.base_url))?;
        Ok(easy)
    }

    pub fn inner<D: serde::de::DeserializeOwned>(&self, addons: &str) -> anyhow::Result<D> {
        let body = self.raw_call(addons)?;
        let parsed_json: D = json::from_slice(&body).map_err(|e| {
            let mut err = Error::new(400);
            err.set_extra(format!("{e}"));
            err
        })?;
        Ok(parsed_json)
    }

    pub fn raw_post(&self, addons: &str, body: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut easy = self.client(addons)?;
        easy.post(true)?;
        easy.post_fields_copy(body)?;

        let mut body = Vec::new();
        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                body.extend_from_slice(data);
                Ok(data.len())
            })?;

            transfer.perform()?;
        }

        let response_code = easy.response_code()?;

        // Check if the response code indicates an HTTP error
        if response_code != 200 {
            let mut err = Error::new(response_code);
            unsafe { err.set_extra(String::from_utf8_unchecked(body)) };
            anyhow::bail!(err);
        }
        Ok(body)
    }

    pub fn raw_call(&self, addons: &str) -> anyhow::Result<Vec<u8>> {
        let mut easy = self.client(addons)?;
        let mut body = Vec::new();
        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                body.extend_from_slice(data);
                Ok(data.len())
            })?;

            transfer.perform()?;
        }
        let response_code = easy.response_code()?;

        // Check if the response code indicates an HTTP error
        if response_code != 200 {
            let mut err = Error::new(response_code);
            unsafe { err.set_extra(String::from_utf8_unchecked(body)) };
            anyhow::bail!(err);
        }
        Ok(body)
    }
}

impl<P: Protocol> Transport<P> for HttpTransport<P> {
    fn new(info: &str, protocol: P) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            base_url: info.to_string(),
            protocol,
        })
    }

    fn call(&self, method: &str, request: &P::InnerType) -> anyhow::Result<P::InnerType> {
        let request = self.protocol.to_request(method, request)?;
        let response = self.raw_post("", &json::to_vec(&request)?)?;
        self.protocol.from_from_request(&response, None)
    }
}
