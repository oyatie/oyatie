# Spec: obs-cloud-domain-audit-topic-rollup

**Crate**: `cloud-observability-domain`
**Slice kind**: Pure deterministic domain extension — no I/O, no new external deps.

## Context

`CloudObservabilityCatalog::read_audit` already implements a validated, filtered page read over
`CloudAuditRecord`. This slice adds a rollup projection that runs the same validation + filter but
instead of returning a page of records returns an `AuditReadSummary` aggregate.

## Public Surface

### `AuditReadSummary`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditReadSummary {
    pub total: u64,
    pub per_topic: BTreeMap<CloudAuditTopic, u64>,
    pub earliest_epoch_seconds: Option<u64>,
    pub latest_epoch_seconds: Option<u64>,
    pub chain_complete: bool,
    pub high_watermark_sequence: Option<u64>,
}
```

### `CloudObservabilityCatalog::summarize_audit`

```rust
pub fn summarize_audit(
    &self,
    request: AuditReadRequest,
) -> Result<AuditReadSummary, CloudObservabilityError>
```

## Invariants

| # | Invariant |
|---|-----------|
| I1 | `total` == count of records where `normalized.matches(record)` is true |
| I2 | `per_topic.values().sum() == total` |
| I3 | `earliest_epoch_seconds` is `None` iff `total == 0` |
| I4 | `latest_epoch_seconds` is `None` iff `total == 0` |
| I5 | When `total > 0`: `earliest_epoch_seconds.unwrap() <= latest_epoch_seconds.unwrap()` |
| I6 | `IncompleteAuditChain` iff `require_complete_chain && !self.chain_verified` |
| I7 | Validation errors (`InvalidReadWindow`, `InvalidPageSize`, `InvalidAuditTopic`) identical to `read_audit` |
| I8 | `per_topic` is ordered by `CloudAuditTopic` (derived `Ord`) — deterministic iteration |
| I9 | No cursor applied — full-window rollup |

## Validation Flow (identical to `read_audit`)

1. `NormalizedAuditReadRequest::new(request)` — validates window, page_size, topics, scope, cell, actor, resource
2. Check `require_complete_chain && !self.chain_verified` → `IncompleteAuditChain`
3. Filter `self.audit_records.values()` with `normalized.matches(record)`
4. Aggregate: count total, accumulate per-topic, track min/max `occurred_at_epoch_seconds`

## Test Matrix

| Test | Description |
|------|-------------|
| `summarize_audit_empty_window` | No records in catalog → total=0, per_topic empty, timestamps None |
| `summarize_audit_multi_topic` | 3 records across 2+ topics → per_topic sums to total, timestamps correct |
| `summarize_audit_scope_control_plane` | scope=ControlPlaneMutations filters out DataPlaneSecurity records |
| `summarize_audit_incomplete_chain_rejected` | require_complete_chain on unverified chain → IncompleteAuditChain |
| `summarize_audit_invalid_window_rejected` | start >= end → InvalidReadWindow |
| `summarize_audit_invalid_topic_for_scope_rejected` | ControlPlaneMutations scope + non-CP topic → InvalidAuditTopic |
