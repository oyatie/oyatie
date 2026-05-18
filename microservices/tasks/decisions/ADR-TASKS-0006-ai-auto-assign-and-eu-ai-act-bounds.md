---
id: ADR-TASKS-0006
status: Accepted
date: 2026-05-17
microservice: tasks
deciders: axis-tasks, council-privacy, council-ai-safety, axis-foundry-runtime
owner: axis-tasks + council-privacy + council-ai-safety
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0117
  - ADR-0131
  - ADR-0132
  - ADR-0140
  - ADR-WS-0005
  - ADR-NET-0002
related_artifacts:
  - microservices/tasks/PRD.md (FR-20; AC-12 EU AI Act refusal; §"Audit + Compliance")
  - microservices/tasks/IP-014-ai-assist-bounds-and-eu-ai-act.md
  - microservices/tasks/capabilities/T0-suggest.yaml
  - microservices/tasks/capabilities/T1-categorise.yaml
  - microservices/tasks/capabilities/T2-auto.yaml
  - microservices/tasks/policy/auto-assign-employment-context.cedar
purpose: |
  Close the EU AI Act Annex III §4 + EEOC UGESP 1978 + KR 근로기준법 §17 +
  Title VII / ADA / UK Equality Act gap for T2 auto-assign. Establish the
  per-pack conformity-assessment-or-refuse contract.
---

# ADR-TASKS-0006: AI auto-assign + EU AI Act Annex III §4 bounds — T2 auto-assign in employment-context REFUSED at Cedar layer pending conformity-assessment ADR per pack

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-tasks §FR-20 declares three autonomy-tier AI capabilities:

- **T0 next-task suggest** — read-only; presents a recommendation to
  the user; the user must accept.
- **T1 auto-categorise + priority-suggest** — produces a suggestion
  that lands in the task UI; the user accepts/rejects.
- **T2 auto-assign** — DIRECTLY mutates the task's `assignees` field
  without prior user acceptance.

T2 in employment context is a regulated activity under multiple
overlapping jurisdictions:

| Jurisdiction | Citation | Effect |
|---|---|---|
| EU | EU AI Act Annex III §4 — employment, worker management, access to self-employment | High-risk AI system; conformity assessment required pre-deployment |
| EU | GDPR Art. 22 — automated individual decision-making | Right to human intervention; explanation; contest |
| US-federal | EEOC UGESP 1978 (Uniform Guidelines on Employee Selection Procedures) | 4/5ths rule for adverse-impact |
| US-federal | Title VII Civil Rights Act 1964 | Disparate-impact liability |
| US-federal | ADA 1990 | Accommodations + disability protection |
| KR | 근로기준법 §17 | Equal-treatment for working conditions |
| KR | 직장갑질 §76 | Workplace-harassment prevention |
| UK | Equality Act 2010 §13 + §19 | Direct + indirect discrimination |
| SG | Tripartite Guidelines on Fair Employment Practices (TAFEP) | Fair-hiring guidance |
| AU | Sex Discrimination Act 1984 + Age Discrimination Act 2004 | Adverse-impact protection |
| BR | LGPD Art. 20 | Automated decision-making rights |
| AE/KSA | UAE PDPL Art. 17; KSA PDPL Art. 16 | Automated processing safeguards |
| IN | DPDPA 2023 + EEO labour code | Equivalent protections |
| JP | APPI Art. 16-2 | Sensitive personal information; employment scrutiny |
| OECD | ISO 30414 §6.2.1 | Workforce-analytics governance |

Auto-assignment in the **employment context** (where the assignment
materially affects who does paid work, who gets opportunities, who
receives a performance signal) directly intersects with every entry
above. Per `feedback_no_silent_regression.md` + Linus-grade
no-defaults-that-violate-law doctrine: the default must REFUSE
auto-assign in employment context until the tenant has uploaded a
per-pack conformity assessment.

Sibling pattern: workflow-studio ADR-WS-0005 + network ADR-NET-0002
both implement the same EEOC-pattern refusal for their AI-driven
employment surfaces. ADR-TASKS-0006 aligns 1:1.

## Decision

The tasks µservice ships:

- **T0 + T1 are unconditionally allowed.** Both are advisory; user
  accepts/rejects in UI. Both still emit audit-chain seals with the
  capability ID.
- **T2 auto-assign in employment context is REFUSED at the Cedar
  policy layer** by default. Cedar policy
  `auto-assign-employment-context.cedar` evaluates the tenant's
  `employment_context` claim; if true AND the tenant has NOT uploaded
  a per-pack conformity-assessment artefact, refuse with
  `EUAIActAnnexIII::ConformityAssessmentMissing` 403.
- **Tenant uploads a per-pack conformity-assessment** as a sibling
  ADR-TASKS-XXXX (one per pack: ADR-TASKS-0007-pack-kr-conformity,
  ADR-TASKS-0008-pack-eu-conformity, etc.). Sign-off lands as an
  artefact at `microservices/tasks/legal/conformity-assessments/
  <pack>.pdf` plus an `evidence/...` record per ADR-0139.
- **T2 in non-employment context** (e.g., a tenant where every task
  is personal todos) is allowed once the tenant explicitly flips off
  the `employment_context` claim — refusal is the safe default.
- **Auto-assign-fairness lane** (`oya-governance-auto-assign-fairness`)
  runs cross-cohort assignment-rate delta analysis (4/5ths-rule
  approximation per UGESP); deviations > 20% across protected cohorts
  trigger the PrometheusRule alert
  `TasksAutoAssignFairnessAnomaly` wired in IP-001.
- **Audit-chain attribution**: every auto-assign emits a
  `TaskAssigned` event with `via_auto_assign=true` + `ai_capability_id`;
  Ed25519-signed per Bominal ADR-0028.
- **Right to human intervention** (GDPR Art. 22 §3): the affected
  task's `assignment_history` records the auto-assignment and exposes
  a one-click "request human review" path that opens a workflow-engine
  ticket.

## Alternatives Considered

### Alternative 1 — Ship T2 unconditionally; rely on tenant policy

- Pros:
  - Fastest time-to-market.
- Cons:
  - Default-violates-law in every employment-context tenant per Annex
    III §4 — the per-pack conformity assessment is mandatory pre-
    deployment.
  - Per `feedback_no_silent_regression.md` the safe default must be
    REFUSE.
- Rejected because: default-deny is non-negotiable for high-risk
  AI systems under Annex III.

### Alternative 2 — Refuse T2 universally (no conformity-assessment
escape hatch)

- Pros:
  - Maximum safety; simplest.
- Cons:
  - Customers in regulated jurisdictions who HAVE done the
    conformity assessment + DPIA cannot use the feature; competitive
    parity gap vs competitors that ship it on by default.
  - PRD §FR-20 mandates T2 ships as a "Should" — refusing universally
    fails that requirement.
- Rejected because: regulatory-fitness ≠ regulatory-refusal; the
  policy is "default-deny + tenant-upgradable-via-assessment".

### Alternative 3 — Refuse only in EU + UK; allow elsewhere

- Pros:
  - Smallest blast radius.
- Cons:
  - KR 근로기준법, EEOC UGESP, ADA, LGPD Art. 20, APPI Art. 16-2,
    UAE/KSA PDPL all carry equivalent or stricter protections.
  - Per-jurisdiction selective enforcement = per-jurisdiction
    cherry-picking = compliance failure across every jurisdiction
    that imposes equal-treatment requirements.
- Rejected because: the universal refusal pattern aligns with the
  Bominal ADR-WS-0005 + ADR-NET-0002 EEOC pattern; the legal
  citations are wider than EU+UK; selective enforcement is the
  highest-risk legal posture.

## Consequences

### Consequence 1 — T2 ships unusable by default; per-pack conformity ADRs land separately

T2 ships at M03 along with the rest of the µservice but is refused at
the Cedar layer. Each pack's tenant-facing T2 enablement requires a
sibling ADR (ADR-TASKS-0007 through ADR-TASKS-0017 for the 11 packs)
with the per-jurisdiction conformity assessment as the load-bearing
artefact. Open Question #3 in PRD-tasks tracks this.

### Consequence 2 — Auto-assign fairness lane becomes a release gate

The `oya-governance-auto-assign-fairness` lane runs against synthetic
cohort-balanced workloads; deviations > 20% block release per the
4/5ths rule approximation. The PrometheusRule wired in IP-001 fires
in production with the same threshold.

### Consequence 3 — GDPR Art. 22 §3 path materialises as a workflow

The "request human review" path opens a workflow-engine ticket per
ADR-TASKS-0005. The reviewer-agent role (Cedar `reviewer_role`)
materialises with a 5-business-day SLA; SLO-gated promotion (ADR-0139)
will not let the µservice pass dev without this path being green.

## References

- EU AI Act Annex III §4 — `eur-lex.europa.eu` (employment +
  worker management high-risk).
- GDPR Art. 22 §3 — `eur-lex.europa.eu` (right to human
  intervention).
- EEOC UGESP 1978 — `eeoc.gov/laws/regulations/29cfr1607.html`
  (4/5ths rule).
- Title VII Civil Rights Act 1964 — `eeoc.gov/statutes/title-vii-
  civil-rights-act-1964`.
- ADA 1990 — `ada.gov`.
- KR 근로기준법 §17 — `law.go.kr`.
- KR 직장갑질 §76 — `law.go.kr`.
- UK Equality Act 2010 — `legislation.gov.uk/ukpga/2010/15/contents`.
- SG TAFEP — `tafep.sg`.
- LGPD Art. 20 — `lgpd-brazil.info`.
- UAE PDPL Art. 17; KSA PDPL Art. 16.
- ISO 30414 §6.2.1 — `iso.org/standard/69338.html`.
- workflow-studio ADR-WS-0005 (sibling EEOC pattern).
- network ADR-NET-0002 (sibling EEOC pattern).
- Bominal ADR-0028 (Ed25519 + Merkle audit chain).
- PRD-tasks §FR-20 + AC-12.
