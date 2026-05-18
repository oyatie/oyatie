# ADR-SVC-CG-004: Sovereignty — grantor-region Pulsar cluster ownership of projection topics

- Status: Accepted
- Scope: service
- Date: 2026-05-18
- Authority: ADR-0214 §2.5 (sovereignty), data-residency.md, IP-009/010.

## Context

Projection topics could live in: grantor region only, grantee region only, both (geo-replicated), or
a shared neutral region. Each carries sovereignty implications.

## Decision

**Projection topic lives in the grantor's Pulsar cluster — always.** Grantee subscribes cross-region
read-only.

Rationale:
- Grantor is the data owner; sovereignty laws (KR PIPA, EU GDPR cross-border, US HIPAA) treat grantor
  region as the residency anchor.
- Storing the topic in grantee region would constitute a cross-border transfer, requiring per-event
  adequacy decision.
- Grantor-region storage means a single regional outage only affects that grantor's projections, not
  global.
- Pulsar cross-region consumer is mature (built-in TLS, JWT auth, automatic reconnect).

Optional opt-in: agreement may set `geo_replicate_to_grantee_region=true`. When true:
- Pulsar geo-replicates topic to grantee region (one-way; grantee region cannot publish back).
- Per pack overlay rules (KR strict-residency: forbidden; EU + Adequacy: permitted; etc.).
- Replicated topic is read-only in grantee region; grantor-region remains authoritative.

## Alternatives

- Grantee region only (rejected: sovereignty violation; cross-border transfer always).
- Both regions (rejected as default: doubles data + sovereignty review per agreement).
- Neutral shared region (rejected: no such concept under GDPR Art. 44; cloud-vendor-region matters).

## Consequences

- Grantee experiences cross-region latency on subscribe (≤300ms RTT typical).
- Mitigated by opt-in geo-replication for latency-critical agreements.
- Grantor-region outage halts cross-tenant reads for that grantor — by design, fail-closed.

## Verification

- Sovereignty audit job validates topic.region == grantor.region for every active agreement (P0 on
  mismatch).
- Pack overlay rules enforce geo_replicate forbidden cases at acceptance time.
- IP-009 kernel invariant `TopicRegionEqualsGrantorRegion` checked on mint + every emit.
