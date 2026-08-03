# Spec: G011 item 2 — pre-push freshness gate (lock + producer faces)

Status: ACTIVE (lane spec + team brief) · Story: G011 · Friction retired: FRIC-1781082000-G011 (also closes FRIC-1781062100 fixes 1–2; FRIC-021 lockfile-consistency member-subset)
Verified facts as of dev @ 5aaa68ab4, 2026-06-10.

## Objective

PR #662 needed TWO serial CI repair pushes (each a full ~15-job round-trip): (1) stale Cargo.lock after rebase — every cargo-based gate leg failed with exit 101 "cannot update the lock file... --locked"; (2) stale `scm-facts.generated.json` after the lock refresh — `generated-output-diff-policy` failed. Nothing checks either locally before push, and the two repair invocations are undocumented tribal knowledge.

After this lands: one fast local check (inside the canonical `oya verify --ci-required` → `gate run-all --ci-required` path) and one fast CI job catch BOTH staleness classes in a single pass, with exact remediation commands in the failure output.

## Verified ground truth

- Canonical local pre-push contract already exists: `libs/oya-check-pre-push` asserts `oya verify --ci-required` dispatches natively into Rust `gate run-all --ci-required` (oya-dev-cli). The dev-cli is retirement-marked but sanctioned as local bridge feedback — merge authority stays in CI. Register the new check there, mirroring an existing dev-cli gate module (e.g. `workspace_topology_gate.rs`); `run-all` registration lives in `oya/developer-sdk/crates/oya-dev-cli/src/lib.rs`.
- Canonical member resolver: `libs/oya-workspace-members-kernel::resolve_member_dirs` (ADR-0538). REUSE it — never re-parse the members array.
- Face regen mechanism: `infra/ci/materialize-cloud-ci-generated-faces.sh` (CI boundary step) builds + runs `oya-cloud-ci-scm-facts-emitter-app` and `oya-cloud-ci-accounting-registry-app-bin` via buck2 and rewrites the 4 faces under `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/`. Faces are a pure function of the tracked tree (registry-drift gate asserts committed == regenerated in CI).
- CI gate matrix: `.github/workflows/oya-ci-required.yml` jobs run with NO needs-edges (PR #660 dropped them for latency). The freshness job must NOT re-serialize the matrix: it is its own fast job; the gate legs may still fail redundantly on a stale lock, but the freshness job is the canonical first diagnosis.
- Sanctioned repair commands (must appear verbatim in failure remediation output):
  - lock: `cargo metadata >/dev/null`
  - faces: `infra/ci/materialize-cloud-ci-generated-faces.sh .`
- ADR slot: 0539. Gate registration surfaces (mirror what PR #662 did for workspace-glob-coverage): `oya-ci.toml`, `libs/oya-ci-config/src/bundled/gate-disposition.json`, `docs/oya-ci/gate-catalog.md`, matrix line in `oya-ci-required.yml`, BUCK targets. Firewall: gate-baseline regen via producer ONLY (never hand-edit *.generated.json; FRIC-009).

## Design (single-concern per ADR-0132)

New crate `cloud/cloud-ci/gates/oya-cloud-ci-freshness-app`:

1. **Lock-freshness (pure, no cargo, no network):** resolve member dirs via `oya-workspace-members-kernel`; read each member `Cargo.toml` `[package]` name+version; parse root `Cargo.lock` `[[package]]` entries (path crates = no `source`). Violations: `lock_missing_member_package` (member not in lock), `lock_stale_member_version` (version mismatch), `lock_orphan_path_package` (sourceless lock entry with no member). This is the exact #647/#662 failure class, detected in milliseconds. (Reuse the lockfile parsing approach from `tools/oya-cargo-lock-merge-driver-app` where sensible — shared kernel extraction is allowed if clean, NOT required.)
2. **Face-freshness:** rematerialize via the same buck2 targets the CI script uses (build + run emitter/producer against the working tree) and byte-diff against the committed 4 faces. Violation: `generated_face_stale` (one finding per stale face). The check must run from a clean tree perspective (diff committed vs regenerated; uncommitted source changes legitimately produce differences — report them as stale, that is correct pre-push semantics).
3. Failure output: one finding per violation code + a remediation block printing the exact sanctioned commands above.
4. Wiring:
   a. dev-cli: new `freshness_gate.rs` registered in `gate run-all --ci-required` (local bridge; mirrors existing gate modules' shape and error reporting).
   b. CI: one fast job `freshness (lock + generated faces, ADR-0539)` in `oya-ci-required.yml` running the same checker via buck2; no needs-edges; folded into the `oya-ci-required` rollup's needs list exactly like other jobs.
5. ADR-0539 (cites FRIC-1781082000 + FRIC-1781062100 + ADR-0538 kernel reuse + enforcement-layering: local check = bridge, CI job = canonical).

## Commands (canonical verification preamble)

buck2 build/test on every affected target; BUCK + (if needed) reindeer wiring = part of done; cargo supplementary only; lock refresh ONLY via `cargo metadata >/dev/null`. Pre-existing local buck2 gate-test REDs (firewall/slo-coverage/registry-drift/generated-artifact-control-plane) are FRIC-009, not yours — but NOTE: after PR #662 those faces were regenerated; re-verify which local REDs remain before attributing anything.

## Testing (full ladder where applicable)

- Unit: lock parser + comparison (GREEN fixture; RED per violation code: missing member, version mismatch, orphan path package).
- Integration: face-freshness against fixture trees (committed==regenerated GREEN; mutated face RED) — if running the real producer in tests is impractical, test the diff harness with injected regenerator output and cover the real path via the CI job itself + a buck2 test that shells the checker binary with a stub regenerator.
- RED fixtures must assert blocking findings (mirror workspace-glob-coverage-app test shape).

## Boundaries

- Always: isolated worktree, PR to dev, rebase-fresh base, buck2-first, SSH-signed commits, remediation text verbatim.
- Never: hand-edit *.generated.json or Cargo.lock; weaken existing gates; new shell scripts (Rust only — the existing materialize .sh stays as-is, out of scope to port); touch the main checkout; run omc orphan-cleanup.

## Success criteria

1. Reproduce #662's two failure shapes locally: a workspace with a missing lock entry and a mutated face both produce RED findings with correct remediation text (fixtures prove it).
2. `gate run-all --ci-required` includes the gate; `oya-check-pre-push` contract still green.
3. CI job green on the PR; appears in branch-protection rollup needs; no needs-edge serialization added.
4. ADR-0539 + all registration surfaces; equivalence of gate registry (firewall) maintained via producer regen.
5. oya-ci-required green on rebased head; adversarial review APPROVE in code.
