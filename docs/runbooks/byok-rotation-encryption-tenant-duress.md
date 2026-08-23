---
purpose: Oyatie Runbook — encryption-BYOK (KMS Root / HSM Partition) Rotation Under Tenant Duress
doc_status: published
---

# Oyatie Runbook — encryption-BYOK (KMS Root / HSM Partition) Rotation Under Tenant Duress

> **Status:** Active
> **Owner:** ops-security + council-security + ops-compliance
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0251 §D-10, ADR-0244 §D-3, ADR-0243, ADR-0009, ADR-0049

---

## §A Trigger Conditions

This runbook covers **encryption-BYOK rotation** — specifically the rotation of a tenant's KMS root key or HSM partition key used for at-rest data encryption (CMEK pattern). This is **distinct** from provider credential BYOK rotation (LLM/API keys, `provider_credential_mode` field); see `docs/runbooks/byok-rotation-provider-tenant-duress.md` for that.

Governed by ADR-0251 §D-10, tracked by `byok_enabled = TRUE` on the `tenants` table (per ADR-0244 §D-3).

Initiate when:

- **Suspected or confirmed HSM partition compromise** — unauthorized access to the tenant's HSM partition is suspected.
- **KMS root key exposure** — the tenant's root encryption key material is suspected to have egressed the HSM (e.g., via HSM firmware vulnerability, insider threat).
- **Regulator subpoena / legal hold interaction** — the tenant wishes to rotate ahead of a government demand to ensure data encrypted under the current key cannot be read even if the key is compelled.
- **Tenant-initiated emergency rotation** — tenant security team triggers rotation citing internal incident (personnel departure with HSM admin access, phishing compromise of HSM admin credentials).
- **HSM vendor advisory** — the HSM vendor (Thales Luna, AWS CloudHSM, Naver Cloud HSM per ADR-0248 §D-10) issues a critical firmware advisory requiring partition re-keying.

---

## §B Pre-Checks

Estimated time: **10–20 min**. This is a higher-risk operation than provider-credential rotation; extra pre-checks are required.

1. **Confirm tenant has encryption-BYOK enabled:**
   ```
   psql -c "SELECT tenant_id, byok_enabled, home_cell, dr_cell
     FROM tenants WHERE tenant_id = '<TENANT_ID>';"
   ```
   `byok_enabled` must be `TRUE`. If `FALSE`, this runbook does not apply (platform manages encryption; escalate to `council-security`).

2. **Identify the tenant's current KMS key and OpenBao transit path:**
   ```
   psql -c "SELECT kms_key_ref, openbao_transit_path, hsm_partition_id,
     key_algorithm, created_at, last_rotated_at
     FROM tenant_encryption_keys
     WHERE tenant_id = '<TENANT_ID>' AND status = 'ACTIVE';"
   ```
   Record `OLD_KMS_KEY_REF`, `OLD_OPENBAO_TRANSIT_PATH`, `OLD_HSM_PARTITION_ID`.

3. **Estimate data volume to re-key.** Get approximate count of encrypted blobs:
   ```
   psql -c "SELECT relname, n_live_tup FROM pg_stat_user_tables
     WHERE relname LIKE 'tenant_<TENANT_ID>%' OR
           relname IN ('objects','messages','documents','audit_rows')
     ORDER BY n_live_tup DESC;"
   ```
   Large volumes (>100M rows) require scheduling the re-key job off-peak. Consult `ops-dr-capacity` if >500M rows.

4. **Verify DR cell is in sync.** Replication lag must be ≤5s before proceeding:
   ```
   observability cell-replication-lag-check \
     --tenant-id <TENANT_ID> --source-cell <HOME_CELL> --target-cell <DR_CELL>
   ```

5. **Verify Cedar permit:**
   ```
   cedar-cli authorize \
     --principal "oyatie.council-security.<operator-id>" \
     --action "EncryptionKey::Action::InitiateTenantKeyRotation" \
     --resource "Tenant::\"<TENANT_ID>\""
   ```

6. **Declare incident.** SEV-2 (planned rotation) or SEV-1 (active compromise). Notify `council-security`, `ops-compliance`, `council-privacy`. If data residency is EU, notify DPO.

---

## §C Procedure

### Step 1 — Pause write operations for the tenant (target: ≤30s)

Install a temporary Cedar fragment to block new write operations that would generate newly encrypted blobs during the re-key window:

```
cat > /tmp/tenant-rekey-pause-<TENANT_ID>.cedar << 'EOF'
// TEMPORARY: encryption re-key pause
// EXPIRES: <ISO8601 +60min>
forbid (
  principal in Tenant::"<TENANT_ID>",
  action in [
    Data::Action::WriteObject,
    Data::Action::UpdateObject,
    Data::Action::CreateRecord
  ],
  resource
)
when { context.encryption_rekey_in_progress == true };
EOF

policy-engine-cli fragment publish \
  --fragment-path /tmp/tenant-rekey-pause-<TENANT_ID>.cedar \
  --scope "tenant/<TENANT_ID>/rekey-pause" \
  --ttl-seconds 3600 \
  --operator oyatie.council-security.<operator-id>
```

Wait for propagation (≤30s per ADR-0243 §D-10), then verify zero new write attempts:
```
audit-stream tail --tenant-id <TENANT_ID> --action Data::Action::WriteObject --window 30s
```

Allow inflight writes to complete (grace period 60s).

### Step 2 — Generate new KMS key / HSM partition (target: ≤10 min)

Depending on the tenant's HSM provider:

**AWS CloudHSM:**
```
cloudhsm-cli key generate-symmetric AES --key-size 256 \
  --label "tenant-<TENANT_ID>-root-<TIMESTAMP>" \
  --partition <NEW_HSM_PARTITION_ID>
```

**Thales Luna HSM:**
```
luna-slot key generate --mechanism AES --size 256 \
  --label "tenant-<TENANT_ID>-root-<TIMESTAMP>" \
  --slot <NEW_SLOT_ID>
```

**Naver Cloud HSM (KR pack):**
```
ncloudhsm key create --algorithm AES256 \
  --label "tenant-<TENANT_ID>-root-<TIMESTAMP>"
```

Register the new key in OpenBao transit:
```
vault write transit/keys/tenant-<TENANT_ID>-<TIMESTAMP> \
  type=aes256-gcm96 \
  exportable=false \
  allow_plaintext_backup=false
```

Record `NEW_KMS_KEY_REF`, `NEW_OPENBAO_TRANSIT_PATH`.

Insert into `tenant_encryption_keys` as `PENDING`:
```
psql -c "INSERT INTO tenant_encryption_keys
  (tenant_id, kms_key_ref, openbao_transit_path, hsm_partition_id, key_algorithm, status, created_at)
  VALUES ('<TENANT_ID>', '<NEW_KMS_KEY_REF>', '<NEW_OPENBAO_TRANSIT_PATH>', '<NEW_HSM_PARTITION_ID>',
          'AES-256-GCM', 'PENDING', now())
  RETURNING key_id;" AS NEW_KEY_ID
```

### Step 3 — Re-key all tenant data via OpenBao transit migration (target: varies by volume)

OpenBao transit re-wraps all key encryption keys (KEKs) that wrap the tenant's data encryption keys (DEKs). This is a key-wrap migration, not a data re-encryption — only the KEK wrapping changes; the DEKs themselves are re-wrapped under the new root.

```
vault write transit/rewrap/tenant-<TENANT_ID> \
  ciphertext="$(vault kv get -field=current_kek secret/tenants/<TENANT_ID>/kek)" \
  name="tenant-<TENANT_ID>-<TIMESTAMP>"
```

For large tenants, use the bulk re-wrap job:
```
microservices/cloud-secrets/bin/tenant-rekey \
  --tenant-id <TENANT_ID> \
  --old-transit-path <OLD_OPENBAO_TRANSIT_PATH> \
  --new-transit-path <NEW_OPENBAO_TRANSIT_PATH> \
  --batch-size 10000 \
  --progress
```

The job emits `TenantDataReKeyBatchCompleted` audit events per batch. Monitor progress:
```
audit-stream tail --tenant-id <TENANT_ID> --event TenantDataReKeyBatchCompleted
```

**Expected timing:** ~1 min per 100k rows (key-wrap operations, not data I/O). For 10M rows, allow ~100 min. Schedule off-peak if necessary.

### Step 4 — Re-seal audit-row integrity (target: ≤15 min)

Per ADR-0251 §D-10 and audit-chain requirements, audit rows for this tenant must remain tamper-evident after re-keying. The audit-chain re-seal process re-wraps the audit-row signing envelope under the new key:

```
audit-chain-cli reseal-tenant \
  --tenant-id <TENANT_ID> \
  --new-kms-key-ref <NEW_KMS_KEY_REF> \
  --verify-merkle-integrity \
  --operator oyatie.council-security.<operator-id>
```

This emits `AuditChainResealCompleted` with a new Merkle root anchored to the new key. The old Merkle chain is preserved as an immutable historical record; the new chain starts from the re-seal point.

### Step 5 — Atomically promote new key to ACTIVE (target: ≤5s)

```
psql -c "BEGIN;
  UPDATE tenant_encryption_keys SET status = 'SUPERSEDED', superseded_at = now()
    WHERE tenant_id = '<TENANT_ID>' AND status = 'ACTIVE';
  UPDATE tenant_encryption_keys SET status = 'ACTIVE', activated_at = now()
    WHERE key_id = '<NEW_KEY_ID>';
COMMIT;"
```

Emit:
```
audit-emit TenantEncryptionKeyRotated \
  --tenant-id <TENANT_ID> \
  --old-kms-key-ref <OLD_KMS_KEY_REF> \
  --new-kms-key-ref <NEW_KMS_KEY_REF> \
  --rotation-reason "duress" \
  --operator oyatie.council-security.<operator-id>
```

### Step 6 — Remove write-pause Cedar fragment (target: ≤5s)

```
policy-engine-cli fragment deactivate \
  --scope "tenant/<TENANT_ID>/rekey-pause" \
  --operator oyatie.council-security.<operator-id>
```

Verify writes resume:
```
sleep 10
audit-stream tail --tenant-id <TENANT_ID> --action Data::Action::WriteObject --window 30s
```

### Step 7 — Destroy old HSM key material (target: ≤10 min)

Once the re-key is verified complete (§D below), destroy the old key. This step is **irreversible**:

```
vault delete transit/keys/tenant-<TENANT_ID>-old
# For HSM-backed keys, physically destroy via HSM management interface:
# Thales: luna-slot key delete --label "tenant-<TENANT_ID>-root-<OLD_TIMESTAMP>"
# AWS CloudHSM: cloudhsm-cli key delete --filter attr.label=tenant-<TENANT_ID>-root-<OLD_TIMESTAMP>
```

Update database:
```
psql -c "UPDATE tenant_encryption_keys SET status = 'DESTROYED', destroyed_at = now()
  WHERE kms_key_ref = '<OLD_KMS_KEY_REF>';"
```

---

## §D Verification

1. **All data successfully re-keyed:**
   ```
   microservices/cloud-secrets/bin/tenant-rekey --tenant-id <TENANT_ID> --verify-only
   ```
   Must report `0 blobs remaining under old key`.

2. **New key is ACTIVE, old key is DESTROYED:**
   ```
   psql -c "SELECT kms_key_ref, status FROM tenant_encryption_keys
     WHERE tenant_id = '<TENANT_ID>' ORDER BY created_at DESC;"
   ```

3. **Audit-chain Merkle integrity verified:**
   ```
   audit-chain-cli verify-integrity --tenant-id <TENANT_ID> --since-reseal
   ```

4. **Tenant read/write operations succeed with new key:**
   ```
   microservices/cloud-secrets/bin/encrypt-probe \
     --tenant-id <TENANT_ID> \
     --kms-key-ref <NEW_KMS_KEY_REF> \
     --test-plaintext "rotation-verification-<TIMESTAMP>"
   ```

5. **DR cell re-key also complete:**
   ```
   microservices/cloud-secrets/bin/tenant-rekey \
     --tenant-id <TENANT_ID> --cell <DR_CELL> --verify-only
   ```

---

## §E Rollback

Encryption re-key has a narrow rollback window — only before Step 5 (key promotion):

- If the re-key job fails partway through (Step 3), the old key is still ACTIVE and data is consistent. Abort the `NEW_KEY_ID` record, clean up the new OpenBao path, and retry.
- After Step 5 (key promotion), rollback is possible only if the old key has **not yet been destroyed** (Step 7). In that case:
  ```
  psql -c "BEGIN;
    UPDATE tenant_encryption_keys SET status = 'ACTIVE' WHERE kms_key_ref = '<OLD_KMS_KEY_REF>';
    UPDATE tenant_encryption_keys SET status = 'REVERTED' WHERE key_id = '<NEW_KEY_ID>';
  COMMIT;"
  ```
  Then re-key back to the old key via Step 3.
- After Step 7 (old key destroyed), rollback is **not possible**. Data remains accessible via the new key. If the new key is also lost, data is irrecoverable — escalate immediately to `council-security`.

---

## §F Post-Incident

1. File `TenantEncryptionKeyCompromise` incident report if triggered by actual compromise.
2. Verify compliance pack retention rules are still satisfied with the new key (audit-chain re-seal does not affect retention counters).
3. Schedule next scheduled key rotation per the tenant's pack requirements (e.g., HIPAA recommends annual rotation; PCI DSS requires annual rotation).
4. If HSM partition was compromised, coordinate with HSM vendor on firmware audit.
5. Post-mortem within 72h for SEV-1 incidents.

---

## §G References

- ADR-0251 §D-10 (encryption-BYOK; `byok_enabled` field)
- ADR-0244 §D-3 (`byok_enabled` on tenants; distinct from `provider_credential_mode`)
- ADR-0009 (Per-cell HSM partition per tenant)
- ADR-0049 (Cross-region replication — DR cell must also be re-keyed)
- ADR-0243 §D-10 (Hot-reload for Cedar pause fragments)
- `docs/runbooks/byok-rotation-provider-tenant-duress.md`
- `docs/runbooks/audit-chain-integrity-recovery.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
