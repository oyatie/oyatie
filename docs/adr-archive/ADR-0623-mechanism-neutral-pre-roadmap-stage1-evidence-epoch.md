---
id: ADR-0623
title: "Keep the pre-roadmap Stage-1 evidence epoch mechanism-neutral"
status: Superseded
date: 2026-07-24
owner_team: council-architecture
co_owners: [council-architecture]
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0363, ADR-0516, ADR-0565, ADR-0613, ADR-0614, ADR-0616, ADR-0617]
tags: [architecture, governance, planning, evidence, stage-1]
purpose: |
  Preserve a Proposed, mechanism-neutral description of a fail-closed Stage-1 evidence
  epoch and planning-exit contract. This docs-only delta neither creates its historical or
  proposed implementation paths nor changes the existing HOLD(Planning) boundary.
authority_chain_declaration: |
  system/developer/user instructions > CLAUDE.md + docs/AGENTS.md >
  specs/root-hub-pointers.json > current machine-readable specs and registries > docs/
  compatibility authority > working drafts. This Proposed ADR does not amend Accepted
  authority or create a planning, roadmap, implementation, admission, or completion claim.
bominal_inheritance: |
  no Bominal equivalent. Oyatie overrides no Bominal ADR: this is an Oyatie-local,
  Proposed documentation contract. Per docs/AGENTS.md, Oyatie governance overlays and wins
  on conflict under Bominal-inheritance precedence.
planning_impact: false
deciders: []
door: one-way
owner: council-architecture
amended_by: []
depends_on: [ADR-0069, ADR-0388, ADR-0515, ADR-0517, ADR-0522, ADR-0619]
amends: []
related_specs: []
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Stage-1 evidence epoch mechanism-neutral — planning contract hygiene

# ADR-0623: Keep the pre-roadmap Stage-1 evidence epoch mechanism-neutral

## Status

**Proposed — 2026-07-24.** This decision records a mechanism-neutral contract proposal and a
history-only prototype. It does not install an executable contract, amend Accepted authority, close
the current planning hold, accept any unresolved ADR, issue a qualified-human decision, or
authorize roadmap or implementation dispatch. Adoption requires an authenticated qualified
acceptance of the control population and its admission role, followed by a fresh protected
implementation.

## Frontmatter

| Field | Value |
|---|---|
| **id** | ADR-0623 |
| **title** | Keep the pre-roadmap Stage-1 evidence epoch mechanism-neutral |
| **status** | Proposed |
| **date** | 2026-07-24 |
| **owner** | `council-architecture` |
| **co-owners** | `council-architecture` |
| **supersedes / superseded-by** | `-` / `-` |
| **related** | ADR-0363, ADR-0516, ADR-0565, ADR-0613, ADR-0614, ADR-0616, ADR-0617 |
| **Bominal inheritance** | no Bominal equivalent; Oyatie-local Proposed contract; no override of a Bominal ADR |
| **planning impact** | `false`; HOLD(Planning) remains in force |

## Governed surfaces

The historical prototype introduced the prototype-owned paths below and modified two existing
current-authority files. The prototype-owned paths are intentionally absent from the final tree
because Buck2's protected full-workspace build and test universes could otherwise make a Proposed
mechanism admission-bearing. Its mutations to `specs/BUCK` and `specs/masterplan.json` are also
absent; those two existing files remain current authority and are byte-unchanged by this proposal.
The absent paths and reverted mutations describe proposed scope only; they are not current
authority or current implementation.

- `libs/oya-stage1-closure/OWNERS`
- `libs/oya-stage1-closure/BUCK`
- `libs/oya-stage1-closure/Cargo.toml`
- `libs/oya-stage1-closure/src/lib.rs`
- `libs/oya-stage1-closure/tests/fixtures/admission-envelope.json`
- `libs/oya-stage1-closure/tests/fixtures/hold-epoch.json`
- `libs/oya-stage1-closure/tests/fixtures/program.json`
- `libs/oya-stage1-closure/tests/stage1_closure.rs`
- `specs/stage1-admission-envelope.schema.json`
- `specs/stage1-closure-program.schema.json`
- `specs/stage1-evidence-epoch.schema.json`
- `specs/stage1-protected-facts.schema.json`

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

The proposed canonical program identity is `correct-way-forward-before-roadmap`. If separately
accepted, its instance could live at
`/specs/masterplan.json#masterplan_v2.planning_entry_contract.stage1_closure_program`, with closed
schemas and a pure semantic evaluator. No such masterplan field, schema, library, registry row,
root-hub pointer, or cloud-CI gate exists in the final tree of this proposal.

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
authority, evidence, parser, producer, delegated materializer, policy, or subject mutation opens a
new epoch; it does not rewrite or retroactively promote an earlier epoch.

`PASS_CANDIDATE` is an explicitly non-authoritative structural candidate, not effective
`PASS(Planning)`, and keeps all three source planning flags false. The history-only prototype
rejected even that structural candidate because no authenticated external producer, signature
verifier, trust-root verifier, or controller existed.
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
   receipt binding. If materialization is delegated, the protected facts and immutable successor
   cross-bind an exact `materializer_binding` containing the materializer principal identity,
   implementation digest, and configuration digest/version. No producer or materializer is
   selected, implemented, or activated here.
   Generated faces remain materialized only by the future canonical producer or its exact-bound
   delegated materializer and are never hand-edited.
3. **Post-merge admission.** A separate closed `oyatie/stage1-admission-envelope/v1` external
   immutable envelope binds the promoted commit (which must differ from the PR head), required
   context and source App, review/protection result, run identity, rollout, rollback, observability,
   browser/user-story, release, and observation-harvest outcomes. It cannot mutate the source
   epoch or turn missing qualified authority into PASS.

### 6. Protected-parent interpretation

A candidate tree is data under test. It cannot define the parser, producer, delegated materializer,
evaluator, policy, or control population that judges its own exit. Any accepted activation would be
staged:

1. merge the dormant parser, grammar, schemas, and evaluator under the prior protected policy;
2. merge the exact program and first HOLD epoch;
3. only a later candidate may present evidence, evaluated by protected-parent code and independently
   supplied protected facts under a declared trust-root authority;
4. the final source-only transition may change the epoch pointer to `PASS_CANDIDATE`, but not the evaluator,
   schemas, control roster, lens roster, or qualification rules.

Every source-state advancement beyond `HOLD_EPOCH_OPEN` requires protected facts. The protected
facts exact-bind every receipt path, blob OID, SHA-256, and frozen subject together
with the program, schema, policy, parser, producer, any delegated materializer, evaluator,
protected base, and candidate.
If a protected-parent tool is missing, fails to build, produces empty or mismatched output, or
cannot bind those exact artifacts and trees, or has a missing or mismatched delegated
`materializer_binding`, the result is HOLD or BLOCKED, never fallback PASS.

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

## Decision drivers

- Preserve a truthful HOLD(Planning) boundary until a qualified, independently admitted successor exists.
- Keep proposed evidence semantics separate from the candidate tree and any proposed implementation.
- Make a later exact-`origin/dev` rebind possible without treating this docs-only delta as admission evidence.

## Why chosen

This proposal satisfies the Stage-1 need for an exact, reviewable evidence population without
turning a document into executable authority. It honors the current authority chain and
ADR-0363/ADR-0515 boundary: ordinary Git and the protected `oya-ci-required` context govern later
admission, while retired wrappers and local bridge commands remain non-authoritative. It is safer
than the alternatives below because it names the future contract and its evidence ceiling while
leaving the live planning hold, roadmap approval, and implementation dispatch untouched.

## Consequences

### Concrete file and crate changes

This Proposed docs-only delta changes exactly four canonical documentation surfaces: this ADR
source, its generated index and machine-readable projection, and the Tier-1 canonical-document
lifecycle row in `docs/CHANGELOG.md`. It deliberately makes no current implementation, schema,
registry, masterplan, root-hub, generated-face, crate, BUCK, or CI-gate change. The historical
prototype paths below are absent and non-authoritative; the proposed paths are future candidates
only and require separate qualified acceptance plus a fresh exact-`origin/dev` rebind before any
implementation work.

| Path / crate | Current change in this Proposed delta | Historical/proposed status | Layer |
|---|---|---|---|
| `docs/decisions/ADR-0623-mechanism-neutral-pre-roadmap-stage1-evidence-epoch.md` | update documentation source | current, Proposed, non-executable | — |
| `docs/ADR-INDEX.md` | producer-regenerated lifecycle index row | current canonical projection | — |
| `docs/machine-readable/decisions.json` | producer-regenerated machine-readable row | current canonical projection | — |
| `docs/CHANGELOG.md` | Tier-1 canonical-document lifecycle entry | current documentation lifecycle evidence | — |
| `libs/oya-stage1-closure/` | none; do not create or restore | historical prototype absent; a future proposal only | no layer assigned |
| `specs/stage1-*.schema.json` | none; do not create or restore | proposed non-authoritative paths absent | — |
| `specs/masterplan.json`, `specs/BUCK`, root-hub, registries, generated faces, cloud-CI gates | none | current authority remains unchanged | — |

### Integration via Workflow + Ontology

Not applicable in this docs-only Proposed delta: it emits and consumes no Workflow events and
performs no Ontology object or link reads or writes. Any future implementation must define its
typed Workflow and Ontology contract in the separately accepted implementation ADR and cannot
infer it from this proposal.

### Positive

- Preserves a precise control population and fail-closed evidence ceiling without a completion claim.
- Makes historical prototype paths explicitly absent so they cannot be mistaken for current authority.
- Gives the later exact-`origin/dev` rebind a stable, reviewable source document without dispatching work.

### Negative

- Does not itself close HOLD(Planning), admit a planning exit, or authorize a roadmap or implementation wave.
- Defers executable parser, schema, protected-facts producer, trust-root, and controller evidence to a separately admitted change.
- Requires qualified-human input for irreducibly human controls, which cannot be fabricated by repository text or CI.

### Operational

- No CI lane, BUCK target, cloud-CI gate, runbook, registry, generated runtime face, or other
  generated-artifact control-plane change occurs in this delta; the producer-regenerated
  documentation projections in the table above are the sole exception.
- The smallest applicable document check is ADR-shape validation; no local legacy `oya` result is admission or merge authority.
- Any later activation must use a protected PR, reviewer approval, the single required `oya-ci-required` context, and post-merge evidence; it must not resurrect the historical prototype.

## Clean Architecture Impact

No implementation layer changes occur. The six lanes remain not affected for this docs-only
Proposed delta; a future implementation must reassess each lane from the exact protected base.

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none; no crate is added or changed |
| `cross-product-refusal` (LEAN-A2) | Not affected | none; no cross-product boundary is introduced |
| `port-location` | Not affected | none; no port trait is introduced or moved |
| `layer-correctness` | Not affected | none; no layer assignment changes |
| `composition-root-only` | Not affected | none; no app-layer binary changes |
| `sdk-kernel-only` | Not affected | none; no SDK crate changes |

## Alternatives considered

**Alternative 1 — Extend the current open-state check in the same exit PR**

- **Description:** Let the candidate edit the evaluator and present the evidence that evaluator accepts.
- **Pros:** Fewer artifacts and a shorter apparent delivery path.
- **Cons:** The candidate can define the semantics that approve itself; protected-parent independence is lost.
- **Reason rejected:** It cannot provide a trustworthy, independently evaluated planning exit.

**Alternative 2 — Use one mutable evidence ledger**

- **Description:** Store source claims, SCM observations, admission results, and qualified authority in one editable record.
- **Pros:** Simple storage and a single lookup surface.
- **Cons:** Conflates incompatible authority boundaries and permits retroactive promotion.
- **Reason rejected:** An immutable source/protected-facts/admission split is required for a fail-closed exit.

**Alternative 3 — Treat bot review or doctrine principles as the sixteen-lens council**

- **Description:** Infer the council from existing automated review or a broader historical principle set.
- **Pros:** Reuses existing review signals and avoids named receipts.
- **Cons:** Does not prove the exact roster, distinct principals, qualifications, fresh context, or same-subject review.
- **Reason rejected:** Inference cannot substitute for independent, object-bound council evidence.

**Alternative 4 — Keep old evidence in a readable archive directory**

- **Description:** Preserve retired prototype material under an in-tree archive or tombstone.
- **Pros:** Convenient repository-local access to historical context.
- **Cons:** Ordinary readers can ingest stale bytes and mistake them for current authority.
- **Reason rejected:** C03 requires retired bytes to remain in authorized Git object history, not readable current-tree paths.

**Alternative 5 — Let PASS dispatch implementation**

- **Description:** Treat a planning exit as authorization to begin implementation work.
- **Pros:** Reduces the number of subsequent governance steps.
- **Cons:** Collapses planning, approval, and implementation authority into one result.
- **Reason rejected:** A future PASS can at most support roadmap planning; separate later approval and dispatch are mandatory.

## Verification

The history-only prototype was developed regression-first and exercised the exact ordered states,
transition graph, A-G groups, C01-C15 controls, L01-L16 lenses, HOLD invariants, qualified-receipt
joins, immutable successor, fresh dissent, and context-free exit. Those receipts demonstrate
design feasibility only.

After qualified acceptance, a fresh implementation must re-establish all RED and GREEN evidence,
formatting, static analysis, generated-artifact policy, cross-artifact integration, independent
review, zero unresolved threads, branch protection, and the single required `oya-ci-required`
context. No `*.generated.json` may be added or modified by hand. The historical prototype cannot
be resurrected or treated as current authority.

## References

- [`templates/adr-template.md`](../../templates/adr-template.md) — live canonical ADR template and required frontmatter shape.
- [`docs/AGENTS.md`](../AGENTS.md) — current operating contract, Bominal-inheritance precedence, and ADR authoring rule.
- [ADR-0363](ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate.md) — plain-Git protected-PR governance; retired agentic VCS ratchet.
- [ADR-0515](ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md) — single protected `oya-ci-required` context and current CI admission boundary.
- ADR-0069, ADR-0388, ADR-0517, ADR-0522, and ADR-0619 — dependencies and surrounding Stage-1 authority context.
- Bominal inheritance: no Bominal ADR equivalent is inherited, translated, or overridden by this Oyatie-local Proposed contract.
- Issue: Refs #1364.
