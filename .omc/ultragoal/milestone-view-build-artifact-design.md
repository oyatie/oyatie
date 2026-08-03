# Milestone View as a Build Artifact — Design (read-only)

Status: Proposed design. Author lane: architect (read-only). Branch basis: origin/dev @ 648c490e4.
Standard: hyperscaler + Torvalds. Core principle: the generator NEVER hardcodes milestone data; it derives
every datum from a canonical machine-readable SSOT, and where a datum is not machine-readable today that is a
FINDING with the minimal gate-maintained field to add — not a license to hardcode.

Reference artifact (SECTIONS only, NOT a data source):
`/Users/jasonlee/.claude/projects/-Users-jasonlee-Developer-oyatie/milestone-visualization.html`. That file is a
hand-authored, hardcoded snapshot stamped "Generated 2026-06-23" — exactly the unmaintained-snapshot anti-pattern
this design retires. We keep its six sections and visual language; we replace its body with SSOT-derived data.

---

## 0. The seven sections to render (from the reference HTML)

| # | Section | HTML anchor |
|---|---------|-------------|
| S1 | North-star KPIs (capability count, G-span, gate count, blocker count) | `<!-- NORTH STARS -->` line 81 |
| S2 | The Mandate (forbid/guarantee/leverage) — STATIC doctrine, not data | line 91 |
| S3 | Strata ladder (kernel/os/k8s/capabilities/app + meta dirs) + capability chips with maturity level | `<!-- STRATA LADDER -->` line 102 |
| S4 | Platform verticals G001–G013 with status pill + % bar | `<!-- G STORIES -->` line 127 |
| S5 | Cloud-service gap vs GCP/AWS (per category claim_status) | `<!-- CLOUD GAP -->` line 146 |
| S6 | Pipeline-as-Product — enforcement gates LIVE + 7-property bar | `<!-- PIPELINE -->` line 160 |
| S7 | Blockers + "In flight now" (ephemeral PR state) | `<!-- BLOCKERS -->` line 203, `<!-- IN FLIGHT -->` line 190 |

S2 (mandate) and the 7-property bar in S6 are fixed engineering doctrine sourced from
`specs/decision-principles.json` / root `CLAUDE.md` `pipeline-four-property-bar`; they are policy text, not metrics,
and may be embedded as a versioned constant string with a `doctrine_source` pointer. Everything else is derived.

---

## 1. DATUM -> SSOT mapping table

Legend for "derivation": **DIRECT** = field read verbatim; **DERIVED** = computed from one or more SSOT fields by a
pure rule; **MISSING** = no machine-readable source today (FINDING + proposed minimal field). All SSOT paths are
repo-relative and were read on origin/dev.

### S1 — North-star KPIs

| Datum | SSOT file + field | Derivation | Notes |
|-------|-------------------|------------|-------|
| Capability count (`23`) | `specs/capability-registry.json` -> `capabilities[]` length | DERIVED (count) | Registry is `closed:true`; count is authoritative. Currently `len(capabilities)=23`. |
| Meta-dir count (`+6 meta`) | `specs/capability-registry.json` -> `meta_directories[]` length | DERIVED (count) | kernel/os/base/governance/build/app = 6. |
| G-span (`G01–G13`) | `.omc/ultragoal/goals.json` -> `goals[].id` | DERIVED (count) | **TAINTED SOURCE — see S4 MISSING finding.** Span (13) is structurally stable; per-G status is not. |
| Gate count (`16` live) | `cloud/cloud-ci/gates/.../enforcement-inventory.generated.json` (gate rows) | DERIVED (count of live gate rows) | Generated face already exists and is gate-maintained. See S6. |
| Blocker count (`1` structural) | `specs/*` blocker SSOT | **MISSING** | No machine-readable structural-blocker registry exists. See S7 MISSING finding. |

### S3 — Strata ladder + capability maturity

| Datum | SSOT file + field | Derivation | Notes |
|-------|-------------------|------------|-------|
| Rung labels (kernel/os/k8s/app) + charters | `specs/capability-registry.json` -> `meta_directories[].{dir,rung,charter}` and `faces[]` | DIRECT | `rung` is an explicit field (0,1,...) — drives ladder order. |
| Capability list (chips) + charter | `specs/capability-registry.json` -> `capabilities[].{name,charter}` | DIRECT | 23 chips. |
| Capability -> current crate set | `specs/capability-registry.json` -> `capabilities[].absorbs_current_dirs[]` | DIRECT | Used to resolve which catalog crates belong to a capability. |
| Capability maturity LEVEL (the `lvl` tag: shell/core/core+facade/strong/durable/engine) | DERIVED from crate-face presence | **DERIVED, see rule M1** | The reference HTML HARDCODES these levels. We REPLACE with a deterministic derivation. |
| Per-crate face (core/ports/adapters/facade) | `registry/catalog/<crate>.yaml` -> `role` + `face` (see liveness) AND crate-id path segment (`/core/`,`/ports/`,`/adapters/`,`/facade/`) | DERIVED | `registry/catalog/*.yaml` carries `context`, `role`, `capability`, `plane`, `api_stability`, `security_review`, `supply_chain`. |
| Crate liveness (does the capability actually have live crates) | `libs/oya-workspace-members-kernel::resolve_member_dirs(repo_root)` cross `registry/catalog/*.yaml` stems | DERIVED | EXACTLY mirrors `oya-cloud-ci-catalog-liveness-app` (in-process member resolution, NO `cargo metadata`/`buck2` shell-out). |

**Rule M1 (capability maturity, deterministic + face-based — replaces hardcoded `lvl` tags):**
For capability C, let `crates(C)` = live catalog records whose `capability`/`context` maps into C (via
`absorbs_current_dirs` + the catalog `capability` field), and let `faces(C)` = the set of distinct faces present
(core, ports, adapters, facade) inferred from the catalog `role` field and the crate path segment. Maturity tier is
the lattice:
- `none` : no live crate in `crates(C)`.
- `ports` : only `ports` present (trait seam, no engine).
- `core` : `core` present (engine exists).
- `core+facade` : `core` AND `facade` present (run + sell faces).
- `strong`/`durable`/`engine` are EDITORIAL refinements (e.g. "strong" = core+facade+adapters+`api_stability>=stable`).
  These refinement labels currently have NO single numeric source. To keep them honest and machine-derived, the
  generator emits only the LATTICE tier (`none|ports|core|core+facade`) PLUS measured signals
  (`api_stability` max over crates, `security_review` min over crates, count of live crates). The narrative
  superlatives ("strong","most advanced") in the reference HTML are NOT reproduced as derived claims.
This is the Torvalds discipline: render measured tiers + signals; do not let the generator assert maturity prose
it cannot prove from a field.

### S4 — Platform verticals G001–G013

| Datum | SSOT file + field | Derivation | Notes |
|-------|-------------------|------------|-------|
| G-id (`G001..G013`) | `.omc/ultragoal/goals.json` -> `goals[].id` | DIRECT | **NOT tracked on origin/dev** (it is a local-only `.omc/**` workflow file) AND **explicitly NOT auto-updated** (project-memory `ultragoal-stories-are-platform-verticals`). |
| G-title / objective | `.omc/ultragoal/goals.json` -> `goals[].{title,objective}` | DIRECT | Same taint. |
| G-status pill (complete/substantial/early/...) | `.omc/ultragoal/goals.json` -> `goals[].status` | **MISSING (taint)** | The status enum in goals.json is `complete|in_progress|pending` — it does NOT carry the reference's `substantial/early/partial/enabling` granularity, and memory says it is stale. The reference HTML HARDCODED its own finer pills + %. |
| G completion % bar | (reference HTML literal `width:NN%`) | **MISSING** | No numeric per-G completion field exists in ANY SSOT. |

**S4 FINDING — there is NO trustworthy machine-readable G-story status SSOT.** Three sources were checked and all
fail:
1. `.omc/ultragoal/goals.json` — local-only (absent on origin/dev), explicitly NOT auto-updated, status enum too coarse.
2. `specs/cloud-hyperscaler-parity-taxonomy.json` -> `next_goal_mapping` (lines 1531-1545) — maps category->G but
   uses a STALE G-numbering (`identity_access_policy:G004`, `compute_instances:G002`) that CONTRADICTS the current
   goals.json numbering (G002=Trust Substrate, G004=Cedar PDP). This is active drift and must not be trusted for status.
3. `specs/planning-closure-status-closure-ledger.json` / `specs/master-plan-sequencing.json` / `specs/masterplan.json`
   — these track FD-001 / IP / milestone (M01/M02) status, NOT the G001–G013 vertical axis. The masterplan uses
   `M0x` milestones, not `G0xx`. No `G0xx.status` field exists anywhere in `specs/`.

**Minimal machine-readable fix (proposed; the CORRECT fix per the core principle):** add a NEW canonical
governance SSOT `specs/platform-vertical-status.json` (closed registry, canonical-json governed, the same class as
`capability-registry.json`), shaped:

```json
{
  "schema_version": "1.0.0",
  "doctrine_adr": "ADR-NEW-platform-vertical-status",
  "closed": true,
  "registry_kind": "platform_vertical",
  "verticals": [
    {
      "id": "G011",
      "title": "Pipeline-as-Product Ratchet",
      "status": "in_progress",          // enum: not_started|in_progress|substantial|complete|terminal
      "status_evidence_kind": "merged_pr|gate_live|closure_ledger_ref",
      "evidence_refs": ["#684","cloud-ci-firewall"],   // crate-ids / gate-ids / closure-ledger ids ONLY (no clock, no PR API)
      "maturity_signal_source": "derived_from_capability_registry"  // OPTIONAL: bind a G to capabilities so % is DERIVED not asserted
    }
  ]
}
```

Crucially, status must be **gate-maintained**: a companion gate (`oya-cloud-ci-platform-vertical-status-app`)
asserts that every `evidence_refs` entry resolves to a LIVE artifact (a registered gate id in
`enforcement-inventory.generated.json`, or a live capability in the registry), so a vertical cannot claim
`complete`/`substantial` without live evidence — the same born-blocking discipline as catalog-liveness. The numeric
% bar should be DROPPED (it is unprovable prose) and replaced by the discrete status enum + the count of live
evidence refs. **Until this SSOT exists, the generator MUST render G-status as `status: unverified (no machine
SSOT)` and cite this finding — it must NOT silently re-hardcode the reference HTML's pills.**

### S5 — Cloud-service gap vs GCP/AWS

| Datum | SSOT file + field | Derivation | Notes |
|-------|-------------------|------------|-------|
| Category list (Compute/Storage/Networking/...) | `specs/cloud-hyperscaler-parity-taxonomy.json` -> `local_oyatie_mapping[].category_id` | DIRECT | 13 categories (compute_instances, storage_object_block_file, networking_dns_edge, databases_data_analytics, serverless_functions, identity_access_policy, kms_secrets_confidentiality, messaging via... see note, observability_operations, billing_finops_quotas, marketplace_isv_ecosystem, security_posture_guardrails, containers_kubernetes, cloud_native_platform_contract). |
| Gap / claim status (the pill: gap/core/RLS-live/strong/VMM-missing) | `specs/cloud-hyperscaler-parity-taxonomy.json` -> `local_oyatie_mapping[].claim_status` | DIRECT | Enum: `metadata_foundation|evidence_required|target_spec_only`. The reference HTML's finer pills ("VMM missing","RLS live") are HARDCODED narrative; we render the SSOT enum + `honest_claim` + `blocked_claim_families[]`. |
| Honest-claim / cannot-claim text | `... -> local_oyatie_mapping[].{honest_claim,cannot_claim_yet[],blocked_claim_families[]}` | DIRECT | These are the honest non-claim guardrails; render verbatim. |
| Per-service resource maturity (optional deeper view) | `specs/cloud-resource-catalog-target.json` -> `[].service` + claim fields | DIRECT | Service-level (cloud-iam, cloud-kms, cloud-compute, ...) parity catalog. |
| Category -> capability link | DERIVED: map `category_id` -> capability via a small policy table | **MISSING (minor)** | There is no field linking `compute_instances` -> capability `compute`. Today this is a 1:1 obvious mapping but it is implicit. Minimal fix: add `local_oyatie_mapping[].capability` to the parity taxonomy (one field) so the dashboard can co-locate S3 maturity with S5 gap without a generator-side hardcoded crosswalk. |

Note: there is NO `messaging` category_id in the parity taxonomy (the reference HTML shows a Messaging card). The
generator renders only categories that EXIST in the SSOT; the Messaging card in the reference is editorial. If a
Messaging parity row is desired, the fix is to ADD it to `cloud-hyperscaler-parity-taxonomy.json`, not to the generator.

### S6 — Enforcement gates (Pipeline-as-Product)

| Datum | SSOT file + field | Derivation | Notes |
|-------|-------------------|------------|-------|
| Gate list (chips: authz-coverage, kernel-purity, freshness, ...) | `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/enforcement-inventory.generated.json` -> gate rows | DIRECT | This is a CONTROLLER-GENERATED face (artifact_class `main-materialized-aggregate`), already gate-maintained by `oya-cloud-ci-accounting-registry-app` + `registry-drift`. Authoritative, machine-readable, hermetic. |
| Gate live/marked status | `enforcement-liveness.generated.json` -> `rows[].{stub_marked,wired_in_*}` | DIRECT | The liveness face distinguishes live vs stub-marked (e.g. `session-start-context-inject.sh stub_marked:true`). |
| Gate count | DERIVED (count live rows) | DERIVED | Feeds S1 KPI. |
| 7-property bar | root `CLAUDE.md` `pipeline-four-property-bar` / `specs/decision-principles.json` | DIRECT (doctrine constant) | Static policy text + `doctrine_source` pointer. |
| Governance pipeline flow (worktree -> PR -> Tide -> merge) | `specs/master-plan-sequencing.json` -> `required_workflow[]` (and root CLAUDE.md `required_workflow`) | DIRECT | The flow steps are machine-readable in the required_workflow block. |

S6 is the GOLD STANDARD section: every datum is already a gate-maintained generated face. This proves the pattern works.

### S7 — Blockers + In-flight (ephemeral)

| Datum | SSOT file + field | Derivation | Notes |
|-------|-------------------|------------|-------|
| Structural blocker: "global merge serialization" (accounting-registry rows/size) | `cloud/cloud-ci/gates/.../accounting-registry.generated.json` -> row count + byte size | DERIVED | The blocker MAGNITUDE (18,495 rows / 12.6 MB) IS machine-derivable from the generated face's length/size — render that as the measured blocker signal. |
| Blocker REGISTRY (which structural blockers exist + their fix) | — | **MISSING** | No `specs/structural-blockers.json` exists. The reference HTML hardcodes the blocker list + fixes. |
| In-flight PRs (#780, #809, #810...), "3 merged", "running"/"HELD" | GitHub PR API | **EPHEMERAL — EXCLUDED from hermetic artifact** | Needs net + clock; violates hermeticity. See Section 4 boundary. |

**S7 FINDING #1 (blockers):** add `specs/structural-blockers.json` (closed registry) shaped
`{id, title, severity, measured_signal_source, fix_plan_ref, resolved:bool}`, where `measured_signal_source`
points at a machine-derivable signal (e.g. `accounting-registry.generated.json#row_count`) so a blocker's MAGNITUDE
is derived and a blocker auto-clears when its measured signal drops below a declared threshold (gate-maintained:
mark `resolved:true` only when the signal proves it). Until then, render blockers as `unverified (no machine SSOT)`.

**S7 FINDING #2 (in-flight):** ephemeral PR/CI state (open PRs, merge state, "running") is NOT hermetic — it needs
network + clock. It is EXCLUDED from the committed structural view. If an in-flight section is wanted, it is fed via
a SEPARATE CI-produced input snapshot (see Section 4), never by a clock/net call inside the generator. The committed
VIEW shows STRUCTURAL state only.

---

## 2. The generator crate

**Canonical name (de-branded, path=namespace per naming-grammar doctrine):** `milestone-view-app`
(buck label `//build/milestone-view:milestone-view-app`, cargo name `milestone-view-app`).
Until the `build/` top-dir exists (it does NOT yet on origin/dev — confirmed: `git ls-tree origin/dev` has no
`build/`/`governance/`/`app/`), it is HELD at its cloud-ci-adjacent interim home and MOVED by the strangler when
`build/` lands, exactly as the registry `_comment` says capability-registry.json itself lives at `specs/` "held...
until the governance/ top-level dir is created."

**Shape-stable home decision (justified):** the registry defines `build/` charter as "the generated sell-catalog
(SKU/pricing) VIEW ... Owns zero capability crates" and "buck2 prelude, toolchains ... CI engines, and the
generated ... VIEW." The milestone view is, definitionally, **a generated VIEW over governance SSOTs** — the same
artifact class as the sell-catalog. Therefore its shape-stable home is **`build/milestone-view/`** (generated-view
charter), NOT `governance/` (governance/ owns the SSOTs and the dep-lint authority — the INPUTS, not the rendered
view). This survives the optimal-shape reorg because `build/` is an explicit closed meta_directory in
`capability-registry.json` with `off_runtime_ladder:true` and a generated-view charter that names exactly this use.

**Interim home (pre-`build/`):** `cloud/cloud-ci/gates/oya-cloud-ci-milestone-view-app/` is REJECTED — it is a gate
home, and this is a view producer not a gate (Torvalds: do not put a non-gate in the gate dir). Correct interim:
`build/milestone-view/` created NOW as the first tenant of `build/` (the registry already authorizes `build/`), or,
if creating `build/` is out of scope for this lane, `tools/oya-milestone-view-app/` flagged `build/`-bound in the
membership-lint `absorbs_current_crate_globs` build/ entry (which already globs `tools/*-app` as build/CI tooling).
Recommendation: create `build/milestone-view/` directly — it is the destination and avoids a second move.

**Crate shape (clean-architecture faces; pure kernel + thin binary):**
- `build/milestone-view/core/` (lib) — PURE. `fn render(model: &MilestoneModel) -> String` (HTML) and
  `fn build_model(inputs: &SsotInputs) -> MilestoneModel`. No I/O, no clock, no net, no fs. `#![forbid(unsafe_code)]`.
  `SsotInputs` is plain deserialized data (the parsed SSOT JSONs + resolved member dirs + catalog records).
- `build/milestone-view/app/` (bin) — the ONLY I/O boundary. Reads the SSOT files from `repo_root`, resolves live
  members via `oya_workspace_members_kernel::resolve_member_dirs` (in-process; NO shell), builds `SsotInputs`,
  calls `core::build_model` then `core::render`, writes the HTML to the declared output path (or stdout-json per the
  generated-artifact-control-plane `output_mode`). Mirrors `oya-cloud-ci-accounting-registry-app` (producer owns ALL
  repo I/O; the kernel stays pure) — the exact split praised in catalog-liveness.

**Inputs (declared, hermetic):** `Cargo.toml` (workspace members), `registry/catalog/*.yaml`,
`specs/capability-registry.json`, `specs/cloud-hyperscaler-parity-taxonomy.json`,
`specs/cloud-resource-catalog-target.json`, `enforcement-inventory.generated.json`,
`enforcement-liveness.generated.json`, `accounting-registry.generated.json` (for blocker magnitude only — row
count/size, read as bytes), the proposed `specs/platform-vertical-status.json` and `specs/structural-blockers.json`
once they exist, and the doctrine constants. ALL are committed tree state — no clock, no net, no rand.

---

## 3. Output VIEW path + committed vs CI-materialized

**Output path:** `build/milestone-view/milestone.generated.html`.

**Committed vs CI-only — RECOMMENDATION: COMMITTED (freshness: committed == regenerated).** Rationale: this mirrors
the existing `*.generated.json` faces (artifact_class `main-materialized-aggregate`, materialization_mode
`merge-candidate-regenerated`, merge_policy `never-manual-merge-regenerate-from-source-tree`). Committing the HTML
gives: (a) a diffable structural snapshot a human/agent can open from any checkout (the whole point — replace the
emailed hand-authored HTML), (b) byte-comparison freshness enforcement (Section 4) that makes drift impossible, and
(c) zero dependency on a live CI artifact store to view current state. The HTML is content-addressed by its own
bytes; freshness = "regenerating from the current tree yields byte-identical output."

**Registration:** declare it in `registry/generated-artifact-control-plane.json` as a new `artifacts[]` entry:
```json
{
  "artifact_id": "milestone-view-html",
  "path": "build/milestone-view/milestone.generated.html",
  "artifact_class": "main-materialized-aggregate",
  "materialization_mode": "merge-candidate-regenerated",
  "merge_policy": "never-manual-merge-regenerate-from-source-tree",
  "owner_team": "governance",
  "source_inputs": ["specs/capability-registry.json","specs/cloud-hyperscaler-parity-taxonomy.json",
                    "specs/cloud-resource-catalog-target.json","registry/catalog/**",
                    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/enforcement-inventory.generated.json",
                    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/enforcement-liveness.generated.json",
                    "specs/platform-vertical-status.json","specs/structural-blockers.json"],
  "final_tree_validation": "freshness gate byte-compares the committed HTML against a fresh in-process regeneration from the candidate tree.",
  "public_product_contract": "A repository's milestone/maturity dashboard is a generated VIEW over its governance SSOTs; humans/agents never hand-edit it.",
  "generator": { "runner":"buck2",
    "generator_target":"//build/milestone-view:milestone-view-app",
    "operation_id":"render-milestone-view", "parameters":{},
    "input_contract":["repo-root","declared-source-inputs"], "output_mode":"declared-artifact-path-write" }
}
```
This makes the milestone view a first-class member of the existing generated-artifact control plane — universal
(any adopter declares the same manifest entry), productized (engine + manifest + control plane), and hermetic.

**The ephemeral "in-flight" layer (if rendered at all)** is a SEPARATE, NON-committed, CI-only fragment produced by
a CI step that has net access, written to a CI artifact (not the tree), and composed at view-time only in the
hosted/published dashboard — never inside the hermetic generator and never in the committed HTML.

---

## 4. Freshness / liveness GATE (drift made impossible)

**Gate crate:** `cloud/cloud-ci/gates/oya-cloud-ci-milestone-view-freshness-app` (a gate IS a gate-dir resident).
It MIRRORS `oya-cloud-ci-freshness-app` + `oya-cloud-ci-generated-artifact-control-plane-app`:

- **Pure predicate (`evaluate_keyed`)** over: (A) the committed `milestone.generated.html` bytes, (B) a fresh
  regeneration produced in-process by calling `milestone-view-core::render(build_model(inputs))` against the
  candidate-tree SSOT inputs. Finding `MilestoneViewStale {key}` iff `committed_bytes != regenerated_bytes`.
- The gate calls NO VCS/shell/net/clock. It reads declared inputs from the candidate tree (the same hermetic
  contract the existing `freshness-app` and `generated-artifact-control-plane-app` headers state verbatim).
- **Born-blocking, EMPTY baseline** (catalog-liveness pattern): the disposition table marks the staleness violation
  `frozen_empty` so a stale HTML can never be laundered into the accepted baseline by regeneration. After the first
  green materialization, the corpus carries zero stale rows and any future drift is born-blocked.
- **Candidate-tree evaluation (NOT frozen-merge-base):** per project-memory
  `gate-baseline-pr-push-asymmetry-false-green`, the staleness predicate MUST evaluate the CANDIDATE tree (regen
  from the integrated tip), not a frozen baseline — otherwise a content change at PR-tier passes while the
  integrated tip is stale. Design-Q satisfied: "could this pass at PR-tier but fail on the integrated tip?" -> No,
  because we regenerate from the candidate tree.
- **Auto-fix (AUTOMATED property):** ship `oya-cloud-ci-milestone-view-settle --settle --commit` (mirrors
  `oya-cloud-ci-face-settle`) and register the regenerate step in `infra/ci/materialize-cloud-ci-generated-faces.sh`
  (the existing materialize path the freshness-app already references) so the fix is mechanical, not flag-only.
- **Registration:** the gate id `cloud-ci-milestone-view-freshness` is added to the gate matrix; it then appears in
  `enforcement-inventory.generated.json` automatically (the inventory face is generated from `gates/**`), so S6 of
  the dashboard renders its OWN freshness gate — the pipeline polices the view that describes the pipeline.

**Why this makes drift impossible:** the committed HTML can only ever be the byte-exact render of current SSOTs;
any SSOT change without a regen, or any hand-edit of the HTML, fails the gate at PR + push tier. The
unmaintained-snapshot class (the reference HTML's "Generated 2026-06-23" hardcode) is structurally retired.

---

## 5. Determinism strategy

- **No clock / rand / net / shell anywhere in core or app.** The bin's only I/O is fs reads of declared inputs +
  one fs write of the output. `#![forbid(unsafe_code)]` on both crates.
- **No timestamp in the artifact.** The reference HTML's "Generated 2026-06-23" stamp is REMOVED (a clock value
  would break byte-stability and is non-hermetic). If provenance is desired, emit a content fingerprint
  (BLAKE3/SHA-256 of the canonicalized input set) computed purely from input bytes — deterministic, no clock.
- **Canonical ordering:** every collection is sorted by a stable key before render — capabilities by `name`, gates
  by gate-id, categories by `category_id`, verticals by `id`, catalog crates by crate-id. Use `BTreeMap`/`BTreeSet`
  exactly as the existing gates do (`use std::collections::{BTreeMap, BTreeSet}` in freshness/control-plane apps).
- **Member resolution is deterministic:** `resolve_member_dirs` already returns "sorted and de-duplicated"
  repo-relative paths (confirmed in its doc-comment) — no ordering nondeterminism from the filesystem.
- **Byte-stable HTML emission:** fixed attribute order, fixed whitespace, `\n` line endings, no map iteration over
  un-ordered containers, no float formatting locale dependence (status is enums/integers, not floats — the % bars
  are DROPPED per S4, removing the only float risk). Numbers formatted with a fixed formatter.
- **Pure JSON/TOML parse:** `serde_json` / `toml` (already in-tree deps) — deterministic deserialization.
- **Result:** regenerating on any machine, any time, from the same tree yields byte-identical output — the
  precondition the freshness gate depends on.

---

## 6. Test plan

Layered ladder (per testing-standards-multilayer), each test PURE and DB-free:

1. **Golden-fixture test (core):** a committed `tests/fixtures/ssot-inputs/` directory containing a small frozen set
   of SSOT JSON/YAML inputs + the expected `golden.html`. `assert_eq!(render(build_model(fixture_inputs)),
   read("golden.html"))`. Proves the render contract + the datum->SSOT mappings of Section 1 end-to-end without the
   real corpus. Update-golden is an explicit, reviewed action.
2. **Determinism test:** call `render(build_model(inputs))` TWICE (and once with input collections pre-shuffled in
   the fixture) and assert byte-identical output. Proves canonical-ordering + no nondeterminism. Mirrors the
   intent of the existing gates' deterministic-face tests.
3. **Planted-staleness RED (the drift detector under test):** take the golden inputs + golden HTML, then MUTATE one
   input (e.g. add a capability to the registry fixture / flip a gate's `stub_marked`) WITHOUT regenerating the
   HTML, and assert the freshness gate's `evaluate_keyed` returns `Verdict::Red` with a `MilestoneViewStale`
   finding keyed to the artifact path. Also assert the inverse GREEN case (regenerated HTML matches). This is the
   RED/GREEN fixture pair the testing doctrine mandates and is the proof the gate actually catches drift.
4. **Honest-unverified test:** with the `platform-vertical-status.json` / `structural-blockers.json` SSOTs ABSENT
   (the current reality), assert the rendered S4/S7 sections contain the literal `unverified (no machine SSOT)`
   marker and DO NOT contain any hardcoded status pill — proving the generator refuses to fabricate the missing data.
5. **Liveness cross-check test:** a fixture where a `registry/catalog/*.yaml` stem names a NON-member crate; assert
   that capability's maturity tier degrades (does not count a dead crate as live) — mirrors catalog-liveness's
   silently-stale detection.
6. **Membership/registration test:** assert the gate id is present in the gate matrix and that the artifact entry in
   `registry/generated-artifact-control-plane.json` validates against the control-plane gate's field schema
   (artifact_class/materialization_mode/merge_policy/generator from the enum constants in
   `oya-cloud-ci-generated-artifact-control-plane-app`).

---

## 7. Findings — datums that CANNOT currently be machine-derived (flagged, with the minimal fix)

| # | Datum | Status | Minimal machine-readable fix (owning SSOT) | Must-be-gate-maintained |
|---|-------|--------|--------------------------------------------|--------------------------|
| F1 | **G001–G013 per-vertical status** | **MISSING / TAINTED** | NEW `specs/platform-vertical-status.json` (closed registry, status enum + evidence_refs to live gate-ids/capabilities) | YES — `oya-cloud-ci-platform-vertical-status-app` asserts evidence_refs resolve to live artifacts; no `complete` without live evidence |
| F2 | **Stale G-numbering crosswalk** | **DRIFT BUG** | `cloud-hyperscaler-parity-taxonomy.json#next_goal_mapping` uses obsolete G-ids (G002=compute vs current G002=trust). Fix the field to current ids OR drop it; add `cross-artifact-agreement` coverage so parity G-ids must match the vertical-status registry | YES — cross-artifact-agreement gate |
| F3 | **Per-G completion %** | **UNPROVABLE** | DROP it. Replace with discrete status enum + live-evidence count (derivable). Do not add an unprovable numeric field. | n/a (removed) |
| F4 | **Structural blocker registry + fixes** | **MISSING** | NEW `specs/structural-blockers.json` `{id,title,severity,measured_signal_source,fix_plan_ref,resolved}` with `measured_signal_source` pointing at a derivable signal (e.g. accounting-registry row_count) | YES — blocker auto-clears when its measured signal crosses the declared threshold |
| F5 | **Category -> capability link (S5<->S3)** | **MISSING (minor/implicit)** | ADD `local_oyatie_mapping[].capability` to `cloud-hyperscaler-parity-taxonomy.json` (one field) instead of a generator-side hardcoded crosswalk | YES — capability-membership gate validates the slug exists in capability-registry |
| F6 | **Capability maturity superlatives** ("strong","durable","most advanced") | **UNPROVABLE PROSE** | DO NOT render. Emit only the derivable lattice tier (none/ports/core/core+facade) + measured signals (api_stability, security_review, live-crate count) from `registry/catalog/*.yaml` | n/a (derived) |
| F7 | **In-flight PR/CI state** | **EPHEMERAL (correctly excluded)** | Not a tree datum. Feed via a SEPARATE CI-produced snapshot consumed only by the hosted view; NEVER inside the hermetic generator | n/a (boundary) |
| F8 | **Messaging parity card** | **MISSING category** | ADD a `messaging` `local_oyatie_mapping[]` row to the parity taxonomy if the card is wanted; generator renders only SSOT-present categories | YES — taxonomy validator |

**Torvalds note on the core principle:** the single biggest temptation here is to "just hardcode the G-status / %
bars / maturity labels" because the reference HTML already has them. That is precisely the unmaintained-snapshot
moved into code. The disciplined answer for every MISSING datum above is: render it as `unverified (no machine
SSOT)` and add the minimal gate-maintained field to the owning SSOT (F1/F4/F5/F8) or DELETE the unprovable datum
(F3/F6). The generator ships honest the day it lands — it derives what it can (S1 counts, S3 lattice tiers, S5
claim_status, S6 gates — all already machine-readable) and refuses to fabricate the rest until its SSOT exists.
