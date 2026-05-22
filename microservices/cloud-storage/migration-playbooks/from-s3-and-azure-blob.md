# Migration playbook — AWS S3 + Azure Blob Storage → Oyatie `cloud-storage`

Audience: a storage team operating production data on AWS S3 + Azure Blob Storage. Goal: migrate to `cloud-storage` without
service disruption and with full versioning + WORM provenance preservation.

## Phase 0 — Inventory (Day 0…7)

### From AWS S3

1. Catalogue buckets + their configurations:
   ```bash
   aws s3api list-buckets > s3-buckets.json
   jq -r '.Buckets[].Name' s3-buckets.json | while read b; do
     aws s3api get-bucket-versioning --bucket "$b" > "versioning-$b.json"
     aws s3api get-bucket-lifecycle-configuration --bucket "$b" > "lifecycle-$b.json" 2>/dev/null
     aws s3api get-bucket-replication --bucket "$b" > "replication-$b.json" 2>/dev/null
     aws s3api get-bucket-encryption --bucket "$b" > "encryption-$b.json" 2>/dev/null
     aws s3api get-object-lock-configuration --bucket "$b" > "object-lock-$b.json" 2>/dev/null
   done
   ```
2. Generate inventory reports if not enabled (allow 24-48 h for first delivery):
   ```bash
   aws s3api put-bucket-inventory-configuration --bucket "$b" --id full-inventory \
     --inventory-configuration file://inventory-cfg.json
   ```
3. Measure data volume + request rate from S3 Storage Lens / CloudWatch.

### From Azure Blob

1. List storage accounts + containers:
   ```bash
   az storage account list > azure-accounts.json
   az storage container list --account-name "$ACCT" > "containers-$ACCT.json"
   ```
2. Capture lifecycle policies + immutable-storage policies + replication state.

## Phase 1 — Tenant + bucket provisioning (Day 7…14)

```bash
./bin/oya storage tenant register --tenant oyatie.b2b.midmarket.acme-corp --tenant-class paid --billing-components per_usage
```

For each source bucket, mint a corresponding `cloud-storage` bucket:
```bash
./bin/oya storage bucket create \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --bucket acme-prod-customer-pii \
  --region us-east-2 \
  --storage-class Hot \
  --enable-versioning \
  --enable-encryption --kms-cmk acme-default \
  --enable-inventory daily-parquet
```

Carry over object-lock semantics by tenant_class and storage-class mapping:
- S3 Object Lock Compliance Mode → `cloud-storage` `--enable-object-lock-default compliance:<duration>`.
- S3 Object Lock Governance Mode → `--enable-object-lock-default governance:<duration>`.
- Azure Immutable Storage Time-Based → compliance mode at corresponding retention.

## Phase 2 — Lifecycle + replication translation (Day 14…21)

```bash
./bin/oya storage migrate s3-lifecycle-to-oya \
  --input lifecycle-acme-prod-customer-pii.json \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --bucket acme-prod-customer-pii \
  --output lifecycle-acme-customer-pii.yaml

./bin/oya storage lifecycle policy create --from-file lifecycle-acme-customer-pii.yaml
```

For replication rules:
```bash
./bin/oya storage migrate s3-replication-to-oya \
  --input replication-acme-prod-customer-pii.json \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --bucket acme-prod-customer-pii \
  --map-targets s3://acme-customer-pii-eu=oyatie://acme-prod-customer-pii-eu
```

Translation lossiness notes:
- S3 storage class names map: STANDARD → Hot, STANDARD_IA / ONEZONE_IA → Warm, GLACIER_IR / GLACIER → Cold, DEEP_ARCHIVE → Archive.
- AWS RTC (Replication Time Control) target SLA → our paid tenant_class replication target matches.
- AWS Cross-Account replication translates only within the same Oyatie tenant boundary (no cross-tenant by design).

## Phase 3 — Historical data backfill (Day 21…56; varies by volume)

For each bucket, run the migrator:
```bash
./bin/oya storage migrate copy \
  --source s3://acme-prod-customer-pii \
  --target oyatie://acme-prod-customer-pii \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --copy-versions all \
  --preserve-object-lock true \
  --preserve-tags true \
  --concurrency 32 \
  --resume-cursor /var/lib/oya/migrate/acme-customer-pii.cursor
```

The migrator:
1. Reads from S3 via boto3 (AWS-Sig-v4 to S3).
2. Writes to `cloud-storage` via S3-compat API.
3. Preserves version-ids (`X-Amz-Copy-Version-Id` header).
4. Preserves object-lock + retention metadata.
5. Resumable cursor — restart picks up where it left off.

Expected throughput: ~80 GB/s per cell at 32 concurrency (limited by S3 source throughput). A 100 TB bucket: ~20-30 h.

For Azure Blob, use the `azure-blob-to-oya` migrator (similar; uses Azure Storage SDK on the source side).

## Phase 4 — Dual-write phase (Day 56…84)

For each application that writes to S3/Azure today, configure dual-write:
```rust
use oya_cloud_storage_sdk::DualWrite;

let storage = DualWrite::builder()
    .primary(oya_cloud_storage_sdk::S3Client::from_aws(...))
    .secondary(oya_cloud_storage_sdk::OyaClient::connect(cfg).await?)
    .strategy(DualWriteStrategy::WriteBoth_ReadPrimary)
    .build()?;
```

Reads continue from S3; writes go to both. Oyatie is shadow-only for 14 d.

Divergence check:
```bash
./bin/oya storage migrate dual-write-divergence \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --bucket-pair s3://acme-prod-customer-pii=oyatie://acme-prod-customer-pii \
  --since "24h ago"
```

Investigate divergence > 0.01 %. Common sources: in-flight multipart uploads, large-object handling differences, tag mismatch.

## Phase 5 — Cut-over (Day 84…112)

1. Switch SDK from `WriteBoth_ReadPrimary` to `WriteBoth_ReadSecondary`.
2. Verify reads coming from `cloud-storage`:
   ```bash
   ./bin/oya storage request-log query --tenant oyatie.b2b.midmarket.acme-corp --since "1h ago" --group-by source
   ```
3. After 7 d clean: flip to `WriteSecondary_ReadSecondary` (S3 becomes read-only safety net).
4. Re-point CDN origins from S3 to `cloud-storage` endpoints.

## Phase 6 — Decommission AWS S3 + Azure Blob (Day 112+)

After 30 d clean run:
1. Set S3 bucket policies to `Deny *` (keeps data; refuses access).
2. After 60 d clean: schedule S3 bucket deletion (`aws s3 rm s3://... --recursive --include "*"; aws s3api delete-bucket ...`).
3. Azure Blob: set immutable-storage policies, then delete containers.
4. Cancel any S3 Replication Time Control SLA contract.

## Rollback strategy

Within Phase 4 dual-write:
- Flip SDK back to `WriteBoth_ReadPrimary` with S3 primary.
- Cost: rollback latency ~30 s per service deploy.

After Phase 5 cut-over:
- Re-enable S3 bucket policies (allow access).
- Flip SDK to `WriteBoth_ReadPrimary` with S3 primary.
- Plan: manual reconciliation of writes that only went to Oyatie (4-8 h).

After S3 bucket deletion: rollback requires S3 lifecycle deletion-archive (if enabled). Otherwise unrecoverable.

## What you gain

- 46 % TCO reduction vs S3 Standard + Glacier + CRR at mid-market scale.
- 2-5× lower GET/PUT latency.
- 6 storage classes (Hot, Warm, Cold, Archive, Tape, Sovereign-Air-Gapped).
- AAD-bound encryption at every PUT (vs S3's optional SSE-KMS without AAD).
- Lifecycle transition latency ≤ 2 min in paid tenant_class regulated profiles (vs S3's 12 h).
- Cross-region sync replication in paid tenant_class regulated profiles (vs S3 CRR 15-min target).
- Bundled inventory + WORM + replication (vs S3 add-ons).
- BLAKE3 audit chain.
- HTTP/3 default.

## What you give up

- AWS S3 ecosystem depth (Lambda, Glue, EMR, Athena pre-wired).
- AWS S3 Express One Zone for ultra-low-latency hot path (we match this only in paid tenant_class regulated profiles).
- Cloudflare R2's $0 egress for edge-near-storage workloads.
- Backblaze B2's cheapest-storage tier for cold archive (we beat S3 Glacier but not B2).
- Public self-service signup.
