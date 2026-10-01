//! `bitreq` transport layer.
//!
//! Drop-in replacement for the curl transport. `bitreq` is a
//! minimal-dependency HTTP client (the rust-bitcoin fork of minreq),
//! so callers that only need JSON-over-HTTP do not pull libcurl.
//!
//! Author <vincenzopalazzo@member.fsf.org>
use serde_json as json;

use crate::protocol::Protocol;
#[cfg(feature = "async")]
use crate::transport::AsyncTransport;
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

    fn url(&self, addons: &str) -> String {
        format!("{}/{addons}", self.base_url)
    }

    pub fn inner<D: serde::de::DeserializeOwned>(&self, addons: &str) -> anyhow::Result<D> {
        let body = self.raw_call(addons)?;
        let parsed_json: D = json::from_slice(&body)?;
        Ok(parsed_json)
    }

    pub fn raw_post(&self, addons: &str, body: &[u8]) -> anyhow::Result<Vec<u8>> {
        let response = bitreq::post(self.url(addons))
            .with_header("Content-Type", "application/json")
            .with_body(body.to_vec())
            .send()
            .map_err(|err| anyhow::anyhow!(err))?;
        check_status(&response)?;
        Ok(response.into_bytes())
    }

    pub fn raw_call(&self, addons: &str) -> anyhow::Result<Vec<u8>> {
        let response = bitreq::get(self.url(addons))
            .send()
            .map_err(|err| anyhow::anyhow!(err))?;
        check_status(&response)?;
        Ok(response.into_bytes())
    }
}

/// Accept the whole 2xx range, matching the curl transport after
/// `7ae1559` ("allow all the 2* range code to be Ok").
fn check_status(response: &bitreq::Response) -> anyhow::Result<()> {
    let code = response.status_code;
    if !(200..300).contains(&code) {
        let body = String::from_utf8_lossy(response.as_bytes());
        anyhow::bail!("http {code} {}: {body}", response.reason_phrase);
    }
    Ok(())
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

#[cfg(feature = "async")]
impl<P: Protocol + Sync> AsyncTransport<P> for HttpTransport<P>
where
    P::InnerType: Sync,
{
    async fn call_async(
        &self,
        method: TransportMethod,
        request: &P::InnerType,
    ) -> anyhow::Result<P::InnerType>
    where
        P::InnerType: Send,
    {
        let response = match method {
            TransportMethod::Get(ref url) => {
                let (url, _) = self.protocol.to_request(url, request)?;
                self.raw_call_async(&url).await?
            }
            TransportMethod::Post(ref url) => {
                let (url, request) = self.protocol.to_request(url, request)?;
                self.raw_post_async(&url, &json::to_vec(&request)?).await?
            }
            TransportMethod::Custom(_, _) => {
                anyhow::bail!("Unsupported the custom transport method")
            }
        };
        self.protocol.from_request(&response, None)
    }
}

#[cfg(feature = "async")]
impl<P: Protocol> HttpTransport<P> {
    pub async fn raw_post_async(&self, addons: &str, body: &[u8]) -> anyhow::Result<Vec<u8>> {
        let response = bitreq::post(self.url(addons))
            .with_header("Content-Type", "application/json")
            .with_body(body.to_vec())
            .send_async()
            .await
            .map_err(|err| anyhow::anyhow!(err))?;
        check_status(&response)?;
        Ok(response.into_bytes())
    }

    pub async fn raw_call_async(&self, addons: &str) -> anyhow::Result<Vec<u8>> {
        let response = bitreq::get(self.url(addons))
            .send_async()
            .await
            .map_err(|err| anyhow::anyhow!(err))?;
        check_status(&response)?;
        Ok(response.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Encoding;

    #[derive(Clone)]
    struct EchoProtocol;

    impl Protocol for EchoProtocol {
        type InnerType = json::Value;

        fn new() -> anyhow::Result<Self> {
            Ok(Self)
        }

        fn to_request(
            &self,
            method: &str,
            request: &Self::InnerType,
        ) -> anyhow::Result<(String, Self::InnerType)> {
            Ok((method.to_owned(), request.clone()))
        }

        fn from_request(
            &self,
            content: &[u8],
            _: Option<Encoding>,
        ) -> anyhow::Result<Self::InnerType> {
            Ok(json::from_slice(content)?)
        }
    }

    #[test]
    fn post_round_trips_json() -> anyhow::Result<()> {
        let server = tiny_http::Server::http("127.0.0.1:0").map_err(|err| anyhow::anyhow!(err))?;
        let addr = server
            .server_addr()
            .to_ip()
            .ok_or_else(|| anyhow::anyhow!("expected an IP listen addr"))?;
        let expected = json::json!({"ok": true, "n": 1});
        let payload = expected.to_string();
        let handle = std::thread::spawn(move || {
            let request = server.recv().expect("server recv");
            let response = tiny_http::Response::from_string(payload).with_status_code(200);
            request.respond(response).expect("server respond");
        });

        let transport =
            HttpTransport::build("http", "127.0.0.1", addr.port() as u64, EchoProtocol)?;
        let response = transport.call(
            TransportMethod::Post("echo".to_owned()),
            &json::json!({"ping": "pong"}),
        )?;
        handle.join().expect("server thread");
        assert_eq!(response, expected);
        Ok(())
    }

    #[test]
    fn non_2xx_is_an_error() -> anyhow::Result<()> {
        let server = tiny_http::Server::http("127.0.0.1:0").map_err(|err| anyhow::anyhow!(err))?;
        let addr = server
            .server_addr()
            .to_ip()
            .ok_or_else(|| anyhow::anyhow!("expected an IP listen addr"))?;
        let handle = std::thread::spawn(move || {
            let request = server.recv().expect("server recv");
            let response = tiny_http::Response::from_string("nope").with_status_code(500);
            request.respond(response).expect("server respond");
        });

        let transport = HttpTransport::<EchoProtocol>::build(
            "http",
            "127.0.0.1",
            addr.port() as u64,
            EchoProtocol,
        )?;
        let err = transport
            .raw_call("missing")
            .expect_err("500 must not be treated as success");
        handle.join().expect("server thread");
        let msg = err.to_string();
        assert!(msg.contains("500"), "{msg}");
        Ok(())
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn post_round_trips_json_async() -> anyhow::Result<()> {
        let server = tiny_http::Server::http("127.0.0.1:0").map_err(|err| anyhow::anyhow!(err))?;
        let addr = server
            .server_addr()
            .to_ip()
            .ok_or_else(|| anyhow::anyhow!("expected an IP listen addr"))?;
        let expected = json::json!({"ok": true, "async": true});
        let payload = expected.to_string();
        let handle = std::thread::spawn(move || {
            let request = server.recv().expect("server recv");
            let response = tiny_http::Response::from_string(payload).with_status_code(200);
            request.respond(response).expect("server respond");
        });

        let transport =
            HttpTransport::build("http", "127.0.0.1", addr.port() as u64, EchoProtocol)?;
        let response = crate::EliteRPC::from_transport(transport)
            .call_async(
                TransportMethod::Post("echo".to_owned()),
                &json::json!({"ping": "pong"}),
            )
            .await?;
        handle.join().expect("server thread");
        assert_eq!(response, expected);
        Ok(())
    }
}
