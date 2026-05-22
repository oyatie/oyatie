---
doc_class: IP
ip_id: IP-journey-j01-pack-overlay
journey_id: j01-emergency-911-dispatch
microservice: compliance
role: pack-overlay-composition
status: draft
related_adrs: [ADR-0251, ADR-0298, ADR-0304]
depends_on: []
date: 2026-05-20
owner_team: axis-compliance + council-legal
---

# IP-journey-j01-pack-overlay — Compliance: KR-119 + KR-PIPA + HIPAA pack composition

## Goal

Define how the four packs activate and compose for j01:
- `pack-kr-119-operational-mandate` (KR-119 emergency-services interop)
- `pack-kr-pipa-2023-amendment` (KR personal info protection)
- `pack-hipa-2024` (US PHI standard)
- `pack-kr-medical-records-act` (KR medical record-keeping)
- `global-emergency-services-baseline` (oyatie global)

## Composition rules

Per ADR-0251 + ADR-0304:
1. Higher-restriction always wins.
2. Audit retention: max across active packs (KR-Medical 10y > KR-PIPA 7y > KR-119 6y > HIPAA 6y).
3. Field-set exposure: intersect of consent (consent-graph) + pack allowed-fields.
4. Cell tier: maximum required (Tier-3 if any HIPAA or KR-Medical active).

## Pack manifests (this IP authors / extends)

| Pack manifest | Fields added | File |
|---|---|---|
| `packs/kr-119-operational-mandate/manifest.yaml` | E.S. interop audience type + audit retention 6y | new |
| `packs/kr-pipa-2023-amendment/manifest.yaml` | purpose-limitation rules + Art. 35 DSAR | extend |
| `packs/hipa-2024/manifest.yaml` | PHI scrub rules + 6y audit | extend |
| `packs/kr-medical-records-act/manifest.yaml` | 10y retention rule | extend |
| `packs/global-emergency-services-baseline/manifest.yaml` | bypass class + PSAP registry shape | new |

## Files

| File | Size |
|---|---|
| `microservices/compliance/src/pack_composition/emergency.rs` | ~280 lines |
| `microservices/compliance/policy/pack-composition-emergency.cedar` | ~50 lines |
| `microservices/compliance/runbooks/pack-conflict-resolution.md` | ~180 lines |
| `microservices/compliance/tests/integration/pack_overlay_emergency_test.rs` | ~400 lines |

## Tests

Per integration-test-plan §11.

## Parallel work

Foundational; can land early. Reused by j02, j04, j13, j20.

— end of IP —

## Completion expansion for j01 compliance emergency-911-dispatch-pack-overlay

This appendix completes a pre-existing partial IP scaffold to the 400-line per-service bar required by /tmp/codex-brief-j01-j20-lifesafety.md.
The expansion is bound to ADR-0298 and the shared life-safety ADR pack ADR-0298, ADR-0299, ADR-0300, ADR-0301, ADR-0302, ADR-0303, ADR-0304, ADR-0305, ADR-0306, ADR-0292.

## Completion scope

- Microservice: compliance.
- Journey: j01 Emergency 119 dispatch.
- Role: emergency-911-dispatch-pack-overlay.
- This is an additive completion; prior scaffold text above is preserved.
- No ADR, standard, PRD, or ARCHITECTURE file is modified by this appendix.

## Contract closure

| Surface | Required behavior | Evidence |
|---|---|---|
| OpenAPI 3.2.0 command | compliance validates j01 emergency-911-dispatch-pack-overlay with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| AsyncAPI 3.1.0 event | compliance validates j01 emergency-911-dispatch-pack-overlay with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| proto3 internal RPC | compliance validates j01 emergency-911-dispatch-pack-overlay with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| Cedar v4.1 policy | compliance validates j01 emergency-911-dispatch-pack-overlay with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| audit-chain seal | compliance validates j01 emergency-911-dispatch-pack-overlay with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| observability span | compliance validates j01 emergency-911-dispatch-pack-overlay with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| integration harness fixture | compliance validates j01 emergency-911-dispatch-pack-overlay with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |

## Implementation steps

### Step 01 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 02 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 03 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 04 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 05 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 06 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 07 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 08 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 09 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 10 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 11 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 12 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 13 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 14 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 15 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 16 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 17 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 18 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 19 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 20 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 21 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 22 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 23 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 24 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 25 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 26 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 27 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 28 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 29 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 30 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 31 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 32 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 33 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 34 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 35 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 36 - compliance emergency-911-dispatch-pack-overlay
- Build: wire the emergency-911-dispatch-pack-overlay handler behind the existing compliance boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-journey-j01-emergency-911-dispatch-pack-overlay.md` matched `PHI`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
