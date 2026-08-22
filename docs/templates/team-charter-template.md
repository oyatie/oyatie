---
doc_status: published
---

# Team Charter: <team-id>

> **Author:** <name / agent>
> **Last reviewed:** YYYY-MM-DD by `council-architecture`
> **Status:** active / standing / tactical / retired
> **Companion:** [RACI-OWNERSHIP.md](../RACI-OWNERSHIP.md), per-team CODEOWNERS file in `.github/CODEOWNERS`.

## Mission

One paragraph: why this team exists, what it owns, what it does NOT own.

## Owned axes / surfaces / contracts

- **Axis(es):** <list — e.g. cloud, search, vertical-healthcare>
- **Per-product PRD(s):** <list of paths>
- **Cross-axis contracts owned/co-owned:** <list rows from DESIGN §10>
- **Catalog records (flat-crates target prefix):** <e.g. `crates/cloud-iam-*`>
- **Runbooks owned:** <list>
- **ADRs authored / co-authored:** <list from new pack 0001-0050>

## In-scope work

Bullet list of work types this team accepts.

## Out-of-scope (anti-scope)

Bullet list. Anti-scope is binding.

## Key dependencies on other teams

| Depends on | What we need | Cadence |
|---|---|---|

## Teams that depend on us

| Consumer | What they need | Cadence |
|---|---|---|

## Success metrics

Team-level OKRs / KPIs with org-level rollup. Cite the org-level metric in [PRD §4](../PRD.md) or per-product PRD §9.

## Escalation path

- **Internal:** tech lead → team manager
- **Cross-team:** [`council-architecture`](../teams/council-architecture/CHARTER.md)
- **Privacy / data-class:** [`council-privacy`](../teams/council-privacy/CHARTER.md)
- **Security:** [`ops-security`](../teams/ops-security/CHARTER.md)
- **Founder:** as last resort

## Communication cadence

- Stand-up: <pattern>
- Weekly: <pattern>
- Cross-team review: <pattern>
- Quarterly: <pattern>

## Bandwidth + hiring

- Current FTE: TBD
- Target FTE: <per [HIRING-CAPACITY-PLAN.md](../HIRING-CAPACITY-PLAN.md) per-wave>
- Open requisitions: link to hiring plan

## Operating norms

- Code review: per [`standards/code-review.md`](../standards/code-review.md)
- PR shape: 5-section H2 template per CLAUDE.md
- Pre-push: targeted Buck2/cloud-ci checks plus `presubmit` per [`../../templates/checklists/pre-push.md`](../../templates/checklists/pre-push.md)
- ADR proposal cadence: monthly batch
- Postmortem authorship: per [`templates/incident-postmortem-template.md`](incident-postmortem-template.md)

## Slice of risk register

Team-level slice of [RISK-REGISTER.md](../RISK-REGISTER.md):

| Risk | Severity | Mitigation | Owner sub-role |
|---|---|---|---|

## Sources scanned

- [PRD.md](../PRD.md)
- [DESIGN.md §10](../DESIGN.md)
- per-product PRD(s) at [`products/`](../products/)
- ADR pack at [`docs/decisions/`](../../../docs/decisions/)
- per-team area contributing to this charter

*Footer regenerated whenever this charter is amended.*
