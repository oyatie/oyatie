---
purpose: Oyatie Runbook — provider-BYOK Credential Rotation Under Tenant Duress
doc_status: published
---

# Oyatie Runbook — provider-BYOK Credential Rotation Under Tenant Duress

> **Status:** Active
> **Owner:** ops-security + axis-intelligence + ops-compliance
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0255 §D-4, ADR-0244 §D-3, ADR-0243, ADR-0242

---

## §A Trigger Conditions

This runbook covers **provider credential rotation** (LLM/AI API credentials — Anthropic, OpenAI, Google Gemini, etc.) for tenants with `provider_credential_mode = 'tenant-byok'` or `'tenant-subscription'` (per ADR-0255 §D-4). This is **distinct** from encryption-BYOK (KMS/HSM root rotation; see `docs/runbooks/byok-rotation-encryption-tenant-duress.md`).

Initiate when a **specific tenant** reports or you detect:

- **Suspected provider-credential leak** — API key exposed in logs, repositories, or incident reports.
- **Regulator subpoena or government demand** — a law enforcement or regulatory authority demands the tenant's provider API key. The tenant may wish to rotate so the demanded key becomes worthless.
- **Tenant-initiated emergency rotation** — tenant requests rotation citing internal security incident (employee termination with key access, phishing compromise of credential vault, etc.).
- **Anomalous spend spike** — provider billing shows unexpected usage on a tenant's BYOK key suggesting unauthorized use.
- **Provider advisory** — Anthropic / OpenAI / Google issues a security advisory for a specific key class.

**For platform-default credential leaks** (where `provider_credential_mode = 'platform_default'`), see `docs/runbooks/provider-credential-leak-response.md`.

---

## §B Pre-Checks

Estimated time: **5 min**.

1. **Confirm tenant's `provider_credential_mode`:**
   ```
   psql -c "SELECT tenant_id, provider_credential_mode, home_cell
     FROM tenants WHERE tenant_id = '<TENANT_ID>';"
   ```
   Must be `tenant-byok` or `tenant-subscription`. If `platform_default`, redirect to `provider-credential-leak-response.md`.

2. **List active SecretReferences for the tenant's provider credentials:**
   ```
   psql -c "SELECT secret_ref_id, provider_type, owner_kind,
     openbao_path, created_at, last_rotated_at, status
     FROM secret_references
     WHERE tenant_id = '<TENANT_ID>'
       AND owner_kind IN ('tenant-byok', 'tenant-subscription')
       AND secret_type LIKE 'api-key-%'
     ORDER BY created_at DESC;"
   ```
   Record all active `secret_ref_id` values as `OLD_SECRET_REFS`.

3. **Count inflight LLM calls using this tenant's credentials:**
   ```
   curl -s http://intelligence-metrics.<CELL_ID>/metrics | \
     grep "llm_inflight_calls{tenant_id=\"<TENANT_ID>\"}"
   ```

4. **Verify Cedar permit for rotation operation:**
   ```
   cedar-cli authorize \
     --principal "oyatie.ops-security.<operator-id>" \
     --action "SecretReference::Action::RotateProviderCredential" \
     --resource "Tenant::\"<TENANT_ID>\""
   ```

5. **Contact tenant's designated security contact** (if tenant-initiated: proceed; if platform-initiated: notify before pausing calls).

---

## §C Procedure

### Step 1 — Pause inflight calls (target: ≤30s)

Install a temporary Cedar fragment that forbids new LLM dispatch for this tenant while the rotation proceeds. This prevents calls from starting with the old credential after it is revoked:

```
cat > /tmp/tenant-rotation-pause-<TENANT_ID>.cedar << 'EOF'
// TEMPORARY: rotation-pause fragment
// EXPIRES: <ISO8601 +10min>
forbid (
  principal in Tenant::"<TENANT_ID>",
  action in [Intelligence::Action::DispatchLLMCall],
  resource
)
when { context.rotation_in_progress == true };
EOF

policy-engine-cli fragment publish \
  --fragment-path /tmp/tenant-rotation-pause-<TENANT_ID>.cedar \
  --scope "tenant/<TENANT_ID>/rotation-pause" \
  --ttl-seconds 600 \
  --operator oyatie.ops-security.<operator-id>
```

Wait for hot-reload propagation (≤30s per ADR-0243 §D-10):
```
policy-engine-cli fragment verify-active \
  --scope "tenant/<TENANT_ID>/rotation-pause" --all-cells --timeout 60s
```

Allow inflight calls already dispatched to complete (grace period 30s):
```
sleep 30
```

Verify inflight count is 0:
```
curl -s http://intelligence-metrics.<CELL_ID>/metrics | \
  grep "llm_inflight_calls{tenant_id=\"<TENANT_ID>\"}" | awk '{print $2}'
```

### Step 2 — Mint new SecretReference(s) for the tenant

**Option A: Tenant provides a new API key (most common under duress)**

The tenant supplies a new API key via their admin portal (which routes through the `SecretReference` write API, never directly into the substrate):

```
# Via tenant admin API (initiated by tenant admin role):
curl -X POST https://api.<CELL_ID>/v1/tenant/<TENANT_ID>/credentials \
  -H "Authorization: Bearer <TENANT_ADMIN_JWT>" \
  -H "Content-Type: application/json" \
  -d '{
    "provider_type": "<PROVIDER>",
    "owner_kind": "tenant-byok",
    "credential": "<NEW_API_KEY>",
    "rotation_reason": "duress-rotation"
  }'
```

The API stores the new credential in OpenBao under a new path and creates a new `secret_references` row. Record `NEW_SECRET_REF_ID`.

**Option B: platform-mediated rotation (for tenant-subscription mode)**

If the tenant has a managed subscription, rotate the API key via the provider's API:

```
# Anthropic example (via provider's rotation API if available):
microservices/intelligence/bin/provider-key-rotate \
  --provider anthropic \
  --tenant-id <TENANT_ID> \
  --operator oyatie.ops-security.<operator-id>
```

### Step 3 — Validate new credential (target: ≤60s)

Before removing the old credential, verify the new one resolves and authorizes a test call:

```
microservices/intelligence/bin/credential-probe \
  --secret-ref-id <NEW_SECRET_REF_ID> \
  --provider <PROVIDER> \
  --dry-run
```

Must return `VALID`. If `INVALID`, do not proceed — troubleshoot the new credential with the tenant before revoking the old one.

### Step 4 — Atomically swap active SecretReference (target: ≤10s)

Update the tenant's active SecretReference to the new one. This is an atomic database operation:

```
psql -c "BEGIN;
  UPDATE secret_references SET status = 'SUPERSEDED', superseded_at = now()
    WHERE secret_ref_id = ANY(ARRAY[<OLD_SECRET_REFS>]::uuid[]);
  UPDATE secret_references SET status = 'ACTIVE', activated_at = now()
    WHERE secret_ref_id = '<NEW_SECRET_REF_ID>';
COMMIT;"
```

Emit:
```
audit-emit ProviderCredentialRotated \
  --tenant-id <TENANT_ID> \
  --old-secret-ref-ids <OLD_SECRET_REFS> \
  --new-secret-ref-id <NEW_SECRET_REF_ID> \
  --rotation-reason "duress" \
  --operator oyatie.ops-security.<operator-id>
```

### Step 5 — Revoke old credential at provider (target: ≤5 min)

Perform provider-side revocation of the old API key. This ensures that even if the old key was exfiltrated, it is no longer valid at the provider:

- **Anthropic:** Log into Anthropic Console → API Keys → Revoke `<OLD_KEY_LAST4>`.
- **OpenAI:** OpenAI Platform → API Keys → Delete `<OLD_KEY_LAST4>`.
- **Google:** GCP Console → APIs & Services → Credentials → Delete key `<OLD_KEY_ID>`.
- **AWS Bedrock:** IAM → Users → `<BEDROCK_IAM_USER>` → Security credentials → Delete access key.

Document revocation confirmation (screenshot or API response) in `evidence/incidents/<INCIDENT_ID>/`.

### Step 6 — Delete old credential from OpenBao (target: ≤2 min)

Once provider-side revocation is confirmed, remove the old secret from OpenBao:

```
for OLD_REF in <OLD_SECRET_REFS>; do
  OLD_PATH=$(psql -t -c "SELECT openbao_path FROM secret_references WHERE secret_ref_id = '${OLD_REF}';")
  vault kv delete "${OLD_PATH}"
  vault kv metadata delete "${OLD_PATH}"
done
```

Update `secret_references` records to `DELETED`:
```
psql -c "UPDATE secret_references SET status = 'DELETED', deleted_at = now()
  WHERE secret_ref_id = ANY(ARRAY[<OLD_SECRET_REFS>]::uuid[]);"
```

### Step 7 — Remove rotation-pause Cedar fragment (target: ≤5s)

```
policy-engine-cli fragment deactivate \
  --scope "tenant/<TENANT_ID>/rotation-pause" \
  --operator oyatie.ops-security.<operator-id>
```

Verify inflight calls resume normally:
```
sleep 10
curl -s http://intelligence-metrics.<CELL_ID>/metrics | \
  grep "llm_call_success_rate{tenant_id=\"<TENANT_ID>\"}"
```

### Step 8 — Tenant communication

If platform-initiated rotation, notify the tenant's designated security contact with:
- Confirmation the rotation completed.
- Timestamp of old-key provider-side revocation.
- New SecretReference ID (not the key itself).
- Instructions to verify their new key is working.

---

## §D Verification

1. **Only the new SecretReference is ACTIVE for this tenant:**
   ```
   psql -c "SELECT secret_ref_id, status FROM secret_references
     WHERE tenant_id = '<TENANT_ID>' AND owner_kind IN ('tenant-byok','tenant-subscription');"
   ```

2. **Old OpenBao paths no longer exist:**
   ```
   vault kv get <OLD_OPENBAO_PATH>
   ```
   Must return `No value found`.

3. **Test LLM call succeeds using new credential:**
   ```
   microservices/intelligence/bin/credential-probe --secret-ref-id <NEW_SECRET_REF_ID> --live-test
   ```

4. **Rotation-pause fragment is inactive:**
   ```
   policy-engine-cli fragment status --scope "tenant/<TENANT_ID>/rotation-pause"
   ```
   Must return `INACTIVE` or `NOT_FOUND`.

5. **Audit trail complete:** Verify `ProviderCredentialRotated` event with Merkle proof in tenant's audit stream.

---

## §E Rollback

If the new credential fails validation (Step 3) or the tenant reports the new credential is incorrect:

1. Do NOT revoke the old credential at the provider yet.
2. Revert the `secret_references` status swap:
   ```
   psql -c "BEGIN;
     UPDATE secret_references SET status = 'ACTIVE' WHERE secret_ref_id = ANY(ARRAY[<OLD_SECRET_REFS>]::uuid[]);
     UPDATE secret_references SET status = 'REVERTED' WHERE secret_ref_id = '<NEW_SECRET_REF_ID>';
   COMMIT;"
   ```
3. Remove rotation-pause fragment (Step 7).
4. Coordinate with tenant to supply a valid new credential and retry from Step 2.

---

## §F Post-Incident

1. For duress rotations triggered by credential leak: file `ProviderCredentialLeak` incident report in `evidence/incidents/`.
2. Review audit-chain for unauthorized calls made using the old credential during the exposure window.
3. If unauthorized calls are found, assess per `docs/runbooks/provider-credential-leak-response.md` §C (audit replay).
4. Check whether the tenant's compliance pack requires breach notification (e.g., if PHI was transmitted via the potentially-compromised credential under a HIPAA pack).

---

## §G References

- ADR-0255 §D-4 (Opt-in LLM/provider-BYOK credential model)
- ADR-0255 §D-4 DDL (`secret_references` table)
- ADR-0244 §D-3 (`provider_credential_mode` on tenants)
- ADR-0243 §D-10 (Hot-reload semantics)
- `docs/runbooks/provider-credential-leak-response.md`
- `docs/runbooks/byok-rotation-encryption-tenant-duress.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
