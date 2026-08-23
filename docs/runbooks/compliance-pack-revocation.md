---
purpose: Oyatie Runbook — Compliance Pack Emergency Revocation
doc_status: published
---

# Oyatie Runbook — Compliance Pack Emergency Revocation

> **Status:** Active
> **Owner:** ops-compliance + ops-security + council-security
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0251 §D-2, ADR-0251 §D-4, ADR-0251 §D-8, ADR-0243 §D-5, ADR-0243 §D-10

---

## §A Trigger Conditions

Initiate this runbook when **any** of the following occur for a published Compliance Pack:

- **Signing-key compromise** — the compliance-office Ed25519 key (or a co-signer key) is suspected or confirmed compromised.
- **Cosign attestation chain break** — Sigstore Rekor record for the pack's `attestation_blob_ref` is found invalid, tampered, or duplicated.
- **Critical legal interpretation error** — the pack's `REGULATORY-MAPPING.md` is found to contain materially incorrect regulatory citations with regulatory or legal exposure.
- **Regulator directive** — a regulator authority directly instructs oyatie to withdraw the pack from active use.
- **Cedar fragment vulnerability** — a pack's Cedar fragment is found to permit actions it should forbid (privilege-escalation class), with active exploitation risk.

**Distinguish from pack suspension** (grace-period based, non-emergency) — use `docs/runbooks/compliance-pack-emergency-suspension.md` when there is no active compromise and a grace period is acceptable.

---

## §B Pre-Checks

Estimated time: **5–10 min**.

1. **Identify the pack and version(s) to revoke.** Confirm pack_id, version range, and whether all versions or only specific versions are affected:
   ```
   psql -c "SELECT pack_id, version, effective_at, sunset_at, signed_by->>'signer_key_id' as key_id,
     signed_by->>'attestation_blob_ref' as rekor_ref
     FROM compliance_packs WHERE pack_id = '<PACK_ID>' ORDER BY version;"
   ```

2. **Enumerate affected tenants:**
   ```
   psql -c "SELECT tenant_id, installed_at, installed_by
     FROM tenant_compliance_packs
     WHERE pack_id = '<PACK_ID>' AND version = ANY(ARRAY[<VERSIONS>])
       AND status = 'ACTIVE';"
   ```
   Record `AFFECTED_TENANT_COUNT`.

3. **Assess regulatory notification requirement.** For packs with `breach_notification_workflow.regulator_notification_deadline_hours` set, determine if the revocation trigger constitutes a breach event requiring regulator notification. Consult `council-legal` if uncertain.

4. **Check Cedar permit:**
   ```
   cedar-cli authorize \
     --principal "oyatie.ops-compliance.<operator-id>" \
     --action "CompliancePack::Action::RevokePackVersion" \
     --resource "CompliancePack::\"<PACK_ID>::<VERSION>\""
   ```

5. **Declare incident.** SEV-1. Open in `#incident-bridge`. Assign incident commander from `council-security`. Notify `council-legal` and `council-privacy` immediately.

---

## §C Procedure

### Step 1 — Freeze new pack installations (target: ≤2 min)

Immediately block new tenant installations of the affected pack versions:

```
psql -c "UPDATE compliance_packs SET status = 'REVOCATION_PENDING',
  revocation_initiated_at = now(),
  revocation_reason = '<REASON>'
  WHERE pack_id = '<PACK_ID>' AND version = ANY(ARRAY[<VERSIONS>]);"
```

Emit:
```
audit-emit CompliancePackRevocationInitiated \
  --pack-id <PACK_ID> \
  --versions <VERSIONS> \
  --operator oyatie.ops-compliance.<operator-id> \
  --reason "<REASON>"
```

### Step 2 — Revoke Cosign + Rekor attestations (target: ≤10 min)

For each pack version being revoked, invalidate its Sigstore Rekor entry. Rekor is append-only so revocation is accomplished by publishing a revocation record:

```
cosign attest --predicate revocation-predicate.json \
  --key /hsm/compliance-revocation-key \
  <PACK_ARTIFACT_DIGEST>
```

Where `revocation-predicate.json` contains:
```json
{
  "predicateType": "https://oyatie.internal/revocation/v1",
  "subject_pack_id": "<PACK_ID>",
  "subject_versions": ["<VERSION>"],
  "revocation_reason": "<REASON>",
  "revoked_at": "<ISO8601_TIMESTAMP>",
  "revoked_by": "oyatie.ops-compliance.<operator-id>"
}
```

Record the Rekor revocation entry ID as `REVOCATION_REKOR_ID`.

### Step 3 — Deactivate Cedar fragments for affected pack (target: ≤5 min per fragment)

For each Cedar fragment declared in the pack's `cedar_fragments[]` array, issue an emergency deactivation. Per ADR-0243 §D-10 hot-reload semantics, all data-plane cells will pick up the change within ≤30s of the next bundle snapshot:

```
for FRAGMENT_ID in $(jq -r '.cedar_fragments[].fragment_id' pack-<PACK_ID>-<VERSION>.yaml); do
  policy-engine-cli fragment deactivate \
    --fragment-id "pack/<PACK_ID>/${FRAGMENT_ID}" \
    --reason "pack-revocation" \
    --operator oyatie.ops-compliance.<operator-id>
done
```

Each deactivation emits `CedarFragmentDeactivated` to audit-chain.

**Wait for hot-reload propagation (target: ≤30s per ADR-0243 §D-10):**
```
policy-engine-cli fragment verify-deactivated \
  --fragment-id "pack/<PACK_ID>/*" \
  --all-cells \
  --timeout 60s
```

### Step 4 — Downgrade cell certification levels (target: ≤20 min)

For each cell whose certification level was contingent solely on the revoked pack's Cedar fragments:

1. Identify affected cells:
   ```
   psql -c "SELECT cell_id, certification_levels FROM cells
     WHERE '<PACK_CERT_LEVEL>' = ANY(certification_levels);"
   ```

2. For each affected cell, check whether other evidence supports the certification level (certificates, audit attestations). If not, downgrade:
   ```
   psql -c "UPDATE cells
     SET certification_levels = array_remove(certification_levels, '<PACK_CERT_LEVEL>'),
         certification_downgrade_reason = 'pack-revocation:<PACK_ID>:<VERSION>',
         certification_downgrade_at = now()
     WHERE cell_id = '<CELL_ID>';"
   ```

3. Tenants in those cells whose packs require the downgraded certification level are now in violation of ADR-0251 §D-5 pinning rules. Quarantine them:
   ```
   microservices/tenancy/bin/quarantine-pinning-violations \
     --cells <AFFECTED_CELL_IDS> \
     --notify-tenants \
     --grace-period-hours 24
   ```

### Step 5 — Uninstall pack from affected tenants (target: ≤30 min)

Bulk-uninstall the revoked pack versions from all affected tenants. This triggers the pack-uninstall workflow (ADR-0251 §D-3) in emergency mode (skipping DPO/council-legal countersign for the immediate operational step, with retroactive documentation required within 24h):

```
microservices/governance/bin/pack-revoke \
  --pack-id <PACK_ID> \
  --versions <VERSIONS> \
  --mode emergency \
  --operator oyatie.ops-compliance.<operator-id> \
  --notify-tenants
```

The workflow emits `CompliancePackUninstalled` for each tenant. Tenant notifications are queued for delivery via the notification substrate.

### Step 6 — Regulator notification (time-bound by pack's `regulator_notification_deadline_hours`)

If the revocation trigger constitutes a reportable event under the pack's `breach_notification_workflow`:

1. Retrieve breach notification workflow parameters:
   ```
   jq '.breach_notification_workflow' pack-<PACK_ID>-<VERSION>.yaml
   ```

2. Trigger the breach notification workflow:
   ```
   workflow-cli start oyatie.foundry.breach-notification \
     --pack-id <PACK_ID> \
     --trigger pack-revocation \
     --regulator-deadline-hours <HOURS> \
     --incident-ref <INCIDENT_ID>
   ```

   For EU GDPR packs: 72h deadline (Article 33). For KR-PIPA packs: 24h deadline (Article 34). For HIPAA packs: 60-day deadline (§164.404) — but initial regulator contact within 72h is best practice.

3. Escalate to `docs/runbooks/breach-notification-council-escalation.md` for regulator communication drafting.

### Step 7 — Publish revocation notice (target: ≤60 min)

Update the compliance pack registry with the final revocation status:

```
psql -c "UPDATE compliance_packs SET status = 'REVOKED',
  revoked_at = now(),
  revocation_rekor_id = '<REVOCATION_REKOR_ID>',
  revocation_operator = 'oyatie.ops-compliance.<operator-id>'
  WHERE pack_id = '<PACK_ID>' AND version = ANY(ARRAY[<VERSIONS>]);"
```

Emit final audit event:
```
audit-emit CompliancePackRevoked \
  --pack-id <PACK_ID> \
  --versions <VERSIONS> \
  --revocation-rekor-id <REVOCATION_REKOR_ID> \
  --affected-tenants <AFFECTED_TENANT_COUNT>
```

---

## §D Verification

1. **Pack status = REVOKED in registry:**
   ```
   psql -c "SELECT pack_id, version, status FROM compliance_packs
     WHERE pack_id = '<PACK_ID>';"
   ```

2. **No active tenant installations remain:**
   ```
   psql -c "SELECT COUNT(*) FROM tenant_compliance_packs
     WHERE pack_id = '<PACK_ID>' AND status = 'ACTIVE';"
   ```
   Must return `0`.

3. **Cedar fragments are deactivated across all cells:**
   ```
   policy-engine-cli fragment verify-deactivated --fragment-id "pack/<PACK_ID>/*" --all-cells
   ```

4. **Audit trail complete.** Verify Merkle proofs for `CompliancePackRevocationInitiated`, `CedarFragmentDeactivated` (all fragments), and `CompliancePackRevoked` events are present and linked.

5. **Regulator notification sent** (where required): confirm `breach-notification` workflow reached `REGULATOR_NOTIFIED` state within the deadline.

---

## §E Rollback

Pack revocation is **not fully reversible** once regulator notification has been sent. For false-positive revocations (e.g., incorrect key compromise report):

1. Restore Cedar fragments via the standard fragment authoring + activation flow (ADR-0243 §D-2), using uncompromised signing keys.
2. Re-publish the pack with a new signing-key attestation and increment the patch version.
3. Notify affected tenants of the false-positive and offer re-installation.
4. File a post-incident report explaining the false positive to `council-security` + `council-legal`.

---

## §F Post-Incident

1. Root-cause analysis on the signing-key compromise or attestation chain break. File in `evidence/incidents/`.
2. If the signing key was compromised, execute `docs/runbooks/meta-trust-root-recovery.md` if the compromised key is part of the meta-trust chain.
3. Author a replacement pack version with a new signing key and the vulnerability addressed.
4. Update the Shamir key-holder roster if the compromise involved an HSM ceremony key.
5. Post-mortem required within 72h.

---

## §G References

- ADR-0251 §D-2 (Pack lifecycle)
- ADR-0251 §D-3 (Tenant pack installation / uninstall)
- ADR-0251 §D-4 (Cell certification level matrix)
- ADR-0251 §D-5 (Tenant-pack cell pinning)
- ADR-0251 §D-8 (Breach notification)
- ADR-0243 §D-5 (Bootstrap chain of trust)
- ADR-0243 §D-10 (Hot-reload semantics)
- `docs/runbooks/breach-notification-council-escalation.md`
- `docs/runbooks/compliance-pack-emergency-suspension.md`
- `docs/runbooks/cedar-fragment-emergency-rollback.md`
- `docs/runbooks/meta-trust-root-recovery.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
