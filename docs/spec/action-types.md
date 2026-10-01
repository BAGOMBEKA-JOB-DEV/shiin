<!-- shiin-doc: kind=spec status=draft implementation=none milestone=v0.1 reviewed=2026-10-01 -->

# Action types

> [!NOTE]
> **Specification: draft.**
> Normative design. No implementation exists yet.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

An [action type](../glossary.md#action-type) is a dotted identifier that classifies an [action](../glossary.md#action).
Policies refer to action types, never to host tool names.

## Conventions

- Action types use lowercase letters, digits, and dots.
- Each top-level namespace groups related effects.
- New namespaces go through the [RFC process](../rfcs/README.md) and produce an ADR.

## Namespace `fs`

Filesystem effects.

| Action type | Meaning |
|---|---|
| `fs.read` | Read a file or directory entry. |
| `fs.write` | Write or overwrite a file. |
| `fs.create` | Create a new file, directory, or symlink. |
| `fs.delete` | Delete a file, directory, or symlink. |
| `fs.move` | Move or rename a file or directory. |
| `fs.chmod` | Change file permissions or ownership. |

**[AT-001]** `fs.read` MUST cover reading a file through any host mechanism, including a file-read tool and a shell command such as `cat`.

**[AT-002]** `fs.write`, `fs.create`, `fs.delete`, `fs.move`, and `fs.chmod` MUST each cover the corresponding operation through any host mechanism.

## Namespace `shell`

Effects of running a program through a shell.

| Action type | Meaning |
|---|---|
| `shell.exec` | Execute a program or shell command. |

**[AT-003]** `shell.exec` MUST cover any program the agent starts through a shell, a command tool, or a terminal.

## Namespace `network`

Network effects.

| Action type | Meaning |
|---|---|
| `net.connect` | Open a connection to a remote host. |
| `net.fetch` | Fetch a URL over HTTP or HTTPS. |

**[AT-004]** `net.connect` MUST cover any outbound TCP or UDP connection.

**[AT-005]** `net.fetch` MUST cover any HTTP or HTTPS request.

## Namespace `db`

Database effects.

| Action type | Meaning |
|---|---|
| `db.query` | Run a read query against a database. |
| `db.mutate` | Run a write query against a database. |
| `db.schema_change` | Change a database schema, including migrations. |

**[AT-006]** `db.query`, `db.mutate`, and `db.schema_change` MUST cover the corresponding operation through any host mechanism, including SQL clients and ORM tools.

## Unknown actions

**[AT-007]** An action that the adapter cannot classify MUST be recorded with an [opaque effect](../glossary.md#opaque-effect) rather than an invented action type.

## Revision history

- v1-draft.1 (2026-10-01): Initial draft.