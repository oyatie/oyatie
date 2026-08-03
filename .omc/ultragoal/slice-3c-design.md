# Slice 3c+3d design — RegenPort + self-validate for register_crate (workflow w4rhlps1j, 2026-06-21)

Target: cloud/cloud-ci/gates/oya-cloud-ci-register-crate-app (origin/dev @ 221c919a7). Verified on origin/dev.

## Central decision: subprocess RegenPort, NOT in-process build_* calls, NOT shell-to-materialize.sh
Root cause: the producer's build_* fns ARE pub (lib.rs:281,449,545,792,929) but every INPUT is assembled by a
collect_* fn that is PRIVATE in main.rs (collect_repo_inputs:2233, collect_crosswalk_inputs:1955,
collect_enforcement_inputs:2093, collect_slo_coverage:614, collect_catalog_liveness:776,
collect_workspace_glob_coverage:823, collect_target_parity:897, collect_enforcement_liveness:932,
collect_bnf_layer_suffix:541, collect_manifest_hygiene:1719, collect_cargo_prefix:576, collect_brand_residue:477)
+ write_face:430 + load_scm_facts:84. The scm-facts emitter is [[bin]]-only (no lib). So in-process regen would
force promoting ~13 private fns OR reimplementing them = the exact drift registry-drift exists to catch.
Doctrine-blessed precedent: oya-cloud-ci-freshness-app subprocesses the built binaries
(regenerate_faces_with_buck2, lib.rs:726-790,1000-1029) — git/buck2 at the graph edge = sanctioned irreducible
glue (ADR-0523 ledger item 2 / ADR-0525 hermetic git-facts boundary). The orchestrator already shells git ls-files
(lib.rs:682). Command::new(built_binary) at the edge is NOT a new CLI surface — it's the established pattern.
gix/git2 rejected (no workspace dep, libgit2 C-FFI purity-hostile, separate initiative).

## (a) RegenPort — executed only when outcome.requires_faces_settle == true; each cmd .current_dir(repo_root)
1. buck2 build //…scm-facts-emitter-app:… //…accounting-registry-app:…-bin //tools/oya-reorg-codemod-app:oya-reorg-codemod --show-output; parse 3 output paths by target-name match (freshness build_face_tools ~726).
2. codemod: manifest --repo-root <root> [--plan specs/reorg/*-move-plan.json] --out specs/reorg/move-manifest.generated.json (first plan only; none ⇒ empty identity manifest). ORDER load-bearing (materialize.sh #64).
3. emitter: --repo-root <root> --out <faces_dir>/scm-facts.generated.json --merge-base-baseline (also writes frozen gate-baseline.merge-base.generated.json; --frozen-base-ref origin/dev reads ratchet policy AT merge-base, FRIC-1781280000). WRITE THE REAL <faces_dir>/scm-facts.generated.json — NOT a temp path (the INVERSE of freshness's read-only temp routing; copy-pasting temporary_scm_facts_path/TempFileCleanup would leave the committed scm-facts stale → registry-drift RED).
4. producer: --repo-root <root> --scm-facts <faces_dir>/scm-facts.generated.json → writes the 6 producer faces.
Missing scm-facts is a HARD producer error (main.rs:84) → emitter MUST run+succeed before producer; propagate failure as RegenFailed, never swallow.
RegenPort = trait in THIS crate's lib (no separate ports crate at 3c size). One Buck2RegenAdapter with explicit
`// IRREDUCIBLE-GLUE LEDGER (ADR-0523 item 2 / ADR-0525): git+buck2 at the graph edge`. materialize-…sh string
survives ONLY as the human remediation message (freshness FACE_REMEDIATION_COMMAND lib.rs:13). FakeRegenPort for unit tests.

## (b) self-validate (SLICE 3d) — minimal high-value subset, crate-scoped, FAIL-CLOSED
| gate | entry | when | scope |
| total-accounting | evaluate_keyed(&Value)->BTreeSet<Finding{code,key}> total-accounting-app/lib.rs:115; key=row path | always | findings key under crate_dir |
| capability-membership | collect(root,&policy)->Result<Value> :145 + evaluate_keyed(&policy,&observed)->BTreeSet<Finding{code,key,detail}> :431 | always | key==crate_dir |
| slo-coverage | evaluate_keyed(&Value)->BTreeSet<Finding> :99 | iff CatalogYaml edit applied | crate row |
| catalog-liveness | evaluate_keyed(&Value)->BTreeSet<Finding> :106 | iff CatalogYaml applied | crate row |
| registry-drift (byte-rediff) | no fn; re-run producer --stdout --face <name>, byte-compare | always (free) | per face |
FAIL-CLOSED = DIFF not assert-empty: before=evaluate_keyed(face_before_edits), after=evaluate_keyed(face_after_settle),
fail iff after.difference(before) has a finding whose key matches crate_dir. Each gate's Finding type DIFFERS
(total=2 fields, membership=3 incl. detail in Ord/Eq) — diff each BTreeSet separately, no shared Finding type.
SKIP merge-base gates in-process: firewall (evaluate_firewall firewall-app/lib.rs:570 needs frozen baseline — CI-tier's
job, duplicating risks divergent verdicts). EXCLUDE full-set crate-authoring gates (target-parity/manifest-hygiene/
cargo-prefix/bnf-layer-suffix/workspace-glob-coverage/cross-artifact) — register_crate doesn't author those, would
flag pre-existing debt. Keep for a future full-dry-run flag.

## (c) Outcome shape (extend, don't break lib.rs:108)
Outcome += faces_settled: Option<FacesSettled{faces_written:Vec<String>, drift_clean:bool}> (3c), validation: Option<SelfValidation{new_findings:BTreeSet<ScopedFinding{gate:&'static str,code,key}>}> (3d).
RegisterError += RegenFailed(String), DriftDetected{face:String} (3c), SelfValidationFailed{findings} (3d).
Keep register_crate/register_crate_detailed byte-compatible. NEW entrypoint:
register_crate_and_settle(repo_root, req, regen: &dyn RegenPort, validate: ValidationMode{Skip,MinimalSubset}) -> Result<Outcome,RegisterError>.
3b's register_crate stays subprocess-free (unit tests unbroken); settle+validate explicit + injectable (FakeRegenPort).

## (d) SPLIT: 3c (RegenPort+Buck2RegenAdapter+register_crate_and_settle(Skip)+FacesSettled/RegenFailed/DriftDetected+byte-rediff; FakeRegenPort unit tests + 1 integration test behind buck2 pre-approval) THEN 3d (SelfValidation+subset evaluate_keyed+before/after crate-scoped diff+ValidationMode::MinimalSubset+the 3 new gate deps).

## (e) dep-direction: SAME-LAYER legal (cloud-ci→cloud-ci). 3c adds NO new lib dep (shells to built binaries). 3d adds total-accounting-app + slo-coverage-app + catalog-liveness-app (producer already deps all 12 gates cycle-free; orchestrator already deps producer+membership). NO new crate — extend oya-cloud-ci-register-crate-app.

## Risks (top 3)
1. False-RED from whole-corpus evaluate_keyed (pre-existing frozen debt) → MUST before/after set-diff filtered by key under crate_dir; per-gate Finding types differ.
2. scm-facts temp-vs-real inversion — write the REAL scm-facts.generated.json (settle mutates on purpose), NOT freshness's temp; byte-rediff catches a mistake.
3. codemod→emitter→producer ORDER load-bearing; --frozen-base-ref origin/dev reads policy at merge-base; missing scm-facts = hard producer error → propagate as RegenFailed.

## SLICE-3c STATUS + review (2026-06-21)
PR #791 (agent/bornaccount-regen-settle, head e58932ee8): RegenPort trait + Buck2RegenAdapter +
register_crate_and_settle(repo_root,req,&dyn RegenPort,ValidationMode{Skip,MinimalSubset}) + FacesSettled +
RegisterError::{RegenFailed,DriftDetected} + verify_drift byte-rediff (all 6 faces). 18 unit (FakeRegenPort) +
1 ignored buck2 integration. LOCK_OK (no new dep), faces unchanged, clippy clean. Adversarial review APPROVE —
line-by-line verified regenerate replicates materialize.sh exactly (order/targets/args), writes REAL scm-facts
(not temp), fail-closed RegenFailed folds stdout+stderr, byte-rediff exact, register_crate backward-compatible,
MinimalSubset safe no-op.
NON-BLOCKING FOLLOW-UP (review suggestion, fold into slice 3d or fast-follow): Buck2RegenAdapter::regenerate has
ZERO execution coverage — only verify_drift is integration-tested. Add a buck2-gated MUTATING integration test:
run regenerate against a throwaway worktree, assert the 6 faces + scm-facts re-settle byte-clean. This is also the
ONLY thing that would exercise the --merge-base-baseline path (needs `git merge-base origin/dev HEAD` to resolve —
a real failure mode on shallow clones / detached fixtures, currently untested in this crate).

## SLICE-3d STATUS + review reconciliation (2026-06-21)
PR #792 (agent/bornaccount-self-validate, head f4d97bb97): fail-closed self-validate — SelfValidation/ScopedFinding +
RegisterError::SelfValidationFailed + ValidationMode::MinimalSubset runs total-accounting(on-disk)+capability-membership(live collect)+
slo/catalog(iff CatalogYaml, via new RegenPort::gate_input_face producer --stdout --face), crate-scoped, fail-closed.
23 unit + 1 ignored, LOCK_OK (3 gate deps added+regenerated), no face drift, clippy clean.
Adversarial review returned REQUEST CHANGES (1 BLOCKER + 2 MAJOR) — RECONCILED via live-data verification, all DISPROVEN/non-blocking:
- BLOCKER-1 (catalog file unjustified/unreachable → fail-open scope drop): FALSE ALARM. Verified the committed
  accounting face: 742/771 existing registry/catalog/*.yaml rows have justification_ref=null + reachable_from=[reachability-registry]
  and PASS on green dev. Catalog files are CORPUS-born-accounted: ADR-0555 D3 reachability-registry prefix "registry/catalog/"
  ("makes every NEW crate's catalog record born-reachable") + registry/catalog/OWNERS ancestor + registry unit_class needs NO
  per-file justification + ttl auto-classify. A new catalog file inherits all → total-accounting emits NO finding → crate_dir
  scope misses nothing real. Reviewer code-traced and missed the reachability PREFIX + the registry-class justification-exemption.
- MAJOR-3 (catalog leaf-collision cross-attribution): NON-EXPLOITABLE. Catalog namespace is leaf-flat (registry/catalog/<leaf>.yaml),
  so two same-leaf crates in different dirs CANNOT both have catalog files (same path collision). Leaf == the unique catalog key →
  leaf scoping is correct, not ambiguous.
- MAJOR-2 (tests seed fixture-only unaccounted_paths, not production rows[]): valid TEST-REALISM improvement, non-blocking
  (production code is correct; self-validate reads the real on-disk rows[] face). FOLLOW-UP: add a production-shaped total-accounting
  test (rows[] incl. a registry/catalog/<leaf>.yaml row with null justification + reachability-prefix coverage → asserts NO finding),
  mechanizing the BLOCKER-1 disproof. + a universality note: on a NON-oyatie repo lacking the registry/catalog/ reachability prefix,
  a new catalog file would be unreachable — the kernel may need to ensure catalog reachability for the universal case (refinement, not blocker).
VERDICT: #792 correct + fail-closed-safe → merge on green CI. Reviewer findings durably disproven above so they aren't re-raised.
