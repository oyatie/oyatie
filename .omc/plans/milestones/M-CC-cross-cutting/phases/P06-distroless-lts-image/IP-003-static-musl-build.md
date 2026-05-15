---
purpose: Auto-backfilled purpose for IP-003-static-musl-build.md
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P06-IP-003
title: Static / musl-linked binary build pipeline
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship static / musl-linked release build pipeline.
---

# M-CC-P06-IP-003 — Static / musl-linked binary build pipeline

## Purpose
Ship static / musl-linked release build pipeline.

## Symbols-to-grit-claim
```
.github/workflows/release-musl.yml::Workflow
Dockerfile.distroless::Distroless
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M-CC-P01 ≥ P5 merged (except for IPs IN M-CC-P01 itself).

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
icm store -t context-oyatie -c 'M-CC-P06-IP-003 Static / musl-linked binary build pipeline shipped; acceptance commands green' -i high -k 'M-CC-P06-IP-003,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- The build step asserts the binary is **not** dynamically linked (`grep -q "dynamically linked"` fail-fast) — a silently-glibc-linked artifact cannot pass as static.
- Dockerfile.distroless uses `gcr.io/distroless/static-debian12:nonroot` — the image-discipline lane will already pass the resulting image (allowlist hit + no shell + no package manager).
- Single multi-arch matrix (`x86_64-unknown-linux-musl` + `aarch64-unknown-linux-musl`) — adding a new arch is one row, not a duplicated workflow.
- `RUSTFLAGS="-C link-arg=-s"` strips debug symbols at link time — release tarballs stay small without a separate `strip` step.
- The musl build job runs first; the distroless image job depends on it — failed binary build short-circuits image build, never producing a bad image to sign.
