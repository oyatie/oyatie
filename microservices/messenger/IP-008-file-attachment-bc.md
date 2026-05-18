---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-008-file-attachment-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger + ops-security
acceptance_lanes: [cargo-nextest, malware-scan-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: file-attachment BC (S3 multipart + OPSWAT + preview)

## Intent

Implement multipart upload to S3-compatible storage (OCI Object Storage
primary); OPSWAT MetaDefender scan (ClamAV fallback); preview generation
for image / pdf / docx; retention TTL based on channel retention policy.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-file-attachment-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-opswat,worker,sdk,app}/...` | create |
| `tests/attachment_lifecycle_e2e.rs` | create |

## Code Shape

```rust
// adapter-opswat/src/scanner.rs
pub struct OpswatScanner { endpoint: Url, api_key: SecretString }

#[async_trait]
impl MalwareScanner for OpswatScanner {
    async fn scan(&self, blob: BlobRef) -> Result<ScanVerdict, ScanError> {
        let resp = self.client.post(...).send().await?;
        let verdict: OpswatScanResponse = resp.json().await?;
        Ok(map_to_verdict(verdict))
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-file-attachment-adapter-s3
cargo nextest run -p oya-messenger-file-attachment-adapter-opswat
cargo nextest run --test attachment_lifecycle_e2e
```

## Test Plan

- EICAR test file → scan → infected → quarantine bucket.
- Clean PDF → scan → preview generated → finalize → exposed via signed URL.
- Multipart 5GB upload completes; resumability honored on partial fail.

## Next IP

[`IP-009-thread-tree-and-mention-router.md`](IP-009-thread-tree-and-mention-router.md)
