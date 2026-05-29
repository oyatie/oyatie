# IP-011: projection-gateway-usecase — scope narrowing + aggregate-mode k-anonymity emission

- Bounded context: projection-gateway
- Layers: domain, usecase, worker
- Crates:
  - `oya-consent-graph-projection-gateway-domain`
  - `oya-consent-graph-projection-gateway-usecase`
  - `oya-consent-graph-projection-gateway-worker`
  - `oya-consent-graph-projection-gateway-sdk`
- Acceptance status: ga
- Authority: ADR-0214 §2.2 (three modes), §2.5 (scope-narrowed projection), ADR-SVC-CG-003 (three
  sharing modes), ADR-0058 (ontology emission pipeline).
- Depends on: `oya-consent-graph-projection-gateway-{kernel}`, `oya-consent-graph-agreement-{kernel,
  domain}`, ontology cross-tenant projection extension (IP-CT-001..005).

## 1. Goal

For each raw entity event flowing through Ontology, *intercept* the projection that targets a
cross-tenant agreement, *narrow* the payload to the agreement's scope (field-level redaction), and —
for Aggregate mode — *bucketize* with k-anonymity + optional differential privacy noise before
emission to the projection topic.

This IP turns the raw Ontology projection stream into a Cedar-enforced, scope-narrowed,
sovereignty-pinned event stream that the grantee consumes.

## 2. Scope

In:
- `ScopeNarrower` impl (pure transformation).
- `Aggregator` impl (pure transformation; with k-anon + DP).
- `EmitWorker` (consumes Ontology emission stream, narrows, emits to projection topic).
- Three mode-specific pipelines: Projection / Aggregate / AttestedQuery.

Out:
- Subscriber side (in Ontology cross-tenant extension).
- Pulsar topic mint/destroy (IP-010).
- Cedar enforcement (IP-006).

## 3. Projection mode pipeline

For each raw entity event from Ontology that matches an active agreement:

```rust
async fn project_row_mode(&self, agreement: &Active, raw: &OntologyRow) -> Result<(), Error> {
    // 3.1: enforcement check (re-verify; defense-in-depth even though ontology checked upstream)
    let decision = self.enforcement_sdk.check_project_emit(...).await;
    if !decision.is_permit() { return Err(Error::Denied(decision)); }

    // 3.2: narrow
    let narrowed = self.narrower.narrow(&raw.payload, &agreement.scope, &agreement.terms)?;

    // 3.3: redaction
    let redacted_fields = self.narrower.apply_redaction(&narrowed, &agreement.terms.redaction)?;

    // 3.4: sovereignty re-assert
    self.kernel.assert_grantor_region(&self.topic, agreement.sovereignty.grantor_region)?;

    // 3.5: PII classifier guard (no cross-border-forbidden classifier leaks through)
    self.pii_classifier.assert_no_residency_violation(&narrowed, &agreement.sovereignty)?;

    // 3.6: emit
    let event = ProjectionEvent {
        event_id: Ulid::now(),
        agreement_id: agreement.agreement_id,
        entity_type: raw.entity_type.clone(),
        entity_id: Some(raw.entity_id.clone()),
        payload: ProjectionPayload::Row(narrowed),
        emitted_at: Timestamp::now(),
        schema_version: agreement.schema_version,
        redaction_applied: redacted_fields,
    };
    self.emitter.emit(self.topic.topic_id, event).await?;

    // 3.7: audit
    self.audit_bridge.emit_projection_emit(...).await?;
    Ok(())
}
```

## 4. Scope narrowing (`narrow` function)

```rust
impl ProjectionScopeNarrower for ScopeNarrowerImpl {
    fn narrow(&self, raw: &JsonValue, scope: &EntityScope, terms: &SharingTerms)
        -> Result<JsonValue, NarrowError>
    {
        let resolved = self.resolve_field_set(&scope.field_set, &raw)?;
        let mut out = JsonValue::Object(Default::default());
        for field in &resolved {
            if let Some(v) = raw.get_path(field) {
                let v = self.apply_redaction_to_field(v, field, &terms.redaction)?;
                out.set_path(field, v);
            }
        }
        Ok(out)
    }
}
```

Redaction modes per-field:
- `Mask` — replace with a fixed-length asterisk pattern.
- `HashSha256` — deterministic salted hash (salt per-agreement, in OpenBao).
- `Null` — drop the field entirely.
- `RangeBucket{step}` — for numeric values; round-down to step (e.g., income → $10K buckets).
- `DateBucket{granularity}` — date/time bucket (e.g., day-of-week, month).
- `KeepRaw` — passthrough (only when scope allows).

## 5. Aggregate mode pipeline

The aggregator runs on the grantor side; the emission is one row per (group-by tuple, aggregation
window), only if observed `k ≥ k_anonymity`.

```rust
async fn project_aggregate_mode(&self, agreement: &Active, batch: &[OntologyRow]) -> Result<(), Error> {
    let buckets = self.aggregator.aggregate(batch.iter().map(|r| &r.payload).collect(), &agreement.terms)?;
    for bucket in buckets {
        if bucket.k < agreement.terms.k_anonymity.unwrap_or(5) {
            // observed k below threshold: SUPPRESS this bucket
            self.audit_bridge.emit_aggregate_suppressed(bucket.group_by.clone(), agreement.agreement_id).await?;
            continue;
        }
        let noised = match &agreement.terms.differential_privacy {
            Some(dp) => self.dp_noise.apply(bucket, dp)?,
            None => bucket,
        };
        let event = ProjectionEvent {
            // ...
            payload: ProjectionPayload::Aggregate(noised),
            // ...
        };
        self.emitter.emit(self.topic.topic_id, event).await?;
    }
    Ok(())
}
```

Differential-privacy noise generator: Laplace mechanism with sensitivity = 1 (counts) or
`max_value - min_value` (sums); ε derived from `terms.differential_privacy.epsilon`. Cryptographic
PRNG via `rand_chacha::ChaCha20Rng` seeded from OpenBao per-agreement secret (so noise is
non-deterministic across agreements but reproducible within one agreement for audit reconstruction).

## 6. AttestedQuery mode pipeline

Request/response, not push-stream. The grantee `partner-directory-rest` submits a query; consent-graph
forwards to grantor-side `attested-query-worker`; grantor evaluates the query against Ontology with
the agreement's scope as the *only* permitted predicate; returns a signed answer.

```rust
pub struct AttestedAnswerPayload {
    pub query_id: Ulid,
    pub agreement_id: AgreementId,
    pub answer: JsonValue,
    pub answered_at: Timestamp,
    pub signature: HmacSha256Bytes,           // signed by grantor-region OpenBao key
    pub witness_audit_seal: ChainLink,        // audit-chain seal id for the answer event
}
```

The grantee verifies the signature using the grantor's published key (fetched via partner-directory
handshake) and the audit-chain seal proof.

## 7. Tests

- `narrow_keeps_only_allow_listed_fields` — Allow{[a,b]} on {a,b,c,d} → {a,b}.
- `narrow_redaction_mask` — Mask redaction on a→"****".
- `narrow_redaction_hash_deterministic_same_agreement` — same input + same agreement → same hash.
- `narrow_redaction_hash_different_across_agreements` — salt differs by agreement.
- `aggregate_suppresses_below_k_anon` — bucket with k=4 + k_anonymity=5 → suppressed.
- `aggregate_dp_noise_within_epsilon_bound` — 1000 buckets, count noise bounded by Laplace(1/ε).
- `attested_query_signature_verifiable` — answer signed; grantee key verifies; tampered → fail.
- `pii_classifier_blocks_cross_border_forbidden_field` — KR-pack agreement + email field → error.

## 8. SDK (`projection-gateway-sdk`)

Used by `agreement-usecase::AcceptAgreement` to call `mint` and by `revocation-usecase` to call
`destroy`. Internal-only SDK (not exposed to partner integrations).

## 9. Worker

`projection-gateway-worker` runs:
- `EmitLoop`: consumes Ontology's internal `oya.ontology.entity-change.v1` stream, joins against
  active-agreements view, dispatches each event to the appropriate mode pipeline.
- `AggregateScheduler`: for each Aggregate-mode agreement, schedules a `aggregation_window`
  computation (e.g., every 5min for weekly windows, every 1min for daily).
- `AttestedQueryHandler`: long-poll for incoming queries from partner-directory.

## 10. Latency budgets

- Projection mode emit: p99 ≤500ms grantor-commit → emitted (per SLO `cross-tenant-projection-freshness`).
- Aggregate mode emit: p99 ≤window_size + 60s (window must elapse + processing).
- AttestedQuery answer: p99 ≤5s (per ADR-0214 §2.2 table).

## 11. Verification

- `cargo build` + `cargo test` clean.
- Integration test: raw Ontology event in → narrowed event out on projection topic; field outside
  scope absent from output.
- Stress test: 10K rows/s through narrower + emit; p99 ≤500ms.
- Privacy test: 1000 aggregate buckets, ε=1.0 noise stays within Laplace-bound 99% of time.

## 12. Risk

- **R**: PII classifier misclassifies a field → leak across border.
  **M**: Classifier is data-driven (`oya-shared-pii-classifier` JSON catalogue); new fields require
  ADR + classifier-PR; default classification on unknown field is "private" (fail-closed).
- **R**: Aggregate window timer drift → some windows emit late.
  **M**: Idempotent emission via `(agreement_id, window_id)` key; late windows still arrive but tagged
  with `late_arrival=true` and audit-emitted as such.
- **R**: DP noise budget exhausted across many queries.
  **M**: Per-agreement noise budget tracked in `consent_graph_dp_budget` table; over-budget queries
  return Indeterminate (effectively Deny); ADR-SVC-CG-* governs budget reset cadence.
- **R**: AttestedQuery query injection (grantee crafts query that bypasses agreement scope).
  **M**: Query is parsed by ontology query-domain, then re-checked against agreement scope; raw query
  text never reaches Ontology unfiltered.

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: OneTrust/TrustArc/Cookiebot model purposes and preferences; Snowflake/Databricks/BigQuery expose table/view scope. This IP combines both sets into Oyatie-specific runtime behavior: field narrowing, aggregate-mode k-anonymity, differential privacy budget checks, and AttestedQuery revalidation before any projection leaves the grantor region.
