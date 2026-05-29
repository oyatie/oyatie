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

## Wave 15 substance conversion — file attachment BC

### §A Problem

Attachments are not ordinary blobs in messenger: they carry malware risk, retention obligations, PHI redaction,
preview generation, and dual-context disclosure boundaries.
This IP closes the attachment gap behind `policy/attachment-malware-quarantine.md` and the attachment scan SLO.

### §B Approach

Build a bounded context around multipart upload, object storage, OPSWAT/ClamAV scanning, preview generation, and
retention TTL.
The message stream stores attachment references only after scan and policy state are known.

### §C Deliverables

- `src/crates/oya-messenger-file-attachment-{kernel,domain,usecase,adapter-s3,adapter-opswat,worker}/...`
- scan freshness tests against `slos/attachment-scan-freshness.openslo.yaml`
- quarantine and restore runbook wiring

### §D Implementation

1. Allocate multipart upload ids scoped by tenant, context, channel/direct-conversation, and retention policy.
2. Stream bytes to S3-compatible storage without loading entire files into gateway memory.
3. Submit scan jobs to OPSWAT MetaDefender and ClamAV fallback.
4. Quarantine suspicious objects and refuse message attachment commit.
5. Generate previews only after clean verdict and PHI redaction rules.
6. Emit audit events for upload, scan verdict, quarantine, restore, and purge.

### §E Acceptance

Smoke tests must prove clean upload, malware quarantine, PHI preview redaction, per-tenant object isolation, and scan
freshness within the OpenSLO target.

### §F Evidence

Local anchors: `policy/attachment-malware-quarantine.md`, `policy/redaction-phi.md`,
`runbooks/attachment-restore.md`, `slos/attachment-scan-freshness.openslo.yaml`.

### §G Counterparts

Slack and Teams anchor enterprise file sharing, Discord anchors consumer uploads, and Mattermost anchors
self-hosted storage; oyatie closes parity with policy-scoped scan and audit-chain evidence.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-008-file-attachment-bc.md` matched `PHI, SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.
