// NOTE: ontology extension; jointly owned axis-ontology + axis-consent-graph.

# IP-CT-005: Sovereignty-preserving zero-copy projection contract — ontology extension

- Microservice: ontology (extension)
- Bounded context: cross-tenant-projection
- Layer: invariant + verification
- Crate: `oya-ontology-cross-tenant-projection-sovereignty`
- Acceptance status: ga
- Authority: ADR-0214 §2.5; ADR-SVC-CG-004; data-residency.md §2; consent-graph IP-009 §5.

## 1. Goal

Encode + enforce the **zero-copy sovereignty contract** at the ontology projection boundary so that
the grantor's authoritative row NEVER physically migrates to the grantee's region/cell. The grantee
only ever sees a denormalized projection in grantor's Pulsar cluster (which they read
cross-region) — the row stays put.

This IP is the *invariant + verification* layer that complements IP-CT-002 (the emitter) by
adding pre-emit + post-emit assertions and a nightly audit job.

## 2. Scope

In:
- Pre-emit invariant assertion: target.region == agreement.sovereignty.grantor_region AND target
  Pulsar cluster is in same region.
- Post-emit assertion: emitted message arrived in correct cluster (verified via async readback).
- Daily sovereignty audit job: enumerate all active cross-tenant topics; cross-check against
  Pulsar admin API + agreement records.
- PII residency classifier: per-pack rules on what may cross borders even with explicit consent.

Out:
- Topic mint (consent-graph IP-010 owns).
- Grantee-side projection cache management (separate ontology IP — grantee-side ontology has its
  own projection cache subject to its own sovereignty rules).

## 3. Pre-emit invariant

Before every `emitter.emit(target, event)`:

```rust
fn assert_pre_emit_sovereignty(target: &CrossTenantProjectionTarget,
                                agreement: &DataSharingAgreement)
    -> Result<(), SovereigntyError>
{
    // 3.1 target region == grantor region
    if target.sovereignty_pin != agreement.sovereignty.grantor_region {
        return Err(SovereigntyError::TargetRegionMismatch);
    }
    // 3.2 Pulsar cluster region check
    let cluster_region = pulsar_cluster_region(&target.topic_name)?;
    if cluster_region != agreement.sovereignty.grantor_region {
        return Err(SovereigntyError::PulsarClusterMismatch);
    }
    // 3.3 grantee region permitted
    if !agreement.sovereignty.permitted_grantee_regions.iter().any(|r| current_grantee_region() == r) {
        return Err(SovereigntyError::GranteeRegionNotPermitted);
    }
    // 3.4 cross-border classifier
    if !agreement.sovereignty.cross_border_transfer_permitted &&
       agreement.sovereignty.grantor_region != current_grantee_region() {
        return Err(SovereigntyError::CrossBorderTransferForbidden);
    }
    Ok(())
}
```

A failure increments `oya_consent_graph_sovereignty_violation_total` (P0 SLO breach).

## 4. Post-emit assertion (sampled)

For 1% sampled emits:
1. Async readback from a verifier consumer subscribed to grantor-region cluster.
2. Verify message arrived in correct cluster + topic + region.
3. If mismatch: P0 alert + auto-suspend agreement.

## 5. Daily sovereignty audit job

`ontology-cross-tenant-sovereignty-audit-worker` runs daily 02:00 UTC:
1. List all active agreements: `consent-graph-sdk::list_active`.
2. For each, query Pulsar admin API for actual topic location.
3. Verify: topic.region == agreement.sovereignty.grantor_region.
4. Verify: topic ACL is grantee-tenant-only.
5. Verify: no geo-replication unless agreement.sovereignty.geo_replicate_to_grantee_region == true.
6. Emit report `evidence/sovereignty-audit-<date>.json`; seal in audit-chain.

Any mismatch triggers `regional-sovereignty-violation.md` runbook + auto-suspends affected agreements.

## 6. PII residency classifier

Per pack overlay rules, certain fields are flagged "cross-border-forbidden" regardless of explicit
consent:
- KR strict-residency: `national_id`, `resident_registration_number`, `biometric_data`
- US-Healthcare: full PHI categories per HIPAA min-necessary
- EU: special category data per GDPR Art. 9

Classifier in `oya-shared-pii-classifier`; consumed by IP-CT-003 narrower AND IP-CT-005 invariant.

```rust
fn assert_no_residency_violation(payload: &JsonValue, sovereignty: &SovereigntyCfg)
    -> Result<(), SovereigntyError>
{
    let cls = PiiClassifier::for_pack(sovereignty.residency_overlay_pack.unwrap_or_default());
    for (field, value) in payload.iter() {
        let category = cls.classify(field, value);
        if category.is_cross_border_forbidden() && needs_cross_border(sovereignty) {
            return Err(SovereigntyError::PiiCategoryForbidden {
                field: field.clone(), category,
            });
        }
    }
    Ok(())
}
```

## 7. Zero-copy contract test

```rust
#[test]
fn raw_row_never_physically_migrates() {
    // Setup: tenant A in us-east-1, tenant B in eu-west-1.
    // Agreement: A grants B read of FinishedGoodsInventory.
    let agreement = make_agreement(/* ... */);
    // Emit a row from A's ontology.
    let row = make_row("FinishedGoodsInventory");
    cross_tenant_emitter.emit_for_agreement(&agreement, &row).await?;

    // Assertion 1: A's Postgres row is untouched (only ontology projection events emitted).
    assert_postgres_row_unchanged(&row);

    // Assertion 2: Pulsar topic resides in us-east-1 cluster.
    let cluster = pulsar_cluster_region(&agreement.projection_topic).await?;
    assert_eq!(cluster, "us-east-1");

    // Assertion 3: NO copy of the row exists in eu-west-1 cluster, eu-west-1 Postgres, or anywhere
    // outside us-east-1.
    assert_no_row_in_region(&row, "eu-west-1").await?;

    // Assertion 4: B's read goes via cross-region Pulsar consumer; B's local cache is a denorm
    // projection (NOT a copy of A's authoritative row).
    let b_view = b_ontology.read_projection(&agreement.agreement_id, &row.entity_id).await?;
    assert_is_projection_not_copy(&b_view);
}
```

## 8. Tests

- `assert_pre_emit_target_region_mismatch_blocks` — synthetic wrong region → blocked.
- `assert_pre_emit_pulsar_cluster_mismatch_blocks` — synthetic cluster mismatch → blocked.
- `assert_pre_emit_cross_border_forbidden` — !cross_border_transfer_permitted + foreign grantee → blocked.
- `pii_classifier_kr_pack_national_id_forbidden` — KR pack + national_id field → blocked.
- `sovereignty_audit_job_reports_zero_violations_synthetic` — 100 synthetic agreements → 0 violations.
- `sovereignty_audit_job_detects_synthetic_violation` — inject 1 misconfigured topic → reported P0.
- `zero_copy_contract_e2e` — full integration test (per §7).
- `post_emit_assertion_sampled_one_percent` — 100K emits, ~1K verifier readbacks.

## 9. Performance

- Pre-emit invariant: <100μs (in-memory check).
- Post-emit readback (1% sample): async; no impact on emit hot path.
- Daily audit: 10K agreements in ≤10min single-threaded; parallelized to ≤2min.

## 10. Dependencies

- IP-CT-001 (kernel)
- `oya-shared-pii-classifier`
- `oya-consent-graph-projection-gateway-sdk` (for topic ↔ region lookup)
- `pulsar-rs = "6"`

## 11. Verification

- `cargo build` + `cargo test`.
- E2E test §7 (`raw_row_never_physically_migrates`) passes.
- Daily audit job runs in dev cluster + produces zero-violation report.
- Synthetic violation injection produces P0 alert.

## 12. Risk

- **R**: Pre-emit invariant adds 100μs to every emit.
  **M**: At 1M emit/s/region, that's 100s of CPU/sec across cluster — acceptable; alternative is
  defense-out-of-depth, which is unacceptable.
- **R**: Post-emit verifier consumer falls behind.
  **M**: Verifier samples 1%; backlog acceptable up to 5min; alert at 10min.
- **R**: Daily audit job times out at scale.
  **M**: Parallelism + checkpointing; per-agreement audit is independent; failed audits flagged for retry.

## 13. Cross-references

- ADR-0214 §2.5 sovereignty
- ADR-SVC-CG-004 grantor-region authority
- microservices/consent-graph/data-residency.md
- microservices/consent-graph/runbooks/regional-sovereignty-violation.md
- microservices/consent-graph/IP-009 (kernel invariants)
- microservices/consent-graph/threat-model.md §9.5 (sovereignty bypass)


## Counterpart Evidence

This already-substantive IP is preserved. Counterpart anchor for Wave 15 verification: Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model. See `microservices/ontology/competitor-parity-matrix.md` for the service-specific parity rows; the implementation PR must update that row when this IP materially changes parity.
