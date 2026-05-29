---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-009-search-index
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, port-location, layer-correctness, oya-governance-search-index-context-partition, oya-governance-encrypted-token-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: oya-mail-search-index-{kernel,domain,usecase,api,adapter,adapter-search-index,worker,sdk,app}

## Intent

Implement encrypted-token search index for mailbox content. Per-tenant + per-context partition (per `policy/dual-context-isolation.md` Invariant DCI-05). Tantivy 0.22 LTS primary backend (`-adapter-search-index` resolves to Tantivy by default); optional Elasticsearch adapter for tenants demanding it (later IP). Encrypted-token scheme: client/server-side tokens never include plaintext; indexer derives deterministic tokens from per-tenant Cipher-Match HMAC.

## ChangeSet boundary

9 Rust crates (full layer set including `-sdk` for client-side encryption helper).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-search-index-kernel/` | create | `EncryptedTokenIndex` port + entities (SearchToken, IndexShard, EncryptedQuery, ResultPage) |
| `microservices/mail/src/crates/oya-mail-search-index-domain/` | create | token derivation; per-tenant HMAC scheme; shard math |
| `microservices/mail/src/crates/oya-mail-search-index-usecase/` | create | orchestrator (auth → context-guard → search → result-page) |
| `microservices/mail/src/crates/oya-mail-search-index-api/` | create | typed contracts |
| `microservices/mail/src/crates/oya-mail-search-index-adapter/` | create | shared adapter logic |
| `microservices/mail/src/crates/oya-mail-search-index-adapter-tantivy/` | create | Tantivy 0.22 LTS impl |
| `microservices/mail/src/crates/oya-mail-search-index-worker/` | create | indexer worker (Postgres CDC → token derivation → Tantivy ingest) |
| `microservices/mail/src/crates/oya-mail-search-index-sdk/` | create | client SDK for token derivation (no plaintext escapes) |
| `microservices/mail/src/crates/oya-mail-search-index-app/` | create | composition root |
| `microservices/mail/catalog/oya-mail-search-index-*.yaml` × 9 | create | catalog rows |

## Code Shape

```rust
// domain/src/token.rs
pub struct CipherMatchTokenizer { tenant_hmac_key: HmacKey }

impl CipherMatchTokenizer {
    pub fn tokenize_for_index(&self, plaintext_term: &str) -> SearchToken {
        // Deterministic HMAC; same term → same token (within tenant).
        let mac = hmac_sha256(&self.tenant_hmac_key, plaintext_term.as_bytes());
        SearchToken(base64url(mac))
    }
}

// usecase/src/search.rs
pub async fn search(req: SearchRequest, principal: &Principal, ports: &Ports)
    -> Result<SearchResultPage, SearchError>
{
    ports.cedar.permit(principal, "search", &req)?;
    let partition = match principal.context {
        ContextKind::Personal     => Partition::PerUser(principal.user_id),
        ContextKind::Professional => Partition::PerTenant(principal.tenant_id.unwrap()),
    };
    let index = ports.index_resolver.resolve(partition).await?;
    let results = index.query(&req.encrypted_tokens, req.date_range, req.limit, req.cursor).await?;
    Ok(SearchResultPage { results, next_cursor: None })
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-mail-search-index-domain
cargo run -p oya-dev-cli -- gate validate search-index-context-partition --microservice mail
cargo run -p oya-dev-cli -- gate validate encrypted-token-conformance --microservice mail
```

## Test Plan

- Correctness: 10k-message synthetic mailbox; search returns correct messages with token match count.
- Encryption integrity: plaintext NEVER appears in Tantivy segment files (greps fail).
- Per-tenant partition: tenant A's term tokenizes differently than tenant B's same plaintext.
- Performance: 100k-message mailbox search p99 ≤ 500ms (PRD performance budget).
- Pack overlay: pack-kr index resides in KR PV; cross-pack queries refused.
- Personal-pillar isolation: org-admin search query returns zero matches for user's Personal mailbox terms.

## Halt Conditions

- Plaintext leakage to index → refactor.
- Cross-tenant index collision → audit + refactor partition layout.


## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-009-search-index.md` matched `p99`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Next IP

[`IP-010-retention-policy.md`](IP-010-retention-policy.md)

## References

- Tantivy — `github.com/quickwit-oss/tantivy`
- Cipher-Match search — Curtmola et al. (CCS 2006); Cash et al. (CRYPTO 2013)
- ADR-0133 (per-tenant pattern)
- Apache Lucene reference — `lucene.apache.org`
- ProtonMail Tokenised Search — `proton.me/blog/encrypted-search`
- PRD Open Question 3 (Tantivy vs Elasticsearch trade-off)
