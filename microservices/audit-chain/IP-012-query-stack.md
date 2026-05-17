---
doc_class: ImplementationPlan
impl_plan_id: IP-012-query-stack
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness, port-location]
---

# IP-012: query BC (8 crates)

## Intent

Full query stack: kernel + domain + usecase + api + adapter + adapter-postgres + rest + sdk. Tenant-scoped + Cedar-gated forensic queries. Auditor export bundle constructor.

## Crates introduced (8)

- `oya-audit-chain-query-kernel`
- `oya-audit-chain-query-domain`
- `oya-audit-chain-query-usecase`
- `oya-audit-chain-query-api`
- `oya-audit-chain-query-adapter`
- `oya-audit-chain-query-adapter-postgres`
- `oya-audit-chain-query-rest`
- `oya-audit-chain-query-sdk`

## Concrete File Targets

Per crate per ADR-0105: `Cargo.toml`, modules, tests, catalog.

## Code Shape (usecase + export builder)

```rust
// query-usecase/src/lib.rs
pub async fn build_export_bundle(
    pg: &impl AuditQueryRepository,
    seal_records: &impl SealRecordReader,
    signer: &impl SignerPort,
    request: ExportRequest,
    auditor_engagement: AuditorEngagement,
) -> Result<ExportBundle, ExportError> {
    cedar_policy_check(&auditor_engagement, &request)?;

    let events = pg.scan_events_for_export(&request).await?;
    let proofs = build_proofs_for(&events, seal_records).await?;
    let bundle = ExportBundle {
        engagement_id: auditor_engagement.engagement_id,
        events,
        proofs,
        signed_roots: collect_roots_for(&events).await?,
        public_key_records: collect_keys_for(&events).await?,  // KeyResolver epochs
        bundle_metadata: BundleMetadata { ... },
    };

    let signature = signer.sign(&bundle.canonical_serialize()).await?;
    Ok(SignedBundle { bundle, signature })
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-audit-chain-query-usecase
cargo nextest run -p oya-audit-chain-query-adapter-postgres --features integration
cargo nextest run -p oya-audit-chain-query-rest --features integration
cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice audit-chain
```

## References

- `microservices/audit-chain/contracts/openapi/audit-chain.yaml` /query + /export paths.
- `microservices/audit-chain/policy/auditor-scope.cedar`.
- `microservices/audit-chain/policy/tenant-scope.cedar`.
- `microservices/audit-chain/runbooks/audit-export.md`.
