//! Turns what a host reports into normalized action types and effects.
//!
//! Classification is deterministic and conservative: anything it cannot
//! determine becomes an opaque effect, which the engine can never allow on a
//! path rule alone ([ADR-0018](../../../docs/adr/0018-conservative-action-classification.md)).
//!
//! Paths are canonicalized by the rules in
//! [ADR-0019](../../../docs/adr/0019-path-canonicalization.md): make absolute against
//! the workspace root, resolve symlinks on the longest existing prefix, then resolve
//! `.` and `..` lexically so a traversal cannot step over a symlink.

use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Component, Path, PathBuf};

use shiin_schema::{Confidence, Effect};

/// Interpreters whose arguments Shiin cannot read, so their effects are opaque.
const INTERPRETERS: &[&str] = &[
    "python", "python3", "node", "ruby", "perl", "sh", "bash", "zsh", "fish", "deno", "bun",
];

/// Shell constructs that hide the real command.
const OBSCURING: &[&str] = &["eval", "$(", "`", "|", ";", "&&", "||", ">", "<"];

/// Commands that read a file, mapped to the argument that names it.
const READERS: &[&str] = &["cat", "head", "tail", "less", "more", "bat"];

/// Canonicalizes a path for policy matching.
///
/// The path is walked from the root one component at a time. A symlink is replaced by
/// its target, and the target's own components are then walked too, so a target that
/// itself contains a symlink is resolved rather than merely tidied. macOS needs this:
/// a temporary directory's real path runs through `/var`, which is a link to
/// `/private/var`, so a lexical pass would leave two spellings of the same directory.
///
/// A `..` segment pops the already-resolved path, so a traversal cannot step over a
/// symlink the way it could if `..` were resolved first. A path that does not exist
/// still canonicalizes, which keeps `fs.create` classifiable.
#[must_use]
pub fn canonical_path(raw: &str, workspace_root: &str) -> String {
    /// Guards against a symlink loop.
    const MAX_HOPS: usize = 64;

    let joined = if Path::new(raw).is_absolute() {
        PathBuf::from(raw)
    } else {
        Path::new(workspace_root).join(raw)
    };

    let mut remaining: VecDeque<OsString> = components_of(&joined);
    let mut resolved = PathBuf::from("/");
    let mut hops = 0_usize;

    while let Some(part) = remaining.pop_front() {
        if part == OsStr::new(".") {
            continue;
        }
        if part == OsStr::new("..") {
            resolved.pop();
            continue;
        }
        let candidate = resolved.join(&part);
        match fs::read_link(&candidate) {
            Err(_) => resolved = candidate,
            Ok(target) => {
                hops = hops.saturating_add(1);
                if hops > MAX_HOPS {
                    // A loop, or something pathological. Stop where we are rather than
                    // spin: the caller gets a path that cannot match a rule by accident.
                    resolved = candidate;
                    continue;
                }
                // Walk the target's own components next, so a target containing another
                // symlink is resolved too.
                if target.is_absolute() {
                    resolved = PathBuf::from("/");
                }
                for component in components_of(&target).into_iter().rev() {
                    remaining.push_front(component);
                }
            }
        }
    }
    display(&resolved)
}

/// A path's components, without its root, as owned names.
fn components_of(path: &Path) -> VecDeque<OsString> {
    path.components()
        .filter_map(|component| match component {
            Component::Prefix(_) | Component::RootDir => None,
            Component::CurDir => Some(OsString::from(".")),
            Component::ParentDir => Some(OsString::from("..")),
            Component::Normal(name) => Some(name.to_os_string()),
        })
        .collect()
}

/// Renders a path with forward slashes, resolving any remaining `.` and `..`.
fn display(path: &Path) -> String {
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            Component::RootDir => parts.clear(),
            Component::Prefix(prefix) => parts.push(prefix.as_os_str().to_os_string()),
            Component::Normal(name) => parts.push(name.to_os_string()),
        }
    }
    let joined = parts
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    format!("/{joined}")
}

/// Canonicalizes a workspace root reported by a host.
///
/// A host reports whatever path it was started with, which may run through a symlink:
/// macOS puts temporary directories under `/var`, which is a link to `/private/var`.
/// Policy globs are resolved against this root while effects are canonicalized, so if
/// the root is not canonical too, a rule written `./src/**` never matches the very paths
/// it is about, and every action falls through to the default decision (`AR-008`).
#[must_use]
pub fn canonical_root(root: &str) -> String {
    canonical_path(root, root)
}

/// A path effect with exact confidence.
#[must_use]
pub fn path_effect(raw: &str, workspace_root: &str) -> Effect {
    Effect {
        kind: "path".to_owned(),
        value: canonical_path(raw, workspace_root),
        confidence: Confidence::Exact,
        action_type: None,
    }
}

/// Classifies a shell command into an action type and its effects.
///
/// A command whose real behavior cannot be read produces an opaque effect, which
/// is what keeps `python cleanup.py` from being allowed by a path rule.
#[must_use]
pub fn classify_command(command: &str, workspace_root: &str) -> (String, Vec<Effect>) {
    let mut effects = vec![Effect {
        kind: "command".to_owned(),
        value: command.trim().to_owned(),
        confidence: if is_opaque(command) {
            Confidence::Opaque
        } else {
            Confidence::Exact
        },
        action_type: None,
    }];

    let words = shell_words(command);
    if let Some((first, rest)) = words.split_first() {
        let program = program_name(first);
        if READERS.contains(&program.as_str()) {
            for argument in rest.iter().filter(|word| !word.starts_with('-')) {
                effects.push(Effect {
                    kind: "path".to_owned(),
                    value: canonical_path(argument, workspace_root),
                    confidence: Confidence::Inferred,
                    // The command performs a read, so a rule for `fs.read` applies.
                    action_type: Some("fs.read".to_owned()),
                });
            }
        }
    }
    ("shell.exec".to_owned(), effects)
}

/// Whether a command's real behavior cannot be determined before it runs.
#[must_use]
pub fn is_opaque(command: &str) -> bool {
    let trimmed = command.trim();
    if OBSCURING.iter().any(|marker| trimmed.contains(marker)) {
        return true;
    }
    shell_words(trimmed)
        .first()
        .map(|first| program_name(first))
        .is_some_and(|program| INTERPRETERS.contains(&program.as_str()))
}

/// Splits a command into words, honoring single and double quotes.
#[must_use]
pub fn shell_words(command: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for character in command.chars() {
        match (quote, character) {
            (Some(open), c) if c == open => quote = None,
            (None, c @ ('\'' | '"')) => quote = Some(c),
            (None, c) if c.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            (_, c) => current.push(c),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// The program name of a command word, without its directory.
fn program_name(word: &str) -> String {
    Path::new(word).file_name().map_or_else(
        || word.to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A workspace root that exists on no platform.
    ///
    /// Canonicalization resolves real symlinks, so a root that happens to exist is not a
    /// fixed point: macOS resolves `/home` through autofs, which rewrote these paths and
    /// made the tests pass on Linux and fail there.
    const ROOT: &str = "/shiin-test-root/project";

    #[test]
    fn a_traversal_cannot_step_outside_the_workspace_lexically() {
        assert_eq!(
            canonical_path("src/../../etc/passwd", ROOT),
            "/shiin-test-root/etc/passwd"
        );
        // A traversal cannot climb above the root. The target has to be a path that
        // exists nowhere: resolving `/etc` would be correct behaviour, because macOS
        // makes it a symlink to `/private/etc`, and that is not what this asserts.
        assert_eq!(
            canonical_path("../../../../shiin-nowhere", "/shiin-test-root"),
            "/shiin-nowhere"
        );
        assert_eq!(
            canonical_path("./src/login.rs", ROOT),
            "/shiin-test-root/project/src/login.rs"
        );
    }

    #[test]
    // Creating a symlink is a Unix call here; Windows path rules are deferred by ADR-0019.
    #[cfg(unix)]
    fn a_symlink_target_is_itself_resolved() {
        // The shape macOS has by default: the link's target is an absolute path that
        // runs through another symlink, so resolving it needs a second pass.
        let base = std::env::temp_dir().join(format!("shiin-classify-{}", std::process::id()));
        let real = base.join("real");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&real).expect("the directory is created");

        let alias = base.join("alias");
        std::os::unix::fs::symlink(&base, &alias).expect("the parent link is created");
        let view = base.join("view");
        std::os::unix::fs::symlink(alias.join("real"), &view).expect("the link is created");

        let through_link = canonical_path(&view.to_string_lossy(), "/unused");
        let direct = canonical_path(&real.to_string_lossy(), "/unused");
        assert_eq!(
            through_link, direct,
            "both spellings name the same directory"
        );

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn a_workspace_root_is_canonical() {
        // A root that exists nowhere is already canonical.
        assert_eq!(
            canonical_root("/shiin-test-root/project"),
            "/shiin-test-root/project"
        );
        // A root reached through a symlink resolves to the same place as its effects.
        let root = canonical_root(&std::env::temp_dir().to_string_lossy());
        assert_eq!(
            canonical_root(&root),
            root,
            "canonicalizing a root is stable"
        );
    }

    #[test]
    fn canonicalization_is_idempotent() {
        let once = canonical_path("src/./login.rs", ROOT);
        let twice = canonical_path(&once, ROOT);
        assert_eq!(once, twice);
    }

    #[test]
    fn an_interpreter_hides_its_effects() {
        assert!(is_opaque("python scripts/cleanup.py"));
        assert!(is_opaque("bash -c 'rm -rf build'"));
        assert!(is_opaque("cat .env | base64"));
        assert!(!is_opaque("cargo test"));
    }

    #[test]
    fn a_reader_exposes_the_path_it_reads_as_a_read() {
        let (action, effects) = classify_command("cat .env", ROOT);
        assert_eq!(action, "shell.exec");
        let read = effects
            .iter()
            .find(|effect| effect.value == "/shiin-test-root/project/.env")
            .expect("the read is exposed");
        assert_eq!(read.action_type.as_deref(), Some("fs.read"));
    }

    #[test]
    fn quoted_words_stay_together() {
        assert_eq!(
            shell_words("git commit -m 'two words'"),
            vec!["git", "commit", "-m", "two words"]
        );
    }
}
