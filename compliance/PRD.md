---
doc_class: Owner-PRD
owner: compliance
status: Active
last_interviewed: 2026-08-26
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Compliance product requirements

<product_boundary>

`compliance/` is the cloud's evidence and Compliance-as-a-Service capability.
It catalogs and binds signed Compliance-as-Code packages, previews and
publishes owner-specific projections, evaluates evidence coverage, and exports
verifiable evidence manifests through one tenant-facing facade.

It does not own root `packs/` data, the Policy evaluator, Audit's tamper log,
Data/Storage/Audit retention execution, DLP, DSR orchestration, eDiscovery, a
trust portal, or a GRC application. First-party applications and marketplace
plugins consume the same CaS contract as every other tenant.

The present tree does not provide that product. It contains typed Rust models
for off-charter Workspace behavior, including a retention evaluator that emits
purge/delete permits through a direct Data-core dependency, plus unconsumed
deployment/SLO documents. All seven packages are terminal burn inventory;
these are current-state facts, not availability or conformance claims.

</product_boundary>

<users>

- Tenant compliance administrators need an authenticated catalog, immutable
  bindings, projection previews, evidence coverage, and portable exports.
- Audit, Data, and Storage owners need deterministic, versioned pack
  projections they can apply idempotently without surrendering authority.
- Policy needs verified pack fragments and schema revisions off the serving
  path; CaS must never become another `Check`.
- First- and third-party applications need the same public contract and tenant
  isolation, not direct pack filesystem or Compliance-core access.
- Operators need bounded cells, replay, restore, stale-version fencing,
  capacity signals, safe upgrades, and exact gap/failure telemetry.

</users>

<requirements>

## Catalog and pack admission

- Admit only bounded, signed, content-addressed pack envelopes with canonical
  package id, semantic version, schema revision, plane, Cedar-schema projection
  dimensions, payload digest, signer/key generation, and validity interval.
- Apply the numeric parser, identifier, list, fan-out, queue, idempotency, and
  export ceilings in `SPEC.md`. Limit-plus-one fails before allocation or
  mutation; operators may lower but never raise protocol hard maxima.
- Compute SHA-256 over exact payload bytes and verify Ed25519 over the v1
  domain-separated canonical binary preimage. Resolve trusted verification
  keys through the agreed key port; envelope-provided keys are never trust.
- Support the v1 namespaces `us`, `eu`, `jp`, and `kr`; reject combinatoric
  country ids and unknown namespace, plane, dimension, or schema values.
- Preserve immutable historical descriptors for evidence verification. A
  changed payload requires a new digest and version; a lower or reused version
  with different content fails closed.
- Treat root `packs/` as external CaC input. No facade request reads git or the
  pack filesystem, and no current Markdown/YAML scaffold is called admitted.

## Bind, list, preview, and project

- Bind a package to a tenant/control scope with compare-and-swap against the
  expected catalog and binding generations, verified authorization, trusted
  interval, idempotency fingerprint, and durable audit receipt.
- List only catalog entries and bindings visible to the verified tenant and
  principal. Pagination tokens bind tenant, revision, cursor, filters, and
  expiry.
- Preview which package fragments and obligations match a proposed
  principal/action/resource/context input, but return no product authorization
  decision and do not compile a private policy snapshot.
- Publish immutable target projections by generation. Retention projections
  carry the classification selector, obligation, target owner, effective
  interval, source pack/binding digest, and supersession rule.
- A target acknowledgement is idempotent and generation-bound. Missing,
  duplicate, reordered, stale, or foreign-tenant receipts remain visible as
  incomplete rather than being inferred away.

## Data-class registry

- Own immutable, versioned entries keyed by the exact `data-classification`
  value identity, with aliases, applicability, evidence obligations, source
  pack/schema/digest, validity, state, and monotonic registry generation.
- Admit registry changes through compare-and-swap. Duplicate aliases,
  conflicting values, stale generations, lower/equal conflicting versions,
  unknown classification values, and unsupported schema fail before mutation.
- Never copy or wrap the classification types. A provider-identity move remains
  a separate Data-led D-29 migration across every consumer.

## Evidence manifests and export

- Consume authenticated Audit references and build coverage manifests against
  the exact bound package, schema, projection, and source cursor/range.
- Mark coverage complete only when every required evidence class is present,
  tenant-bound, digest-verified, and within its required interval.
- Export a deterministic manifest and referenced artifacts through a bounded
  job. Audit remains event authority and Storage may hold export bytes; CaS
  does not create a fifth operational record store.
- Preserve lineage across rebinding, supersession, export retry, key rotation,
  and schema upgrade. An old manifest always names the old immutable inputs.

## Security and isolation

- Authenticate and authorize before catalog disclosure, binding mutation,
  projection publication, target acknowledgement, or export admission.
- Bind decisions to tenant, principal, operation, resource scope, policy
  revision, issuer, audience, request, idempotency identity, and expiry.
- Use mTLS internally, tenant-scoped encryption references, trusted Cell time,
  bounded queues, and durable pre-ACK Audit evidence for privileged control
  operations.
- Reject cross-tenant ids, forged or expired policy evidence, stale binding
  epochs, digest/signature mismatch, unavailable Audit authority, and changed
  idempotency fingerprints before mutation or disclosure.

## Portability and operations

- Ship owned Rust contracts and engines with no mandatory external GRC suite,
  PostgreSQL, Redis, Kafka, or proprietary policy runtime as authority.
- Consume Packs, Policy, Audit, Data, Storage, Cell, IAM, Secrets, and
  Observability only through agreed ports/adapters. Draft ports are owner-local
  and cannot gain external consumers without D-29 review.
- Support deterministic replay, snapshots, point-in-time restore, mixed-version
  upgrades, schema negotiation, drain, and cell-loss recovery before
  production promotion.
- Expose catalog/binding revision, projection lag, evidence gaps, export queue
  age, stale refusals, signature failures, per-tenant work/bytes, and unit cost.

</requirements>

<service_objectives>

## Target SLOs

These are promotion objectives, not claims about the current tree. They are
measured at the sold facade under a declared admitted-load and cell profile.

| Signal | Target |
|---|---:|
| CaS admitted-request availability | At least 99.99% per cell per calendar month |
| Catalog/list/preview latency | p99.9 at or below 250 ms at 70% admitted cell load, excluding caller WAN |
| Binding commit latency | p99.9 at or below 500 ms under the same profile |
| Projection publication freshness | p99 at or below 60 seconds from committed binding to durable target-ready projection |
| Evidence export admission | p99.9 at or below 1 second from accepted request to durable queued job |
| Required evidence coverage | 100% before a manifest can report `COMPLETE` |
| Malformed/stale pack refusal | 100% before catalog, binding, or projection mutation |
| Tenant isolation | Zero cross-tenant disclosure, mutation, projection, or evidence inclusion |
| Acknowledged binding durability | RPO 0 within the declared quorum/device tolerance |
| In-tolerance recovery | p99 at or below 15 minutes from fault declaration to admitted CaS service and caught-up projection publication |

No compliance-certification, legal-conformance, availability, or evidence-
coverage marketing claim is valid until the relevant pack, plant, workload,
and independent audit evidence measure it.

</service_objectives>

<promotion_targets>

Production promotion requires:

- differential pack parsing and canonical digest fixtures;
- independent golden canonical-preimage, SHA-256, Ed25519, key-revocation, and
  hard-bound limit/limit-plus-one fixtures;
- registry compare-and-swap, supersession, replay, alias-conflict, and exact-
  type compatibility fixtures;
- deterministic binding/projection replay and stale-generation fencing;
- default-deny contract tests through the ordinary gateway and Policy path;
- Audit-gap and target-receipt reconciliation under loss, duplication, reorder,
  outage, and recovery;
- snapshot restore plus N/N+1 protocol/schema upgrade and rollback barriers;
- noisy-tenant tests proving bounded queues and isolation; and
- removal of every off-charter package and unconsumed artifact named in
  `PLAN.md`.

Success means a tenant can bind a verified pack, obtain deterministic
owner-specific projections, and export complete, verifiable evidence without a
second PDP, log, or private path. Failure includes stale or malformed pack
acceptance, authorization by CaS, a complete manifest with a gap, a projection
applied to the wrong tenant/generation, acknowledged binding loss, unbounded
queues, or a current scaffold presented as production evidence.

</promotion_targets>

<non_goals>

- Implementing legal advice, certification, DLP, DSAR, eDiscovery, breach
  notification, or an auditor/trust portal.
- Evaluating product authorization, storing ReBAC tuples, or fetching packs on
  a serving request.
- Owning Audit records or directly applying Data/Storage/Audit retention.
- Replacing the target engine with a vendor GRC system or treating docs and
  deployment YAML as executable Compliance-as-Code.

</non_goals>
