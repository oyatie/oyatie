# Plan: obs-cloud-domain-audit-topic-rollup

## Objective

Extend `CloudObservabilityCatalog` with `summarize_audit()` — a pure domain
read-side rollup projection that reuses the existing `NormalizedAuditReadRequest`
filter predicate WITHOUT paging/cursoring.

## Scope

Single crate: `oya-cloud-observability-domain` (`crates/oya-cloud-observability-domain/src/lib.rs`).
No new dependencies. No new workspace members.

## Types to Add

### `AuditReadSummary`

```rust
pub struct AuditReadSummary {
    pub total: u64,
    pub per_topic: BTreeMap<CloudAuditTopic, u64>,
    pub earliest_epoch_seconds: Option<u64>,
    pub latest_epoch_seconds: Option<u64>,
    pub chain_complete: bool,
    pub high_watermark_sequence: Option<u64>,
}
```

### Method on `CloudObservabilityCatalog`

```rust
pub fn summarize_audit(
    &self,
    request: AuditReadRequest,
) -> Result<AuditReadSummary, CloudObservabilityError>
```

## Behavioural Contract

1. Run identical validation via `NormalizedAuditReadRequest::new()` — same
   `InvalidReadWindow`, `InvalidPageSize`, `InvalidAuditTopic` rejection paths.
2. Apply `require_complete_chain` guard identically to `read_audit`:
   if `require_complete_chain && !self.chain_verified` → `IncompleteAuditChain`.
3. Apply `normalized.matches(record)` predicate over all `audit_records` values.
4. Aggregate matched records:
   - `total` = count of matched records (u64, saturating)
   - `per_topic` = BTreeMap count per `CloudAuditTopic`, each entry saturating
   - `earliest_epoch_seconds` = min `occurred_at_epoch_seconds` (None if empty)
   - `latest_epoch_seconds` = max `occurred_at_epoch_seconds` (None if empty)
5. No cursor logic; no page truncation.
6. `chain_complete` = `self.chain_verified`, `high_watermark_sequence` = `self.high_watermark_sequence`.

## Invariants

- `per_topic` values sum to `total`
- `earliest <= latest` when both `Some`
- Both `None` when no records matched
- BTreeMap provides deterministic ordering by `CloudAuditTopic: Ord`

## Test Coverage

- empty window (no ingested records) — total=0, per_topic empty, earliest/latest None
- multi-topic — ingested chain with CloudResourceCreated + CloudIamPolicy + CloudKmsUse;
  AllTenantAudit returns total=3, per_topic with 3 entries summing to 3
- scope=ControlPlaneMutations filtering — only ControlPlaneMutation topics counted
- incomplete-chain rejection — `require_complete_chain=true` on unverified catalog
- invalid-window rejection — start >= end
- invalid-page-size is irrelevant (page_size unused in summarize), but the field is
  still validated by NormalizedAuditReadRequest::new; test page_size=0 still fails

## Steps

1. Add `AuditReadSummary` struct after `AuditReadResult`
2. Add `summarize_audit` method on `CloudObservabilityCatalog` after `read_audit`
3. Write tests (red → green)
4. cargo check + cargo nextest run
