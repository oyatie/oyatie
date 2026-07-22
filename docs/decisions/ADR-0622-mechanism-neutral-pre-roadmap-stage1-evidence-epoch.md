---
id: ADR-0622
title: "Mechanism-neutral pre-roadmap Stage-1 evidence epoch and planning-exit contract"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-07-22
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0069, ADR-0388, ADR-0515, ADR-0517, ADR-0522, ADR-0619]
amends: []
related: [ADR-0363, ADR-0516, ADR-0565, ADR-0613, ADR-0614, ADR-0616, ADR-0617]
related_specs:
  - /specs/masterplan.json
  - /specs/stage1-closure-program.schema.json
  - /specs/stage1-evidence-epoch.schema.json
milestone: W0
---

# ADR-0622: Mechanism-neutral pre-roadmap Stage-1 evidence epoch and planning-exit contract

## Status

**Proposed — 2026-07-22.** This decision installs a dormant, pure contract and regression corpus.
It does not amend Accepted authority, close the current planning hold, accept any unresolved ADR,
issue a qualified-human decision, or authorize roadmap or implementation dispatch. Lifecycle
promotion and control-plane activation require later protected changes carrying the authority and
admission evidence defined here.

## Context

The current `masterplan_v2.planning_entry_contract` correctly keeps binding planning and execution
dispatch closed, but its open-state evaluator is not a complete exit contract. Its evidence pointer
is a dated snapshot, its authority-choice matrix mixes current facts with an earlier PR candidate,
and a candidate can edit the same evaluator that interprets its proposed proof. Current authority
also lacks one exact machine-readable population for the required Stage-1 controls, an explicit
sixteen-lens council contract, fresh dissent, qualified-human receipts, and a cold-reader exit.

The repository contains additional evidence risks:

- source-authored evidence can describe Git facts it did not independently observe;
- generated projection determinism can be mistaken for producer independence;
- a label such as `archive` or `historical` leaves stale bytes readable to ordinary agents;
- a green pull-request check can be mistaken for post-merge product completion;
- same-principal reviews can be mistaken for an independent council;
- a machine can validate a qualified receipt but cannot create legal, affected-party, operations,
  custody, veto, or pilot authority.

A safe closure program must therefore be executable, content-addressed, independently reviewable,
and deliberately unable to authorize more than its narrow exit.

## Decision

### 1. One exact Stage-1 program

The canonical program identity is `correct-way-forward-before-roadmap`. Its instance will live at
`/specs/masterplan.json#masterplan_v2.planning_entry_contract.stage1_closure_program`; the schemas
in this change define its structural wire contract and `libs/oya-stage1-closure` defines the
dormant pure semantic candidate evaluator. It is deliberately not registered as a cloud-ci gate.

The exact control population is:

| ID | Control | Evidence class |
| --- | --- | --- |
| C01 | controlling ADR chronology | machine-verifiable |
| C02 | canonical parser and IR | machine-verifiable |
| C03 | corpus, archive, and freshness | machine-verifiable |
| C04 | decision population | machine-verifiable |
| C05 | comparator | machine-verifiable |
| C06 | legal and JCR | qualified human |
| C07 | affected party | qualified affected party |
| C08 | operations | qualified operations |
| C09 | custody | qualified custody |
| C10 | veto | authorized veto |
| C11 | pilot | machine and qualified human |
| C12 | immutable successor | machine-verifiable |
| C13 | sixteen-lens council | independent council |
| C14 | fresh dissent | independent dissent |
| C15 | context-free exit | independent oracle |

No alias, historical roster, doctrine-principle subset, or larger legacy panel may silently replace
this exact population.

### 2. A-G is a closure program, not a roadmap

The program groups work without dispatching product implementation:

- **A — control-plane repair:** owns C01-C03.
- **B — certificate definitions:** owns C04-C05 after A. It may freeze comparator protocols and
  inactive source pointers, but C05 cannot become satisfied before C06 supplies a fresh,
  scope-specific qualified legal/JCR disposition.
- **C — pilot and abstention contract:** owns C10-C11 after B and D have satisfied C04-C09.
- **D — parallel qualified evidence:** owns C06-C09 after A and runs concurrently with B, so legal
  authority never depends on comparator collection it must first authorize.
- **E — qualified authority closure:** owns no second copy of a control; it joins C04-C11 after
  B, C, and D.
- **F — freeze and challenge:** owns C12-C14 after C01-C11.
- **G — context-free exit and admission:** owns C15 after C01-C14.

The owned-control sets partition C01-C15 exactly once. Dependencies may join earlier controls but
never duplicate or relabel them.

### 3. Exact sixteen-lens roster

The council evaluates the same frozen subject through exactly these lenses:

1. product and user value;
2. intuitive no-code UX and accessibility;
3. ontology, data, and temporal semantics;
4. workflow automation and compensation;
5. architecture, modularity, and hyperscale;
6. cloud, platform, and enterprise infrastructure;
7. developer, build, release, and supply chain;
8. reliability, operations, and observability;
9. security, identity, and abuse resistance;
10. privacy, residency, and data governance;
11. legal, regulatory, and JCR;
12. affected-party safety and ethics;
13. economics, FinOps, and business viability;
14. interoperability, supply chain, and ecosystem;
15. maintainability, evolvability, and deprecation;
16. evidence, audit, governance, and dissent.

Each lens has a distinct reviewer identity, an object-bound receipt, author independence, fresh
context, and the same subject digest. This roster is a proposed contract until lifecycle promotion;
it must not be backfilled from the retired 21-facet multispectrum convention or inferred from
P0-P16 doctrine principles.

### 4. Immutable epoch state machine

An evidence epoch has one of six states:

1. `HOLD_EPOCH_OPEN`;
2. `HOLD_EVIDENCE_COMPLETE`;
3. `HOLD_SUCCESSOR_FROZEN`;
4. `HOLD_EXIT_CANDIDATE`;
5. `PASS_CANDIDATE`;
6. `BLOCKED_QUALIFIED_HUMAN_INPUT`.

The only success path is the ordered progression from state 1 through state 5. Each hold state can
terminate at state 6 when the remaining input is irreducibly qualified-human. Any program,
authority, evidence, parser, producer, policy, or subject mutation opens a new epoch; it does not
rewrite or retroactively promote an earlier epoch.

`PASS_CANDIDATE` is an explicitly non-authoritative structural candidate, not effective
`PASS(Planning)`, and keeps all three source planning flags false. In the current dormant build,
even that structural candidate is rejected because no authenticated external producer, signature
verifier, trust-root verifier, or controller exists.
Only a future, independently operated post-merge admission envelope can derive effective
`PASS(Planning)` from an exact promoted commit; it is not implemented or activated by this ADR.
Even after that external derivation:

- binding plan approval remains false;
- implementation dispatch remains false;
- no work wave is dispatched;
- a later roadmap must have its own review, admission, approval, and dispatch authority.

### 5. Three artifacts, three authority boundaries

The pipeline separates:

1. **Source epoch.** A reviewed machine-readable record of control statuses and receipt references.
   It never asserts its own Git commit, tree, blob, or admission facts.
2. **Protected facts.** The closed `oyatie/stage1-protected-facts/v1` contract (not
   `oya-ci/scm-facts/v2`) binds protected-base and candidate repository, commit, tree, source-path
   and blob roles; program/parser/future-producer/evaluator/policy/schema digests; predecessor epoch and
   transition receipt; immutable successor bundle; authority-chain result; trust root; and every
   receipt binding. No producer is implemented or activated here. Generated faces remain materialized
   only by the future canonical producer and are never hand-edited.
3. **Post-merge admission.** A separate closed `oyatie/stage1-admission-envelope/v1` external
   immutable envelope binds the promoted commit (which must differ from the PR head), required
   context and source App, review/protection result, run identity, rollout, rollback, observability,
   browser/user-story, release, and observation-harvest outcomes. It cannot mutate the source
   epoch or turn missing qualified authority into PASS.

### 6. Protected-parent interpretation

A candidate tree is data under test. It cannot define the parser, producer, evaluator, policy, or
control population that judges its own exit. Activation is staged:

1. merge the dormant parser, grammar, schemas, and evaluator under the prior protected policy;
2. merge the exact program and first HOLD epoch;
3. only a later candidate may present evidence, evaluated by protected-parent code and independently
   supplied protected facts under a declared trust-root authority;
4. the final source-only transition may change the epoch pointer to `PASS_CANDIDATE`, but not the evaluator,
   schemas, control roster, lens roster, or qualification rules.

Every source-state advancement beyond `HOLD_EPOCH_OPEN` requires protected facts. The protected
facts exact-bind every receipt path, blob OID, SHA-256, and frozen subject together
with the program, schema, policy, parser, producer, evaluator, protected base, and candidate.
If a protected-parent tool is missing, fails to build, produces empty or mismatched output, or
cannot bind those exact artifacts and trees, the result is HOLD or BLOCKED, never fallback PASS.

### 7. Qualified authority is validated, never invented

For C06-C11, the evaluator requires object-bound receipts independently matched by the protected
facts, naming principal identity, qualification, authority source, scope, frozen subject,
independence where applicable, and verification outcome. C11 requires both independently bound
machine evidence and qualified-human evidence; neither half can stand in for the other.
Repository authors, bots, agents, code owners, and green CI do not become qualified principals by
assertion. Missing authority terminates truthfully at `BLOCKED_QUALIFIED_HUMAN_INPUT` only for
C06-C11, names the control that is itself `blocked`, and carries the exact input class,
qualification, scope, and authority source required.

### 8. Comparator and evidence standards

C05 is a minimum-baseline comparison, not product copying or competitive parity. It binds current
qualified primary sources, retrieval time, version, supported fact, uncertainty, negative evidence,
and the Oyatie-specific exit gate. Source inventory and protocol design may run before C06, but
collection, expansion, analysis, citation, and C05 satisfaction may not. Palantir Foundry/AIP/Apollo
may be one minimum comparator; GitHub merge
queue and attestations, SLSA, NIST SSDF and AI RMF, Kubernetes admission, OPA policy tests, SRE
error budgets, and progressive-delivery systems are independent control-loop comparators. No
external product taxonomy becomes Oyatie's architecture or authority.

Game-engine architectures are a second mechanism comparator for the ontology/workflow substrate:
a revisioned world, stable entities, composable components or traits, explicit systems, declared
schedules, durable events, distinct clocks, deterministic preview/replay, rebuildable scenes/views,
and a visual authoring-to-runtime compilation boundary. Unity Entities, Unreal Gameplay
Framework/Mass/StateTree, and Bevy ECS/schedules/time URLs remain inactive source pointers until C06
authorizes fresh scope-specific collection; only then may qualified, digested source versions become
primary-source baselines. They are never dependencies or feature templates. C05 must also record
where the metaphor stops:
tenant-defined types remain open and versioned rather than vendor enums, while legal authority,
privacy, accounting, distributed consistency, irreversible external effects, and human
accountability remain explicit domain controls above simulated world state.

### 9. Retired context is absent from the current tree

When a source is superseded or retired, its exact bytes remain only in authorized Git object
history. A directory named `archive`, a tombstone containing copied prose, or another readable
in-tree relocation does not satisfy C03. Successors retain neutral path, object OID, SHA-256, byte
count, disposition, and successor references without copying retired content.

## Consequences

- The current planning hold becomes an executable state machine rather than a prose convention.
- Evidence can accumulate in parallel while authority joins remain fail-closed.
- A later PASS is reproducible without conversation history and cannot smuggle implementation
  authorization through a planning exit.
- Qualified humans remain necessary only where their authority is substantively irreducible.
- The first implementation adds a small pure crate and schemas; it does not add a service, CLI,
  generated merge surface, or new required status context.
- Until this ADR is Accepted and its staged activation is admitted, current authority remains the
  existing open hold.

## Alternatives considered

### Extend the current open-state check in the same exit PR

Rejected. A candidate would define the evidence semantics that approve itself.

### Use one mutable evidence ledger

Rejected. It conflates source claims, SCM observations, admission, and qualified authority, and
permits retroactive promotion.

### Treat bot review or the existing doctrine principles as the sixteen-lens council

Rejected. Neither proves the exact roster, distinct principals, qualifications, fresh context, or
same-subject review.

### Keep old evidence in a readable archive directory

Rejected. Ordinary agents can still ingest it and mistake stale context for current authority.

### Let PASS dispatch implementation

Rejected. Stage-1 exits only into roadmap planning; approval and dispatch are separate later gates.

## Verification

1. RED tests precede the pure evaluator implementation.
2. Program validation requires exact ordered states, transition graph, A-G groups, C01-C15
   controls, and L01-L16 lenses.
3. HOLD fixtures are green only with roadmap, binding approval, and dispatch false.
4. PASS fixtures are green only when all controls and lenses share one frozen subject, qualified
   receipts exist, the successor is immutable, fresh dissent is independently preserved, and a
   distinct blind reader reproduces the exit without conversation context.
5. PASS still leaves binding approval and implementation dispatch false.
6. Missing or duplicate controls, lenses, reviewers, receipts, subject digests, or SCM facts fail.
7. Cargo, Buck, formatting, clippy, generated-artifact policy, and cross-artifact integration are
   green before activation.
8. No `*.generated.json` is added or modified by hand.
9. Protected PR review, zero unresolved threads, branch protection, and the single required
   `oya-ci-required` context remain mandatory; post-merge admission is recorded separately.
