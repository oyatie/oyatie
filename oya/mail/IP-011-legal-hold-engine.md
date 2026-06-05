---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-011-legal-hold-engine
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + council-privacy + ops-legal
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, port-location, layer-correctness, oya-governance-personal-pillar-hold-forbidden, oya-governance-ediscovery-chain-of-custody]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: oya-mail-legal-hold-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}

## Intent

Implement the legal-hold engine: scoped hold; hold-before-purge invariant; four-eyes approval for engage + release + plaintext disclosure; Ed25519 chain-of-custody seal; eDiscovery export job (eDiscovery file format per EDRM XML). Personal-pillar holds forbidden per `policy/dual-context-isolation.md` Invariant DCI-04.

## ChangeSet boundary

9 Rust crates spanning legal-hold BC. Cedar policy fragment for four-eyes.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-legal-hold-kernel/` | create | `LegalHold`, `HoldScope`, `HoldApproval`, `ChainOfCustodySeal`, `EDiscoveryExportJob` + ports |
| `microservices/mail/src/crates/oya-mail-legal-hold-domain/` | create | four-eyes verification; pillar check; seal math; chain assembly |
| `microservices/mail/src/crates/oya-mail-legal-hold-usecase/` | create | orchestrator (engage/release/export) |
| `microservices/mail/src/crates/oya-mail-legal-hold-api/` | create | typed contracts |
| `microservices/mail/src/crates/oya-mail-legal-hold-adapter/` | create | adapter to mailbox-store + audit-chain |
| `microservices/mail/src/crates/oya-mail-legal-hold-rest/` | create | REST handlers per OpenAPI 3.2.0 |
| `microservices/mail/src/crates/oya-mail-legal-hold-worker/` | create | export-job worker (long-running; chunked S3 read; encryption; chain-of-custody append) |
| `microservices/mail/src/crates/oya-mail-legal-hold-sdk/` | create | client SDK for tenant compliance officer tooling |
| `microservices/mail/src/crates/oya-mail-legal-hold-app/` | create | composition root |
| `microservices/mail/catalog/oya-mail-legal-hold-*.yaml` × 9 | create | catalog rows |

## Code Shape

```rust
// usecase/src/engage.rs
pub async fn engage_hold(req: EngageRequest, p: &Ports) -> Result<LegalHold, LegalHoldError> {
    // pillar check (DCI-04)
    let mbs = p.mailbox.resolve_scope(&req.scope).await?;
    for mb in &mbs {
        if mb.context_kind == ContextKind::Personal {
            p.metrics.inc("mail_personal_pillar_hold_attempt_total");
            return Err(LegalHoldError::PersonalPillarForbidden);
        }
    }
    // four-eyes check
    if req.approver_a.oidc_subject == req.approver_b.oidc_subject {
        return Err(LegalHoldError::SameApprover);
    }
    if (req.approver_a.signed_at - req.approver_b.signed_at).abs() > Duration::minutes(5) {
        return Err(LegalHoldError::ApproverSignatureWindowExceeded);
    }
    p.verify_signature(&req.approver_a)?;
    p.verify_signature(&req.approver_b)?;
    // engage
    let hold = LegalHold::new(req.scope, [req.approver_a, req.approver_b]);
    p.hold_store.write(&hold).await?;
    let seal = p.audit_chain.emit_seal("LegalHoldEngaged", &hold).await?;
    p.events.emit_legal_hold_engaged(&hold, &seal).await?;
    Ok(hold)
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-mail-legal-hold-domain
cargo nextest run -p oya-mail-legal-hold-usecase
buck2 build //:quality-lane-registry-authority-check # lane=personal-pillar-hold-forbidden
buck2 build //:quality-lane-registry-authority-check # lane=ediscovery-chain-of-custody --microservice mail
```

## Test Plan

- Personal-pillar engage attempt → `PersonalPillarForbidden`.
- Same-subject A and B → `SameApprover`.
- Signature outside window → `ApproverSignatureWindowExceeded`.
- Engage → retention sweep skips held messages.
- Hold engage ≤ 2s p99 per PRD AC-10.
- eDiscovery: sealed bundle digest re-derives from source blocks per AC-09.

## Halt Conditions

- Personal-pillar hold path bypassed → fail (regulatory critical).
- Four-eyes check bypassed → fail.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-011-legal-hold-engine.md` matched `p99`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Next IP

[`IP-012-ediscovery-export.md`](IP-012-ediscovery-export.md)

## References

- Bominal ADR-0215 (retention/legal-hold dual-context)
- HIPAA §164.502(b) (Minimum Necessary)
- GDPR Art. 18 (right to restriction)
- ISO 27001:2022 A.5.34 (privacy + PII protection)
- FedRAMP Moderate baseline AU-9 (audit information protection)
- eIDAS Regulation (EU) No 910/2014 (qualified signature for some legal contexts)
- `microservices/mail/policy/dual-context-isolation.md` Invariant DCI-04
- M3AAWG Litigation Hold guidance
