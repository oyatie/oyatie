---
doc_class: Runbook
title: Profile-export vCard 4.0 / JSON Resume corruption
microservice: network
severity: "Sev-2 (GDPR Art. 20 portability obligation degraded)"
status: Accepted
owner_team: axis-network + council-privacy
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/network/failure-modes.md (FM-23)
  - microservices/network/decisions/ADR-NET-0006-profile-portability-and-export.md
  - microservices/network/policy/data-residency.md (§DSR cascade — Art. 20)
doc_status: published
---

# Runbook: Profile-export vCard 4.0 / JSON Resume corruption (FM-23)

## Trigger

- `network_profile_export_corruption_total` > 0 OR
- User-reported corrupt download (tenant support ticket) OR
- Auto-detector finds vCard schema-validation failure or JSON Resume schema drift OR
- DSR cascade audit reveals PII redactor over-applied (sensitive fields stripped incorrectly).

## Severity

Sev-2 by default — GDPR Art. 20 portability obligation is degraded; tenants cannot rely on export integrity for regulatory submissions.

Escalate to Sev-1 if:
- The corruption is silent (downloads appear valid but contain wrong data; cross-user data mixed).
- A tenant has cited the corruption in a regulator submission.

## Immediate Mitigation (≤ 1 h)

| Step | Action | Time |
|---|---|---|
| 1 | Identify the corrupt emitter path: vCard 4.0 emitter, JSON Resume emitter, or GDPR Art. 20 portable-JSON emitter | ≤ 5 min |
| 2 | Disable the corrupt path at REST handler: return 503 + JSON `{error: "Export temporarily unavailable; engineering investigating"}` | ≤ 5 min |
| 3 | Revert to last-known-good emitter version per release-pointer revert | ≤ 10 min |
| 4 | Regenerate affected exports server-side for any user who downloaded in the corrupt window | up to 30 min depending on count |
| 5 | Notify affected users via in-app notification + email: "Your recent profile export may have been incomplete; a corrected copy is attached" | ≤ 30 min |
| 6 | Audit-chain seal of the corruption event + remediation | ≤ 1 min |
| 7 | Identify root cause: schema drift, PII redactor regression, character-encoding bug | ≤ 30 min |

## Common Root Causes

### Schema drift (vCard 4.0 or JSON Resume)

- vCard 4.0 RFC 6350 properties added/removed in upstream library; emitter not aligned.
- JSON Resume open schema version bump; emitter producing v1.x.x while consumer expects v0.x.x or vice-versa.

Mitigation: pin vCard emitter library version; pin JSON Resume schema version; surface schema-version in export metadata; regenerate impacted exports.

### PII redactor over-applies

- Redactor over-zealous: strips legitimate fields (employment record entries, certifications).
- Cause: pack-overlay redactor pattern matches too broadly (e.g., pack-us-healthcare PHI redactor matches non-PHI employment text).

Mitigation: adjust pack-overlay redactor regex; test against golden-set; regenerate impacted exports.

### Character-encoding bug

- UTF-8 → ISO-8859-1 / Windows-1252 mistranslation; CJK / RTL characters mangled.
- Cause: vCard 4.0 emitter using legacy ENCODING; or non-UTF-8 file-write path.

Mitigation: enforce UTF-8 end-to-end; emit BOM in vCard if downstream parsers expect it; regenerate impacted exports.

### Cross-user data leak in batch export

- Critical: batch export accidentally mixes two users' data.
- Cause: shared state in emitter; concurrent batch invocation; missing tenant_id GUC on Postgres read.

Mitigation: **Sev-1 escalation**; engage ops-security + council-privacy; treat as data-breach; follow GDPR Art. 33 clock; per pack regulator notification.

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Schema drift | recent dep bump on vCard / JSON Resume library; CI lane previously green | rollback dep; pin version; regenerate |
| Redactor regression | pack-overlay redactor recently changed; affects pack-us / pack-us-healthcare | review redactor regex; test golden-set; regenerate |
| Encoding bug | UTF-8 / locale-related; affects CJK / RTL tenants disproportionately | fix encoding path; regenerate |
| Concurrency bug | export latency spike + correlation with batch invocations | engage axis-network for code-fix; regenerate; consider Sev-1 if cross-user mix |

## Recovery Verification

- vCard 4.0 emitter output validates against RFC 6350 reference parser (e.g., `vobject` library) for golden-set inputs.
- JSON Resume emitter output validates against schema `https://jsonresume.org/schema/`.
- GDPR Art. 20 portable-JSON emitter output validates against internal schema; signed-URL list resolves correctly.
- `network_profile_export_corruption_total` rate at 0 for ≥ 24h.
- Tenant-support ticket queue cleared.

## Tenant Communication Template

```
Subject: Profile export issue resolved — corrected copy available

We identified an issue affecting profile exports requested between <start> and <end>.
The issue caused the following symptom: <symptom>.

We have:
1. Disabled the affected export path and rolled back to a known-good version.
2. Regenerated your profile export server-side. A corrected copy is available at <url>.
3. Sealed an audit-chain record of this incident.

If you previously submitted the affected export to a regulator or third party,
please contact us at <DPO-contact> so we can supply a corrected copy + a signed
attestation of the correction.

Affected window: <start> to <end>.
Resolution timestamp: <ts>.
```

## Postmortem Triggers

- Recurring schema drift: tighten CI lane `oya-gate validate profile-export-schema-conformance`.
- Cross-user mix: Sev-1 postmortem; council-privacy sign-off; possibly regulator notification.
- Pack-overlay redactor regression: review pack-overlay test coverage.

## Drill Pattern

Quarterly profile-export drill:

1. Synthetic golden-set of 1000 diverse profiles (CJK names, RTL, edge-cases, minor accounts, pack-us-healthcare PHI overlay).
2. Run all three exporters end-to-end.
3. Validate against RFC 6350 + JSON Resume schema + internal GDPR Art. 20 schema.
4. Verify no cross-user data leak (each export contains only the requested user).
5. Verify pack-overlay redaction is correct (PHI stripped where appropriate; legitimate employment record preserved).

## References

- `microservices/network/failure-modes.md` FM-23.
- `microservices/network/decisions/ADR-NET-0006-profile-portability-and-export.md`.
- `microservices/network/policy/data-residency.md` §DSR Art. 20.
- vCard 4.0 RFC 6350.
- JSON Resume open schema `jsonresume.org/schema/`.
- GDPR Art. 20 (right to data portability).
- DPDPA 2023 (India) portability obligations.
