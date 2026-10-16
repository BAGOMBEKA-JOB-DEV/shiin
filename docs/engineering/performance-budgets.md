<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-16 -->

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

The `ci.yml` release-latency jobs run the following measurements on hosted Linux and macOS runners.
The same commands provide a repeatable local latency sample:

```console
$ cargo build --release --workspace --locked
$ cargo run --release --locked -p xtask -- performance target/release/shiin
$ cargo run --release --locked -p xtask -- performance-scaling target/release/shiin
```

The command checks every observed latency against the existing budgets and reports median, p95, and maximum.
Hook samples include process start, policy loading, evaluation, and durable audit writes against a small growing log.
They exercise the real hook executable without launching a host.
The scaling command checks engine/compilation latency with 100 and 500 rules and complete hook latency with 100 rules and an audit log seeded with 1,000 events.
The hosted Linux and macOS jobs run both commands against the same budgets and refuse to measure a fixture that returns a denial.
The reported Linux process peak RSS includes the harness and dependencies; it does not establish the separate engine and policy memory budgets.
Audit append and verification now stream existing records, retaining the current chain head instead of all prior event payloads.
Append still checks the entire chain and syncs the new record before success; tests seed a thousand-event chain and reject early, middle, and late corruption without changing the file.
Reading the log for display still collects its records, and individual-record bounds and component peak-memory measurements remain open.
Workloads beyond those bounded scaling samples, component memory profiling, and results on other platform architectures remain open.

## Read next

- [Testing strategy](testing-strategy.md)
- [Platform support](../reference/platform-support.md)
