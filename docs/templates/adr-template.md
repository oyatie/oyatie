---
doc_class: Template
template_id: TPL-ADR
status: Accepted
date: 2026-05-13
purpose: |
  Canonical ADR shape for every oyatie architectural decision. Enforces BNF v4.1
  naming justification, Bominal inheritance citation, and concrete file-path consequences.
  Every executor authoring a new ADR MUST start from this template.
enforcing_fitness_lane: oya-governance-plan-hierarchy
owner_team: council-architecture
related:
  - docs/standards/naming.md
  - docs/decisions/ADR-INDEX.md
  - docs/templates/INDEX.md
adrs_cited:
  - ADR-0056  # BNF v4.1 + layer enum
  - ADR-0057  # LEAN checks
  - ADR-0346  # historical full CI mirror semantics; destination is the Cargo workspace merge path
doc_status: published
---

# ADR-####: <Decision Title — imperative present tense>

---

## Frontmatter

| Field | Value |
|---|---|
| **id** | ADR-#### |
| **title** | <Decision Title> |
| **status** | Proposed \| Accepted \| Superseded |
| **date** | YYYY-MM-DD |
| **supersedes** | ADR-YYYY (or `-`) |
| **superseded_by** | ADR-ZZZZ (or `-`) |
| **owner** | `<team-id>` from `docs/teams/` |
| **related** | comma-separated ADR identifiers, or `-` |
| **bominal_source** | Bominal ADR-#### (inherited) \| oyatie override \| no Bominal equivalent |

---

## Context

What is the problem? What forces drove the decision? Quote relevant constraints
(regulatory, technical, organizational). Cite Bominal ADRs that bear on this decision
using the format: "per Bominal ADR-#### (inherited)" or "per oyatie
[[memory-slug]] (override)".

State whether this decision **inherits** from Bominal (default) or **overrides**
it (per `feedback_bominal_inheritance_precedence.md`).

Two paragraphs maximum unless the context requires deeper exposition.

---

## Decision

The decision in declarative form. Active voice. Present tense. Specific.

If the decision introduces new artifact names (crates, binaries, modules, BCs),
include the **Naming Justification block** for EACH name (mandatory per
`feedback_naming_justification.md`):

```
NAME: oya-<microservice>[-<bc-tokens>]-<layer>
JUSTIFICATION:
- microservice = <kebab-token(s)>: <product/capability name; registered in
  [workspace.metadata.oya.microservices]; cite ADR-0056 v4.1 flat BNF — no
  shared|vertical bisection>
- bc-tokens = <kebab-token(s)> (OPTIONAL): <omit when µservice has single
  binary/concept; include when µservice has multiple binaries or BC-level splits
  at the same layer; cite ADR-0056 v4.1 BC-optionality rule>
- layer = <layer>: <which of the 12 enum values; cite ADR-0056 §"Layer semantics"
  rule that fits: e.g., "use-case orchestrator holding port-trait bounds →
  application", "composition-root binary → app", "framework/driver glue →
  infrastructure">
- exemptions claimed (if any): <cite specific BNF exception, e.g.,
  "check-namespace per ADR-0056 line 79-80">
```

Glossary enforcement (auto-reject if any of these appear unqualified):
- "platform" → use "shared"
- "Object Graph" → use "Ontology"
- "Application Shell" / "Modular Product Shell" → use "Application"
- "Product Group" / "Arm" → use flat µservice catalog
- `shared|vertical` BNF slot → use open kebab µservice name (BNF v4.1)

---

## Consequences

### Concrete file and crate changes

List every file path, crate name, or layer assignment that changes as a result
of this decision. An autonomous executor must be able to act without escalation:

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `crates/oya-<ms>-<bc>-<layer>/` | create \| rename \| delete | `oya-<ms>[-<bc>]-<layer>` | `<layer-enum>` |
| `docs/standards/<file>.md` | update | — | — |

### Integration via Workflow + Ontology

State which typed events this decision emits to Workflow and which Object Types /
Link Types it writes to Ontology (per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`):

- **Workflow events produced**: `<EventType>` — consumed by `<µservice>`
- **Workflow events consumed**: `<EventType>` — produced by `<µservice>`
- **Ontology writes**: Object Type `<T>`, Link Type `<L>`
- **Ontology reads**: Object Type `<T>` via `<query shape>`

If this ADR does not touch Workflow or Ontology directly, state "not applicable"
and cite the integration point in the affected µservice's PRD.

### Positive
- Bullet list of benefits.

### Negative
- Bullet list of trade-offs (be honest).

### Operational
- CI lane changes (new LEAN check, fitness lane flip from `--report-only` to BLOCKER).
- ADR-0346 verification posture: the retired `./bin/oya verify --ci-required` path is historical/provenance-only; required verification is the Cargo workspace command set and the `oya-ci-required` context.

---

## Clean Architecture Impact

State which CI lanes this decision affects. Required before ADR can be marked Accepted.

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Affected \| Not affected | `<new crate added; layer declared>` or `none` |
| `cross-product-refusal` (LEAN-A2) | Affected \| Not affected | `<new cross-product boundary introduced>` or `none` |
| `port-location` | Affected \| Not affected | `<port trait moved to kernel>` or `none` |
| `layer-correctness` | Affected \| Not affected | `<layer enum value declared>` or `none` |
| `composition-root-only` | Affected \| Not affected | `<app-layer binary changed>` or `none` |
| `sdk-kernel-only` | Affected \| Not affected | `<SDK crate added/changed>` or `none` |

Port traits introduced by this decision (must live in `kernel` layer, per
`feedback_clean_architecture_requirements.md` §3):

```rust
// In oya-<ms>-<bc>-kernel/src/ports.rs
// Port trait declarations — ZERO business logic; ZERO I/O

#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait <PortTrait>: Send + Sync + sealed::Sealed {
    async fn <method>(&self, ...) -> Result<..., ...>;
}
```

Implementations live in `oya-<ms>-<bc>-adapter` (or `-infrastructure`).
Domain calls through port; domain never imports adapter.

---

## Alternatives Considered

For each alternative:

**Alternative N — <name>**
- Description
- Pros
- Cons
- Reason rejected

---

## References

- Bominal ADR-####: <title> (inherited \| translated \| overridden)
- oyatie memory: `feedback_<slug>.md` (override rationale)
- ADR-0056 BNF v4.1 (naming authority)
- ADR-0057 LEAN checks (cross-vertical enforcement)
- ADR-0346 historical full CI mirror semantics; Cargo/`oya-ci-required` destination authority
- Related oyatie ADRs: list concrete ADR identifiers, or state `-`
- Issues: `Refs #N`, `Closes #N`
