---
doc_class: IncidentResponse
title: Incident Response Playbook
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + axis-foundry
deciders: ops-sre-reliability, ops-security, axis-foundry, council-architecture
related_adrs: [ADR-0117, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/governance/failure-modes.md
  - microservices/governance/runbooks/lane-failure-triage.md
  - microservices/governance/runbooks/lane-bypass-emergency.md
review_cadence: quarterly + post-incident
doc_status: published
---

# Incident Response Playbook: governance µservice

## Purpose

Incident response procedures for governance µservice; aligns with Google SRE Workbook ch. 7 (managing incidents) + Atlassian Incident Management framework. Maps each failure mode to a response runbook + paging tier + post-incident posture.

## Severity Classification

| Severity | Description | Examples | RTO target | Paging | Auto-promote? |
|---|---|---|---|---|---|
| **Sev-1 CRITICAL** | Production-tier gate bypass; multi-µservice impact; audit-chain compromise | F-11 lane bypass; audit-chain tamper detected; mass false-positive halting all merges | ≤15 min detect + ≤1h mitigate | OnCall immediate page; council-architecture + ops-security + axis-foundry | yes (from Sev-2 if ETA missed) |
| **Sev-2 HIGH** | Single-µservice impact; one-axis conformance failure; audit-chain seal gap | F-02 false-positive blocking single µservice; F-03 seal gap; F-05 Postgres failover; F-07 GHA outage | ≤30 min detect + ≤2h mitigate | OnCall page within 5 min | yes (from Sev-3 if ETA missed) |
| **Sev-3 MEDIUM** | Performance degradation; cost overrun; queue depth | F-14 autoscaler stuck; cost-budget yellow alert; aggregation-indexer slowness | ≤2h detect + ≤4h mitigate | OnCall alert; non-paging | yes (from Sev-4 if recurring) |
| **Sev-4 LOW** | Single-PR transient | F-01 OOM single PR; F-08 baseline refresh transient | ≤1d detect + ≤72h mitigate | non-paging; ticket | no |

## Response Pyramid

```text
                    Incident Commander (IC)
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
   Operations          Communications      Subject Matter
    Lead (Ops)            Lead (Comms)          Lead (SME)
        │                   │                   │
   ┌────┴────┐          ┌──┴──┐            ┌────┴────┐
   │ Runbook │          │ Stk │            │ Lane    │
   │ Execn   │          │ hldr│            │ Author  │
   │         │          │ updt│            │ Or RCA  │
   └─────────┘          └─────┘            └─────────┘
```

### Roles + responsibilities

| Role | First-fill | Responsibility |
|---|---|---|
| Incident Commander (IC) | ops-sre-reliability on-call | Owns the incident timeline; calls roles; promotes severity; declares stand-down |
| Ops Lead | ops-sre-reliability on-call | Executes runbook steps; coordinates with infra |
| Comms Lead | ops-sre-reliability secondary; council-architecture for Sev-1 | Stakeholder updates; tenant notifications when applicable; status page |
| SME | axis-foundry / ops-security / council-privacy as applicable | Domain expertise; RCA owner |

## Detection Sources

| Source | Sev → Page | Latency |
|---|---|---|
| Grafana alert (Alertmanager → Grafana OnCall) | Sev-1/2 → page; Sev-3/4 → alert | ≤2 min |
| Audit-chain seal-verification daemon | Sev-1 | ≤5 min |
| Per-failure-mode synthetic probe | per mode | ≤5 min |
| GitHub Actions webhook outage | Sev-2 | ≤1 min (workflow failure) |
| External tenant escalation | varies | bounded by tenant report time |
| Quarterly external audit finding | bounded by audit cadence | n/a |

## Communication Posture

| Audience | Channel | Cadence during incident |
|---|---|---|
| Internal stakeholders (ops, eng leadership) | dedicated #incident-<id> Slack channel | every 30 min for Sev-1/2; every 2h for Sev-3 |
| External tenants (when impacted) | Application Shell status page + per-tenant email | first notice within 1h; updates every 2h |
| Public (when broadly visible) | public status page (public-read.cedar surfaces aggregate posture only) | first notice within 4h; updates every 4h |
| Regulators (when reportable; e.g., GDPR Art. 33 breach) | per-pack regulator notification per DPA | within 72h per GDPR Art. 33 |
| External auditors | post-incident addendum to next audit window | bounded by audit cycle |

## Per-Failure-Mode Response Routing

| Failure mode | Runbook | Sev (default) | Owner |
|---|---|---|---|
| F-01 lane runner OOM (single PR) | `lane-failure-triage.md` | Sev-4 | axis-foundry |
| F-02 false-positive blocks merge | `lane-failure-triage.md` | Sev-2 | axis-foundry |
| F-03 evidence emission gap | `evidence-replay.md` | Sev-2 | ops-security |
| F-04 aggregation-index corruption | `aggregation-rebuild.md` | Sev-3 | axis-foundry |
| F-05 Postgres failover | `migration-execution.md` | Sev-2 | ops-sre-reliability |
| F-06 S3 outage | inherited from `cloud-iac/runbooks/object-storage-outage.md` | Sev-2 | ops-sre-reliability |
| F-07 GitHub Actions outage | inherited; status posture review only | Sev-2 | ops-sre-reliability |
| F-08 baseline refresh fetch fails | `industry-baseline-refresh.md` | Sev-4 | council-architecture |
| F-09 lane registry corruption | `lane-failure-triage.md` §"registry corruption" | Sev-2 | axis-foundry |
| F-10 Cedar mis-deployment | inherited from `cloud-secrets/runbooks/cedar-rollback.md` | Sev-2 | ops-security |
| F-11 lane bypass (admin-merge) | `lane-bypass-emergency.md` | Sev-1 | ops-security |
| F-12 quarterly refresh softer baseline | `industry-baseline-refresh.md` | Sev-3 | council-architecture |
| F-13 self-application bootstrap paradox | `lane-failure-triage.md` §"self-lock" | Sev-2 | axis-foundry + council-architecture |
| F-14 autoscaler stuck | inherited; capacity-model review | Sev-3 | ops-sre-reliability |
| F-15 aggregation-indexer scope overrun | `aggregation-rebuild.md` §"scope overrun" | Sev-2 | axis-foundry + ops-security |

## Incident Lifecycle

```text
DETECT → TRIAGE → MITIGATE → STAND-DOWN → RCA → POSTMORTEM → CHANGE
   │        │         │            │         │         │            │
   ≤5min   ≤15min   per-RTO     IC-call    ≤1week  ≤2weeks    successor-IP IP
```

### 1. DETECT

Source = alert / probe / report (per "Detection Sources" above). IC acknowledges in `#incident-<id>` channel within 5 min.

### 2. TRIAGE

IC declares severity. Promotes roles. Confirms runbook. Communicates initial impact assessment.

### 3. MITIGATE

Ops Lead executes runbook. SME consults. Comms Lead updates stakeholders.

Pre-promotion check: if mitigation ETA exceeds the severity's RTO target → IC promotes severity one tier.

### 4. STAND-DOWN

When mitigation complete + stakeholders confirmed unblocked, IC declares stand-down. Auto-promotion thresholds reset.

### 5. RCA

SME drafts root-cause analysis within 1 week. Posted at `evidence/audits/postmortems/<incident-id>.md`. Format = Atlassian "5 Whys" + Google SRE blameless-postmortem template.

### 6. POSTMORTEM

Public (internal) postmortem review with council-architecture + ops-security + axis-foundry. Blameless; focuses on systemic gaps.

### 7. CHANGE

Follow-up IP filed at `microservices/governance/IP-INCIDENT-<id>-<slug>.md` for any structural change (runbook update, lane addition, capacity adjustment, rule-pack edit). ChangeSet contract per ADR-0110.

## Regulatory Notification Triggers

| Trigger | Authority | Deadline | Owner |
|---|---|---|---|
| Personal-data breach (GDPR Art. 33) | EU DPA + national DPAs | 72h | council-privacy |
| Personal-data breach (KR PIPA Art. 34) | KCC + PIPC | 5d (with subject notification within 7d) | council-privacy |
| Personal-data breach (HIPAA §164.404) | OCR | 60d (and tenant within 60d) | council-privacy + ops-compliance |
| Cybersecurity incident (NIS2 Annex I/II thresholds) | per-MS DPA | 24h initial + 72h successor-IP | ops-security |
| SOC 2 Type 2 material event | external auditor | per-engagement letter | ops-compliance |
| ISO 27001 nonconformity | certification body | per-certification process | ops-compliance |
| SLSA L3 attestation gap | OpenSSF | per-attestation cycle | axis-foundry |

## On-Call Rotation

| Tier | Rotation | Members | Coverage |
|---|---|---|---|
| Primary | weekly | ops-sre-reliability (3 engineers) | 24/7 |
| Secondary | weekly | ops-sre-reliability + axis-foundry | 24/7 |
| SME (security) | by escalation | ops-security (2 engineers) | business hours + on-call for Sev-1 |
| SME (privacy) | by escalation | council-privacy (1 engineer + DPO) | business hours + on-call for Sev-1 |
| Comms (Sev-1) | by escalation | council-architecture | business hours + on-call for Sev-1 |

On-call schedule managed in Grafana OnCall per `microservices/observability/iac/helm/oncall/values.yaml`.

## Tools + integrations

| Tool | Purpose |
|---|---|
| Grafana OnCall | Paging + rotation |
| Slack `#incident-<id>` | Incident war-room |
| Grafana dashboards | Real-time observation |
| GitHub Issues | Postmortem + successor-IP IP tracking |
| audit-chain CLI | Forensic seal verification |
| `oya-dev-cli governance` | Lane status + Finding query |

## Verification

- Quarterly tabletop incident exercise.
- Quarterly review of MTTR + MTTD per severity tier.
- Annual external review by ops-security + council-architecture.

## References

- `failure-modes.md` (15 failure modes).
- `runbooks/*.md` (6 runbooks at M01).
- `microservices/observability/incident-response.md` (shape reference).
- Google SRE Workbook ch. 7 (managing incidents); ch. 11 (capacity).
- Atlassian Incident Management — `atlassian.com/incident-management`.
- PagerDuty Incident Response — `response.pagerduty.com`.
- GDPR Art. 33; KR PIPA Art. 34; HIPAA §164.404; NIS2.
