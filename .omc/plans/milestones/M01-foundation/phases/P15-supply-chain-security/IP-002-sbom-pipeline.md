---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P15-IP-002
title: SBOM generation per build (CycloneDX / SPDX)
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship SBOM generation per build at releases/<tag>/sbom.json.
---

# M01-P15-IP-002 — SBOM generation per build (CycloneDX / SPDX)

## Purpose
Ship SBOM generation per build at releases/<tag>/sbom.json.

## Symbols-to-grit-claim
```
.github/workflows/sbom.yml::Workflow
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
icm store -t context-oyatie -c 'M01-P15-IP-002 SBOM generation per build (CycloneDX / SPDX) shipped; acceptance commands green' -i high -k 'M01-P15-IP-002,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- Both CycloneDX and SPDX emitted from the same job — no "which one did this build use" ambiguity; downstream tooling can pick either format.
- Empty-SBOM detection via `jq -e '.components // .packages // empty'` — a broken generator producing `{}` cannot pass as a valid SBOM.
- Outputs land at deterministic `releases/<tag>/sbom.{cdx,spdx}.json` paths — the supply-chain kernel doesn't need to discover artifacts at verify time.
