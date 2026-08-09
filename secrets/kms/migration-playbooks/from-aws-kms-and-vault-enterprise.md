# Migration playbook — AWS KMS + Vault Enterprise → Oyatie `cloud-kms`

Audience: a security/SRE team running AWS KMS for cloud-side envelope encryption and HashiCorp Vault Enterprise for application
secrets + transit encryption. Goal: migrate to `cloud-kms` without re-encrypting payloads in place and without secret outage.

## Phase 0 — Inventory (Day 0…5)

### From AWS KMS

1. List all CMKs across all regions:
   ```bash
   for r in us-east-1 us-east-2 eu-west-1 ap-northeast-2; do
     aws kms list-keys --region $r > "aws-kms-$r.json"
   done
   ```
2. For each CMK, get the policy, key spec, and aliases:
   ```bash
   jq -r '.Keys[].KeyId' aws-kms-us-east-1.json | while read kid; do
     aws kms describe-key --region us-east-1 --key-id "$kid" > "cmk-$kid.json"
     aws kms get-key-policy --region us-east-1 --key-id "$kid" --policy-name default > "policy-$kid.json"
   done
   ```
3. Quantify usage: pull last 30 d of `kms:GenerateDataKey` + `kms:Decrypt` from CloudTrail to estimate hot vs cold CMKs.

### From Vault Enterprise

1. List secret engines (`vault secrets list`).
2. For `transit/` engines, list keys:
   ```bash
   vault list transit/keys > vault-transit-keys.json
   ```
3. For each transit key, export the metadata (key material is not exportable from HSM-sealed Vault):
   ```bash
   jq -r '.[]' vault-transit-keys.json | while read k; do
     vault read transit/keys/$k > "vault-key-$k.txt"
   done
   ```
4. List policies (`vault policy list`) — these map to Cedar policies on `cloud-kms` resources.

## Phase 1 — Tenant + CMK provisioning in cloud-kms (Day 5…14)

For each AWS KMS CMK + each Vault transit key, mint a corresponding `cloud-kms` CMK:
```bash
./bin/oya kms cmk create \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --alias acme-prod-customer-pii \
  --algorithm AES-256-GCM \
  --rotation-cadence 30d \
  --grace-window 90d \
  --policy pii-data
```

Tag the new CMK with the source identifier for provenance:
```bash
./bin/oya kms cmk tag \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --cmk acme-prod-customer-pii \
  --tag "source=aws-kms-arn:aws:kms:us-east-1:123456789012:key/abc-def-…"
```

## Phase 2 — Cedar policy translation (Day 14…28)

For each AWS KMS key policy + each Vault policy, author a Cedar policy on the corresponding `cloud-kms` CMK. Use the assist:
```bash
./bin/oya kms migrate translate-policy \
  --source-format aws-kms-policy \
  --source-file policy-abc-def-….json \
  --target cedar \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --output policies/acme/cmks/customer-pii.cedar
```

The translator is lossy — review every output for unrepresentable IAM principal patterns. Lint:
```bash
./bin/oya policy lint --tenant oyatie.b2b.midmarket.acme-corp policies/acme/cmks/
```

## Phase 3 — Dual-write phase (Day 28…56)

For each workload that calls AWS KMS / Vault transit today, switch the SDK to dual-write mode: every encrypt call writes to BOTH
`cloud-kms` and the legacy provider; every decrypt is fanned out and the first-to-succeed wins.

The SDK ships a dual-write wrapper:
```rust
use oya_cloud_kms_sdk::DualWrite;

let kms = DualWrite::builder()
    .primary(oya_cloud_kms_sdk::KmsClient::connect(prim_cfg).await?)
    .secondary(LegacyAwsKms::new(aws_region, cmk_arn))
    .strategy(DualWriteStrategy::EncryptBoth_DecryptFirstSuccess)
    .build()?;
```

For payloads at rest (e.g. database rows): leave existing AWS-KMS-encrypted records alone — they continue to decrypt via the
legacy path. New writes go to both.

Run divergence telemetry:
```bash
./bin/oya kms migrate divergence-report \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --since "24h ago"
```

Divergence > 0.1 % indicates AAD mismatch or schema drift — investigate before proceeding.

## Phase 4 — Re-encryption (Day 56…112; varies by data volume)

For at-rest records, batch re-encrypt under the `cloud-kms` CMK:
```bash
./bin/oya kms migrate reencrypt-at-rest \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --source legacy-aws-kms \
  --target acme-prod-customer-pii \
  --batch-size 5000 \
  --concurrency 8 \
  --resume-cursor /var/lib/oya/migrate/customer-pii.cursor
```

This iterates the source table, decrypts under AWS KMS, re-encrypts under `cloud-kms`, and writes back the new ciphertext. The
operation is resumable (cursor); a paid-tenant table of 100 M rows takes ~6 d at 10k rec/sec.

## Phase 5 — Cut-over (Day 112…140)

1. Switch the SDK from `DualWrite` to `cloud-kms` direct.
2. Verify zero traffic to AWS KMS / Vault transit for 7 d.
3. Set AWS KMS key policies to `Deny *` (keeps the key alive for scheduled deletion later).
4. Disable Vault transit engines (`vault secrets disable transit/`).

## Phase 6 — Decommission overlaps (Day 140+)

After 30 d clean run on `cloud-kms`:
- Schedule AWS KMS key deletion (`aws kms schedule-key-deletion --pending-window-in-days 30`).
- Archive Vault snapshots; revoke Vault tokens.
- Reduce Vault Enterprise license tenant_class (no transit usage).

## Rollback strategy

Within Phase 3 dual-write:
1. Flip the SDK config to `EncryptLegacy_DecryptFirstSuccess`.
2. New writes go to AWS KMS / Vault only; reads continue to fan out.
3. Quarantine `cloud-kms` for the affected tenant + investigate.

After Phase 5 cut-over:
1. Re-enable AWS KMS key policies; re-enable Vault transit engine.
2. Switch SDK back to dual-write.
3. For ciphertext stored only under `cloud-kms`: re-encrypt back to AWS KMS via the reverse playbook.

After AWS KMS key deletion: there is no rollback — the AWS CMK is destroyed. Plan deletion windows generously.

## What you gain

- 2-5× lower DEK issuance latency (4 ms vs 11-22 ms).
- 34 % TCO reduction vs AWS KMS API at mid-market scale.
- Mandatory AAD on every DEK (vs AWS KMS optional).
- Cryptoshredding as a first-class operation with HSM attestation receipts (vs scheduled deletion).
- PQC GA (ML-DSA-65, ML-KEM-768).
- BLAKE3 audit chain.
- Per-tenant compliance pack overlays.
- HTTP/3 QUIC RPC.

## What you give up

- AWS-native integration depth (200+ AWS services pre-wired to AWS KMS).
- Vault dynamic secrets ergonomics (deferred to `cloud-secrets` in Oyatie).
- Mature ecosystem tooling (Terraform AWS provider has decade-long maturity on AWS KMS).
- Marketplace HSM breadth (Vault supports more HSM vendors out-of-the-box).
