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
        exercise_transport(false).await?;
        exercise_transport(true).await?;
        redirects_are_not_followed().await
    }

    async fn exercise_transport(http: bool) -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("sm-{}", shiin_schema::clock::new_id()));
        fs::create_dir_all(root.join(".shiin"))?;
        fs::create_dir_all(root.join("daemon"))?;
        let workspace = Workspace(fs::canonicalize(root)?);
        fs::write(
            workspace.0.join(".shiin/policy.toml"),
            "version = 1\n[defaults]\ndecision = 'pending'\n[[rule]]\nid = 'echo'\neffect = 'allow'\naction_types = ['mcp.tool.echo']\n[[rule]]\nid = 'denied'\neffect = 'deny'\naction_types = ['mcp.tool.denied']\n",
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
        let http_server = if http {
            Some(http_fixture(&mut command, forwarded.clone()).await?)
        } else {
            command
                .arg("--")
                .arg(std::env::current_exe()?)
                .arg("--reference-server")
                .arg(&forwarded);
            None
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
    ) -> Result<tokio::task::JoinHandle<std::io::Result<()>>, Box<dyn std::error::Error>> {
        let service: StreamableHttpService<ReferenceServer, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(ReferenceServer(log.clone())),
                Default::default(),
                StreamableHttpServerConfig::default().with_legacy_session_mode(false),
            );
        let router = axum::Router::new().nest_service("/mcp", service);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        command
            .arg("--http")
            .arg(format!("http://{}/mcp", listener.local_addr()?));
        Ok(tokio::spawn(
            async move { axum::serve(listener, router).await },
        ))
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
}
