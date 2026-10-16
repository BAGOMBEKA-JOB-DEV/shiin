//! Explicit downstream transport selection with bounded connection setup.

use rmcp::service::{ClientLifecycleMode, ClientServiceExt};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::{StreamableHttpClientTransport, TokioChildProcess};
use std::{ffi::OsString, net::IpAddr, path::PathBuf, time::Duration};

pub(super) enum Endpoint {
    Stdio {
        command: PathBuf,
        arguments: Vec<OsString>,
    },
    Http(reqwest::Url),
}

impl Endpoint {
    pub(super) async fn connect(self) -> Result<super::Downstream, String> {
        let lifecycle = ClientLifecycleMode::Discover {
            preferred_versions: vec![rmcp::model::ProtocolVersion::V_2026_07_28],
        };
        tokio::time::timeout(Duration::from_secs(10), async {
            match self {
                Self::Stdio { command, arguments } => {
                    let mut child = tokio::process::Command::new(command);
                    child.args(arguments);
                    let transport = TokioChildProcess::new(child)
                        .map_err(|_| "could not start downstream MCP server")?;
                    ().serve_with_lifecycle(transport, lifecycle)
                        .await
                        .map_err(|_| "could not connect to downstream MCP server")
                }
                Self::Http(url) => {
                    let client = reqwest::Client::builder()
                        .redirect(reqwest::redirect::Policy::none())
                        .retry(reqwest::retry::never())
                        .no_proxy()
                        .connect_timeout(Duration::from_secs(5))
                        .timeout(Duration::from_secs(60))
                        .build()
                        .map_err(|_| "could not configure downstream HTTP transport")?;
                    let transport = StreamableHttpClientTransport::with_client(
                        client,
                        StreamableHttpClientTransportConfig::with_uri(url.as_str())
                            .reinit_on_expired_session(false),
                    );
                    ().serve_with_lifecycle(transport, lifecycle)
                        .await
                        .map_err(|_| "could not connect to downstream MCP server")
                }
            }
        })
        .await
        .map_err(|_| "downstream MCP connection timed out".to_owned())?
        .map_err(str::to_owned)
    }
}

pub(super) fn http_endpoint(value: &str) -> Result<Endpoint, String> {
    let url = reqwest::Url::parse(value).map_err(|_| "invalid downstream HTTP endpoint")?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.host_str().is_none()
    {
        return Err(
            "endpoint requires a host and cannot contain credentials, query, or fragment"
                .to_owned(),
        );
    }
    let loopback = url.host_str().is_some_and(|host| {
        host.trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
    });
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err("downstream HTTP requires HTTPS or a literal loopback address".to_owned());
    }
    Ok(Endpoint::Http(url))
}

#[cfg(test)]
mod tests {
    use super::http_endpoint;

    #[test]
    fn endpoints_require_tls_except_for_literal_loopback() {
        for endpoint in [
            "https://mcp.example/mcp",
            "http://127.0.0.1:4312/mcp",
            "http://[::1]:4312/mcp",
        ] {
            assert!(http_endpoint(endpoint).is_ok(), "{endpoint}");
        }
        for endpoint in [
            "http://mcp.example/mcp",
            "http://localhost/mcp",
            "file:///tmp/mcp",
            "https://EXAMPLE:mypass@mcp.example/mcp",
            "https://mcp.example/mcp?token=EXAMPLE",
            "https://mcp.example/mcp#EXAMPLE",
        ] {
            assert!(http_endpoint(endpoint).is_err(), "{endpoint}");
        }
    }
}
