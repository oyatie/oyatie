---
doc_class: Reference
shape: Reference
length_cap: 300
microservice: policy
companion_docs:
  - policy/CONTRACT.md
  - policy/PROMOTION.md
related_adrs:
  - ADR-0615
  - ADR-0280
  - ADR-0702
  - ADR-0711
inbound_citations: []
---

# policy

The Cedar-backed **PBAC+ReBAC authorization decision plane** — the 24th registered capability
(`governance/capability-registry.json`, `dag_node: policy-engine`). Extracted from the coarse `iam`
collapse by ADR-0615 §5 so that the decision plane and the identity provider are separately owned:
`policy` **consumes** the verified principal `iam` produces, and is **consumed by ~all** capabilities'
PEPs — the gateway edge and every protected service.

It is **not** a singleton global PDP. It is cell-distributed (ADR-0280 §D-13.D).

## Faces

- **G face** — `policy.authoring.cp` in `specs/substrate-dependency-dag.json`. Authoring, signing and
  distribution of policy versions and ReBAC tuples. The control plane.
- **C0 face** — `policy.local-pdp`. The per-cell runtime PDP plus its last-known-good versioned
  tenant-policy / ReBAC snapshot store. The data plane.

## The invariant this capability exists to hold

ADR-0280 §D-13.E, the **static-stability invariant**:

> Cell runtime never synchronously depends on the G-plane. Existing sessions and routes continue on
> cached, signed, versioned state (policy bundles, tenant directory, placement snapshots). Only *new*
> identity / tenant / placement / migration operations may safely stop when G is unavailable — and
> even those fail closed (deny or route-to-authoritative), never fail open.

and §D-13.D, which states the rule every artifact here is measured against:

> A stale snapshot must **DENY or route to the authoritative shard — never silently authorize**.

## Entry points

- `CONTRACT.md` — the C0 snapshot-store port contract: the staleness-bounded, versioned
  last-known-good store that ADR-0280 §D-13.E requires and that no crate in the tree implements yet.
- `policy/` — eight Cedar fragments governing this capability's own control surface (who may author,
  sign, publish, activate and distribute), plus `schema.cedarschema`. Authority is carried by entity
  membership and entity references, never by a string the caller supplies.
- `cedar/policies.cedar` — the consolidated bundle; the fragments are closed under concatenation.
- `cedar/CONFORMANCE.md` — what was measured against the real Cedar engine, and the mutation matrix
  that shows the suite can fail.
- `cedar/README.md` — the two authoring rules these fragments follow, and what they do not buy.
- `runbooks/` — Day-2 procedures for the failure modes the invariant names.
- `PROMOTION.md` — what this capability still owes, and the exact out-of-envelope edits that unblock
  it. Read this before adding a crate here.

## What is deliberately NOT here

The Cedar PDP crates that physically live under `iam/**` **stay iam-mapped** (ADR-0615 §5, verbatim:
"the Cedar PDP crates physically under `iam/**` stay iam-mapped to avoid a double-map"). That is
`iam/core/policy-cedar-domain`, `iam/ports/policy-cedar-api`, `iam/core/cloud-pdp-kernel`,
`iam/adapters/pdp-cedar`, `iam/adapters/cloud-pdp-bundle-file` and `iam/facade/cloud-pdp-app`.
This capability builds **alongside** them and depends inward on the shared kernels
(`libs/oya-shared-pdp-kernel`, `libs/oya-shared-platform-contracts-kernel`). Nothing is moved;
ADR-0711 B-1a forbids rename-only waves.

`absorbs_current_dirs` is `[]` and `oya/policy/` does not exist — this is a greenfield birth, not a
migration.

## A note on authority

Every ADR this capability leans on for *design* is Superseded and lives in `docs/adr-archive/`:
ADR-0615 and ADR-0280 (→ **ADR-0701**), ADR-0243 and ADR-0538 (→ **ADR-0700**), ADR-0294 and ADR-0191
(→ **ADR-0702**). They are cited here as **provenance for the design**, because the live apexes are
consolidations that preserve one-paragraph gists rather than the face model, the static-stability
invariant or the soak window in full. The quotations were checked against the archived text; the
*authority* is the successor apex in each case. No file in `policy/**` writes a
`docs/decisions/ADR-…` path, and `policy/` is not an authority surface, so `adr-citation-closure`
raises nothing — this note is about honesty, not about a gate.

## Hyperscaler precedents

AWS Verified Permissions (Cedar, store-per-tenant) as against AWS IAM; Google Zanzibar
(logically-global, physically-distributed-with-local-replicas) as against Google IAM.
