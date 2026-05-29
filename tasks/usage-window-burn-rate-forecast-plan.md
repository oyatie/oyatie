# Plan: usage-window-burn-rate-forecast

## Objective

Add a pure deterministic burn-rate forecasting API to the existing
`oya-intelligence-usage-window-kernel` crate. The new method
`UsageEnforcement::forecast` computes a linear projection of how long before
the two enforcement thresholds are breached.

## Requirements Analysis

### Core computation

- **Elapsed time**: `elapsed = now - started_at` (u64, saturating)
- **Used tokens**: `used = tokens_in + tokens_out` (u64, saturating)
- **Burn rate**: `rate = used / elapsed` (tokens per second; integer division)
- **ETA to usage_limit_pct**: tokens remaining until limit / rate
- **ETA to reserve_breach**: tokens remaining above reserve floor / rate
- Both ETAs must be capped at `remaining_wall_secs = ends_at - now`

### Edge cases

| Condition | Behaviour |
|-----------|-----------|
| `ends_at <= started_at` | `Err(InvalidWindow)` |
| `budget_tokens == 0` | `Err(InvalidWindow)` |
| `now < started_at` | `Err(ClockBeforeWindowStart)` |
| `now >= ends_at` | `Ok(UsageForecast { exhaustion: WindowExpired })` |
| `elapsed == 0 && used == 0` | `Ok(UsageForecast { exhaustion: NoBurn })` |
| `elapsed == 0 && used > 0` | treated as zero-elapsed ⇒ `NoBurn` (guard) |
| `used >= budget * usage_limit_pct / 100` | `Ok(AlreadyBreached)` |
| `used >= budget * (100 - reserve_remaining_pct) / 100` | `Ok(AlreadyBreached)` |
| burn_rate == 0 (no burn after elapsed > 0) | `Ok(NoBurn)` |

### ExhaustionForecast enum

```
UsageLimitEtaSecs(u64)    — ETA to usage_limit_pct breach
ReserveBreachEtaSecs(u64) — ETA to reserve floor breach (precedes limit when reserve is higher)
NoBurn                    — zero burn rate; no projected breach
AlreadyBreached           — already past a threshold right now
WindowExpired             — now >= ends_at
```

When both ETAs are finite, return the **smaller** one (earliest breach).

### Saturating arithmetic

All arithmetic uses `saturating_*` or `u128` intermediate to prevent overflow
on `u64::MAX` inputs. No panics permitted.

## Subtasks

1. [x] Write plan (this file)
2. [ ] Write spec (`docs/specs/task-usage-window-burn-rate-forecast.md`)
3. [ ] Write RED tests in `lib.rs` (table-driven, covering all acceptance criteria)
4. [ ] Confirm RED: `cargo check -p oya-intelligence-usage-window-kernel --all-targets`
5. [ ] Implement `UsageForecast` + `ExhaustionForecast` types + `UsageEnforcement::forecast`
6. [ ] Confirm GREEN: `cargo nextest run -p oya-intelligence-usage-window-kernel`
7. [ ] Self-review (correctness / security / perf / cloud-native)
8. [ ] Simplify: guard clauses, dead-code removal, naming pass; re-run nextest
9. [ ] Commit and push

## Acceptance Criteria

1. Constant burn yields exact integer ETA matching hand-computed fixtures.
2. ETA never exceeds remaining wall-clock to `ends_at`.
3. Zero usage → `NoBurn`; current breach → `AlreadyBreached`; `now >= ends_at` → `WindowExpired`;
   `now < start` → `ClockBeforeWindowStart`; `budget == 0` / `ends <= start` → `InvalidWindow`.
4. Reserve-breach ETA precedes usage-limit ETA when reserve floor is higher.
5. Saturating arithmetic; no panics/overflow on `u64::MAX` usage.
6. Pure no-I/O; only account-domain types reused.
7. Table-driven unit tests in `lib.rs` all pass.
