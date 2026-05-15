---
purpose: Auto-backfilled purpose for IP-001-cosign-rekor.md
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P08-IP-001
title: Cosign + Rekor signing pipeline
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship Cosign signing + Rekor anchoring for every artifact.
---

# M-CC-P08-IP-001 — Cosign + Rekor signing pipeline

## Purpose
Ship Cosign signing + Rekor anchoring for every artifact.

## Symbols-to-grit-claim
```
.github/workflows/cosign.yml::Workflow
crates/oya-foundry-fitness-supply-chain-kernel/src/lib.rs::check_signed
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
icm store -t context-oyatie -c 'M-CC-P08-IP-001 Cosign + Rekor signing pipeline shipped; acceptance commands green' -i high -k 'M-CC-P08-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- `Option<CosignSignature>` separates "no signature" from "signature failed verification" — the report cannot conflate the two failure modes.
- Digest is parsed once via `is_sha256_hex` (length + char-class) — empty / wrong-length / uppercase / non-hex all surface as distinct violations, not as a single "bad digest" catchall.
- Duplicate-artifact submission is an explicit `Err` — runners that double-feed cannot mask a missing-signature finding behind a "already passed" sibling.
- The workflow YAML uses keyless OIDC signing — no long-lived key material in repo/CI secrets.
- Verification (Rekor lookup) is separate from signing in the workflow — a failed verification step is observable, not hidden by signing success.
