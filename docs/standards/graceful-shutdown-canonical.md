---
doc_class: Standard
title: Graceful Shutdown (Canonical)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-18
owner_team: ops-sre-reliability
deciders: ops-sre-reliability, council-architecture
related_adrs: [ADR-0145]
review_cadence: annually
doc_status: published
---

# Graceful Shutdown (Canonical)

## Authority

Every Kubernetes Deployment in every oyatie µservice MUST implement
the graceful-shutdown contract below, matching AWS/Google/Microsoft
hyperscaler practice for in-flight request draining.

## Contract

### 1. terminationGracePeriodSeconds

Every Deployment + StatefulSet pod template MUST declare:

```yaml
spec:
  template:
    spec:
      terminationGracePeriodSeconds: 30   # default
      # 60 for workers with long-running batches
      # 120 for stateful (postgres, redis) cells
```

Implemented via the canonical helper:

```yaml
{{- include "oya.gracefulShutdown" $ | nindent 6 }}
```

### 2. SIGTERM handler discipline

Every µservice MUST install a SIGTERM handler that:

1. Marks the readiness probe failing (causes service-mesh to stop
   sending new requests).
2. Drains in-flight HTTP/gRPC connections (no abort; await
   completion or `terminationGracePeriodSeconds - 5` seconds).
3. Releases distributed locks held in Redis/etcd/postgres advisory
   locks.
4. Flushes pending metrics + traces to the OTel collector.
5. Closes the database connection pool.
6. Exits with code 0.

### 3. preStop hook

Each Deployment MUST set the preStop hook to give the load-balancer
time to detect readiness failure before SIGTERM lands:

```yaml
lifecycle:
  preStop:
    exec:
      command: ["/bin/sh", "-c", "sleep 5"]
```

Provided by the canonical helper `oya.preStopHook.gracefulDelay`.

### 4. PodDisruptionBudget interaction

`PodDisruptionBudget.minAvailable` MUST be set on every Deployment
so node drains cannot all-at-once delete the µservice. The canonical
PDB helper enforces `minAvailable: 1` or `maxUnavailable: 50%`
whichever applies.

### 5. Validation

The canonical helper `oya.gracefulShutdown` is enforced by
`check-statelessness` (existing) — non-stateless components are
flagged. New µservices MUST use the helper.

## References

- Kubernetes docs — Pod Lifecycle / Termination of Pods.
- AWS — Graceful container shutdown best practices.
- Google SRE Workbook — connection draining.
- ADR-0145-inter-microservice-communication-reform.
