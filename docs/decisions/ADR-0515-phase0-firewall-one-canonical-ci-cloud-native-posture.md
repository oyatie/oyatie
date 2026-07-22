---
id: ADR-0515
title: "Phase-0 firewall + one-canonical-CI + the 2026-06-07 cloud-native posture — oya-ci-required is the single GitHub-Actions-produced blocking gate; gates are Rust binaries; no CLI/shell; declarative gitops"
status: Accepted
planning_impact: true
deciders: founder, council-architecture
date: 2026-06-07
door: one-way
supersedes: [ADR-0124, ADR-0349, ADR-0359, ADR-0361, ADR-0511, ADR-0513, ADR-0514]
superseded_by: []
depends_on: [ADR-0392]
amends: [ADR-0092, ADR-0363]
related:
  - ADR-0111
  - ADR-0116
  - ADR-0124
  - ADR-0181
  - ADR-0247
  - ADR-0363
  - ADR-0366
  - ADR-0367
  - ADR-0369
  - ADR-0374
  - ADR-0388
  - ADR-0510
  - ADR-0525
related_specs:
  - /specs/phase0-ci-enforcement-baseline.json
  - /specs/phase0-ci-enforcement-result-schema.json
  - /specs/phase0-automation-matrix.json
  - /specs/phase0-claim-evidence-map.json
  - /specs/masterplan.json
  - /specs/master-plan-sequencing.json
homes: [cloud/cloud-scm, cloud/cloud-ci, cloud/cloud-cd]
affected_surfaces:
  crates:
    - oya-cloud-ci-accounting-registry-app
    - oya-cloud-ci-cross-artifact-agreement-app
    - oya-cloud-ci-total-accounting-app
    - oya-cloud-ci-staleness-reaper-app
    - oya-cloud-ci-automation-ratchet-app
    - registry-drift
    - oya-cloud-ci-firewall-app
    - oya-ci-controller
  microservices: []
  specs:
    - /specs/phase0-ci-enforcement-baseline.json
    - /infra/branch-protection/dev.json
    - /.github/branch-protection.yaml
session_context:
  authored: 2026-06-07
  basis: >
    Founder rulings D-CICD-AUTHORITY (GitHub Actions = sole CI authority; oya-ci-required =
    the single blocking context), the ONE CANONICAL CI amendment, D-CLOUD-NATIVE (pipeline-not-CLI;
    no shell; declarative gitops; jenkins=drop-now; forge=scrub + infra/forge -> infra/gitops),
    D-GOVERNANCE-CENTRAL (PaC/CaC/PaaS/CaaS), D-MULTISPECTRUM-RETIRED, D-SSOT-CURRENT-TRUTH,
    D-SEQUENCE (firewall-first), D-DOCTRINE (maintainable-by-enforcement) in
    decision-record-oyatie-canon.md; the ralplan-APPROVED CICD-DESIGN-PLAN.md (one-canonical-CI plan);
    the live firewall substrate (cloud/cloud-ci/gates/* + specs/phase0-*.json); and the Task #26
    oya-ci-required Rust producer driven by GitHub Actions replacing the 108-lane CLI + shell/groovy.
purpose: >
  Establish the SINGLE current-truth CI/CD enforcement decision for oyatie. The Phase-0 firewall
  (one generated accounting-registry + four born-blocking gates + the cloud-ci-firewall ratchet) is
  the enforcement substrate; oya-ci-required is the ONE canonical blocking required context, produced
  by GitHub Actions now (the live authority/runner) and by the owned oya-ci runner after cutover;
  gates are Rust binaries run automatically by the pipeline (no CLI, no shell, declarative gitops).
  Collapses and supersedes the CI/CD ADR cluster (0124/0349/0359/0361/0511/0513/0514) and the
  multispectrum enforcement doctrine; depends on (does not absorb) the Buck2 build substrate (0408/0392).
---

# ADR-0515: Phase-0 firewall, one-canonical-CI, and the cloud-native enforcement posture

## Status

**Accepted — 2026-06-07 (founder-ruled; door: one-way).**

This is the single current-truth CI/CD enforcement ADR. It re-authors the earlier 2026-06-06
CI/CD consolidation off the superseded ADR-0513 "Prow-native authority, GitHub = shadow" worldview
onto the 2026-06-07 posture: GitHub Actions is the sole live CI authority, `oya-ci-required` is the
one canonical blocking context, gates are Rust binaries the pipeline runs automatically, and there is
no CLI/shell interaction model. It **supersedes** ADR-0124, ADR-0349, ADR-0359, ADR-0361, ADR-0511,
ADR-0513, ADR-0514, and the 21-facet multispectrum-review enforcement doctrine; it **depends on**
(does not absorb) ADR-0392 (the Buck2 build substrate, a distinct bounded context); ADR-0525 later
refines the CI orchestration boundary. It
**amends** ADR-0092 (strip the multispectrum dependency-seam subchecks).

The concise reviewer-facing projection of this cloud-native/no-CLI/no-shell posture is
`docs/standards/cloud-native-infrastructure-automation.md`; it is guidance over this ADR's accepted
decision, not a separate runtime or merge authority.

Under **D-SSOT-CURRENT-TRUTH** the SSOT holds only current truth: the superseded cluster is
hard-destroyed (git history is the sole archive) and every dangling reference scrubbed once this ADR
lands — there is no kept `_archive/` tombstone. This ADR is the only CI/CD enforcement document that
remains.

## Context

### 1. The enforcement was a façade — and a façade is the mechanism of the drift
The required merge context `oya-ci-required` had **no live producer**. Live GitHub `dev` protection
required `github-lane-unlocker-required` (a context the committed SSOT itself brands "shadow
compatibility only … must not be treated as merge authority"), while the SSOT named `oya-ci-required`
as the intended required context — so the live ruleset diverged from its own SSOT on **every axis**
(context, `enforce_admins`, `required_signatures`, review-count). Net: **0 gates blocked a merge**.
Per **D-SEQUENCE**, you cannot fix the canon on fake enforcement — enforcement must become **real
first**, then the canon is fixed *through* it (firewall-first).

### 2. The CI/CD canon was a contradictory seven-ADR cluster
The decision was spread across seven ADRs that disagreed, one pair (0511 Argo-wholesale ↔ 0513
bespoke-Prow) directly contradictory **with no supersession edge**: 0124 (own webhook merge-queue),
0349/0359/0361 (Jenkins ± ArgoCD substrate), 0511 (adopt Argo Workflows wholesale), 0513 (bespoke-Rust
Prow clone), 0514 (target-architecture/hyperscaler remediation). A scattered canon with no real gate
is exactly how the drift accumulated (**D-DOCTRINE**: drift + contradiction = a process + enforcement
failure; the fix is the enforcement that makes recurrence impossible, not hand-reconciliation).

### 3. The 2026-06-07 posture rulings changed the shape of "enforcement"
Five founder rulings, recorded after the 2026-06-06 consolidation, supersede the earlier "Prow-native
authority / parallel shadow / per-changeset multi-lens critique / CLI-and-shell governance" framing:

- **D-CICD-AUTHORITY** — GitHub Actions + branch-protection required checks are the **sole** CI/CD
  authority until an explicit cutover; the owned `oya-ci`/`oya-cd` is still built but is the **future
  runner of the same canonical pipeline**, not a parallel live authority.
- **ONE CANONICAL CI (amendment)** — do not maintain two parallel CIs. **One** shared Rust gate-logic
  + one pipeline definition + one surface-all aggregator → **one** canonical blocking context
  `oya-ci-required`. "One logic, two runners" is corrected to "the same canonical pipeline across
  **time**" (GitHub Actions now → owned runner later = a runner migration). The non-blocking-shadow
  clause and the verdict-agreement apparatus are **dropped**.
- **D-CLOUD-NATIVE** — oyatie is cloud-native, **not a collection of CLI tools**. Governance + evidence
  move from the `oya gate`/`oya-governance-*` CLI lanes to an automated pipeline of Rust gate binaries;
  **no shell scripts** (`.sh`/`.bash`/`.groovy` forbidden bar a justified, narrowly-scoped exception);
  gitops is **declarative** (ArgoCD app-of-apps), not a shell `bring-up`. Forbidden vocab
  (`forgejo`/`forge`, `foundry`, `jenkins`, `oya-vcs`) is eradicated: **jenkins = drop now** (no bridge
  retention; GitHub Actions is the sole CI), **forge = scrub** (`infra/forge` consolidated into the
  generic live `infra/gitops`). Palantir-Foundry is the only carve-out.
- **D-GOVERNANCE-CENTRAL** — governance authority is consolidated + central, delivered as the
  cloud-native quartet PaC / CaC / PaaS / CaaS. The keystone gates (cross-artifact-agreement,
  total-accounting, status-enum, no-dangling-ref) **are** the CaC checks; CaaS is the automated
  pipeline that runs them and emits evidence (GitHub Actions live / oya-ci shadow).
- **D-MULTISPECTRUM-RETIRED** — the 21-facet multispectrum-review enforcement doctrine is retired. Its
  accounting/structural half is **superseded** by this firewall + one-canonical-CI substrate; its
  per-changeset multi-lens **critique** half is **dropped** (no replacement built — a consciously
  recorded loss, not a silent one).

### 4. The firewall substrate already exists on disk (the verified ground truth)
`cloud/cloud-ci/gates/*` carries seven Rust crates: the `oya-cloud-ci-accounting-registry-app` (one record per
`git ls-files` path + the TTL / decision-crosswalk / enforcement-inventory faces), four born-blocking
gates (`oya-cloud-ci-cross-artifact-agreement-app` GATE-1, `oya-cloud-ci-total-accounting-app` GATE-2,
`oya-cloud-ci-staleness-reaper-app` GATE-3, `oya-cloud-ci-automation-ratchet-app` GATE-4), the `registry-drift`
committed==regenerated gate, and the `oya-cloud-ci-firewall-app` ratchet that layers the committed baseline as
a second predicate (blocks only **new** debt). The verdict is **data, not control flow**
(`compare()` loops the union of `(gate, code)` keys with no early exit; surface-all is a property of the
`BTreeSet` aggregation). The committed branch-protection SSOT is **already re-authored**
(`infra/branch-protection/dev.json`: `contexts:[oya-ci-required]`, `enforce_admins:true`,
`required_signatures:true`, `required_pull_request_reviews:null`).

Issue #901's affected-set progress helper is part of that same enforcement substrate.
ci/facade/affected-target-set/src/bin/oya-cloud-ci-step-telemetry.rs
reports long Buck2 phase/status/elapsed telemetry for operator visibility; it does not change the required-check verdict model.

The fixed historical ADR-census parent-receipt validator is likewise part of this Buck-live Rust
substrate. `ci/facade/scm-facts-snapshot/src/bin/adr-census-parent-receipt-gate.rs` validates only
the immutable controller-materialized historical receipt; it carries no planning-closure,
roadmap-dispatch, current-authority, or product-readiness claim.

## Decision

### D1. The Phase-0 firewall is the enforcement substrate
The Phase-0 false-green firewall is the substrate that makes merge-gate enforcement **real**:

1. **One generated accounting-registry.** `oya-cloud-ci-accounting-registry-app` emits one deterministic record
   per tracked path (+ the TTL / crosswalk / enforcement-inventory faces); `registry-drift` enforces
   `committed == regenerated` byte-for-byte, making a hand-edit of any generated face structurally
   impossible.
2. **Four born-blocking gates** over that registry, each proving it **DETECTS** the live exhibit
   (RED on today's corpus), with the exact on-disk violation codes as the contract:
   - **GATE-1 cross-artifact-agreement** — `orphan_decision`, `unpropagated_decision`,
     `dual_decision_collision`, `supersession_half_edge`, `status_disagreement`,
     `generated_face_drift`. GATE-1 additionally carries three BORN-ADVISORY
     coverage checks (enforce-no-regression vs the frozen ratchet
     `ci/facade/cross-artifact-agreement/gate-coverage-baseline.json`, the born-empty
     gate-coverage program), each closing a review class no born-blocking code keys
     on: `adr_prose_status_contradiction` (an ADR body prose that disagrees with its
     own front-matter status, matched against the DATA phrase policy
     `ci/facade/cross-artifact-agreement/prose-status-agreement-policy.json`),
     `registry_derived_policy_desync` (a capability root in
     `specs/capability-registry.json` absent from a derived gate policy —
     module-membership, root-hygiene, or tier-dependency), and
     `adr_index_projection_stale` (the generated `docs/ADR-INDEX.md` /
     `docs/machine-readable/decisions.json` projections re-rendered through the
     `oya-check-adr-index` producer kernel, implementing the
     `docs/automation/adr-index-pipeline.md` regeneration promise without shelling out).
     The same cross-artifact-agreement gate additionally consumes
     `ci/facade/cross-artifact-agreement/src/preplanning-candidate-policy.json`, the
     reviewed DATA policy binding the historical pre-planning candidate identity and
     non-closure claim ceiling.
     The three checks are implemented as born-advisory submodules
     `ci/facade/cross-artifact-agreement/src/prose_frontmatter_status.rs`,
     `ci/facade/cross-artifact-agreement/src/registry_policy_sync.rs`, and
     `ci/facade/cross-artifact-agreement/src/adr_index_projection_parity.rs`, sharing the
     frozen-baseline ratchet in
     `ci/facade/cross-artifact-agreement/src/gate_coverage_baseline.rs`; they ride the
     already-registered `ci-cross-artifact-agreement-gate` and are NOT wired into the
     born-blocking §5.2 verdict.
   - **GATE-2 total-accounting** — `unaccounted`, `unowned`, `unjustified`, `unreachable`,
     `no_ttl_class`, `registry_drift`.
   - **GATE-3 staleness-reaper** — REPORTS, never reaps (`report → git mv → _archive/`,
     second-verifier-gated, **never `rm`**): `stale_over_budget_unreachable`, `untyped_staleness`,
     `reap_without_report`.
   - **GATE-4 automation-ratchet** — polices the *other* gates (anything enforceable/automatable must
     be a wired Rust gate, never a CLI call, never an unwired surface that *claims* to enforce):
     `advisory_claiming_enforced`, `blocking_invariant_mapped_to_oya_cli`, `ratchet_regression`,
     `enforceable_or_automatable_marked_human_judgment`, `duplicate_row_id`, `unknown_classification`,
     `missing_or_empty_required_field`.
3. **The `cloud-ci-firewall` ratchet** layers the committed `gate-baseline.generated.json` as a second
   data predicate: it blocks only **new** debt (`regressions = current_keys \ baseline_keys`), tolerates
   frozen pre-existing debt, and the baseline may only ever **shrink** — growth is a
   `ratchet_regression` failure unless every grown key is in the founder-signed
   `gate-baseline.signoff.json` allowlist (the one-way door). Per-code blocking-vs-advisory is **DATA**
   (`gate-disposition.json` `mode` + `frozen_empty`), not code: five codes are `advisory-until-infra`
   today only because their infra is absent (`owners-files-tree-wide`,
   `masterplan-reachability-wiring`, `ttl-policy-table-complete`) and flip to blocking via a
   founder-signed disposition flip → producer-regen → byte-parity (never a hand-edit).

Every gate is RED/GREEN-proven (a known-bad it fails + a known-good it passes), per the
**D-DOCTRINE robustness bar** — no advisory shell claiming enforced.

### D2. `oya-ci-required` is the ONE canonical blocking required context, produced by GitHub Actions
There is **one** canonical CI: one shared Rust gate-logic + one pipeline definition + one surface-all
aggregator → **one** canonical **blocking** required context `oya-ci-required` (name founder-ratified;
supersedes the interim `oya-ci-firewall-required` pick). It is produced by **GitHub Actions now** (the
live authority/runner) and by the owned `oya-ci` runner after cutover — the **same** pipeline across
time (a runner migration), **not** two parallel CIs and **not** a non-blocking shadow. The fan-in is a
pure **zero-build** job that is green **iff** every constituent lane is green; it never runs its own
narrower command set (the false-green cardinal sin) and never re-points the SSOT's "shadow
compatibility only" bridge name to authority. A gate-registration-completeness meta-test asserts every
in-tree `cloud/cloud-ci/gates/*` crate is registered in the aggregator (closing the silent-skip
false-green one level down). **Surface-all, not fail-fast:** lanes fan out `fail-fast:false` and
aggregate into one `FirewallReport` → one Check-Run with per-finding annotations + one summary — one
sweep surfaces every error. A presubmit branch-protection **drift gate** byte-diffs live `gh`
protection against the re-authored SSOT, making the protection config self-defending.

The hyperscaler parity taxonomy Rust gate is admitted as a born-blocking cloud-ci gate under this
substrate; the implementation surface is intentionally limited to:

- `ci/facade/parity-claim-evidence/BUCK`
- `ci/facade/parity-claim-evidence/Cargo.toml`
- `ci/facade/parity-claim-evidence/src/lib.rs`
- `ci/facade/parity-claim-evidence/tests/hyperscaler_parity_taxonomy.rs`

The cloud-resource contracts Rust gate is admitted under the same one-canonical-CI substrate as the
API/config-shaped replacement for the P0 Python cloud-resource validators. Its intentionally bounded
surface is:

- `ci/facade/resource-contract-conformance/BUCK`
- `ci/facade/resource-contract-conformance/Cargo.toml`
- `ci/facade/resource-contract-conformance/README.md`
- `ci/facade/resource-contract-conformance/cloud-resource-contracts-policy.json`
- `ci/facade/resource-contract-conformance/src/lib.rs`
- `ci/facade/resource-contract-conformance/tests/cloud_resource_contracts.rs`

The contract-slice conformance gate is the paved-road owned-Rust/Buck2 replacement for the fleet-wide
`scripts/tests/*_check.py` "contract slice" validators (class-fix). Its intentionally bounded surface is:

- `ci/facade/contract-slice-conformance/BUCK`
- `ci/facade/contract-slice-conformance/Cargo.toml`
- `ci/facade/contract-slice-conformance/OWNERS`
- `ci/facade/contract-slice-conformance/README.md`
- `ci/facade/contract-slice-conformance/contract-slice-policy.json`
- `ci/facade/contract-slice-conformance/fixtures/exemplar-slice.json`
- `ci/facade/contract-slice-conformance/src/lib.rs`
- `ci/facade/contract-slice-conformance/tests/contract_slice_conformance.rs`

The contract-slice-policy sharding follow-up (CI class-fix #3 of the 2026-07-10 merge train:
every slice PR editing the one shared `contract-slice-policy.json` re-dirtied every other open
slice PR) extends that same bounded surface with a per-slice fragment directory plus the
owned-Rust materializer that regenerates the aggregate from it (ADR-0539 `check_equals_fix`
doctrine, applied locally to this gate — the fixer and the byte-parity gate test share one
`aggregate_policy`/`render_policy_json` definition):

- `ci/facade/contract-slice-conformance/slices/cell-002-promotion-automation.json`
- `ci/facade/contract-slice-conformance/slices/cell-002-rollback-audit-fixture.json`
- `ci/facade/contract-slice-conformance/slices/compliance-001-contract-spec.json`
- `ci/facade/contract-slice-conformance/slices/compliance-001-pack-fixture.json`
- `ci/facade/contract-slice-conformance/slices/compliance-001-pack-schema.json`
- `ci/facade/contract-slice-conformance/slices/compliance-001-portability-manifest-fixture.json`
- `ci/facade/contract-slice-conformance/slices/contract-slice-conformance-exemplar.json`
- `ci/facade/contract-slice-conformance/slices/finops-001-cost-attribution.json`
- `ci/facade/contract-slice-conformance/slices/release-001-runtime-safety-policy.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-attestation.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-audit-events.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-fixture-cross-border-refusal.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-fixture-signed-footprint.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-governance-artifact-registry.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-governance-root-hub.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-pack-matrix.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-regimes.json`
- `ci/facade/contract-slice-conformance/slices/residency-001-regions.json`
- `ci/facade/contract-slice-conformance/slices/resilience-001-messenger-manifest-trace-sampling.json`
- `ci/facade/contract-slice-conformance/slices/resilience-001-messenger-runtime-control-loop.json`
- `ci/facade/contract-slice-conformance/slices/talos-001-substrate-slice.json`
- `ci/facade/contract-slice-conformance/src/bin/oya-cloud-ci-materialize-contract-slice-policy.rs`
- `ci/facade/contract-slice-conformance/src/fragments.rs`
- `ci/facade/contract-slice-conformance/tests/policy_fragment_migration.rs`

The run-observability packet/status contract is part of the same single-required-context substrate:
`specs/cloud-ci-run-observability-packet.schema.json` defines the machine-readable packet,
`specs/cloud-ci-run-observability-status.schema.json` defines the separately serialized status/query
projection for console/API consumers, `ci/facade/baseline-ratchet/src/run_observability_packet.rs`
implements the pure validators, and `ci/facade/baseline-ratchet/tests/run_observability_packet.rs`
locks the regression contract. The packet fixture corpus includes
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-actions-log-scrape-only.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-context-mismatch.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-failed-without-diagnostics.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-unstable-nested-ids.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-unstable-packet-id.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-good-failed-gate-diagnosable.json`, and
`specs/fixtures/cloud-ci-run-observability/tc-1003-good-passed-no-failure-diagnostics.json`; the status fixture corpus includes
`specs/fixtures/cloud-ci-run-observability/tc-1003-good-failed-status.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-good-running-status.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-invalid-status-ready.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-status-actions-log-scrape-only.json`,
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-status-context-mismatch.json`, and
`specs/fixtures/cloud-ci-run-observability/tc-1003-bad-status-untyped-refs.json`. The contract closes the false-green class where a failed `oya-ci-required` run is only diagnosable by scraping GitHub Actions logs instead of reading typed cloud-ci/oya-ci status artifacts.
GATE-4's review-admission fixture extension is likewise governed by this ADR: `specs/fixtures/phase0-automation-ratchet/tc-0.16-bad-untrusted-review-authority-source.json`, `specs/fixtures/phase0-automation-ratchet/tc-0.16-bad-standalone-multispectrum-review-evidence.json`, and `specs/fixtures/phase0-automation-ratchet/tc-0.16-bad-missing-review-title-body-evidence.json` prove that review authority comes from trusted cloud-ci/oya-ci review packets, not target-only branch-protection shadows, retired standalone multispectrum files, or packets missing PR title/body binding.
The GH #983 metadata packet implementation is the Rust binary `libs/oya-check-pr-traceability/src/bin/pr-traceability-admission.rs`, owned through `libs/oya-check-pr-traceability/OWNERS`; it is a PR title/body hygiene preflight and not the trusted review-producer closure for F-PR5-06. The binary's `--scaffold`/`--check`/`--all-violations` author workflow (scaffold an admission-passing body, edit, validate locally before opening the PR) is documented in `libs/oya-check-pr-traceability/README.md`.
The total-accounting producer carries the paired author-side preflight for the `unjustified regressions` class (CI class-fix #4: every PR adding tracked files hit `[cloud-ci-total-accounting] unjustified regressions`, discoverable only after materializing scm-facts faces + running the firewall). The producer binary `ci/facade/artifact-inventory-registry/src/main.rs` gains a `--check-paths`/`--check-diff` mode that, for each ADDED tracked file, reports reachable?/justified? and the exact remediation — reusing the SAME resolvers + face-builder + firewall evaluator (no drift) and requiring NO materialized scm-facts face (the added set is the tracked universe). The pre-push author workflow is documented in `ci/facade/artifact-inventory-registry/README.md`.

The board-projection de-commit follow-up registers the pure renderer below as a **non-gate**
dependency of the generated-artifact controller/freshness path. This is file-accounting and CI
wiring provenance only: it does not accept ADR-0377, authorize board actuation, authorize roadmap
planning or implementation dispatch, or lift `HOLD(Planning)`.

Its operational contract is fail closed:

- **Success criteria:** for every admitted candidate tree, the controller reads the candidate's
  materialized masterplan projection, renders a non-empty board projection in the approved legacy
  byte format, and produces byte-identical output in two independent regenerations. The face stays
  absent from Git tracking and contributor staging; successful rendering does not actuate a board.
- **Failure criteria:** the same required-CI run is red when the masterplan projection is missing or
  malformed, a deliverable is missing its identity, identities collide, rendering or writing fails,
  regeneration omits the board face, or two regenerations differ. The controller must not publish a
  partial projection, reuse a stale committed copy, or downgrade any such failure to advisory.
- **SLO objective:** 100% of admitted heads pass regenerate-twice byte parity with zero tracked board
  copies; at least 99.9% of controller materialization attempts complete within five seconds per
  board face over a rolling 30-day window, measured from controller start to durable local write.
  This is an objective for controller telemetry, not a claim that roadmap or board actuation is live.
- **Failure modes and injection:** missing or malformed deliverables and duplicate identities are
  injected by `board_sync_projection_rejects_a_masterplan_without_deliverables`,
  `malformed_deliverable_entries_do_not_materialize_an_empty_projection`,
  `malformed_deliverable_cannot_be_hidden_among_valid_deliverables`, and
  `duplicate_deliverable_ids_fail_closed`; controller read, parse, and write failures are injected by
  `board_sync_materialization_fails_closed_on_read_error`,
  `board_sync_materialization_fails_closed_on_parse_error`, and
  `board_sync_materialization_fails_closed_on_write_error`; legacy-format drift is caught
  byte-for-byte by `board_sync_projection_is_byte_stable_and_uses_the_legacy_wire_shape`; missing
  output and nondeterminism are injected by
  `decommit_class_face_is_stale_when_regeneration_stops_producing_it` and
  `determinism_canary_detects_board_sync_drift_when_other_projections_are_stable`. Each injected mode
  must return a blocking error/finding and must not leave a tracked fallback face.

- `ci/facade/planning-projection/BUCK`
- `ci/facade/planning-projection/Cargo.toml`
- `ci/facade/planning-projection/src/lib.rs`
- `ci/facade/planning-projection/src/fixtures/board-sync-legacy-canonical.json`

### D3. Gates are Rust binaries run automatically — no CLI, no shell, declarative gitops
- **Pipeline, not CLI** (D-CLOUD-NATIVE / D-GOVERNANCE-CENTRAL). All CI / governance / automation are
  **Rust gate binaries run by GitHub Actions** (live) and oya-ci (shadow readiness); evidence is
  **emitted by the pipeline**, not a CLI. The `oya gate` / `oya check` / `oya verify` / `oya-dev-cli`
  governance-CLI family and the `oya-governance-*` lanes are retired into the pipeline. The `oya` CLI is
  retired and **stays retired** — no revival is recorded anywhere.
- **No shell** (D-CLOUD-NATIVE). `.sh`/`.bash`/`.groovy` are forbidden bar a documented, justified,
  narrowly-scoped exception. Live shell behavior is ported to a Rust gate or a declarative manifest,
  never re-authored as more shell.
- **Declarative gitops.** GitOps is a declarative ArgoCD app-of-apps that **syncs itself** — not a shell
  `bring-up`. `infra/forge` is consolidated into the generic live `infra/gitops` (the live Application
  keeps running under the generic name; `bring-up.sh` is eliminated).
- **Everything just works.** The platform is automatic + declarative + self-operating: gates run
  themselves on push, gitops syncs itself, config is declarative (convention-over-configuration). CLI is
  a justified fallback, never the interaction model.
- **Governance authority is central** (D-GOVERNANCE-CENTRAL): one PaC/CaC/PaaS/CaaS quartet, not
  scattered lanes or a CLI; the four keystone gates are the CaC checks, the pipeline is the CaaS that
  runs them — owned + dogfooded (oya = tenant `oyatie-internal`).

### D4. Throughput — wall-clock tracks the size of the change, not the repo (the deciding factor)
Keep caches **warm** and ensure **only the part that CHANGED is cold**: content-addressed actions + a
remote/shared read-through cache (Buck2 RE-API → BuildBuddy/EngFlow/BuildBarn/NativeLink) so an
unchanged action's input-hash is a cache **HIT on any runner**; affected-targeting (owner→rdeps closure
runs only the changed targets + reverse-deps); content-addressed producer artifact (0 recompiles when
unchanged); `actions/cache` for the buck2 binary/toolchain/buck2-out so ephemeral runners do not
bootstrap cold; **hermetic** actions (non-hermetic actions poison the cache); **precise, not coarse**
cache keys; merge-queue batching to amortize the whole-tree cost. **Surface-all is preserved at every
cache layer** — a HIT still executes + reports the cached result; nothing short-circuits a finding.

### D5. The owned oya-ci/oya-cd remains the destination (build-first-cutover-later)
The owned `oya-ci`/`oya-cd` is still built + maintained — a bespoke Rust-native cloud-native CI/CD
product reimplementing the **patterns** of Prow + Tekton + Argo Workflows + Argo CD + Argo Rollouts
("do what Go does, cloud-native, in Rust"), homed as tenant-facing dogfood products under
`cloud/cloud-scm`, `cloud/cloud-ci`, `cloud/cloud-cd` (D-PURESPLIT: a service dir exists only under
`oya/` or `cloud/`, exactly once; no `oya/`→`cloud/` internal dependency). It depends on (does not
absorb) the Buck2 build substrate (ADR-0392). It runs the **same** canonical gate logic as the
live GitHub Actions runner (owned-runner readiness), so the GitHub→owned cutover is a **runner-swap of
the one canonical pipeline**, validated at swap time — **out of this campaign's scope**. The forge
ratchet is GitHub-interim → bespoke Sapling-inspired `cloud/cloud-scm` (ADR-0510 cutover-trigger;
Forgejo is dropped entirely, not a bridge).

## What this supersedes

| Item | Fate | Retained substance (re-homed into this ADR / git history) |
|---|---|---|
| **ADR-0513** bespoke-Rust Prow clone | Superseded | the bespoke-Rust + own-the-build-graph-brain instinct → D5 (owned destination); "Prow-native authority, GitHub = shadow" framing dropped (→ D-CICD-AUTHORITY) |
| **ADR-0514** target-arch / hyperscaler remediation | Superseded | the throughput / affected-gate / trunk-sourced-gate / structured-failure-summary items → D2/D4 (Phase-1 work-items) |
| **ADR-0511** Argo-Workflows wholesale | Superseded | the DAG / event-correlation **ideas** → owned-destination DAG face; etcd-CRD substrate rejected |
| **ADR-0349 / 0359 / 0361** Jenkins ± ArgoCD substrate | Superseded | none as authority; **jenkins = drop now** (no bridge); the license-vetted supply-chain tool stack (cargo-deny / gitleaks / Syft / Trivy / osv / cosign / SLSA) → pipeline gate steps; resolve the `byp_adr_0349` bypass record |
| **ADR-0124** own webhook merge-queue | Superseded-in-mechanism | merge-queue intent + the blocker taxonomy → GitHub-native `merge_group` interim → owned Tide; file-overlap clustering → graph-exact `conflicts(a,b)` |
| **Multispectrum-review enforcement** (21-facet doctrine; the legacy bridge tie + the Proposed 0327/0323/0322/0247 framings) | Superseded (accounting half) + Dropped (critique half) | accounting/structural half → this firewall + one-canonical-CI; per-changeset multi-lens critique → **dropped, no replacement** (recorded loss of ADR-0322 anti-template-stamping + ADR-0247 SOC2 CC8.1 self-modification attestation) |
| **ADR-0092** workspace dependency-seam policy | **Amended** (not superseded) | keep the 3 mechanical seam subchecks; **strip the 3 multispectrum subchecks** before the `oya-check-dependency-seam` lane logic is removed |
| **ADR-0392** Buck2 build substrate | **NOT superseded — depends_on** | the build substrate stays authoritative as a separate bounded context; ADR-0525 carries the later orchestration refinement |

Per **D-SSOT-CURRENT-TRUTH**: the superseded ADR files are hard-destroyed (`git rm`; git history is the
sole archive), every dangling reference to a deleted id is scrubbed (no-dangling satisfied by full
excision, not tombstones), and the producer is re-generated so the accounting registry + gate baselines
stay consistent — executed under a read-only kill-list manifest with per-batch door:one-way founder
review and a `grep`-verified zero-references check after each batch.

## The five 2026-06-07 posture rulings (recorded)

1. **GitHub Actions = sole CI authority** (D-CICD-AUTHORITY). Branch-protection required checks are the
   only thing that gates merges; nothing else has merge authority until an explicit, proven cutover.
2. **One canonical CI** (ONE CANONICAL CI amendment). One gate-logic + one pipeline + one aggregator →
   one blocking `oya-ci-required`; runner migration over time, not two parallel CIs; the shadow + verdict-
   agreement apparatus is dropped.
3. **Cloud-native, not CLI** (D-CLOUD-NATIVE). Gates are Rust binaries run by the pipeline; **no shell**;
   declarative gitops; jenkins dropped now; `forge` scrubbed and `infra/forge → infra/gitops`; the `oya`
   CLI retired with no revival.
4. **Central governance** (D-GOVERNANCE-CENTRAL). One PaC/CaC/PaaS/CaaS authority; the keystone gates
   are CaC, the pipeline is CaaS — owned + dogfooded.
5. **Multispectrum retired** (D-MULTISPECTRUM-RETIRED). Accounting half superseded by this substrate;
   critique half dropped (recorded loss); ADR-0092 amended; GATE-4 / AC-0.12 reviewer-multispectrum rows
   de-required in lockstep so GATE-4 does not red on an unbacked row.

## Alternatives considered

- **Two CIs / a parallel non-blocking shadow run + verdict-agreement** — rejected by ONE CANONICAL CI:
  one pipeline + one context means a shadow has nothing to shadow; verdict-agreement was confidence-
  building, not enforcement.
- **Re-pointing the SSOT's "shadow compatibility only" bridge context to authority-of-record** —
  rejected: the committed SSOT explicitly forbids treating that name as merge authority; the honest move
  is the GitHub-Actions-produced canonical `oya-ci-required`.
- **Keeping governance/evidence on the `oya gate` CLI + shell/groovy lanes** — rejected by
  D-CLOUD-NATIVE: CLI/shell is not the interaction model; the platform is automatic + declarative.
- **Retaining Jenkins as a build-first-cutover bridge** — superseded by the 2026-06-07 ruling
  (jenkins = drop now; GitHub Actions is the sole CI). This deliberately overrides the earlier 2026-06-06
  "Jenkins stays operative-but-unratified until oya-ci is proven" clause.
- **Re-homing the multispectrum critique into a cloud-ci reviewer-status producer** — rejected: the
  founder chose to drop the critique rather than build a reviewer-status producer (recorded loss).
- **Per-axis multiple required contexts** — rejected by the `oya-pr-review` HTTP-501 multi-producer-
  deadlock lesson; one fan-in context with per-finding annotations instead.
- **Hand-editing `gate-baseline.generated.json` to flip advisory→blocking** — rejected: it drifts the
  producer face and trips registry-drift; the flip is a signed disposition + producer-regen + byte-parity.
- **Coarse whole-tree cache keys** — rejected: a coarse key collapses delta-only-cold back to whole-
  repo-cold; precise per-target keys + hermetic actions are required.

## Consequences

**Positive.** One true blocking gate that stops lying; one-sweep surface-all; drift caught by
enforcement even if automation fails (defense-in-depth); no silent-skipped gate (registration meta-test);
throughput that tracks change-size not repo-size (warm-cache + delta-only-cold), every cache layer still
reporting; central, owned, dogfooded governance; the shadow + verdict-agreement apparatus eliminated
(less to build/maintain). The canon collapses from a seven-ADR contradictory cluster to one current-truth
ADR — drift made structurally impossible, not merely discouraged.

**Negative / cost.** The go-live requires **three founder-paired GitHub-admin one-way doors** (the
inherently-manual irreducible set — see below); advisory→blocking flips must be infra-sequenced AND
producer-regenerated or they deadlock/drift; hash-based affected-selection can over/under-include and
must be validated empirically against a periodic whole-graph run; the multispectrum critique is dropped
with no replacement (a recorded substance loss); a fresh-root import (topology B) is manual and
provenance-lossy and spans committed + uncommitted sources (mitigated by filtering + the brand-residue
gate). Dropping Jenkins now (no bridge) means GitHub Actions must carry the full gate load from day one.

**Neutral.** The Buck2 build substrate (0408/0392) is unchanged (depended-on, not absorbed); the gate
*logic* is identical across the GitHub-Actions and owned-oya-ci runners (a runner migration, not a logic
change).

## Go-live sequence (firewall-first; the three founder-paired branch-protection doors)

Per D-SEQUENCE, enforcement becomes real *before* the canon is fixed through it. The non-admin work is
`[automatable-now]`; the three live-protection changes are HALT → founder-paired GitHub-admin
`door:one-way` steps (`GITHUB_TOKEN` cannot self-request Administration-write), executed in **one
sitting** to bound the half-hardened window:

1. **Stand up real enforcement (automatable-now).** Land the firewall substrate + the
   `oya-cloud-ci-firewall-app` aggregator bin (with the TAMPER/empty-input-exits-non-zero test and the
   gate-registration-completeness meta-test) + the zero-build `oya-ci-required` fan-in + the
   branch-protection drift gate (report-only until the SSOT re-author lands and the live state is
   hardened — its soak target is "live == re-authored SSOT") + kill within-lane fail-fast. Prove the
   fan-in GREEN on a real PR. Topology **B**: a thin fresh-root-import PR lands the canonical pipeline
   source (committed `phase0/producer` + uncommitted working-tree) plus the re-authored SSOT onto `dev`.
2. **Door 1 — SSOT re-author off ADR-0513.** A framing change to `infra/branch-protection/dev.json` +
   `.github/branch-protection.yaml`: KEEP the `oya-ci-required` context name; change only the
   producer/authority story to "produced by GitHub Actions now = the live blocking authority; owned
   `oya-ci` runner later." Reconcile the two-branch SSOT divergence onto one D-CICD-AUTHORITY SSOT. This
   is the prerequisite that gives the drift gate a zero-able target (else it is RED-on-arrival forever).
   *(Already substantially reflected in the committed `dev.json` — confirm parity.)*
3. **Door 2 — required-context IDENTITY change.** On live `dev`, make `oya-ci-required` the
   required-context-of-record and retire the `github-lane-unlocker-required` bridge name. (Per the
   jenkins=drop-now ruling, this same sitting also removes any jenkins-produced required contexts.)
4. **Door 3 — branch-protection security hardening to match the re-authored SSOT.**
   `enforce_admins:true`, `required_signatures:true`, review-count reconciled to `0/null` (the SSOT's
   deliberate guard against the queue-stalling approving-reviews blocker). This is security-to-match-SSOT,
   **NOT** an authority flip (GitHub stays authority). Confirm SSH `id_ed25519` commit signing is live on
   the merge-bot first. Only after all three doors does the drift gate flip blocking.
5. **Then fix the canon through it (Phase-1+, each gate-verified).** Hard-destroy the superseded cluster
   under the kill-list manifest; amend ADR-0092; de-require the GATE-4/AC-0.12 multispectrum rows;
   re-home generators off the retired CLI; eradicate forbidden vocab (`forgejo`/`forge`/`foundry`/
   `jenkins`/`oya-vcs`, Palantir-Foundry carve-out) tree-wide via the brand-residue gate; consolidate
   `infra/forge → infra/gitops`.

**Hard constraints (restated):** push **github-mirror only** (`origin` = the retired Forgejo remote —
superseded per ADR-0363, kept only as a do-not-push tripwire — never push there); all
commits **signed**; **mutate nothing in source until sign-off** — this is a DRAFT; no blind
`git add -A`; verify each step in a separate verifier lane (no self-approval).

## Verification

Enforcement is proven, not asserted (D-DOCTRINE robustness bar — every gate has a known-bad it fails +
a known-good it passes + proof it runs in the pipeline and BLOCKS):
- **Firewall data predicate (unit):** `firewall_is_green_on_the_live_corpus_with_the_baseline` (GREEN);
  `firewall_goes_red_on_a_synthetic_new_violation` + the new `firewall_goes_non_zero_on_tampered_or_empty_input`
  TAMPER test (RED / tooling-error). `compare()` no early-exit; `is_green()` = no failing code AND no
  un-signed-off ratchet growth.
- **Aggregator (integration):** seed one lane RED → the bin lists it and exits non-zero, the fan-in goes
  red; the gate-registration meta-test fails on an in-tree-but-unregistered gate; the workflow meta-test
  fails if a lane is dropped from `needs:`.
- **E2E (GitHub Actions presubmit):** a PR with a new violation + a protection drift shows BOTH in one
  Check-Run summary and blocks; a clean PR is green in one pass with file:line annotations.
- **Phase-0 producer contract:** the `oya-ci-required` producer posts a `Phase0CiEnforcementResult` on
  the candidate PR-head SHA per `/specs/phase0-ci-enforcement-result-schema.json`, with the
  `/specs/phase0-ci-enforcement-baseline.json` fixture set (GREEN
  `tc-0.0-good-cloud-ci-required-and-isolated`; the `tc-0.0.1*`/`tc-0.0.2`/`tc-0.0.3` RED fixtures each
  MUST fail — bad-producer, legacy-CLI-authority, missing/not-required context, candidate-mutable
  producer, override-without-TTL-audit, cross-tenant-shared-cache).
- **Claim ceiling (until live receipts exist on real SHAs):** gap-packet language only — no "Phase 0
  complete" / "P0.0 green" / "mechanically enforced" / "production-ready" until the producer posts on
  real candidate SHAs AND the live ruleset requires `oya-ci-required` (closing the SSOT→live drift); only
  then may `claim_boundary.p0_0_green` flip true.
- **G015/G012 readiness evidence registration (2026-06-26):** the repo-relative evidence artifacts
  `evidence/quality-gate/g015-wave-a-m0-m4-g012-readiness-evidence-20260626.json` and
  `evidence/multispectrum/g015-wave-a-m0-m4-g012-readiness-evidence-20260626-1782515430.json` are
  non-authoritative Wave A team readiness evidence for the G015/G012 closeout. They are justified only as
  audit/readiness records for the `oya-ci-required` governed path; they do not create merge authority,
  bypass `oya-ci-required`, or mutate generated artifacts.
- **CRATEADR-002A governance-check-gates owner-batch fixture registration (2026-07-01):** the
  repo-relative fixture artifact
  `specs/fixtures/crate-adr-design-doc-coverage/tc-CRATEADR-002A-good-governance-check-gates-owner-batch.json`
  is non-authoritative RED/GREEN corpus evidence for the crate-ADR design-document coverage gate. It is
  justified only as fixture coverage for the `governance-check-gates` owner-batch scenario; it does not
  create merge authority, bypass `oya-ci-required`, or mutate generated artifacts.
- **CRATEADR-002B ci-control-plane owner-batch fixture registration (2026-07-02):** the
  repo-relative fixture artifact
  `specs/fixtures/crate-adr-design-doc-coverage/tc-CRATEADR-002B-good-ci-control-plane-owner-batch.json`
  is non-authoritative RED/GREEN corpus evidence for the crate-ADR design-document coverage gate. It is
  justified only as fixture coverage for the `ci-control-plane` owner-batch scenario; it does not create
  merge authority, bypass `oya-ci-required`, or mutate generated artifacts.
- **CRATEADR-002D billing-metering-reorg owner-batch fixture registration (2026-07-02):** the
  repo-relative fixture artifact
  `specs/fixtures/crate-adr-design-doc-coverage/tc-CRATEADR-002D-good-billing-metering-reorg-owner-batch.json`
  is non-authoritative RED/GREEN corpus evidence for the crate-ADR design-document coverage gate. It is
  justified only as fixture coverage for the `billing-metering-reorg` owner-batch reorg-target scenario;
  it records current REORG-003/ADR-0562 successor evidence and does not create merge authority, bypass
  `oya-ci-required`, authorize another move, or mutate generated artifacts.
- **GJC governance-integrity Dispatch checkpoint evidence registration (2026-06-28):** the repo-relative evidence artifacts
  `evidence/goals/governance-integrity-dispatch-command-evidence-20260628.json`,
  `evidence/goals/governance-integrity-dispatch-ledger-20260628.json`,
  `evidence/goals/governance-integrity-dispatch-quality-gate-20260628.json`, and
  `evidence/goals/governance-integrity-dispatch-validation-20260628.json` are Dispatch-control evidence for the
  approval-gated G1/G2 lane claim checkpoint. They are justified only as audit/quality-gate records under this
  `oya-ci-required` governed path; they do not create merge authority, bypass CI, or authorize implementation.
- **Pre-planning authority-closure evidence registration (2026-07-13):** the repo-relative artifact
  `evidence/consolidation/preplanning-authority-closure-20260713.json` is non-authoritative inventory,
  lifecycle-reconciliation, and admission evidence for the founder decision hold before a planning session.
  It is justified only as an audit record under this `oya-ci-required` governed path; it does not make a
  planning choice, authorize dispatch or implementation, prove product readiness, bypass CI, or mutate
  generated artifacts.
- **Review-admission GitHub adapter ownership and regression registration (2026-07-20):** the
  repo-relative ownership marker
  `oya/ci-controller/crates/oya-ci-controller-github-adapter/OWNERS` assigns the adapter crate narrowly
  to `cloud-ci-platform`, and
  `oya/ci-controller/crates/oya-ci-controller-github-adapter/tests/review_admission.rs` is its
  repo-owned regression suite. These paths implement and test only the non-live `oya-pr-review`
  producer contract; they do not deploy the producer, modify branch protection, bypass
  `oya-ci-required`, or claim F-PR5-06 closure.

## Amendment (2026-06-08, WAVE-1 Agentic Delivery Fabric convergence — refined, NOT superseded)

ADR-0515 remains the **governing floor** and stays Accepted. The WAVE-1 convergence (ADR-0516
umbrella) refines this ADR in place without a tombstone; the prior text above stands and git history
preserves the pre-amendment body. The refinements:

- **ADR-0516** names this ADR's firewall + one-canonical-CI (`oya-ci-required`) as the **W0 floor** of
  fabric Component 2 (not the whole of it).
- **ADR-0519 / ADR-0529** extend the born-blocking/advisory model with the AUTO/ADVISE/GATE
  per-finding-code tier as schema-enforced DATA in `gate-disposition.json`, the untagged-code
  meta-gate, and the five-property AUTO-promotion proof.
- **ADR-0522 / ADR-0523** make the lifecycle-wide hermeticity and the closed irreducible-glue ledger
  precise — ADR-0523 refines D3's "no shell bar a narrow exception" into the closed five-item ledger.
- **ADR-0525** supplies the concrete hermetic buck2 execution model (git-facts boundary, buck2-native
  gates, RBE/CAS) that operationalizes D2/D3/D4/D5; **ADR-0526** renames that boundary to scm-facts
  (amends D3 vocabulary).
- **ADR-0527 / ADR-0530** add the engine-vs-policy seam and the engineering-excellence floor gates;
  the firewall predicates stay byte-unchanged.
- **ADR-0528** adds `remediate()` to the gate contract (WS-D) this ADR established.

None of these reverses a decision of ADR-0515; it is the governing floor the fabric builds upon.

---
*Accepted 2026-06-07 (founder-ruled; door:one-way). Authority: D-CICD-AUTHORITY · ONE CANONICAL CI · D-CLOUD-NATIVE · D-GOVERNANCE-CENTRAL · D-MULTISPECTRUM-RETIRED · D-SSOT-CURRENT-TRUTH · D-SEQUENCE · D-DOCTRINE (decision-record-oyatie-canon.md). Plan: CICD-DESIGN-PLAN.md (ralplan-APPROVED). Reciprocal `superseded_by` edges written on 0124/0349/0359/0361/0511/0513/0514; 0092 amended; ADR-0392 is the Buck2 dependency and ADR-0525 carries the later CI/CD refinement. Refined (not superseded) 2026-06-08 by the WAVE-1 fabric cluster ADR-0516/0519/0522/0523/0525/0526/0527/0528/0529/0530. D-SSOT-CURRENT-TRUTH still requires hard deletion with git history as the archive. The repository's present retention of superseded ADR files is implementation drift, not a silent amendment; changing that rule requires a higher Accepted founder-ratified ADR.*
