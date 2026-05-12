# ADR-0011: Cross-axis contract registry — `contracts/axis-contracts.yaml` source-of-truth, openapi/proto/asyncapi sub-directories, oya-foundry-fitness-contracts CI lane, cross-axis contract change protocol, auto-generated SDKs

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-foundry` (registry surface) + `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0007, ADR-0010, ADR-0015, ADR-0019

---

## Context

The cohesion thesis (ADR-0001) hinges on cross-axis contracts being mechanical artifacts, not slide-deck text. DESIGN §10 enumerates ~25 cross-axis contract rows (Tenant kernel, Identity / RBAC, Capability invocation, Autonomy ceiling policy, Audit-chain event, Capability registry record, Plane class, Cloud resource type, Region/AZ/Cell, IAM/SSO, Object Graph property tier, Search index lifecycle, Ad slot inventory, Billing event, DSR cascade, Webhook delivery, Public REST stability tier, Marketplace listing, Eventing backbone, Cloud↔Search capacity & residency, Search↔Ads SERP & query privacy, Foundry↔Cloud mutation control, Foundry↔Search retrieval boundary, Tenant↔Ads/Analytics eligibility, Revenue/metering/tax invoice). Without a single registry, contracts drift independently and the cohesion claim degrades.

The ledger LEDG-007 records the prior under-coverage of contracts; LEDG-025 records ownership ambiguity. The resolution is to make the registry the *source of truth* for every contract — not a convenience copy of an OpenAPI spec scattered across axis-team READMEs. Auto-generated SDKs from the registry close the loop: every consumer pulls the same spec; drift between client and server becomes a versioning event, not silent corruption.

---

## Decision

We adopt **`contracts/axis-contracts.yaml`** as the source-of-truth registry of cross-axis contracts, with sub-directories for protocol-specific specs, a gating CI lane, an explicit cross-axis contract change protocol, and auto-generated multi-language SDKs.

### Registry layout

```
contracts/
  axis-contracts.yaml                # source of truth: every cross-axis contract row
  openapi/<contract-id>.yaml         # public REST + webhook contracts
  proto/<contract-id>.proto          # gRPC + service-to-service contracts
  asyncapi/<contract-id>.yaml        # event topic + schema contracts (CloudEvents per ADR-0005)
  cedar/<contract-id>.cedar          # Cedar policy contracts (ADR-0007)
  schemas/<contract-id>.json         # JSON Schema contracts (e.g. capability records, catalog records)
  sdks/                              # generated; never hand-edited
    rust/<contract-id>/
    typescript/<contract-id>/
    python/<contract-id>/
    go/<contract-id>/
```

### `axis-contracts.yaml` shape

```yaml
contracts:
  - id: TENANT_KERNEL
    owner_axis: SaaS
    consumer_axes: [Workspace, Vertical, Foundry, Cloud, Search, Ads]
    surface_kind: kernel-type
    source_of_truth: crates/oya-platform-tenant-kernel
    spec_paths: [proto/tenant-kernel.proto, schemas/tenant-record.json]
    plane: control
    review_class: cross-axis-tenant-mutation
    required_reviewers: [council-architecture, council-privacy, all-axis-leads]
    audit_emission_topic: oya.platform.tenant.kernel-mutation.v1
    sdk_languages: [rust, typescript, python, go]

  - id: CAPABILITY_INVOCATION
    owner_axis: Foundry
    consumer_axes: [SaaS, Workspace, Vertical, Cloud, Search, Ads]
    surface_kind: rest-api
    source_of_truth: crates/oya-foundry-runtime-api-*
    spec_paths: [openapi/capability-invocation.yaml]
    plane: control
    review_class: cross-axis-capability
    required_reviewers: [axis-foundry, consuming-axis-team]
    audit_emission_topic: oya.foundry.capability.invoked.v1
    sdk_languages: [rust, typescript, python, go]

  - id: AUDIT_CHAIN_EVENT
    owner_axis: Foundation
    consumer_axes: [SaaS, Workspace, Vertical, Foundry, Cloud, Search, Ads]
    surface_kind: event
    source_of_truth: crates/oya-platform-audit-chain-kernel
    spec_paths: [asyncapi/audit-chain.yaml]
    plane: control
    review_class: cross-axis-audit
    required_reviewers: [platform-audit-chain, downstream-consumer]
    audit_emission_topic: oya.platform.audit.event.v1
    sdk_languages: [rust, typescript, python, go]
  # ... (≈ 25 rows total per DESIGN §10)
```

### CI lane: `oya-foundry-fitness-contracts`

The lane runs on every PR and:

1. Validates that every contract row's `source_of_truth` crate exists and exports the named types.
2. Validates that every spec path exists and is syntactically valid (OpenAPI 3 / Protobuf / AsyncAPI 2.6 / Cedar / JSON Schema).
3. Detects breaking changes per protocol (`oasdiff` for OpenAPI, `buf breaking` for Protobuf, AsyncAPI delta for events, Cedar policy diff for policies).
4. Verifies the PR's labels include `cross-axis-<contract-id>` whenever the spec changes.
5. Hard-fails any contract change PR whose `## Code Review` block lacks the `required_reviewers` set.
6. Regenerates SDKs in `contracts/sdks/` and verifies the generated tree is committed (so consumers see the new SDK on the same SHA).

### Cross-axis contract change protocol

A PR that touches any contract row MUST:

- Carry the `cross-axis-<contract-id>` label (auto-applied by `oya-foundry-fitness-contracts`).
- Include a regenerated diff of the affected SDK in the PR's `## Evidence` section.
- Carry sign-off from the `required_reviewers` set in the `## Code Review` block.
- Emit `EVT-CROSS-AXIS-CONTRACT-CHANGED` to the audit chain (ADR-0003) at merge time.
- Cite the contract row id in the PR's `## Traceability` section.

Breaking-change protocol (per ADR-0019 deprecation governance):

- Wave N: introduce v(n+1) alongside v(n); both shipped.
- Wave N+1: deprecation-warning emission on v(n) consumers.
- Wave N+2: v(n) removal.

### SDK generation auto from registry

The registry is the single input to per-language SDK codegen (`crates/oya-tooling-sdk-gen-*`). SDKs are committed under `contracts/sdks/<lang>/<contract-id>/` so consumers (axis crates, external ISVs, regional packs) pull the SDK like any other dep. Hand-edited SDKs are rejected by the lane.

### Boundary

- Applies to: every cross-axis contract per DESIGN §10 (Tenant kernel through Revenue/metering).
- Does not apply to: per-axis-internal APIs (within one axis, the per-axis convention applies; if a surface becomes consumed cross-axis, it is promoted to the registry).

---

## Consequences

### Positive

- The registry becomes the auditable cohesion artifact. "Are all cross-axis contracts respected?" is a CI question.
- Closes LEDG-007 (cross-axis contracts incomplete) and LEDG-025 (ownership ambiguity) at the registry level.
- SDK consumers pull a versioned, signed contract; drift between server and client surfaces as a build break, not a runtime mystery.
- Auditor evidence: the registry generates a contract-change history per release.

### Negative

- Promotion bar for axis-internal → cross-axis is real; some PRs become contract-level events.
- Codegen pipelines for four SDK languages add CI time; mitigation: per-PR-affected SDK regen only.
- Per-row required_reviewers can stall a PR if a reviewer is unavailable; mitigation: per-team rotation policy + bypass procedure (`# review-bypass: <reason>` per CLAUDE.md, logged + audit-emitted).

### Operational

- On-call: `EVT-CONTRACT-LANE-FAIL-RATE > N` and `EVT-CROSS-AXIS-CONTRACT-CHANGED` daily rollups.
- Runbooks: `runbooks/contract-introduction.md`, `runbooks/contract-breaking-change.md`, `runbooks/sdk-regen-failure.md`.
- CI: `oya-foundry-fitness-contracts` is a P0 lane; failure blocks merge.
- Per-quarter audit by `council-architecture`: every contract row reviewed for actuality; orphan rows removed.

---

## Alternatives considered

### Alternative A — OpenAPI files scattered per-axis (no registry)

- **Pros:** zero centralization cost.
- **Cons:** drift demonstrated; cross-axis impact analysis impossible; SDK generation per-axis.
- **Rejected because:** LEDG-007.

### Alternative B — Backstage / Cortex catalog with per-service entries

- **Pros:** mature tooling.
- **Cons:** Backstage is service-cataloging, not contract-cataloging; cross-axis-contract semantics absent.
- **Rejected because:** scope mismatch.

### Alternative C — Per-spec-format registry (separate OpenAPI registry, separate Protobuf registry, etc.)

- **Pros:** per-format ergonomics.
- **Cons:** the cohesion-relevant data — owner axis, consumer axes, plane, review class — is the same regardless of protocol; splitting destroys the single review surface.
- **Rejected because:** cohesion review needs a single artifact.

---

## Open questions

1. **Q1.** SDK languages — limit to `[rust, typescript, python, go]` or include `[java, csharp, kotlin, swift]` for mobile + enterprise? Default: limit to four for v1; expand per demand. → owner: `axis-foundry`.
2. **Q2.** Versioning across protocols — uniform semver per contract-id, or per-spec-file semver? Default: per-contract-id; spec-files share the contract version. → owner: `axis-foundry`.
3. **Q3.** Public-facing REST contracts (consumed by external ISVs) — additional stability tier requirement? Default: yes; per ADR-0019 doc-update protocol classifies them as stable surfaces requiring 2-wave deprecation notice. → ADR-0019.
4. **Q4.** Cedar policy contracts — versioned as cross-axis contracts, or per-tenant artifacts? Default: cross-axis (the policy *contract* is shared; per-tenant policies are local impls). → ADR-0007.
5. **Q5.** Per-pack seam contracts (ADR-0010) — included in the registry or in a sibling pack-contracts.yaml? Default: included in this registry with `surface_kind: pack-seam`. → ADR-0010.

---

## References

- `docs/DESIGN.md` §10 (cross-axis contract surface — full table; new contract `FOUNDATION_BUILDER_CONTRACT_REGISTRY` row), §11 (cross-axis contradiction audit)
- `docs/CONTRADICTION-LEDGER.md` LEDG-007 (cross-axis contracts incomplete), LEDG-025 (ownership ambiguity)
- `docs/TOOLCHAIN.md` §3 ("Schema-first SDK gen — In-house Rust codegen emitting Rust + TS + Python + Go SDKs from OpenAPI + protobuf + AsyncAPI")
- `docs/STANDARDS-AND-TEMPLATES.md`
- ADR-0001 (cohesion), ADR-0002 (Tenant kernel as registry row), ADR-0003 (audit emission per contract change), ADR-0004 (plane field on every contract), ADR-0005 (event schemas as contracts), ADR-0006 (OG property tier as contract), ADR-0007 (Cedar policy contracts), ADR-0010 (pack seams as contracts), ADR-0015 (flat-crates convention referenced from contracts), ADR-0019 (doc-update + deprecation governance)
- OpenAPI 3.2, Protobuf 3, AsyncAPI 2.6, Cedar policy spec
