# oya-connector-epic-fhir-adapter

Epic FHIR R4 + USCDI connector.

## Coverage

* `Patient` — demographics
* `Encounter` — visits
* `MedicationRequest` — orders
* `Observation` — labs / vitals

HL7v2 fallback (for legacy integrations that lack FHIR) is tracked as a
follow-up `oya-connector-hl7v2-adapter`.

## Auth

SMART-on-FHIR backend services profile — OAuth 2.0 with signed JWT
client assertion. SecretReference resolves to
`sref://<tenant>/epic-fhir/jwt` (RS384 signing key).

## Sandbox

Epic on FHIR sandbox: fhir.epic.com/Developer/Apps — `R4` endpoint with
synthetic patient data (Camila Lopez, Derrick Lin, etc.).

## Rate limits

Tenant-configurable. Conservative default: 5 req/sec; no published
daily quota.

## Capabilities

`list`/`get`/`create`/`update` supported.
`delete` UNSUPPORTED — Epic forbids hard-delete of clinical resources.
`subscribe` UNSUPPORTED in v1 — FHIR Subscription requires tenant-side
configuration.

## OpenAPI snapshot

See `specs/openapi.snapshot.yaml`.

## Cedar policy

See `specs/cedar-policy.cedar`.

## Compliance note

PHI passes through the audit-chain seal (ADR-0145). Tenants must
configure their FHIR endpoint base URL via the `Connect` µservice; this
adapter does not bake an endpoint in.
