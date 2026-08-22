---
doc_class: Template
template_id: TPL-BC-REG
status: Accepted
date: 2026-05-13
purpose: |
  Schema for registering a new Bounded Context in docs/standards/bounded-contexts.md.
  Every BC must be registered before a crate carrying its bc-tokens can be scaffolded.
  Carries naming justification (BNF v4.1), ownership, Ontology entries, Workflow
  events, and acceptance criteria.
enforcing_fitness_lane: governance-plan-hierarchy
owner_team: council-architecture
related:
  - docs/standards/bounded-contexts.md
  - docs/templates/microservice-template.md
  - docs/templates/INDEX.md
adrs_cited:
  - ADR-0056  # BNF v4.1 BC-optionality rule
  - ADR-0125  # domain naming canon (Tenant/Org/User/Person/Employee distinctions)
  - ADR-0106  # Ontology architecture
  - ADR-0103  # Workflow hexagonal
doc_status: published
---

# Bounded Context Registration: `<bc-name>`

---

## BC Name

**Name (kebab):** `<bc-name>` (e.g., `employee`, `payroll-period`, `messenger`)

**Naming justification** (mandatory per `feedback_naming_justification.md`):

```
BC NAME: <bc-name>
JUSTIFICATION:
- Token(s): <kebab-token(s)> — <why this name; what domain concept it maps to>
- Owner µservice: <µservice-name> — <why this BC belongs to this µservice
  and not another; cite entity ownership rule per Bominal ADR-0125 if applicable>
- Distinctness: <why this is a separate BC and not a module within an existing BC>
- BNF v4.1 placement: bc-tokens slot is USED because <µservice has multiple
  BCs at the same layer / multiple binaries with BC-level splits>
  OR
  bc-tokens slot OMITTED because <µservice has a single concept at this layer>
```

---

## Owner µservice

**µservice:** `<µservice-name>`

This BC belongs to `<µservice-name>` because: `<one-line ownership rationale>`.

No other µservice may directly import crates in this BC's family. Cross-µservice
access flows through Workflow events or Ontology reads (LEAN-A2 enforcement).

---

## Purpose

One to two sentences. What domain problem this BC solves. Which entities it owns.
Which invariants it enforces at its boundary.

---

## Entities

All entity types owned by this BC. Register each as an Ontology Object Type
(per Bominal ADR-0106 + `feedback_glossary_ontology_not_object_graph.md`).

| Entity | Ontology Object Type | Key properties | Audit trail |
|---|---|---|---|
| `<EntityName>` | `<ObjectTypeName>` | `id`, `tenant_id`, `<field>` | Ed25519 audit event on every mutation |

**Pillar assignment** (per Bominal ADR-0132 org/person pillar):
- `org-pillar`: `<entities owned by an org; e.g. Employment, Department>`
- `person-pillar`: `<entities owned by a person; e.g. PersonProfile, Credentials>`

---

## Workflow Events Produced

Events this BC emits to the Workflow adapter layer. Consuming µservices
subscribe; they do not call this BC directly (LEAN-A2).

| Event type | Trigger | Payload shape | Consumed by |
|---|---|---|---|
| `<EventType>` | `<condition>` | `{ tenant_id, <fields> }` | `<µservice>` |

---

## Workflow Events Consumed

Events from other µservices that this BC reacts to via Workflow subscription.

| Event type | Produced by | Handler | Action |
|---|---|---|---|
| `<EventType>` | `<µservice>` | `<use-case>` | `<mutation or side-effect>` |

---

## Ontology Writes

Object Types and Link Types this BC writes to the Ontology substrate.

| Object Type | Link Type | Written by | RLS scope |
|---|---|---|---|
| `<ObjectType>` | `<LinkType>` (or `-`) | `<use-case>` | `tenant_id` partition |

---

## Crate Family (BNF v4.1)

Expected crate names for this BC. Fill the justification block for each new
crate (mandatory per `feedback_naming_justification.md`):

| Crate | Layer | Purpose |
|---|---|---|
| `oya-<ms>-<bc>-domain` | `domain` | Entities + port-traits |
| `oya-<ms>-<bc>-application` | `application` | Use-case orchestrators |
| `oya-<ms>-<bc>-infrastructure` | `infrastructure` | Port-trait impls |
| `oya-<ms>-<bc>-rest` | `rest` | REST handler wiring (if applicable) |
| `oya-<ms>-<bc>-grpc` | `grpc` | gRPC handler wiring (if applicable) |

---

## Acceptance Criteria for BC Registration

A BC is considered validly registered when ALL of the following are true:

1. Entry added to `docs/standards/bounded-contexts.md` with this template filled.
2. Crate family listed above exists in `[workspace.members]` of root `Cargo.toml`.
3. Each crate carries `[package.metadata.oya]` with `microservice`, `bc`, and `layer` fields.
4. Naming justification block present in this registration AND in each crate's
   `[package.metadata.oya]` or associated ADR.
5. Buck2/cloud-ci lean-a3 gate target exits 0 for `<bc-name>` (BC boundary check).
6. the Buck2/cloud-ci naming-conformance gate target exits 0 for `<bc-name>`.
7. Ontology Object Types registered in Ontology type registry.
8. Workflow event types registered in Workflow event registry.

---

## References

- Owner µservice PRD: `docs/prds/<µservice-name>.md`
- Bominal ADR-0125: domain naming canon (Tenant/Org/User/Person/Employee)
- Bominal ADR-0106: Ontology architecture (Object Types + Link Types)
- ADR-0056: BNF v4.1 (BC-optionality rule)
- ADR-0057: LEAN-A2 cross-vertical refusal + LEAN-A3 BC boundary
- Memory: `feedback_naming_justification.md`,
  `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`
