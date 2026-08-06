---
id: ADR-0326
status: Superseded
date: 2026-05-20
owner: council-compliance
owners:
  - council-architecture
  - council-privacy
  - council-security
  - council-compliance
  - council-product
  - axis-tenancy
  - axis-policy-engine
  - axis-audit-chain
  - axis-storage
  - axis-network
  - ops-compliance
  - ops-sre-reliability
supersedes: []
amends:
  - ADR-0244-tenant-scoping-primitive.md (adds residency dimension to tenancy envelope)
  - ADR-0246-cellular-architecture.md (binds cell placement to declared residency)
  - ADR-0251-compliance-pack-primitive.md (residency attestations become a compliance pack obligation)
  - ADR-0263-audit-event-registry-doctrine.md (adds residency-attestation event classes)
  - ADR-0304-cross-jurisdiction-conflict-resolution.md (codifies residency as a first-class conflict primitive)
superseded_by: [ADR-0702]
related:
  - ADR-0144
  - ADR-0145
  - ADR-0151
  - ADR-0152
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0247
  - ADR-0248
  - ADR-0249
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0263
  - ADR-0301
  - ADR-0304
  - ADR-0306
  - ADR-0311
  - ADR-0312
  - ADR-0313
  - ADR-0322
  - ADR-0325
  - ADR-0327
related_specs:
  - /specs/tenant-model.json
  - /specs/residency-attestation-schema.json
  - /specs/cellular-topology-schema.json
  - /specs/compliance-pack-schema.json
  - /specs/audit-events/registry.json
  - /specs/cedar-fragment-schema.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/residency-doctrine.md
  - docs/decisions/ADR-0244-tenant-scoping-primitive.md
  - docs/decisions/ADR-0246-cellular-architecture.md
  - docs/decisions/ADR-0251-compliance-pack-primitive.md
inbound_citations:
  - docs/decisions/ADR-0244-tenant-scoping-primitive.md
  - docs/decisions/ADR-0325-capability-tier-pricing-anchors-public.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
purpose: >
  Define per-tenant data residency as a first-class tenancy attribute,
  define the named residency tiers (multi-region, single-region,
  sovereign-cell, airgapped-cell), define the attestation mechanism
  through which a tenant's residency assertion is signed, verified, and
  auditable, define the audit event classes added to the ADR-0263
  registry for residency events, define the Cedar policy pattern that
  enforces residency at request time, and cross-reference the named
  regulatory regimes whose residency clauses this ADR satisfies.
enforcement_status: blocker-day-one
enforced_by:
  - oya-governance-residency-attestation
  - oya-governance-residency-cell-placement-enforcement
  - oya-governance-residency-cross-border-bar
  - oya-governance-residency-audit-event-completeness
  - oya-governance-residency-pack-binding
decision_owner: council-compliance
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0326: Per-Tenant Data Residency Attestation

## Status

Proposed (2026-05-20). The residency tiers and attestation mechanism are
canonical from the publication date; existing tenants without explicit
residency declarations are migrated to `multi_region` default and given a
30-day window to upgrade per the migration plan (D-10).

## Context

### Named pressure

The ADR-0325 pricing anchor cross-references a `residency` dimension but
ADR-0325 alone does not define the dimension's semantics; this ADR is its
companion specification. Beyond pricing, residency pressure arises from
multiple regulatory regimes whose residency clauses the substrate must
satisfy:

- **EU CSRD (Corporate Sustainability Reporting Directive)** — requires
  data residency for sustainability-reporting datasets such that the
  controller can demonstrate the data did not leave EU jurisdictional
  control during processing.
- **KR PIPA (Personal Information Protection Act)** — requires that
  personal data of Korean residents either remains within Korean
  jurisdictional control or carries an explicit cross-border transfer
  consent record.
- **India DPDP (Digital Personal Data Protection Act)** — requires that
  personal data of Indian residents follow residency rules with named
  carve-outs for processors and named cross-border transfer registries.
- **Brazil LGPD (Lei Geral de Proteção de Dados Pessoais)** — analogous
  residency rules plus an ANPD attestation requirement for processors.
- **Saudi Arabia PDPL (Personal Data Protection Law)** — strict in-Kingdom
  residency requirements; sovereign-cell deployment expected for
  in-Kingdom tenants.
- **UAE PDPL** — similar in-region residency expectation with explicit
  registry of cross-border transfers.
- **Australia Privacy Act (Notifiable Data Breaches scheme)** — requires
  residency-relevant breach reporting; binds residency to incident
  response.
- **China PIPL (Personal Information Protection Law)** — strict residency
  rules; outside-China access requires CAC (Cyberspace Administration of
  China) approval; air-gapped sovereign cell expected.
- **EU AI Act (Regulation (EU) 2024/1689)** — high-risk system data sets
  carry residency obligations that compose with model-lifecycle
  obligations from ADR-0308.

ADR-0304 (cross-jurisdiction conflict resolution) sketched the
multi-jurisdiction conflict primitive; this ADR makes residency a
first-class attribute of the conflict-resolution input set rather than a
context dimension.

### Named constraints

- **C-1 Tenant-first** — per ADR-0244, every record carries tenant
  context; residency is a property of the tenant context, not an
  orthogonal concept.
- **C-2 Cellular topology** — per ADR-0246, cells are the unit of
  substrate isolation; residency tiers map onto cell-placement rules.
- **C-3 Cedar gate** — per ADR-0243, every policy decision is a Cedar
  eval; residency enforcement is Cedar-expressible.
- **C-4 Compliance packs** — per ADR-0251, packs are the unit of
  regulatory composition; certain packs imply residency requirements
  that this ADR's tier mapping satisfies.
- **C-5 Audit chain** — per ADR-0263, residency events are registered
  audit classes.
- **C-6 Tenancy precedence** — per the conglomerate-tenant-hierarchy
  doctrine (ADR-0313), a sub-tenant's residency cannot be more
  permissive than its parent tenant's residency.
- **C-7 No silent regression** — per `feedback_no_silent_regression.md`,
  any change to a tenant's residency must be explicit, attested, and
  audited.

### Named prior incidents

- **Incident I-1 (2026-04-12)**: a KR-resident tenant's data was
  transiently routed through an EU cell during a multi-region failover;
  no PIPA violation was triggered because the data was encrypted in
  transit, but the lack of a residency-attestation record meant the
  controller could not prove non-leakage. Required an after-the-fact
  attestation reconstruction effort.
- **Incident I-2 (2026-05-03)**: a tenant claiming `single_region` was
  provisioned into a multi-region cell because the placement code did
  not honour the declared tier; postmortem at
  `docs/postmortems/postmortem-residency-placement-divergence-2026-05-03.md`.
- **Incident I-3 (2026-05-15)**: a CSAP-pack KR tenant requested a quote;
  the absence of a residency attestation mechanism delayed the quote by
  five business days while engineering manually verified the residency
  configuration.

## Decision

Residency is a first-class tenant attribute with four named tiers:

- **R-1 `multi_region`** — tenant accepts the global default cell
  topology; data may move freely across regions; no cross-border bar.
- **R-2 `single_region`** — tenant's data must stay within a named region
  (e.g. `region: eu_west`, `region: kr_central`). The cell-placement
  enforcer admits the tenant only to cells in the named region.
- **R-3 `sovereign_cell`** — dedicated cell with no shared substrate
  components from outside the cell's jurisdictional control. Sovereign
  cells have their own control plane, key management substrate, and
  audit-chain shard scoped within the jurisdiction.
- **R-4 `airgapped_cell`** — fully air-gapped cell with no network path
  to the public oyatie control plane. Updates and patches arrive via a
  named offline-channel protocol; audit events are exported via a
  named offline-channel protocol.

Each tier is paired with an attestation: a signed JSON record at
`/evidence/residency/<tenant-id>/<attestation-id>.signed.json` that
captures the tier declaration, the regulatory regime it satisfies, the
named compliance packs it supports, the named cell or cell-group, and
the signing principals (tenant administrator + oyatie council-compliance
agent + relevant regulatory pack attestor where applicable).

Cell placement is enforced by `oya-governance-residency-cell-placement-
enforcement` at provisioning and at every subsequent placement decision
(failover, scaling, migration). Cross-border transfers are gated by
`oya-governance-residency-cross-border-bar` Cedar fragments that consult
the tenant's attestation and the requested operation's residency footprint.

## Consequences

Treating residency as a first-class tenant attribute with the R-1..R-4 tiers and signed attestations means cell placement and cross-border operations are gated against each tenant's attested residency footprint via Cedar policy; the detailed mechanics, SLO implications, and migration path below enumerate the operational consequences of that enforcement.

## Detailed Mechanics

### D-1 Residency attestation schema

`/specs/residency-attestation-schema.json` defines the record. Required
fields:

- `attestation_id` (UUID v7).
- `tenant_id` (string; per ADR-0244 envelope).
- `tier` (enum: `multi_region`, `single_region`, `sovereign_cell`, `airgapped_cell`).
- `region` (string; required for `single_region` and `sovereign_cell`;
  values from the named region registry at `/specs/regions/registry.json`).
- `cell_id_or_group` (string; cell identifier or cell-group identifier;
  required for `sovereign_cell` and `airgapped_cell`).
- `regulatory_regimes` (array<string>; e.g. `["EU_CSRD", "KR_PIPA"]`;
  values from the named regime registry at
  `/specs/regulatory-regimes/registry.json`).
- `compliance_packs` (array<string>; per ADR-0251).
- `tenant_signing_principal` (string).
- `tenant_signature` (ed25519, base64url).
- `oyatie_compliance_signing_principal` (string).
- `oyatie_compliance_signature` (ed25519, base64url).
- `regulatory_attestor_signature` (optional; required when the regime
  registry entry mandates a third-party attestor).
- `effective_at` (RFC 3339).
- `expires_at` (RFC 3339; default = effective_at + 365 days; renewable).
- `previous_attestation_id` (optional; references the prior attestation if
  this one is a refresh or upgrade).

The schema is a Tier-2 artifact governed by ADR-0322.

RESIDENCY-001 materializes the contract-only slice through exact tracked
paths `specs/residency-attestation-schema.json`,
`specs/regions/registry.json`,
`specs/regulatory-regimes/registry.json`,
`specs/compliance-pack-residency-matrix.json`,
`specs/residency-placement-audit-events.json`,
`specs/fixtures/residency/residency-001-signed-footprint.fixture.json`,
`specs/fixtures/residency/residency-001-cross-border-refusal.fixture.json`, and
`ci/facade/contract-slice-conformance/contract-slice-policy.json` (slices
`residency-001-attestation`, `residency-001-regions`, `residency-001-regimes`,
`residency-001-audit-events`, `residency-001-pack-matrix`). These artifacts are
schema, registry, fixture, and owned Rust/Buck2 gate validation evidence only;
they do not claim runtime
attestation, placement enforcement, cross-border data-plane enforcement,
tenant migration, pack activation, certification, audit-chain delivery, or
production readiness.

### D-2 Region and regime registries

The region registry (`/specs/regions/registry.json`) and the regulatory
regime registry (`/specs/regulatory-regimes/registry.json`) are the
canonical sources of valid values for the `region` and `regulatory_regimes`
fields. Each registry entry is itself a small documentation artifact
governed by ADR-0322 (≥120 lines, Tier-4); the registries are governed at
Tier-2 (≥500 lines).

Sample regulatory-regime entries (initial set; extensible by amendment):

- `EU_CSRD` — EU Corporate Sustainability Reporting Directive.
- `EU_GDPR` — EU General Data Protection Regulation.
- `EU_AI_ACT` — EU Regulation (EU) 2024/1689.
- `KR_PIPA` — Korea Personal Information Protection Act.
- `IN_DPDP` — India Digital Personal Data Protection Act.
- `BR_LGPD` — Brazil Lei Geral de Proteção de Dados Pessoais.
- `SA_PDPL` — Saudi Arabia Personal Data Protection Law.
- `AE_PDPL` — UAE Personal Data Protection Law.
- `AU_PRIVACY_ACT` — Australia Privacy Act.
- `CN_PIPL` — China Personal Information Protection Law.
- `US_HIPAA` — US Health Insurance Portability and Accountability Act
  (treated as a residency regime when paired with the HIPAA pack).
- `US_CCPA` — California Consumer Privacy Act.
- `JP_APPI` — Japan Act on the Protection of Personal Information.
- `CA_PIPEDA` — Canada Personal Information Protection and Electronic
  Documents Act.

### D-3 Cell-placement enforcement

`oya-governance-residency-cell-placement-enforcement` is invoked at every
placement decision in the substrate. Inputs:

- Tenant ID.
- Operation type (provision, scale, failover, migrate).
- Candidate cell set.

The crate consults the tenant's current attestation, filters the candidate
cell set to those compatible with the declared tier, and refuses
placement if no candidate remains. For sovereign-cell and airgapped-cell
tiers, the placement is pre-pinned to a specific cell and the crate
verifies that the operation targets that cell.

Placement decisions emit `tenancy.residency.placement.decided` events
(see D-7) with full provenance.

### D-4 Cross-border bar

`oya-governance-residency-cross-border-bar` is consulted at the data-plane
level for any operation that may move data across a cell boundary. The
crate determines whether the source and destination cells lie within the
tenant's declared residency footprint. Mechanisms:

- Synchronous read operations from outside the residency footprint are
  refused by Cedar policy (see D-7 Cedar fragments).
- Asynchronous replication paths declare a residency footprint at
  configuration time; the crate verifies that the footprint is a subset
  of the declared tier's footprint.
- The crate emits `tenancy.residency.cross_border.refused` events on
  refusals, including the requested cell pair, the tenant's tier, and
  the operation identifier.

For `multi_region` tenants the cross-border bar is permissive but still
audits the crossings (so that an after-the-fact regulatory inquiry can
reconstruct data flows).

### D-5 Attestation refresh and rotation

Attestations expire after 365 days by default. A refresh:

- Carries `previous_attestation_id` to chain the new attestation to the
  prior one.
- Reuses the prior tier unless an explicit upgrade is declared.
- Re-runs all signers; signatures from the prior attestation do not
  carry over.
- Emits `tenancy.residency.attestation.refreshed`.

A rotation (change of signing principal) is a refresh with a different
signing principal value. The chain of attestations forms an attestation
ledger queryable by regulatory inquiries.

### D-6 Tier upgrade and downgrade

Tier upgrades (e.g. `multi_region` → `single_region` → `sovereign_cell`
→ `airgapped_cell`) are permitted at any time. Each upgrade may trigger:

- Re-placement to a compatible cell (possibly with data migration).
- Re-keying under the cell's KMS.
- New pricing per the ADR-0325 residency uplift schedule (effective at
  next renewal).

Tier downgrades are permitted but require explicit acknowledgement of
loss of guarantees plus a residency-downgrade attestation signed by the
tenant administrator. Downgrades emit
`tenancy.residency.tier.downgraded` events at WARN severity.

Tier changes are atomic from the data-plane perspective: until the
re-placement completes, all data continues to honour the prior tier.
The transition window is documented in
`docs/operations/residency-tier-transition-playbook.md`.

### D-7 Audit event class additions

Added to the ADR-0263 registry:

| Class                                              | Severity | Source crate                                        |
|----------------------------------------------------|----------|-----------------------------------------------------|
| tenancy.residency.attestation.created              | INFO     | oya-governance-residency-attestation                |
| tenancy.residency.attestation.refreshed            | INFO     | oya-governance-residency-attestation                |
| tenancy.residency.attestation.expired              | WARN     | oya-governance-residency-attestation                |
| tenancy.residency.attestation.signature.invalid    | BLOCKER  | oya-governance-residency-attestation                |
| tenancy.residency.attestation.missing              | BLOCKER  | oya-governance-residency-attestation                |
| tenancy.residency.placement.decided                | INFO     | oya-governance-residency-cell-placement-enforcement |
| tenancy.residency.placement.refused                | BLOCKER  | oya-governance-residency-cell-placement-enforcement |
| tenancy.residency.cross_border.refused             | BLOCKER  | oya-governance-residency-cross-border-bar           |
| tenancy.residency.cross_border.permitted           | INFO     | oya-governance-residency-cross-border-bar           |
| tenancy.residency.tier.upgraded                    | INFO     | oya-governance-residency-attestation                |
| tenancy.residency.tier.downgraded                  | WARN     | oya-governance-residency-attestation                |
| tenancy.residency.pack.binding_changed             | INFO     | oya-governance-residency-pack-binding               |
| tenancy.residency.regulatory_attestor.engaged      | INFO     | oya-governance-residency-attestation                |

Each class carries the canonical envelope; the `tenant_id` field is the
controlled-data tenant rather than `oyatie.governance` because the events
are about a specific tenant's data.

### D-8 Pack binding

Some compliance packs (per ADR-0251) imply residency obligations:

- HIPAA pack: implies at least `single_region` with the region anchored
  to US jurisdictions where HIPAA applies, or a sovereign cell.
- GDPR pack: implies at least `single_region` with the region anchored to
  EU/EEA.
- CSAP pack (KR Cloud Security Assurance Program): implies
  `sovereign_cell` within KR.
- PCI pack: residency-neutral but binds the cell to PCI-DSS scope rules.
- SOC2 pack: residency-neutral.
- EU AI Act pack: composes with GDPR pack rules.

`oya-governance-residency-pack-binding` enforces these implications: a
tenant that activates HIPAA pack without an at-least-`single_region`
attestation has its pack activation refused; the refusal emits a
`tenancy.residency.pack.binding_changed` event with detail.

### D-9 Conflict resolution interplay with ADR-0304

ADR-0304 (cross-jurisdiction conflict resolution) treats residency as a
context dimension. With this ADR, residency becomes a first-class input
to the conflict resolver. Mechanism:

- The conflict resolver consults the tenant's current attestation as a
  primary input.
- Where two regulatory regimes give conflicting answers, the regime
  whose residency footprint matches the operation's actual physical
  cell wins, modulo the explicit override list in ADR-0304.
- The conflict resolver's decision carries the attestation chain in its
  evidence.

### D-10 Migration of existing tenants

Pre-ADR tenants:

- Default to `multi_region` tier with an implicit attestation generated
  by `oya-governance-residency-attestation` at the ADR's effective date,
  signed by the council-compliance agent and a placeholder tenant
  signature with a 30-day grace window.
- Receive a notification (per the tenant's preferred channel) that
  residency declaration is now expected; the 30-day window allows the
  tenant administrator to upgrade tier or formalise the multi-region
  attestation.
- After 30 days without an explicit tenant signature, the tenant
  receives a `WARN` event (`tenancy.residency.attestation.expired`); the
  attestation continues to function but is flagged for follow-up.
- After 90 days without explicit tenant signature, the attestation is
  marked invalid; new operations against the tenant fall back to the
  most restrictive permitted residency (`multi_region` retained as the
  default but cross-border operations now require an explicit consent
  record).

This migration is documented in
`docs/operations/residency-migration-2026-05-20.md`.

### D-11 Audit-chain replication and residency

The audit chain itself carries data that may be subject to residency
restrictions. The doctrine resolves this:

- Audit events tagged with a tenant's tier are stored in the audit
  chain shard that is co-located with the tenant's primary cell.
- Cross-shard replication of audit events is bounded by the tenant's
  residency footprint; events of a `sovereign_cell` tenant are not
  replicated outside the sovereign-cell shard.
- Audit-event queries that cross a residency boundary are subject to
  the cross-border bar Cedar fragment in D-7.
- Regulatory inquiries are served from the in-jurisdiction shard;
  cross-jurisdiction inquiries require explicit court-warrant
  piercing per ADR-0312.

The audit chain's compliance with this rule is verified by
`oya-governance-residency-audit-event-completeness` daily.

### D-12 Backup and disaster-recovery alignment with residency

Backups, snapshots, and disaster-recovery copies of tenant data
inherit the tenant's residency tier:

- A `single_region` tenant's backups are stored in additional cells
  within the same region; cross-region backup is forbidden.
- A `sovereign_cell` tenant's backups are stored in the sovereign-
  cell substrate; no shared-substrate backup nodes participate.
- An `airgapped_cell` tenant's backups are exported via the offline
  channel protocol; the export carries the tenant's residency
  attestation as metadata.
- Disaster recovery failover targets are pre-pinned by tier; failover
  to an out-of-tier cell is forbidden and emits a
  `tenancy.residency.dr_failover.refused` event.

This composes with the ADR-0306 disaster-mode doctrine.

### D-13 Per-pack residency interaction matrix

The interaction matrix captures the residency obligations that each
compliance pack imposes on a tenant. The matrix is canonical:

| Pack         | Min residency tier         | Min region (if any)            |
|--------------|----------------------------|--------------------------------|
| HIPAA        | single_region              | US jurisdictions               |
| GDPR         | single_region              | EU/EEA                          |
| SOC2         | (no minimum)               | (no minimum)                    |
| CSAP         | sovereign_cell             | KR                              |
| PCI          | (no minimum)               | (no minimum; PCI-DSS scope)     |
| EU_AI_ACT    | single_region (composed)   | EU/EEA (per GDPR composition)   |

Combined-pack tenants take the strictest constraint across all
activated packs. The matrix is published at
`/specs/compliance-pack-residency-matrix.json`.

### D-14 Sovereign-cell control-plane substrate

A sovereign-cell tenant gets its own slice of the substrate:

- A dedicated control plane within the cell's jurisdictional
  boundary; the cell does not call into the global control plane.
- A dedicated KMS instance with keys generated and rotated within
  the cell.
- A dedicated identity provider (or a regionally-anchored federation
  with the global IdP through a stateless gateway).
- A dedicated audit-chain shard that publishes events only to the
  in-jurisdiction sink.
- A dedicated observability stack with no cross-jurisdiction
  telemetry export.

The dedicated substrate is enumerated in
`docs/architecture/sovereign-cell-substrate-2026-05-20.md`.

### D-15 Air-gapped cell offline-channel protocol

The air-gapped tier requires a named offline-channel protocol:

- **Inbound (patches, updates, configuration)**: arrives via signed
  media (USB/SSD) couriered by oyatie operators; verified by the
  cell's local TUF metadata repository; applied after operator
  review.
- **Outbound (audit events, telemetry)**: exported daily via signed
  media; the export carries a Merkle root that the in-jurisdiction
  audit-chain sink verifies on receipt.
- **Emergency channel**: a one-way satellite link is permitted for
  emergency egress (e.g. critical security advisories); the link
  is not used for routine data movement.
- **Periodic operator presence**: a council-compliance operator
  physically visits the cell ≥ quarterly for attestation refresh
  and key rotation.

The protocol is documented in
`docs/operations/airgapped-cell-channel-2026-05-20.md`.

## Cedar Policy Hooks

```cedar
// Fragment: cedar/residency/operation-must-match-attestation.cedar
forbid (
  principal,
  action,
  resource
) when {
  resource has tenant_id &&
  context.operation_cell_jurisdiction != context.tenant_attested_jurisdiction &&
  context.tenant_attested_tier != "multi_region"
};
```

```cedar
// Fragment: cedar/residency/cross-border-data-export.cedar
forbid (
  principal,
  action == Data::"export_across_cell_boundary",
  resource is DataObject
) when {
  context.destination_cell_jurisdiction != context.tenant_attested_jurisdiction &&
  context.tenant_attested_tier in ["single_region", "sovereign_cell", "airgapped_cell"] &&
  context.explicit_cross_border_consent == false
};
```

```cedar
// Fragment: cedar/residency/airgapped-cell-no-network.cedar
forbid (
  principal,
  action,
  resource
) when {
  context.tenant_attested_tier == "airgapped_cell" &&
  context.operation_traverses_public_network == true
};
```

```cedar
// Fragment: cedar/residency/sovereign-cell-control-plane-isolation.cedar
forbid (
  principal == Service::"oyatie.platform.control_plane.cross_jurisdiction",
  action,
  resource is Cell
) when {
  context.cell_tier == "sovereign_cell" &&
  context.cell_jurisdiction != context.principal_jurisdiction
};
```

```cedar
// Fragment: cedar/residency/attestation-refresh-permitted.cedar
permit (
  principal in Group::"oyatie.tenant.administrators",
  action == ResidencyAttestation::"refresh",
  resource is TenantResidencyAttestation
) when {
  context.signing_principal_matches_tenant == true &&
  context.attestation_chain_valid == true
};
```

```cedar
// Fragment: cedar/residency/regulatory-attestor-required.cedar
forbid (
  principal,
  action == ResidencyAttestation::"finalize",
  resource is TenantResidencyAttestation
) when {
  context.regime_requires_third_party_attestor == true &&
  context.regulatory_attestor_signature_present == false
};
```

## Audit Event Classes Emitted

Detailed in D-7 above. Summary count: 13 new classes added to the
ADR-0263 registry. Each class carries the canonical envelope plus the
class-specific payload (tenant_id, tier, region, cell, attestation_id,
operation_id where applicable).

Sample payload fixture for `tenancy.residency.cross_border.refused`:

```json
{
  "event_class": "tenancy.residency.cross_border.refused",
  "event_id": "evt-7a9c...-b34e",
  "ts": "2026-05-20T14:23:11.103Z",
  "tenant_id": "acme-kr-finance",
  "principal": "Service::worker-pool-kr-central",
  "resource": "DataObject::trade-blotter-2026-05-20",
  "attestation_id": "att-9f2c...-118a",
  "tier": "single_region",
  "region": "kr_central",
  "requested_destination_cell": "eu_west_3",
  "operation_id": "op-bb12...-9c4f",
  "audit_chain_attested": true
}
```

## SLO Implications

`microservices/governance/residency/slos/residency.openslo.yaml`:

- `attestation_verification_p99_latency`: ≤ 25 ms (request-path hot).
- `cell_placement_decision_p99_latency`: ≤ 100 ms.
- `cross_border_bar_decision_p99_latency`: ≤ 50 ms.
- `attestation_refresh_p95_latency`: ≤ 5 s.
- `residency_audit_completeness`: ≥ 99.999% of placement and cross-border
  decisions produce audit events (the regulatory inquiry primitive
  depends on completeness).
- `airgapped_cell_offline_channel_latency`: ≤ 24 hours per export cycle.

## Migration Path / Phased Rollout

- **Phase 0 (T-0, ADR Proposed)**: schema, registries, and crates land in
  shadow mode; attestation generation begins for new tenants only.
- **Phase 1 (T+7 days)**: existing tenants receive implicit
  `multi_region` attestations; notifications dispatched.
- **Phase 2 (T+30 days)**: explicit tenant signatures expected; pack
  binding enforcement active for HIPAA, GDPR, CSAP.
- **Phase 3 (T+60 days)**: cross-border bar active in BLOCKER mode for
  `single_region` and stronger tiers.
- **Phase 4 (T+90 days)**: airgapped-cell offline-channel protocol
  operational; sovereign-cell attestation refresh cadence active.
- **Phase 5 (T+120 days)**: ADR eligible for promotion per ADR-0327.

## Failure Modes + Recovery

### F-1: Attestation expires unrenewed

A tenant's attestation reaches its 365-day expiry without renewal.
Recovery: the tenant continues to operate at the prior tier for a 14-day
grace window during which renewal reminders escalate; after 14 days, the
tenant is downgraded to `multi_region` automatically with a `WARN` event;
no data loss, but cross-border guarantees relax until renewal occurs.

### F-2: Signing principal compromised

A tenant administrator's signing key is compromised. Recovery: the
tenant invokes the key-rotation protocol (per ADR-0247 self-modification
doctrine for principals); a fresh attestation is signed by a new
principal; the compromised principal is added to the revocation list;
in-flight attestations signed by the compromised principal are revoked
and re-signed.

### F-3: Cell-placement enforcer false negative

A placement decision proceeds despite tier mismatch (regression bug).
Recovery: the audit chain's completeness check catches the missing
event; postmortem opens; the affected tenant receives a controlled
re-placement and an attestation-chain entry documenting the divergence.

### F-4: Airgapped cell offline channel fails

The offline channel (typically a courier or operator-driven media
transfer) fails. Recovery: the airgapped tenant's operations continue
locally; audit-export accrual is bounded at ≤72 hours before the
operator escalates per the ADR-0306 disaster mode runbook.

### F-5: Conflicting attestation between parent and child tenant

A child tenant attempts to declare a more permissive residency than its
parent (per ADR-0313 conglomerate hierarchy). Recovery: the attestation
service refuses the declaration and emits
`tenancy.residency.placement.refused`; the operator must either upgrade
the parent's tier or maintain the child at the parent's level.

### F-6: Regulatory attestor unavailable

A regime that requires a third-party attestor cannot produce a signature
in the contracted window. Recovery: the attestation remains in pending
state; the tenant continues at the prior tier; if the pending state
exceeds 30 days, the attestation is voided and the prior tier persists.

## Verification

Named CI checks:

- `oya-governance-residency-attestation/schema-roundtrip`
- `oya-governance-residency-attestation/signature-verification`
- `oya-governance-residency-cell-placement-enforcement/placement-respects-tier`
- `oya-governance-residency-cross-border-bar/refusal-on-mismatch`
- `oya-governance-residency-pack-binding/hipaa-implies-region`
- `oya-governance-residency-pack-binding/gdpr-implies-region`
- `oya-governance-residency-pack-binding/csap-implies-sovereign`
- `oya-governance-residency-audit-event-completeness`

Named crates:

- `oya-governance-residency-attestation`
- `oya-governance-residency-cell-placement-enforcement`
- `oya-governance-residency-cross-border-bar`
- `oya-governance-residency-audit-event-completeness`
- `oya-governance-residency-pack-binding`

Verification fixtures: `tests/governance/residency/` with scenarios for
each tier, each major regulatory regime, the parent-child hierarchy
case, the attestor unavailability case, the airgapped offline-channel
case, and the cross-tier upgrade and downgrade cases.

## Cross-References

### Other ADRs

- ADR-0144 / ADR-0145 (inter-microservice reform) — residency checks
  compose with direct gRPC invariants.
- ADR-0151 / ADR-0152 (audit-chain substrate) — events land in audit
  chain.
- ADR-0242 (oyatie tenant) — tenant doctrine.
- ADR-0243 (Cedar universal gate) — Cedar fragment convention.
- ADR-0244 (tenant scoping) — tenant envelope.
- ADR-0245 (substrate-product layering) — residency is a substrate
  primitive.
- ADR-0246 (cellular topology) — cell placement basis.
- ADR-0247 (self-modification doctrine) — principal rotation.
- ADR-0248 (Amazon cellular) — shuffle-sharding interaction with
  residency.
- ADR-0249 (multi-category marketplace) — marketplace residency.
- ADR-0251 (compliance packs) — pack binding.
- ADR-0252 (HLC default) — attestation timestamps under HLC.
- ADR-0253 (HTTP/3 default) — attestation API transport.
- ADR-0254 (K8s + Cloud Hypervisor) — cell substrate.
- ADR-0255 (intelligence two-layer) — AI workloads inherit residency.
- ADR-0263 (audit-event registry) — class registration.
- ADR-0301 (survivor safety) — survivor-mode tenants typically pin
  to `single_region` or `sovereign_cell`.
- ADR-0304 (cross-jurisdiction conflict) — residency is a first-class
  input.
- ADR-0306 (disaster mode) — airgapped offline-channel runbook
  inheritance.
- ADR-0311 (dual-tenant identity) — personal and work tenants may
  carry distinct residency tiers; the dual-tenant boundary respects
  both.
- ADR-0312 (court-warrant scoped piercing) — warrant-piercing audit
  records pinned to residency footprint.
- ADR-0313 (conglomerate tenant hierarchy) — parent-child residency
  precedence per F-5.
- ADR-0322 (substance bar) — residency artifacts subject.
- ADR-0325 (capability-tier pricing anchors) — residency uplift basis.
- ADR-0327 (wave-3 completion) — promotion gates consume.

### Standards

- `docs/standards/residency-doctrine.md` — companion Tier-2 standard
  (W2 wave).
- `docs/standards/documentation-rigor.md` §3.2.

### Microservices

- `microservices/governance/residency/` — substrate.
- `microservices/tenancy/attestation/` — attestation service.
- `microservices/cloud-network/cell-router/` — enforces cross-border bar.
- `microservices/storage/replication/` — declares per-pipeline footprint.
- `microservices/audit-chain/` — event sink.
- `microservices/observability/` — SLO substrate.

### Journeys

- `journeys/tenancy/jou-2026-05-20-declare-residency/` — tenant
  administrator journey.
- `journeys/tenancy/jou-2026-05-20-refresh-attestation/` — refresh
  journey.
- `journeys/tenancy/jou-2026-05-20-upgrade-to-sovereign/` — upgrade
  journey.
- `journeys/compliance/jou-2026-05-20-regulatory-inquiry-response/` —
  inquiry-response journey.

### Specs

- `/specs/residency-attestation-schema.json`
- `/specs/regions/registry.json`
- `/specs/regulatory-regimes/registry.json`
- `/specs/cellular-topology-schema.json`
- `/specs/tenant-model.json` (updated to carry residency dimension).

### External standards referenced

- Regulation (EU) 2022/2464 — EU CSRD.
- Regulation (EU) 2016/679 — EU GDPR.
- Regulation (EU) 2024/1689 — EU AI Act.
- KR Act No. 17347 — KR PIPA (current revision).
- IN Act No. 22 of 2023 — DPDP.
- BR Lei nº 13.709/2018 — LGPD.
- SA Royal Decree M/19 — SA PDPL.
- AE Federal Decree-Law No. 45 of 2021 — UAE PDPL.
- AU Privacy Act 1988.
- CN PIPL (2021).
- US 45 CFR Parts 160 and 164 — HIPAA.
- CA Civil Code §§1798.100–1798.199.100 — CCPA.
- JP APPI (Act on the Protection of Personal Information).
- CA PIPEDA (Personal Information Protection and Electronic Documents
  Act).

### Feedback notes consumed

- `feedback_compliance_pack_primitive.md`
- `feedback_no_silent_regression.md`
- `feedback_canonical_base_localization.md`
- `feedback_tenant_as_universal_scoping_primitive.md`
- `feedback_substrate_vs_product_layering.md`
- `feedback_amazon_shape_cellular_architecture.md`
- `feedback_build_ahead_of_certification.md`
