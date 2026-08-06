---
doc_class: Runbook
title: Drift remediation
microservice: cloud-iac
severity: "Sev-3 (single resource) / Sev-2 (cascade > 20 events / 5min) / Sev-1 (security-suspect mutation)"
status: Accepted
owner_team: ops-sre-reliability + axis-cloud-iac
date: 2026-05-17
related_artifacts:
  - iac/failure-modes.md (FM-02, FM-11, FM-15)
  - iac/incident-response.md
doc_status: published
---

# Runbook: Drift remediation

## Trigger

ONE of:
1. **Single drift event**: drift-detector emits `DriftDetected{microservice, resource}` event for a specific µservice resource.
2. **Drift cascade** (FM-02): rate of drift events > 100/min for ≥ 5min.
3. **Drift coverage gap** (FM-11): `oya_cloud_iac_drift_coverage_pct < 99.5` over 1h window.
4. **Render non-determinism** (FM-15): re-render produces different content digest.

## Severity

- Single resource drift, known cause: Sev-3.
- Cascade pattern OR unknown cause: Sev-2.
- Drift on a security-sensitive resource (RBAC, NetworkPolicy, ServiceAccount, secret reference): Sev-1.
- Coverage gap > 1h: Sev-2.

## Pre-checks

1. Identify drifted resource: cloud-native IaC controller/API `drift report` workflow.
2. Compare live state vs git-declared state.
3. Check Kubernetes audit log for the actor that mutated the resource: `kubectl get events --all-namespaces --field-selector involvedObject.name=<resource>`.
4. Check if mutation is recent (< 1h ago) or longstanding.

## Recovery Path A — Authorised operator mutation (legitimate)

Cause: a workload owner / SRE applied a hotfix via kubectl directly.

| Step | Action |
|---|---|
| 1 | Identify the actor + the rationale (often Slack thread + PR successor-IP) |
| 2 | If the hotfix is correct: have the actor open a PR backporting the change into IaC; merge; ArgoCD reconciles back to git-declared state |
| 3 | If the hotfix is wrong: ArgoCD reverts live state to git within reconcile interval (~3min); workload owner notified |
| 4 | Postmortem-lite: document the gap in IaC that necessitated the hotfix |

## Recovery Path B — Unauthorised mutation (drift on security-sensitive resource — Sev-1)

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security; open `#inc-sec-<id>` |
| 2 | Quarantine: pause ArgoCD auto-sync on affected µservice (via `argocd app sync-policy --no-auto-prune <app>`) to preserve evidence |
| 3 | Forensic trace: who/what mutated the resource? Kubernetes audit log + cluster-rbac log |
| 4 | If attacker: standard incident-response per `incident-response.md` §"Severity 1"; engage privacy-governance if data-impactful |
| 5 | After forensics: revert via ArgoCD reconcile OR via signed rollback per `runbooks/rollback-orchestration.md` |
| 6 | Rotate any credentials touched by the mutation; engage cloud-secrets |

## Recovery Path C — Drift cascade (FM-02)

Cause: one mutation cascaded into a wave of drift events (e.g., shared ConfigMap changed).

| Step | Action |
|---|---|
| 1 | Activate drift-event grouping in OnCall: group by (microservice, resource_kind) to surface signal |
| 2 | Identify the root mutation: typically the earliest event in the cascade timeline |
| 3 | Apply throttle: pause drift-detection cycles on affected µservices for ≤ 15min while remediating |
| 4 | Fix the root mutation (revert or accept) |
| 5 | Resume drift-detection; verify cascade subsides |

## Recovery Path D — Drift coverage gap (FM-11)

Cause: validator-worker outage; Postgres lag; apiserver throttling.

| Step | Action |
|---|---|
| 1 | Verify validator-worker pods: `kubectl -n cloud-iac get pods -l app=iac-validator-worker` |
| 2 | If pods are unhealthy: restart; investigate via `runbooks/evaluator-down.md` (cross-µservice pattern) |
| 3 | If apiserver-throttled: throttle other workloads or scale up apiserver |
| 4 | Verify drift cycles resume: `rate(oya_cloud_iac_drift_cycles_completed_total[5m])` > expected |
| 5 | Compute coverage backfill: gap-cycles re-run to ensure no drift went undetected |

## Recovery Path E — Render non-determinism (FM-15)

Cause: a chart references current timestamp, env var, or non-deterministic source.

| Step | Action |
|---|---|
| 1 | Identify non-deterministic value in the chart: cloud-native IaC controller/API `render --chart` workflow shows the diff between renders |
| 2 | Common patterns: `{{ now }}`, `{{ randAlphaNum }}`, env-var interpolation, file-mtime |
| 3 | Replace with deterministic source: PR review; merge |
| 4 | LEAN check `oya-cloud-iac-render-determinism` will catch future regressions |
| 5 | Document the convention violation in `microservices/<ms>/iac/<chart>/README.md` |

## Verification

After recovery:
- `oya_cloud_iac_drift_events_total` rate < 10 / min for affected µservice for ≥ 30min.
- `oya_cloud_iac_drift_coverage_pct >= 99.5` over 1h window.
- Live state matches git-declared state (verified via spot-check cloud-native IaC controller/API `diff --microservice` workflow).
- No security-sensitive resources in drift state.

## Post-incident updates

- Postmortem within 5 business days (Sev-2+).
- If unauthorised mutation: action item for cluster-rbac tightening + audit-log alerting.
- If cascade pattern: tune drift-event grouping + throttle thresholds.
- If non-determinism: file Issue for the µservice's IaC; harden LEAN check coverage.

## References

- `iac/failure-modes.md` FM-02, FM-11, FM-15.
- `iac/incident-response.md`.
- `iac/runbooks/rollback-orchestration.md`.
- ArgoCD drift docs — `argo-cd.readthedocs.io/en/stable/user-guide/auto_sync/`.
