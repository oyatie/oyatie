---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P14-IP-002
title: 1ES-templated CI pipelines
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Adopt Microsoft 1ES-style templated CI pipelines.
---

# M01-P14-IP-002 — 1ES-templated CI pipelines

## Purpose
Adopt Microsoft 1ES-style templated CI pipelines.

## Symbols-to-grit-claim
```
.github/workflows/_template-ci-lane.yml::Template
.github/workflows/_template-release.yml::Template
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged (except for IPs IN M01-P08 itself).

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-foundry-fitness-cohesion -- <owning-crate-glob>
scripts/check.sh
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M01-P14-IP-002 1ES-templated CI pipelines shipped; acceptance commands green' -i high -k 'M01-P14-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- Toolchain + cache config + lane-runner setup live in `_template-ci-lane.yml` exactly once — adding a new lane is ~10 lines of caller YAML, not 50+ lines of duplicated boilerplate.
- Release template defers signing/SBOM/SLSA to the M01-P15 workflows by convention — the release pipeline cannot accidentally ship without those gates running.
- Inputs are explicit (lane-name, package-glob, acceptance-commands) so a caller can't silently inherit the wrong toolchain or cache key from the template.
- `RUSTFLAGS: -D warnings` is baked into the lane template — a lane cannot quietly accept warnings by forgetting to set it.
