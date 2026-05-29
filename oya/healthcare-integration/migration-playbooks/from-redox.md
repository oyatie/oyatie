---
doc_class: MigrationPlaybook
microservice: healthcare-integration
source_vendor: Redox
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Migration Playbook — Redox → oyatie healthcare-integration

Audience: a healthcare-tech team currently using Redox as their integration platform who wants to migrate to oyatie's substrate over 10-14 weeks.

Outcome: all source EHRs re-pointed to oyatie's MLLP listeners, all FHIR data mappings preserved, all in-flight integrations cut over, Redox decommissioned.

## Phase 0 — discovery (week 1)

1. Inventory Redox configuration via the Redox Engine portal:
   - Source EHR connections (Cerner, Epic, Meditech, athenahealth, etc).
   - Subscriptions (`/dataModelEvents`).
   - Active data models (PatientAdmin, ClinicalSummary, Order, Results, etc).
   - Transform configurations (Redox-to-tenant + tenant-to-Redox).
   - Destination endpoints (your tenant's webhooks).
   - User accounts + API keys.
2. Inventory commercial exposure:
   - Redox contract end date.
   - Per-connection pricing.
   - Volume tier (typically 100k-1M msgs/month at enterprise).
3. Identify migration priorities:
   - High-volume EHR connections first (where Redox's per-connection cost is highest).
   - Pack-bound integrations (HIPAA-Provider) first.
   - Long-tail one-off integrations last.

Deliverable: `migration-plan.md`.

## Phase 1 — stand up oyatie + smoke test (week 2)

1. Deploy oyatie healthcare-integration IaC into the target cell per `iac/healthcare-integration-paid-helm.yaml`.
2. Sign the BAA via your tenant's `contract-lifecycle-management` µservice if not already.
3. Smoke-test: configure a test channel, send a synthetic ADT message, verify FHIR resource created, audit-chain event emitted.

## Phase 2 — re-establish source EHR connections (weeks 3-6)

For each EHR currently feeding Redox:

1. Coordinate with the EHR vendor for a parallel connection (most EHR vendors support multiple outbound feeds; you'd keep Redox + add oyatie temporarily).
2. Provision the oyatie channel:
   - Portal → Channels → "New channel".
   - MLLP-over-TLS configuration (TLS cert issued via `kms` µservice).
   - Source IP allowlist for the EHR's outbound IP.
   - Subscribe to the same message types Redox is receiving (typically ADT/ORM/ORU/MDM/SIU/DFT).
3. Have the EHR send to BOTH Redox and oyatie for 2-4 weeks. Compare outputs:
   - Same Patient resources created on both sides? Compare via `oya fhir diff --source-a redox-export.json --source-b oyatie-fhir-query.json`.
   - Same number of messages processed?
   - Same DLQ rate?

Typical issues:
- Redox's Patient resources use Redox-internal identifiers; oyatie uses your PIX identifiers. Reconcile.
- Redox parses MSH-12 = 2.5 but the source EHR actually sends 2.5.1 fragments; oyatie's stricter validation may reject. Tune.
- Redox auto-creates missing parent resources (e.g. Practitioner from PV1-7 if not seen before); oyatie's strict mode rejects. Choose your strictness.

## Phase 3 — transform translation (weeks 7-8)

Redox's data-model abstraction maps HL7v2 to JSON via Redox's proprietary "Data Models" (PatientAdmin, ClinicalSummary, Order, Results, etc). Oyatie maps to FHIR R5 directly.

Mapping examples:

| Redox PatientAdmin field | oyatie FHIR field |
|---|---|
| `Patient.Identifiers[].ID` | `Patient.identifier[].value` |
| `Patient.Demographics.Name.First` | `Patient.name[0].given[0]` |
| `Patient.Demographics.DOB` | `Patient.birthDate` |
| `Visit.Location.Department` | `Encounter.location[0].location.display` |
| `Visit.Type` | `Encounter.class.code` (with terminology lookup) |

| Redox ClinicalSummary field | oyatie FHIR field |
|---|---|
| `Encounters[]` | `Encounter` resources |
| `Diagnoses[]` | `Condition` resources |
| `Procedures[]` | `Procedure` resources |
| `Medications[]` | `MedicationStatement` resources |
| `Allergies[]` | `AllergyIntolerance` resources |
| `Vitals[]` | `Observation` resources with `category.coding[0].code = 'vital-signs'` |

Run the converter:

```sh
cargo run -p oya-dev-cli -- healthcare-integration transform-import \
    --source redox \
    --redox-config redox-export.json \
    --output microservices/healthcare-integration/transforms/<channel>.yaml
```

The converter handles ~ 85 % of Redox data-model fields automatically. Manual review per channel for the remaining 15 % (custom extensions, tenant-specific code mappings).

## Phase 4 — destination webhook reconfiguration (week 9)

Redox delivered to tenant webhooks via Redox-defined JSON payloads. oyatie delivers via FHIR Subscription notifications + native FHIR JSON.

For each Redox subscription:
1. Identify the equivalent FHIR resource type(s) — `PatientAdmin.NewPatient` → `Patient` subscription topic; `Order.NewOrder` → `ServiceRequest` subscription topic; `Results.NewReport` → `DiagnosticReport` subscription topic.
2. Create an oyatie FHIR Subscription:
   ```http
   POST /Subscription
   {
     "resourceType": "Subscription",
     "status": "active",
     "topic": "http://oyatie.io/fhir/subscription-topic/servicerequest-created",
     "channel": {
       "type": "rest-hook",
       "endpoint": "https://your-tenant.example/oyatie-webhook"
     }
   }
   ```
3. Update your tenant's webhook handler to parse FHIR JSON instead of Redox JSON. The FHIR shape is more verbose but lossless; expect ~ 2-3× payload size.

## Phase 5 — cutover (weeks 10-11)

1. With both Redox and oyatie running in parallel for 2-4 weeks (Phase 2), all data discrepancies should be resolved.
2. On cutover day: disable the Redox subscription delivery, leave oyatie's active. The source EHRs continue sending to both for a grace period.
3. Monitor for 1 week: verify oyatie continues to process all messages, your webhook handlers receive FHIR payloads, downstream systems behave as expected.
4. After 1 week clean: ask each source EHR to remove the Redox outbound feed. Some EHRs require a change ticket + lead time (Epic: typically 2-4 weeks; Cerner: typically 1-2 weeks).

## Phase 6 — Redox wind-down (weeks 12-14)

1. Cancel Redox subscriptions per the connection.
2. Receive final invoice; pay any minimum-commit residual.
3. Update tenant ARCHITECTURE.md to reference oyatie exclusively.
4. Audit-chain: emit a `redox::migration_complete` event with the count of channels migrated + cutover dates.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| Redox uses HTTPS-delivered HL7 (Redox-specific) instead of MLLP | Configure the source EHR to send MLLP directly to oyatie; some EHRs require additional work to support MLLP if they've been on the HTTPS-relay model |
| Patient identifier reconciliation between Redox internal IDs and your true source-EHR identifiers | Run a 2-week reconciliation phase before cutover; build a mapping table for any legacy references to Redox-internal IDs |
| Redox's "Encounter" model differs from FHIR Encounter (Redox is flatter) | The transform converter handles 90 %; manual review for edge cases like nested encounters or transferred patients |
| Custom Redox fields (Z-segments + custom payload extensions) | Map to FHIR extensions with a registered profile; substrate refuses unknown extensions in strict mode |
| Webhook signature verification (Redox HMAC) | oyatie supports HMAC signatures too; configure via `Subscription.channel.header` array with the HMAC-derived header |
| Order-of-operations issues (Redox guarantees ADT before ORM; do you?) | oyatie's per-tenant queue is FIFO per source-EHR by default; verify your downstream system can handle reordered messages |
| Redox's USCDI v3 mapping vs oyatie's USCDI v4 | Most fields are upward-compatible; review v4 additions (e.g. ServiceRequest extensions for value-based care) |
