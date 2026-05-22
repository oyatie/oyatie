---
doc_class: Tutorial
microservice: itsm
related_adrs: [ADR-0316]
date: 2026-05-20
doc_status: published
---

# Tutorial — Configure ITIL v4 Change Enablement with CMDB impact analysis

Goal: build a Change Enablement workflow that uses the CMDB to auto-compute the blast-radius of a proposed change, routes to the appropriate approval class (standard / normal / emergency), and provides CAB members with structured impact data before they decide.

Prereqs: `itsm::admin` Cedar role, retired-standard tier or higher, CMDB already populated with ~ 1000 CIs minimum, ~ 90 min.

## Step 1 — define change types

Portal → Changes → "Configure types":

- **Standard Change**: pre-approved templates. Examples: "Apply security patch to non-production server", "Restart non-critical batch job". Auto-approved on submission.
- **Normal Change**: requires CAB review. Examples: "Upgrade production database to PostgreSQL 17", "Replace load balancer".
- **Emergency Change**: requires E-CAB review (subset of CAB, fast turnaround). Examples: "Patch CVE-2026-XXXX disclosed today".

For each type, configure: default approver group, SLA for approval, evidence requirements.

## Step 2 — author standard change templates

Portal → Changes → "Standard Change Templates" → "New template".

Example template "Apply OS patch to dev server":
- CI type allowed: Computer with `environment = dev` OR Server with `environment = dev`.
- Pre-conditions: backup verified within last 24 h; change window 22:00-06:00 local.
- Steps: documented runbook (link to KB article).
- Post-conditions: CI health check passes.
- Risk: Low.
- Auto-approval: yes.

Test: submit a change ticket matching this template; verify auto-approval fires within 60 s.

## Step 3 — configure CMDB impact analysis

Portal → Changes → "Impact analysis rules". The substrate uses CMDB CI relationships to compute blast-radius.

Configure rules:
- Direct impact: CIs the change touches directly.
- 1-hop impact: CIs depending on direct-impact CIs (via `depends_on` relationships).
- 2-hop impact: CIs 2 hops away.
- Service impact: BusinessService CIs whose `runs_on` chain includes any direct or 1-hop CI.
- Customer impact: estimated count of customers using affected BusinessServices.

Test: submit a change for a database CI. The impact analysis should report: direct = 1 CI (the DB); 1-hop = 8 CIs (apps depending on DB); 2-hop = 23 CIs (services depending on those apps); BusinessServices affected = 4; estimated customers = 1 200.

## Step 4 — author normal change workflow

Portal → Workflows → "New workflow" → "Normal Change with CAB review".

Stages:
1. **Draft** (requester drafts the change). Required fields: title, description, scheduled window, expected duration, CIs affected, rollback plan, post-change verification plan.
2. **Risk Assessment** (auto-computed by the substrate using CMDB impact analysis). Risk = function(direct CIs × criticality + 1-hop CIs × 0.5 + business services × 5).
3. **CAB Review** (CAB-members review weekly; can approve, reject, or request changes).
4. **Scheduled** (after approval; ticket transitions to scheduled state until the change window).
5. **In Progress** (change being executed).
6. **Verifying** (post-change checks running).
7. **Closed** (verification passed) or **Backed Out** (verification failed; rollback executed).

For each transition, configure: Cedar policy (who can transition), required evidence (e.g. "rollback plan must be ≥ 100 chars before submission"), notifications to send.

## Step 5 — schedule the CAB meeting

Portal → CAB → "Meeting cadence":
- Weekly Wednesday 14:00-15:00 UTC.
- Members: CTO + VP-Eng + 2 Senior SREs + Security Lead + tenant ITSM admin.
- Auto-populate agenda with all changes in "CAB Review" state.
- Auto-send pre-read 48 h before meeting (CAB members can pre-vote async).
- During meeting: each change discussed for 5 min max; approval requires 3-of-5 quorum.

## Step 6 — author the rollback policy

Every change must have a rollback plan. The substrate enforces this at draft submission. The rollback plan must include:
- Triggers: under what conditions to rollback (e.g. "error rate > 5x baseline for > 5 min after change").
- Procedure: step-by-step rollback steps.
- Verification: how to confirm rollback succeeded.
- Communications: who to notify.

The change cannot advance to "CAB Review" without a rollback plan; the CAB cannot approve without reviewing the rollback plan.

## Step 7 — emergency-change workflow

For emergencies (CVEs, customer outages), the standard 1-week CAB cycle is too slow.

Portal → Workflows → "Emergency Change":
- 1-hour SLA for E-CAB review (subset of CAB: CTO + Security Lead + on-call IC).
- Async approval via Slack/Discord/Telegram polls; 2-of-3 quorum.
- Post-change retrospective within 5 business days (mandatory).
- All emergency changes audit-chain-anchored with elevated retention (10 y instead of standard 7 y).

## Step 8 — integrate with DevOps (PRs as changes)

At retired-advanced tier, enable git integration:

```sh
oya itsm git-integration enable \
  --repo github.com/oyatie/oyatie \
  --branch dev \
  --auto-create-changes-for-paths "microservices/*/iac/*,microservices/*/iac/helm/*"
```

Now any PR touching infrastructure-as-code files auto-creates a Standard Change ticket linked to the PR. The PR cannot merge without the Change being approved + tests passing.

## Step 9 — reporting + metrics

Portal → Reports → "Change-management metrics":
- Changes/month by type (Standard / Normal / Emergency).
- Approval cycle time (submission → approved).
- Success rate (% of changes that complete without rollback).
- Failure rate (% requiring rollback or causing post-change incidents).
- E-CAB usage rate (should be low; high indicates change-management process bypassed).
- DORA metrics (lead time for changes, deployment frequency, change failure rate, time to restore service).

Schedule monthly to the leadership team.

## Step 10 — audit-chain evidence

```sh
oya audit-chain query --tenant <tenant-id> \
    --event-class "itsm::change::*" \
    --since "1 month ago" \
    --output json
```

Every change lifecycle event is anchored. This is your ITIL v4 + ISO 20000-1 + SOC 2 CC8.1 audit evidence.

## What you've built

A production ITIL v4 Change Enablement workflow with:
- Standard / Normal / Emergency change classes.
- CMDB-driven impact analysis.
- CAB review + approval workflow.
- Mandatory rollback plans.
- DevOps integration (PRs as Standard Changes at retired-advanced+).
- Cryptographic audit-trail.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| CMDB stale data → wrong impact analysis | Schedule discovery scans weekly; flag CIs not touched in 90 d for review |
| Standard change templates too broad → real risks slip through | Periodically audit standard changes that caused issues; tighten templates |
| E-CAB used for non-emergencies | Set per-quarter cap on E-CAB usage; require justification |
| CAB meetings without quorum | Set explicit member responsibilities + backup members |
| PR-as-change without rollback plan → DevOps cycle skips safety | Substrate refuses to link a PR-change without a rollback section |
