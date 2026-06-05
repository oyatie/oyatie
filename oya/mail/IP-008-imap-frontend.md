---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-008-imap-frontend
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, port-location, layer-correctness, statelessness, oya-governance-jmap-conformance, oya-governance-imap-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: oya-mail-imap-frontend-{kernel,domain,usecase,api,adapter,rest,worker,app}

## Intent

Implement IMAP4rev2 (RFC 9051), JMAP-Core (RFC 8620), JMAP-Mail (RFC 8621), and REST (JMAP-shaped JSON) mailbox-read frontends. Per-folder pagination; flag synchronization; encrypted-token search hand-off; Apple Mail / Thunderbird / Outlook IMAP compatibility verified via Letterdrop test harness. ManageSieve (RFC 5804) for filter authoring.

## ChangeSet boundary

8 Rust crates spanning frontend BC.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-imap-frontend-kernel/` | create | port traits (ImapSessionHandler, JmapHandler) + entities |
| `microservices/mail/src/crates/oya-mail-imap-frontend-domain/` | create | IMAP state-machine; FETCH/SELECT/LIST/UID/SEARCH semantics; JMAP method dispatch |
| `microservices/mail/src/crates/oya-mail-imap-frontend-usecase/` | create | orchestrator: principal → context guard → mailbox-store read |
| `microservices/mail/src/crates/oya-mail-imap-frontend-api/` | create | typed contracts |
| `microservices/mail/src/crates/oya-mail-imap-frontend-adapter/` | create | wire codecs (IMAP + JMAP) |
| `microservices/mail/src/crates/oya-mail-imap-frontend-rest/` | create | REST shape (per OpenAPI 3.2.0 `mail.yaml`) |
| `microservices/mail/src/crates/oya-mail-imap-frontend-worker/` | create | IMAP listener (:143 + STARTTLS / :993 implicit-TLS) + JMAP HTTP server (:443) |
| `microservices/mail/src/crates/oya-mail-imap-frontend-app/` | create | composition root |
| `microservices/mail/catalog/oya-mail-imap-frontend-*.yaml` × 8 | create | catalog rows |
| `microservices/mail/tests/e2e/imap-fetch.sh` | create | latest-50-headers drill ≤ 300ms p99 |

## Code Shape

```rust
// rest/src/get_message.rs
pub async fn get_message(state: AppState, Path((mb, mid)): Path<(MailboxId, String)>,
                          headers: HeaderMap) -> Result<Json<MailMessage>, ApiError>
{
    let principal = principal_from_oidc(&headers, &state).await?;
    let principal_ctx = parse_mail_context(&headers)?;
    let mb_obj = state.usecase.read_mailbox(&principal, mb).await?;
    state.context_guard.assert(principal_ctx, mb_obj.context_kind)?;
    let msg = state.usecase.read_message(&principal, mb, &mid).await?;
    Ok(Json(msg))
}
```

```rust
// adapter/src/jmap_dispatch.rs (RFC 8620 method dispatch)
match method {
    "Mailbox/get" => handle_mailbox_get(args).await,
    "Email/query" => handle_email_query(args).await,
    "Email/get"   => handle_email_get(args).await,
    "Thread/get"  => handle_thread_get(args).await,
    "EmailSubmission/set" => handle_email_submission(args).await,
    // ... full JMAP-Mail method set per RFC 8621
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-mail-imap-frontend-domain
cargo nextest run -p oya-mail-imap-frontend-usecase
buck2 build //:quality-lane-registry-authority-check # lane=imap-conformance --microservice mail
buck2 build //:quality-lane-registry-authority-check # lane=jmap-conformance --microservice mail
bash microservices/mail/tests/e2e/imap-fetch.sh
```

## Test Plan

- IMAP4rev2 conformance: imaptest harness (Dovecot test set) passes.
- JMAP conformance: `jmap-test` reference suite passes.
- Performance: latest-50-headers fetch p99 ≤ 300ms.
- Apple Mail / Thunderbird compatibility: scripted IMAP login + SELECT + FETCH against mock mailbox.
- Concurrent sessions: 50k IMAP sessions sustained per cell baseline per PRD.
- Cross-context refusal: IMAP login with mailbox in wrong context returns `NO`.

## Halt Conditions

- IMAP4rev2 conformance failure → fix before merge.
- Sub-300ms p99 not met → optimize Postgres query plan + cache layer.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-008-imap-frontend.md` matched `p99`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Next IP

[`IP-009-search-index.md`](IP-009-search-index.md)

## References

- RFC 9051 (IMAP4rev2), RFC 3501 (IMAP4rev1; deprecated but still in client base)
- RFC 8620 (JMAP-Core), RFC 8621 (JMAP-Mail)
- RFC 5804 (ManageSieve), RFC 5228 (Sieve)
- RFC 8314 (TLS), RFC 3207 (STARTTLS)
- imaptest — `imapwiki.org/ImapTest`
- jmap-test — `github.com/fastmail/jmap-test`
