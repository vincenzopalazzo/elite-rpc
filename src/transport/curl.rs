//! Curl Transport Layer.
//!
//! Author <vincenzopalazzo@member.fsf.org>
use curl::easy::Easy;
use curl::Error;
use serde_json as json;

use crate::protocol::Protocol;
use crate::transport::Transport;

use super::TransportMethod;

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

        let mut list = curl::easy::List::new();
        list.append("Content-Type: application/json")?;

        // Set the content type header
        easy.http_headers(list)?;

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
        if response_code < 200 && response_code >= 300 {
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

    fn call(
        &self,
        method: TransportMethod,
        request: &P::InnerType,
    ) -> anyhow::Result<P::InnerType> {
        let response = match method {
            TransportMethod::Get(ref url) => {
                let (url, _) = self.protocol.to_request(url, request)?;
                self.raw_call(&url)?
            }
            TransportMethod::Post(ref url) => {
                let (url, request) = self.protocol.to_request(url, request)?;
                self.raw_post(&url, &json::to_vec(&request)?)?
            }
            TransportMethod::Custom(_, _) => {
                anyhow::bail!("Unsupported the custom transport method")
            }
        };
        self.protocol.from_request(&response, None)
    }
}
