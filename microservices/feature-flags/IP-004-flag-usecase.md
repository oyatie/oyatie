# IP-004 — Flag Use-Case Crate

**microservice**: feature-flags
**bc**: flag
**layer**: usecase
**crate**: oya-feature-flags-flag-usecase
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0244, ADR-0245, ADR-0248, ADR-0263, ADR-0297
**companion_ips**: IP-002, IP-003, IP-005, IP-006

## Scope

Application commands and queries: `EvaluateFlagCommand`, `BatchEvaluateFlagsCommand`, `StreamFlagUpdatesCommand`, `CreateFlagCommand`, `UpdateFlagCommand`, `ArchiveFlagCommand`. Orchestrates domain services; enforces Cedar gates; emits observability signals.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `EvaluateFlagCommand` | Cache-first; default-off on error; emits `FlagEvaluated` event (when audit_required=true); ≤1ms p99 in-cache |
| 2 | `BatchEvaluateFlagsCommand` | Up to 100 flags per batch; parallel evaluation with `FuturesUnordered`; all-or-nothing error semantics |
| 3 | `StreamFlagUpdatesCommand` | SSE / gRPC server-streaming for flag state changes; per-tenant channel isolation |
| 4 | `CreateFlagCommand` | Cedar `FlagCreate` permit + step-up class A; calls `FlagMutationService::create` |
| 5 | `UpdateFlagCommand` | Cedar `FlagUpdate` permit + step-up class B for live flags; undo window: 15s via `PendingUndo` state |
| 6 | `ArchiveFlagCommand` | Cedar `FlagArchive` permit + step-up class B; checks no active rollouts |
| 7 | Abuse-defence integration | Rate check: >60 mutations/min → Cedar `FORBID`; EMERGENCY_SERVICES bypass via `audience_type` header |
| 8 | Tests | Undo-window test; rate-limit enforcement test; EMERGENCY_SERVICES bypass test |

## Definition of Done

- All 8 use-cases wired to domain services with Cedar pre-check
- `cargo test -p oya-feature-flags-flag-usecase` green
- Undo window: `UpdateFlag` within 15s can be reversed without step-up
- Rate limiter fires at mutation #61 for non-EMERGENCY_SERVICES principals
