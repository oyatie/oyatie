---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-006-inbound-smtp
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + ops-deliverability
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, port-location, layer-correctness, statelessness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-mail-inbound-smtp-{kernel,domain,usecase,api,adapter,adapter-smtp,worker,app}

## Intent

Implement inbound SMTP receiver on :25 (plain + STARTTLS per RFC 8314) + :465 (implicit TLS) per RFC 5321 + RFC 8314. DKIM verify (RFC 6376 + RFC 8463 Ed25519), SPF check (RFC 7208), DMARC alignment (RFC 7489), ARC chain validation (RFC 8617). Inbound abuse classification via Rspamd integration. Cross-tenant routing via recipient resolution. ARC-seal append for forwarded mail.

## ChangeSet boundary

8 Rust crates spanning the full inbound-smtp BC. SMTP wire-level handled via `mail-parser` + `mail-send` crates; backend swap to a Stalwart-embedded mode possible via `-adapter-stalwart-inbound` (later IP).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-inbound-smtp-kernel/` | create | port traits (SmtpInboundReceiver, DkimVerifier, AbuseClassifier) |
| `microservices/mail/src/crates/oya-mail-inbound-smtp-domain/` | create | RFC 5321 state-machine + MIME parser + DKIM verify + ARC chain |
| `microservices/mail/src/crates/oya-mail-inbound-smtp-usecase/` | create | orchestrator (accept → verify → classify → persist → emit MessageReceived) |
| `microservices/mail/src/crates/oya-mail-inbound-smtp-api/` | create | typed I/O |
| `microservices/mail/src/crates/oya-mail-inbound-smtp-adapter/` | create | Rspamd milter + recipient resolution |
| `microservices/mail/src/crates/oya-mail-inbound-smtp-adapter-smtp/` | create | SMTP wire impl (tokio-based listener) |
| `microservices/mail/src/crates/oya-mail-inbound-smtp-worker/` | create | long-lived listener binary |
| `microservices/mail/src/crates/oya-mail-inbound-smtp-app/` | create | composition root |
| `microservices/mail/catalog/oya-mail-inbound-smtp-*.yaml` × 8 | create | catalog rows |
| `microservices/mail/tests/e2e/inbound-smtp.sh` | create | end-to-end DKIM-verified inbound drill |

## Crate Naming

```
NAME: oya-mail-inbound-smtp-{layer}
JUSTIFICATION:
- microservice = mail
- bc-tokens = inbound-smtp
- layer = per ADR-0105 13-value enum
- adapter-smtp = backend-qualified (per ADR-0105 Amendment 3)
- exemptions claimed: none
```

## Code Shape

```rust
// domain/src/dkim.rs (excerpt)
pub fn verify_dkim(message: &Mime, dns: &impl DnsResolver) -> DkimResult {
    let signatures = parse_dkim_signatures(message);
    for sig in signatures {
        match sig.algorithm {
            Algo::Ed25519Sha256 | Algo::RsaSha256 => {
                let dns_key = dns.txt(&format!("{}._domainkey.{}", sig.selector, sig.domain));
                if verify_signature(&sig, &dns_key, message.canonical_form()) {
                    return DkimResult::Pass;
                }
            }
        }
    }
    DkimResult::Fail
}

// usecase/src/orchestrator.rs
pub async fn receive_session(session: IncomingSession, ports: &Ports) -> Result<(), InboundError> {
    let dkim = ports.dkim_verifier.verify(&session.message).await?;
    let spf  = ports.spf_checker.check(&session.envelope.from, session.peer_ip).await?;
    let dmarc = ports.dmarc_evaluator.evaluate(&session, dkim, spf).await?;
    let arc   = ports.arc_evaluator.evaluate(&session).await?;
    let abuse = ports.abuse_classifier.classify(&session).await?;
    if abuse.is_phishing_or_malware() {
        return reject_with(session, "550 5.7.1 phishing/malware");
    }
    let recipient = ports.recipient_resolver.resolve(&session.envelope.to).await?;
    ports.context_guard.assert(/* inferred for tenant + context */)?;
    let persisted = ports.mailbox_store.persist_inbound(&session, &recipient).await?;
    ports.events.emit_message_received(persisted, dkim, spf, dmarc, arc).await?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-mail-inbound-smtp-domain
cargo nextest run -p oya-mail-inbound-smtp-usecase
buck2 build //:quality-lane-registry-authority-check # lane=statelessness --microservice mail
bash microservices/mail/tests/e2e/inbound-smtp.sh   # DKIM-verified inbound drill
```

## Test Plan

- 1000+ unit tests for SMTP state-machine corner cases (RFC 5321 §3.3).
- DKIM verify property tests vs RFC 6376 test vectors + RFC 8463 Ed25519 test vectors.
- E2E: external sender → inbound :25 + STARTTLS → DKIM verify → DMARC pass → persisted → MessageReceived emitted.
- ARC test against M3AAWG reference vectors.
- Abuse classifier integration: Rspamd container; phishing sample rejected with `550 5.7.1`.
- Performance: SMTP DATA p99 ≤ 1s on M03 reference benchmark.

## Halt Conditions

- Tampered DKIM signature accepted → test fails; refactor.
- Open-relay path discoverable (unauthenticated relay to external recipient) → block; refactor.
- ARC chain validation incorrect against M3AAWG vectors.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-006-inbound-smtp.md` matched `p99`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Next IP

[`IP-007-outbound-smtp.md`](IP-007-outbound-smtp.md)

## References

- RFC 5321 (SMTP), RFC 5322 (Internet Message Format)
- RFC 6376 (DKIM), RFC 8463 (Ed25519 for DKIM)
- RFC 7208 (SPF), RFC 7489 (DMARC), RFC 8617 (ARC)
- RFC 8314 (TLS for mail submission/access), RFC 3207 (STARTTLS)
- M3AAWG Sender Best Common Practices v3
- Rspamd — `rspamd.com`
- `mail-parser` (Rust) — `github.com/stalwartlabs/mail-parser`
