---
purpose: Oyatie — Prevention Doctrine
doc_status: published
---

# Oyatie — Prevention Doctrine

> **Owner:** `council-architecture`. The doctrine for how Oyatie responds to mistakes: fix the SYSTEM, not the symptom.
> **Companion:** [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md), [`MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md), [CHANGELOG.md](../CHANGELOG.md), [`templates/incident-postmortem-template.md`](../templates/incident-postmortem-template.md).

---

## 1. The doctrine

When work goes wrong — bug shipped, incident fired, contract drift, audit miss, brand residue — the response is not "fix the immediate symptom." The response is to **fix the system that allowed the failure**.

Concretely: every mistake that surfaces produces an **authoritative mechanical prevention** (blocking CI gate / fitness function / runtime check / config-as-code) that makes the failure mode structurally impossible. An optional local hook may provide earlier feedback, but it cannot satisfy this requirement without the CI/runtime backstop. The prevention is checked into the repo; the per-incident postmortem cites it; the [`MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md) records it.

## 2. Process-only fixes are anti-pattern

| Anti-pattern (process-only) | Pattern (mechanical) |
|---|---|
| "We'll remember to verify X next time" | CI lane fails if X not verified |
| "Add it to the team's checklist" | Protected CI validator fails if the required state is absent; an optional local adapter may suggest the fix earlier |
| "Train on the new procedure" | Validator emits error if procedure not followed |
| "Reviewer should catch it" | Per-change-class reviewer agent + per-PR fitness function |
| "Document the pattern" | Schema validation enforces the pattern |

Process is the COMPLEMENT to mechanical, not the substitute. Process fills the seams the mechanical can't reach. But every per-incident prevention starts with the question "what's the mechanical fix?"

## 3. Per-Sev SLA for prevention

Per [INCIDENT-MANAGEMENT.md §3.6](../INCIDENT-MANAGEMENT.md):

- **Sev 1**: mechanical prevention shipped within 30 days
- **Sev 2**: 60 days
- **Sev 3**: best-effort within current quarter
- **Sev 4**: backlog (P-tier per RISK-REGISTER)

## 4. Where mechanical preventions live

| Surface | Examples |
|---|---|
| Optional runtime-managed adapters | Installed outside the repository; never merge authority |
| CI lanes (`.github/workflows/`) | per-PR + nightly per [RELEASE-MANAGEMENT.md §2](../RELEASE-MANAGEMENT.md) |
| Foundry fitness functions | `oya-governance-{license, data-class, cohesion, doc-catalog, slo-coverage, blast-radius, glossary, adr-citation, ...}` |
| Schema validators | proto / OpenAPI / AsyncAPI / capability-record / catalog-record / regional-pack |
| Runtime gates | Cedar policy + per-capability autonomy ceiling + per-class data boundary |
| Catalog-driven | per-crate / per-capability / per-pack records that machine-validate |

## 5. Per-mistake workflow

When a mistake surfaces (incident, audit finding, customer report, drift discovery):

1. ☐ **Identify the system gap** that allowed it (5-Whys to the system / process / contract layer)
2. ☐ **Author the mechanical fix** as the prevention
3. ☐ **Ship the prevention** within Sev SLA
4. ☐ **Add row to** [`MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md): mistake / system gap / prevention / shipped-on / link-to-CI-lane-or-validator
5. ☐ **Cite the prevention** in the per-incident postmortem
6. ☐ **Verify** prevention catches the original failure mode (test the gate)
7. ☐ **Audit-emit** `EVT-PREVENTION-SHIPPED` per ADR-0003

## 6. Prevention as a Foundry capability

Per [DESIGN §3](../DESIGN.md) Foundry-as-accelerator:
- `oya.prevention.draft-from-incident` — Foundry capability that proposes a mechanical prevention from an incident postmortem
- `oya.prevention.verify-coverage` — runs the proposed prevention against the original failure mode (or replay trace)
- `oya.prevention.ship` — opens PR with the prevention + mistakes-and-fixes-ledger row

## 7. Long-running prevention areas

- License drift → `oya-governance-license` (per ADR-0013)
- Data-class annotation gap → `oya-governance-data-class` (per ADR-0008)
- Cross-axis contract drift → `oya-governance-cohesion` (per ADR-0011)
- Brand residue (deprecated aliases or tautological brand transitions) → `oya-governance-brand-residue` (per ADR-0017 / MFL-0011)
- Legacy ADR citation in active docs → `oya-governance-adr-citation`
- Glossary term drift → `oya-governance-glossary`
- Forward-reference (markdown link to path not on origin/main) → `pre-commit-forward-ref.sh`
- YAML date integrity (unquoted dates) → `pre-commit-yaml-date.sh`
- Worktree leakage / branch-name collision → spawn-time check
- Foundation-bypass expiry → `bypass-expiry-monitor` background job
- Per-capability eval-set drift → nightly run + regression gate
- Per-region regulator-watch lane (per [COMPLIANCE-MATRIX](../COMPLIANCE-MATRIX.md))
- Per-vertical control-evidence cadence

## 8. Sources
ADR-0003/0008/0013/0017, [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md), [`MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md), CLAUDE.md prevention-doctrine pointer, Google SRE workbook (post-incident learning chapters).
