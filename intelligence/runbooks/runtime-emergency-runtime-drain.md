---
doc_class: Runbook
title: Emergency runtime drain (controlled pod retirement + cluster-wide drain)
microservice: foundry-runtime
severity: "Sev-2 typically (planned); Sev-1 if drain failure (FM-11)"
status: Accepted
owner_team: ops-sre-reliability + axis-foundry-runtime
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-11)
  - microservices/intelligence/incident-response.md
  - microservices/intelligence/policy/runtime-isolation.md (TI-08 autonomy)
doc_status: published
---

# Runbook: Emergency runtime drain

## Purpose

Retire runtime pods safely while in-flight invocations complete or park (no data loss). Drain is invoked operationally (scheduled maintenance), reactively (provider compromise + credential rotation), or in response to autonomy-violation incidents (per `autonomy-violation-quarantine.md`).

## Trigger

ONE of:
- Scheduled maintenance: node upgrade, Helm chart rollout.
- Provider credential rotation (FM-07 recovery): drain pods bound to old credential generation.
- Autonomy-violation incident: drain affected pods OR quarantine specific principal's invocations.
- Image vulnerability response: drain pods running affected image.
- Cluster-wide drain (rare): pack-wide kill-switch in extreme breach-response scenarios.

## Drain primitive (per pod)

The DrainController port (per PRD §"BC layer mapping") implements the canonical drain procedure:

| Phase | Step | Time |
|---|---|---|
| 1 | Set pod readiness gate to "not ready"; HPA stops sending new invocations | ≤1s |
| 2 | Emit `InvocationStarted{drain_pending=true}` for new attempts → caller routes elsewhere | ≤1s |
| 3 | Wait for in-flight invocations to complete naturally; max wait = longest in-flight budget (per `failure-modes.md` FM-15 cap: 300s) | up to 60s budget; 300s ceiling |
| 4 | For invocations still in-flight at the 60s budget: emit `InvocationCancelled{reason=runtime_drain}` and park session-state durably | ≤5s per invocation |
| 5 | Verify zero in-flight on this pod: `oya_foundry_runtime_pod_inflight_invocations{pod=<id>} == 0` | ≤2min |
| 6 | Pod terminates gracefully (SIGTERM → app exits → SIGKILL fallback after 90s grace) | ≤90s |

Total per-pod drain: ≤60s typical; ≤90s max grace.

## Triggering a per-pod drain (operational)

```bash
cargo run -p oya-dev-cli -- foundry-runtime drain \
  --pod <pod-name> \
  --reason "<rfc>"
```

Audit-chain seal: every drain emits `RuntimePodDrained{pod, reason, in_flight_at_start, parked_count, drained_at}`.

## Cluster-wide drain (extreme; 2-person rule required)

```bash
cargo run -p oya-dev-cli -- foundry-runtime drain-cluster \
  --pack <pack> \
  --reason "<rfc>" \
  --approver <second-operator>
```

2-person rule + OpenBao JIT elevation + ExecSponsor approval. Cluster-wide drain takes ≥10min; during drain, new invocations 503 until pool re-warms.

## Drain failure (FM-11)

If a pod terminates before drain completes (e.g., SIGKILL after grace period elapsed):

| Step | Action | Time |
|---|---|---|
| 1 | Identify lost invocations: `oya_foundry_runtime_invocations_lost_in_drain_total` | ≤2min |
| 2 | Emit `InvocationFailed{reason=runtime_drain_lost}` for each lost invocation; tenant retries | – |
| 3 | Investigate cause: pod grace period too short? slow-running invocation > 90s? | ≤30min |
| 4 | If grace period too short: update Helm values; rollout new pod template | ≤15min |
| 5 | If long-invocation pattern: tighten capability descriptor timeouts | ≤1h |

## Post-drain verification

After drain (or cluster-wide drain):
- Affected pods terminated; `kubectl get pods` shows them gone.
- HPA replaces them; new pods Ready ≥5min before declaring complete.
- `oya_foundry_runtime_invocations_lost_in_drain_total == 0` for the drain window (target; rare exceptions per FM-11).
- Audit-chain entries for `RuntimePodDrained` per pod.
- For credential-rotation drain: verify in-flight invocations using new credential generation succeed.

## Verification

```bash
# Validate drain primitive idempotency + safety
cargo nextest run -p oya-intelligence-runtime-runtime-pool-worker --test drain_parks_in_flight
cargo nextest run -p oya-intelligence-runtime-runtime-pool-worker --test drain_failure_emits_invocation_failed
```

## Post-incident updates

- For repeated drain failures: revisit pod grace period; consider capability-side timeout reduction.
- For cluster-wide drain: postmortem MANDATORY; ExecSponsor + council-architecture review of the trigger criterion.

## References

- `microservices/intelligence/failure-modes.md` FM-11.
- `microservices/intelligence/incident-response.md`.
- `microservices/intelligence/policy/runtime-isolation.md` TI-08.
- ADR-0020 (cold-start budget); ADR-0022 (autonomy tiers).
- Kubernetes graceful pod termination — `kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-termination`.
