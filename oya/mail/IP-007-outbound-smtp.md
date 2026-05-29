---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-007-outbound-smtp
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + ops-deliverability
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-governance-dkim-key-rotation-conformance, oya-governance-mta-sts-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: oya-mail-outbound-smtp-{kernel,domain,usecase,api,adapter,adapter-smtp,worker,app}

## Intent

Implement outbound SMTP submission on :587 (RFC 6409); DKIM-sign every outbound (RFC 6376 + RFC 8463 Ed25519); per-tenant deliverability queue; bounce processor; per-tenant SMTP IP reputation tracking + auto-throttle per `capabilities/T2-auto.yaml` T2-mail-reputation-auto-throttle. DLP scan integration per `policy/data-residency.md` per-pack overlays. MTA-STS + TLS-RPT publication for tenant domains.

## ChangeSet boundary

8 Rust crates. Helm deployment for the submission frontend pod + queue spool PV.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-outbound-smtp-kernel/` | create | port traits (SmtpOutboundSubmitter, DkimSigner, ReputationStore, DlpScanner) |
| `microservices/mail/src/crates/oya-mail-outbound-smtp-domain/` | create | submission state-machine + envelope canonicalisation + DKIM sign + bounce classification |
| `microservices/mail/src/crates/oya-mail-outbound-smtp-usecase/` | create | orchestrator (validate → DLP scan → DKIM sign → queue → deliver → audit) |
| `microservices/mail/src/crates/oya-mail-outbound-smtp-api/` | create | typed I/O |
| `microservices/mail/src/crates/oya-mail-outbound-smtp-adapter/` | create | DLP scanner integration + reputation provider |
| `microservices/mail/src/crates/oya-mail-outbound-smtp-adapter-smtp/` | create | SMTP wire impl; per-tenant IP pool allocation; per-recipient throttle |
| `microservices/mail/src/crates/oya-mail-outbound-smtp-worker/` | create | submission listener + deliverability cron + bounce processor |
| `microservices/mail/src/crates/oya-mail-outbound-smtp-app/` | create | composition root |
| `microservices/mail/catalog/oya-mail-outbound-smtp-*.yaml` × 8 | create | catalog rows |
| `microservices/mail/tests/e2e/outbound-smtp.sh` | create | end-to-end DKIM-signed outbound drill |

## Crate Naming

```
NAME: oya-mail-outbound-smtp-{layer}
JUSTIFICATION:
- microservice = mail
- bc-tokens = outbound-smtp
- layer = per ADR-0105 13-value enum
- adapter-smtp = backend-qualified
- exemptions claimed: none
```

## Code Shape

```rust
// usecase/src/orchestrator.rs
pub async fn submit(env: OutboundEnvelope, principal: &Principal, ports: &Ports)
    -> Result<SubmissionReceipt, OutboundError>
{
    ports.cedar.permit(principal, "send_message", &env)?;
    ports.context_guard.assert(principal.context, env.context())?;
    let dlp_verdict = ports.dlp_scanner.scan(&env).await?;
    if dlp_verdict.matches() {
        let quarantine_id = ports.dlp_quarantine.hold(&env, dlp_verdict).await?;
        return Ok(SubmissionReceipt {
            message_id: env.message_id(), queued_at: now(),
            dkim_selector_used: String::new(),
            dlp_quarantined: true, quarantine_id: Some(quarantine_id),
        });
    }
    let signed = ports.dkim_signer.sign(env, principal.tenant_id()).await?;
    let queued = ports.smtp_queue.enqueue(signed, principal.tenant_id()).await?;
    ports.events.emit_message_sent(queued).await?;
    Ok(receipt(queued))
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-mail-outbound-smtp-domain
cargo nextest run -p oya-mail-outbound-smtp-usecase
cargo run -p oya-dev-cli -- gate validate dkim-key-rotation-conformance --microservice mail
cargo run -p oya-dev-cli -- gate validate mta-sts-conformance --microservice mail
bash microservices/mail/tests/e2e/outbound-smtp.sh
```

## Test Plan

- DKIM sign vs RFC 6376/8463 reference vectors.
- Per-tenant IP allocation: 2 tenants observed on different IPs per ADR-0133.
- Bounce classification: 100 DSN fixtures classified correctly (hard / soft / transient / mailbox-full / rejected-policy).
- DLP scan: synthetic PHI sample held; non-PHI sample passes.
- Reputation auto-throttle: synthetic complaint spike → throttle engages within 5 min.
- E2E: submission → DKIM signed → recipient MX 2xx → MessageDelivered emitted.

## Halt Conditions

- Unsigned outbound message (DKIM absent) leaves submission → fail.
- Open-relay: unauthenticated submission to external recipient → fail.
- Cross-tenant IP leak (tenant X's mail egresses on tenant Y's IP) → fail.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-007-outbound-smtp.md` matched `PHI`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Next IP

[`IP-008-imap-frontend.md`](IP-008-imap-frontend.md)

## References

- RFC 6409 (Submission), RFC 5321 (SMTP), RFC 6376 (DKIM), RFC 8463 (Ed25519 DKIM)
- RFC 7208 (SPF alignment), RFC 7489 (DMARC alignment)
- RFC 8461 (MTA-STS), RFC 8460 (TLS-RPT)
- RFC 3463 (Enhanced SMTP Status Codes), RFC 3464 (DSN format)
- M3AAWG Sender Best Common Practices v3
- ADR-0133 (per-tenant SMTP IP pool)
