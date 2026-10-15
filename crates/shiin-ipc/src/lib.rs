//! Internal Shiin API; it can change in any release.
//!
//! Local-only, bounded JSON-RPC. Unix peers must have the same effective UID.
//! Unsupported transports and unreachable services are errors, never fallbacks.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[cfg(unix)]
use shiin_schema::clock;
#[cfg(unix)]
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Maximum serialized frame, including its terminating newline.
pub const MAX_FRAME: usize = 65_536;
/// Deadline for a complete client exchange.
pub const TIMEOUT: Duration = Duration::from_millis(1500);

/// One request. No caller-supplied role or identity is trusted.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Exactly `2.0`.
    pub jsonrpc: String,
    /// Correlation identifier.
    pub id: Value,
    /// Method selected by the caller.
    pub method: String,
    /// Method arguments.
    pub params: Value,
}

/// The per-user runtime endpoint, optionally explicitly configured by the operator.
///
/// # Errors
/// Relative overrides and missing home directories are rejected.
pub fn endpoint() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("SHIIN_SOCKET") {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err("SHIIN_SOCKET must be absolute".to_owned());
        }
        return Ok(path);
    }
    let home = std::env::var_os("HOME").ok_or("HOME is required for local daemon state")?;
    Ok(PathBuf::from(home).join(".shiin/runtime/shiind.sock"))
}

/// Whether this invocation explicitly requires the daemon.
#[must_use]
pub fn enabled() -> bool {
    std::env::var_os("SHIIN_SOCKET").is_some()
        || std::env::var("SHIIN_DAEMON").is_ok_and(|value| value == "1")
}

/// Sends one RPC without exposing async runtime types to callers.
///
/// # Errors
/// Authentication, framing, timeouts, protocol errors, and unsupported platforms fail closed.
pub fn call(method: &str, params: Value) -> Result<Value, String> {
    let path = endpoint()?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(async {
        tokio::time::timeout(TIMEOUT, exchange(&path, method, params))
            .await
            .map_err(|_| "local daemon deadline exceeded".to_owned())?
    })
}

#[cfg(unix)]
async fn exchange(path: &Path, method: &str, params: Value) -> Result<Value, String> {
    use tokio::io::AsyncWriteExt;
    let stream = tokio::net::UnixStream::connect(path)
        .await
        .map_err(|error| error.to_string())?;
    if stream.peer_cred().map_err(|error| error.to_string())?.uid()
        != own_uid().map_err(|error| error.to_string())?
    {
        return Err("daemon peer does not belong to this user".to_owned());
    }
    let id = clock::new_id();
    if id.is_empty() {
        return Err("OS randomness failed".to_owned());
    }
    let request = Request {
        jsonrpc: "2.0".to_owned(),
        id: json!(id),
        method: method.to_owned(),
        params,
    };
    let mut frame = serde_json::to_vec(&request).map_err(|error| error.to_string())?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME {
        return Err("RPC request exceeds frame limit".to_owned());
    }
    let mut stream = stream;
    stream
        .write_all(&frame)
        .await
        .map_err(|error| error.to_string())?;
    let (_, text) = receive(stream).await.map_err(|error| error.to_string())?;
    let response: Value = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    if response["jsonrpc"] != "2.0"
        || response["id"] != request.id
        || response.get("result").is_some() == response.get("error").is_some()
    {
        return Err("invalid daemon response".to_owned());
    }
    if response.get("error").is_some() {
        return Err("daemon refused the request".to_owned());
    }
    Ok(response["result"].clone())
}

#[cfg(not(unix))]
async fn exchange(_path: &Path, _method: &str, _params: Value) -> Result<Value, String> {
    Err("the daemon transport is not supported on this platform yet".to_owned())
}

/// Returns the effective UID through a local socket pair, without unsafe code.
///
/// # Errors
/// Missing peer credentials are a hard authentication failure.
#[cfg(unix)]
pub fn own_uid() -> io::Result<u32> {
    let (stream, _peer) = tokio::net::UnixStream::pair()?;
    Ok(stream.peer_cred()?.uid())
}

/// Receives a bounded, newline-terminated frame and returns the connection.
///
/// # Errors
/// Truncated, oversized, and invalid UTF-8 frames are rejected.
#[cfg(unix)]
pub async fn receive(
    stream: tokio::net::UnixStream,
) -> io::Result<(tokio::net::UnixStream, String)> {
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
    let mut reader = BufReader::new(stream.take((MAX_FRAME + 1) as u64));
    let mut text = String::new();
    reader.read_line(&mut text).await?;
    if text.len() > MAX_FRAME || !text.ends_with('\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid RPC frame",
        ));
    }
    Ok((reader.into_inner().into_inner(), text))
}

/// A JSON-RPC failure with a fixed, non-sensitive diagnostic.
#[must_use]
pub fn failure(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "error":{"code":code,"message":message}})
}
