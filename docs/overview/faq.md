<!-- shiin-doc: kind=explanation status=draft implementation=none milestone=m0 reviewed=2026-10-01 -->

# FAQ

> [!NOTE]
> **Design document: draft.**
> Describes the intended design. No implementation exists yet.

## What is Shiin?

Shiin is an agent-agnostic authorization layer for AI agent actions.
It sits between agents and the files, commands, networks, and services they act on.

## Is Shiin ready to use?

No. Shiin is in its specification phase.
This repository contains the design: documentation, specifications, architecture decision records, and a small Rust tool that checks the documentation.

## What makes Shiin different?

Four things:

1. **Task-intent binding.** Actions are checked against the constraints of the task the developer confirmed.
2. **An open, agent-neutral contract.** The action request, decision, and approval protocol are versioned specifications with schemas and test vectors.
3. **Analyzable policy.** Policies compile to Cedar, a formally verified authorization language.
4. **Explicit enforcement levels.** Every integration is labeled cooperative, mediated, or OS-enforced.

## Does Shiin use a language model to decide?

No. No decision ever depends on the output of a language model.

## Does Shiin collect telemetry?

No. Shiin is local-first and collects no telemetry.

## What can Shiin do that my host's built-in controls cannot?

Shiin applies one policy across every agent and binds decisions to the task the developer gave.
Host controls are configured separately per agent and do not know the task.

## What can Shiin not do?

Shiin is not a sandbox, a network firewall, a prompt-injection detector, or a DLP.
It complements those tools; it does not replace them.

## Read next

- [Vision](vision.md)
- [Landscape](landscape.md)
- [Limitations](../concepts/limitations.md)