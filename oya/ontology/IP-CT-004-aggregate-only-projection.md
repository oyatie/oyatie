// NOTE: ontology extension; jointly owned axis-ontology + axis-consent-graph.

# IP-CT-004: Aggregate-only projection (k-anonymity + differential privacy) — ontology extension

- Microservice: ontology (extension)
- Bounded context: cross-tenant-projection
- Layer: usecase + worker
- Crates: `oya-ontology-cross-tenant-projection-aggregate-usecase`,
  `oya-ontology-cross-tenant-projection-aggregate-worker`
- Acceptance status: ga
- Authority: ADR-0214 §2.2 (Aggregate mode); ADR-SVC-CG-003; IP-CT-001; consent-graph IP-011 §5.

## 1. Goal

For agreements in `SharingMode::Aggregate`, ontology emits **pre-aggregated buckets** (not row-level)
to the projection topic, with k-anonymity ≥k_anonymity and optional DP noise applied. Below-k
buckets are suppressed; suppression itself is audit-emitted (per IP-011).

## 2. Scope

In:
- Aggregation window scheduling per agreement (5min / 15min / 1h / 1d configurable).
- Group-by + measure computation against ontology's denormalized aggregate tables.
- k-anonymity guard (suppress if observed_k < required_k).
- DP noise generator (Laplace mechanism; seeded from per-agreement OpenBao key).
- DP budget tracking (per-agreement spent ε; refuse aggregation when exhausted).
- Bucket emission via IP-CT-002 emitter.

Out:
- Row-level Projection mode (IP-CT-003 covers).
- AttestedQuery mode (separate request-response path).

## 3. Aggregator

```rust
pub struct AggregateWorker {
    schedule: BTreeMap<AgreementId, AggregateSchedule>,    // per-agreement next-run time
    ontology_rollup: OntologyRollupReader,                  // existing intra-tenant aggregate reader
    narrower: CrossTenantNarrower,
    emitter: CrossTenantProjectionEmitter,
    dp_noise: DpNoiseGenerator,
    dp_budget: DpBudgetTracker,
    audit_bridge: AuditBridgeClient,
}

impl AggregateWorker {
    async fn run_window(&self, agreement: &Active, window: TimeWindow) -> Result<(), Error> {
        let buckets = self.ontology_rollup
            .aggregate(agreement.scope.entity_type, agreement.terms.group_by, agreement.terms.measures, window)
            .await?;
        for bucket in buckets {
            let required_k = agreement.terms.k_anonymity.unwrap_or(5);
            if bucket.k < required_k {
                self.audit_bridge.emit_aggregate_suppressed(agreement.agreement_id, bucket.group_by).await?;
                continue;
            }
            let noised = match &agreement.terms.differential_privacy {
                Some(dp) => {
                    self.dp_budget.charge(agreement.agreement_id, dp.epsilon).await?;
                    self.dp_noise.apply(bucket, dp)?
                }
                None => bucket,
            };
            let event = ProjectionEvent::aggregate(agreement.agreement_id, noised, window);
            self.emitter.emit(&target, &event).await?;
        }
        Ok(())
    }
}
```

## 4. Differential privacy noise

Laplace mechanism per bucket measure:
- `count` measure: sensitivity = 1; noise = Laplace(scale = 1/ε).
- `sum` measure: sensitivity = max_value - min_value of underlying field; noise = Laplace(scale = sensitivity/ε).
- `avg` measure: sensitivity = sensitivity_of_sum / k; noise = Laplace(scale = sensitivity_of_sum / (k×ε)).

Seed: cryptographic PRNG (`rand_chacha::ChaCha20Rng`) seeded from OpenBao per-agreement secret. This
makes noise non-deterministic across agreements but reproducible within one agreement for audit.

## 5. Budget tracking

Per-agreement DP budget:
- Initial: ε_total from agreement.terms.differential_privacy.epsilon.
- Each aggregate query consumes ε_per_bucket × bucket_count.
- Budget exhausted → all future aggregate emits Indeterminate (effectively Deny).
- Budget reset cadence: PHASE-02 decision (currently per-agreement lifetime).

Schema:
```sql
CREATE TABLE consent_graph_dp_budget (
    agreement_id ulid PRIMARY KEY,
    epsilon_total double precision NOT NULL,
    epsilon_spent double precision NOT NULL DEFAULT 0.0,
    last_charged_at timestamptz
);
```

## 6. k-anonymity sensitive-cohort floor

If pack overlay marks the entity-type as sensitive (e.g., health, race, ideology under KR PIPA §23),
k floor is raised to ≥10 regardless of agreement.terms.k_anonymity. Enforced at narrower-level (not
just kernel) via `oya-shared-sensitive-category-classifier`.

## 7. Tests

- `aggregate_suppress_below_k` — bucket with k=4 + k_required=5 → suppressed.
- `aggregate_dp_noise_within_bound` — 1K buckets, ε=1.0; noise distribution matches Laplace(1.0) 99%.
- `dp_budget_exhausts_after_n_queries` — budget=1.0, 10 queries × 0.1 each → 11th refused.
- `sensitive_cohort_k_floor_10` — KR pack + health entity → k=8 bucket suppressed even though
  agreement.k=5.
- `noise_deterministic_within_agreement` — same input + same agreement_id → same noise.
- `noise_differs_across_agreements` — different agreements → different noise.

## 8. Performance

- Aggregation window: 5min window = ~10s compute time.
- Bucket emission: 1K buckets/sec per worker.
- 10K agreements with 5min window = 30K bucket emits per 5min = 100/s avg per region.

## 9. Verification

- `cargo build` + `cargo test`.
- E2E: aggregate-mode agreement; window computes; below-k buckets suppressed + audited; above-k
  noised + emitted.
- Privacy stress: 10K buckets with ε=1.0; noise distribution analysis confirms within Laplace bound.

## 10. Risk

- **R**: Aggregate window scheduling drift.
  **M**: Idempotent via (agreement_id, window_id) key; late windows emit with `late_arrival=true`.
- **R**: DP noise insufficient for high-sensitivity measure.
  **M**: Per-measure sensitivity analysis required at agreement-template authoring; PR review.
- **R**: Budget tracking race between concurrent queries.
  **M**: Postgres row-level lock on dp_budget row; serialization ensures monotonic decrement.

## 11. Cross-references

- IP-CT-001 (kernel types)
- microservices/consent-graph/IP-011 §5 (aggregate impl)
- microservices/consent-graph/threat-model.md §5.2 (k-anon bypass) + §5.4 (DP exhaustion)
- ADR-SVC-CG-003 (three modes)


## Counterpart Evidence

This already-substantive IP is preserved. Counterpart anchor for Wave 15 verification: Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model. See `microservices/ontology/competitor-parity-matrix.md` for the service-specific parity rows; the implementation PR must update that row when this IP materially changes parity.
