//! MCP proxy that authorizes each downstream tool call through Shiin.

#![expect(
    clippy::print_stderr,
    reason = "startup and transport failures must be visible to the operator"
)]

use rmcp::{
    ErrorData as McpError, RoleClient, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig,
    },
    service::{RequestContext, ServiceExt},
    transport::stdio,
};
use sha2::{Digest, Sha256};
use shiin_client::Client;
use shiin_schema::{
    ActionRequest, Agent, Assurance, Confidence, DecisionValue, Effect, Host, Workspace, clock,
};
use std::{borrow::Cow, env, path::PathBuf, process::ExitCode};

mod downstream;

type Downstream = rmcp::service::RunningService<RoleClient, ()>;

const SUPPORTED_PROTOCOL_VERSIONS: &[ProtocolVersion] = &[ProtocolVersion::V_2026_07_28];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("shiin-mcp-proxy: {error}");
            ExitCode::FAILURE
        }
    }
}

#[tokio::main]
async fn run() -> Result<(), String> {
    let (workspace, endpoint) = parse_args()?;
    if !shiin_ipc_enabled() {
        return Err("configure SHIIN_DAEMON=1 or an absolute SHIIN_SOCKET".to_owned());
    }
    let client = Client::from_environment().map_err(|_| "local daemon configuration failed")?;
    let downstream = endpoint.connect().await?;
    let session = clock::new_id();
    if session.is_empty() {
        return Err("OS randomness failed".to_owned());
    }
    let service = Proxy {
        downstream,
        client,
        workspace,
        session,
    }
    .serve(stdio())
    .await
    .map_err(|_| "could not start MCP stdio service")?;
    service
        .waiting()
        .await
        .map_err(|_| "MCP stdio service stopped unexpectedly".to_owned())?;
    Ok(())
}

fn shiin_ipc_enabled() -> bool {
    env::var_os("SHIIN_SOCKET").is_some()
        || env::var("SHIIN_DAEMON").is_ok_and(|value| value == "1")
}

fn parse_args() -> Result<(String, downstream::Endpoint), String> {
    let mut args = env::args_os().skip(1);
    let mut workspace = None;
    let mut command = None;
    let mut downstream_args = Vec::new();
    let mut http = None;
    while let Some(argument) = args.next() {
        if argument == "--" {
            if http.is_some() {
                return Err("HTTP and subprocess transports are mutually exclusive".to_owned());
            }
            command = Some(PathBuf::from(
                args.next().ok_or("downstream server command missing")?,
            ));
            downstream_args.extend(args);
            break;
        }
        if argument == "--workspace" {
            if workspace.is_some() {
                return Err("--workspace may be specified only once".to_owned());
            }
            workspace = Some(PathBuf::from(
                args.next().ok_or("--workspace requires a path")?,
            ));
            continue;
        }
        if argument == "--http" && http.is_none() {
            let value = args.next().ok_or("--http requires an endpoint")?;
            http = Some(downstream::http_endpoint(
                value.to_str().ok_or("endpoint must be UTF-8")?,
            )?);
            continue;
        }
        return Err(
            "usage: shiin-mcp-proxy --workspace PATH (-- SERVER [ARG ...] | --http URL)".to_owned(),
        );
    }
    let workspace = workspace.ok_or("--workspace is required")?;
    let workspace = std::fs::canonicalize(workspace).map_err(|_| "workspace is unavailable")?;
    if !workspace.is_dir() {
        return Err("workspace must be a directory".to_owned());
    }
    let endpoint = match (command, http) {
        (Some(command), None) if command.is_absolute() => downstream::Endpoint::Stdio {
            command,
            arguments: downstream_args,
        },
        (None, Some(http)) => http,
        _ => {
            return Err(
                "select exactly one HTTP endpoint or absolute downstream server command".to_owned(),
            );
        }
    };
    let workspace = workspace
        .into_os_string()
        .into_string()
        .map_err(|_| "workspace must be UTF-8")?;
    Ok((workspace, endpoint))
}

struct Proxy {
    downstream: Downstream,
    client: Client,
    workspace: String,
    session: String,
}

impl ServerHandler for Proxy {
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(SUPPORTED_PROTOCOL_VERSIONS)
    }

    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        self.downstream
            .list_tools(request)
            .await
            .map_err(|_| McpError::internal_error("downstream MCP server failed", None))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let Ok(action) = action_request(&request, &self.workspace, &self.session) else {
            return Ok(tool_error(
                "Shiin could not construct a safe authorization request.",
            ));
        };
        let Ok(decision) = self.client.evaluate(&action).await else {
            return Ok(tool_error(
                "Shiin could not authorize this MCP tool call. The action was stopped.",
            ));
        };
        if !should_forward(decision.decision) {
            let ticket = decision
                .approval
                .as_ref()
                .map(|approval| format!(" Ticket: {}", approval.ticket))
                .unwrap_or_default();
            return Ok(tool_error(&format!("{}{}", decision.message, ticket)));
        }
        self.downstream
            .call_tool(request)
            .await
            .map(Into::into)
            .map_err(|_| McpError::internal_error("downstream MCP server failed", None))
    }
}

fn tool_error(message: &str) -> CallToolResponse {
    CallToolResult::error(vec![ContentBlock::text(message.to_owned())]).into()
}

fn should_forward(decision: DecisionValue) -> bool {
    decision == DecisionValue::Allow
}

fn action_request(
    tool: &CallToolRequestParams,
    workspace: &str,
    session: &str,
) -> Result<ActionRequest, String> {
    let policy_path = PathBuf::from(workspace).join(".shiin/policy.toml");
    let policy = std::fs::read_to_string(policy_path).map_err(|_| "policy unavailable")?;
    let source = shiin_policy::toml_front_end::parse(&policy).map_err(|_| "policy invalid")?;
    let policy_snapshot = source
        .layers
        .first()
        .ok_or("policy snapshot missing")?
        .digest
        .clone();
    action_request_for(tool, workspace, session, &policy_snapshot)
}

fn action_request_for(
    tool: &CallToolRequestParams,
    workspace: &str,
    session: &str,
    policy_snapshot: &str,
) -> Result<ActionRequest, String> {
    let id = clock::new_id();
    if id.is_empty() {
        return Err("OS randomness failed".to_owned());
    }
    let invocation =
        serde_json::to_vec(tool).map_err(|_| "tool invocation could not be encoded")?;
    let digest = format!("sha256:{:x}", Sha256::digest(invocation));
    let action_type = tool_action_type(&tool.name)?;
    let request = ActionRequest {
        schema: "shiin.action-request/v1-draft.1".to_owned(),
        id,
        session: session.to_owned(),
        agent: Agent {
            id: "mcp-client".to_owned(),
            assurance: Assurance::Asserted,
        },
        action_type,
        effects: vec![Effect {
            kind: "mcp_tool".to_owned(),
            value: digest,
            confidence: Confidence::Opaque,
            action_type: None,
        }],
        task: None,
        workspace: Workspace {
            root: workspace.to_owned(),
            vcs: None,
        },
        policy_snapshot: policy_snapshot.to_owned(),
        host: Host {
            name: "mcp-proxy".to_owned(),
            tool: Some(tool.name.clone().into_owned()),
        },
    };
    request.validate().map_err(str::to_owned)?;
    Ok(request)
}

fn tool_action_type(name: &str) -> Result<String, String> {
    if name.is_empty() || name.len() > 256 {
        return Err("invalid MCP tool name".to_owned());
    }
    let mut action_type = String::from("mcp.tool.");
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-') {
            action_type.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(action_type, "_{byte:02x}").map_err(|_| "invalid MCP tool name")?;
        }
    }
    Ok(action_type)
}

#[cfg(test)]
mod tests {
    use super::{action_request_for, should_forward, tool_action_type};
    use rmcp::model::CallToolRequestParams;
    use serde_json::json;

    #[test]
    fn tool_arguments_are_opaque_and_bound_without_logging_plaintext() {
        let tool = CallToolRequestParams::new("delete_record").with_arguments(
            json!({"record":"EXAMPLE-sensitive-value"})
                .as_object()
                .expect("arguments are an object")
                .clone(),
        );
        let request = action_request_for(
            &tool,
            "/tmp/EXAMPLE",
            "0190f3a0-1c2d-4e3f-8a4b-5c6d7e8f9a0b",
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        )
        .expect("valid normalized action");
        assert_eq!(request.action_type, "mcp.tool.delete_record");
        assert_eq!(
            request.effects[0].confidence,
            shiin_schema::Confidence::Opaque
        );
        assert!(request.effects[0].value.starts_with("sha256:"));
        assert!(!request.effects[0].value.contains("sensitive-value"));
        assert_eq!(request.host.tool.as_deref(), Some("delete_record"));
    }

    #[test]
    fn tool_names_cannot_add_policy_wildcards() {
        assert_eq!(
            tool_action_type("db.read*"),
            Ok("mcp.tool.db.read_2a".to_owned())
        );
        assert!(tool_action_type("").is_err());
    }

    #[test]
    fn only_allow_reaches_the_downstream_server() {
        assert!(should_forward(shiin_schema::DecisionValue::Allow));
        assert!(!should_forward(shiin_schema::DecisionValue::Deny));
        assert!(!should_forward(shiin_schema::DecisionValue::Pending));
    }
}
