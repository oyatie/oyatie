---
doc_class: Template
template_id: TPL-PHASE-SPEC
status: Accepted
date: 2026-05-13
purpose: |
  Canonical Phase Spec shape for every phase under
  .omc/plans/milestones/M0X-slug/phases/P0Y-slug/. Carries entry/exit gates,
  in/out scope, implementation-plan list, acceptance commands, grit symbol space,
  ICM rationale fields, and dependency citations.
enforcing_fitness_lane: oya-foundry-fitness-plan-hierarchy
owner_team: council-architecture
related:
  - docs/templates/impl-plan-template.md
  - docs/templates/milestone-readme-template.md
  - docs/templates/INDEX.md
adrs_cited:
  - ADR-0053  # sanctioned primitives
  - ADR-0054  # scaffold-claim pattern
  - ADR-0056  # BNF v4.1
doc_status: published
---

```yaml
# Required frontmatter for every phase spec file
---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M0X-<slug>          # e.g. M03-first-paying-tenant
phase: P0Y-<slug>              # e.g. P01-hr-payroll-scaffold
status: Proposed | Active | Complete
entry_gate: |
  Exact condition that must be true before this phase begins.
  Name the prior phase and its exit criterion, or "none" for P01.
  Example: "M03/P00 complete; oya-tenancy-kernel ships; cargo check clean."
exit_gate: |
  Exact condition that declares this phase complete.
  Must reference at least one cargo/CI command and one grit primitive.
  Example: "All impl-plan acceptance gates green; grit done called on all
  phase symbols; ICM phase-handoff row emitted."
depends_on:
  - milestone: M0X
    phase: P0Z-<slug>
    reason: "<one line>"
owner_team: <team-id>
---
```

# P0Y-<slug>: <one-line phase title, present tense>

## Purpose

One paragraph. What this phase delivers and why it sits here in the milestone
sequence. State which Master Plan principles it advances (cite by number from
`.omc/plans/MASTERPLAN.md §2`). Present tense; durable outcome framing.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `<µservice>` | `<bc1>`, `<bc2>` | `crates/oya-<ms>-<bc>-<layer>/` | `oya-<ms>[-<bc>]-<layer>` |

Naming justification for any NEW crate introduced in this phase (mandatory per
`feedback_naming_justification.md`):

```
NAME: oya-<microservice>[-<bc-tokens>]-<layer>
JUSTIFICATION:
- microservice = <kebab-token(s)>: <rationale; ADR-0056 v4.1 flat BNF>
- bc-tokens = <kebab-token(s)> (OPTIONAL): <rationale>
- layer = <layer>: <ADR-0056 §"Layer semantics" rule>
- exemptions claimed: <none | cite exception>
```

### Out-of-scope

- `<item>` — deferred to `M0X/P0Z-<slug>` because `<reason>`.
- `<item>` — owned by `<µservice>` executor; not touched here.

---

## Implementation Plans

Ordered list. Each IP is an executable plan file under this phase directory.

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`<impl-plan-name>.md`](<impl-plan-name>.md) | <one-line> | pending \| open \| in-progress \| merged | `<team-id>` |

---

## Acceptance Gates

All gates must pass before `exit_gate` is declared. Each row is a runnable
command + expected exit code.

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P0Y-<slug>   # LEAN-A1: layer ordering
oya gate validate lean-a2 --phase P0Y-<slug>   # LEAN-A2: cross-vertical refusal
oya gate validate lean-a3 --phase P0Y-<slug>   # LEAN-A3: BC boundary
oya gate validate lean-a4 --phase P0Y-<slug>   # LEAN-A4: naming conformance
```

### Workflow + Ontology integration gates

```bash
# Verify typed events registered in Workflow
oya gate validate workflow-event-registry --phase P0Y-<slug>
# Verify Ontology object types registered
oya gate validate ontology-type-registry --phase P0Y-<slug>
```

---

## Clean Architecture Compliance

Executor must declare the following before scaffolding any new crate in this
phase (per `feedback_clean_architecture_requirements.md`):

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-<ms>-<bc>-kernel` | `kernel` | Yes — list traits | N/A | No |
| `oya-<ms>-<bc>-domain` | `domain` | N/A — calls through ports | N/A | No |
| `oya-<ms>-<bc>-application` | `application` | N/A | N/A | No |
| `oya-<ms>-<bc>-adapter` | `adapter` | N/A | Yes — implements kernel ports | No |
| `oya-<ms>-<bc>-rest` | `rest` | N/A | No direct adapter import | Yes |
| `oya-<ms>-<bc>-app` | `app` | N/A | Unrestricted inward (wiring only) | No |

### Port traits declared in kernel (list all new ones)

```rust
// oya-<ms>-<bc>-kernel/src/ports.rs
pub trait <RepositoryTrait>: Send + Sync {
    async fn find_by_id(&self, id: &<IdType>) -> Result<<Entity>, ...>;
    async fn save(&self, entity: &<Entity>) -> Result<(), ...>;
}

pub trait <ServiceTrait>: Send + Sync {
    async fn <operation>(&self, ...) -> Result<..., ...>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P0Y-<slug>` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P0Y-<slug>` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P0Y-<slug>` | exit 0 |
| `layer-correctness` | `oya gate validate layer-correctness --phase P0Y-<slug>` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P0Y-<slug>` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P0Y-<slug>` | exit 0 |

### New BCs registered in this phase

Each new BC must be registered in `docs/standards/bounded-contexts.md` using
`docs/templates/bounded-context-registration-template.md` before the first
crate in that BC's family can be scaffolded.

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `<bc-name>` | `<µservice>` | `<PR link or "pending">` |

---

## Grit Claim Symbols

Symbol space for this phase. Claim ALL before beginning; release with
`grit done --agent <id>` at IP completion (per `feedback_grit_claim_work_done.md`).

```
# Format: file::Identifier
crates/oya-<ms>-<bc>-<layer>/src/lib.rs::<TypeOrTrait>
crates/oya-<ms>-<bc>-<layer>/src/<module>.rs::<fn_name>
contracts/<surface>.openapi.yaml::<operationId>
docs/standards/<file>.md::Section
```

TTL recommendation: `--ttl 3600` (1 h) per IP; re-claim if exceeding.

Fallback: if grit v0.3.0 has no active locks, use ICM topic
`scaffold-locks-oyatie` as coordination ledger (per ADR-0054
§"scaffold-claim pattern").

---

## ICM Rationale Fields

Emit these ICM rows at phase completion (mandatory per `CLAUDE.md`):

```bash
# At phase start
icm store \
  -t context-oyatie \
  -c "Phase P0Y-<slug> started; milestone M0X-<slug>; scope: <µservices>; entry gate met: <evidence>" \
  -i high \
  -k "M0X,P0Y,phase-start,<µservice>"

# At phase complete
icm store \
  -t context-oyatie \
  -c "Phase P0Y-<slug> complete; IPs merged: <list>; grit symbols released: <list>; lanes green: <list>; next phase: P0Z-<slug>" \
  -i high \
  -k "M0X,P0Y,phase-complete,<µservice>"
```

---

## References

- Milestone README: `../../README.md`
- Bominal ADRs inherited: `<ADR-NNNN (inherited)>`
- oyatie ADRs cited: `ADR-XXXX`
- Memory files: `feedback_milestone_phase_hierarchy.md`,
  `feedback_grit_claim_work_done.md`, `feedback_naming_justification.md`
