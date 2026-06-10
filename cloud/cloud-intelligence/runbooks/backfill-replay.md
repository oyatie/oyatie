# Backfill / Replay Runbook

**Service:** cloud-intelligence  
**Classification:** Stateless service — no gateway-owned persistent state to replay  
**ADR authority:** ADR-0373 (cloud-intelligence gateway production design)

## Stateless declaration

The cloud-intelligence is a **stateless** reverse proxy. It holds no durable tenant data of its own:

- **Pooled provider keys** are ephemeral in-memory cache resolved from owned
  secret-provider/KMS handles at startup and on periodic refresh. Keys are not
  persisted by the gateway; they are re-fetched through the secret-provider port
  on restart or key rotation.
- **Request/response bodies** are never buffered or stored by the gateway. SSE streams are forwarded byte-for-byte through the proxy to the caller; no body is written to disk or any backing store.
- **Key-pool state** (failure counts, blacklist timestamps, cooldown windows) lives only in process memory. A restart resets pool state to all-active, which is the correct recovery posture — keys that were rate-limited upstream will simply trip their failure threshold again on the next cold start if they remain in cooldown.
- **Audit events** (`llm_invocation_audit`, `key_pool_refresh`, `key_blacklisted`, `provider_breaker_open`, `budget_exceeded`) are emitted to the audit-chain service over the mesh. The audit-chain service owns the durable event store and its replay/backfill runbook; the gateway has no local event store to replay from.

## RPO / RTO posture

| Metric | Value | Rationale |
|--------|-------|-----------|
| RPO    | 0 s   | No gateway-owned persistent state; nothing to lose on failure. |
| RTO    | < 60 s | Process restart re-initializes the key pool from the owned secret-provider port. Recovery time is bounded by adapter handle-resolution latency (typically < 5 s) plus Kubernetes liveness probe grace period. |

## Recovery procedure

A gateway pod failure or restart recovers automatically:

1. Kubernetes restarts the pod (liveness probe triggers within configured `failureThreshold × periodSeconds`).
2. On startup the process resolves pooled keys through the owned secret-provider port (`OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN` projected from a k8s Secret).
3. The key pool initializes with all slots active (cold-start; failure history is not persisted).
4. The readiness probe at `/healthz` returns `200 OK` once the pool is loaded; traffic resumes.

No manual backfill, replay, or data restoration step is required.

## Audit-chain replay (delegated)

If audit-chain events were lost during a gateway outage, the recovery path is:

1. Consult the audit-chain microservice runbook (`microservices/audit-chain/runbooks/`).
2. The audit-chain service owns its own Merkle-sealed append-only event store and replay procedures.
3. The gateway cannot retroactively reconstruct `llm_invocation_audit` events for requests that completed during the outage window — this is an accepted gap per ADR-0373 §5 deferred items.

## Related runbooks

- `runbooks/key-exhaustion.md` — handling a fully-blacklisted key pool.
- `runbooks/provider-outage.md` — upstream LLM provider outage response.
