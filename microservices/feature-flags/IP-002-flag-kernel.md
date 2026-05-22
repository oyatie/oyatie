# IP-002 — Flag Kernel Crate

**microservice**: feature-flags
**bc**: flag
**layer**: kernel
**crate**: oya-feature-flags-flag-kernel
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0242, ADR-0243, ADR-0244, ADR-0248, ADR-0252, ADR-0263
**companion_ips**: IP-003, IP-004, IP-005

## Scope

Innermost-ring data model and evaluation primitives for the `flag` BC. Zero outward dependencies — only `oya-shared-policy-eval`, `oya-shared-hlc`, and `oya-shared-tenant-context` permitted.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `FlagDefinition` struct | Fields: flag_key, tenant_id, flag_type (bool/string/number/json), variants, default_off_treatment, rollout_stage, sunset_at, pack_locked_fields, hlc_created, hlc_updated |
| 2 | `FlagAssignment` struct | Fields: flag_key, tenant_id, user_id_hash (one-way HMAC), variant, reason (OpenFeature ResolutionReason), evaluation_timestamp_hlc |
| 3 | `FlagEvaluator` trait | Async `evaluate(ctx: EvaluationContext) -> FlagAssignment`; default-off on error |
| 4 | `FlagCache` | DashMap<(tenant_id, flag_key), FlagDefinition>; TTL 30s; LKG 30-min disk tier |
| 5 | Cedar policy fragments | Default-deny `FlagRead`; pack-locked field guard |
| 6 | Unit tests | ≥90% branch coverage; property tests for HLC ordering |

## Capacity Targets

- Single `evaluate()` call: ≤50µs (in-cache)
- Cache hit rate: ≥99% under steady state
- HLC tick: monotonic; drift ≤1ms vs NTP

## Definition of Done

- `cargo test -p oya-feature-flags-flag-kernel` green
- Cedar fragment passes `cedar validate`
- No outbound network calls in kernel layer (enforced via `#[forbid_net_calls]` lint)
- `lsp_diagnostics` zero errors
