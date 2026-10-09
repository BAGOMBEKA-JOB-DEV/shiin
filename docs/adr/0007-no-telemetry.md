<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0007: No telemetry

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Vision](../overview/vision.md), [Privacy and telemetry](../project/privacy-and-telemetry.md), [ADR-0022](0022-local-first-daemonless-deployment.md)

## Context and problem statement

Shiin intercepts actions on the developer's machine and records decisions about them.
An authorization tool that reports usage back to a server creates a new attack surface and a conflict of interest: the tool's incentives align with collecting data, not with keeping that data private.
The project needed to decide whether any data leaves the machine.

## Decision drivers

- The machine running an agent already has the operator's secrets, so any network call is a potential leak.
- Trust must be earned before it can be required.
- A tool that collects no data does not need a data-retention or privacy-compliance program.

## Considered options

1. No telemetry or data collection by default; opt-in for any remote feature.
2. Anonymous usage statistics with an opt-out.
3. Mandatory telemetry for security and improvement.

## Decision outcome

Chosen option: "No telemetry or data collection by default", because Shiin handles security-sensitive actions and cannot ethically collect data without an explicit, informed opt-in.

- No events, metrics, crash reports, or usage data are sent anywhere by default.
- No HTTP client is a dependency of the engine crates.
- Any network feature (remote approvals, remote policy bundles) is explicitly opt-in and uses a separate transport crate.
- The daemon never phones home, even to check for updates.
- This decision is referenced from [ADR-0022](0022-local-first-daemonless-deployment.md).

### Consequences

- Good, because operators can run Shiin in sensitive environments without concern about data leakage.
- Good, because the absence of telemetry simplifies the supply chain and threat model.
- Bad, because the maintainers receive no usage signals about which features matter.

### Confirmation

- [Privacy and telemetry](../project/privacy-and-telemetry.md) documents that no telemetry is collected.
- Use-case [UC-08](../overview/use-cases.md#uc-08-govern-mcp-tools) requires the MCP proxy to be controllable without a remote service.

## Pros and cons of the options

### No telemetry, opt-in for remote features

- Good, because it earns trust and avoids a data-handling compliance burden.
- Bad, because maintainers have no passive usage data.

### Anonymous usage statistics

- Good, because it provides development signals.
- Bad, because even "anonymous" data can be re-identified, and it requires an HTTP client in the default install.

### Mandatory telemetry

- Good, because it maximizes data quality.
- Bad, because it is unacceptable for a security-adjacent tool.

## More information

The threat model treats any outbound connection that is not explicitly opt-in as a security concern. See [ADR-0022](0022-local-first-daemonless-deployment.md) for the deployment model that enables this.
