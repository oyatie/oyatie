---
id: ADR-0539
title: "Cloud CI freshness gate for Cargo.lock member parity and generated-face byte parity"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-10
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0132, ADR-0363, ADR-0515, ADR-0538]
amends: []
related: [ADR-0083, ADR-0131, ADR-0132, ADR-0346, ADR-0363, ADR-0513, ADR-0515, ADR-0538]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0539: Cloud CI freshness gate

## Status

**Proposed - 2026-06-10 (authored for founder sign-off; door: one-way).**

## Context

FRIC-1781082000-G011 captured the PR #662 repair loop where stale `Cargo.lock` state and stale
generated cloud-ci faces required two serial CI round trips. FRIC-1781062100 fixes 1-2 and
FRIC-021 both point at the same underlying class: freshness defects were not diagnosed before the
full gate matrix consumed stale inputs.

ADR-0538 already established `libs/oya-workspace-members-kernel` as the canonical root-workspace
member resolver. ADR-0515 keeps merge authority in cloud-ci/oya-ci required contexts, while the
retirement-marked dev-cli remains only a local bridge feedback surface. The freshness check
therefore belongs in both layers: local bridge diagnosis for fast pre-push feedback, and a
dedicated cloud-ci job as the canonical admission signal.

## Decision

Add `ci/facade/generated-artifact-freshness` as a single-concern Rust gate.

NAME: oya-cloud-ci-freshness-app
JUSTIFICATION:
- microservice = cloud-ci: the cloud-ci admission product owns gate execution per ADR-0515.
- bc-tokens = freshness: the bounded concern is candidate-tree freshness, not general registry
  accounting.
- layer = app: the crate exposes a composition-root binary plus a pure library used by the
  dev-cli bridge.
- exemptions claimed: none.

The gate enforces two freshness classes:

- **Lock freshness:** resolve workspace members with
  `oya-workspace-members-kernel::resolve_member_dirs`, read each member `[package]` name and
  version, parse root `Cargo.lock`, and compare against sourceless `[[package]]` entries. The
  violation codes are `lock_missing_member_package`, `lock_stale_member_version`, and
  `lock_orphan_path_package`.
- **Generated-face freshness:** rebuild the same Buck2 targets used by
  `buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`, regenerate the SCM facts face plus the
  accounting-registry producer faces, and byte-diff them against the committed
  `*.generated.json` files. The violation code is `generated_face_stale`.

Failure output MUST include the exact remediation commands:

```text
cargo metadata >/dev/null
buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
```

Register `cloud-ci-freshness` as `frozen-empty-meta` in the oya-ci config and disposition table.
The accounting-registry baseline stamps its four codes permanently empty, while the dedicated
freshness app performs the live check in Buck and GitHub Actions.

The generated-output diff policy keeps blocking undeclared generated outputs and unsafe
normal-source merge policy, but it allows edits to declared materialized artifacts when their
manifest row is `never-manual-merge-regenerate-from-source-tree` and the artifact is
`merge-candidate-regenerated` or `main-branch-materialized`. ADR-0539 supplies the missing byte
freshness check for those cloud-ci faces.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `ci/facade/generated-artifact-freshness/` | create | `oya-cloud-ci-freshness-app` | app |
| `ci/facade/generated-artifact-freshness/src/bin/oya-cloud-ci-materialize-generated-faces.rs` | add Rust/Buck2 generated-face materializer bridge | `oya-cloud-ci-materialize-generated-faces` | app |
| `marketplace/facade/dev-cli/src/freshness_gate.rs` | create bridge module | `marketplace-dev-cli` | cli |
| `.github/workflows/oya-ci-required.yml` | add independent freshness job and fan-in need | - | - |
| `oya-ci.toml` | register `cloud-ci-freshness` | - | - |
| `libs/oya-ci-config/src/bundled/gate-disposition.json` | add frozen-empty freshness dispositions | - | - |
| `docs/oya-ci/gate-catalog.md` | document gate, input kind, key shapes, and frozen-empty codes | - | - |
| `ci/facade/generated-artifact-policy/` | allow declared materialized generated artifact edits | `oya-cloud-ci-generated-artifact-control-plane-app` | app |
| `registry/generated-artifact-control-plane.json` | document declared materialized artifact freshness policy | - | - |

### Integration via Workflow + Ontology

Not applicable. This ADR changes repository admission checks only; it does not emit Workflow
events, consume Workflow events, or write Ontology objects.

### Positive

- Stale lock entries and stale generated faces are diagnosed by one fast check instead of serial
  full-matrix failures.
- The lock check reuses ADR-0538 workspace membership expansion and does not invoke Cargo or the
  network.
- The face check reuses the exact materialization boundary that CI already trusts.
- Local bridge feedback and canonical CI enforcement now report the same remediation commands.

### Negative

- The freshness job rebuilds the SCM facts emitter and accounting-registry producer, so it is
  heavier than a pure parser-only check.
- Uncommitted source changes can legitimately make regenerated faces differ from committed faces;
  this is correct pre-push behavior but may require contributors to run the remediation command.

### Operational

- Local bridge: `gate run-all --ci-required` includes freshness through `oya-dev-cli`, but remains
  retirement-marked bridge evidence only.
- Canonical CI: `.github/workflows/oya-ci-required.yml` runs `gate-freshness` independently, with
  no `needs` edges to the other jobs, and folds it into the `oya-ci-required` fan-in result.
- Generated faces remain generator-owned. Contributors must run
  `buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .`; they must not hand-edit
  `*.generated.json`.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` | Affected | App crate depends inward on `oya-workspace-members-kernel`; no kernel imports app code. |
| `cross-product-refusal` | Not affected | No product boundary is introduced. |
| `port-location` | Not affected | No new port traits. |
| `layer-correctness` | Affected | New crate declares BNF app layer. |
| `composition-root-only` | Affected | Binary composition root lives in the app crate and dev-cli remains bridge-only. |
| `sdk-kernel-only` | Not affected | Developer SDK bridge consumes the app crate; no SDK kernel boundary change. |

## Alternatives Considered

**Alternative 1 - Rely on existing cargo-based gate failures**
- Description: let stale lockfiles fail during downstream cargo-based jobs.
- Pros: no new gate.
- Cons: diagnosis remains late and noisy; generated-face staleness still appears as a separate
  second failure class.
- Reason rejected: FRIC-1781082000 requires a single first diagnosis.

**Alternative 2 - Extend generated-output-diff-policy only**
- Description: add lockfile checks to the existing generated-output diff gate.
- Pros: one fewer CI job.
- Cons: mixes source freshness, lock parity, and generated artifact policy into one gate.
- Reason rejected: ADR-0132 single-concern discipline keeps freshness separate from diff policy.

**Alternative 3 - Use Cargo as the lockfile oracle**
- Description: run `cargo metadata` inside the freshness gate.
- Pros: exact Cargo semantics.
- Cons: slower, network/registry sensitive unless carefully constrained, and conflicts with the
  lane requirement that the gate itself be pure Rust over checked-in files.
- Reason rejected: `cargo metadata >/dev/null` is the sanctioned remediation command, not the
  checker implementation.

## References

- FRIC-1781082000-G011: pre-push freshness gate for lock and producer faces.
- FRIC-1781062100 fixes 1-2: stale lock and stale generated face repair loop.
- FRIC-021: lockfile-consistency member-subset class.
- ADR-0538: globbed root workspace membership and coverage gate.
- ADR-0515: cloud-ci required status context as merge authority.
- ADR-0363: plain git plus protected PR pipeline, retired bespoke VCS ratchet.
