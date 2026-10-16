//! Explicit downstream transport selection with bounded connection setup.

use rmcp::service::{ClientLifecycleMode, ClientServiceExt};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::{StreamableHttpClientTransport, TokioChildProcess};
use std::{ffi::OsString, io::Read as _, net::IpAddr, path::PathBuf, time::Duration};

pub(super) enum Endpoint {
    Stdio {
        command: PathBuf,
        arguments: Vec<OsString>,
    },
    Http {
        url: reqwest::Url,
        authorization: Option<reqwest::header::HeaderValue>,
        ca: Option<reqwest::Certificate>,
    },
}

impl Endpoint {
    pub(super) fn with_bearer_env(self, variable: &str) -> Result<Self, String> {
        let Self::Http { url, ca, .. } = self else {
            return Err("bearer authentication requires an HTTP endpoint".to_owned());
        };
        if url.scheme() != "https" {
            return Err(
                "bearer authentication requires HTTPS, including loopback endpoints".to_owned(),
            );
        }
        if variable.is_empty()
            || !variable
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err("invalid credential environment variable name".to_owned());
        }
        let token = zeroize::Zeroizing::new(
            std::env::var(variable)
                .map_err(|_| "credential environment variable is unavailable")?,
        );
        let authorization = authorization_header(&token)?;
        Ok(Self::Http {
            url,
            authorization: Some(authorization),
            ca,
        })
    }

    pub(super) fn with_ca_file(self, path: &std::path::Path) -> Result<Self, String> {
        let Self::Http {
            url, authorization, ..
        } = self
        else {
            return Err("custom CA requires an HTTP endpoint".to_owned());
        };
        if url.scheme() != "https" || !path.is_absolute() {
            return Err("custom CA requires HTTPS and an absolute certificate path".to_owned());
        }
        let file = std::fs::File::open(path).map_err(|_| "custom CA file is unavailable")?;
        let mut bytes = Vec::new();
        file.take(65_537)
            .read_to_end(&mut bytes)
            .map_err(|_| "custom CA file could not be read")?;
        if bytes.len() > 65_536 {
            return Err("custom CA file exceeds size limit".to_owned());
        }
        let ca = reqwest::Certificate::from_pem(&bytes)
            .map_err(|_| "custom CA certificate is invalid")?;
        Ok(Self::Http {
            url,
            authorization,
            ca: Some(ca),
        })
    }

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
                Self::Http {
                    url,
                    authorization,
                    ca,
                } => {
                    let mut headers = reqwest::header::HeaderMap::new();
                    if let Some(authorization) = authorization {
                        headers.insert(reqwest::header::AUTHORIZATION, authorization);
                    }
                    let mut builder = reqwest::Client::builder()
                        .default_headers(headers)
                        .redirect(reqwest::redirect::Policy::none())
                        .retry(reqwest::retry::never())
                        .no_proxy()
                        .connect_timeout(Duration::from_secs(5))
                        .timeout(Duration::from_secs(60));
                    if let Some(ca) = ca {
                        builder = builder.add_root_certificate(ca);
                    }
                    let client = builder
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
    Ok(Endpoint::Http {
        url,
        authorization: None,
        ca: None,
    })
}

fn authorization_header(token: &str) -> Result<reqwest::header::HeaderValue, String> {
    let unpadded = token.trim_end_matches('=');
    if unpadded.is_empty()
        || token.len() > 4096
        || !unpadded
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/".contains(&byte))
    {
        return Err("invalid bearer credential".to_owned());
    }
    let mut header = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|_| "invalid bearer credential")?;
    header.set_sensitive(true);
    Ok(header)
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

    #[test]
    fn credentials_are_validated_and_redacted_from_debug_output() {
        let token = "EXAMPLE-credential_123";
        let header = super::authorization_header(token).expect("valid fixture credential");
        assert_eq!(
            header.to_str().expect("ASCII header"),
            format!("Bearer {token}")
        );
        assert!(header.is_sensitive());
        assert!(!format!("{header:?}").contains(token));
        for token in [
            "",
            "EXAMPLE secret",
            "EXAMPLE\r\nInjected: true",
            "EXAMPLE-é",
            "EXAMPLE=middle",
        ] {
            assert!(super::authorization_header(token).is_err());
        }
        assert!(super::authorization_header(&"a".repeat(4097)).is_err());
        assert!(
            http_endpoint("http://127.0.0.1/mcp")
                .expect("loopback endpoint")
                .with_bearer_env("EXAMPLE_TOKEN")
                .is_err()
        );
    }
}
