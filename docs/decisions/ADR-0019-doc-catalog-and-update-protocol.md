---
id: ADR-0019
status: Accepted
doc_status: published
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Doc catalog update protocol

# ADR-0019: Doc catalog and update protocol — every consolidated doc has owner / trigger / cadence / dependent-docs / validation; pre-flight + authoring + validation + review + publish stages; agent-authoring policy (agents propose; humans approve; catalog-validated additions auto-approved by the catalog gate); machine-readable mirror at machine-readable/catalog.json

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0011, ADR-0013, ADR-0016, ADR-0017, ADR-0018

---

## Context

The consolidated docs tree (PRD, DESIGN, ROADMAP, PRIVACY-PROGRAM, COMPLIANCE-MATRIX, GLOSSARY, TOOLCHAIN, CONTRADICTION-LEDGER, ADR-INDEX, plus per-microservice + per-vertical + per-pack + per-runbook entries) is a living artifact whose freshness depends on every team noticing when their domain shifts. Without a catalog that names *who owns each doc*, *what events trigger an update*, *what cadence the doc is reviewed at*, *which other docs depend on it*, and *what validation runs at PR time*, the inevitable failure mode is doc drift — the PRD shifts but COMPLIANCE-MATRIX doesn't catch up; the Glossary retires a term but a downstream README still uses it.

Cohesion (ADR-0001) compounds the risk. The flat-catalog cohesion claim is articulated across multiple docs; if any one doc lags, the cohesion artifact fragments. The audit chain (ADR-0003) needs to record doc-level events (`EVT-DOC-PUBLISHED`, `EVT-DOC-UPDATED`, `EVT-CROSS-AXIS-CONTRADICTION-FOUND`) so doc freshness becomes auditor-visible. Agent-authoring discipline (PRD §3.1 commitment 5) requires explicit roles for what agents may write directly vs what agents propose for human ratification.

---

## Decision

We adopt a **doc catalog as protocol** with a structured per-doc record, a five-stage lifecycle, an agent-authoring policy with explicit roles, and a machine-readable mirror at `machine-readable/catalog.json`.

### Per-doc catalog record

Every consolidated doc has a row in `docs/DOC-CATALOG.md` and a mirror in `docs/machine-readable/catalog.json`:

```yaml
doc_id: doc.privacy_program
title: PRIVACY-PROGRAM.md
path: docs/PRIVACY-PROGRAM.md
owner: council-privacy
secondary_owners: [foundry, council-architecture]
trigger_events:
  - EVT-REGULATORY-CHANGE-DETECTED        # per-pack regulator-watch lane
  - EVT-AUDIT-FINDING                      # per regulator audit
  - EVT-CROSS-AXIS-CONTRADICTION-FOUND     # contradiction ledger
  - EVT-DSR-CASCADE-FAILURE                # DSR SLA breach
cadence: monthly                            # baseline review cadence
dependent_docs:
  - doc.compliance_matrix
  - doc.security_program
  - doc.glossary                            # data class names
  - doc.adr_index                           # ADR cross-references
validation_lane: oya-governance-doc-privacy-program
agent_authoring_policy: propose-only        # agents draft; humans approve
machine_readable_mirror: machine-readable/privacy-program.json
publish_topic: oya.docs.privacy-program.published.v1
```

### Five-stage lifecycle

Every doc change passes through:

1. **Pre-flight.** PR author checks the doc's catalog row for owner, dependent docs, validation lane. The author confirms the change is in scope; cross-doc impact is enumerated in the PR's `## Traceability` block.
2. **Authoring.** Doc edit happens; if agent-authored, the agent attaches its trace evidence to the PR's `## Evidence` block. Per-doc `agent_authoring_policy` controls what an agent may write directly vs what requires human draft + agent assist.
3. **Validation.** Per-doc CI lane runs (cohesion lane for cross-microservice docs; glossary lane for terminology; license lane for vendor-partner-ledger; data-class lane for privacy program; etc.). Failure blocks merge.
4. **Review.** Owner team reviews + approves; for cross-microservice docs the dependent-docs owners co-review per ADR-0011 protocol.
5. **Publish.** On merge: catalog row updated with `last_updated_at`, `last_updated_pr`; `EVT-DOC-PUBLISHED` emitted to the audit chain (ADR-0003); machine-readable mirror regenerated; trust portal updated for customer-facing docs.

### Agent-authoring policy

Three roles based on doc class:

| Class | Examples | Agent role |
|---|---|---|
| **catalog-validated additions** | catalog records, capability records, machine-readable mirror entries, per-pack regulator-watch publications | **Agent direct-write** (after catalog validator passes); human reviews PR but does not co-author |
| **propose-only** | PRD, DESIGN, PRIVACY-PROGRAM, COMPLIANCE-MATRIX, GLOSSARY, ROADMAP, per-pack regulatory text, ADR pack text | **Agent drafts; human approves**; the PR records the agent's draft + human's edits separately |
| **human-only** | CONSTITUTION.md, source-of-truth.md, council-decision logs, founder ratifications | **Human writes**; agent may surface diffs / cross-references in review comments |

Agents that violate the policy are caught by `oya-governance-agent-authoring`, which checks the PR's `## Evidence` block for agent-trace metadata vs the doc's catalog policy.

### Per-doc validation lanes

| Doc class | Lane |
|---|---|
| PRD / DESIGN / ROADMAP | `oya-governance-cohesion` (cross-microservice claims must be backed by ADRs) |
| PRIVACY-PROGRAM | `oya-governance-data-class` (data-class taxonomy consistency per ADR-0008) |
| COMPLIANCE-MATRIX | `oya-governance-regulatory-binding` (every regulator row maps to a pack per ADR-0010) |
| GLOSSARY | `oya-governance-glossary` (forbidden vocab + industry alignment per ADR-0018) |
| TOOLCHAIN / VENDOR-PARTNER-LEDGER | `oya-governance-license` (per ADR-0013) + `oya-governance-build-vs-buy` (per ADR-0014) |
| ADR pack | `oya-governance-adr-template` (template conformance) + cross-reference check |
| Catalog records | `oya-governance-catalog-records` (each workspace crate has a catalog record per ADR-0015) |
| Per-pack docs | `oya-governance-pack` (seam coverage per ADR-0010) |

### Machine-readable mirror

Every doc has a JSON mirror at `docs/machine-readable/<doc-id>.json` regenerated on publish. The mirror exposes:

- Doc identity + last-updated metadata.
- Structured tables (e.g. PRIVACY-PROGRAM's data-class taxonomy; COMPLIANCE-MATRIX's regulator × control rows; GLOSSARY's term × analog rows; ADR-INDEX's ADR × status rows).
- Cross-references to dependent docs.

The mirrors are agent-consumed: Foundry capabilities (e.g. `pr.review.draft`, `adr.promotion.review`, `dep.license.review`) read mirrors instead of parsing markdown.

### Trigger-driven cadence

A doc's cadence is the *floor*, not the ceiling. Trigger events (regulator change, audit finding, contradiction found, DSR SLA breach, supply-chain CVE, vendor-license drift) override cadence — when a trigger fires, the doc enters review immediately regardless of when the next cadenced review is.

### Boundary

- Applies to: every doc in `docs/`, every ADR in the new pack, every catalog record under `registry/catalog/`, every capability record under `registry/capability-templates/`, every per-pack doc under `regional-packs/<pack>/`, every per-team CHARTER under `teams/`.
- Does not apply to: legacy ADRs (forensic), per-PR ephemeral evidence in PR bodies, dev-side scratch docs in worktrees.

---

## Consequences

### Positive

- Doc freshness becomes a tracked invariant, not a hope.
- Every doc has explicit owner, trigger, cadence, dependent-docs, and validation; reviewers know exactly what to check.
- Agent-authoring policy lets agents do real work (catalog records, mirrors, triage labels) without crossing into human-only territory.
- Machine-readable mirrors close the loop: agents consume the same canonical source, eliminating "agent reads outdated copy" failure modes.
- Closes LEDG-026 (Foundry fitness vs microservice-team autonomy) at the protocol level by codifying the fitness-fn dispute path through this same lifecycle.

### Negative

- Per-doc catalog row maintenance is real ops work; mitigation: agent-direct-write for catalog-validated additions.
- Five-stage lifecycle adds review time; mitigation: per-doc-class concurrent reviewers + auto-rebase merge queue.
- Agent-authoring boundary requires every PR to declare agent involvement; some authors will under-declare.

### Operational

- On-call: `EVT-DOC-CADENCE-OVERDUE` weekly rollup; `EVT-DOC-TRIGGER-FIRED` immediate notification to owner.
- Runbooks: `runbooks/doc-update-pr.md`, `runbooks/cross-doc-impact-analysis.md`, `runbooks/agent-authoring-evidence-attach.md`, `runbooks/machine-readable-mirror-regenerate.md`.
- CI: per-doc validation lanes (above), `oya-governance-doc-cadence` (overdue cadence detection), `oya-governance-doc-catalog` (every doc has a catalog row).
- Trust portal: customer-facing doc subset published at `docs.oyatie.com` with chain-anchored versioning.

---

## Alternatives considered

### Alternative A — No catalog; freshness by team memory

- **Pros:** zero structure cost.
- **Cons:** failure mode demonstrated in legacy corpus; inconsistencies LEDG-007 / LEDG-014 / LEDG-024.
- **Rejected because:** scale.

### Alternative B — Catalog without lifecycle stages (just owner + cadence)

- **Pros:** simpler.
- **Cons:** no audit emission; no agent-authoring boundary; cross-doc impact untracked.
- **Rejected because:** missing pieces.

### Alternative C — Per-doc README with self-declared cadence (no central catalog)

- **Pros:** self-contained per doc.
- **Cons:** cross-doc dependent-docs graph cannot be derived; council oversight impossible.
- **Rejected because:** cohesion.

---

## Open questions

1. **Q1.** Per-doc cadence baseline — quarterly default, or per-doc declared? Default: per-doc declared in the catalog row (canonical); quarterly is the safe minimum when no per-doc declaration is present. No grandfathered deviations; cadence changes are catalog-row edits, not exceptions. → owner: `council-architecture`.
2. **Q2.** Agent-direct-write for catalog records — does this require a per-PR human approve, or is the validator + automated sign-off sufficient? Default: human approve initially; promote to validator-only when agent reliability proven via eval (ADR-0007 eval harness ancestry, owned by `foundry`). → owner: `foundry`.
3. **Q3.** Machine-readable mirror format — JSON Schema versioned per mirror, or unified across mirrors? Default: per-mirror schema versioning. → owner: `foundry`.
4. **Q4.** Per-pack doc cadence — does each pack have its own catalog or share this one? Default: shared catalog with `regional_pack:` field on per-pack rows. → ADR-0010.
5. **Q5.** Customer-facing doc subset (trust portal) — automatic from catalog `customer_facing: true` flag, or hand-selected? Default: catalog flag. → ADR-0003 (trust portal).
6. **Q6.** Council-secretariat conflict (LEDG-029 — `platform-privacy-dub` drafts ADRs + runs governance) — does this ADR codify a secretariat-rotation rule? Default: yes; per-doc council chair cannot be from the doc's owning team. → owner: `council-architecture`.

---

## References

- `docs/DOC-CATALOG.md` (the live catalog this ADR formalizes)
- `docs/DOC-UPDATE-PROTOCOL.md` (the protocol this ADR formalizes)
- `docs/CONTRADICTION-LEDGER.md` LEDG-026 (Foundry vs microservice-team autonomy), LEDG-029 (council secretariat conflict)
- `docs/PRD.md` §3.1 commitment 5 (quality of contract over time-to-launch)
- ADR-0001 (cohesion), ADR-0003 (`EVT-DOC-PUBLISHED` audit emission), ADR-0011 (cross-microservice contract registry — catalog is generated), ADR-0013 (per-release SBOM cadence), ADR-0016 (per-wave gate evidence emission cadence), ADR-0017 (brand-rename batch evidence emission), ADR-0018 (glossary lane consumes machine-readable mirror)
- Diátaxis documentation framework (https://diataxis.fr/), Google docs-as-code references
