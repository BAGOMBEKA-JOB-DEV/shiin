<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Identity

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.
>
> Version: `shiin.identity/v1-draft.1`.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

The identity stage answers: do we know which agent is asking, and how sure are we?

## Assurance levels

**[ID-001]** An [agent identity](../glossary.md#agent-identity) MUST carry an [assurance level](../glossary.md#assurance-level).

**[ID-002]** The assurance levels, weakest first, are:

| Level | Meaning |
|---|---|
| `asserted` | The host reports the agent id. Nothing verifies it. |
| `attested-local` | The agent id is attested by a local trust root. |
| `attested-os-user` | The agent id is attested by the operating system user. |
| `attested-crypto` | The agent id is attested cryptographically. Reserved for the future. |

## Behavior

**[ID-003]** The identity stage is a [gate](../glossary.md#gate): it may return `deny`, `pending`, or `pass`, but never `allow`.

**[ID-004]** An unknown agent identity MUST produce `deny` with reason code `deny.identity`.

**[ID-005]** An identity whose assurance is too low for the action class MUST produce `pending` with reason code `pending.identity`.

**[ID-006]** An identity that is accepted MUST produce `pass`.

## Untrusted input

**[ID-007]** Agent-supplied identity claims, including self-reported ids and tool annotations, can make a decision stricter but never more permissive.

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.