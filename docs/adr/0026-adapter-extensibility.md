<!-- shiin-doc: kind=adr status=accepted implementation=n/a milestone=n/a reviewed=2026-09-14 -->

# ADR-0026: Adapter extensibility

> [!NOTE]
> **Architecture decision: accepted.**

- **Date:** 2026-09-14
- **Deciders:** Founding maintainer
- **Supersedes:** none
- **Superseded by:** none
- **Related:** [ADR index](README.md), [Adapter contract](../spec/adapter-contract.md), [ADR-0025](0025-adapter-roadmap.md), [Glossary](../glossary.md#adapter), [Roadmap](../project/roadmap.md#not-planned)

## Context and problem statement

Adapters connect hosts to the Shiin engine.
One option is to allow third parties to load custom adapters as dynamic libraries at runtime.
Another is to compile adapters statically and ship them as part of the CLI or daemon.
The project needed to decide which model to use and whether dynamic loading is supported.

## Decision drivers

- Third parties must be able to integrate, but the attack surface must be minimized.
- Dynamic loading adds complexity and a code-loading trust boundary.
- Compiled adapters are simpler and auditable.

## Considered options

1. Static adapters only: compile all adapters into the relevant binary, no dynamic loading.
2. Dynamic-library plugins loaded at runtime.
3. A plugin server that adapters connect to remotely.

## Decision outcome

Chosen option: "Static adapters only", because dynamic code loading is an unnecessary attack surface for an authorization tool.

- Adapters are compiled into the `shiin` binary or the `shiind` daemon, not loaded at runtime.
- New adapters are added by contributing them to the project, reviewed like any other code.
- This is listed under "Not planned" in the [roadmap](../project/roadmap.md).
- Third-party integrators implement adapters in other languages through the [local API](../spec/local-api.md) and [adapter contract](../spec/adapter-contract.md).

### Consequences

- Good, because there is no dynamic code-loading trust boundary.
- Good, because all adapter code is auditable in the same build and test pipeline.
- Bad, because organizations cannot ship their own private adapters without forking or using the API.

### Confirmation

- The [roadmap not-planned section](../project/roadmap.md#not-planned) lists "Dynamic-library plugins for adapters" as deliberately out of scope.
- The adapter contract specification defines how external integrations work through the local API instead.

## Pros and cons of the options

### Static adapters only

- Good, because it removes the dynamic-loading attack surface.
- Bad, because it requires upstream contribution for new adapters.

### Dynamic-library plugins

- Good, because it allows private adapters.
- Bad, because it adds a code-loading trust boundary.

### Plugin server

- Good, because it isolates adapter failures.
- Bad, because it adds a process boundary and IPC complexity.

## More information

Third parties that need a custom integration should use the [adapter contract](../spec/adapter-contract.md) and the [local API](../spec/local-api.md). Language bindings are tracked in [ADR-0042](0042-language-bindings.md).
