//! Internal Shiin API; it can change in any release.
//!
//! Pure approval transitions. The daemon authenticates the operator, serializes
//! transitions, audits them, and persists the result before returning an allow.

use chrono::DateTime;
use globset::Glob;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shiin_schema::{ActionRequest, ApprovalRequest, Decision, DecisionValue, Resolution, clock};

#[cfg(test)]
mod tests;

/// Maximum lifetime of an operator grant, in seconds.
pub const MAX_LIFETIME: u64 = 86_400;
const VERSION: &str = "shiin.approval-store/v1-draft.1";
const MAX_ENTRIES: usize = 256;

/// An explicitly selected reusable grant scope.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrantScope {
    /// Agent covered by this grant.
    pub agent: String,
    /// Exact action types covered by this grant.
    pub action_types: Vec<String>,
    /// Glob applied to every path effect; `*` explicitly covers non-path effects.
    pub resource: String,
    /// Canonical workspace root.
    pub workspace: String,
}

/// A pending action and its human-facing wire request.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pending {
    /// The approval protocol message.
    pub approval: ApprovalRequest,
    /// The normalized action; never an agent-written description.
    pub action: ActionRequest,
    /// Canonical retry binding.
    binding: String,
    /// Bound task deadline, if a task is active.
    task_deadline: Option<u64>,
}

/// One expiring authorization. Fields are private to prevent accidental mutation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    id: String,
    binding: String,
    scope: Option<GrantScope>,
    session: String,
    task: Option<String>,
    snapshot: String,
    workspace: String,
    expires: u64,
    consumed: bool,
    revoked: bool,
}

/// Durable state. Missing, malformed, or unknown fields are not defaulted on load.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    schema: String,
    requests: Vec<Pending>,
    grants: Vec<Grant>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            schema: VERSION.to_owned(),
            requests: Vec::new(),
            grants: Vec::new(),
        }
    }
}

/// A state transition that must be audited before the updated state is persisted.
#[derive(Debug, Clone)]
pub struct Transition {
    /// Audit event kind.
    pub kind: &'static str,
    /// Ticket or grant identifier, without operator credentials.
    pub id: String,
}

/// Parses an RFC 3339 deadline, rejecting negative or malformed timestamps.
///
/// # Errors
/// Returns a fixed error rather than treating a malformed expiry as no expiry.
pub fn deadline(text: &str) -> Result<u64, String> {
    DateTime::parse_from_rfc3339(text)
        .map_err(|_| "invalid approval or task deadline".to_owned())?
        .timestamp()
        .try_into()
        .map_err(|_| "invalid approval or task deadline".to_owned())
}

/// Binds every semantic request field, excluding only the fresh correlation UUID.
///
/// # Errors
/// Serialization failure prevents authorization.
pub fn binding(request: &ActionRequest) -> Result<String, String> {
    let mut value = serde_json::to_value(request).map_err(|error| error.to_string())?;
    value
        .as_object_mut()
        .ok_or("action must be an object")?
        .remove("id");
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

impl Store {
    /// Validates a loaded state version and bounds before accepting any grants.
    ///
    /// # Errors
    /// Unknown versions, invalid timestamps, or oversized stores fail closed.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != VERSION
            || self.requests.len() > MAX_ENTRIES
            || self.grants.len() > MAX_ENTRIES
        {
            return Err("unsupported or oversized approval store".to_owned());
        }
        for pending in &self.requests {
            pending.action.validate().map_err(str::to_owned)?;
            if pending.binding != binding(&pending.action)?
                || pending.approval.decision != DecisionValue::Pending
                || deadline(&pending.approval.expires_at)?
                    <= deadline(&pending.approval.created_at)?
            {
                return Err("invalid stored approval binding".to_owned());
            }
        }
        for grant in &self.grants {
            if grant.id.is_empty() || grant.expires == 0 {
                return Err("invalid stored grant".to_owned());
            }
            if let Some(scope) = &grant.scope {
                validate_scope(scope)?;
            }
        }
        Ok(())
    }

    /// Unresolved, unexpired actions shown to the operator.
    #[must_use]
    pub fn pending(&self, now: u64) -> Vec<Pending> {
        self.requests
            .iter()
            .filter(|item| {
                item.approval.resolution.is_none()
                    && deadline(&item.approval.expires_at).is_ok_and(|expires| expires > now)
            })
            .cloned()
            .collect()
    }

    /// Grant records for operator inspection.
    #[must_use]
    pub fn grants(&self) -> &[Grant] {
        &self.grants
    }

    /// Checks a historical grant snapshot without changing state or issuing a ticket.
    ///
    /// # Errors
    /// Invalid snapshots and request serialization errors prevent replay.
    pub fn covered(&self, request: &ActionRequest, now: u64) -> Result<bool, String> {
        self.validate()?;
        let digest = binding(request)?;
        Ok(self
            .grants
            .iter()
            .any(|grant| grant.matches(request, &digest, now)))
    }

    /// Workspace whose audit log records a ticket or grant transition.
    #[must_use]
    pub fn workspace_for(&self, id: &str) -> Option<&str> {
        self.requests
            .iter()
            .find(|item| item.approval.ticket == id)
            .map(|item| item.action.workspace.root.as_str())
            .or_else(|| {
                self.grants
                    .iter()
                    .find(|grant| grant.id == id)
                    .map(|grant| grant.workspace.as_str())
            })
    }

    /// Satisfies only pending decisions, consuming single-use grants atomically in memory.
    ///
    /// # Errors
    /// Invalid state, randomness failure, and exceeded storage limits deny the request.
    pub fn apply(
        &mut self,
        request: &ActionRequest,
        decision: &mut Decision,
        now: u64,
        task_deadline: Option<u64>,
    ) -> Result<Option<Transition>, String> {
        self.validate()?;
        if decision.decision != DecisionValue::Pending {
            return Ok(None);
        }
        let digest = binding(request)?;
        if let Some(grant) = self
            .grants
            .iter_mut()
            .find(|grant| grant.matches(request, &digest, now))
        {
            grant.consumed |= grant.scope.is_none();
            decision.decision = DecisionValue::Allow;
            "allow.grant".clone_into(&mut decision.reason);
            DecisionValue::Allow
                .message()
                .clone_into(&mut decision.message);
            "grant".clone_into(&mut decision.stage);
            decision.approval = None;
            return Ok(Some(Transition {
                kind: "grant.used",
                id: grant.id.clone(),
            }));
        }
        if let Some(pending) = self.requests.iter().find(|item| {
            item.binding == digest
                && deadline(&item.approval.expires_at).is_ok_and(|expires| expires > now)
        }) {
            decision.approval = Some(pending.approval.clone());
            return Ok(None);
        }
        self.requests
            .retain(|item| deadline(&item.approval.expires_at).is_ok_and(|expires| expires > now));
        self.grants.retain(|grant| grant.expires > now);
        if self.requests.len() >= MAX_ENTRIES {
            return Err("approval capacity reached".to_owned());
        }
        let expires = now
            .checked_add(300)
            .ok_or("approval deadline overflow")?
            .min(task_deadline.unwrap_or(u64::MAX));
        if expires <= now {
            return Err("task no longer permits approval".to_owned());
        }
        let nonce = clock::new_id();
        if nonce.is_empty() {
            return Err("OS randomness failed".to_owned());
        }
        let ticket = format!("{nonce}:{}", digest.trim_start_matches("sha256:"));
        let approval = ApprovalRequest {
            schema: "shiin.approval-protocol/v1-draft.1".to_owned(),
            request_id: request.id.clone(),
            decision: DecisionValue::Pending,
            reason: decision.reason.clone(),
            message: decision.message.clone(),
            ticket: ticket.clone(),
            created_at: clock::rfc3339_from_unix_seconds(now),
            expires_at: clock::rfc3339_from_unix_seconds(expires),
            resolution: None,
        };
        decision.approval = Some(approval.clone());
        self.requests.push(Pending {
            approval,
            action: request.clone(),
            binding: digest,
            task_deadline,
        });
        Ok(Some(Transition {
            kind: "approval.requested",
            id: ticket,
        }))
    }

    /// Resolves one pending action after the service authenticates an operator.
    ///
    /// # Errors
    /// Unknown, already resolved, expired, or invalidly scoped requests are rejected.
    pub fn resolve(
        &mut self,
        ticket: &str,
        resolution: Resolution,
        scope: Option<GrantScope>,
        lifetime: u64,
        now: u64,
    ) -> Result<Vec<Transition>, String> {
        self.validate()?;
        if lifetime == 0 || lifetime > MAX_LIFETIME || self.grants.len() >= MAX_ENTRIES {
            return Err("invalid grant lifetime or capacity".to_owned());
        }
        let pending = self
            .requests
            .iter_mut()
            .find(|item| item.approval.ticket == ticket)
            .ok_or("unknown approval ticket")?;
        if pending.approval.resolution.is_some() || deadline(&pending.approval.expires_at)? <= now {
            return Err("approval is expired or already resolved".to_owned());
        }
        let mut events = vec![Transition {
            kind: "approval.resolved",
            id: ticket.to_owned(),
        }];
        if resolution != Resolution::Deny {
            match (&scope, resolution) {
                (Some(scope), Resolution::AllowAlways) => {
                    validate_scope(scope)?;
                    if scope.agent != pending.action.agent.id
                        || scope.workspace != pending.action.workspace.root
                        || !scope.action_types.contains(&pending.action.action_type)
                        || !scope_matches(scope, &pending.action)
                    {
                        return Err("scope does not cover the displayed action".to_owned());
                    }
                }
                (None, Resolution::AllowOnce) => {}
                _ => {
                    return Err(
                        "allow_always requires an explicit scope; allow_once does not accept one"
                            .to_owned(),
                    );
                }
            }
            let id = clock::new_id();
            if id.is_empty() {
                return Err("OS randomness failed".to_owned());
            }
            let expires = now
                .checked_add(lifetime)
                .ok_or("grant deadline overflow")?
                .min(pending.task_deadline.unwrap_or(u64::MAX));
            if expires <= now {
                return Err("task no longer permits a grant".to_owned());
            }
            self.grants.push(Grant {
                id: id.clone(),
                binding: pending.binding.clone(),
                scope,
                session: pending.action.session.clone(),
                task: pending.action.task.clone(),
                snapshot: pending.action.policy_snapshot.clone(),
                workspace: pending.action.workspace.root.clone(),
                expires,
                consumed: false,
                revoked: false,
            });
            events.push(Transition {
                kind: "grant.created",
                id,
            });
        }
        pending.approval.resolution = Some(resolution);
        Ok(events)
    }

    /// Revokes either a pending ticket or a grant.
    ///
    /// # Errors
    /// Unknown identifiers are not silently accepted.
    pub fn revoke(&mut self, id: &str) -> Result<Transition, String> {
        if let Some(grant) = self.grants.iter_mut().find(|grant| grant.id == id) {
            grant.revoked = true;
            return Ok(Transition {
                kind: "grant.revoked",
                id: id.to_owned(),
            });
        }
        let pending = self
            .requests
            .iter_mut()
            .find(|pending| pending.approval.ticket == id)
            .ok_or("unknown ticket or grant")?;
        if pending.approval.resolution.is_some() {
            return Err("approval is already resolved".to_owned());
        }
        pending.approval.resolution = Some(Resolution::Deny);
        Ok(Transition {
            kind: "approval.resolved",
            id: id.to_owned(),
        })
    }
}

impl Grant {
    fn matches(&self, request: &ActionRequest, digest: &str, now: u64) -> bool {
        !self.revoked
            && !self.consumed
            && self.expires > now
            && self.session == request.session
            && self.task == request.task
            && self.snapshot == request.policy_snapshot
            && self.scope.as_ref().map_or_else(
                || self.binding == digest,
                |scope| scope_matches(scope, request),
            )
    }
}

fn validate_scope(scope: &GrantScope) -> Result<(), String> {
    if scope.agent.is_empty()
        || scope.workspace.is_empty()
        || scope.action_types.is_empty()
        || scope
            .action_types
            .iter()
            .any(|kind| kind.is_empty() || kind.contains('*'))
    {
        return Err("grant scope requires an agent, workspace, and exact action types".to_owned());
    }
    Glob::new(&scope.resource).map_err(|error| error.to_string())?;
    Ok(())
}

fn scope_matches(scope: &GrantScope, request: &ActionRequest) -> bool {
    scope.agent == request.agent.id
        && scope.workspace == request.workspace.root
        && scope.action_types.contains(&request.action_type)
        && request.effects.iter().all(|effect| {
            effect
                .action_type
                .as_ref()
                .is_none_or(|kind| scope.action_types.contains(kind))
        })
        && Glob::new(&scope.resource).is_ok_and(|glob| {
            let matcher = glob.compile_matcher();
            request.effects.iter().all(|effect| {
                if effect.kind == "path" {
                    matcher.is_match(&effect.value)
                } else {
                    scope.resource == "*"
                }
            })
        })
}
