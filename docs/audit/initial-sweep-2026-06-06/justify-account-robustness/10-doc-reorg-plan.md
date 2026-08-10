# 10 — DOC-REORG PLAN (executable, on the existing scheme)

**Lane:** justify-account-robustness / DOC-REORG.
**Date:** 2026-06-06.
**Mode:** READ-ONLY audit + executable plan. No file was edited; every claim cites path + line + verbatim snippet.
**Charter lens applied:** D-DOCTRINE robust-not-false (every claimed gate proven by RED/GREEN to actually block), hyperscaler generated-not-hand-maintained registries, Linus no-special-cases / one good data structure, total accounting (owner + reachability + TTL per doc).

---

## 0. EVIDENCE BASE + CORPUS LOCATION (read this first — corrects the brief)

The brief and the founder ruling both say the three schemes are "verified present in source." **They are — but in the SOURCE oyatie monorepo, not in `linux/`.** This must be stated plainly so the plan is not mis-applied:

- The reorg corpus is **`/Users/jasonlee/Developer/source/docs/`** — verified present, **44 subdirs at depth 1, 2888 files** (`find … -type d | wc -l` = 44; `find … -type f | wc -l` = 2888). This matches the brief's "~40 current subdirs."
- `linux/docs/` holds only **10 non-audit subdirs** (audit artifacts + the kernel/stack pilot's `context/ migration/ research/`). The scheme files (`doc-style.md`, `DOCUMENTATION.md`, `DESIGN.md`, `catalog.json`, `contracts.json`, `planning-ssot-consolidation.md`) **do not exist anywhere under `linux/`** (verified by repo-wide `find`); they exist only under `source/docs/`. The WF2 register already states this corpus boundary: `docs-sweep/00-REST-OF-DOCS-REGISTER.md:5` — *"Corpus: `/Users/jasonlee/Developer/source/docs/` (the live oyatie docs tree) — NOT `linux/`."*
- **Consequence for execution:** this reorg runs against `source/docs/` and lands during consolidation; the migrated `linux/stack` monorepo mirrors the same canon homes (`docs/DOC-CATALOG.md` + `docs/CHANGELOG.md` per `prelane-0.7/00-GOVERNANCE-BOOTSTRAP.md:242`). The plan is written so the same topology + record + gate apply to both trees after consolidation (one-version rule).

**The three schemes — verified, with primary sources:**

| Scheme | What it classifies | Authoritative source (verified) |
|---|---|---|
| **Diátaxis quadrant** (`doc_class`) | content shape / folder home | `source/docs/standards/doc-style.md:41` "## 1. Diátaxis — the four quadrants"; `source/docs/DOCUMENTATION.md:18` "We adopt the **Diátaxis framework**" |
| **7-product-axis** (`axis`) | which bounded context / product the doc serves | `source/docs/DESIGN.md:17` "## 1. Cohesion thesis (one product, seven axes)"; machine mirror `source/docs/machine-readable/products.json` `"axes": 7` |
| **DOC-CATALOG tier** (`tier`) | governance weight / authoring rights | `source/docs/DOC-CATALOG.md:0` reading guide + §2 tiered tables; mirror `source/docs/machine-readable/catalog.json` per-doc `"tier"` |

These three are **orthogonal** (a doc has a shape AND a product-axis AND a governance weight). The reorg unifies them into ONE per-doc record and turns the three half-built gates blocking.

---

## (a) TARGET `docs/` FOLDER TOPOLOGY — Diátaxis quadrants, on the EXISTING scheme

The folder topology is **not invented here** — it is the unexecuted plan in `source/docs/ideas/planning-ssot-consolidation.md:107` ("## Diátaxis reorganization of `docs/` (confirmed)") + `:108-113`:
> "Four quadrants, each doc has exactly one home: **tutorials/** — learning-oriented … **how-to/** — task-oriented (runbooks, playbooks…) … **reference/** — information-oriented, **generated** … **explanation/** — understanding-oriented (architecture, ideas…)."

`doc-style.md:47-52` fixes the four quadrants + their length caps; `DOCUMENTATION.md:25` adds the **fifth home** for project artifacts:
> "Plus a fifth, project-management quadrant for the consolidated docs themselves (PRD/ROADMAP/RISK-REGISTER/etc.). The consolidated docs are *project artifacts*, not user docs."

`planning-ssot-consolidation.md:134` independently names that fifth home `decision` ("tutorial/how-to/reference/explanation/**decision**"). **Reconciliation ruling for this plan:** keep the canonical Diátaxis FOUR as `doc_class` values for user-facing docs, plus the explicit fifth **`Project`** home for governed project artifacts, and `decisions/` stays its own immutable namespace (NOT folded into any quadrant — ADRs are SSOT). This is 4 + 1-project + decisions-immutable = the only special case, and it is justified (ADRs are the generative SSOT, not Diátaxis content).

**Current → target home (root quadrants are the only top-level dirs; everything else nests under exactly one):**

| Current subdir(s) in `source/docs/` (file count) | Target Diátaxis home | `doc_class` | Rule / evidence |
|---|---|---|---|
| `tutorials/` (10) [ALREADY EXISTS], `onboarding/` (12) | **`tutorials/`** | `Tutorial` | `doc-style.md:49` "Tutorial … `README.md` quickstart, onboarding tracks"; cap ≤500 |
| `runbooks/` (207), `../../../../templates/checklists/` (31), `release/` (28), `advanced-cicd/` (39), `operators/` (2), `customer-success/` (8) | **`how-to/`** | `HowTo` | `doc-style.md:50` "How-to … runbooks, `RUNBOOKS-INDEX.md` rows, `../../../../templates/checklists/`"; `planning-ssot-consolidation.md:110` "how-to/ (runbooks)"; cap ≤300 |
| `specs/` (116), `api/` (8), `machine-readable/` (11), `standards/` (103), `localization-packs/` (16), `regional-packs/` (3), `performance-budgets/` (6), `policies/` (2), `automation/` (19) | **`reference/`** (GENERATED where derivable) | `Reference` | `doc-style.md:51` "Reference … `SPEC.md`, `ADR-INDEX.md`, `GLOSSARY.md`, `contracts/`"; `planning-ssot-consolidation.md:111` "reference/ — information-oriented, **generated** … build output"; cap ≤600 |
| `architecture/` (45), `ideas/` (20), `research/` (4), `teams/` (41) | **`explanation/`** | `Explanation` | `doc-style.md:52` "Explanation … `DESIGN.md`, this file"; `planning-ssot-consolidation.md:113` "explanation/ — architecture, ideas, regional-pack rationale"; cap ≤400 |
| `products/` (34), `prds/` (10), `plans/` (11), `implementation-plans/` (1), `gtm/` (6), `investor/` (6), `governance/` + `governance-lanes/` (66), `quality/` (7), `audits/` + `audit/` (2), top-level `MASTERPLAN/PRD/ROADMAP/RISK-REGISTER/DESIGN/SPEC/GLOSSARY/DOC-CATALOG/AGENTS` | **`reference/` (generated mirrors) or `_project/`** | `Reference` (generated) / `Project` | `DOCUMENTATION.md:25` "fifth, project-management quadrant … project artifacts, not user docs." MASTERPLAN/PRD/products are GENERATED-REFERENCE per CC-3 (`00-REST-OF-DOCS-REGISTER.md:198`). |
| `decisions/` (355) | **`decisions/` — UNCHANGED, immutable** | `Decision` | ADRs = SSOT (FOUNDER DOCTRINE; `decision-record-oyatie-canon.md`). The brief mandates "decisions/ stays immutable." NOT a Diátaxis quadrant — its own namespace. |
| `user-journeys/` (1413), `personas/` (131), `user-stories/` (2) | **`reference/journeys/` (GENERATED from templates)** | `Reference` (generated) | `00-REST-OF-DOCS-REGISTER.md:198` classes journeys/personas as "acceptance-narrative layer — RE-GENERATE via templates." 1413+131 = 53% of the corpus; these MUST be generated, never hand-maintained (hyperscaler generated-registry rule). |
| `foundry/` (6) | **re-home per CC-1 sense-route** then to `reference/` or `explanation/` | (per content) | `00-REST-OF-DOCS-REGISTER.md:208` "products/foundry/supervisor/** … ORPHAN unless reachable" — sense-route foundry→intelligence\|governance FIRST (CC-1), then file. |
| `raw/` (5), `wiki/` (2), `site/` (9), `harness/` (1), `ci/` (1), `agents/` (11) | **triage: `explanation/` or ARCHIVE** | (per content) | `raw/`+`wiki/` are staging; total-accounting requires owner+reachability or archive (D-DOCTRINE). |

**Topology invariants (Linus no-special-cases):**
1. **Exactly one home per doc.** `doc-style.md:180` "## antipattern: Mixing Diátaxis quadrants in one doc — split into two docs." The gate (part d) rejects any doc reachable from two quadrant roots.
2. **`reference/` is build output.** Generated mirrors (MASTERPLAN, products README, journeys, machine-readable) are emitted by a generator from ADRs/specs, never hand-edited — mirrors the existing `automation/adr-index-pipeline.md:29` "ADR pack … is the source of truth … manual edits rejected" (`00-REST-OF-DOCS-REGISTER.md:36`).
3. **`decisions/` is the only namespace exempt from Diátaxis** — and it is the SSOT, so the exemption is principled, not a carve-out of convenience.
4. **Top level holds only the quadrant roots** (`tutorials/ how-to/ reference/ explanation/ _project/ decisions/`) + `README.md`. The 44→6 collapse is the measurable reorg outcome.

---

## (b) DRIFT RECONCILIATION

### Drift 1 — `axes_count: 6` → 7 (the founder's named live exhibit of false-state)
- **Stale source:** `source/docs/machine-readable/catalog.json:12` `"axes_count": 6,` with `:13` `"axes_v2_consolidation_note": "engineering-platform surfaces consolidated into Foundry on 2026-05-09"`.
- **Correct value, already elsewhere:** `source/docs/machine-readable/products.json` `"axes": 7` + `"added": "2026-05-09 (NEW axis)"` (Workspace); `DESIGN.md:17` "one product, **seven axes**"; `DESIGN.md:19` "Workspace / Productivity Platform added as Axis 2 on 2026-05-09." The 7 axes are enumerated in `DESIGN.md:23-31`: SaaS, **Workspace (NEW)**, Vertical, Foundry, Cloud, Search, Ads+analytics.
- **The drift is INTERNAL to the machine-readable mirror:** `catalog.json` (6) disagrees with `products.json` (7) and with `DESIGN.md` (7) — two generated mirrors of the same fact hold different values. This is precisely the hyperscaler anti-pattern the charter forbids: hand-maintained registries drift.
- **Fix (generated-not-hand-maintained):** `catalog.json._metadata.axes_count` and `products.json._metadata.axes` are BOTH GENERATED from a single source — the axis enum in `DESIGN.md §1` (or better, an ADR-0015 axis spec). Delete the hand-set integers; emit them. The gate (part d) RED-fixture: a `catalog.json` with `axes_count != count(axis enum)` must fail.
- **Note (not a contradiction):** the `prd-axis-coverage` PROSE check is already 7-aware — `DOC-CATALOG.md:306` "All **7 axes** appear in PRD §3." So the prose is correct and only the integer mirror is stale; this proves the value is knowable and should be generated.

### Drift 2 — competing fifth-quadrant name (`Project` vs `decision`)
- `DOCUMENTATION.md:25` calls it "project-management quadrant"; `planning-ssot-consolidation.md:134` calls the fifth home `decision`. **Reconcile:** they are TWO different things conflated — `Project` = governed project artifacts (PRD/ROADMAP/RISK) which are GENERATED-REFERENCE; `decision` = ADRs which are SSOT-immutable. Split them: `_project/` (generated) and `decisions/` (immutable). No competing scheme survives.

### Drift 3 — `doc-catalog` lane path bug (`docs/CATALOG.md` vs `docs/DOC-CATALOG.md`)
- **Bug, not fabrication:** `source/docs/governance-lanes/doc-catalog.md:8` "Verify every canonical doc has a row in `docs/CATALOG.md`"; `:13` `inputs: docs/CATALOG.md` — but the real file is `docs/DOC-CATALOG.md` (`docs/CATALOG.md` does not exist). Cross-confirmed `prelane-0.7/00-GOVERNANCE-BOOTSTRAP.md:20` "the doc-catalog lane spec reads `docs/CATALOG.md`, but the real file is `docs/DOC-CATALOG.md`." The active lane therefore reads a non-existent file and silently passes — a live false-enforcement (gate that claims to enforce but does not). **Fix:** repoint lane input to `docs/DOC-CATALOG.md`; RED-fixture = an uncataloged doc must FAIL (today it cannot, because the input is empty).

### Drift 4 — DOC-CATALOG authority inversion (CC-3) intersects the reorg
- `00-REST-OF-DOCS-REGISTER.md:35-37` (CC-3): `DOC-CATALOG.md:70` catalogs MASTERPLAN as hand-authored apex (`agent_authoring_allowed: NO`) while MASTERPLAN itself says it is a generated projection. The reorg **reclassifies MASTERPLAN + products/README → GENERATED-REFERENCE** and the unified record's `reachability` field encodes this (must trace to an ADR), closing CC-3 mechanically rather than by prose.

---

## (c) THE UNIFIED PER-DOC MACHINE-READABLE RECORD (generated-validated)

**One record per doc, carrying all three orthogonal attributes + ownership + reachability + TTL.** This SUPERSEDES the current `CatalogRow` (`doc-catalog.md:11`: `CatalogRow { path, doc_class, owner_axis, last_reviewed }` — which already fuses doc_class + owner_axis but lacks tier-as-data, reachability, and TTL). It also subsumes the `catalog.json` per-doc shape (`catalog.json:43-56`: `path/owner_team/tier/update_trigger/update_cadence/dependent_docs/validation_check/agent_authoring_allowed`).

**Schema (`docs/machine-readable/doc-record.schema.json`; one entry per doc in a generated `doc-records.generated.json`):**

```json
{
  "id":            "doc.<stable-id>",
  "path":          "reference/<...>.md",
  "doc_class":     "Tutorial | HowTo | Reference | Explanation | Project | Decision",
  "axis":          "saas | workspace | vertical | foundry | cloud | search | ads | cross-cutting",
  "tier":          "0 | 1 | 2 | 3 | cross-cutting",
  "owner_team":    "<teams/<id>/CHARTER.md id>",
  "reachability":  {
    "class":       "DECISION | INSTRUCTION | GENERATED-REFERENCE | ORPHAN",
    "source_ref":  "ADR-#### | session-context-bundle | <generator> | null",
    "in_masterplan": true
  },
  "generated":     true,
  "agent_authoring_allowed": true,
  "ttl": {
    "update_cadence": "per-change | quarterly | per-event",
    "last_reviewed":  "ISO-8601",
    "stale_after_days": 90
  },
  "validation_check": ["<lane-id>", "..."],
  "dependent_docs": ["doc.<id>", "..."]
}
```

**Field provenance (every field is an EXISTING attribute, unified — no invention):**

| Field | From which existing scheme | Evidence |
|---|---|---|
| `doc_class` | Diátaxis | `doc-style.md:54` "MUST declare its quadrant in frontmatter `doc_class:` (one of `Tutorial`, `HowTo`, `Reference`, `Explanation`)" |
| `axis` | 7-product-axis | `products.json` `"axis": "..."`; `DESIGN.md:23-31` axis table; closed enum of the 7 (+ `cross-cutting`) |
| `tier` | DOC-CATALOG tier | `catalog.json` per-doc `"tier": 0\|1\|2\|3\|"cross-cutting"` (e.g. `:46` `"tier": "cross-cutting"`, `doc.masterplan` `"tier": 0`) |
| `owner_team` | DOC-CATALOG | `DOC-CATALOG.md:0` reading-guide column `owner_team`; `catalog.json:45` `"owner_team"` |
| `reachability.class` | Total-accounting (charter) + WF2 | `00-REST-OF-DOCS-REGISTER.md:11` legend "DECISION→ADR · INSTRUCTION→session-context-bundle · GENERATED-REFERENCE→built-from-specs/ADRs · ORPHAN→not-needed/archive" |
| `reachability.in_masterplan` | masterplan SSOT rule | MEMORY masterplan-ssot rule: worth-documenting⇒reachable-from-masterplan-else-archive |
| `generated` | Diátaxis reorg + CC-3 | `planning-ssot-consolidation.md:111` "reference/ … generated … build output, never hand-edited" |
| `agent_authoring_allowed` | DOC-CATALOG | `catalog.json:56` `"agent_authoring_allowed": true`; `DOC-CATALOG.md:0` column |
| `ttl.*` | DOC-CATALOG cadence + freshness | `catalog.json` `"update_cadence"`, `"update_trigger"`; `doc-style.md:133` runbooks `last_verified`; staleness TTL (charter total-accounting) |
| `validation_check`, `dependent_docs` | DOC-CATALOG | `catalog.json:53-54` `"dependent_docs"`, `"validation_check"` |

**Generated-validated rule (robust-not-false):**
1. `doc-records.generated.json` is **emitted** by a generator that walks `docs/**`, reads each doc's frontmatter, and joins to the axis enum + tier table. It is NOT hand-edited (mirrors `products.json` `_schema.description` "Generated; do not hand-edit"). Hand-edit detection = the gate fails if the file's checksum diverges from a fresh regen (the `masterplan.generated.json` / `board-sync.generated.json` precedent already uses the `.generated.json` suffix convention in `machine-readable/`).
2. **One data structure, no special cases** (Linus): every doc — tutorial, runbook, ADR-index, PRD, journey — has exactly one record of the same shape. `decisions/` records carry `doc_class:"Decision"` and `reachability.class:"DECISION"` with `source_ref` = the ADR id itself.

---

## (d) THE GATE — advisory → BLOCKING (the three gates, proven by RED/GREEN)

**Verified current state (the founder's "defined-not-active / planned-not-blocking / unwired" exhibits — confirmed on disk):**

| Gate | Spec on disk? | In `lanes.yaml` active roster? | Severity | Real state |
|---|---|---|---|---|
| `diataxis-doc-class` | YES — `governance-lanes/diataxis-doc-class.md:5` "status: Accepted", `:21` "severity: MED", full Rust kernel sketch | **NO** — not found in `registry/quality/lanes.yaml` (grep returned only `oya-governance-doc-catalog` at `:44`) | MED (spec) | **PLANNED-NOT-BLOCKING** (matches `decision-record:181` "`diataxis-doc-class` planned-not-blocking") |
| `prd-axis-coverage` | **NO lane spec** — only a `validation_check` STRING: `DOC-CATALOG.md:306` "`prd-axis-coverage` \| All 7 axes appear in PRD §3 … no axis is absent"; referenced `catalog.json:207,462` | **NO** | none (no runner) | **DEFINED-NOT-ACTIVE** (matches `decision-record:181` "`prd-axis-coverage` defined-not-active") |
| `doc-catalog` | YES — `governance-lanes/doc-catalog.md:6` "status: Accepted", `:21` "severity: BLOCKER" | **YES** — `lanes.yaml:44` `id: oya-governance-doc-catalog`, `:51` `check_command: … gate validate doc-catalog` | BLOCKER | **ENFORCED but BROKEN** — reads `docs/CATALOG.md` (non-existent) per `doc-catalog.md:13`; checks row-coverage only, NOT the unified triple |

**Promotion plan (turn all three blocking, each backed by a RED fixture that proves it actually blocks):**

1. **`oya-governance-doc-record` (NEW unifying lane — replaces/extends `doc-catalog`).** Validates that every doc in `docs/**` has exactly one record in `doc-records.generated.json` carrying ALL of `{doc_class, axis, tier, owner_team, reachability.class, ttl.last_reviewed}` non-null.
   - Add to `lanes.yaml` with `status: active`, `stage: foundation`, `severity: BLOCKER`, `check_command: cargo run -p oya-dev-cli -- gate validate doc-record`.
   - **Fix the path bug first:** input = `docs/DOC-CATALOG.md` + `doc-records.generated.json`, NOT `docs/CATALOG.md`.
   - **RED fixture:** a doc with no record, or a record missing `axis`, MUST exit non-zero. **GREEN fixture:** fully-recorded doc passes. (Today the broken `doc-catalog` lane passes a RED input because it reads an empty file — the new lane must demonstrably fail it.)

2. **`oya-governance-diataxis-doc-class` → active + BLOCKER.** Move the existing kernel (`oya-governance-diataxis-doc-class-kernel`, `diataxis-doc-class.md:11`) from spec-only into `lanes.yaml`; raise severity MED→BLOCKER after a soak (use the existing `report-only@day-0 → error@day-8` ratchet precedent, `prelane-0.7:18`).
   - **RED fixture:** the spec's own `failure_modes` (`diataxis-doc-class.md:15`): "declared `tutorial` but no `Steps` section"; "`reference` doc contains opinion/narrative"; "declared class unknown" — each must block.
   - Also enforces the topology invariant: a doc reachable from two quadrant roots = `UnknownClass`/duplicate-home failure (closes `doc-style.md:180` anti-pattern mechanically).

3. **`oya-governance-prd-axis-coverage` (NEW lane — give the orphan check a runner).** Author the lane spec under `governance-lanes/` + the kernel, then add to `lanes.yaml`, `status: active`, BLOCKER. Check = `DOC-CATALOG.md:306` verbatim: all 7 axes (from the GENERATED axis enum, NOT the stale `axes_count:6`) appear in PRD §3 or §3.2.
   - **RED fixture:** a PRD missing the `workspace` axis (the 7th, newest) MUST fail. **GREEN:** all 7 present. This simultaneously closes Drift 1 (the check sources the live enum, so a stale `axes_count` cannot pass).

4. **`axes_count` generation guard (folds into `doc-record` lane).** RED fixture: `catalog.json.axes_count != len(axis enum)` fails; the value is regenerated, never hand-set. Closes the founder's named exhibit.

**Robust-not-false acceptance bar (D-DOCTRINE):** none of the four is "wired" until its RED fixture is committed and shown to exit non-zero in CI, AND it appears in `registry/quality/lanes.yaml` with `status: active` (the real roster — `prelane-0.7:24` "the real machine floor … is `oya verify --ci-required`"). A lane spec that exists in `governance-lanes/` but not in `lanes.yaml` is, by the evidence above, advisory theatre — exactly what the charter forbids. Also wire the 22 unwired `oya-governance-*` crates (`decision-record:181`) that these lanes depend on into the roster, or the gates cannot run.

---

## RETURN DIGEST
(below is the compact return value for the orchestrator)
