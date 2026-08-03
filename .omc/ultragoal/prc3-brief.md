# PR-C3 executor brief — catalog-liveness gate (doctrine-fix capstone)

Dispatch AFTER PR-C2 (#751) push-tier verifies GREEN. Worktree off the post-PR-C2 dev tip.
Closes the catalog-liveness false-green permanently + completes the doctrine-fix phase (last item before resuming capability moves at move-10).

## Goal
Make the founder policy **live-OR-explicitly-marked** mechanically enforced: every `registry/catalog/<stem>.yaml` is OK iff its stem ∈ live workspace crate-ids OR the record carries an explicit non-live marker. PR-C1+PR-C2 brought silently-stale records to ZERO, so the gate can be born-blocking with an EMPTY frozen baseline (no accepted debt).

## Scope (one PR)
1. **New gate** `cloud/cloud-ci/gates/oya-cloud-ci-catalog-liveness-app/` — mirror the existing born-blocking pure-evaluator pattern of `oya-cloud-ci-slo-coverage-app` (read its lib.rs: pure `evaluate_keyed(face)` + `evaluate(face).verdict` + `empty_corpus_is_red`; its producer-face wiring; its tests/slo_coverage.rs structure).
   - **Predicate**: for each `registry/catalog/<stem>.yaml`: RED iff (stem ∉ live workspace crate-ids) AND (record carries NO explicit non-live marker). Marker = a `status:` field with a non-live value (retired-compatibility-row-no-crate / designed-ahead-row-no-crate / planned / aspirational) OR a `non_claims` entry indicating no-crate. (Match exactly the markers PR-C1/PR-C2 used so all 8+34 marked records pass.)
   - **Live resolver**: IN-PROCESS via `libs/oya-workspace-members-kernel` (`read_workspace_member_crate_ids` — the same oracle the cohesion gate + the PR-C2 worklist used). NEVER a `cargo metadata`/`buck2` shell-out (all-CLI-retirement + buck2-denied-in-review-sessions + hermetic). 
   - **Candidate-tree eval** (gate-baseline PR/push-asymmetry lesson): evaluate over the candidate tree, not a frozen baseline of stems.
   - Born pack-shaped: policy-as-data in oya-ci.toml (reuse catalog_record_globs); no hardcoded oyatie paths in the gate logic beyond the pack.
2. **Register** the gate: oya-ci.toml `[[gates.enabled]] id="cloud-ci-catalog-liveness"` (+ any `[catalog_liveness]` policy block reusing the catalog globs), the firewall gate-registration meta-test, and the materialized faces (producer-regen + scm-facts + accounting-registry as needed). EMPTY frozen baseline (frozen_empty, like slo-coverage) — post-PR-C2 there are zero silently-stale, so empty is achievable; do NOT baseline-existing (that would re-accept debt = false-green).
3. **Tighten slo-coverage**: in oya-cloud-ci-slo-coverage-app, additionally require each row's crate_id be live-OR-marked (compose with the new predicate) → closes the original false-green at its own surface too. Keep the existing per-row `slo:` assertion.
4. **ADR-0139 born-record refresh**: update the ADR-0139 born-record / discovery-root doc to reflect the 9 moved caps + marketplace (the SLO home convention `<cap>/observability/slos/` + the catalog-liveness predicate). Verbatim-path-justify any new committed spec file (PR-C1 lesson: justification_ref via ADR path-mention + reachability seed, or firewall unjustified RED).

## Verify (cold + dev-cli gates — MANDATORY per the #749 lesson)
- buck2 test //cloud/cloud-ci/... → READ output: new catalog-liveness gate test PASS on the live corpus (verdict Green, empty findings), firewall GO-LIVE GREEN incl the new gate registered, total-accounting 0-regression, registry-drift committed==regenerated, slo-coverage still GREEN with the tightened predicate. RED-fixture test: synthesize a fake dead-unmarked record → gate goes RED (proves discrimination, not inert).
- **dev-cli gates**: `oya gate validate supply-chain --require-adr0039-evidence` exit 0 + `oya gate validate loop-recovery-patterns` exit 0 (the graph-invisible class).
- MATERIALIZE-LAST: run materialize as final step, commit faces, re-run, git status CLEAN.
- cargo metadata --locked clean; conflict-marker sweep.
- Born-blocking meta: the automation-ratchet / gate-registration meta-test must show the new gate is registered + born-blocking (not advisory).

## Then
Commit (trailer Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>), push, open PR vs dev. Report PR# + head SHA + the explicit gate-pass evidence (catalog-liveness GREEN + RED-fixture proof + firewall GO-LIVE + supply-chain/loop-recovery exit 0).

## Independent review focus (after build)
- The empty frozen baseline genuinely accepts ZERO stale stems (not baselined-existing).
- In-process resolver (no shell-out); candidate-tree eval (not frozen-stem-set).
- RED-fixture proves the gate discriminates (not inert/always-green).
- slo-coverage tightening doesn't false-RED any live-or-marked record.
- The marker predicate matches ALL the markers PR-C1/PR-C2 used (no kept record false-REDs).

## Backlog spun off (NOT in PR-C3 unless trivial):
- Catalog COMPLETENESS gap (task #70): ~158 live crates with NO record (oya/office/* reference-org tree, oya-cloud-os-*, self-gates). The liveness gate checks catalog→live; the inverse live→catalog is separate. Decide PR-C4 vs defer.
- oya/social/catalog/ parallel 23-record tree outside the gate-enforced registry/catalog/ surface (pre-existing, unenforced) — fold into the gate's scan_roots or migrate.
- slo-coverage floor magic-number (500) → data-derive from actual catalog count.
