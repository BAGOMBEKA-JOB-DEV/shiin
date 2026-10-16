//! Cooperative command wrapper. Only the main thread can start an authorized child.

use sha2::{Digest, Sha256};
use shiin_schema::{
    ActionRequest, Agent, Assurance, Confidence, DecisionValue, Effect, Host, Workspace, clock,
    versions,
};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

pub(super) fn run(arguments: &[OsString]) -> ExitCode {
    if !cfg!(unix) {
        eprintln!("The command wrapper currently requires Unix path handling.");
        return ExitCode::from(1);
    }
    let Ok((session, command, arguments)) = parse(arguments) else {
        eprintln!("usage: shiin exec --session ID -- /ABSOLUTE/PROGRAM [ARG ...]");
        return ExitCode::from(1);
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    let child_arguments = arguments.clone();
    std::thread::spawn(move || {
        let outcome = authorize(&session, &command, &arguments);
        let _ = sender.send(outcome);
    });
    let Ok(Ok((decision, program))) = receiver.recv_timeout(super::HOOK_TIMEOUT) else {
        eprintln!("Shiin could not authorize this command. The action was stopped.");
        return ExitCode::from(1);
    };
    if decision.decision != DecisionValue::Allow {
        eprintln!("{}", decision.decision.message());
        if let Some(approval) = decision.approval {
            eprintln!("Ticket: {}", approval.ticket);
        }
        return ExitCode::from(if decision.decision == DecisionValue::Pending {
            2
        } else {
            1
        });
    }
    if let Ok(status) = Command::new(program).args(child_arguments).status() {
        ExitCode::from(
            status
                .code()
                .and_then(|code| u8::try_from(code).ok())
                .unwrap_or(1),
        )
    } else {
        eprintln!("Shiin authorized the command, but the executable could not be started.");
        ExitCode::from(1)
    }
}

fn parse(arguments: &[OsString]) -> Result<(String, PathBuf, Vec<OsString>), String> {
    let [option, session, separator, command, rest @ ..] = arguments else {
        return Err("missing wrapper arguments".to_owned());
    };
    let session = session
        .to_str()
        .filter(|value| !value.is_empty() && value.len() <= 256)
        .ok_or("invalid wrapper session")?;
    if option != "--session" || separator != "--" || !PathBuf::from(command).is_absolute() {
        return Err("wrapper requires a session and an absolute executable".to_owned());
    }
    Ok((session.to_owned(), PathBuf::from(command), rest.to_vec()))
}

fn authorize(
    session: &str,
    program: &std::path::Path,
    arguments: &[OsString],
) -> Result<(shiin_schema::Decision, PathBuf), String> {
    if shiin_ipc::enabled() {
        let runtime = shiin_ipc::RuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        super::DAEMON_RUNTIME
            .set(runtime)
            .map_err(|_| "local transport runtime already initialized")?;
    }
    let program = std::fs::canonicalize(program).map_err(|_| "executable is unavailable")?;
    if !program.is_file() {
        return Err("executable is not a file".to_owned());
    }
    let root = super::workspace_root();
    let argv: Vec<&str> = std::iter::once(program.as_os_str())
        .chain(arguments.iter().map(OsString::as_os_str))
        .map(|value| value.to_str().ok_or("wrapper arguments must be UTF-8"))
        .collect::<Result<_, _>>()?;
    let bytes = serde_json::to_vec(&argv).map_err(|error| error.to_string())?;
    let command = argv
        .iter()
        .map(|argument| quote(argument))
        .collect::<Vec<_>>()
        .join(" ");
    let request = ActionRequest {
        schema: versions::ACTION_REQUEST.to_owned(),
        id: clock::new_id(),
        session: session.to_owned(),
        agent: Agent {
            id: "shell-wrapper".to_owned(),
            assurance: Assurance::Asserted,
        },
        action_type: "shell.exec".to_owned(),
        effects: vec![
            Effect {
                kind: "command".to_owned(),
                value: command,
                confidence: Confidence::Opaque,
                action_type: None,
            },
            Effect {
                kind: "argv_digest".to_owned(),
                value: format!("sha256:{:x}", Sha256::digest(bytes)),
                confidence: Confidence::Opaque,
                action_type: None,
            },
        ],
        task: None,
        workspace: Workspace {
            root: root.clone(),
            vcs: None,
        },
        policy_snapshot: String::new(),
        host: Host {
            name: "shell-wrapper".to_owned(),
            tool: Some("exec".to_owned()),
        },
    };
    let lock = super::lock_workspace()?;
    let decision = super::decide_normalized(request, &root, lock)?;
    Ok((decision, program))
}

fn quote(argument: &str) -> String {
    if !argument.is_empty()
        && argument
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"/_-.=:".contains(&byte))
    {
        argument.to_owned()
    } else {
        format!("'{}'", argument.replace('\'', "'\\''"))
    }
}
