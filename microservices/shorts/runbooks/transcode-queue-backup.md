---
doc_class: Runbook
title: Transcode queue backup
microservice: shorts
severity: "Sev-2 (degradation) / Sev-1 (sustained > 30 min)"
status: Accepted
owner_team: ops-sre-reliability + axis-shorts
date: 2026-05-17
related_artifacts:
  - microservices/shorts/failure-modes.md (FM-04, FM-05)
  - microservices/shorts/capacity-model.md (ffmpeg worker pool)
  - microservices/shorts/threat-model.md (T-D-02, T-E-05)
doc_status: published
---

# Runbook: Transcode queue backup (FM-04 + FM-05)

## Trigger

- `oya_shorts_transcode_queue_depth` > 1000 sustained 5min.
- `oya_shorts_transcode_worker_error_rate` > 1 % sustained 10min (potentially indicates ffmpeg CVE).
- Tenant-reported: upload→playable latency degraded.
- KEDA autoscaler hit cap (`max workers`) without queue drain.

## Severity

Sev-2 default; escalate to Sev-1 if:
- Sustained > 30 min affecting > 30 % of tenants.
- ffmpeg worker error rate suggests CVE / RCE (engage ops-security).
- Mass-upload abuse attack pattern detected.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify worker pool status: `kubectl -n shorts get pods -l app=shorts-video-transcode-worker` (replicas Running) | ≤ 2 min |
| 2 | Inspect queue depth by tenant: `topk(10, oya_shorts_transcode_queue_depth) by (tenant_id)` | ≤ 3 min |
| 3 | Check for abuse pattern: any single tenant > 80% of queue → rate-limit that tenant | ≤ 5 min |
| 4 | KEDA autoscale check: ensure max-replicas not hit; if hit, raise via `kubectl scale` (temporary) | ≤ 5 min |
| 5 | Priority lane: pause Free-tier transcode; preserve Premium-tier throughput | ≤ 5 min |
| 6 | Verify ffmpeg version pin: `cargo run -p oya-dev-cli -- gate validate version-pinning-conformance` | ≤ 5 min |
| 7 | If ffmpeg CVE suspected: cordon affected worker pods; engage ops-security | ≤ 10 min |

## ffmpeg CVE Path (T-E-05)

If worker error rate spike correlates with recent ffmpeg LTS-pin bump:

1. Cordon affected worker pool via `kubectl drain` on each worker node.
2. Engage ops-security; pull ffmpeg SBOM via Trivy + Grype.
3. Roll back to last-known-good ffmpeg LTS pin via Helm rollback.
4. Verify gVisor sandbox containment held (no host pivot).
5. If sandbox breached: declare Sev-1; full cluster forensic review.
6. Postmortem with cloud-k8s + ops-security.

## Full Queue Drain (Sev-1 path)

If sustained > 30 min and KEDA cap hit:

| Step | Action | Time |
|---|---|---|
| 1 | Engage axis-shorts senior on-call + ops-sre-reliability lead | ≤ 5 min |
| 2 | Communicate to affected tenants via status page (per `incident-response.md`) | ≤ 10 min |
| 3 | Provision additional worker capacity via Terraform-driven node pool expansion | ≤ 30 min |
| 4 | If sustained celebrity-event traffic: switch to GPU transcode path (OCI BM.GPU.A10) for high-priority tier | ≤ 15 min |
| 5 | Verify queue drain rate; revise ETA per tenant comms | continuous |
| 6 | Once drained, scale back; resume normal autoscale | post-resolution |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Celebrity creator viral-event mass-upload | single creator dominates queue | rate-limit per creator; accept legitimate; expand cap temporarily |
| Mass-upload abuse | spike from many tenants concurrently | per-tenant rate-limit; engage gtm-customer-success |
| ffmpeg CVE → worker crash loop | error rate elevated; pod restarts | cordon + roll back ffmpeg pin; engage ops-security |
| KEDA autoscaler config drift | autoscaler not scaling | check ScaledObject CR; verify queue-depth source |
| gVisor sandbox overhead | worker latency elevated | check gVisor metrics; possibly switch to Kata for hot tier |
| Hardware exhaustion in pool | CPU saturation across all workers | provision additional nodes |

## Recovery Verification

- `oya_shorts_transcode_queue_depth` < 100 for ≥ 30 min.
- `oya_shorts_transcode_duration_seconds` p95 within budget (≤ 30s for 60s video).
- No active alerts on transcode path.
- Worker error rate < 0.1 % for ≥ 1h.

## Postmortem Triggers

- If ffmpeg CVE confirmed: emergency LTS-pin update + cluster-wide rollout + ops-security review.
- If celebrity-event sustained pattern: capacity-model.md revision; GPU-pool consideration.
- If KEDA cap hit: capacity-model.md max-replicas revision.

## References

- `microservices/shorts/failure-modes.md` FM-04, FM-05.
- `microservices/shorts/capacity-model.md` §"ffmpeg Transcode Worker Pool".
- `microservices/shorts/threat-model.md` T-D-02, T-E-05.
- KEDA docs.
- gVisor / Kata Container sandbox docs.
- ffmpeg LTS pin tracking.
- Trivy + Grype CVE scanners.
