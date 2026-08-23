---
purpose: Oyatie Runbook — Compliance Pack Emergency Suspension
doc_status: published
---

# Oyatie Runbook — Compliance Pack Emergency Suspension

> **Status:** Active
> **Owner:** ops-compliance + council-legal + council-privacy
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0251 §D-2, ADR-0251 §D-3, ADR-0251 §D-8, ADR-0243

---

## §A Trigger Conditions

**Suspension is distinct from revocation.** Revocation (`docs/runbooks/compliance-pack-revocation.md`) is for active key compromise or security vulnerability requiring immediate hard removal. Suspension is for situations where continued activation of a pack is no longer safe or appropriate, but a grace period is feasible:

- **Regulator pulls a framework** — a regulation is amended, repealed, or superseded and the current pack version no longer accurately represents the regulatory obligation (e.g., a new PCI DSS major version makes the old pack version obsolete with a grace period before deadline).
- **New vulnerability discovered in pack's Cedar fragments** — a Cedar fragment in the pack has a logic error that permits or denies more than intended, but exploitation requires deliberate action (not an auto-exploit like a key compromise). There is time to draft a replacement pack version while suspending new installations.
- **Pack author organization loses regulatory standing** — the external co-signer (e.g., auditor firm, legal counsel) loses their accreditation or is subject to regulatory sanction.
- **Regulatory authority issues corrective guidance** — the regulator issues guidance that conflicts with the pack's current interpretation; pack must be suspended pending re-interpretation.
- **DPIA or legal review finds material gap** — internal DPO or legal counsel finds the pack's regulatory mapping materially incomplete or incorrect, with legal exposure but not an active security incident.

---

## §B Pre-Checks

Estimated time: **10–15 min**.

1. **Identify the pack version(s) to suspend:**
   ```
   psql -c "SELECT pack_id, version, effective_at, sunset_at, status
     FROM compliance_packs WHERE pack_id = '<PACK_ID>' ORDER BY version DESC;"
   ```

2. **Enumerate currently active tenant installations:**
   ```
   psql -c "SELECT tenant_id, installed_at, pack_version
     FROM tenant_compliance_packs
     WHERE pack_id = '<PACK_ID>' AND status = 'ACTIVE';"
   ```
   Record `ACTIVE_TENANT_COUNT` and `TENANT_IDS`.

3. **Determine grace period.** Based on the trigger:
   | Trigger | Recommended grace period |
   |---|---|
   | Regulatory amendment with published transition date | Align with regulator's stated transition date |
   | Pack logic error (non-exploitable) | 30 days (time to author + review replacement) |
   | Co-signer loss of accreditation | 90 days (time to find replacement co-signer) |
   | Regulatory corrective guidance | 30 days or until regulator confirms interpretation |
   
   For grace periods affecting regulated industries, confirm with `council-legal` before announcing.

4. **Identify the replacement path:**
   - Is a newer pack version already in draft?
   - Is the fix a minor amendment (patch version) or a major re-interpretation (major version)?
   - Who is the replacement co-signer if required?

5. **Verify Cedar permit:**
   ```
   cedar-cli authorize \
     --principal "oyatie.ops-compliance.<operator-id>" \
     --action "CompliancePack::Action::SuspendPackVersion" \
     --resource "CompliancePack::\"<PACK_ID>::<VERSION>\""
   ```

6. **Declare incident.** SEV-2 for active regulatory guidance conflict. SEV-3 for planned pack version supersession. Notify `council-legal`, `council-privacy`, relevant `ops-compliance` contacts.

---

## §C Procedure

### Step 1 — Block new tenant installations (target: ≤5 min)

Prevent new tenants from installing the suspended pack version(s):

```
psql -c "UPDATE compliance_packs
  SET status = 'SUSPENDED',
      suspended_at = now(),
      suspension_reason = '<REASON>',
      suspension_grace_period_ends_at = now() + interval '<GRACE_DAYS> days'
  WHERE pack_id = '<PACK_ID>'
    AND version = ANY(ARRAY[<VERSIONS>]);"
```

Emit:
```
audit-emit CompliancePackSuspended \
  --pack-id <PACK_ID> \
  --versions <VERSIONS> \
  --grace-period-days <GRACE_DAYS> \
  --operator oyatie.ops-compliance.<operator-id> \
  --reason "<REASON>"
```

The pack-install workflow will now return an error for these versions pointing to the successor version or the suspension notice.

### Step 2 — Notify existing tenants (target: ≤24h)

Draft and send tenant notification. For each tenant in `TENANT_IDS`:

```
workflow-cli start oyatie.foundry.compliance-pack-notification \
  --pack-id <PACK_ID> \
  --versions <VERSIONS> \
  --notification-type suspension \
  --grace-period-days <GRACE_DAYS> \
  --replacement-version "<REPLACEMENT_VERSION_IF_KNOWN>" \
  --tenant-ids <TENANT_IDS>
```

The notification must include:
- Which pack version is suspended.
- Why (at the appropriate level of detail for the audience — regulatory change, not security incident).
- The grace period end date.
- The replacement path (upgrade to a new pack version OR migrate away from the regulation).
- Tenant's responsibilities during the grace period (continue compliance practices even though the pack is suspended).
- Contact for questions (`council-legal` escalation path for regulated tenants).

### Step 3 — Author replacement pack version (target: within grace period)

Assign a pack author to draft the replacement pack version. Per ADR-0251 §D-2:

1. Create the new pack version directory:
   ```
   mkdir -p microservices/governance/packs/<PACK_ID>/v<NEW_VERSION>/
   cp -r microservices/governance/packs/<PACK_ID>/v<OLD_VERSION>/ \
     microservices/governance/packs/<PACK_ID>/v<NEW_VERSION>/
   ```

2. Apply the required changes:
   - Update `pack.yaml` with corrected regulatory citations and new `version` field.
   - Update `REGULATORY-MAPPING.md` with corrected Article/Section citations.
   - Update Cedar fragments if the logic error is being fixed.
   - Update `CHANGELOG.md` with the suspension reason and fix description.
   - Update the `supersedes_pack_versions` field to reference the suspended version.

3. Route through multispectrum review (ADR-0251 §D-2 Stage 2). Fast-track review for pack suspension remediation:
   ```
   workflow-cli start oyatie.foundry.pr-review \
     --target-branch pack/<PACK_ID>-v<NEW_VERSION> \
     --review-mode pack-suspension-remediation
   ```

4. Sign and publish per ADR-0251 §D-2 Stage 3–4.

### Step 4 — Tenant migration workflow (target: before grace period end)

Once the replacement version is published, run the tenant migration workflow for all currently-suspended-pack tenants:

```
workflow-cli start oyatie.foundry.compliance-pack-version-migration \
  --pack-id <PACK_ID> \
  --from-version <OLD_VERSION> \
  --to-version <NEW_VERSION> \
  --tenant-ids <TENANT_IDS> \
  --deadline "<GRACE_PERIOD_END_DATE>"
```

The migration workflow:
1. Invokes the pack-install workflow for the new version for each tenant.
2. Uninstalls the old version if the new version successfully installs.
3. Emits `CompliancePackVersionMigrated` for each tenant.

For tenants that cannot upgrade (e.g., they do not meet the new version's `cell_eligibility` requirements):
- Identify via:
  ```
  presubmit (retired CLI gate validate) tenant-pack-cell-pinning --pack-id <PACK_ID> --version <NEW_VERSION> --tenant-ids <TENANT_IDS>
  ```
- Escalate blockers to `ops-compliance` for case-by-case resolution.

### Step 5 — Regulator communication (where required)

For suspensions triggered by regulatory guidance conflict:

1. Prepare a regulatory communication explaining:
   - The conflict between the prior pack interpretation and the new regulatory guidance.
   - The corrective action (new pack version authored).
   - Timeline for tenant migration.
   - Interim compliance posture of existing tenants during the grace period.

2. Trigger communication via the breach-notification framework where the suspension constitutes a reportable event:
   ```
   workflow-cli start oyatie.foundry.regulator-communication \
     --pack-id <PACK_ID> \
     --communication-type pack-suspension \
     --incident-ref <INCIDENT_ID>
   ```

   EU NIS2 (Article 23): If the pack suspension reveals a cybersecurity incident, three-stage cadence applies (24h early warning, 72h incident notification, 1-month final report per synthesis §5.15 / F13 P1 fix).
   EU DSA (Article 24+28): Semi-annual transparency report must reference material pack changes.

### Step 6 — Archive suspended version (after grace period)

Once the grace period ends:

1. Confirm all tenants have migrated:
   ```
   psql -c "SELECT COUNT(*) FROM tenant_compliance_packs
     WHERE pack_id = '<PACK_ID>'
       AND version = ANY(ARRAY[<OLD_VERSIONS>])
       AND status = 'ACTIVE';"
   ```
   Must return `0`.

2. Move the suspended version to archived status:
   ```
   psql -c "UPDATE compliance_packs SET status = 'ARCHIVED', archived_at = now()
     WHERE pack_id = '<PACK_ID>' AND version = ANY(ARRAY[<OLD_VERSIONS>]);"
   ```

3. Emit:
   ```
   audit-emit CompliancePackVersionArchived \
     --pack-id <PACK_ID> \
     --versions <OLD_VERSIONS> \
     --operator oyatie.ops-compliance.<operator-id>
   ```

---

## §D Verification

1. **Suspended versions show status SUSPENDED (blocking new installations):**
   ```
   psql -c "SELECT pack_id, version, status FROM compliance_packs
     WHERE pack_id = '<PACK_ID>';"
   ```

2. **All tenants notified** (notification workflow completed for all `TENANT_IDS`).

3. **Replacement version published and passing cell-pinning validation:**
   ```
   presubmit (retired CLI gate validate) compliance-pack-schema --pack-id <PACK_ID> --version <NEW_VERSION>
   ```

4. **After grace period: zero tenants on old version** (§C Step 6 check).

5. **Audit trail complete:** `CompliancePackSuspended`, `CompliancePackVersionMigrated` (per tenant), `CompliancePackVersionArchived` events present with Merkle proofs.

---

## §E Rollback

If the suspension was issued in error (e.g., regulatory guidance was misinterpreted and the pack is actually compliant):

1. Restore the pack to ACTIVE status:
   ```
   psql -c "UPDATE compliance_packs SET status = 'ACTIVE', suspended_at = NULL,
     suspension_reason = NULL, suspension_grace_period_ends_at = NULL
     WHERE pack_id = '<PACK_ID>' AND version = ANY(ARRAY[<VERSIONS>]);"
   ```

2. Notify tenants of the suspension reversal.
3. Cancel any in-progress migration workflows:
   ```
   workflow-cli cancel --workflow-id <MIGRATION_WORKFLOW_IDS>
   ```
4. Document the false-positive in `evidence/incidents/<INCIDENT_ID>/`.

---

## §F Post-Incident

1. Root-cause: what process gap allowed the regulatory interpretation error or pack logic error to ship?
2. Update multispectrum review `F13 (regulatory-compliance)` facet guidance to catch this class of error earlier.
3. Review whether similar issues exist in other compliance packs (scan `REGULATORY-MAPPING.md` files for the same misinterpreted article/section).
4. Schedule a `council-legal` + `council-privacy` quarterly pack review to proactively catch regulatory amendments before they become suspension-trigger events.

---

## §G References

- ADR-0251 §D-2 (Pack lifecycle — Authored → Signed → Published → Sunset → Tombstoned)
- ADR-0251 §D-3 (Tenant pack installation + uninstallation)
- ADR-0251 §D-8 (Breach notification workflow)
- ADR-0243 §D-10 (Hot-reload; fragment deactivation propagation)
- Synthesis §5.15 (F13 P1: EU NIS2 Article 23 three-stage cadence + EU DSA Article 24+28)
- `docs/runbooks/compliance-pack-revocation.md` (for active compromise vs. this grace-period procedure)
- `docs/runbooks/breach-notification-council-escalation.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
