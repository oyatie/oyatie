---
id: ADR-TASKS-0001
status: Accepted
date: 2026-05-17
microservice: tasks
deciders: axis-tasks, council-architecture, council-product, ops-data-platform
owner: axis-tasks + council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0117
  - ADR-0135
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-NOTES-0001
related_artifacts:
  - microservices/tasks/PRD.md (FR-01; FR-09; FR-16; AC-04; AC-10; §Hyrum #2 + #7)
  - microservices/tasks/contracts/openapi/tasks.yaml
  - microservices/tasks/specs/naming-justification.md
purpose: |
  Close the PRD-tasks gap on how Task entities + per-project custom fields
  + importer field mapping are modelled. The model decision drives the
  Postgres schema (IP-004), the strict-coercion lane (IP-006), the gRPC
  contract (`Task` + `CustomFieldValue` oneof), and the importer mapping
  policy (IP-002 of the importers BC).
---

# ADR-TASKS-0001: Task data model + custom fields — hybrid typed-schema-per-project + flexible JSON; strict type coercion (refuse silent coerce); 6 importers with strict assignee resolution

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The `tasks` µservice serves a deeply heterogeneous workload: engineering
teams want Linear/Jira-style strict-issue-tracker fidelity; marketing
teams want Asana/Trello board flexibility; ops teams want Monday.com-
style spreadsheet-database. PRD §FR-16 mandates eight custom-field
kinds (text/number/date/dropdown/multi_select/person/url/checkbox);
PRD §FR-09 mandates six importers (CSV/Jira/Asana/Trello/Linear/Todoist)
with full custom-field preservation; PRD §"Hyrum" #2 + #7 warn against
silent type coercion.

Three modelling strategies compete:

1. **Typed schema per project** (Linear approach). Each project owns a
   strict relational sub-schema; custom fields become typed columns.
   Pros: O(1) per-field lookup; index-friendly; query planner sees real
   types. Cons: schema explosion across tenants; migration cost per
   change; importer mapping has to materialise typed columns for every
   field encountered.
2. **Flexible JSON** (Notion approach). Single JSONB column holds the
   entire custom-field bag. Pros: zero migration cost; importers trivial.
   Cons: silent coercion attack surface; type queries slow; no enforced
   typing without external validator; querying for "all tasks where
   field X is between 5 and 10" is a JSON-path scan.
3. **EAV (entity-attribute-value)**. Separate
   `task_custom_field_value` table keyed by `(task_id, field_id)`.
   Pros: arbitrary cardinality; middle-band query performance. Cons:
   N+1 joins for any non-trivial render; cannot enforce type at storage
   without a sidecar typing table; widely-criticised antipattern.

Strict typing matters because of the second-order downstream:
importers (FR-09) and AI features (FR-20 / ADR-TASKS-0006). If the
storage layer silently coerces `"42"` (string) to `42` (number) on
write, then a Jira import that misclassifies a string field as a number
field will corrupt customer data without any signal. Per PRD §"Hyrum"
#2 + #7 + the broader Linus-grade no-silent-regression doctrine, silent
coercion is refused.

## Decision

The tasks µservice ships a **hybrid model**:

- **Custom-field definitions** are typed-schema-per-project, stored in a
  `custom_field_definitions` table keyed by `(project_id, field_id)`
  with `kind ∈ {text, number, date, dropdown, multi_select, person, url, checkbox}`.
- **Custom-field values** are stored in a JSONB column on the `tasks`
  table (`custom_fields jsonb not null default '{}'`).
- **Strict type coercion at the domain layer**: writing a custom-field
  value MUST match the declared `kind` exactly. `"42"` written to a
  `Number` field returns `CustomFieldCoercion::Refused` (422). No
  implicit string→number; no implicit array→scalar truncation; no
  implicit ISO-8601 normalisation. Per ADR-0105 strict-typing doctrine.
- **Importers map field-by-field with refusal-on-ambiguity**:
  the CSV/Jira/Asana/Trello/Linear/Todoist importers each maintain an
  explicit `FieldMapping` per source format. Where the source's typing
  is ambiguous, the importer refuses the import row with a structured
  error rather than silently coerce — per PRD AC-10.
- **Assignee resolution is strict-email-match**: importers do NOT
  auto-create tenant users. An assignee email that doesn't match an
  existing tenant member is REFUSED at the row level with
  `AssigneeResolution::NoTenantMember`. Per PRD §Security importer
  sandboxing.
- **GIN index** on `tasks.custom_fields` for JSONB-path queries.

## Alternatives Considered

### Alternative 1 — Pure typed-schema-per-project (Linear-style)

- Pros:
  - Strongest typing; query planner has perfect information.
  - Best per-field lookup performance.
- Cons:
  - Schema explosion: 10k tenants × 30 fields/project × 100 projects/tenant
    = 30M columns across the cluster; Postgres `pg_attribute` bloat is
    measurable past 1600 columns/table.
  - Every field change becomes a DDL migration; competitors don't bear
    this overhead.
  - Importers can't pre-create columns at import time without DBA
    privileges; UX cost.
- Rejected because: schema explosion + importer DDL coupling are
  unacceptable at the 10k-tenant scale per PRD §"Horizontal Scalability".

### Alternative 2 — Pure flexible JSON (Notion-style)

- Pros:
  - Zero schema migration cost.
  - Importers trivial.
- Cons:
  - Silent-coercion attack surface — the load-bearing PRD §"Hyrum" #2
    + #7 refusal must be enforced somewhere; without a sidecar schema
    table, there is no authoritative typing to enforce.
  - No structural index on per-field cardinality without ad-hoc
    expression indexes.
- Rejected because: refusing silent coerce is non-negotiable;
  without an authoritative type registry per field, refusal is
  impossible.

### Alternative 3 — EAV (entity-attribute-value)

- Pros:
  - Mature pattern for arbitrary cardinality.
- Cons:
  - N+1 joins on read; PRD §"Performance" board render p95 ≤ 200ms is
    impossible against EAV with a 200-task project + 30 fields.
  - Widely-criticised antipattern (Fowler "EAV considered harmful").
- Rejected because: read-performance budget is incompatible with EAV
  shape.

## Consequences

### Consequence 1 — IP-004 + IP-006 + IP-013 land in close sequence

The Postgres schema (IP-004), the strict-coercion validator (IP-006),
and the OpenAPI/proto surface (IP-013) all encode the same typing
contract. They must land as a single bundleable ChangeSet group; a
partial landing would let a client write a typed field via REST that
the domain layer would then refuse on read.

### Consequence 2 — Importers carry per-source field maps as load-bearing artefacts

Each of the six importers (CSV/Jira/Asana/Trello/Linear/Todoist) must
ship a versioned `FieldMapping` lookup table that maps source-format
typing to the eight oyatie kinds. When a source schema changes upstream
(e.g., Linear adds a new field type), the importer must refuse rather
than silently coerce. Source-format version drift is a regression risk
tracked in `runbooks/importer-schema-drift.md`.

### Consequence 3 — Audit-chain seals attach to custom-field schema mutations

Adding / dropping a custom field is a tenant-visible policy event that
gets an audit-chain seal per Bominal ADR-0028 (Ed25519 + Merkle). This
prevents silent in-place field-kind reassignment (a known data-corruption
attack vector against typed-schema-per-project systems).

## References

- ADR-0056 (BNF v4.1); ADR-0105 (13-layer); ADR-0106 (usecase rename).
- ADR-0117 (residency); ADR-NOTES-0001 (Personal-context E2E default
  pattern; tasks shares the dual-context boundary).
- Bominal ADR-0028 (audit-chain Ed25519+Merkle); Bominal ADR-0111
  (envelope encryption).
- Fowler, "Patterns of Enterprise Application Architecture" — EAV
  critique.
- Postgres JSONB GIN — `www.postgresql.org/docs/16/datatype-json.html`.
- Linear API custom fields — `developers.linear.app`.
- Notion data model — `developers.notion.com/reference/property-object`.
- PRD-tasks §FR-01 + FR-09 + FR-16 + AC-10 + §"Hyrum" #2 + #7.
