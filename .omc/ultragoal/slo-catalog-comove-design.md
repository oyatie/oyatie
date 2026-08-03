# SLO/catalog co-move + liveness gate + backfill — implementation plan (architect design, 2026-06-17)

Drives the doctrine-fix phase (founder: fix BEFORE move-10). Closes the 2 HIGH cloud-native audit findings:
(a) moves strand promotion-gating SLOs at dead stems; (b) slo-coverage gate false-green (no crate-liveness).
SINGLE SOURCE OF TRUTH for all of this: `specs/capability-registry.json` `absorbs_current_dirs` (read it, don't reinvent mappings).

## Decisions (sound, doubt-driven)
1. **SLO home = capability-rooted `<cap>/observability/slos/*.openslo.yaml`** (NOT per-crate). SLOs are per-SERVICE/promotion-unit; capability = the ADR-0139 "component" post-reorg. `<cap>/observability/` (named sub-tree, OWNERS-scoped, NOT under a core/ports/adapters/facade face fold — it's non-crate data). Multi-facade caps may later refine to `<cap>/observability/slos/<facade>/` (doesn't change the discovery root). ADR-0139's `SloTargetRepository` adapter is NOT yet implemented (only kernel types at observability/core/domain/src/slo.rs) — so the discovery root is uncommitted; setting this convention now is cheap + correct.
2. **Catalog records: keep FLAT under `registry/catalog/`, RE-KEY to live de-branded crate-id** (rename `<old-crate-id>.yaml`→`<new-crate-id>.yaml` + update in-file capability/identity fields). The flat registry is the contract slo-coverage producer + cohesion gate + sell-catalog projection (ADR-0562 §5, a generated view over face:facade) all read by crate-id; moving would fork it. Re-key fixes all 3 consumers' inputs.

## PR-A — codemod ArtifactMove co-move (engine fix; unblocks move-10 + backfill)
`tools/oya-reorg-codemod-app/`:
- model.rs: add `ArtifactMove { old_path, new_path }` + `pub artifacts: Vec<ArtifactMove>` on MovePlan (default empty → existing plans/tests unaffected). Extend `MovePlan::validate` to cover artifacts (no old/new collisions across moves+artifacts; reuse is_normalized_rel_path). Add `artifact_file_pairs(tracked)`: for a dir artifact enumerate NEW-dir descendants→old (mirror crate logic ~model.rs:165-176); for a single file emit the (old,new) pair directly. Merge into `move_manifest_value` files list (so ADR-0563 relabel + total-accounting follow). `inverse()` swaps artifact pairs too.
- plan.rs: add step-8 after step-7 crate `git mv` (~:133-139): `git mv` each ArtifactMove old→new (reuse move_dir ~:424-462, mkdir -p parents); append to outcome.dirs_moved. walk_repo_files unchanged (artifacts moved wholesale; SLO/catalog YAML carry no cargo/buck/rust idents to rewrite — content-preserving per ADR-0563 §C2).
- main.rs load_plan (~:240-261): parse optional `artifacts: [{old_path,new_path}]` (absent → empty, back-compat with the 4-field marketplace plan).
- Tests: artifact co-move applies + manifest carries pairs + validate rejects collision + inverse round-trips + empty-artifacts no-op (existing plans unaffected).
- ADR: amend ADR-0139 (or ADR-0562/0563-sibling) recording the SLO home convention (`<cap>/observability/slos/`) + that the future SloTargetRepository discovery root derives from capability-registry absorbs_current_dirs.

## PR-B — backfill the 9 moved caps (depends on PR-A)
Codemod artifact-ONLY plans (zero crate moves) per cap; SLOs → `<cap>/observability/slos/`, catalog re-key in-place. Drive enumeration from absorbs_current_dirs + the historical move-manifests' crate_idents (catalog re-keys are machine-derivable, e.g. oya-cloud-observability-domain→observability-domain).
Orphaned SLO stems → new home (verify counts in-worktree):
- observability: oya/observability/slos(9), oya/diagnostics/slos(9) → observability/observability/slos
- storage: cloud/cloud-storage/slos(1), oya/drive/slos(10), oya/imaging/slos(14), oya/recordings/slos(11) → storage/observability/slos
- compute: cloud/cloud-compute/slos → compute/observability/slos
- gateway: oya/api-gateway/slos(9), oya/connector/slos(6) → gateway/observability/slos
- marketplace: oya/marketplace/slos(14), oya/plugin-app-store/slos(10), oya/developer-sdk/slos(10) → marketplace/observability/slos
- flags: oya/feature-flags/slos(6), oya/oya-flags/slos(1) → flags/observability/slos
- cell: cloud/cell-lifecycle/slos(1), cloud/cell-rebalancer/slos(1) (+cloud-cell/cloud-capacity if present) → cell/observability/slos
- iac: cloud/cloud-iac/slos(7) → iac/observability/slos
- messaging: none (oya/eventing pure-crate)
⚠ CROSS-SERVICE SLO-NAME COLLISIONS when merging multiple absorbed services into one <cap>/observability/slos/ (e.g. shared autosharding-events.openslo.yaml) — resolve (prefix by source-service or merge); MovePlan::validate dup-new_path fail-closed is the backstop. Grouping: one SLO-backfill PR for all 9 + per-cap catalog re-key PRs, OR per-cap atomic (reviewer preference).

## PR-C — catalog-liveness gate + slo-coverage tightening (depends on PR-B; backfill-first)
- New `cloud/cloud-ci/gates/oya-cloud-ci-catalog-liveness-app/src/lib.rs` (mirror slo-coverage-app pure evaluate_keyed). Predicate: every registry/catalog/<stem>.yaml stem ∈ live workspace crate-ids. USE THE IN-PROCESS workspace-member resolver (read_workspace_member_crate_ids / oya_check_cohesion resolver — the same oracle validate_cohesion_gate trusts) — NOT a `cargo metadata`/buck shell-out (shell-free per all-CLI-retirement; also buck2-denied-in-review-sessions). Evaluate over CANDIDATE tree (gate-baseline PR/push-asymmetry lesson). Register in oya-ci.toml `[[gates.enabled]] id="cloud-ci-catalog-liveness"` + `[catalog_liveness]` reusing catalog_record_globs (born-pack DATA).
- Tighten slo-coverage: also require row crate_id be LIVE (compose) → closes the false-green at the original surface too.
- Baseline: backfill-FIRST (PR-B), THEN gate born-blocking with EMPTY frozen baseline (no stale stem accepted — matches slo-coverage frozen_empty). Do NOT baseline-existing-stale (would violate no-false-green, the very thing we're fixing).

## Principle flags (HARD)
- NEVER backfill-then-baseline-stale (false-green). Backfill first, empty baseline.
- NEVER a live cargo-metadata shell-out in the gate (shell-free). In-process workspace resolver only.

## PR-C SUPERSEDED — FOUNDER DECISION 2026-06-18: "live-OR-explicitly-marked" (NOT strict empty-baseline)
Categorization (workflow w7semoq2e, full plan at .omc/ultragoal/catalog-dead-stem-dispositions.json): 220 dead catalog stems = 46 REKEY (reorg-renamed crates → live de-branded ids) + ~173 non-live, which split into 3 classes:
  - class-1 SILENTLY-STALE: the renamed live crate is ALREADY cataloged under its current name (superseded duplicate), OR a reorg-dropped no-crate record that is NOT status-marked → DELETE (genuine garbage).
  - class-2 SELF-MARKED retired-compatibility (records that carry status: retired-compatibility-row-no-crate / non_claims 'no matching crate') → KEEP (deliberate markers).
  - class-3 ASPIRATIONAL designed-no-crate (e.g. oya/social/catalog/ with no crate; design-ahead roadmap) → KEEP, but ENSURE status-marked (add status: planned/aspirational if missing).
GATE PREDICATE (live-OR-marked): a registry/catalog/<stem>.yaml is OK iff stem ∈ live workspace crates OR the record carries an explicit non-live status field. RED = silently-stale (no live crate AND no status marker). This catches the real false-green (records claiming SLO coverage for nonexistent crates) while preserving compatibility-markers + design-ahead roadmap.
PR split (reviewable; deletes are destructive → independent review verifies every delete is genuinely class-1):
  - **PR-C1**: reorg-caused catalog fixes (62): rekey the 46 (rename old-id.yaml→<live-id>.yaml via codemod ArtifactMove so ADR-0563 relabel + total-accounting follow + content-edit capability/identity fields) + delete the 16 reorg-dropped-no-crate (class-1).
  - **PR-C2**: pre-existing catalog hygiene (~158): re-bucket per evidence — DELETE class-1 superseded-duplicates (live crate already cataloged under new name); KEEP class-2 (already status-marked); KEEP+status-mark class-3 aspirational. (worktree: read .omc/ultragoal/catalog-dead-stem-dispositions.json; the agents' evidence strings carry the class signal.)
  - **PR-C3**: catalog-liveness gate (live-OR-marked predicate, in-process workspace resolver, candidate-tree) + slo-coverage tightening (require live-OR-marked) + ADR-0139 born-record refresh (marketplace included / 9 caps). Born-blocking after C1+C2 (zero silently-stale remain). Plus create the missing observability-aggregate catalog record (live crate, no record — flagged by the categorizer).
HUMAN-CHECK items from categorization: observability domain-pair (oya-observability-domain rekey vs oya-cloud-observability-domain delete — both map to one live observability-domain; keep the de-branded-identity one). cloud-* unmoved cluster (13): verify none are still-live-unmoved before delete.

## After this workstream: pipeline-glue (also before move-10) — close rust-first YAML-inline-shell ratchet blind spot + productize FRIC-017 (data-driven preflight + infra-red/code-red label). See [[reorg-doctrine-audit-fixes]].
