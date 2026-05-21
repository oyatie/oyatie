# IP-014 Healthcare Integration Marketplace DealSet Settlement

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Date: 2026-05-20
Owner: axis-healthcare-integration
Capability focus: commercial settlement for clinical interoperability transactions
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Primary local citations:
- microservices/healthcare-integration/PRD.md
- microservices/healthcare-integration/ARCHITECTURE.md
- microservices/healthcare-integration/cost-budget.md
- microservices/healthcare-integration/capacity-model.md
- microservices/healthcare-integration/compliance.md
- microservices/healthcare-integration/dpia.md
- microservices/healthcare-integration/capabilities/fhir-read.yaml
- microservices/healthcare-integration/capabilities/hl7-route.yaml
- microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml
- microservices/healthcare-integration/capabilities/consent-sync.yaml
- microservices/healthcare-integration/contracts/openapi-v1.yaml
- microservices/healthcare-integration/contracts/asyncapi-v1.yaml
- microservices/healthcare-integration/dashboards/tenant-cost-and-capacity.json
- microservices/healthcare-integration/runbooks/dealset-provider-network-hold.md
- docs/standards/documentation-rigor.md
- specs/root-hub-pointers.json
- specs/master-plan-sequencing.json

## 1. Executive Intent
- This IP binds healthcare-integration commercial activity to marketplace DealSet settlement.
- The settlement path exists because clinical integrations often create chargeable provider-network, source-system, transformation, validation, and replay work.
- The settlement path must never make emergency care wait for payment rails.
- The settlement path must never expose PHI to billing records.
- The settlement path must preserve tenant scope, source-system provenance, data class, workflow run, and audit-chain linkage.
- The settlement path turns ADR-0314 from a generic marketplace statement into a healthcare B2B operating control.
- It makes settlement evidence portable across vendor networks.
- It avoids opaque per-connector pricing hidden inside middleware projects.
- It gives tenants a cost and entitlement ledger for each clinical exchange.
- It lets product leaders claim healthcare integration without silently accepting vendor lock-in or billing leakage.
- It keeps commercial logic out of healthcare domain invariants by following ADR-0105 layer separation.
- It keeps policy decisions before provider access by following ADR-0243 and ADR-0244.
- It keeps documentation current against ADR-0321 without editing ADR-0321.

## 2. B2B Leader Problem
- Redox-like integration networks often bundle commercial access, connectivity, and operational support into the network relationship.
- Interface-engine projects often hide transformation cost until after message volume grows.
- Enterprise healthcare tenants need deal terms, usage, and settlement disputes tied to auditable integration events.
- SMB healthcare tenants need predictable caps so a backfill or replay does not create surprise charges.
- Marketplace providers need proof that a transaction was authorized, delivered, transformed, and accepted before settlement.
- Compliance teams need billing records that exclude PHI but preserve enough linkage for audit.
- SRE teams need settlement hold controls when source systems fail or replay produces duplicate events.
- Finance teams need unit economics by tenant, pack, source system, data class, route, and workflow.

## 3. Settlement Objects
- `DealSetBinding` links tenant, provider, route, capability, and pack.
- `ClinicalExchangeCharge` records chargeable non-PHI usage.
- `SettlementEvidenceRef` links charge to audit-chain event id.
- `ProviderNetworkHold` pauses settlement when delivery or compliance evidence is incomplete.
- `ReplayAdjustment` reverses duplicate or corrected replay charges.
- `EmergencySettlementDeferral` marks emergency access as authorized before commercial finalization.
- `ResidencySurcharge` is allowed only when the pack and contract allow it.
- `TransformationUnit` counts parse, map, validate, normalize, enrich, and provenance-seal work.
- `MessageRouteUnit` counts HL7 route deliveries by source system and accepted destination.
- `FHIRBundleUnit` counts FHIR read or exchange operations without payload content.
- `ConsentSyncUnit` counts consent reconciliation outcomes.
- `PatientMatchReviewUnit` counts reviewer queue work and resolution outcome.
- `EvidenceExportUnit` counts regulator or tenant audit export packets.
- Every settlement object must carry tenant id, source system id, cell id, capability id, and policy decision id.
- No settlement object may carry raw PHI.

## 4. Scope
- Define DealSet settlement for `fhir-read`.
- Define DealSet settlement for `hl7-route`.
- Define DealSet settlement for `consent-sync`.
- Define DealSet settlement for `ehr-provenance-seal`.
- Define DealSet settlement for `patient-match-review`.
- Define emergency deferral rules for `break-glass-authorize`.
- Define settlement hold when audit-chain evidence is incomplete.
- Define settlement hold when source-system acknowledgement is missing.
- Define adjustment rules for backfill and replay.
- Define cost-budget integration.
- Define capacity model integration.
- Define tenant dashboard dimensions.
- Define marketplace dispute packet.

## 5. Non-Goals
- Do not implement the marketplace service itself.
- Do not move payment rails into healthcare-integration.
- Do not expose PHI in invoices, event names, metrics, or DealSet records.
- Do not turn settlement into an authorization prerequisite for emergency access.
- Do not create vendor-specific settlement code paths.
- Do not create a suite boundary around healthcare commerce.
- Do not replace the cost-budget enforcer in IP-017.
- Do not edit ADR-0321.

## 6. Implementation Steps
- Add a settlement-port trait in the application boundary.
- Add a DealSet reference to each chargeable clinical exchange result.
- Emit settlement-intent events only after Cedar permit and workflow acceptance.
- Emit settlement-final events only after delivery, provenance, and audit evidence are complete.
- Emit settlement-hold events when delivery, consent, residency, or audit evidence is incomplete.
- Emit settlement-reversal events when replay identifies duplicate delivery.
- Emit settlement-adjustment events when a transform fix changes accepted units.
- Compute charge units from typed counts, not payload inspection.
- Attach charge units to trace id, workflow id, source system id, and capability id.
- Keep pricing rules in marketplace/DealSet ownership.
- Keep charge facts in healthcare-integration evidence ownership.
- Redact patient identifiers before settlement emission.
- Hash non-PHI correlation keys when needed for dispute lookup.
- Add dashboard panels for billed units, held units, reversed units, and disputed units.
- Add runbook path for provider network hold.
- Add replay adjustment path for IP-016.
- Add budget hook for IP-017.
- Add capacity hook for IP-018.

## 7. Settlement State Machine
- `planned` means the tenant has an active DealSet binding.
- `authorized` means clinical policy permitted the underlying operation.
- `accepted` means workflow accepted the work.
- `delivered` means the source or destination acknowledgement is complete.
- `sealed` means provenance and audit references exist.
- `settlement_intent_emitted` means chargeable units are ready for marketplace.
- `held` means settlement cannot finalize.
- `finalized` means marketplace accepted the evidence.
- `reversed` means a later replay corrected the unit.
- `disputed` means tenant or provider challenged the unit.
- `expired` means settlement evidence missed its freshness window.
- State transitions must be idempotent.
- State transitions must include policy decision id.
- State transitions must include audit event id.
- State transitions must not include raw clinical payload.

## 8. Privacy Boundary
- Billing records can include tenant id.
- Billing records can include provider id.
- Billing records can include source system id.
- Billing records can include capability id.
- Billing records can include route class.
- Billing records can include data class.
- Billing records can include count and size buckets.
- Billing records can include workflow id.
- Billing records can include audit event id.
- Billing records cannot include patient name.
- Billing records cannot include MRN.
- Billing records cannot include FHIR body.
- Billing records cannot include HL7 segment payload.
- Billing records cannot include free-text clinical reason.
- Billing records cannot include consent text.
- Billing records cannot include emergency incident narrative.

## 9. Benchmark Displacement
- Redox displacement: Redox monetizes network and integration access; this IP separates clinical authorization from non-PHI settlement evidence and gives tenants unit-level dispute control.
- Rhapsody displacement: Rhapsody can operate high-volume routes; this IP adds DealSet-native settlement states and replay-based adjustment rather than leaving charges in project accounting.
- InterSystems IRIS for Health displacement: IRIS can consolidate data and apps; this IP keeps settlement as portable marketplace evidence outside suite storage.
- Lyniate/Corepoint displacement: Corepoint-style interface work can create service-heavy billing; this IP exposes route, transformation, and evidence units directly to tenant dashboards.
- Mirth Connect displacement: Mirth channel customization makes cost attribution ad hoc; this IP binds every chargeable unit to typed capability events.
- NextGate displacement: NextGate identity resolution can create review work; this IP makes patient-match review units auditable without embedding identity payload in billing.
- Health Catalyst displacement: Health Catalyst analytics cost often follows downstream data use; this IP gates clinical integration settlement at operational exchange and replay evidence.
- Combined displacement: competitors monetize connectivity, routing, identity, or analytics; this plan makes commercial settlement transparent, PHI-safe, reversible, and policy-bound.

## 10. Cost and Capacity Coupling
- Cost budget reads settlement-intent volume.
- Capacity admission reads pending chargeable work.
- Budget holds do not block emergency bypass.
- Budget holds can pause elective backfill.
- Budget holds can slow low-priority replay.
- Capacity pressure can move settlement to hold if delivery evidence is delayed.
- Replay corrections must adjust cost before final settlement.
- Provider outage holds must prevent billing for undelivered messages.
- Consent denial must not create successful delivery charges.
- Partial delivery must settle only accepted units.
- Tenant dashboards must show actual, held, reversed, disputed, and forecast units.
- Cost budget must show per-pack and per-source-system dimensions.

## 11. Evidence Packets
- Settlement evidence packet includes DealSet id.
- Packet includes tenant id.
- Packet includes provider id.
- Packet includes capability id.
- Packet includes route id.
- Packet includes source system id.
- Packet includes workflow id.
- Packet includes policy decision id.
- Packet includes audit event id.
- Packet includes provenance seal id.
- Packet includes unit count.
- Packet includes size bucket.
- Packet includes residency pack id.
- Packet includes delivery acknowledgement id.
- Packet includes replay adjustment id when relevant.
- Packet includes hold reason when relevant.
- Packet excludes PHI.

## 12. Failure Modes
- Missing audit event creates `held.audit_missing`.
- Missing provider acknowledgement creates `held.delivery_unconfirmed`.
- Consent denial creates `not_chargeable.consent_denied`.
- Emergency deferral creates `deferred.emergency`.
- Duplicate replay creates `reversed.duplicate_replay`.
- Transform correction creates `adjusted.transform_correction`.
- Residency conflict creates `held.residency_conflict`.
- DealSet expiration creates `held.dealset_inactive`.
- Marketplace outage creates `held.marketplace_unavailable`.
- Budget exhaustion creates `held.budget_policy`.
- Capacity throttle creates `held.capacity_backpressure`.
- Any PHI leak into settlement payload is a security incident.

## 13. Rollback
- Disable settlement-intent emission for healthcare-integration.
- Keep clinical operations active unless separately unsafe.
- Hold all non-finalized settlement events.
- Emit reversal events for incorrectly finalized units.
- Rebuild settlement evidence from audit-chain references.
- Reconcile dashboard counts against marketplace acknowledgements.
- Notify tenant finance owners.
- Notify provider network owners.
- Preserve dispute packets.
- Keep DealSet bindings intact unless marketplace owner revokes them.

## 14. Acceptance Evidence
- The IP cites PRD and architecture.
- The IP cites cost-budget and capacity-model.
- The IP cites DealSet provider hold runbook.
- The IP names all chargeable healthcare capabilities.
- The IP states that PHI is excluded from settlement records.
- The IP defines settlement states.
- The IP defines hold and reversal semantics.
- The IP defines emergency deferral semantics.
- The IP defines replay adjustment semantics.
- The IP includes all seven named benchmark families.
- The IP keeps ADR-0314 as settlement authority.
- The IP keeps ADR-0321 referenced but unmodified.

## 15. Done Criteria
- OpenAPI examples include settlement references on chargeable operations.
- AsyncAPI examples include intent, final, hold, reversal, adjustment, and dispute events.
- Dashboards expose held and reversed units.
- Runbook covers provider network hold.
- Cost budget consumes settlement dimensions.
- Capacity admission consumes chargeable work dimensions.
- No PHI appears in settlement examples.
- Replay worker can generate adjustment evidence.
- Emergency bypass can defer settlement.
- No other file is required for this IP deepening pass.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md:22` - - microservices/healthcare-integration/contracts/openapi-v1.yaml; `microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md:23` - - microservices/healthcare-integration/contracts/asyncapi-v1.yaml.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md:33` - - The settlement path must never make emergency care wait for payment rails.; `microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md:34` - - The settlement path must never expose PHI to billing records..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md:14` - - microservices/healthcare-integration/cost-budget.md; `microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md:24` - - microservices/healthcare-integration/dashboards/tenant-cost-and-capacity.json.
