---
doc_class: Runbook
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0028
  - ADR-0243
  - ADR-0263
  - ADR-0276
companion_docs:
  - console/runbooks/incident-command.md
  - microservices/ops-dashboard-control-center/compliance.md
  - console/runbooks/tenant-scope-violation-detected.md
  - console/runbooks/step-up-auth-bypass-attempt.md
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Runbook: Forensic Investigation Handoff

## A — Trigger conditions

- Suspected or confirmed insider threat (operator data exfiltration, privilege escalation, audit tampering).
- Confirmed tenant scope violation that succeeded (action_succeeded = true).
- Council-security requests forensic evidence for legal proceedings.
- Regulator demands evidence under GDPR Art. 15 DSAR, SOX 802, or equivalent.
- SEV0 incident where root cause is suspected malicious actor.
- Law enforcement request with valid warrant.

## B — Pre-checks

1. **[≤5min]** Identify the scope: which operator, which tenants, which time window.
2. **[≤5min]** Confirm `oyatie.ops.forensics` principal access (requires council-security authorization).
3. **[≤5min]** Determine legal basis for evidence collection (internal investigation vs regulator vs law enforcement).
4. **[≤5min]** Verify audit chain integrity BEFORE any remediation steps (remediation can add events; original evidence must be preserved first).

## C — Procedure

### Step 1 — Preserve audit chain (MUST be first)

1. **[≤10min]** Export sealed audit chain for the investigation scope:
   ```
   POST /ops/v1/audit/evidence-export
   Headers: X-Step-Up-Token: <T3_forensics_token>
   Body: {
     "principal_ids": ["<operator_id>"],
     "tenant_ids": ["<tenant_id>"],
     "time_range": { "from": "<ISO8601>", "to": "<ISO8601>" },
     "export_format": "signed-jsonl-zstd",
     "chain_of_custody_reason": "<investigation_id>"
   }
   ```
   Expected: `202 Accepted` with `export_ticket_id`.

2. **[≤15min]** Wait for export to complete: `GET /ops/v1/audit/evidence-export/{ticket_id}` → poll until `state: COMPLETED`.

3. **[≤5min]** Verify export integrity: `sha256sum <export_file>` matches `export_manifest.sha256`. Merkle root matches ADR-0028 chain seal.

4. **[≤5min]** Store export in forensics-grade storage: `${openbao:secret/oyatie/forensics/<investigation_id>/evidence-pack}`. Access requires `oyatie.ops.forensics` principal.

### Step 2 — Session recordings (T3 sessions only)

1. List T3 session recordings for the operator in scope:
   ```
   GET /ops/v1/session-recordings?operator_id={id}&window={from}/{to}
   ```
2. Export recordings (requires `oyatie.ops.forensics`):
   ```
   POST /ops/v1/session-recordings/export
   Body: { "session_ids": [...], "investigation_id": "<id>" }
   ```
3. Chain-of-custody: every export action emits `ForensicEvidenceExported` audit event (sealed).

### Step 3 — UEBA signal export

1. `GET /ops/v1/detection/ueba/{operator_id}/history?window={from}/{to}` → UEBA signal timeline.
2. Include in evidence pack alongside audit chain.

### Step 4 — Hand off to council-security / legal

1. Prepare evidence summary: operator timeline, Cedar verdicts, UEBA score trajectory, session recordings list.
2. Transmit evidence pack via secure channel (NOT email unless encrypted + signed).
3. Evidence pack hash + chain-of-custody: `ForensicInvestigationHandoffCompleted` audit event.
4. Incident ticket updated with `forensic_evidence_pack_id` and `investigation_status: HANDED_OFF`.

### Step 5 — Regulatory / law enforcement handoff

1. Requires explicit sign-off from General Counsel + council-security.
2. Warrant review: `GET /ops/v1/legal/warrant-review/{warrant_id}` — confirm scope matches investigation.
3. Export scoped to warrant scope only (principle of minimum disclosure).
4. Transparency report: per-pack warrant-canary update within 90d if legally permitted.

## D — Verification

- Evidence pack `sha256` matches chain — no tampering.
- `ForensicEvidenceExported` + `ForensicInvestigationHandoffCompleted` events in audit chain, both sealed.
- Evidence pack stored in forensics-grade storage with access-controlled principal.

## E — Rollback

Evidence preservation is append-only (no rollback). If export scope was too broad: delete the over-scoped export from forensics storage and re-export with correct scope. Both actions audit-emitted.

## F — Post-incident

- Post-mortem: identify control failure that enabled the incident.
- Remediation: Cedar policy fix, UEBA threshold adjustment, or operator re-training.
- Legal hold: preserve audit chain for duration of any legal proceeding (may exceed standard 7yr retention).

## G — References

- `ARCHITECTURE.md §abuse-defence`
- `compliance.md §insider-threat-controls`
- `policy/cedar/audit-emission-required.cedar`
- `runbooks/tenant-scope-violation-detected.md`
- `runbooks/step-up-auth-bypass-attempt.md`
- ADR-0028 (audit chain seal)
- ADR-0263 (audit emission contract)
- ADR-0276 (backup portability)
