<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0003: Project name: Shiin

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Documentation style guide](../project/docs-style-guide.md), [Overview](overview/vision.md)

## Context and problem statement

The project began under the working name "latch."
As it moved from a private sketch to a public specification, a permanent name was needed that conveys the project's role: it stops an agent, briefly, to ask whether an action is permitted.
The name must be pronounceable, memorable, and free of collisions on crates.io and standard domain names.

## Decision drivers

- The name should evoke stopping or gating, without implying that Shiin is a sandbox or a blocker.
- It must not collide with an existing high-profile crate or domain.
- It should be language-neutral enough for an agent-agnostic tool.

## Considered options

1. Keep "latch" as the permanent name.
2. "Gate" — conveys the gating function directly.
3. "Shiin" — the Japanese word for the conceptual barrier that blocks or permits.

## Decision outcome

Chosen option: "Shiin", because it conveys the gating role in an understated, memorable way and is free of known collisions at the time of writing.

- The binary is `shiin`, the daemon is `shiind`, matching the convention described in the [glossary](../glossary.md#shiin).
- The former name "latch" may appear only in this record and in the [landscape](../overview/landscape.md), as enforced by `cargo xtask docs-check`.

### Consequences

- Good, because the project has a distinctive, non-colliding name.
- Bad, because existing references to the old name in discussions need updating.

### Confirmation

- `cargo xtask docs-check` rejects the former working name outside the two allowed pages.

## Pros and cons of the options

### Keep "latch"

- Good, because no migration of references is needed.
- Bad, because the name does not convey the gating role, and it is already used as a GitHub organization name ([latchagent/latch](https://github.com/latchagent/latch), verified 2026-09-14).

### "Gate"

- Good, because it is short and directly descriptive.
- Bad, because it is a common word with many existing packages and domains, and it implies a single point of control rather than layers.

### "Shiin"

- Good, because it is distinctive and pronounceable.
- Bad, because readers outside Japanese-speaking contexts may not know the meaning.

## More information

The name change does not affect the technical design. See [ADR-0004](0004-apache-2-license-and-dco.md) for the licensing implications of the rename.
