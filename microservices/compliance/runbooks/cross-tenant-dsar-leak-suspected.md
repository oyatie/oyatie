# Runbook — Cross-tenant DSAR leak suspected (Sev-1)

## Trigger

Any of:
- Kernel invariant assertion fired in production logs.
- CI integration test caught at PR build (block + flag).
- External report (auditor / customer / subject).

## Immediate actions (≤ 15 minutes)

1. **Acknowledge page** in PagerDuty.
2. **Suspend DSAR API**: scale `dsar-handler` deployment to 0 OR add feature flag `dsar.enabled=false`.
3. **Notify** axis-compliance + axis-security on-call lead via Slack #incident-current.
4. **Open incident** at `evidence/postmortems/INC-<date>-cross-tenant-dsar-leak.md`.

## Triage (≤ 1 hour)

1. Audit chain query: list DSAR responses produced in the last 24 hours; verify each `request.tenant_id == subject.tenant_id`.
2. Identify scope: which subjects, which tenants, which response payloads.
3. Quantify: count of distinct (subject, tenant) tuples impacted.

## Notification (≤ 72 hours, per GDPR Art. 33)

1. Notify affected tenants (template at `runbooks/communication-templates/tenant-breach.md`).
2. Notify affected subjects (template at `runbooks/communication-templates/subject-breach.md`).
3. Notify data-protection authority (per jurisdiction):
   - EU: relevant DPA.
   - KR: PIPC.
   - UAE: Data Office.
   - JP: PPC.

## Remediation

1. Reproduce locally; identify which guard layer failed.
2. Land fix + integration test exercising the failure mode.
3. Staged rollout with extra logging.
4. Re-enable DSAR API.

## Postmortem

Blameless postmortem within 5 business days. Required sections: timeline, detection lag, root cause, action items, owners.

## Cross-references

- threat-model.md A2 cross-tenant DSAR leak.
- IP-003 — DSAR pipeline.
- ADR-0209 — substrate authority.
