---
doc_class: Template
template_id: TPL-IP
status: Accepted
date: 2026-05-12
purpose: |
enforcing_fitness_lane: governance-plan-hierarchy
owner_team: council-architecture
related:
  - .omc/plans/MASTERPLAN.md
  - docs/templates/phase-index-template.md
  - docs/templates/milestone-index-template.md
  - templates/checklists/per-implementation-plan-checklist.md
adrs_cited:
  - ADR-0054  # scaffold-claim pattern
  - ADR-0052  # inventory ledger (migration-class IPs)
doc_status: published
---

```yaml
# Required frontmatter on every IP file
---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-NNN-<slug>          # NNN is zero-padded per phase
parent: ../INDEX.md            # phase INDEX (relative path)
milestone: M0N                 # M01..M12
phase: P0N-<slug>
status: pending approval | open | in-progress | merged | blocked
execution_unit: ChangeSet      # every IP is a ChangeSet-sized execution unit
changeset_contract: claimable-verifiable-bundleable-promotable
purpose: |
  One paragraph stating what this IP delivers and why it sits at this position in the phase. Inherits Master Plan principles 1-12 by reference.
grit_claim_symbols:            # MUST be real file::Identifier; pre-scaffolded if new (ADR-0054)
  - "crates/oya-<context>-<role>/src/lib.rs::<TypeOrFn>"
agent_prerequisites:           # files/docs the agent MUST read before claiming
  - .omc/plans/MASTERPLAN.md
  - ./INDEX.md                 # phase index
  - docs/AGENTS.md
  - docs/CONSTITUTION.md
final_shape_compliance: true   # principle 3: no MVP placeholders
dependency_additions: []       # [{crate: "...", lts: true|false, adr_exception: "ADR-####|null"}]
decision_log: |
  Linus good-taste row: <special cases eliminated by this IP, or "none — no candidates">
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

# IP-NNN-<slug>: <one-line title>

## Purpose

One paragraph. What this IP delivers. Why now (its position in the phase dependency graph). Inherits which Master Plan principles (1-12) explicitly. **MUST** state the durable outcome in the present tense (e.g., "Ships `<capability-or-api>` with audit-chain emission on every invocation.") — not "will ship."


## ChangeSet boundary

State why this IP is one cohesive ChangeSet: exact issue-level scope, affected symbols/artifact pointers, affected crates/packages/deployables, required tests, evidence bundle, and promotion boundary. If the work cannot be claimed, verified, bundled, and promoted independently, split it before execution. Whole-tree locks or full-workspace cold builds require explicit graph-proven rationale.


Real `file::Identifier` list. New-crate IPs scaffold the symbols first (per ADR-0054); the scaffold commit is the first claimable artifact.

```
crates/oya-<context>-<role>/src/lib.rs::<Type>
crates/oya-<context>-<role>/src/<module>.rs::<fn>
contracts/<surface>.openapi.yaml::<operationId>
```

## Agent prerequisites

<!-- agent-instructions:start -->
2. Read `.omc/plans/MASTERPLAN.md §2 principles` (inherited verbatim by this IP).
3. Read parent phase `INDEX.md` (`./INDEX.md`).
4. Read `docs/AGENTS.md §Pre-flight checklist`.
5. Read `docs/CONSTITUTION.md §Decision principles + §Prohibitions`.
<!-- agent-instructions:end -->

**Human path:** read the same files; run the Buck2/cloud-ci plan-hierarchy gate target to confirm parent pointers resolve and frontmatter is well-formed.

## Acceptance test commands

Each row is a runnable command + expected pass token. CI replays these on every PR that touches this IP.

```
$ buck2 test <targeted-test-targets>                        # expect: PASS, 0 failures
$ buck2 build <targeted-build-targets>                        # expect: PASS
$ buck2 test <supply-chain-cloud-ci-target>                   # expect: PASS
$ buck2 test <lane-specific-cloud-ci-target>                  # expect: PASS
$ <sanctioned command>                                        # expect: <captured evidence shape>
```

## Done criteria

- [ ] `docs/AGENTS.md §Done-Definition checklist` D1-D18 walked for merge readiness.
- [ ] All acceptance commands PASS; outputs captured in PR `## Verification`.
- [ ] Dependency additions cleared the Cargo/cloud-ci supply-chain lane and are named in PR `## Summary`.
- [ ] Audit-chain `EVT-<topic>` emitted; ID pasted in PR `## Verification` when required.
- [ ] Phase INDEX `§Implementation Plans` row updated to `merged`.
- [ ] Inventory ledger row added if migration-class (per ADR-0052).

## Rollback procedure

1. Identify rollback boundary: `<git revision range | feature flag | capability T-tier downshift>`.
3. Verify: `<command + expected output>` (SLOs return to within budget; audit chain emits `EVT-IP-ROLLED-BACK`).
4. Postmortem trigger threshold: Sev-2 if rollback executed in production; Sev-3 if in staging.

## Next IP pointer

`IP-NNN+1-<slug>.md` (or `phases/P0N+1-<slug>/IP-001-<slug>.md` if phase boundary). Cite the exact file path.



```
  -t context-<project> \
  -i high \
  -k "M0N,P0N,IP-NNN,<axis>"
```

## Decision log (Linus good-taste row)

One row stating: what special case was eliminated? what data reshape removed the branch? If "none — no candidates," state that explicitly. Empty = `governance-plan-hierarchy` fail.

## Cross-references

- Master Plan: `.omc/plans/MASTERPLAN.md` §<section>.
- Phase INDEX: `./INDEX.md`.
- Related ADR(s): `ADR-####`. ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim), ADR-0052 (inventory if migration-class).
- Hyperscaler practice inherited (if any): per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md §<domain>`.
