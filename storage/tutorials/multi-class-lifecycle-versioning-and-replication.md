# Tutorial — Multi-class lifecycle + versioning + cross-region replication + S3-API client

Goal: provision a tenant bucket with versioning + WORM object-lock, upload objects, walk lifecycle Hot→Warm→Cold→Archive
transitions, enable bi-directional cross-region replication, and exercise the bucket via the standard AWS S3 SDK. Loopback
`cloud-storage` cell.

Pre-reqs:
- Loopback storage cell: `make dev-cell.up CELL=storage-loopback-1 PROFILE=cloud-storage-dev`
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid BILLING_COMPONENTS=per_usage`
- AWS CLI v2.x + boto3 on PATH.

## Step 1 — bucket with versioning + WORM

```bash
./bin/oya storage bucket create \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --region us-east-2 \
  --storage-class Hot \
  --enable-versioning \
  --enable-object-lock-default compliance:7y \
  --enable-encryption \
  --kms-cmk acme-financial \
  --enable-inventory daily-parquet
```

Expected:
```
bucket_id            : bkt-2026-05-20-...
versioning           : enabled
object_lock_default  : compliance; 7y retention (SEC Rule 17a-4 + FINRA 4511 compatible)
encryption           : AES-256-GCM under cmk-acme-financial (cloud-kms BYO-CMK)
inventory_report     : daily-parquet to s3://acme-financial-records-inventory/
audit_chain_event    : ce-2026-05-20T10:01:33Z-…
```

## Step 2 — upload a financial record

```bash
cat > /tmp/trade-2026-05-20.json <<'EOF'
{
  "trade_id": "TRD-2026-05-20-001",
  "timestamp": "2026-05-20T10:14:23.123Z",
  "instrument": "AAPL",
  "side": "buy",
  "quantity": 100,
  "price_usd": 248.50,
  "executing_trader": "alice@acme-trades.com",
  "regulatory_reporting": "FINRA-OATS",
  "checksum_sha256": "..."
}
EOF

./bin/oya storage object put \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --key "trades/2026/05/20/TRD-2026-05-20-001.json" \
  --source /tmp/trade-2026-05-20.json \
  --content-type application/json \
  --metadata "regulatory_class=FINRA-OATS,retention_class=7y"
```

Verify the WORM lock applies:
```bash
./bin/oya storage object head \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --key "trades/2026/05/20/TRD-2026-05-20-001.json"
```

Expected (truncated):
```
object_key       : trades/2026/05/20/TRD-2026-05-20-001.json
storage_class    : Hot
content_length   : 412
last_modified    : 2026-05-20T10:14:24.118Z
version_id       : 01HKM5N8XWY4Z7P3R9SQGCDF2T
encryption       : AES-256-GCM under cmk-acme-financial; kek_version: 1
object_lock_mode : compliance
retain_until     : 2033-05-20T10:14:24.118Z  (7y)
```

Attempt to delete (must fail):
```bash
./bin/oya storage object delete \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --key "trades/2026/05/20/TRD-2026-05-20-001.json" \
  --version-id 01HKM5N8XWY4Z7P3R9SQGCDF2T
```

Expected: `ERROR: StorageError::ObjectLockProtected { mode: compliance, retain_until: 2033-05-20T10:14:24.118Z }`

## Step 3 — versioning behavior

Add a corrected version (e.g. price adjustment):
```bash
jq '.price_usd = 248.55 | .correction_note = "T+0 fix"' /tmp/trade-2026-05-20.json > /tmp/trade-2026-05-20.v2.json

./bin/oya storage object put \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --key "trades/2026/05/20/TRD-2026-05-20-001.json" \
  --source /tmp/trade-2026-05-20.v2.json
```

Both versions are now retained (V1 is locked + V2 is the current):
```bash
./bin/oya storage object versions list \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --key "trades/2026/05/20/TRD-2026-05-20-001.json"
```

## Step 4 — lifecycle policy

```bash
./bin/oya storage lifecycle policy create \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --rule-name "trade-records-lifecycle" \
  --filter-prefix "trades/" \
  --transition-warm-after-days 30 \
  --transition-cold-after-days 180 \
  --transition-archive-after-days 730 \
  --non-current-version-expiration-after-days 2555 \
  --respect-object-lock true
```

The `respect-object-lock true` ensures lifecycle transitions never violate WORM. Even Archive class respects retention.

Fast-forward 31 d (dev profile) + observe:
```bash
./bin/oya storage lifecycle fast-forward \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --days 31

./bin/oya storage object head \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records \
  --key "trades/2026/05/20/TRD-2026-05-20-001.json"
```

Expected: `storage_class: Warm`, `last_transition: 2026-06-20T10:14:24Z`.

## Step 5 — cross-region replication (bi-directional)

```bash
./bin/oya storage replication enable \
  --tenant oyatie.b2b.smb.acme-software \
  --source-bucket acme-financial-records \
  --source-region us-east-2 \
  --target-bucket acme-financial-records-replica \
  --target-region eu-west-1 \
  --replicate-versions all \
  --replicate-deletes false \
  --replicate-object-lock true \
  --target-storage-class-override Warm

./bin/oya storage replication enable \
  --tenant oyatie.b2b.smb.acme-software \
  --source-bucket acme-financial-records-replica \
  --source-region eu-west-1 \
  --target-bucket acme-financial-records \
  --target-region us-east-2 \
  --replicate-versions all \
  --conflict-resolution last-modified-wins
```

Wait ~5 s for the trade record to replicate:
```bash
./bin/oya storage object head \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records-replica \
  --key "trades/2026/05/20/TRD-2026-05-20-001.json" \
  --region eu-west-1
```

Expected: same version-id + WORM lock; `storage_class: Warm` (overridden per replication rule).

## Step 6 — exercise via standard AWS S3 SDK

Issue short-lived credentials:
```bash
CREDS_JSON=$(./bin/oya storage s3-credential issue \
  --tenant oyatie.b2b.smb.acme-software \
  --ttl 1h \
  --read-only \
  --prefix "trades/" \
  --output json)

export AWS_ACCESS_KEY_ID=$(echo "$CREDS_JSON" | jq -r .access_key)
export AWS_SECRET_ACCESS_KEY=$(echo "$CREDS_JSON" | jq -r .secret_key)
export AWS_SESSION_TOKEN=$(echo "$CREDS_JSON" | jq -r .session_token)
export AWS_ENDPOINT_URL=https://s3.cloud-storage.loopback.oyatie.local
```

List + get via aws-cli:
```bash
aws s3 ls s3://acme-financial-records/trades/2026/05/20/
aws s3 cp s3://acme-financial-records/trades/2026/05/20/TRD-2026-05-20-001.json /tmp/trade-via-aws.json
```

Verify the data is identical:
```bash
diff /tmp/trade-via-aws.json /tmp/trade-2026-05-20.v2.json && echo "S3-API round-trip OK"
```

## Step 7 — inventory report

```bash
./bin/oya storage inventory list \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-financial-records-inventory \
  --since "1d ago"
```

The daily inventory report is a Parquet file with one row per object containing `(key, version_id, size, storage_class,
last_modified, encryption_key_id, object_lock_mode, retain_until)`. Used by `cloud-billing` to attribute storage cost +
by audit teams to prove WORM compliance.

## Step 8 — audit-chain trail

```bash
./bin/oya audit-chain query \
  --tenant oyatie.b2b.smb.acme-software \
  --kind 'cloud_storage.*' \
  --since "2h ago"
```

You should see:
- `cloud_storage.bucket.created`
- `cloud_storage.object.put` (×2)
- `cloud_storage.object_lock.applied`
- `cloud_storage.object.delete.refused` (the WORM-protected attempt)
- `cloud_storage.lifecycle.policy_applied`
- `cloud_storage.lifecycle.transition` (Hot → Warm)
- `cloud_storage.replication.enabled` (×2)
- `cloud_storage.replication.copy_complete` (cross-region)
- `cloud_storage.inventory.report_generated`

## What you just demonstrated

- Versioned bucket with SEC-Rule-17a-4-compliant WORM.
- Per-tenant CMK envelope encryption with AAD.
- Lifecycle transition Hot → Warm under respect-object-lock semantics.
- Bi-directional cross-region replication with storage-class override.
- Standard AWS S3 SDK compatibility — boto3, aws-cli, aws-sdk-rust all work.
- Daily inventory report in Parquet format.
- BLAKE3 audit-chain anchoring of every storage action.
