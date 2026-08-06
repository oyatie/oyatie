---
id: ADR-0064
status: Superseded
superseded_by: [ADR-0709]
amended_by: [ADR-329]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0064: Canonical base + localization packs — pack-pluggable µservice architecture, Korea is pack #1

> **Status:** Accepted
>
> **Amendment note — 2026-06-02 platform-readiness:** pack/catalog/evidence roots authorized by this ADR remain
> canonical during the `{oya,cloud}` pure-split migration. The anti-sprawl rule removes service-shaped code in pack
> roots, not valid shared/versioned pack authoring artifacts, unless a future ADR supersedes this pack model.
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0056, ADR-0058, ADR-0059, ADR-0060, ADR-0062, ADR-0063, Bominal ADR-0140 (retired per ADR-0145) (inherited)

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
| **Pack** | A coherent bundle of seams + adapters + Cedar fragments + Workflow templates + Typst templates, shipped as one deployable unit per jurisdiction | The deployable bundle and the doc-set + audit-chain + acceptance-evidence unit | `oya-pack-<pack>-<microservice>-<layer>` for discrete bundle crates; pack as a whole is the catalog entry in `docs/localization-packs/<pack>.md` |

**Canonical global base = universal product.** Examples:

- `oya-payroll-run-domain` — "gross-to-net payroll run" in neutral terms; statutory rates **injected via seam**, never hardcoded
- `oya-medical-encounter-domain` — "patient encounter" without HIRA-specific fields
- `oya-accounting-journal-domain` — "double-entry journal" without K-GAAP / IFRS account-class baked in

**Choosing the form** (per-concern):

- **Seam** when variation = a value or a small `Provider`-style trait. Smallest blast radius. Preferred for statutory rates, tax tables, leave-day counts, holiday calendars.
- **Adapter** when variation = a discrete I/O protocol. Preferred for EDI formats, government API ingestion, vendor cross-walks.
- **Pack** for the deployable bundle. A pack composes seams + adapters; it is the **unit of release** and the **unit of doc-set enforcement** (per ADR-0063).

The three forms compose: a pack composes adapters + seam impls + policy fragments + templates. There is no "or" between them — they are layers, not alternatives.

### 1.5 Decision rules for ambiguous semantic-localization cases

The seam / adapter / pack trichotomy is a clean split for **mechanical** localization (values, protocols, bundles). For **semantic** localization (where the jurisdiction shapes the domain model itself), the line is less obvious. Concrete rules:

| Semantic case | Default | Rationale |
|---|---|---|
| **K-GAAP / IFRS / US-GAAP account classes** | Canonical base defines `AccountClass` as an open enum + `AccountClassResolver` **seam**; pack supplies the resolver impl with the chart-of-accounts | Account classes have universal structure (Asset/Liability/Equity/Revenue/Expense) but jurisdiction-specific code lists (K-GAAP `511 급여비용` vs US-GAAP `5000 wages`); the universal structure stays canonical, the list ships in the pack |
| **FHIR records** (R5 entity types) | `records` substrate µservice declares canonical FHIR R5 entity types in kernel; pack **adapters** translate EMR-vendor formats (Epic, Cerner, 유비케어) ↔ canonical | FHIR R5 is itself a universal standard; jurisdiction-specific extensions (USCDI v3, K-FHIR profiles) ship as pack adapters |
| **Retention windows** (legal hold / audit / messaging) | Canonical base declares `RetentionPolicy` **seam**; pack supplies impl with jurisdiction-specific windows (KR 3–84 months for connect; US HIPAA 6 years; EU GDPR varies by basis) | Pure value variation; seam is the cleanest form |
| **PIPA / GDPR / HIPAA legal-basis policies** | Canonical base declares Cedar **policy slots** (placeholder names like `data_subject_consent`, `legitimate_interest`); pack ships full Cedar policy fragments under those slot names | Cedar policy fragments are the canonical pack composition mechanism (per §3); they are not seam-injected values but discrete policy crates |
| **Tax brackets / statutory rates / minimum-wage tables** | Canonical base declares `StatutoryTableProvider` **seam**; pack supplies impl with the table | Pure value variation; seam is unambiguously correct |
| **Government EDI / API protocols** (NPS, HIRA, IRS, HMRC) | Pack **adapter** crates | Discrete I/O surfaces; adapter is the canonical mechanism |
| **Language overlays** (UI strings, document templates) | Canonical base uses i18n keys; pack ships Typst template overrides + locale bundles | Standard i18n pattern; the templates are part of pack composition (§3) |
| **Calendar / holiday tables** | Canonical base declares `HolidayCalendarProvider` **seam**; pack ships the calendar | Pure value variation |

**Hard rule (mechanically enforced):** if a domain concept would only ever exist for one jurisdiction (e.g., `KrSijungwol`, `JpInvoiceFormat`), that concept does NOT belong in canonical base — it ships entirely in the pack. The canonical base must remain the universal-product abstraction. CI lane `oya-check-architecture --canonical-base-neutrality` enforces: any identifier in a canonical-base crate matching `[A-Z][a-z]+(Kr|Us|Eu|Jp|Sea|Mena)\w*` or string literals containing a jurisdiction code triggers a violation.

**Soft rule (review-time):** when in doubt, prefer **seam** over adapter, and **adapter** over pack-bc inline. Smallest blast radius wins. Promote to a heavier form only when concrete evidence shows the lighter form is insufficient.

### 2. Pack-pluggability rule (mandatory)

A customer-facing µservice ships to a paying tenant only when:

- (a) its canonical base passes the M02 substrate quality bar (per ADR-0062), **AND**
- (b) at least one localization pack exists for it, **OR** an explicit ADR declares the µservice pack-neutral (e.g., `messenger` core protocol is jurisdiction-neutral; only retention windows are pack-specific)

The canonical base alone is **not** shippable to a paying tenant — a pack is mandatory. The canonical-base neutrality CI lane (per ADR-0064 §"Lane enforcement") refuses any paying-tenant deployment without a pack; pack-neutral µservices satisfy condition (b) above via the explicit `pack-neutral` ADR declaration, which is itself a canonical pack designation, not an exemption.

### 3. Pack composition

Each pack consists of:

| Component | Path |
|---|---|
| Pack manifest | `docs/localization-packs/<pack>/pack.yaml` |
| Pack catalog entry | `docs/localization-packs/INDEX.md` (table row) |
| Pack overview doc | `docs/localization-packs/<pack>.md` |
| Per-µservice overlay crates | `crates/oya-<microservice>-<pack>-*` OR `crates/oya-pack-<pack>-<microservice>-*` |
| Per-µservice overlay PRD | `docs/prds/<microservice>-<pack>.md` (required when pack adds material scope) |
| Per-µservice regulatory ADR | `docs/decisions/ADR-####-<pack>-<microservice>-regulatory.md` |
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

## Alternatives considered

| Alternative | Why rejected |
|---|---|
| **Single-tier (no canonical/pack split)** — bake jurisdiction logic directly into each µservice; ship `oya-payroll-kr` as a self-contained µservice instead of canonical `oya-payroll` + KR pack | Forces a fork per jurisdiction; M09 US payroll becomes a new µservice instead of a new pack; combinatorial blow-up; violates Bominal ADR-0140 inheritance |
| **Per-jurisdiction microservice fork** — `oya-payroll-kr`, `oya-payroll-us`, `oya-payroll-eu` as parallel µservices sharing nothing | Forks the business model per jurisdiction; cross-jurisdictional features (multi-country tenants) become impossible; code duplication explodes |
| **Single mega-pack containing all jurisdictions** — one `oya-pack-international` covering KR + US + EU + JP | Forces every tenant to load the world; KR-only customers ship US PII-handling code; data-residency posture breaks; pack quarterly refresh cadence becomes unmanageable |
| **Pack as runtime-only feature flag** (no separate crates) — single deployable with KR/US/EU branches gated by runtime flag | Removes compile-time guarantee that canonical-base stays jurisdiction-neutral; KR statutory bugs ripple into US tenants; observability per jurisdiction collapses |
| **Seam ONLY (no adapter form)** — all jurisdiction overlay via DI-injected traits | Discrete I/O surfaces (EDI / government API) become enormous trait families with implausible signatures; the abstraction breaks under real protocol complexity (NPS EDI v5.0 has 11 message types, 200+ fields) |
| **Per-µservice naming ADR** (every microservice gets ADR-####-microservice-X.md) | (Chosen — ADR-0063 §1 row 3.) Per-µservice ADRs are the canonical home for naming + scope decisions; cluster-level rollups (e.g., "all healthcare µservices") would coarse-grain the decision record and bury per-µservice exceptions |
| **Cluster-level naming ADR** (e.g., one "ADR-####-healthcare-cluster.md" covering medical/pharmacy/patient/emergency/clinical) | Rejected — would force every cluster decision through a single ADR; conflicts between cluster members couldn't be resolved at the ADR level; per-µservice exceptions would create ADR drift. Cluster-level ADRs are acceptable for **cross-µservice rules** but not for individual µservice scope/naming |
| **Pack-bc inline ONLY** (always inline the overlay as a BC inside the canonical µservice; never use discrete pack crates) | Works for small packs but breaks scale — KR pack covers ~28 µservices across 6 clusters; forcing inline forces every µservice crate to carry the KR pack baggage even in non-KR cells. Discrete pack crate form is required for multi-µservice-spanning overlays |
| **Discrete pack crate ONLY** (always use `oya-pack-<pack>-<microservice>-*`; never inline as a BC) | Adds crate-graph overhead for trivial seam impls; bloats the workspace member list; obscures the relationship between the canonical µservice and its overlay |

**Chosen design (this ADR):**

- Trichotomy seam / adapter / pack (§1) — gives the right tool for each concern density
- Hard rule on canonical-base neutrality (§1.5, §8) — mechanically enforced
- Pack as both inline-BC and discrete-crate (§1, §3) — flexibility per overlay size

## References

- Bominal ADR-0140 (regional-pack pattern; inherited 1:1)
- Bominal ADR-0190 (versioned regulatory corpus.lock; inherited 1:1)
- ADR-0056 (BNF v4.1 naming grammar)
- ADR-0058 (flat µservice catalog)
- ADR-0062 (quality/perf/scale bar applies to canonical base AND packs)
- ADR-0063 (documentation set coverage; pack overlay docs enforced)
- `docs/MASTERPLAN.md` §2.5, §5.5
- `docs/localization-packs/INDEX.md` (canonical pack catalog)
- `docs/localization-packs/kr.md` (pack #1 spec)
- `crates/oya-check-doc-coverage/` (LEAN-A5 enforcement binary)
