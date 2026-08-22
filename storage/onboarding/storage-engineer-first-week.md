# Storage Engineer — First Week on `cloud-storage`

Audience: a storage engineer with AWS S3 + GCS + Azure Blob + MinIO experience joining the `cloud-storage-*` lane.
Goal: by Friday EOD you can create a tenant bucket, upload objects with versioning, configure a lifecycle policy, walk a
cross-region replication, and use the S3-compat API.

## Day 1 — read before touching

- `docs/decisions/ADR-0700-ci-admission-live-apex.md` — per-cell storage planes.
- `docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md` — §D-10 binds AAD-mandatory envelope encryption.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md` — tenant bucket scoping.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md`, `docs/decisions/ADR-0702-identity-authz-live-apex.md`, and `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` — tenant_class model + storage classes.
- AWS S3 API reference (object lock, lifecycle, replication) — these guide our compatibility layer.
- SEC Rule 17a-4 + FINRA 4511 — at minimum understand WORM retention.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-storage-week1 .worktrees/$USER-storage-week1
cd .worktrees/$USER-storage-week1
```

## Day 2 — bring up a loopback storage cell

```bash
make dev-cell.up CELL=storage-loopback-1 PROFILE=cloud-storage-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid BILLING_COMPONENTS=per_usage
```

Create your first bucket:
```bash
./bin/oya storage bucket create \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --region us-east-2 \
  --storage-class Hot \
  --enable-versioning \
  --enable-encryption \
  --kms-cmk acme-default
```

Expected:
```
bucket_id        : bkt-2026-05-20-...
bucket           : acme-customer-pii
region           : us-east-2
storage_class    : Hot
versioning       : enabled
encryption       : AES-256-GCM under cmk-acme-default (cloud-kms)
endpoint_s3      : https://s3.cloud-storage.loopback.oyatie.local/acme-customer-pii
endpoint_azure   : https://acme-customer-pii.blob.cloud-storage.loopback.oyatie.local
audit_chain_event: ce-2026-05-20T10:01:33Z-…
```

Upload an object:
```bash
echo '{"customer_id":"cust-42","name":"Jane Doe"}' > /tmp/cust-42.json

./bin/oya storage object put \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --key "customers/cust-42.json" \
  --source /tmp/cust-42.json \
  --content-type application/json
```

List + get:
```bash
./bin/oya storage object list --tenant oyatie.b2b.smb.acme-software --bucket acme-customer-pii --prefix customers/
./bin/oya storage object get --tenant oyatie.b2b.smb.acme-software --bucket acme-customer-pii --key customers/cust-42.json --output /tmp/cust-42.downloaded.json
diff /tmp/cust-42.json /tmp/cust-42.downloaded.json && echo "round-trip OK"
```

## Day 3 — versioning

Overwrite the object to create a new version:
```bash
echo '{"customer_id":"cust-42","name":"Jane Doe Smith"}' > /tmp/cust-42.json

./bin/oya storage object put \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --key "customers/cust-42.json" \
  --source /tmp/cust-42.json
```

List versions:
```bash
./bin/oya storage object versions list \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --key customers/cust-42.json
```

Expected (truncated):
```
version_id                                    is_latest  size  last_modified
01HKM5N8XWY4Z7P3R9SQGCDF2T (the new one)      true       42 B  2026-05-20T11:14:23Z
01HKM5K2P7R3D6Q9NTYABCXFEH (the original)     false      38 B  2026-05-20T10:08:12Z
```

Retrieve an older version:
```bash
./bin/oya storage object get \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --key customers/cust-42.json \
  --version-id 01HKM5K2P7R3D6Q9NTYABCXFEH \
  --output /tmp/cust-42.v1.json

cat /tmp/cust-42.v1.json   # original "Jane Doe"
```

## Day 4 — lifecycle policy + transitions

Configure a lifecycle policy that transitions objects to Warm after 30 d, Cold after 90 d, deletes after 365 d:
```bash
./bin/oya storage lifecycle policy create \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --rule-name "tier-down-customer-pii" \
  --filter-prefix "customers/" \
  --transition-warm-after-days 30 \
  --transition-cold-after-days 90 \
  --expiration-after-days 365 \
  --non-current-version-expiration-after-days 90
```

Simulate fast-forwarded time (dev profile):
```bash
./bin/oya storage lifecycle fast-forward \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --days 31
```

Verify transitions:
```bash
./bin/oya storage object head \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --key customers/cust-42.json
```

Expected: `storage_class: Warm`, `last_transition: 2026-06-20T10:01:33Z`.

## Day 5 — replication + S3-compat client

Enable a cross-region async read replica:
```bash
./bin/oya storage replication enable \
  --tenant oyatie.b2b.smb.acme-software \
  --source-bucket acme-customer-pii \
  --target-region eu-west-1 \
  --replicate-existing true \
  --replicate-deletes false \
  --replicate-versions all
```

Verify by writing in source region + reading in target region:
```bash
./bin/oya storage object put \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --key "customers/cust-99.json" \
  --source /tmp/cust-42.json \
  --region us-east-2

# Wait ~5 s for replication
./bin/oya storage object get \
  --tenant oyatie.b2b.smb.acme-software \
  --bucket acme-customer-pii \
  --key "customers/cust-99.json" \
  --region eu-west-1 \
  --output /tmp/cust-99-eu.json
```

Now use the **S3-compatible API** via `aws` CLI:
```bash
export AWS_ACCESS_KEY_ID=$(./bin/oya storage s3-credential issue --tenant oyatie.b2b.smb.acme-software --ttl 1h --keys-only)
export AWS_SECRET_ACCESS_KEY=$(./bin/oya storage s3-credential issue --tenant oyatie.b2b.smb.acme-software --ttl 1h --secret-only)
export AWS_ENDPOINT_URL=https://s3.cloud-storage.loopback.oyatie.local

aws s3 ls s3://acme-customer-pii/customers/
aws s3 cp s3://acme-customer-pii/customers/cust-42.json /tmp/cust-42-via-aws.json
```

Standard S3 SDKs (boto3, aws-sdk-rust, aws-sdk-go-v2) Just Work against this endpoint.

## What "done with week 1" means

- [ ] You can recite the `tenant_class` model and which storage classes each tenant profile uses.
- [ ] You created a bucket + put + got + listed objects with versioning enabled.
- [ ] You retrieved a specific object version by version-id.
- [ ] You configured a lifecycle policy and observed automatic class transitions.
- [ ] You enabled cross-region replication + verified the replica.
- [ ] You used the S3-compatible API via standard tooling.
- [ ] You read ADR-0248 + ADR-0251 §D-10 + ADR-0244 + S3 API basics + SEC Rule 17a-4.

## Rookie traps

1. **Skipping AAD.** Direct API calls without AAD binding decrypt as long as the DEK matches — but the SDK refuses on `cloud-storage` ≥ v0.42.
   Always use the SDK or `oya storage` CLI which inject AAD automatically.
2. **Lifecycle without "filter-prefix".** A broad lifecycle policy without filter prefix can transition / delete objects you
   didn't intend. Always scope.
3. **Object lock in compliance mode.** Once you apply object lock in compliance mode with a retention period, no one — including
   the tenant admin — can delete the object until the retention expires. Test on a dev bucket first.
4. **Cross-region replication of versioned buckets without "all-versions".** Default only replicates the current version; you
   lose version history on the replica.
5. **Forgetting to bump the lifecycle SLO clock.** Lifecycle uses tenant timezone; verify with `oya storage lifecycle preview`.
6. **Trying to use static S3 credentials.** Credentials issued via `oya storage s3-credential` are short-lived (≤ 4 h for paid baseline profiles);
   refresh via the SDK or CLI.
