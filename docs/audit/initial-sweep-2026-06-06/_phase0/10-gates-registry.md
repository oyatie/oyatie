---
title: "PHASE-0 — GENERATED ACCOUNTING-REGISTRY + 4 KEYSTONE GATES (concrete design with RED/GREEN)"
lane: _phase0 / 10-gates-registry
charter: D-SEQUENCE (FIREWALL-FIRST) · D-CICD (oya-ci/oya-cd bespoke-Rust, adopts Prow+Tekton+Argo patterns) · D-DOCTRINE (maintainable-by-enforcement, total-accounting, robust-not-false)
date: 2026-06-06
mode: READ-ONLY audit artifact (no source/audit file edited; this is the only file written)
status: DESIGN-NOT-BUILD (concrete enough to implement; nothing here is claimed shipped)
distinguishes: WHAT EXISTS (read from source, cited) vs WHAT TO BUILD (net-new, marked [BUILD])
---

# 10 — ACCOUNTING-REGISTRY SCHEMA + 4 KEYSTONE GATE CRATES

This artifact takes the prior DESIGN lane (`justify-account-robustness/10-enforcement-primitives.md` + `10-total-accounting.md` + `10-robustness-enforcement.md`) from "four crisp designs" to **one concrete generated-registry schema + four implementable gate-crate specs**, each with its REQUIRED RED fixture (known-bad input it MUST fail) + GREEN fixture (known-good it passes), grounded in exhibits re-verified live on disk 2026-06-06. Per D-SEQUENCE: these are the Phase-0 firewall that makes enforcement REAL before any Phase-1 canon amendment, so amendments are gate-verified and cannot re-drift.

---

## 0. GROUND TRUTH — WHAT EXISTS vs WHAT TO BUILD (re-verified live 2026-06-06)

**EXISTS on disk (the seed substrate this design extends, NOT reinvents):**
- `source/specs/phase0-automation-matrix.json` — Gate-4's seed contract; `_meta.status: "seed-contract-not-green"`; `gate_contract.id: "cloud-ci-automation-ratchet"`; `required_row_fields` (10 fields); `classifications` (4-enum); 9 `seed_rows`. (read in full)
- `source/specs/fixtures/phase0-automation-ratchet/` — **4 live fixtures** (1 GREEN, 3 RED). Their on-disk `expected_violations` codes are the AUTHORITY (see §C below — they differ from the prior design doc's invented names; I reconcile to the real codes).
- `source/specs/phase0-claim-evidence-map.json` (9996 B) + `source/specs/merge-queue-parked-pr.json` — registries Gate-1/Gate-4 consume. (existence confirmed)
- `source/specs/fixtures/phase0-claim-ceiling/` (5 fixtures) + `phase0-ci-enforcement-baseline/` + `phase0-exit-gate/` — the on-disk fixture convention template (`expected_verdict` + `expected_violations` + `rows[]` + `source_test`). (confirmed)
- **Live drift exhibits (RED-fixture seeds), each re-read this session:**
  - `docs/machine-readable/catalog.json:12` = `"axes_count": 6,` **vs** `docs/machine-readable/contracts.json:9` = `"axes_count": 7,` (generated-face drift — verbatim, both files).
  - `docs/adr-archive/ADR-0377-github-board-git-ref-cas-fallback.md` **and** `docs/adr-archive/ADR-0377-github-board-git-ref-cas-fallback.md` — TWO files, one number 0377 (dual-decision collision).
  - `ADR-0511-...md:11` `supersedes: [ADR-0359]`, `:12` `superseded_by: []` (supersession recorded one-directionally; reciprocal flip on ADR-0359 is a *pending* line, `:105`).
  - `ADR-0365-...md` `verified_by: "oya gen propagate --check"` (`:26`), `"oya gate validate propagation-drift"` (`:30`), ×5 more `oya gate`/`oya gen` — oya-CLI-authority defect, the exact thing register #20 forbids.
  - `find -name OWNERS` (excl target/.git) = **0** tree-wide (the O-1 systemic owner gap → Gate-2 `unowned` is born blocking).

**TO BUILD [BUILD] (net-new; confirmed absent in source — grep for `cloud-ci-cross-artifact` / `cloud-ci-total-accounting` / `cloud-ci-staleness` / `accounting-registry` = 0 hits):**
- `accounting-registry.generated.json` schema + producer (this artifact §A).
- 3 net-new gate crates `cloud-ci-cross-artifact-agreement`, `cloud-ci-total-accounting`, `cloud-ci-staleness-reaper` (§B Gates 1/2/3).
- The hardening of the EXISTING `cloud-ci-automation-ratchet` seed into a wired gate (§B Gate 4).
- 3 net-new fixture dirs `specs/fixtures/cross-artifact-agreement/`, `specs/fixtures/total-accounting/`, `specs/fixtures/staleness-reaper/` + 2 net-new ratchet fixtures.
- buck2 wiring (§D) — no source `BUCK` target for any of these exists yet.

**Charter anchors (cited):** register #20 automation-ratchet (`backlog:661-668`, "anything enforceable or automatable must be enforced/automated … New `oya` CLI commands are forbidden"); §P cross-artifact-agreement (`backlog:522`, "they must ALL AGREE … AUTOMATED + ENFORCED (a cross-artifact-agreement gate), not hand-maintained"); pillar-G accounted-GC (`backlog:302-312`, "every resource ACCOUNTED (owner + TTL + reaping) … a CI hygiene GATE that BLOCKS if accounting drifts"); G-INTEGRITY ships Phase-0 with NO buck2-build-graph dep (`backlog:341`); register #21 claim-ceiling ("mechanically enforced means a required cloud-ci context plus RED/GREEN fixtures, not local `oya` output", `backlog:679`).

---

## A. THE GENERATED ACCOUNTING-REGISTRY SCHEMA  (`accounting-registry.generated.json`)  [BUILD]

**One generated record per VCS-tracked path** (`git ls-files`, the tracked-truth discipline; worktrees/`target/`/`buck-out/` are runner-local, accounted by `resource_class` not by row). This is the ONE good data structure; Gates 1/3/4 are predicates/views over it (Gate-2 owns + produces it). Regenerated every run from `git ls-files × OWNERS × ADR-front-matter × specs × masterplan`.

### A.1 Per-record schema (the 11 task-required fields, made concrete)

| field | type | meaning | generated-from (source-of-truth) | gate that consumes |
|---|---|---|---|---|
| `path` | string | repo-relative path of the accounted unit | `git ls-files` | all |
| `unit_class` | enum | `code` \| `doc` \| `spec` \| `registry` \| `evidence` \| `vendor` \| `build_config` \| `generated` \| `ephemeral` \| `husk` | classifier (path-glob + content sniff) | Gate-2 (#1), Gate-3 (TTL-class map) |
| `owner` | string\|null | OWNERS-resolvable team id (nearest `OWNERS` up-tree); `null` ⇒ Gate-2 RED | nearest `OWNERS` file (must be created — closes O-1) | Gate-2 (#2 unowned) |
| `justification_ref` | string\|null | the decision/need it traces to: `ADR-####` \| spec `$id` \| `need:<ticket>` | ADR front-matter `affected_surfaces` + spec back-refs | Gate-2 (#3 unjustified) |
| `reachable_from` | string[] | registries that point to it: `masterplan` \| `root-hub` \| `cargo-members` \| `doc-catalog` \| `crosswalk` | masterplan.json / root-hub-pointers.json / Cargo.toml members / DOC-CATALOG / Gate-1 crosswalk | Gate-2 (#4 unreachable) |
| `ttl` | object | `{ttl_class, budget_days, action: report\|archive\|delete, protected: bool}` | `ttl-policy.generated.json` keyed by `unit_class`/`resource_class` | Gate-3 (#1/#2), Gate-2 (#5 no-ttl) |
| `last_touch_commit` | string | last commit SHA + author-date touching `path` | `git log -1 -- <path>` | Gate-3 (staleness signal) |
| `tracked` | int | git-tracked descendant file count (for tree-rows), excl `target/` | `git ls-files \| count` | Gate-2 (husk = 0-tracked ⇒ ARCHIVE) |
| `verdict` | enum | auto-derived: `KEEP` \| `ARCHIVE` \| `MERGE` \| `NEEDS-OWNER` \| `RED` | rules over the above | Gate-2 final; Gate-3 archive |
| `dup_of` | string\|null | if duplicate, the canonical path it must merge into | dedup index (basename+slug+content-hash) | Gate-2 (MERGE verdict) |
| `_provenance` | object | `{generated_at, producer_target, source_inputs_digest}` — proves regeneration | producer | all (drift assertion) |

### A.2 Registry-level invariants (the keystone contract, all gate-checked)

1. **`committed == regenerated`.** Every gate's FIRST assertion: re-run the producer, byte-compare against the committed `accounting-registry.generated.json`. Mismatch ⇒ RED `registry_drift`. **A hand-edit to a generated file is itself a RED** — drift is structurally impossible because the only legal way to change a generated face is to change source + rerun. (the ADR-0365 `committed == regenerated` discipline, `ADR-0365:28`, but producer moved OFF `oya` CLI per register #20.)
2. **Total coverage.** `set(registry.path) == set(git ls-files) − (gitignored ∪ resource_class∈{ephemeral})`. A tracked path with no row ⇒ Gate-2 RED `unaccounted`. No file escapes.
3. **No special cases in the scanner.** `vendor`/`generated`/`ephemeral` carve-outs live as DATA in the `unit_class`/`ttl-policy` tables, never as ad-hoc ignore-list code (Linus: the exception lives in the table).
4. **Producer is a buck2 `rust_binary`, never an `oya` CLI command** (register #20; retires the ADR-0365 `oya gen`/`oya gate` authority defect).

### A.3 Companion generated faces (also `committed == regenerated`)
- `decision-crosswalk.generated.json` [BUILD] — Gate-1's join: one row per ADR `{adr_id, status_3axis, spec_ids[], masterplan_node, roadmap_node, affected_surfaces[], superseded_by[], supersedes[]}` from ADR front-matter.
- `ttl-policy.generated.json` [BUILD] — Gate-3's policy: `ttl_class → {budget_days, action, protected}` from the pillar-G resource-class table (`backlog:304-310`).
- `enforcement-inventory.generated.json` [BUILD] — Gate-4's join: one row per enforcement *claim* in the corpus.

---

## B. THE FOUR GATE CRATES (each: inputs · blocking semantics · generated-vs-committed · RED fixture · GREEN fixture)

Naming: all four are `rust_test` crates under the `cloud-ci-*` namespace (Gate-4's id `cloud-ci-automation-ratchet` already exists on disk in the matrix; the other three follow it). Each emits `{verdict: RED|GREEN, violations: [code,…]}` and its own test asserts `assert_eq!(report.violations, fixture.expected_violations)` per fixture. **Bootstrapping honesty:** a gate's matrix classification stays `automated_advisory_until_p0_0` until its self-test reproduces its live exhibits as RED on the current corpus; only then may it flip to `automated_blocking_now` (register #21; Gate-4 polices this on Gates 1-3).

---

### GATE 1 — `cloud-ci-cross-artifact-agreement`  [BUILD]
*Charter: §P (`backlog:522`); the gate whose ABSENCE let two consensus bodies drift (`decision-record:166`). Amends ADR-0365 (de-dup O2).*

**Inputs:** ADR front-matter (`docs/decisions/ADR-*.md`, SSOT) · `specs/*.json` contracts · `specs/masterplan.json` (generated projection) · roadmap (`master-plan-sequencing.json`) · the generated `decision-crosswalk.generated.json`.

**Blocking semantics (RED + violation code):**
| # | code | RED when |
|---|---|---|
| 1 | `orphan_decision` | ADR `Accepted` but lacks ≥1 of {spec, masterplan node, roadmap node} |
| 2 | `unpropagated_decision` | a spec/masterplan/roadmap node references a decision id with no live ADR (dangling edge) |
| 3 | `status_disagreement` | 3-axis status differs across the four faces (ADR `Accepted` vs masterplan `proposed`) |
| 4 | `generated_face_drift` | committed generated face ≠ regenerated (the `axes_count 6 vs 7` class) |
| 5 | `dual_decision_collision` | two ADR files share one number, OR two decisions claim one masterplan node (the dup-0377 class) |
| 6 | `supersession_half_edge` | `superseded_by` set on one side without reciprocal `supersedes` on the other (the 0511↔0359 class) |

**Generated-vs-committed:** crosswalk + masterplan + ADR-INDEX are 100% generated; humans edit ONLY ADR front-matter + spec source. Assertion #4 makes any hand-edit to a generated face a RED.

**REQUIRED RED fixture** — `specs/fixtures/cross-artifact-agreement/tc-XA-bad-axes-count-drift.json`:
```json
{ "fixture_id":"TC-XA-BAD-axes-count-drift", "expected_verdict":"RED",
  "expected_violations":["generated_face_drift"],
  "description":"Frozen live exhibit: catalog.json axes_count 6 vs contracts.json axes_count 7.",
  "faces":{ "catalog.json":{"axes_count":6}, "contracts.json":{"axes_count":7} },
  "regenerated_axes_count":7, "source_test":".omx/plans/test-spec-phase0.md#T-XA.4" }
```
Known-bad input it MUST fail: the verbatim `catalog.json:12`=6 vs `contracts.json:9`=7 frozen from disk. (Companion REDs to author: `tc-XA-bad-dup-adr-number.json` freezing the two ADR-0377 files; `tc-XA-bad-half-supersession.json` freezing 0511 `superseded_by:[]`.)

**REQUIRED GREEN fixture** — `tc-XA-good-decision-all-four-agree.json`:
```json
{ "fixture_id":"TC-XA-GOOD-all-four-agree", "expected_verdict":"GREEN", "expected_violations":[],
  "description":"Decision with ADR Accepted + spec + masterplan node + roadmap node + reciprocal supersession passes.",
  "decision":{ "adr_id":"ADR-0500", "status_3axis":["Accepted","stable","one-way"],
    "spec_ids":["spec.example"], "masterplan_node":"mp.example", "roadmap_node":"rm.example",
    "supersedes":["ADR-0499"], "reciprocal_ok":true },
  "source_test":".omx/plans/test-spec-phase0.md#T-XA.0" }
```

**Self-test (born-blocking proof):** run over the current corpus — MUST emit `generated_face_drift` (axes_count), `dual_decision_collision` (ADR-0377), `supersession_half_edge` (0511) as RED before flipping to `automated_blocking_now`.

---

### GATE 2 — `cloud-ci-total-accounting`  [BUILD]  (owns + produces `accounting-registry.generated.json`)
*Charter: D-DOCTRINE total-accounting (`decision-record:180`); pillar-G accounting model (`backlog:302,312`). This is the producer of record; Gates 1/3/4 join its registry.*

**Inputs:** `git ls-files` (tracked truth) · `OWNERS` files · ADR front-matter (justification) · `specs/masterplan.json` + `root-hub-pointers.json` + Cargo.toml members + DOC-CATALOG (reachability) · the Gate-1 crosswalk.

**Blocking semantics (RED + violation code):**
| # | code | RED when |
|---|---|---|
| 1 | `unaccounted` | a tracked path has no registry row (new file landed without accounting) |
| 2 | `unowned` | row has no OWNERS-resolvable owner (born blocking: 0 OWNERS exist today) |
| 3 | `unjustified` | `justification_ref` empty or points at a non-existent ADR/spec/need (orphan) |
| 4 | `unreachable` | `reachable_from` is empty / does not resolve to a live masterplan node |
| 5 | `no_ttl_class` | path has no `ttl.ttl_class` (feeds Gate-3) |
| 6 | `registry_drift` | committed `accounting-registry.generated.json` ≠ regenerated (hand-edit) |

**Auto-archive (report-then-archive, NEVER silent delete):** orphans (#3) + unreachables (#4) past their `ttl.budget_days` are REPORTED, then moved to `_archive/` by `git mv` (reversible) — gated by a SECOND verifier pass, never an in-gate `rm` (founder rule "never delete/amend on an unverified verdict"; pillar-G "report before delete", `backlog:306`).

**Generated-vs-committed:** registry fully generated from `git ls-files × OWNERS × ADR-front-matter`. Humans add an OWNERS entry + an ADR/need justification; the path becomes accounted on regeneration. Assertion #6 makes hand-editing the table a RED.

**REQUIRED RED fixture** — `specs/fixtures/total-accounting/tc-TA-bad-orphan-no-justification.json`:
```json
{ "fixture_id":"TC-TA-BAD-orphan-no-justification", "expected_verdict":"RED",
  "expected_violations":["unjustified"],
  "description":"Frozen live exhibit class: a file justified by a claim that says it does not exist (foundry-residue: 780 oya-foundry-* files vs ADR-0363 'eradicated').",
  "rows":[{ "path":"oya/intelligence/oya-foundry-eval/src/lib.rs", "unit_class":"code",
    "owner":"platform-intelligence", "justification_ref":"ADR-0363",
    "justification_resolves":false, "justification_claims_absent":true,
    "reachable_from":["cargo-members"], "ttl":{"ttl_class":"code","budget_days":null} }],
  "source_test":".omx/plans/test-spec-phase0.md#T-TA.3" }
```
Known-bad input it MUST fail: an orphan whose justifying ADR-0363 claims the file does not exist. (Companion REDs: `tc-TA-bad-new-file-no-row.json` → `unaccounted`; `tc-TA-bad-no-owner.json` → `unowned`; `tc-TA-bad-unreachable-from-masterplan.json` → `unreachable`; `tc-TA-bad-hand-edited-registry.json` → `registry_drift`.)

**REQUIRED GREEN fixture** — `tc-TA-good-fully-accounted.json`:
```json
{ "fixture_id":"TC-TA-GOOD-fully-accounted", "expected_verdict":"GREEN", "expected_violations":[],
  "description":"Path with owner + resolving justification + non-empty reachability + ttl_class passes.",
  "rows":[{ "path":"specs/masterplan.json", "unit_class":"spec",
    "owner":"council-architecture", "justification_ref":"ADR-0364", "justification_resolves":true,
    "reachable_from":["root-hub","masterplan"], "ttl":{"ttl_class":"spec","budget_days":null,"protected":true} }],
  "source_test":".omx/plans/test-spec-phase0.md#T-TA.0" }
```
(Plus `tc-TA-good-archive-candidate-reported.json` → GREEN-with-report: an over-TTL orphan REPORTED for archive, not deleted in-gate.)

**Self-test (born-blocking proof):** over the live tree MUST flag the 780 `oya-foundry-*` files as `unjustified` and the 57 unwired `oya-governance-*` crates as `unreachable`, AND (per the live `find -name OWNERS = 0`) flag broadly `unowned` until OWNERS files are created.

---

### GATE 3 — `cloud-ci-staleness-reaper`  [BUILD]
*Charter: pillar-G accounted-GC / "garbage accumulation = a missing reaper" (`backlog:311`); linux Task-#14 ">48h stale-file (ai-slop pileup)". Maps to oya-ci `sinker` (Prow GC, ADR-0513) + extensions.*

**Inputs:** Gate-2's `accounting-registry.generated.json` (every row carries `ttl` + `last_touch_commit`) · generated `ttl-policy.generated.json` (per resource-class budgets from `backlog:304-310`: worktree/branch/build_artifact/container/process/`stale_doc`) · `git log` per-path last-touch.

**Blocking semantics (RED + violation code):** primarily report-then-act, but with a blocking face (pillar-G "a CI hygiene GATE that BLOCKS if accounting drifts", `backlog:311`):
| # | code | RED when |
|---|---|---|
| 1 | `stale_over_budget_unreachable` | path past TTL budget AND unreachable-from-masterplan (Gate-2 join) → blocks a merge that ADDS to the pile. Reachable-but-old ⇒ report-only (a live ADR may be old). |
| 2 | `untyped_staleness` | a resource with no `ttl_class` → RED (defers to Gate-2 #5; reaper refuses to run on un-TTL'd resources — no silent immortal files) |
| 3 | `reap_without_report` | an archive/delete action with no prior report record → RED (enforces report-then-archive ordering; never reap on an unverified verdict) |

**Generated-vs-committed:** `ttl-policy` generated from the single pillar-G resource-class table; per-path TTL assignment generated by Gate-2. Humans set CLASS budgets in one policy source, never per-file expiry. **Protected classes** (release tags, founder `door:one-way` records, ADR history) carry `protected:true` and are NEVER reaped — declared in policy DATA, not scanner code.

**REQUIRED RED fixture** — `specs/fixtures/staleness-reaper/tc-SR-bad-stale-unreachable-doc.json`:
```json
{ "fixture_id":"TC-SR-BAD-stale-unreachable-doc", "expected_verdict":"RED",
  "expected_violations":["stale_over_budget_unreachable"],
  "description":"Frozen Task-#14 ai-slop class: a scratch audit doc untouched > budget AND unreachable from masterplan.",
  "rows":[{ "path":"docs/audit/.../synthesis/_partial-scratch.md", "unit_class":"doc",
    "ttl":{"ttl_class":"stale_doc","budget_days":2,"action":"archive","protected":false},
    "days_since_last_touch":9, "reachable_from":[] }],
  "source_test":".omx/plans/test-spec-phase0.md#T-SR.1" }
```
Known-bad input it MUST fail: a >48h-untouched, masterplan-unreachable scratch doc. (Companion REDs: `tc-SR-bad-untyped-resource.json` → `untyped_staleness`; `tc-SR-bad-reap-without-report.json` → `reap_without_report`.)

**REQUIRED GREEN fixtures** — `tc-SR-good-old-but-reachable-adr.json` (a 2-year-old LIVE ADR passes — age alone ≠ stale) and `tc-SR-good-protected-not-reaped.json`:
```json
{ "fixture_id":"TC-SR-GOOD-protected-not-reaped", "expected_verdict":"GREEN", "expected_violations":[],
  "description":"A release tag past budget but protected:true is NOT reaped.",
  "rows":[{ "path":".git/refs/tags/v0.1.0-equivalent-record", "unit_class":"registry",
    "ttl":{"ttl_class":"release_tag","budget_days":30,"action":"report","protected":true},
    "days_since_last_touch":400, "reachable_from":["root-hub"] }],
  "source_test":".omx/plans/test-spec-phase0.md#T-SR.0" }
```
(Plus `tc-SR-good-stale-reported-then-archived.json` → GREEN-with-report: report-then-`git mv`-to-`_archive/`, no `rm`.)

**Self-test (born-blocking proof):** over the live audit tree MUST report the `synthesis/_partial-*` / `_verify-*` scratch artifacts (if untouched > budget AND unreachable) as archive candidates — proving it would have caught the very pileup Task-#14 exists for.

---

### GATE 4 — `cloud-ci-automation-ratchet`  (EXISTS as seed; this HARDENS it)
*Charter: register #20 (`backlog:661-668`). Seed `phase0-automation-matrix.json` + 4 live fixtures ALREADY ON DISK.*

**Inputs (EXISTS):** `specs/phase0-automation-matrix.json` (the seed, `classifications` 4-enum, `required_row_fields` 10) · [BUILD] generated `enforcement-inventory.generated.json` (one row per enforcement CLAIM across every ADR `enforcement_status`, every spec `gate_contract`, every operating-contract requirement, every branch-protection constraint, every generated registry, every reviewer/multispectrum requirement) · the live buck2 gate-target set (to verify each "automated" row has a wired producer).

**Blocking semantics — RECONCILED to the REAL on-disk violation codes** (the prior design doc invented different names; the 4 live fixtures are authority):
| # | code (ON-DISK authoritative) | RED when | live fixture |
|---|---|---|---|
| 1 | `enforceable_or_automatable_marked_human_judgment` | a row with `enforceable_or_automatable:true` classified `not_automatable_human_judgment` | `tc-0.16-bad-operator-checklist-for-automatable-rule.json` (EXISTS) |
| 2 | `blocking_invariant_mapped_to_oya_cli` | a row whose `target_gate_or_controller` is an `oya` CLI invocation (the live ADR-0365 `oya gen propagate --check` defect) | `tc-0.16-bad-oya-cli-authority.json` (EXISTS) |
| 3 | `duplicate_row_id` + `unknown_classification` + `missing_or_empty_required_field` | duplicate ids / classifier not in the 4-enum / missing any of the 10 required fields | `tc-0.16-bad-missing-field-unknown-classifier-duplicate.json` (EXISTS) |
| 4 | `advisory_claiming_enforced` [BUILD] | a rule whose doc says "enforces"/"blocks" but has NO wired buck2 target (57 `oya-governance-*` crates; `diataxis-doc-class`; `prd-axis-coverage`) | `tc-AR-bad-advisory-claiming-enforced.json` (NET-NEW) |
| 5 | `ratchet_regression` [BUILD] | an item previously `automated_blocking_now` downgraded vs the committed baseline (monotonic — the ratchet only tightens) | `tc-AR-bad-ratchet-regression.json` (NET-NEW) |

> CORRECTION (WHAT EXISTS vs the prior design): the design doc `10-enforcement-primitives.md:171-173` named violations `enforceable_or_automatable_marked_human_judgment`, `oya_cli_authority`, `incomplete_exception`. Only the first matches disk. The real codes are `blocking_invariant_mapped_to_oya_cli` (not `oya_cli_authority`) and the triple `duplicate_row_id`/`unknown_classification`/`missing_or_empty_required_field` (not `incomplete_exception`). **Implement to the disk codes**; do not rename the live fixtures.

**Generated-vs-committed:** `enforcement-inventory` is GENERATED by scanning all enforcement CLAIMS; the matrix CLASSIFICATION is founder-decided per row (genuine human judgment). The gate verifies the classification is HONEST (#1/#2/#4), COMPLETE (#3), and MONOTONIC (#5) — it does not decide automatability. This is register #21's claim-ceiling applied to the enforcement layer itself.

**REQUIRED RED fixture (EXISTS, reuse)** — `tc-0.16-bad-oya-cli-authority.json`: row with `target_gate_or_controller: "oya gate run-all --ci-required"`, `expected_violations:["blocking_invariant_mapped_to_oya_cli"]`. Known-bad it MUST fail: a blocking invariant satisfied by an `oya` CLI call.
**REQUIRED RED fixture (NET-NEW)** — `specs/fixtures/phase0-automation-ratchet/tc-AR-bad-advisory-claiming-enforced.json`:
```json
{ "fixture_id":"TC-AR-BAD-advisory-claiming-enforced", "expected_verdict":"RED",
  "expected_violations":["advisory_claiming_enforced"],
  "description":"Frozen live exhibit: a rule that claims 'enforces Directive 10' (diataxis-doc-class) with NO wired buck2 target.",
  "rows":[{ "id":"BAD-diataxis-enforces-no-target", "classification":"automated_blocking_now",
    "enforceable_or_automatable":true, "claims_enforcement":true,
    "owner":"platform-governance", "target_gate_or_controller":"docs/governance-lanes/diataxis-doc-class.md",
    "wired_buck2_target":false, "blocking_fixture":"", "retirement_phase":"none",
    "evidence_path":"", "no_new_oya_cli_surface":true }],
  "source_test":".omx/plans/test-spec-phase0.md#T0.16.4" }
```
**REQUIRED GREEN fixture (EXISTS, reuse)** — `tc-0.16-good-human-judgment-with-retirement-path.json`: a genuine human-judgment row WITH owner+target+fixture+retirement+evidence, `enforceable_or_automatable:false`, `expected_violations:[]`.

**Self-test (born-blocking proof):** over the live corpus the inventory MUST emit `advisory_claiming_enforced` for the 57 `oya-governance-*` crates (0 root-BUCK refs), `diataxis-doc-class` ("enforces Directive 10", no gate), `prd-axis-coverage` (event-name only), AND `blocking_invariant_mapped_to_oya_cli` for ADR-0365's 7 `oya gate`/`oya gen` `verified_by` lines.

---

## C. WHY THE FIXTURE CODES MATTER (robustness bar)

The founder's robustness bar = every gate proven by a RED fixture (known-bad it MUST fail) + a GREEN fixture (known-good it passes) + proof it runs in CI and BLOCKS. Two concrete robustness rules from reading the live fixtures:
1. **Violation codes are a contract, not prose.** The 4 live ratchet fixtures pin EXACT `expected_violations` arrays; a gate that returns the right verdict but wrong code is a test failure. Gates 1-3 inherit this — every code in §B is the literal string the gate must emit.
2. **The RED fixtures are FROZEN LIVE EXHIBITS, not synthetic.** axes_count 6-vs-7, dup-0377, 0511 half-edge, oya-CLI `verified_by`, the 780 foundry orphans, the OWNERS=0 gap — each RED fixture is a real defect read from disk this session, so each gate is *born blocking the real drift it targets* (the §0 self-test bar). A gate may NOT flip from `automated_advisory_until_p0_0` to `automated_blocking_now` until its self-test reproduces its live exhibit as RED (Gate-4 enforces this on Gates 1-3 — the ratchet polices itself).

---

## D. BUCK2-NATIVE WIRING (no new `oya` CLI command)

Per register #20 ("New `oya` CLI commands are forbidden") and the matrix `gate_contract.producer: "cloud-ci/oya-ci Rust gate packet"` + `non_negotiables` ("Mechanically enforced means required cloud-ci context plus RED/GREEN fixtures … not advisory output"):

1. **Producer** = one buck2 `rust_binary` `//cloud/cloud-ci/gates:accounting-registry-producer` [BUILD] that emits `accounting-registry.generated.json` + the 3 companion generated faces from `git ls-files × OWNERS × ADR × specs × masterplan`. NOT an `oya gen`/`oya gate` subcommand (retires the ADR-0365 defect).
2. **Each gate** = a buck2 `rust_test` target: `//cloud/cloud-ci/gates:cross-artifact-agreement`, `:total-accounting`, `:staleness-reaper`, `:automation-ratchet` [BUILD; the 4th hardens the existing seed]. Each test `glob`s its `specs/fixtures/<gate>/tc-*.json` and asserts `assert_eq!(report.violations, fixture.expected_violations)`.
3. **G-INTEGRITY track, NO build-graph dep** (`backlog:341`): these four operate on specs+filesystem+git, NOT on the buck2 product build-graph, so they SHIP IN PHASE-0 before the build migration — "the false-green firewall must not wait."
4. **Required cloud-ci context, not advisory:** post-P0.0 the four become REQUIRED merge contexts produced by the live `oya-ci-required` controller (D-SEQUENCE Phase-0 apex). Until that controller is live + proven (FE-1), the honest matrix status is `automated_advisory_until_p0_0` — never overstated (register #21).
5. **`committed == regenerated` enforcement is itself a buck2 test:** `//cloud/cloud-ci/gates:registry-drift` re-runs the producer in a sandbox and byte-diffs against the committed registry — a hand-edit to any generated face fails this target. No `oya` CLI in the loop.

---

## E. RETURN DIGEST (also the final message)

**Accounting-registry schema** (`accounting-registry.generated.json` [BUILD]): one generated record per `git ls-files` path, 11 fields — `path · unit_class · owner(OWNERS-derived) · justification_ref(→ADR/spec/need) · reachable_from[] · ttl{class,budget,action,protected} · last_touch_commit · tracked · verdict · dup_of · _provenance`. Invariants: `committed==regenerated` (hand-edit=RED), total coverage (every tracked path has a row), carve-outs as DATA not code, producer = buck2 `rust_binary` never `oya` CLI. Companion generated faces: `decision-crosswalk` (Gate-1), `ttl-policy` (Gate-3), `enforcement-inventory` (Gate-4).

**4 gate crates** (all `cloud-ci-*` `rust_test`; Gate-2 owns the registry, Gates 1/3/4 are predicates over it):
1. **`cloud-ci-cross-artifact-agreement`** [BUILD] — §P. Codes: `orphan_decision · unpropagated_decision · status_disagreement · generated_face_drift · dual_decision_collision · supersession_half_edge`. RED=`tc-XA-bad-axes-count-drift` (frozen catalog 6 vs contracts 7). GREEN=`tc-XA-good-decision-all-four-agree`. Self-test reproduces axes_count + dup-0377 + 0511 half-edge.
2. **`cloud-ci-total-accounting`** [BUILD] — owns the registry. Codes: `unaccounted · unowned · unjustified · unreachable · no_ttl_class · registry_drift`. RED=`tc-TA-bad-orphan-no-justification` (foundry-residue class). GREEN=`tc-TA-good-fully-accounted`. Auto-archive=report→`git mv`→`_archive/`, second-verifier-gated, never `rm`. Self-test flags 780 foundry orphans + 57 unreachable governance crates + OWNERS=0.
3. **`cloud-ci-staleness-reaper`** [BUILD] — pillar-G sinker++/Task-#14. Codes: `stale_over_budget_unreachable · untyped_staleness · reap_without_report`. RED=`tc-SR-bad-stale-unreachable-doc` (>48h + unreachable ai-slop). GREEN=`tc-SR-good-protected-not-reaped` + `tc-SR-good-old-but-reachable-adr` (age alone ≠ stale). Protected classes never reaped (policy data).
4. **`cloud-ci-automation-ratchet`** (EXISTS as seed `phase0-automation-matrix.json` + 4 live fixtures; HARDEN). REAL on-disk codes: `enforceable_or_automatable_marked_human_judgment · blocking_invariant_mapped_to_oya_cli · {duplicate_row_id, unknown_classification, missing_or_empty_required_field}` + NET-NEW `advisory_claiming_enforced · ratchet_regression`. RED=`tc-0.16-bad-oya-cli-authority` (EXISTS) + `tc-AR-bad-advisory-claiming-enforced` (NEW). GREEN=`tc-0.16-good-human-judgment-with-retirement-path` (EXISTS). Self-test flags 57 governance crates + diataxis + prd-axis + ADR-0365 oya-CLI `verified_by`.

**buck2 wiring (no `oya` CLI):** 1 `rust_binary` producer + 4 `rust_test` gates + 1 `registry-drift` test under `//cloud/cloud-ci/gates`; G-INTEGRITY track, no build-graph dep, ships Phase-0; required cloud-ci context post-P0.0.

**Coverage I did NOT do (no silent caps):** This is DESIGN — I did NOT author the Rust crates, the producer, or write the fixture files into source (read-only lane). I re-verified the live exhibits this session (axes_count 6 vs 7 — confirmed `contracts.json:9`=7 AND `catalog.json:12`=6; dup-0377 two files; 0511 `superseded_by:[]`; ADR-0365 seven `oya gate`/`oya gen` `verified_by`; OWNERS=0) but did NOT re-scan all 354 ADRs or all ~95 specs individually — I reused prior-verified counts (780 foundry from `20-verify-foundry-hygiene.md`; 57 governance crates from `10-enforcement-primitives.md`; the design doc's "22" is a lower bound). I corrected the prior design doc's invented Gate-4 violation names to the REAL on-disk fixture codes; I did NOT re-confirm Gate-1/2/3 codes against on-disk fixtures because those fixture dirs do not yet exist ([BUILD]) — their codes here are this design's contract. I confirmed `phase0-claim-evidence-map.json` + `merge-queue-parked-pr.json` exist as registries Gate-1/4 consume but did not read their full bodies.

**Artifact written:** `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/_phase0/10-gates-registry.md`
