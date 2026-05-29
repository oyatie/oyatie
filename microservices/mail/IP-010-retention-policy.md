---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-010-retention-policy
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + council-privacy
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, port-location, layer-correctness, oya-governance-retention-floor-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: oya-mail-retention-policy-{kernel,domain,usecase,api,adapter,worker,app}

## Intent

Implement per-tenant + per-mailbox retention policies with statutory floor enforcement per pack (KR-FSS 5y; HIPAA 6y; GDPR Art. 5(1)(e) minimum-necessary; KR PIPA enforcement decree Art. 30 1y default). Expiry scheduler with hold-before-purge invariant. Retention ledger (append-only Ed25519-sealed).

## ChangeSet boundary

7 Rust crates. Per-pack statutory-floor configuration in `microservices/mail/policy/data-residency.md` per-pack overlays (additive edit; per CLAUDE.md user directive).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-retention-policy-kernel/` | create | `RetentionPolicy`, `RetentionClass`, `ExpiryBatch`, `RetentionLedgerEntry` + ports |
| `microservices/mail/src/crates/oya-mail-retention-policy-domain/` | create | floor arithmetic + expiry sieve + ledger emission |
| `microservices/mail/src/crates/oya-mail-retention-policy-usecase/` | create | orchestrator (load policy → enumerate candidates → enforce hold-check → soft-delete → audit) |
| `microservices/mail/src/crates/oya-mail-retention-policy-api/` | create | typed contracts |
| `microservices/mail/src/crates/oya-mail-retention-policy-adapter/` | create | policy-source provider (per-tenant config + pack-overlay) |
| `microservices/mail/src/crates/oya-mail-retention-policy-worker/` | create | nightly cron worker (02:00 per pack); idempotent |
| `microservices/mail/src/crates/oya-mail-retention-policy-app/` | create | composition root |
| `microservices/mail/policy/data-residency.md` | additive edit | per-pack retention floors (already present; verify) |
| `microservices/mail/catalog/oya-mail-retention-policy-*.yaml` × 7 | create | catalog rows |

## Code Shape

```rust
// domain/src/floor.rs
pub fn statutory_floor(pack: PackId, data_class: DataClass) -> Duration {
    match (pack, data_class) {
        (PackId::KrFss, _) => Duration::days(5*365),                  // KR commercial code
        (PackId::UsHealthcare, DataClass::PHI) => Duration::days(6*365),  // HIPAA §164.316(b)(2)
        (PackId::Eu, _) => Duration::days(7),                          // GDPR Art. 5(1)(e) minimum
        (PackId::Kr, _) => Duration::days(365),                        // KR PIPA enforcement decree Art. 30
        _ => Duration::days(7),
    }
}

// usecase/src/sweep.rs
pub async fn sweep(p: &Ports, batch: ExpiryBatch) -> Result<(), RetentionError> {
    for candidate in batch.candidates {
        if !candidate.legal_hold_ids.is_empty() {
            p.ledger.append(RetentionLedgerEntry::SkippedByHold(candidate.id)).await?;
            continue;
        }
        let configured = candidate.retention_policy.configured_floor;
        let statutory  = statutory_floor(candidate.pack, candidate.data_class);
        let floor      = configured.max(statutory);
        if candidate.age() < floor {
            p.ledger.append(RetentionLedgerEntry::SavedByFloor(candidate.id)).await?;
            continue;
        }
        p.mailbox.soft_delete(&candidate).await?;
        p.ledger.append(RetentionLedgerEntry::Expired(candidate.id)).await?;
        p.events.emit_retention_expired(candidate).await?;
    }
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-mail-retention-policy-domain
cargo nextest run -p oya-mail-retention-policy-usecase
cargo run -p oya-dev-cli -- gate validate retention-floor-conformance --microservice mail
```

## Test Plan

- Per-pack statutory floor: KR-FSS 5y enforced even when tenant configures 1y.
- Hold-check: held message survives expiry sweep.
- Idempotency: re-running sweep does not re-process already-soft-deleted messages.
- Ledger integrity: every action sealed Ed25519.
- Per-pack overlay applied: HIPAA 6y; KR-FSS 5y; KR PIPA 1y default.

## Halt Conditions

- Sweep deletes a message past statutory floor below the floor → fail; refactor.
- Sweep deletes a held message → fail; refactor (hold-before-purge invariant).


## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-010-retention-policy.md` matched `PHI`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/mail/IP-010-retention-policy.md` matched `emission`; anchors `microservices/mail/manifest.json, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Next IP

[`IP-011-legal-hold-engine.md`](IP-011-legal-hold-engine.md)

## References

- KR Commercial Code Art. 33 (5y retention for FSS-regulated comms)
- KR PIPA Enforcement Decree Art. 30 (1y audit-log default; 3y for sensitive)
- HIPAA §164.316(b)(2) (6y retention)
- GDPR Art. 5(1)(e) (storage-limitation principle)
- ISO 27001:2022 A.5.33 (retention)
- ePrivacy Directive Art. 5
- ADR-0117 (residency)
- Bominal ADR-0215 (retention/legal-hold dual-context)
