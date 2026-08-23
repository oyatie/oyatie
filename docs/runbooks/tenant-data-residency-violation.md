---
purpose: Oyatie Runbook — Tenant Data Residency Violation Detection and Remediation
doc_status: published
---

# Oyatie Runbook — Tenant Data Residency Violation Detection and Remediation

> **Status:** Active
> **Owner:** ops-compliance + council-privacy + ops-security
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0240, ADR-0244 §D-3, ADR-0251 §D-8, ADR-0248 §D-6, ADR-0049

---

## §A Trigger Conditions

Initiate this runbook when tenant data is detected or suspected to have egressed a cell or region not in the tenant's declared `data_residency_allowed` set (per ADR-0244 §D-3).

Specific triggers:

- **Cell perimeter egress alert** — Cilium network-policy audit log reports data-class-tagged traffic crossing a cell boundary to a cell in a non-allowed region (ADR-0248 §D-6 hot-path rule violation).
- **Audit-chain cross-cell-coordination event to disallowed region** — `CrossCellCoordinationEvent` in audit stream references a destination cell in a jurisdiction not in `data_residency_allowed`.
- **Tenant or regulator complaint** — tenant reports their data appeared in an unexpected region; a regulator requests an explanation of data flows.
- **Cross-border replication alert** — ADR-0049 replication health monitor detects data-class-tagged rows being replicated to a region not in `data_residency_allowed`.
- **Sovereignty audit finding** — `presubmit` (retired CLI `gate validate tenant-pack-cell-pinning`) CI lane reports a tenant is placed in a cell outside their pack's `cell_eligibility.minimum_certification_level_set`.

---

## §B Pre-Checks

Estimated time: **10–20 min**.

1. **Identify the affected tenant(s) and their declared residency:**
   ```
   psql -c "SELECT tenant_id, data_residency_allowed, home_cell, dr_cell,
     read_replica_cells, primary_jurisdiction
     FROM tenants WHERE tenant_id = '<TENANT_ID>';"
   ```

2. **Identify the egress event.** Pull the specific audit record(s) that triggered the alert:
   ```
   audit-chain-cli query \
     --tenant-id <TENANT_ID> \
     --event-class "CrossCellCoordinationEvent,DataEgressAuditEvent" \
     --window-start "<ALERT_TIMESTAMP - 1h>" \
     --window-end "<ALERT_TIMESTAMP + 1h>" \
     --output /tmp/egress-events-<INCIDENT_ID>.json
   ```

3. **Determine the data classes involved:**
   ```
   jq '.events[] | {event_type, destination_cell, data_class_tags, tenant_id, timestamp}' \
     /tmp/egress-events-<INCIDENT_ID>.json
   ```

4. **Identify the destination cell's region and jurisdiction:**
   ```
   psql -c "SELECT cell_id, region, jurisdiction_code, certification_levels
     FROM cells WHERE cell_id = '<DESTINATION_CELL_ID>';"
   ```

5. **Assess regulatory scope.** Determine which regulations apply based on the tenant's compliance packs and data classes:
   ```
   psql -c "SELECT tcp.pack_id, cp.regulation->>'jurisdiction' as reg_jurisdiction,
     cp.breach_notification_workflow->>'regulator_notification_deadline_hours' as notif_deadline_h
     FROM tenant_compliance_packs tcp
     JOIN compliance_packs cp ON tcp.pack_id = cp.pack_id AND tcp.version = cp.version
     WHERE tcp.tenant_id = '<TENANT_ID>' AND tcp.status = 'ACTIVE';"
   ```

   Key jurisdictions with strict residency obligations:
   - **EU GDPR**: cross-border transfer outside EEA requires adequacy decision or SCCs.
   - **KR PIPA**: personal information must not leave Korea without explicit consent + notification.
   - **China PIPL**: personal information export requires CAC security assessment for large transfers.
   - **KSA PDPL**: personal data may not leave KSA without NDMO-approved transfer mechanism.

6. **Declare incident.** Severity based on data class and regulatory exposure:
   - SEV-1: classified data (DoD IL5/6) or large-volume PII egress to non-adequate jurisdiction.
   - SEV-2: regulated PII/PHI egress with known notification deadline.
   - SEV-3: non-PII technical data or egress within adequate jurisdictions.

---

## §C Procedure

### Step 1 — Automatic quarantine: block further egress (target: ≤2 min)

Install an emergency Cedar fragment to block all cross-cell data movement for the affected tenant until the scope is determined:

```
cat > /tmp/tenant-egress-quarantine-<TENANT_ID>.cedar << 'EOF'
// EMERGENCY: data-residency quarantine
forbid (
  principal in Tenant::"<TENANT_ID>",
  action in [
    Data::Action::CrossCellReplication,
    Data::Action::CrossCellDataShare,
    Data::Action::CrossBorderTransfer
  ],
  resource
)
when { context.destination_jurisdiction not in ["<ALLOWED_JURISDICTIONS>"] };
EOF

policy-engine-cli fragment publish \
  --fragment-path /tmp/tenant-egress-quarantine-<TENANT_ID>.cedar \
  --scope "tenant/<TENANT_ID>/egress-quarantine" \
  --ttl-seconds 86400 \
  --operator oyatie.ops-compliance.<operator-id>
```

Wait for propagation (≤30s per ADR-0243 §D-10).

### Step 2 — Audit-chain analysis to determine scope (target: ≤60 min)

Run a comprehensive replay to determine the full extent of data that may have egressed:

```
audit-chain-cli data-residency-scope \
  --tenant-id <TENANT_ID> \
  --disallowed-jurisdictions "<DISALLOWED_JURISDICTION>" \
  --window-start "<EARLIEST_POSSIBLE_EGRESS>" \
  --window-end "<NOW>" \
  --include-data-classes \
  --output /tmp/residency-scope-<INCIDENT_ID>.json
```

Extract key metrics:
```
jq '{
  total_egress_events: (.events | length),
  data_classes_involved: [.events[].data_class_tags] | flatten | unique,
  affected_records_estimate: .total_record_estimate,
  first_egress_at: (.events | map(.timestamp) | min),
  destination_cells: [.events[].destination_cell] | unique
}' /tmp/residency-scope-<INCIDENT_ID>.json
```

### Step 3 — Root-cause analysis to ADR-0240 pack policies

Determine why the violation occurred by tracing to the specific system failure:

```
# Check if the tenant's cell assignment violates pack pinning (ADR-0251 §D-5):
presubmit (retired CLI gate validate) tenant-pack-cell-pinning --tenant-id <TENANT_ID>

# Check if cross-region replication was misconfigured (ADR-0049):
psql -c "SELECT replication_targets, data_class_filter
  FROM replication_configurations WHERE tenant_id = '<TENANT_ID>';"

# Check if a Cedar fragment permitted a cross-border transfer it should not have:
audit-chain-cli cedar-replay \
  --tenant-id <TENANT_ID> \
  --action "Data::Action::CrossBorderTransfer" \
  --window-start "<FIRST_EGRESS_AT>" \
  --window-end "<NOW>" \
  --output /tmp/cedar-replay-residency-<INCIDENT_ID>.json
```

Common root causes and remediation paths:
| Root cause | Remediation |
|---|---|
| Tenant placed in wrong cell (pack-pinning violation) | Execute `docs/runbooks/cell-evacuation.md` to migrate tenant to a compliant cell |
| Cross-region replication misconfigured | Update `replication_configurations.replication_targets` to exclude non-allowed regions |
| Cedar fragment incorrectly permitted cross-border transfer | Execute `docs/runbooks/cedar-fragment-emergency-rollback.md` |
| ADR-0240 sovereign-cloud-pack policy gap | File a pack policy update; escalate to `council-architecture` |

### Step 4 — Remediate the root cause

Execute the appropriate sub-runbook from the root-cause table above. Most common case (cell placement violation):

```
# Migrate tenant to a compliant cell:
tenancy cell-migration-apply \
  --tenant-id <TENANT_ID> \
  --target-cell <COMPLIANT_CELL_ID> \
  --scope home_cell \
  --traffic-drain-seconds 30 \
  --confirm
```

For replication misconfiguration:
```
psql -c "UPDATE replication_configurations
  SET replication_targets = array_remove(replication_targets, '<NON_ALLOWED_REGION>'),
      updated_at = now(),
      update_reason = 'data-residency-violation-remediation'
  WHERE tenant_id = '<TENANT_ID>';"
```

### Step 5 — Regulator notification (jurisdiction-dependent, time-bound)

Based on the scope assessment (Step 2) and applicable regulations (§B pre-check 5), determine notification obligations:

**EU GDPR (Art. 33/34):** If personal data was transferred outside EEA without adequate mechanism:
- 72h regulator notification deadline (DPA in lead supervisory authority jurisdiction).
- Subject notification if high risk to rights and freedoms.
- Trigger: `workflow-cli start oyatie.foundry.breach-notification --pack-id EU-GDPR-2018-baseline --trigger data-residency-violation`

**KR PIPA (Art. 34/62):**
- 24h notification to PIPC (Personal Information Protection Commission).
- Subject notification if likely to cause harm.
- For health data: notify Ministry of Health and Welfare.

**China PIPL (Art. 38/40):**
- Large-scale cross-border transfers require CAC security assessment (this may need immediate legal engagement if triggered).
- Notify affected data subjects.

**KSA PDPL:**
- Notify NDMO and data subjects per the PDPL breach framework.

```
workflow-cli start oyatie.foundry.breach-notification \
  --tenant-id <TENANT_ID> \
  --trigger data-residency-violation \
  --affected-jurisdictions <JURISDICTIONS> \
  --incident-ref <INCIDENT_ID>
```

Escalate to `docs/runbooks/breach-notification-council-escalation.md` for regulator communication drafting.

### Step 6 — Tenant communication

Notify the affected tenant of:
- What data egressed (data classes, estimated record count).
- The destination jurisdiction.
- Root cause.
- Remediation taken.
- Whether regulatory notification has been or will be filed.
- Timeline for restoration of normal service.

Send via the tenant's designated legal/privacy contact (not the standard notification channel, given potential regulatory sensitivity).

### Step 7 — Remove egress quarantine

Once the root cause is remediated and verified:

```
policy-engine-cli fragment deactivate \
  --scope "tenant/<TENANT_ID>/egress-quarantine" \
  --operator oyatie.ops-compliance.<operator-id>
```

Verify cross-cell operations resume normally within `data_residency_allowed` bounds:
```
audit-chain-cli query \
  --tenant-id <TENANT_ID> \
  --event-class "CrossCellCoordinationEvent" \
  --window-start "now - 10m"
```

---

## §D Verification

1. **No further egress to disallowed jurisdiction:**
   ```
   audit-chain-cli data-residency-check \
     --tenant-id <TENANT_ID> \
     --window-start "<QUARANTINE_ACTIVATED_AT>" \
     --disallowed-jurisdictions "<DISALLOWED_JURISDICTION>"
   ```
   Must return `0 violations`.

2. **Tenant cell placement complies with pack pinning:**
   ```
   presubmit (retired CLI gate validate) tenant-pack-cell-pinning --tenant-id <TENANT_ID>
   ```

3. **Replication configuration excludes non-allowed regions.**

4. **Regulator notification filed** (where required) within the applicable deadline.

5. **Audit trail complete:** `DataResidencyViolationDetected`, `EgressQuarantineActivated`, `DataResidencyViolationRemediated` events present with Merkle proofs.

---

## §E Rollback

The egress quarantine is a protective measure and should not be rolled back until the root cause is remediated (Step 4). If the quarantine is causing unacceptable service impact for the tenant before the root cause is fixed:

1. Narrow the quarantine to block only the specific data classes involved (not all cross-cell operations).
2. Obtain `council-legal` approval before partially lifting the quarantine if data classes are regulated.

---

## §F Post-Incident

1. Root-cause documentation filed in `evidence/incidents/<INCIDENT_ID>/`.
2. Update the tenant's `data_residency_allowed` declaration if the violation revealed it was misconfigured (vs. a system bug).
3. File MFL row if the violation was caused by a platform system bug (cedar fragment, replication config, cell assignment service).
4. Assess whether the `presubmit` (retired CLI `gate validate tenant-pack-cell-pinning`) CI lane would have caught this earlier — if not, improve the lane.
5. If the violation involves EU GDPR Article 46 mechanisms (SCCs, etc.), ensure agreement lifecycle is updated.
6. Post-mortem within 72h.

---

## §G References

- ADR-0244 §D-3 (`data_residency_allowed` on tenants)
- ADR-0240 (Sovereign cloud per regional pack)
- ADR-0251 §D-5 (Tenant-pack cell pinning)
- ADR-0251 §D-8 (Breach notification workflow)
- ADR-0248 §D-6 (Per-cell vs cross-cell bright line)
- ADR-0049 (Cross-region replication)
- ADR-0243 §D-10 (Hot-reload for quarantine fragments)
- `docs/runbooks/breach-notification-council-escalation.md`
- `docs/runbooks/cell-evacuation.md`
- `docs/runbooks/cedar-fragment-emergency-rollback.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
