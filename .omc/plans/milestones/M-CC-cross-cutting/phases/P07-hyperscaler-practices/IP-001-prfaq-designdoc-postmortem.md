---
purpose: Land standard templates per AWS / Google / SRE.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P07-IP-001
title: PRFAQ + Design-Doc + Postmortem templates
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Land standard templates per AWS / Google / SRE.
---

# M-CC-P07-IP-001 — PRFAQ + Design-Doc + Postmortem templates

## Purpose
Land standard templates per AWS / Google / SRE.

## Symbols-to-grit-claim
```
docs/standards/prfaq-template.md::Template
docs/standards/design-doc-template.md::Template
docs/standards/postmortem-template.md::Template
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
icm store -t context-oyatie -c 'M-CC-P07-IP-001 PRFAQ + Design-Doc + Postmortem templates shipped; acceptance commands green' -i high -k 'M-CC-P07-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP:
- All three templates declare `(required)` sections explicitly — reviewers can't waive their way past empty rationale, alternatives, or action items.
- Postmortem template names blameless wording out front and gives reviewers permission to call out blame-language during review.
- Design-doc template forces ≥ 2 considered alternatives — single-option docs cannot be marked approved.
- PRFAQ template requires a customer-readable press release **before** the design doc — prevents technically elegant work on features no customer asked for.
