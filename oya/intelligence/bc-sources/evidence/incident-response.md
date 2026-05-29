---
doc_class: IncidentResponse
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + council-privacy + axis-foundry-evidence
related_artifacts:
  - microservices/intelligence-evidence/runbooks/
  - microservices/intelligence-evidence/failure-modes.md
  - microservices/intelligence-evidence/threat-model.md
doc_status: published
---

# foundry-evidence — incident response

## Roles

| Role | Person/team | Responsibility |
|---|---|---|
| IC (Incident Commander) | ops-sre-reliability on-call (or council-privacy for compliance-class incidents) | Single decision-maker; runs incident |
| Scribe | rotating | Real-time log; transcripts retained in `evidence/incidents/foundry-evidence/<inc-id>/` |
| Subject Matter Expert (SME) | axis-foundry-evidence on-call | Domain expertise on pack-builder / aggregator / bridge |
| Substrate SME | axis-audit-chain on-call | When substrate is implicated |
| Communications lead | council-privacy chair (for Sev-1 with regulator implications) or ops-product (for tenant comms) | External + internal comms |
| Legal counsel | legal-counsel rotation | For breach-class events |

## Severity matrix

| Severity | Trigger | Response | Notification |
|---|---|---|---|
| Sev-1 | substrate-integrity event (FM-09 blob corruption with WORM intact; suspected tamper); audit-chain backlog > 10 min sustained; regulator-export delivered defective (FM-07/08 with regulator already in receipt); 2-person-rule violation detected | IC + Comms + Legal engaged within 15 min; ExecSponsor notified | Tenant + regulator (per scope) within 4 h; postmortem within 5 business days; ExecSponsor + Board for substrate-integrity |
| Sev-2 | pack-assembly failure rate > 0.01 sustained; audit-chain backlog 60s–10min; regulator-export pre-delivery defect; archive cascade lag > 36 h | IC engaged within 30 min | Tenant DPA-bound notification if affected; postmortem within 10 business days |
| Sev-3 | partial-pack rate elevated but bounded; capacity headroom alert; late-signal spike below materiality threshold | Engineering rotation handles in-hours | Internal-only; postmortem at discretion |

## Detection sources

| Source | Triggers what |
|---|---|
| Mimir alerts on foundry-evidence SLI | Sev-2 / Sev-3 |
| Substrate (audit-chain) Sev-1 escalation | Sev-1 (auto-join) |
| CI lane red on `regulator-profile-drill` | Sev-2 |
| Regulator-reported defect in delivered bundle | Sev-1 |
| Tenant support ticket | varies; triage by support → on-call |
| Internal QA on bundle pre-delivery | Sev-2 |

## On-call rotation

- ops-sre-reliability primary on-call (24x7) is the default IC contact.
- axis-foundry-evidence rotation supplies SME.
- council-privacy chair handles compliance-class incidents.

## Lifecycle

1. **Detection** → page + Slack `#inc-<id>` opened.
2. **Triage** → IC declares severity; engages roles.
3. **Stabilise** → halt downstream consumers (e.g., pause regulator-exports if pack integrity in doubt); engage relevant runbook.
4. **Recover** → execute runbook; verify recovery.
5. **Communicate** → tenants + regulators (Sev-1/2 per scope) + internal stakeholders.
6. **Postmortem** → blameless; 5 business days for Sev-1, 10 for Sev-2.
7. **Follow-up** → CI lane added if class was previously undetected; runbook updates; ADR if substantive policy change.

## Tenant notification template (Sev-1, evidence-class)

```
Subject: [Sev-1] foundry-evidence service incident — tenant <tenant_id>

What happened:
<one-sentence factual description>

What is affected:
- Evidence-pack records for invocations between <start_ts> and <end_ts>.
- <specifics>

What we are doing:
- IC engaged.
- Affected packs identified and quarantined (if applicable).
- Substrate (audit-chain) cooperation engaged (if applicable).

What you should do:
- <action items, e.g., "no action required; we will issue corrected pack bundle">

We will provide an update by <ts + 4h>.

oyatie incident desk — incident #<inc-id>
```

## Regulator notification template (Sev-1, bundle defect already delivered)

Authored by council-privacy chair + legal-counsel; delivered via established engagement channel; references the engagement_id + previously-delivered bundle_id.

## Forensic preservation

Per `microservices/intelligence-evidence/policy/evidence-pack-integrity.md` EPI-11 + EPI-13:

- All artifacts of an incident go into `evidence/incidents/foundry-evidence/<inc-id>/`.
- Slack transcripts.
- Postgres snapshots (of relevant rows).
- Audit-chain inclusion proofs.
- Cedar policy fingerprints at time of incident.
- Helm + Kustomize + SPIFFE state at time of incident.
- All on-chain audit-emitted events related to the incident.

The directory is itself audit-emitted on creation (`incident.<id>.declared`) and on close (`incident.<id>.closed`).

## Postmortem cadence

- 5 business days for Sev-1.
- 10 business days for Sev-2.
- Optional for Sev-3.

Postmortem includes:
- Timeline.
- What happened.
- What worked / what didn't.
- Action items (each tied to an issue + owner).
- Customer impact assessment.
- CI lane gap analysis.
- Runbook updates.

## Drill cadence

- Monthly: pack-assembly-fail drill (simulate FM-04 + FM-05).
- Monthly: audit-chain-backlog drill (simulate FM-02/03).
- Quarterly: regulator-export-reissue drill.
- Semi-annual: blob-storage-restore drill (table-top + live for FM-09c access-path).
- Annual: full-pack catastrophic failure table-top.

## ADR-0133 honesty annotation

The incident-response process is itself the subject of CI assertion. `hyperscaler-maturity-claims` lane requires that every runbook listed in `runbooks/` has a drill cadence and a passed drill within the last drill window. Untested runbooks are a claim violation.

## References

- `runbooks/` (all runbooks).
- `failure-modes.md` (FM-01..FM-12).
- `threat-model.md`.
- ADR-0133 (honest claims).
