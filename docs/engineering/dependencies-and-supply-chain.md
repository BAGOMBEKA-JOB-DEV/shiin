<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-16 -->

# Dependencies and supply chain

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's dependency and supply-chain rules ([ADR-0038](../adr/0038-supply-chain-policy.md)).

## Admission

Every third-party dependency must be admitted with a reason.
Reviewers weigh:

- How much unsafe code the crate contains, and whether it is necessary.
- Whether its unsafe blocks are documented and tested.
- Its history of soundness advisories in the RustSec database.
- Whether a safe alternative of similar quality exists.

## Declaring dependencies

Key storage uses `keyring-core` with explicit native platform stores, following [ADR-0032](../adr/0032-signing-key-storage.md).
The Linux store uses the vendored `dbus-secret-service-keyring-store` with its Rust cryptography feature; macOS and Windows retain their native stores.
The [store contract](https://docs.rs/dbus-secret-service-keyring-store/1.0.1/dbus_secret_service_keyring_store/) uses the same `service` and `username` attributes and requires encrypted Secret Service communication (verified 2026-10-18).
This removes the prior Zbus dependency chain identified by [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370), [RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371), and [RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372).
No advisory or yanked-crate check is disabled.
Vendored native DBus code adds a C dependency within the store; the native key-store lifecycle job exercises it against the runner's actual Secret Service.

The v0.6 downstream MCP HTTP transport admits `reqwest` for HTTP and TLS rather than implementing either protocol in Shiin.
It is confined to the proxy binary; it does not enter the engine crates.
The SDK transport uses its asynchronous client with redirects, automatic retries, and environment proxies disabled.
`axum` is admitted only as a development dependency to run the HTTP MCP fixture; it adds no product listener.
Both dependencies disable default features and are checked by the existing dependency-policy CI job.
Transitive HTTP/TLS dependencies still require supply-chain review before a release.
Development-only `rcgen`, `rustls`, `tokio-rustls`, and `hyper-util` generate a temporary certificate and serve the authenticated TLS MCP fixture.
Private fixture keys are generated at runtime and are never committed.

`allocation-counter` is admitted only for the optional `xtask/memory-profile` feature.
It measures allocations made by synchronous component calls through a safe API; no product binary or engine dependency graph includes the counter.
The [measurement contract](https://docs.rs/allocation-counter/0.8.1/allocation_counter/fn.measure.html) counts only the current thread (verified 2026-10-16).
Its allocator wrapper contains the unsafe system-allocator calls; Shiin adds no unsafe allocator implementation.
The ordinary latency jobs keep the default allocator, so instrumentation does not change the reported latency samples.
This admission is for measurement tooling and does not replace dependency review before release.

Every dependency is declared in `[workspace.dependencies]` with a version requirement and `default-features = false`.
Members reference it with `workspace = true` and enable only the features they use.

## Supply chain

- `cargo deny` is enforced in CI, following [ADR-0038](../adr/0038-supply-chain-policy.md); `cargo vet` is not configured.
- Dependencies are updated through ordinary pull requests.
- A dependency that gains a soundness advisory is treated as a potential vulnerability.

## Where the policy lives

`deny.toml` at the repository root is the enforced form of this policy, and `ci.yml` runs
`cargo deny check` on every pull request. Two decisions in that file are worth knowing:

- **Two extra licenses are allowed**: `Apache-2.0 WITH LLVM-exception`, which is standard
  for the Rust project's own low-level crates, and `MIT-0`, which is MIT without the
  attribution requirement. Both are permissive.
- **Wildcard versions are refused, except for path dependencies.** Shiin's own crates are
  path dependencies inside this workspace, so they carry no version requirement to review.
  An external wildcard would let an unreviewed upgrade arrive silently.

Duplicate versions warn rather than fail, because they are usually a transitive dependency's
choice rather than ours.

## Read next

- [Workspace and crates](workspace-and-crates.md)
- [Unsafe code policy](unsafe-policy.md)
