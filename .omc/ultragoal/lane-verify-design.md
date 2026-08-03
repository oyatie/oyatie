# G011 lane self-verify — design (ADR-0542, Proposed) — UNDER ADVERSARIAL REVIEW

Founder Stop-hook re-engagement 2026-06-21: "productize the pipeline; make it impossible to ship
anti-patterns through enforcement+automation; hermetic/universal/canonical; close the loop." This is
the response to this session's 4× friction (every auth/gate PR failed CI after buck2-green-local:
stale Cargo.lock → born-accounting → ADR justification → generated-faces settle).

Status: architect design DONE; adversarial design review IN FLIGHT (a38ba0042336b8921). Build only
after a SOUND-TO-BUILD verdict. One-way-door + planning_impact ADR → founder sign-off gates the
irreversible slices (2 workflow-generation + born-blocking parity gate; 5 supersession).

## Core idea
`oya-lane-verify` = NEUTRAL engine + policy-as-data pack (`specs/lane-verify-policy.oyatie.json`).
Runs the FULL oya-ci-required predicate set LOCALLY, byte-identically to CI, + ships auto-fixes →
green-by-construction before push. Reuses the EXACT buck2 predicate binaries CI runs (NO reimpl).

## Anti-drift (load-bearing): local==CI by construction
- The `.github/workflows/oya-ci-required.yml` matrix becomes a GENERATED VIEW of the pack.
- Born-blocking `lane-verify-parity` meta-gate asserts pack == workflow legs == gate_registration set
  (3-way agreement; precedent oya-cloud-ci-cross-artifact-agreement-app). Must ship in slice 2 or drift returns.
- Pack names the `-gate` GO-LIVE targets, never `-unittest` (closes friction #4: executors skipping the ratchet).

## Auto-fix partition (AUTOMATED property)
Auto-fix only decision-free/row-input-neutral: `lock_*` → `cargo metadata` (only allowed cargo subcmd);
`generated_face_stale` → `oya-cloud-ci-face-settle --settle` (faces-only). RED-with-gate-remediation on
human-decision items: new-crate registration/ADR-justification (scaffold stub, then RED), firewall
baseline-block-on-new, baseline growth w/o sign-off, dep decisions.

## Folds in OPEN FRIC-1781112000 (baseline laundering)
Pack `baseline_ref: "merge-base"` → engine resolves baseline faces via `git show <merge-base>:<face>`
(VcsPort), never the working tree → structurally blocks baseline-block-on-new laundering for the whole
family (firewall, manifest-hygiene, target-parity) in ONE place.

## Structure (clean-arch, per ADR-0132 single-concern; cloud-ci owns gate-exec per ADR-0515)
cloud/cloud-ci/oya-lane-verify-{core,ports,adapters,app}/ — core pure (mirrors firewall evaluate_firewall),
ports VcsPort/BuildPort/RegenPort/ClockPort(evidence-only), adapters git/buck2/materialize.sh (irreducible-glue ledger),
app = typed API (canonical) + lane-supervisor pre-pr-open call + CRD-shaped LaneVerifyRun + retirement-marked CLI/hook adapter.

## Supersede (3 contradictory prior attempts)
- libs/oya-check-pre-push (DEAD: mandates `oya verify --ci-required`→`gate run-all` aggregator that does NOT exist)
- scripts/agent-pre-push-validate.sh (cargo-shaped false-confidence; shell+Python+hardcoded aarch64 path)
- scripts/hooks/pre-push.sh (already retired the verifier path) → becomes one-line API-adapter hook

## 5 slices (leaf-first, each mergeable PR + own born-accounting)
1. oya-lane-verify-core (pure engine PolicyPack/Step/Verdict/evaluate) + specs/lane-verify-policy.{schema,oyatie}.json + RED/GREEN fixtures. ADDITIVE/reversible — buildable autonomously after review.
2. ports + born-blocking `lane-verify-parity` meta-gate + workflow generated-from-pack. ONE-WAY → founder sign-off.
3. adapters (git incl. merge-base baseline, buck2, RegenPort→materialize.sh) + integration: reproduce firewall GO-LIVE byte-for-byte.
4. auto-fix loop + fixable/RED partition (cargo metadata, face-settle, accounting scaffolder+ADR stub).
5. surfaces (typed API, lane-supervisor integration, CRD stub, hook) + SUPERSEDE the 3 priors. ADR Proposed→Accepted on founder sign-off.

## ADR-0542 key fields
id ADR-0542; status Proposed; door one-way; planning_impact true; deciders founder+council-architecture;
depends_on [ADR-0515,0539,0540,0132,0363]; milestone W0. Precedent: Bazel/Buck2 affected-targets,
OPA/Conftest policy-as-data, GH merge-queue projected-state (ADR-0111) merge-base baseline, k8s admission (CRD),
pre-commit autofix — all Rust-reimplemented citing precedent.

## Key file refs (verify before building)
.github/workflows/oya-ci-required.yml:92-104 (matrix) :361 (fan-in); oya-cloud-ci-firewall-app/tests/firewall.rs:214 (GO-LIVE) BUCK :gate target;
oya-cloud-ci-freshness-app/src/lib.rs:12-15 (auto-fix consts) :266 (face-settle); infra/ci/materialize-cloud-ci-generated-faces.sh (regen, 5 callers);
tools/oya-lane-supervisor-app (expected_surfaces integration); cloud-os .../admission.rs (CRD precedent); .omc/ultragoal/friction-ledger.jsonl (FRIC-1781112000 OPEN).

## Risks
1. Drift re-emerge if parity gate not born-blocking from slice 2 (highest). 2. Auto-fix masking real RED (restrict to lock+faces-only + merge-base baseline). 3. Local toolchain divergence (pin rust-toolchain.toml + parity byte-diff). 4. Hook --no-verify bypass (advisory; CI remains merge authority per ADR-0515 — by design).
