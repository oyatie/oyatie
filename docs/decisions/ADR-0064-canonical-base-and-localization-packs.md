# ADR-0064: Canonical base + localization packs — pack-pluggable µservice architecture, Korea is pack #1

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0056, ADR-0058, ADR-0059, ADR-0060, ADR-0062, ADR-0063, Bominal ADR-0140 (inherited)

---

## Context

Per user instruction 2026-05-13: "We work with canonical base and localization packs. So the first localization pack is Korea."

Oyatie has always implicitly distinguished jurisdiction-neutral business logic from jurisdiction-specific binding (Bominal ADR-0140 regional-pack pattern; oyatie inherits). What was missing: a load-bearing architectural rule, surfaced in MASTERPLAN, that makes the distinction explicit and enforceable.

This ADR locks the rule.

---

## Decision

### 1. Canonical global base + three overlay forms

Every customer-facing µservice has a **canonical global base** (jurisdiction-agnostic) and zero or more **localization overlays**. The overlay form is chosen per-concern — three forms exist, all valid:

| Form | Definition | When to use | Naming (BNF v4.1) |
|---|---|---|---|
| **Seam** | A port (trait) inside the canonical base where a jurisdiction-specific value or thin trait impl plugs in via DI | The variation is a value or small trait impl (statutory rate, tax bracket, leave-day count) | Port lives in canonical kernel; impl in pack crate: `oya-<microservice>-<pack>-<bc>-<layer>` |
| **Adapter** | A separate adapter crate translating jurisdiction-specific I/O (protocol, format, government portal) into canonical domain types | There is a discrete I/O surface (EDI, government API, vendor protocol) | `oya-<microservice>-<pack>-<bc>-adapter` |
| **Pack** | A coherent bundle of seams + adapters + Cedar fragments + Workflow templates + Typst templates, shipped as one deployable unit per jurisdiction | The deployable bundle and the doc-suite + audit-chain + acceptance-evidence unit | `oya-pack-<pack>-<microservice>-<layer>` for discrete bundle crates; pack as a whole is the catalog entry in `docs/localization-packs/<pack>.md` |

**Canonical global base = universal product.** Examples:

- `oya-payroll-run-domain` — "gross-to-net payroll run" in neutral terms; statutory rates **injected via seam**, never hardcoded
- `oya-medical-encounter-domain` — "patient encounter" without HIRA-specific fields
- `oya-accounting-journal-domain` — "double-entry journal" without K-GAAP / IFRS account-class baked in

**Choosing the form** (per-concern):

- **Seam** when variation = a value or a small `Provider`-style trait. Smallest blast radius. Preferred for statutory rates, tax tables, leave-day counts, holiday calendars.
- **Adapter** when variation = a discrete I/O protocol. Preferred for EDI formats, government API ingestion, vendor cross-walks.
- **Pack** for the deployable bundle. A pack composes seams + adapters; it is the **unit of release** and the **unit of doc-suite enforcement** (per ADR-0063).

The three forms compose: a pack composes adapters + seam impls + policy fragments + templates. There is no "or" between them — they are layers, not alternatives.

### 2. Pack-pluggability rule (mandatory)

A customer-facing µservice ships to a paying tenant only when:

- (a) its canonical base passes the M02 substrate quality bar (per ADR-0062), **AND**
- (b) at least one localization pack exists for it, **OR** an explicit ADR declares the µservice pack-neutral (e.g., `connect-messenger` core protocol is jurisdiction-neutral; only retention windows are pack-specific)

The canonical base alone is **not** shippable to a paying tenant — a pack is mandatory unless explicitly exempted.

### 3. Pack composition

Each pack consists of:

| Component | Path |
|---|---|
| Pack manifest | `docs/localization-packs/<pack>/pack.yaml` |
| Pack catalog entry | `docs/localization-packs/INDEX.md` (table row) |
| Pack overview doc | `docs/localization-packs/<pack>.md` |
| Per-µservice overlay crates | `crates/oya-<microservice>-<pack>-*` OR `crates/oya-pack-<pack>-<microservice>-*` |
| Per-µservice overlay PRD | `docs/prds/<microservice>-<pack>.md` (required when pack adds material scope) |
| Per-µservice regulatory ADR | `docs/decisions/ADR-NNNN-<pack>-<microservice>-regulatory.md` |
| Cedar policy fragments | `crates/oya-policy-<pack>-*` |
| Workflow Studio templates | `crates/oya-workflow-templates-<pack>-*` |
| Typst document templates | `crates/oya-document-templates-<pack>-*` |
| Acceptance evidence | `docs/localization-packs/<pack>/evidence/<microservice>.md` |
| Signed corpus.lock | `docs/localization-packs/<pack>/corpus.lock` (per ADR-0190 inheritance) |

### 4. Pack manifest (`pack.yaml`) shape

```yaml
pack:
  code: kr                            # lowercase ISO-3166-1 alpha-2 (or kebab-extended for sub-pack)
  name: "Korea"
  status: active | planned | future
  languages: [ko, en]                 # supported display languages
  microservices_in_scope:
    - hr
    - payroll
    - accounting
    - medical
    - pharmacy
    - patient
    - emergency
    - connect
    - payments
    - insurance
    - manufacturing
    - logistics
    - facility-ops
    - procurement
    - security
  regulatory_bindings:
    - id: 4dae-edi
      regimes: [NPS, NHIS, 고용보험, 산재보험]
      milestone: M03
    - id: yearly-tax-settlement
      regimes: [연말정산]
      milestone: M03
    # ...
  connectors:
    - id: hira-dur
      surface: "REST + EDI"
      milestone: M04
    # ...
  acceptance_milestones: [M03, M04, M05, M06, M07]
  corpus_lock: "kr/corpus.lock"
```

### 5. Pack lifecycle

- **`planned`**: pack catalog entry exists; manifest authored; no crates yet. Allowed in `docs/localization-packs/INDEX.md`.
- **`active`**: at least one µservice has the pack overlay shipped and a paying tenant uses it.
- **`maintained`**: pack remains active; regulatory corpus.lock refreshed at least quarterly with signed bumps.
- **`retired`**: pack removed; all crates physically deleted; INDEX entry moved to retired-packs section (no marked-retired flags inside crates — stale removed in reality per `feedback_autonomous_implementation_artifacts.md`).

### 6. Korea is pack #1 (foundational)

Pack `kr` is the foundational localization pack. Implications:

- M01–M07 milestones ship the canonical base **plus** the KR pack in lock-step (oyatie's first paying tenant is KR).
- The KR pack must be complete and CI-green before any second pack (US / EU) can graduate from `planned` to `active`.
- KR-specific decisions (4대보험 EDI, 연말정산, HIRA DUR, K-GAAP, PIPA, 전자금융업, 산업안전보건법, etc.) live in the KR pack — never in canonical base µservice crates.

### 7. Pack-isolation rule

Pack crates MUST NOT import from other pack crates. Cross-pack integration (e.g., US tenant ingesting KR pharma data for clinical trial) flows via Workflow + Ontology (per ADR-0059). Pack-to-pack cross-talk would couple jurisdictions and break the pack-pluggability invariant.

CI enforcement: `oya-check-architecture --cross-pack-refusal` (new sub-command; M02-P20 scope).

### 8. Canonical base must remain pack-neutral

CI enforcement: `oya-check-architecture --canonical-base-neutrality` (new sub-command; M02-P20 scope).

Forbidden patterns in canonical base crates:

- Hardcoded statutory rates (e.g., `0.09` for KR NPS rate) — must come from pack overlay
- Hardcoded country codes in business logic (acceptable in pack adapters only)
- Hardcoded language strings in domain types (use i18n keys; locale resolution lives at presentation)
- Hardcoded regulatory authority names (e.g., "HIRA", "ANS-MES") in domain types

---

## Consequences

**Positive:**

- Architectural clarity: jurisdictional concerns are isolated, swappable, and individually shippable.
- International expansion (M09 US, M10 EU, M12+ JP/SEA/MENA) is a matter of authoring packs, not refactoring canonical base.
- Compliance audits per jurisdiction stay scoped to the relevant pack.
- Multi-tenant deployments serving different jurisdictions enable different packs per tenant.

**Negative:**

- Up-front discipline cost: every new µservice now has to decide canonical-vs-pack scope explicitly.
- Pack `kr` is on the critical path for M01–M07; if KR pack work falls behind, milestones slip.

**Neutral:**

- Inherits Bominal ADR-0140 regional-pack pattern; oyatie codifies enforcement.

---

## Compliance

CI lanes (M02-P20 scope; lane bodies authored in P20 phase):

- `oya-check-architecture --cross-pack-refusal` — no inter-pack imports
- `oya-check-architecture --canonical-base-neutrality` — no jurisdiction hardcoding in canonical-base crates
- `lean-a5-doc-coverage` (per ADR-0063) — every active pack has manifest + INDEX entry + per-µservice overlay docs

Owner team: `council-architecture` (pack architecture) + per-pack `gtm-customer-success` lane (KR lead).

---

## References

- Bominal ADR-0140 (regional-pack pattern; inherited 1:1)
- Bominal ADR-0190 (versioned regulatory corpus.lock; inherited 1:1)
- ADR-0056 (BNF v4.1 naming grammar)
- ADR-0058 (flat µservice catalog)
- ADR-0062 (quality/perf/scale bar applies to canonical base AND packs)
- ADR-0063 (documentation suite coverage; pack overlay docs enforced)
- `docs/MASTERPLAN.md` §2.5, §5.5
- `docs/localization-packs/INDEX.md` (canonical pack catalog)
- `docs/localization-packs/kr.md` (pack #1 spec)
