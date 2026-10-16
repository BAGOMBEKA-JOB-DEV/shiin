//! Runs the proxy against stdio and Streamable HTTP fixtures and the daemon service.

#[cfg(not(unix))]
fn main() {}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--reference-server") {
        return unix::reference_server().await;
    }
    tokio::time::timeout(std::time::Duration::from_secs(30), unix::exercise()).await?
}

#[cfg(unix)]
mod unix {
    use rmcp::model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig, Tool,
    };
    use rmcp::service::{ClientLifecycleMode, ClientServiceExt, RequestContext};
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };
    use rmcp::transport::{TokioChildProcess, stdio};
    use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
    use serde_json::json;
    use std::borrow::Cow;
    use std::fs::{self, OpenOptions};
    use std::io::Write as _;
    use std::path::PathBuf;
    use tokio::io::AsyncWriteExt as _;

    struct ReferenceServer(PathBuf);

    impl ServerHandler for ReferenceServer {
        fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
            Cow::Borrowed(&[ProtocolVersion::V_2026_07_28])
        }

        fn get_info(&self) -> ServerConfig {
            let mut config =
                ServerConfig::new(ServerCapabilities::builder().enable_tools().build());
            config.protocol_version = ProtocolVersion::V_2026_07_28;
            config
        }

        fn list_tools(
            &self,
            _: Option<PaginatedRequestParams>,
            _: RequestContext<RoleServer>,
        ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_
        {
            let schema = serde_json::Map::from_iter([("type".to_owned(), json!("object"))]);
            std::future::ready(Ok(ListToolsResult::with_all_items(
                ["echo", "hold", "denied"]
                    .into_iter()
                    .map(|name| Tool::new(name, "EXAMPLE fixture tool", schema.clone()))
                    .collect(),
            )))
        }

        fn call_tool(
            &self,
            request: CallToolRequestParams,
            _: RequestContext<RoleServer>,
        ) -> impl std::future::Future<Output = Result<CallToolResponse, ErrorData>> + Send + '_
        {
            let result = (|| -> Result<CallToolResponse, ErrorData> {
                let mut log = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.0)
                    .map_err(|_| ErrorData::internal_error("fixture log unavailable", None))?;
                writeln!(
                    log,
                    "{}",
                    serde_json::to_string(&request)
                        .map_err(|_| ErrorData::internal_error("fixture encoding failed", None))?
                )
                .map_err(|_| ErrorData::internal_error("fixture log unavailable", None))?;
                Ok(
                    CallToolResult::success(vec![ContentBlock::text("EXAMPLE downstream result")])
                        .into(),
                )
            })();
            std::future::ready(result)
        }
    }

    pub(super) async fn reference_server() -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::args_os()
            .nth(2)
            .ok_or("fixture log argument missing")?;
        ReferenceServer(PathBuf::from(path))
            .serve(stdio())
            .await?
            .waiting()
            .await?;
        Ok(())
    }

    struct Workspace(PathBuf);

    impl Drop for Workspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    async fn daemon(
        mut service: shiind::Service,
        listener: tokio::net::UnixListener,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let (stream, _) = listener.accept().await?;
            let (mut stream, frame) = shiin_ipc::receive(stream).await?;
            let request: shiin_ipc::Request = serde_json::from_str(&frame)?;
            let response = match service.dispatch(&request.method, request.params) {
                Ok(result) => json!({"jsonrpc":"2.0", "id":request.id, "result":result}),
                Err(_) => shiin_ipc::failure(&request.id, -32000, "Request refused"),
            };
            stream.write_all(format!("{response}\n").as_bytes()).await?;
        }
    }

    pub(super) async fn exercise() -> Result<(), Box<dyn std::error::Error>> {
        exercise_transport(Transport::Stdio).await?;
        exercise_transport(Transport::Http).await?;
        exercise_transport(Transport::HttpsBearer).await?;
        configuration_errors_do_not_disclose_credentials().await?;
        redirects_are_not_followed().await
    }

    enum Transport {
        Stdio,
        Http,
        HttpsBearer,
    }

    async fn exercise_transport(transport: Transport) -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("sm-{}", shiin_schema::clock::new_id()));
        fs::create_dir_all(root.join(".shiin"))?;
        fs::create_dir_all(root.join("daemon"))?;
        let workspace = Workspace(fs::canonicalize(root)?);
        fs::write(
            workspace.0.join(".shiin/policy.toml"),
            "version = 1\n[capabilities.EXAMPLE]\nid = 'EXAMPLE'\nallow = true\naction_types = ['*']\n[defaults]\ndecision = 'pending'\n[[rule]]\nid = 'echo'\neffect = 'allow'\naction_types = ['mcp.tool.echo']\n[[rule]]\nid = 'denied'\neffect = 'deny'\naction_types = ['mcp.tool.denied']\n",
        )?;
        let socket = workspace.0.join("s");
        let listener = tokio::net::UnixListener::bind(&socket)?;
        let service = shiind::Service::open(
            &workspace.0.join("daemon"),
            shiind::OperatorKey::generate()?,
        )?;
        let daemon = tokio::spawn(daemon(service, listener));
        let forwarded = workspace.0.join("forwarded.jsonl");
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_shiin-mcp-proxy"));
        command
            .arg("--workspace")
            .arg(&workspace.0)
            .env("SHIIN_SOCKET", &socket);
        let http_server = if matches!(transport, Transport::Stdio) {
            command
                .arg("--")
                .arg(std::env::current_exe()?)
                .arg("--reference-server")
                .arg(&forwarded);
            None
        } else {
            Some(
                http_fixture(
                    &mut command,
                    forwarded.clone(),
                    matches!(transport, Transport::HttpsBearer),
                )
                .await?,
            )
        };
        let transport = TokioChildProcess::new(command)?;
        let client = ()
            .serve_with_lifecycle(
                transport,
                ClientLifecycleMode::Discover {
                    preferred_versions: vec![ProtocolVersion::V_2026_07_28],
                },
            )
            .await?;
        let tools = client.list_tools(None).await?;
        assert_eq!(
            tools
                .tools
                .iter()
                .map(|tool| tool.name.as_ref())
                .collect::<Vec<_>>(),
            ["echo", "hold", "denied"]
        );
        let allowed = client
            .call_tool(CallToolRequestParams::new("echo").with_arguments(
                serde_json::Map::from_iter([("value".to_owned(), json!("EXAMPLE-argument"))]),
            ))
            .await?;
        assert_ne!(allowed.is_error, Some(true));
        for tool in ["hold", "denied"] {
            let refused = client.call_tool(CallToolRequestParams::new(tool)).await?;
            assert_eq!(refused.is_error, Some(true), "{tool} reached downstream");
            if tool == "hold" {
                assert!(serde_json::to_string(&refused)?.contains("Ticket:"));
            }
        }
        daemon.abort();
        let _ = daemon.await;
        let unavailable = client.call_tool(CallToolRequestParams::new("echo")).await?;
        assert_eq!(unavailable.is_error, Some(true));
        client.cancel().await?;
        let calls = fs::read_to_string(forwarded)?;
        assert_eq!(calls.lines().count(), 1, "only allow may forward a call");
        assert!(calls.contains("EXAMPLE-argument"));
        assert!(
            !fs::read_to_string(workspace.0.join(".shiin/audit.jsonl"))?
                .contains("EXAMPLE-argument")
        );
        if let Some(server) = http_server {
            server.abort();
            let _ = server.await;
        }
        Ok(())
    }

    async fn http_fixture(
        command: &mut tokio::process::Command,
        log: PathBuf,
        authenticated: bool,
    ) -> Result<tokio::task::JoinHandle<std::io::Result<()>>, Box<dyn std::error::Error>> {
        let directory = log.parent().ok_or("fixture directory missing")?.to_owned();
        let service: StreamableHttpService<ReferenceServer, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(ReferenceServer(log.clone())),
                std::sync::Arc::default(),
                StreamableHttpServerConfig::default().with_legacy_session_mode(false),
            );
        let router = axum::Router::new().nest_service("/mcp", service);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        if authenticated {
            return tls_fixture(command, listener, router, &directory);
        }
        command
            .arg("--http")
            .arg(format!("http://{}/mcp", listener.local_addr()?));
        Ok(tokio::spawn(
            async move { axum::serve(listener, router).await },
        ))
    }

    fn tls_fixture(
        command: &mut tokio::process::Command,
        listener: tokio::net::TcpListener,
        router: axum::Router,
        directory: &std::path::Path,
    ) -> Result<tokio::task::JoinHandle<std::io::Result<()>>, Box<dyn std::error::Error>> {
        let certificate = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()])?;
        let ca = directory.join("EXAMPLE-ca.pem");
        fs::write(&ca, certificate.cert.pem())?;
        let key =
            rustls::pki_types::PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.cert.der().clone()], key.into())?;
        let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(config));
        let router = router.layer(axum::middleware::from_fn(
            |request: axum::extract::Request, next: axum::middleware::Next| async move {
                if request
                    .headers()
                    .get(reqwest::header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
                    != Some("Bearer EXAMPLE-fixture-token")
                {
                    return Err(axum::http::StatusCode::UNAUTHORIZED);
                }
                Ok::<_, axum::http::StatusCode>(next.run(request).await)
            },
        ));
        command
            .arg("--http")
            .arg(format!(
                "https://localhost:{}/mcp",
                listener.local_addr()?.port()
            ))
            .arg("--ca-file")
            .arg(ca)
            .arg("--bearer-env")
            .arg("SHIIN_EXAMPLE_MCP_TOKEN")
            .env("SHIIN_EXAMPLE_MCP_TOKEN", "EXAMPLE-fixture-token");
        Ok(tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await?;
                let acceptor = acceptor.clone();
                let router = router.clone();
                tokio::spawn(async move {
                    if let Ok(tls) = acceptor.accept(stream).await {
                        let io = hyper_util::rt::TokioIo::new(tls);
                        let service = hyper_util::service::TowerToHyperService::new(router);
                        let _ = hyper_util::server::conn::auto::Builder::new(
                            hyper_util::rt::TokioExecutor::new(),
                        )
                        .serve_connection_with_upgrades(io, service)
                        .await;
                    }
                });
            }
        }))
    }

    async fn redirects_are_not_followed() -> Result<(), Box<dyn std::error::Error>> {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        let followed = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&followed);
        let router = axum::Router::new()
            .route(
                "/redirect",
                axum::routing::any(|| async { axum::response::Redirect::temporary("/trap") }),
            )
            .route(
                "/trap",
                axum::routing::any(move || {
                    observed.fetch_add(1, Ordering::SeqCst);
                    async { "EXAMPLE redirect target" }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/redirect", listener.local_addr()?);
        let server = tokio::spawn(async move { axum::serve(listener, router).await });
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_shiin-mcp-proxy"));
        command
            .arg("--workspace")
            .arg(std::env::current_dir()?)
            .arg("--http")
            .arg(url)
            .env("SHIIN_DAEMON", "1")
            .env_remove("SHIIN_SOCKET");
        let result = ()
            .serve_with_lifecycle(
                TokioChildProcess::new(command)?,
                ClientLifecycleMode::Discover {
                    preferred_versions: vec![ProtocolVersion::V_2026_07_28],
                },
            )
            .await;
        assert!(result.is_err(), "redirect must fail connection setup");
        assert_eq!(followed.load(Ordering::SeqCst), 0);
        server.abort();
        let _ = server.await;
        Ok(())
    }

    async fn configuration_errors_do_not_disclose_credentials()
    -> Result<(), Box<dyn std::error::Error>> {
        for (endpoint, variable, value) in [
            (
                "http://127.0.0.1/mcp",
                "SHIIN_EXAMPLE_TOKEN",
                Some("EXAMPLE-private-token"),
            ),
            ("https://mcp.example/mcp", "SHIIN_EXAMPLE_TOKEN", None),
            (
                "https://mcp.example/mcp",
                "SHIIN_EXAMPLE_TOKEN",
                Some("EXAMPLE-private token"),
            ),
        ] {
            let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_shiin-mcp-proxy"));
            command
                .arg("--workspace")
                .arg(std::env::current_dir()?)
                .arg("--http")
                .arg(endpoint)
                .arg("--bearer-env")
                .arg(variable)
                .env("SHIIN_DAEMON", "1")
                .env_remove("SHIIN_SOCKET")
                .env_remove(variable);
            if let Some(value) = value {
                command.env(variable, value);
            }
            let output = command.output().await?;
            assert!(!output.status.success());
            assert!(!String::from_utf8_lossy(&output.stderr).contains("EXAMPLE-private"));
            assert!(output.stdout.is_empty());
        }
        Ok(())
    }
}
