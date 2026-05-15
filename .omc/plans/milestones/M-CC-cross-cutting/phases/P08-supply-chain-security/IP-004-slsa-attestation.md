---
purpose: Auto-backfilled purpose for IP-004-slsa-attestation.md
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P08-IP-004
title: SLSA level ≥3 attestation publishing
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Publish SLSA-3 attestation per release tag.
---

# M-CC-P08-IP-004 — SLSA level ≥3 attestation publishing

## Purpose
Publish SLSA-3 attestation per release tag.

## Symbols-to-grit-claim
```
.github/workflows/slsa.yml::Workflow
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
icm store -t context-oyatie -c 'M-CC-P08-IP-004 SLSA level ≥3 attestation publishing shipped; acceptance commands green' -i high -k 'M-CC-P08-IP-004,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- Reuses `slsa-framework/slsa-github-generator@v2.0.0` — provenance generation isn't reimplemented per-repo; we inherit L3 builder isolation guarantees from upstream.
- Hash + provenance split into separate jobs — a hash failure surfaces immediately and the provenance job is skipped (`if: needs.hash-artifacts.outputs.digests != ''`) rather than producing an empty attestation.
- One canonical output filename (`provenance.intoto.jsonl`) — verifiers don't need to discover or fuzzy-match attestation files.
