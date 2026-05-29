# consent-graph failure modes catalog

- Owner: axis-consent-graph + sre-axis
- Date: 2026-05-18
- Method: FMEA — Failure Modes + Effects Analysis.

For each failure mode: trigger, detection, mitigation, residual, blast radius, recovery runbook.

## 1. Agreement bounded context

### 1.1 Agreement create on Postgres write failure
- **Trigger**: Postgres unavailable / RLS denial / constraint violation.
- **Detection**: gRPC error; metric `consent_graph_agreement_write_errors_total`.
- **Mitigation**: 3× retry with exp-backoff (50ms/200ms/1s); fail fast on RLS denial.
- **Residual**: client receives 503; idempotent retry safe.
- **Blast radius**: single tenant, single agreement.
- **Recovery**: `consent-graph-restart.md`.

### 1.2 Outbox stuck (Pulsar shipper failing)
- **Trigger**: Pulsar broker outage in grantor region.
- **Detection**: `consent_graph_agreement_outbox_oldest_seconds` > 30.
- **Mitigation**: HPA scales outbox shipper; alerts page on 60s.
- **Residual**: agreement state stale on downstream subscribers; cross-tenant operations involving
  this agreement fail closed.
- **Blast radius**: agreements created during outage in this region.
- **Recovery**: `consent-graph-restart.md` + manual outbox replay.

### 1.3 Agreement state machine illegal transition
- **Trigger**: programming error in usecase code allowing skip-state transition.
- **Detection**: kernel-layer invariant check returns Err; usecase converts to gRPC error.
- **Mitigation**: kernel + property tests + invariants.
- **Residual**: extremely low post-test.
- **Blast radius**: single agreement.

### 1.4 Optimistic concurrency conflict
- **Trigger**: two concurrent updates to same agreement.
- **Detection**: Postgres update returns 0 rows.
- **Mitigation**: caller refetch + retry (handled by SDK).
- **Residual**: rare; bounded by retry count.

### 1.5 Cedar compilation timeout
- **Trigger**: pathological scope/predicate (high field count, deep predicate).
- **Detection**: `cedar_compile_duration_seconds` > 500ms.
- **Mitigation**: 1s hard timeout; agreement transitions to Revoked{PolicyViolation}; client receives
  4xx with diagnostic.
- **Residual**: legitimate-but-complex policies blocked; ADR-SVC-CG-* covers tuning.

### 1.6 Bilateral chain seal lag
- **Trigger**: audit-chain µservice slow.
- **Detection**: `audit_bridge_outbox_oldest_seconds` > 30.
- **Mitigation**: HPA scales audit-bridge-worker; alerts.
- **Residual**: agreement-state-divergence SLO burn.

## 2. Enforcement bounded context

### 2.1 Cache miss + agreement-sdk timeout
- **Trigger**: agreement-app pod down.
- **Detection**: agreement-sdk call timeout (1s).
- **Mitigation**: fail-closed Deny; emit audit event.
- **Residual**: false-denies during outage; clients receive Deny with `Indeterminate`.
- **Blast radius**: all enforcement during agreement-app outage in this region.
- **Recovery**: restart agreement-app; cache warms automatically.

### 2.2 Cedar evaluator panic
- **Trigger**: malformed input + Cedar library bug.
- **Detection**: panic caught by `catch_unwind`; returns Indeterminate.
- **Mitigation**: caught upstream; logged; Deny returned.
- **Residual**: very low.

### 2.3 Cache poisoned by stale policy
- **Trigger**: revocation event missed.
- **Detection**: SLO `revocation-propagation-latency` burn; observability dashboard cross-checks
  cache-age vs revocation log.
- **Mitigation**: revocation worker re-publishes hourly catch-up sweep.
- **Residual**: ≤1s stale window per design.

### 2.4 Cache hit-rate collapse
- **Trigger**: traffic-shift surfaces 1M new agreements/min — compile storm.
- **Detection**: `cache_hit_rate` < 50% for 5min.
- **Mitigation**: pre-warm cache from `consent_graph_compiled_policies`; HPA scales enforcement-app
  pool; rate-limit incoming evaluate.
- **Residual**: bounded latency degradation during storm.

## 3. Revocation bounded context

### 3.1 Pulsar publish failure
- **Trigger**: Pulsar broker outage.
- **Detection**: producer error.
- **Mitigation**: 3× retry; on permanent failure, dead-letter table; alert.
- **Residual**: revocation propagation delayed; downstream subscribers fail closed via
  StaleRevocationCheck.
- **Recovery**: `revocation-incident.md`.

### 3.2 Subscriber pod crash mid-receipt
- **Trigger**: OOM or panic.
- **Detection**: pod restart; Pulsar redelivery.
- **Mitigation**: at-least-once delivery; idempotent invalidation.
- **Residual**: duplicate receipts handled by uniqueness constraint.

### 3.3 Deadline reconciler lag
- **Trigger**: too many in-flight revocations.
- **Detection**: `revocation_pending_deadline_count` > 10K.
- **Mitigation**: HPA scales revocation-app; alert.
- **Residual**: late PartiallyPropagated transitions; alerts fire late.

### 3.4 Cross-region georep paused
- **Trigger**: WAN issue.
- **Detection**: `pulsar_geo_replication_paused`.
- **Mitigation**: alert; deny-by-default in dest region for revocations not yet replicated.
- **Recovery**: `revocation-incident.md`.

## 4. Projection-gateway bounded context

### 4.1 Topic mint fails on Pulsar admin
- **Trigger**: admin API outage / quota exceeded.
- **Detection**: gRPC error to caller.
- **Mitigation**: 3× retry; on failure, agreement remains in Accepted state (not Active); workflow
  resumes when Pulsar admin restored.
- **Residual**: agreement temporarily in Accepted-but-not-Active.

### 4.2 Projection emit fails (Pulsar partial outage)
- **Trigger**: 1 of 16 partitions unavailable.
- **Detection**: emit error on subset of partitions.
- **Mitigation**: Pulsar producer fail-over to surviving partitions; messages queued in shipper
  outbox until full restoration.
- **Residual**: events for affected keys delayed.

### 4.3 Scope-narrowing produces wrong field set (programming bug)
- **Trigger**: bug in narrower.
- **Detection**: kernel invariant `RedactionAppliedConsistentWithScope` check fails pre-emit.
- **Mitigation**: invariant aborts emission; logs P0; agreement auto-suspended.
- **Residual**: very low post-test.

### 4.4 Aggregate bucket below k-anonymity
- **Trigger**: small cohort.
- **Detection**: kernel check pre-emit.
- **Mitigation**: suppress bucket; emit `aggregate_suppressed` audit.
- **Residual**: legitimate small cohort just not visible — that IS the privacy mitigation.

### 4.5 Sovereignty assert fails (region mismatch)
- **Trigger**: misconfigured georep or stale agreement.
- **Detection**: kernel `assert_grantor_region` returns Err.
- **Mitigation**: P0 alert + auto-suspend agreement + sovereignty-violation-zero SLO burn.
- **Recovery**: `regional-sovereignty-violation.md`.

## 5. Audit-bridge bounded context

### 5.1 Partial seal (grantor sealed, grantee not)
- **Trigger**: audit-chain failure on one side.
- **Detection**: `try_join!` returns Err for one side; emission rolls back.
- **Mitigation**: full rollback; retry from queue.
- **Residual**: bounded by retry policy.

### 5.2 Cross-pointer table write failure
- **Trigger**: Postgres outage.
- **Detection**: write error.
- **Mitigation**: retry; on permanent failure, dead-letter table; alert.
- **Residual**: bilateral seal succeeded but cross-pointer missing — caught by reconciliation IP-013.

### 5.3 Reconciliation detects HmacMismatch
- **Trigger**: tampering or wrong pair key version.
- **Detection**: IP-013 hourly reconciliation.
- **Mitigation**: P0 auto-suspend; runbook `audit-chain-divergence-recovery.md`.
- **Residual**: 1h detection window.

## 6. Partner-directory bounded context

### 6.1 Handshake Leg 2 timeout
- **Trigger**: peer's audit-chain query slow / unreachable.
- **Detection**: 30s timeout.
- **Mitigation**: handshake retried up to 3× over 24h; user-visible failure with diagnostic.
- **Residual**: onboarding delay only.

### 6.2 Schema version mismatch detected post-handshake
- **Trigger**: peer upgrades audit-chain incompatibly.
- **Detection**: bilateral emit fails with schema error.
- **Mitigation**: auto-suspend partner; alert; runbook `audit-chain-divergence-recovery.md` covers
  schema-mismatch remediation.

### 6.3 Peer offboards while agreements active
- **Trigger**: peer initiates offboarding.
- **Detection**: offboard event arrives via partner-directory-rest.
- **Mitigation**: cascade revocation of all active agreements with peer; orderly destroy.
- **Recovery**: `partner-offboarding.md`.

## 7. Cross-µservice failure interactions

### 7.1 Identity µservice outage
- **Trigger**: identity-app down.
- **Detection**: principal-resolution timeout.
- **Mitigation**: enforcement fails closed (Deny{InvalidPrincipal}).
- **Residual**: cross-tenant ops halt during outage.

### 7.2 OpenBao outage
- **Trigger**: vault down.
- **Detection**: secret-fetch error.
- **Mitigation**: cached secrets honored with 1h TTL; new agreements fail; existing reads continue.
- **Residual**: limited; OpenBao HA = 3 nodes.

### 7.3 Audit-chain µservice outage
- **Trigger**: audit-chain down.
- **Detection**: SDK timeout.
- **Mitigation**: emission queued to local Pulsar outbox; usecases that inline-await fall back to
  async with `audit_pending=true`.
- **Residual**: audit-chain backlog drains on restoration.

### 7.4 Ontology µservice outage
- **Trigger**: ontology unavailable.
- **Detection**: projection-gateway worker stalls on read.
- **Mitigation**: stall + retry; projection emission paused until restoration.
- **Residual**: freshness SLO burn.

## 8. Failure mode interactions matrix

| Concurrent failures | Worst outcome |
|---------------------|---------------|
| Pulsar + Postgres | full halt in region; cross-tenant ops globally degrade |
| Audit-chain + agreement-app | all writes fail; reads continue via cache; recovery is read-only |
| Cedar evaluator + identity | enforcement defaults to Deny — by design |
| Revocation Pulsar + identity | revocations queue; enforcement fails closed (correct) |

## 9. Dependency-chain diagram

```
        [Browser/Client]
              │
              ▼
        [api-gateway]
              │
              ▼
  ┌───[consent-graph-app]───┐
  │           │             │
  ▼           ▼             ▼
[Postgres] [Pulsar]   [enforcement-app] ──► [agreement-sdk] ──► (loop)
                            │                      │
                            ▼                      ▼
                       [OpenBao]              [identity]
                            │
                            ▼
                       [audit-chain] ◄────── (bilateral seals)
                            │
                            ▼
                       [observability]
```

Failure in any leaf cascades up; SLOs at each level track health independently.

## 10. Verification

- Chaos schedule: monthly each-component kill drill in staging.
- Annual full-stack chaos game-day in production canary region.
- All failure modes have runbook + automated detection.
