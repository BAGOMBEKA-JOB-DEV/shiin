//! Shiin's per-user approval service. Operator credentials never enter agent responses.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the daemon reports startup and operator instructions"
)]

use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("shiind: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(unix))]
fn run() -> Result<(), String> {
    Err("local approval services currently require Unix peer credentials".to_owned())
}

#[cfg(unix)]
fn run() -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(serve())
}

#[cfg(unix)]
async fn serve() -> Result<(), String> {
    use serde_json::{Value, json};
    use shiind::{OperatorKey, Service};
    use std::fs::{self, DirBuilder};
    use std::io::IsTerminal;
    use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};
    use std::sync::{Arc, Mutex};
    use tokio::io::AsyncWriteExt;
    use tokio::sync::Semaphore;

    let path = shiin_ipc::endpoint()?;
    let directory = path.parent().ok_or("daemon endpoint has no parent")?;
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)
        .map_err(|error| error.to_string())?;
    let uid = shiin_ipc::own_uid().map_err(|error| error.to_string())?;
    let metadata = fs::symlink_metadata(directory).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 {
        return Err("daemon directory must be owned by this user with mode 0700".to_owned());
    }
    let operator = OperatorKey::generate()?;
    let console = std::env::args().any(|argument| argument == "--operator");
    if console && (!std::io::stdin().is_terminal() || !std::io::stderr().is_terminal()) {
        return Err("operator mode requires a dedicated terminal".to_owned());
    }
    // Do not reveal a credential until exclusive ownership and bind both succeed.
    let display = if console {
        Some(operator.for_operator_terminal().to_owned())
    } else {
        None
    };
    let service = Service::open(directory, operator)?;
    if let Ok(metadata) = fs::symlink_metadata(&path) {
        if !metadata.file_type().is_socket() || metadata.uid() != uid {
            return Err("unsafe existing daemon endpoint".to_owned());
        }
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    let listener = tokio::net::UnixListener::bind(&path).map_err(|error| error.to_string())?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    if let Some(display) = display {
        eprintln!(
            "Operator credential (keep in this terminal; never give it to an agent): {display}"
        );
        eprintln!("Use shiin pending, shiin approve, or shiin revoke from your own terminal.");
    }
    eprintln!("Listening on {}", path.display());
    let service = Arc::new(Mutex::new(service));
    let capacity = Arc::new(Semaphore::new(32));
    loop {
        let (stream, _) = tokio::select! {
            value = listener.accept() => value.map_err(|error| error.to_string())?,
            signal = tokio::signal::ctrl_c() => { signal.map_err(|error| error.to_string())?; break; }
        };
        if stream.peer_cred().map_err(|error| error.to_string())?.uid() != uid {
            continue;
        }
        let Ok(permit) = Arc::clone(&capacity).try_acquire_owned() else {
            continue;
        };
        let service = Arc::clone(&service);
        tokio::spawn(async move {
            let _permit = permit;
            let exchange = async {
                let (mut stream, frame) = shiin_ipc::receive(stream).await?;
                let parsed = serde_json::from_str::<shiin_ipc::Request>(&frame);
                let response = match parsed {
                    Ok(request) if request.jsonrpc == "2.0" && request.id.is_string() => {
                        let id = request.id;
                        let result = tokio::task::spawn_blocking(move || {
                            let mut service = service
                                .lock()
                                .map_err(|_| "daemon state lock failed".to_owned())?;
                            service.dispatch(&request.method, request.params)
                        })
                        .await;
                        match result {
                            Ok(Ok(result)) => json!({"jsonrpc":"2.0", "id":id, "result":result}),
                            _ => shiin_ipc::failure(
                                id,
                                -32000,
                                "Shiin could not authorize this request",
                            ),
                        }
                    }
                    _ => shiin_ipc::failure(Value::Null, -32600, "Invalid request"),
                };
                let mut bytes = serde_json::to_vec(&response)?;
                bytes.push(b'\n');
                if bytes.len() > shiin_ipc::MAX_FRAME {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "response too large",
                    ));
                }
                stream.write_all(&bytes).await?;
                Ok::<(), std::io::Error>(())
            };
            let _ = tokio::time::timeout(shiin_ipc::TIMEOUT, exchange).await;
        });
    }
    fs::remove_file(path).map_err(|error| error.to_string())?;
    Ok(())
}
