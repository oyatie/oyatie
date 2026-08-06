# FOUNDRY-PROSE-SCRUB-MAP — lane S3 adjudication (read-only)

**Lane:** S3 — bare-"foundry" prose eradication ADJUDICATION (ZERO mutation; map only).
**Source spine (read-only):** `/Users/jasonlee/Developer/source` @ `d96898239aa0ef5b03860574515eafd1c716131c`, branch `cleanup/whole-tree-2026-06-07`.
**Session repo (NOT mutated):** `/Users/jasonlee/Developer/linux` (the linux port — a different repo).
**This file is the only write.** No source mutation, no push, no `git add -A`.

---

## 0. Scope and exclusions

Subject = the **bare word `foundry`** (case-insensitive) in PROSE / paths / milestone-dir-names.
This is the residue AFTER the already-completed campaigns: the `oya-foundry-*` IDENT campaign, `oya-vcs`, and the 15 ADR-slug renames. Those are DONE and OUT OF SCOPE here.

**Exclusions applied (per lane brief):**
- `*.generated.json` — spine regenerates; never hand-edit. Excluded from every count.
- The **6 risky ADR slugs deferred to founder**: `0335 / 0347 / 0363 / 0374 / 0377 / 0387`. Lines referencing these slugs (`ADR-0335…`, `adr_0347:`, `foundry-fitness-to-governance`, etc.) are EXCLUDED from class A (treated as defer/keep-context).
- **Palantir-Foundry carve-out** — any line matching `/palantir/i` (307 lines). Excluded.
- Already-handled `oya-foundry-*` / `oya_foundry_*` idents (373 lines) — out of scope; only their *non-ident* siblings within the same files are considered.

**Counting note (HONEST):** the bare word is endemic — it appears on **19,218 in-scope line-occurrences** across **3,181 files** (after excluding generated.json + palantir). The vast majority are a *small set of recurring strings* (event surfaces `foundry.x.y`, `context: foundry` lane tags, `axis-foundry` team brand, `FOUNDRY_*` proto enums, `foundry-runtime` backend) replicated across the ~150-product fleet — NOT 19k distinct prose sentences. Per-class line counts below are the adjudication unit; per-path tables (A, C) are file/family-level.

---

## 1. Per-class counts (the adjudication)

| Class | Meaning | Disposition | Line-occurrences | Distinct files |
|---|---|---|---|---|
| **A** | LIVE / forward-looking surface still carrying `foundry` (prose, idents, tags, event namespaces) that **contradicts ADR-0335 retirement** | **SCRUB → intelligence / governance** per context | **14,228** | 2,033 |
| **B** | HISTORICAL ADR / evidence bodies + retirement-context cross-refs where the vocab IS the subject | **KEEP** | **3,419** (2,247 bodies + 1,172 retirement-context refs) | ~430 |
| **DEFER** | Lines referencing the 6 founder-deferred ADR slugs (0335/0347/0363/0374/0377/0387) | **DEFER to founder** (excluded from A; folded into B-keep until slugs ruled) | **1,186** | overlaps B |
| **C** | Paths (dirs/files) whose NAME contains `foundry` | **RENAME path + repoint** (live) / **KEEP** (legacy archive) | **181 paths** | 181 |
| **D** | `.omc/` working-state | **NOTE only** (working state, not shipped corpus) | **1,362** | 199 |

`19,218 = A 14,228 + B-bodies 2,247 + B-retirement-refs 1,172 + DEFER 1,186 + D(.omc, non-overlapping share) 1,362` (residual reconciles via files that span buckets; A excludes all DEFER lines — overlap verified = 0).

**Key fact driving A-vs-B:** `foundry` is **RETIRED** (ADR-0335 "foundry µservice retired; absorbed by intelligence", Accepted 2026-05-21; ADR-0363 retires agentic-vcs foundry→intelligence). Yet LIVE artifacts still ship the brand: `docs/MASTERPLAN.md` lists `foundry` as a parallel-lane microservice; `docs/prds/foundry.md` (status Accepted, `microservice: foundry`); `docs/products/foundry/PRD.md` (status Draft, owner `axis-foundry`); 36 `docs/runbooks/foundry-*` (doc_status published/stub); 253 `registry/catalog/*` tagged `context: foundry`; live event surfaces `foundry.evidence.emit` / `foundry.autonomy.decision` / `foundry.run.start`. These are class A: forward-looking and must be repointed to **intelligence** (runtime/eval/rag/registry/providers) or **governance** (CI lanes/fitness) per local context.

---

## 2. Class A — LIVE forward-looking surface to SCRUB (per recurring-surface table)

Class A is best operated on by recurring SURFACE TYPE (the same string repeated across the fleet), not 14k individual lines. Each row = one scrub motion.

| # | Surface (string pattern) | Occurrences | Where | Proposed scrub → | Repoint note |
|---|---|---|---|---|---|
| A1 | `foundry.<verb>.<noun>` dotted event/action surfaces (`foundry.evidence.emit`, `foundry.autonomy.decision`, `foundry.run.start`, `foundry.capability.invoke`, `oya.foundry.providers.*`) | 1,444 (`foundry\.[a-z]`) | oya/application, oya/intelligence (contracts asyncapi/proto/openapi, src, tests) | `intelligence.<…>` event namespace (per ADR-0335 D-clause "retired foundry namespace → intelligence event namespace") | HIGH coupling — event-name change ripples to emitters+consumers+contracts; do as one rename motion |
| A2 | `FOUNDRY_<CONST>` proto3 / enum constants (e.g. `FOUNDRY_SUPERVISOR = 3`) | 1,120 | oya/intelligence/contracts/proto, asyncapi | `INTELLIGENCE_…` / drop `FOUNDRY_` prefix | proto enum value renames = wire-compat review (keep tag numbers) |
| A3 | `axis-foundry` team brand (`owner_team: axis-foundry`, charters, IP owners) | 1,127 (225 are `owner_team:`) | fleet-wide registry/catalog, runbooks, IPs, docs/teams/axis-foundry | `axis-intelligence` (team rename) | team-rename; pairs with C6 dir rename `docs/teams/axis-foundry/` |
| A4 | `context: foundry` lane-context tag | 253 (1 per file) | `registry/catalog/*.yaml` | `context: intelligence` (or `governance` for `oya-governance-*` lanes) | machine-readable governance metadata; bulk one-per-file |
| A5 | `oya.foundry` / `oya/foundry` namespaced refs (`oya.foundry-guardrails.*`, `foundry/prod-rollout-gate`, `/foundry/providers`) | 262 | oya/intelligence (IP journeys, capabilities, helm routes) | `oya.intelligence…` / `intelligence/…` route | path-style + evidence-topic idents |
| A6 | `foundry-eval` / `foundry-providers` / `foundry-runtime` / `foundry-supervisor` context tags (build/catalog) | ~440 (`foundry-runtime`) + 301 (`foundry-supervisor`) + 359 (capability/eval/policy/rag/registry) | oya/intelligence catalog, bc-sources, oya/translate catalog backends | `intelligence-eval` / `intelligence-runtime` / `intelligence-supervisor` | A6 overlaps C-path families (templates/foundry-supervisor, foundry-runtime backends) |
| A7 | Proper-noun `Foundry` brand in live prose (`Foundry as oyatie.foundry.* principals`, manifest rationale "…and foundry (vector retrieval per ADR-0192)", PRD/runbook bodies) | 1,934 (`\bFoundry\b`) | docs/products/foundry, docs/prds/foundry.md, docs/runbooks/foundry-*, docs/personas, docs/user-journeys, oya/*/manifest.json | "intelligence" (vector-retrieval / hosted-agent context) | judgement per sentence; preserve meaning, drop brand |
| A8 | `docs/MASTERPLAN.md` lists `foundry` as a live microservice lane (§ line 60, §104 parallel lanes) | 2 | docs/MASTERPLAN.md | remove/relabel `foundry` lane → `intelligence` | MASTERPLAN is reachability SSOT — must not advertise a retired brand |
| A9 | `oyatie.foundry.<role>` Cedar ServicePrincipal idents (`oyatie.foundry.cell-orchestrator`) | 413 | oya/*/cedar/policies.cedar (fleet), docs/policies | `oyatie.intelligence.<role>` principal | **ADJUDICATION FLAG:** this is a *namespaced principal ident* — may belong to the IDENT campaign, not bare-prose. Recommend founder confirm whether `oyatie.foundry.*` principals were in-scope of the completed ident campaign; if NOT, scrub here. Do NOT silently rename auth principals without a policy-eval gate. |

**Class A primary files (forward-looking docs that MUST be scrubbed, not just idents):**

| File | Disposition | Why class A |
|---|---|---|
| `docs/MASTERPLAN.md` | scrub lane name | advertises retired `foundry` as live parallel lane |
| `docs/prds/foundry.md` | scrub body + rename file (see C) | status Accepted, `microservice: foundry` — live PRD for a retired service |
| `docs/products/foundry/PRD.md` + tree (28 files) | scrub body + rename tree (see C2) | status Draft, owner axis-foundry — live product spec |
| `docs/checklists/foundry-capability-publishing.md` | scrub + rename | live capability-publishing checklist |
| `docs/architecture/foundry-fitness-to-governance-transition-2026-05-21.md` | **DEFER** (slug 0347 family) | transition doc tied to deferred slug — keep as context |
| `docs/governance-lanes/foundry-corpus-citation.md`, `.omc/fitness-lanes/foundry-corpus-citation.md` | scrub + rename | live governance-lane spec |
| `oya/*/manifest.json` "rationale …foundry (vector retrieval per ADR-0192)" (fleet) | scrub prose | live architectural description naming retired service |
| `oya/intelligence/IP-journey-j41-prod-rollout-gate.md` | scrub `foundry/prod-rollout-gate` refs | live IP journey using foundry path-idents |

---

## 3. Class B — HISTORICAL bodies (KEEP; vocab is the subject)

KEEP unchanged — editing these would erase the retirement record.

| Family | Files / lines | Why KEEP |
|---|---|---|
| `docs/adr-archive/ADR-0335-intelligence-microservice-consolidation.md` | body | the retirement decision itself; **slug = DEFER (0335)** |
| `docs/adr-archive/ADR-0347-governance-fitness-bulk-rename.md` | body | bulk-rename decision; **slug = DEFER (0347)** |
| `docs/adr-archive/ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate.md` | body | vcs retirement; **slug = DEFER (0363)** |
| `docs/decisions/**` other ADR bodies citing foundry | 1,875 lines / 212 files | historical cross-refs documenting retirement chain |
| `evidence/**`, `registry/check-empirical-evidence/**` | 343 lines | acceptance/empirical evidence of the retirement (`2026-05-16-m02-exit-on-prem-foundry-live.json`, `score-card-…-foundry-pr126.json`) |
| `oya/intelligence/_legacy-foundry/` (3 files: README.md, manifest.json, scorecards/overrides.json) | KEEP (class B by design) | `_legacy-` prefix already marks it as a retirement archive; 0 content refs; dated 2026-05-21. The foundry name IS the archived subject. (Path is NOT a class-C rename target.) |
| `oya-governance-no-foundry-fitness-residue` guard-lane name (826 lines) | KEEP as guard | a LIVE governance lane whose JOB is to grep-forbid the retired vocab; the retired word in its name is intentional (the thing it guards). Confirm with founder it stays under the deferred-0347 umbrella. |
| Retirement-context cross-refs (`retired`/`absorbed`/`former`/`legacy`/`superseded` + foundry) | 1,172 lines | prose that explicitly frames foundry as past |

---

## 4. Class C — foundry-NAMED paths (rename + repoint). 181 tracked paths.

Path families. "Repoint" = files containing the family path string in CONTENT (excl generated.json).

| # | Path family | Files in tree | Proposed new path | Repoint surface (content refs) | Disposition |
|---|---|---|---|---|---|
| C0 | **`.omc/plans/milestones/M02-foundry-preview/`** | **36** | `M02-intelligence-preview/` (founder confirm milestone label) | **24 files / 253 line-refs** outside the tree | **RENAME — see §5 blast radius** |
| C1 | `docs/products/foundry/` (incl. supervisor/* adapters/kernels) | 28 | `docs/products/intelligence/` | 58 files | rename + repoint (live product tree) |
| C2 | `docs/runbooks/foundry-*` + `docs/runbooks/foundry/` subdir | 24 | `docs/runbooks/intelligence-*` + `docs/runbooks/intelligence/` | 13 files | rename + repoint (live runbooks) |
| C3 | `contracts/openapi/foundry/` (capability/eval/policy/rag/registry v1 +meta) | 10 | `contracts/openapi/intelligence/` | 39 files | rename + repoint (live contracts) |
| C4 | `oya/intelligence/contracts/{asyncapi,openapi,proto}/*foundry*` (evidence/runtime/supervisor) | 9 | drop `foundry` infix → `…-evidence`, `…-runtime`, `…-supervisor` | within oya/intelligence | rename + repoint (pairs with A1/A2) |
| C5 | `templates/foundry-supervisor/` (claude/codex/gemini.toml) | 3 | `templates/intelligence-supervisor/` | 13 files | rename + repoint |
| C6 | `docs/teams/axis-foundry/CHARTER.md` | 1 | `docs/teams/axis-intelligence/` | 7 files | rename + repoint (pairs with A3) |
| C7 | `registry/capabilities/foundry-internal.json`, `foundry-supervisor.toml` | 2 | `intelligence-internal.json`, `intelligence-supervisor.toml` | 6 files | **WARN:** `registry/` may be untracked store territory per guardrails — verify these 2 are TRACKED before any motion |
| C8 | `oya/translate/catalog/*-foundry-runtime.yaml` (langdetect/qe/stream adapters) | 3 | `*-intelligence-runtime.yaml` | within oya/translate | rename + repoint backend `foundry-runtime` → `intelligence-runtime` (A6) |
| C9 | `docs/foundry/` (supervisor docs + checkpoint) | 6 | `docs/intelligence/` (or fold into C1) | 16 files | rename + repoint |
| C10 | `docs/prds/foundry.md` | 1 | `docs/prds/intelligence.md` | 8 files | rename + repoint (live PRD, also class A body) |
| C11 | `specs/microservices/foundry.json`, `specs/design-system/foundry-agent-run-timeline.json` | 2 | `intelligence.json`, `intelligence-agent-run-timeline.json` | 6 files | **WARN:** check if `.json` here regenerates (NOT *.generated.json, so likely hand-tracked) |
| C12 | `oya/**/tests/foundry_*.rs`, `oya/developer-sdk/.../src/foundry_*.rs`, `libs/.../foundry_eval_run_api.rs` | ~16 test/src files | `intelligence_*.rs` test names | low external repoint (test fns) | rename test/src files (pairs with A) |
| C13 | `docs/policies/foundry-supervisor.cedar`, `docs/decisions/templates/foundry-phase00-template.md`, `scripts/validate-foundry-phase00-evidence.mjs`, `docs/checklists/foundry-capability-publishing.md`, `docs/governance-lanes/foundry-corpus-citation.md` | ~5 | `intelligence-*` equivalents | per-file | rename + repoint (live policy/template/script) |
| C14 | `docs/decisions/ADR-0335…md`, `ADR-0347…md`, `ADR-0363…md` (filenames carry foundry) | 3 | **DO NOT RENAME** | — | **DEFER** — these are 3 of the 6 founder-deferred slugs; filename rename = slug rename = founder gate |
| C15 | HISTORICAL path families (KEEP): `oya/intelligence/_legacy-foundry/` (3), `.omc/archive/**/ralplan-foundry-*` (7), `docs/raw/agentic-delivery-foundry-critical-challenge.md`, `docs/localization-packs/kr/evidence/foundry.md` | — | KEEP | — | class B-by-design (legacy/archive/raw/evidence) |

**Class C live-rename paths (excl DEFER C14 + KEEP C15) ≈ 141 paths** across families C0–C13.

---

## 5. M02-foundry-preview directory rename — BLAST RADIUS (flagged)

`/.omc/plans/milestones/M02-foundry-preview/` is the single largest path-rename hazard.

- **Tree size:** 36 tracked files (6 phases P00–P06 + INDEX + IP-*).
- **External repoint surface:** **24 files** reference the literal `M02-foundry-preview` path string, **253 line-refs** total (excl generated.json), spread across:
  - `evidence/goals/` (4), `evidence/ledger/`, `evidence/audits/` — milestone-exit evidence
  - `specs/masterplan.json`, `registry/milestone-audit/`, `registry/fixuptasks.jsonl`, `goal.json` — machine-readable milestone registries
  - `.omc/archive` (7), `.omc/scratch`, `.omc/handoffs`, `.omc/automation` — working state
  - `docs/automation/`, `docs/AGENT-INSTRUCTION-SOURCES.md`
  - `oya/developer-sdk/` (2)
- **Hazards:**
  1. `git mv` of 36 files + 253 string-repoints in 24 files; the milestone *label* (`M02-foundry-preview`) is itself referenced as a goal/milestone KEY in machine-readable registries — a rename must update those keys atomically or the milestone-audit/masterplan spine breaks.
  2. `specs/masterplan.json` + `registry/*` may be **spine-generated faces** — confirm they are not `*.generated.json` (they are not by name, but verify the regeneration source) before hand-editing; if generated, the rename must flow through the producer, not the face.
  3. `M02b-substrate` sibling milestone has `P01-foundry-engine-consolidation/` (3 files) + `acceptance-evidence/2026-05-16-m02-exit-on-prem-foundry-live.json` — a separate but adjacent rename; `M02b` evidence is class-B historical (KEEP) while `M02-foundry-preview` (forward milestone) is the rename target. Do NOT conflate the two.
  4. Proposed label `M02-intelligence-preview` is a **founder-gate decision** (milestone naming = roadmap surface). Recommend NOT auto-renaming; surface to founder with this blast-radius table.

**Recommendation:** treat M02 dir rename as its own gated motion (founder-go), executed as: `git mv` tree → repoint 24 files (explicit paths, `git add -u`) → regenerate any spine face → verify 0 dangling via BOTH `git grep -F M02-foundry-preview` AND `git ls-files | grep -F M02-foundry-preview`.

---

## 6. Class D — `.omc/` working-state (NOTE only)

- **1,362 line-occurrences across 199 `.omc/` files.** Working state, not the shipped corpus.
- Families: `.omc/advanced-cicd/{branch-pipeline/foundry-pipeline-mirror.md, progressive-delivery/playbook-foundry.md}`, `.omc/fitness-lanes/foundry-corpus-citation.md`, `.omc/plans/milestones/{M02-foundry-preview, M02b-substrate/.../P01-foundry-engine-consolidation}/`, `.omc/scratch/foundry-salvage-*`, `.omc/archive/**/ralplan-foundry-*` (7, historical).
- **Disposition:** This is the **Batch-7** territory of the foundry-vocab eradication (AP2). NOTE: `.omc/archive/**` is historical (leave); `.omc/plans/milestones/M02-foundry-preview/` is the working twin of C0 and rides the same gated rename; the rest are working scratch that can be scrubbed opportunistically but are NOT shipped-corpus blockers. Do not block the corpus scrub on `.omc/` state.

---

## 7. Adjudication flags for founder / next lane

1. **A9 `oyatie.foundry.*` Cedar principals (413 lines):** confirm whether the completed ident campaign already covered these auth principals. If not, scrub — but gate behind a Cedar policy-eval test (renaming an authz principal is behavior-affecting, not prose).
2. **B `oya-governance-no-foundry-fitness-residue` guard-lane (826 lines):** the retired word lives in the NAME of the lane that forbids the retired word. Keep, but confirm it falls under the deferred-0347 umbrella so the residue-guard and the slug move together.
3. **C7 / C11 registry+specs `.json`:** verify TRACKED + non-generated before any motion (guardrail: don't touch untracked stores; don't hand-edit generated faces).
4. **C14 + DEFER:** the 3 foundry-named ADR filenames (0335/0347/0363) and all 1,186 deferred-slug lines stay FROZEN pending founder slug-ruling.
5. **Sequencing:** A1/A2/A4 (event surfaces + proto enums + lane tags) are the highest-coupling motions; run them as atomic rename passes with contract/wire-compat gates, not free-text sed.

---

## 8. Verification performed (read-only)

- Source HEAD pinned: `d96898239` ✓ (matches lane SHA).
- Dangling-readiness checks use BOTH content (`git grep -F`) AND path (`git ls-files | grep -F`) — demonstrated for M02 family (§5) and all C families (§4 repoint column).
- defer-slug ↔ class-A overlap = **0** (verified: deferred-slug lines fully excluded from the A bucket).
- `_legacy-foundry/` content-refs = 0 (confirms it is a dead archive → class B-by-design, not a rename target).
- No mutation of `/Users/jasonlee/Developer/source`. Only write = this file.
