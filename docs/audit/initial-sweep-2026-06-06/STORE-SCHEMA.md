# STORE-SCHEMA.md — Canonical JSON SSOT Store Schema (Workstream B1 schema-freeze deliverable)

**STATUS: `FROZEN — door:one-way founder sign-off GRANTED 2026-06-07 (B2 schema-freeze)`**

> **FREEZE RIDER (founder sign-off 2026-06-07, B2 schema-freeze; door:one-way).** The §6 OPEN items are ruled as follows: **OPEN-1** ADOPT no-tombstone (per D-SSOT-CURRENT-TRUTH). **OPEN-2** KEEP forward-plans as first-class `proposed-forward-plan` entities. **OPEN-3 CORRECTED** — the draft "collapse `application`→`app`" is WRONG and is overruled: per the F-0029 reconciled layer enum (ADR-0106), the retired role `application`→**`usecase`** and the non-canonical `runtime`→**`app`**; `api`/`rest`/`grpc` stay distinct. The closed `role` enum (§3.2) therefore EXCLUDES `application` and `runtime` as canonical values (normalized at ingest). **OPEN-4** ADOPT `registry/stores/*.json`, one JSON per store. **OPEN-5** THREE stores (instructions · design · registry); the stale "two-store" doctrine phrase is corrected to THREE. This rider is the authoritative freeze; §3.2/§6 bodies below are read through it. Schema is now contract-frozen for B2 truth-capture; further schema changes require a new door:one-way ratification.

> **Role of this file:** The B1 schema-freeze deliverable of the BIG HYGIENE PASS (`BIG-HYGIENE-PASS-PLAN.md` §B1). It DRAFTS the data-structure contract for the three canonical JSON SSOT stores + the four guards' contracts + the limited-markdown set + the json→web path. It is a SPEC. It does NOT author content (that is B2/B3) and does NOT mutate `/Users/jasonlee/Developer/source`.
> **Authority:** `decision-record-oyatie-canon.md` — D-DOCORG, D-SSOT-CURRENT-TRUTH, D-DOCTRINE, D-CLOUD-NATIVE.
> **Grounding:** all closed enums below were surveyed READ-ONLY against the live `/Users/jasonlee/Developer/source` frontmatter on 2026-06-07 (see §0).
> **Discipline:** closed enums (no open `other`), data-structures-first, minimal (Torvalds taste).

---

## §0 — EVIDENCE: the real current-state the enums are grounded in (SOURCE-FORCED survey, 2026-06-07)

| Surveyed | Real values in `/Users/jasonlee/Developer/source` | Schema consequence |
|---|---|---|
| `oya/**/*.md` `doc_class` | 40+ variants: `ImplementationPlan`(1131), `Implementation-Plan`(919), `IP`(300), `Runbook`(637), `PRD`(73), `Architecture`(96), `ThreatModel`/`Threat-Model`, `Benchmark`(39)… casing+synonym chaos | `doc_axis`+`kind` MUST be CLOSED enums (§1.2); the 40+ map onto ADR-0388's 7 axes. |
| `status` (oya+docs) | ~40 values: `Accepted`(2099)/`accepted`(86), `Proposed`(532)/`PROPOSED`(68), `draft`(1686), `pending`(681), `Superseded`(22)/`superseded`(4), `wave-15-zf-scaffold`(204), `OK`(3)… | `lifecycle` MUST be a CLOSED enum (§1.3); raw casing/wave-noise normalized at ingest. |
| `rust_code_status` | `not-authored-in-this-wave`(408) — the ONLY value | the forward-plan signal (§1.4). |
| `lifecycle_rule` | `PROPOSED`(68) — the ONLY value | second forward-plan signal (§1.4). |
| `docs/decisions/` (347 ADRs) keys | `id,status,date,supersedes,superseded_by,related,related_specs,amends,depends_on,owner(s),deciders,door,enforced_by` | design-store entity shape (§1.1)+edges (§1.5). |
| ADRs with NON-EMPTY `superseded_by` | **22** (e.g. `[ADR-0145]`,`[ADR-0515]`,`[ADR-0363]`,`[ADR-0383]`) | THE load-bearing call — no tombstones → excise+scrub (§1.5, OPEN-1). |
| `registry/catalog/*.yaml` | **903** files, stable ~11 fields: `context,role,capability,plane,slo,data_classes_owned,operational_classes_owned,api_stability,security_review,supply_chain`(+`traceability,non_claims`) | registry-store maps ~1:1 (§3) — already clean. |
| catalog `role` | `kernel`(238) `domain`(146) `adapter`(141) `app`(96) `worker`(88) `runtime`(55) `api`(45) `usecase`(30) `application`(23) `rest`(17) `test`(11) `grpc`(6) `infrastructure`(3) `bindings`(3) `cli`(1) | CLOSED enum w/ near-dupes → normalize (OPEN-3). |
| catalog `plane` | `control`(707) `data`(167) `audit`(21) `analytics`(8) | clean CLOSED enum (§3.2). |
| catalog `data_classes_owned` | `INTERNAL_ONLY,PUBLIC,PII_IDENTIFYING,PII_QUASI_IDENTIFIER,FINANCIAL,FINANCIAL_REGULATED_CREDIT,PHI,PCI,BEHAVIORAL_TENANT_PRODUCT,BEHAVIORAL_ADS,SEARCH_QUERY,DECLARED_PREFERENCE,SENSITIVE_PIPA_ART` | CLOSED `data_class` enum (§3.2). |
| masterplan/index | `specs/masterplan.json`(hand)+`docs/machine-readable/masterplan.generated.json`(generated) | PLANS/INDEX are generated VIEWS, NOT entities (§1.6,§8). |
| instructions sources | `AGENTS.md`,`CLAUDE.md`,`.claude/{commands,skills,settings.json}` | instructions-store sources (§2). |

**Method note (load-bearing):** the per-file frontmatter read AT EXECUTION is authoritative; the counts are INDICATIVE (vary by grep). The METHOD — close every open enum to a canonical set, normalize the chaos into it at ingest — is the deliverable, not the count.

---

## §0.1 — Design invariants (every store obeys)

1. **Closed enums only.** No `other`/free-form. An ingested off-enum value = gate RED, never silently accepted.
2. **Current-truth only — no tombstones.** No superseded entities, no history, no `superseded_by`/`amended_by` back-edges, no `_archive`. Superseded entities are EXCISED; inbound refs SCRUBBED. Git history is the sole archive.
3. **Edges point only to CURRENT entities.** `supersedes`/`depends_on`/`related` reference ONLY surviving ids. Dangling = gate RED.
4. **Keyed / enum-accessed.** Every entity reachable by a stable canonical KEY; agents enum to the exact section ("like search", §5).
5. **Authored from live state, never from the mess.** Stores populated from build-graph/CI/gate-registry/contracts/running-services; docs are truth-extraction residue only.
6. **Forward-plans are KEPT, modeled distinctly** (founder ruling 2026-06-07). PROPOSED-scaffold docs for not-yet-authored code → `lifecycle: proposed-forward-plan`, NOT destroyed, NOT conflated with current-accepted truth (§1.4). *(Supersedes the BIG-HYGIENE-PASS-PLAN §A.0.1 "PROPOSED-vapor=DESTROY" line; abandoned-vapor subset remains a future prune-candidate — OPEN-2.)*
7. **No forbidden vocab** (`forgejo`/`foundry`/`jenkins`/`oya-vcs`/retired-CLI `bin/oya`,`oya gate/verify`). Carve-out: external "Palantir Foundry". Enforced by the B2 N1 pre-filter + the C-lane gate.

---

## §1 — STORE 1: `design-store` (decisions · PRDs · specs · plans · masterplan)

**File:** `registry/stores/design-store.json`. PLANS+INDEX (masterplan/ADR-index) are NOT entities — generated VIEWS (§1.6,§8).

### §1.1 — Entity shape
```jsonc
{
  "key": "ADR-0196",
  "doc_axis": "DECISIONS",            // CLOSED §1.2
  "kind": "adr",                      // CLOSED §1.2
  "title": "...",
  "lifecycle": "accepted",            // CLOSED §1.3
  "owner": "council-architecture",    // required (total-accounting)
  "date": "2026-05-18",
  "body_ref": "obsidian://decisions/ADR-0196.md",  // §7 — prose in a limited-md view; store holds metadata+pointer, NOT prose
  "edges": { "supersedes": [], "depends_on": ["ADR-0064"], "related": ["ADR-0184"] },  // CURRENT-only §1.5
  "enforcement": { "enforcement_status": "enforced", "enforced_by": ["oya-check-doc-axis"] },  // optional
  "door": "one-way"                   // one-way|two-way|none
}
```
IP/PRD/spec entities share the shape; axis-specifics hang off `subtype` (e.g. IP: `{microservice, journey_ref, availability, rust_code_status}`).

### §1.2 — CLOSED `doc_axis` (ADR-0388 seven + transient IDEAS)
`DECISIONS | PLANS | INDEX | SPECS-MS | SPECS-CRATE | RUNBOOKS | IPS | IDEAS`. PLANS+INDEX appear only on generated VIEWS. SPECS-CRATE = registry-store's domain (edge-target only here).
**CLOSED `kind`** (normalizes the 40+ raw `doc_class`): `adr | prd | spec_ms | spec_crate | implementation_plan | runbook | idea`.

### §1.3 — CLOSED `lifecycle` (normalizes ~40 raw `status`)
`accepted | proposed-forward-plan | implemented | draft | idea`. Current-accepted-truth = only `accepted`/`implemented`. **EXCLUDED by construction (no enum value):** `superseded`/`deprecated`/`archived`/`retired` — these are EXCISION triggers, not states (structural enforcement of "no tombstones").
- §1.3b CLOSED `enforcement_status`: `enforced | shadow | planned | none`.
- §1.3c CLOSED `availability`: `paid | free | internal`.

### §1.4 — Forward-plan rule (founder ruling)
`lifecycle: proposed-forward-plan` (KEPT) IFF a forward-plan signal: `rust_code_status==not-authored-in-this-wave` OR `lifecycle_rule==PROPOSED` OR (`status∈{Proposed…}` AND describes not-yet-authored code). Preserved in `subtype.rust_code_status`. Never folded into an `accepted` view; IS a first-class store entity.

### §1.5 — Edges (CURRENT-only — no-tombstone enforcement)
`supersedes` → empty at steady state (predecessor excised). `depends_on`/`related` → target MUST exist (dangling=RED). **`superseded_by` / `amends`/`amended_by` DO NOT EXIST in the schema** (the structural form of no-tombstones; the 22 source `superseded_by` ADRs are excised, successors keep no back-edge; amendments re-author in place, prior text = git-history).

### §1.6 — PLANS/INDEX = generated VIEWS (§8), never entities; store↔view drift = RED.
### §1.7 — KEY scheme: `ADR-NNNN` (slug dropped) / `IP-NNN-slug` / `PRD-<ms>` / `SPEC-MS-<ms>` / `IDEA-<slug>`. Accessor `read design <key> [<section>]`.

---

## §2 — STORE 2: `instructions-store` (canonical agent-instructions)
**File:** `registry/stores/instructions-store.json`. Replaces scattered `AGENTS.md`+`CLAUDE.md`+`.claude/{commands,skills}` narration; keyed by directive-section.
### §2.1 — Entity: `{key, scope, category, title, directive, applies_when, authority, edges}`.
### §2.2 — CLOSED `scope`: `all-agents | executor | architect | planner | critic | analyst | verifier | qa-tester | writer | designer`.
### §2.3 — CLOSED `category`: `discipline | routing | tool-usage | commit-protocol | boundaries | verification | forbidden-vocab`. CLOSED `applies_when`: `always | mutation | delete | author | review | commit | session-start`. CLOSED `authority`: `founder | doctrine | tool-mandated`.
### §2.4 — KEY = directive slug (`source-forced-protocol`,`forbidden-vocab`,`commit-signing`,`verifier-lane`…). `read instructions <key>` / `read instructions --scope executor`.

---

## §3 — STORE 3: `registry-store` (per-crate catalog metadata; consolidates 903 yaml)
**File:** `registry/stores/registry-store.json`. Maps ~1:1 to the clean catalog.
### §3.1 — Entity: `{key(=crate), context, role, capability, plane, slo, data_classes_owned[], operational_classes_owned[], api_stability, security_review, supply_chain, traceability{changeset, source_adrs[]→design-store CURRENT}, non_claims[]}`.
### §3.2 — CLOSED enums:
```
role            : kernel|domain|adapter|app|usecase|runtime|api|worker|rest|grpc|test|infrastructure|bindings|cli
plane           : control|data|audit|analytics
api_stability   : preview|beta|stable|deprecated       (today only preview)
security_review : unreviewed|reviewed|exempt            (today only unreviewed)
supply_chain    : source-only|vendored|binary           (today only source-only)
data_class      : PUBLIC|INTERNAL_ONLY|PII_IDENTIFYING|PII_QUASI_IDENTIFIER|FINANCIAL|FINANCIAL_REGULATED_CREDIT|PHI|PCI|BEHAVIORAL_TENANT_PRODUCT|BEHAVIORAL_ADS|SEARCH_QUERY|DECLARED_PREFERENCE|SENSITIVE_PIPA_ART
operational_class : AUDIT|SECRET
```
Normalization flag (OPEN-3): `role` near-dupes `app`/`application`, `api`/`rest`/`grpc` → collapse vs keep-distinct (founder).
### §3.3 — KEY = crate name. `read registry <crate> [<field>]`.

---

## §4 — The 4 GUARDS (contracts only — BUILT in Workstream C, demoted off A-critical-path)
- **(a) ACCESSOR** `read <store> <id> [<section>]`: returns entity/field; `--filter` enums (search pattern); missing key→error(non-zero); duplicate key→error; read-only; deterministic.
- **(b) FORMATTER**: idempotent `format(format(x))==format(x)` byte-identical; entities key-sorted; stable field order; clean per-entity diffs; un-formatted store→RED.
- **(c) MERGE-DRIVER** (`.gitattributes` for `registry/stores/*.json`): different-key edits auto-merge (union+reformat); same-key→conflict surfaced (no silent clobber).
- **(d) ENTITY-INCREMENTAL GATE**: validates ONLY changed-key entities (only-the-changed-is-cold); checks closed-enum conformance, no-dangling edges, no forbidden-vocab, no excised-lifecycle, forward-plans-not-mislabeled. C1 consumes this.

## §5 — Accessor key-scheme (the "enum to the exact section, like search" pattern)
`read design ADR-0196 edges` · `read instructions forbidden-vocab directive` · `read registry oya-cloud-iac-runtime data_classes_owned`. Filters: `read design --axis=DECISIONS --lifecycle=accepted`, `read instructions --scope=executor`, `read registry --context=cloud-iac --role=kernel`.

---

## §6 — SCHEMA-DESIGN DECISIONS REQUIRING FOUNDER RATIFICATION (B1 gate)
| # | Decision | Recommendation | Status |
|---|---|---|---|
| OPEN-1 | No `superseded_by`/`amended_by`; 22 source ADRs carry them → model supersession as EXCISION (predecessor removed, no back-edge). | Adopt no-tombstone | **pre-resolved by D-SSOT-CURRENT-TRUTH; confirm** (the 22 ADRs' rationale becomes git-archaeology-only). |
| OPEN-2 | Forward-plan KEEP (§1.4) vs plan §A.0.1 "PROPOSED-vapor=DESTROY". | KEEP per founder's 2026-06-07 ruling; abandoned-vapor subset = later prune-candidate. | **resolved (founder ruled KEEP); abandoned-subset deferred.** |
| OPEN-3 | registry `role` near-dupe collapse (`app`/`application`; `api`/`rest`/`grpc`). | ~~Collapse `application→app`~~ **CORRECTED:** `application→usecase` + `runtime→app` (per F-0029/ADR-0106 reconciled enum); keep `api`/`rest`/`grpc` distinct; closed `role` enum excludes `application`/`runtime`. | **RESOLVED (founder freeze 2026-06-07).** Normalization (903 catalog + gate baselines) executes in B2 truth-capture. |
| OPEN-4 | Store file location/format: `registry/stores/*.json`, one JSON/store; the 903 yaml CONSOLIDATE-then-`git rm`. | Adopt | **minor — founder confirm** (one-way: accessor/merge-driver/gate hard-code the path). |
| OPEN-5 | Doctrine line says "two-store"; allow-list+task name THREE (instructions·design·registry). | THREE | **doc-fix — correct the stale "two-store" phrase in the SSOT.** |

## §7 — LIMITED Obsidian-format markdown (the only surviving hand-authored md)
Carve-out (`README`/`CLAUDE`/`AGENTS`/`LICENSE`/GitHub-specials/`SKILL.md`) + DECISIONS/RUNBOOKS/IPS prose bodies as **Obsidian wikilink md** (`[[ADR-0064]]`), each the `body_ref` of a store entity. Wikilinks resolve to store keys ONLY (a wikilink to an excised id = RED). Nothing else survives; PLANS/INDEX/SPECS carry no hand-prose.

## §8 — json→web generation (DEFINED, built later; pipeline not CLI)
`stores + Obsidian md → [GitHub Actions / oya-ci pipeline producer] → generated VIEWS (masterplan.generated.json [PLANS], ADR-INDEX [INDEX], web human-readable site)`. One direction only (stores→views); views never hand-edited (drift-gated); regenerated on store change (only-changed-cold).

## §9 — Out of scope (Torvalds-minimal fence)
No `other`-enum. No history/tombstone/`_archive`/back-edges. No CLI. No content authoring (B2/B3). No guard implementation (C). No source mutation. No migration of the mess — author fresh from live state.
