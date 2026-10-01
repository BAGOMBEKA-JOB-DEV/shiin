<!-- shiin-doc: kind=policy status=draft implementation=n/a milestone=n/a reviewed=2026-10-01 -->

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

These budgets are measured in CI on every Tier 1 platform.
A regression opens a bug, not an exception.

## Read next
- [Testing strategy](testing-strategy.md)
- [Platform support](../reference/platform-support.md)