---
purpose: Block provider-specific imports outside adapter crates.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P05-IP-001
title: Provider-coupling lane kernel
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Block provider-specific imports outside adapter crates.
---

# M-CC-P05-IP-001 — Provider-coupling lane kernel

## Purpose
Block provider-specific imports outside adapter crates.

## Symbols-to-grit-claim
```
crates/oya-foundry-fitness-provider-coupling-kernel/src/lib.rs::check
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
icm store -t context-oyatie -c 'M-CC-P05-IP-001 Provider-coupling lane kernel shipped; acceptance commands green' -i high -k 'M-CC-P05-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- One `is_adapter_crate` predicate (single `contains("-adapter-")` check) replaces an enum of "which rings may name which providers" — adding a ring or adapter pattern is a one-line change.
- `BANNED_PROVIDER_TOKENS` is a single `const` array — adding a provider family is a single edit; no scattered match arms.
- Tokens are lower-cased once before comparison — case-evasion ("Anthropic_SDK") cannot bypass the check.
- Empty crate-name / empty import surface as `Err`, not silent passes — a malformed runner cannot generate a false-green report.
- The kernel is I/O-free; runners (cargo metadata, walkers) can change without touching the rule.
