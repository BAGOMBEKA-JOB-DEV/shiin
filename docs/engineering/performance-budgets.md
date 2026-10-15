<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-15 -->

# Performance budgets

> [!NOTE]
> **Project policy: draft.**
> This page is the only home of Shiin's performance budgets.

## Latency

| Path | Budget | Platform |
|---|---|---|
| Hook evaluation | 50 ms | Tier 1 |
| Hook total including process start | 200 ms | Tier 1 |
| Policy compilation | 1 s | Tier 1 |

## Memory

| Path | Budget |
|---|---|
| Engine evaluation | 10 MB |
| Policy compilation | 100 MB |

## Measurement

Cross-platform CI measurement remains a release requirement and is not yet configured.
The following command provides a repeatable local latency sample:

```console
$ cargo build --release --workspace --locked
$ cargo run --release --locked -p xtask -- performance target/release/shiin
```

The command checks every observed latency against the existing budgets and reports median, p95, and maximum.
Hook samples include process start, policy loading, evaluation, and durable audit writes against a small growing log.
They exercise the real hook executable without launching a host.
The reported Linux process peak RSS includes the harness and dependencies; it does not establish the separate engine and policy memory budgets.
Large-policy/log measurements, component memory profiling, and results on other platforms remain open.

## Read next

- [Testing strategy](testing-strategy.md)
- [Platform support](../reference/platform-support.md)
