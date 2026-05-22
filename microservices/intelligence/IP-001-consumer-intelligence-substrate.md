---
doc_kind: implementation-plan
id: IP-001
title: Consumer intelligence substrate scaffold
status: Accepted
owner_team: axis-intelligence
related_adrs: [ADR-0136, ADR-0215, ADR-0219, ADR-0220]
---

# IP-001: Consumer intelligence substrate scaffold

## Intent

Create the first repo-native surface for `microservices/intelligence/`, the canonical AI substrate µservice per ADR-0255 KS#14 two-layer intelligence doctrine. Per ADR-0335 (Wave 15I), intelligence absorbs the retired foundry µservice; the "Hermes" pipeline brand is RETIRED corpus-wide per ADR-0247 D-10 + ADR-0328 D-9.22 + ADR-0335 D-26..D-36.

## Scope

- `manifest.json` declares the µservice, bounded contexts, capabilities, contracts, SLOs, audit events, and mesh posture.
- REST, AsyncAPI, and proto contracts expose assist-draft and context-aware retrieval boundaries.
- Capability records bind autonomy tier, data classes, Cedar policy fragments, eval sets, and audit topics.
- Tenant policy refuses calls without active context, consent, and budget.

## Acceptance

- The µservice path is `microservices/intelligence/`, not `microservices/oyatie-intelligence/`.
- Every call carries `principal_id`, `context_id`, and `consent_grant_id`.
- Intelligence is the canonical AI substrate; foundry is RETIRED per ADR-0335 (Wave 15I) and its responsibilities are absorbed into intelligence. Intelligence consumes approved model/tool adapters through explicit seams.
- AI draft output is advisory and must be importable into deterministic builders instead of directly mutating tenant configuration.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-001-consumer-intelligence-substrate.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.
