---
doc_class: Runbook
title: Cedar engine restart + bundle reload
microservice: foundry-guardrails
severity: "Sev-2 (cedar timeout / config drift)"
status: Accepted
owner_team: axis-foundry-guardrails
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-04, FM-05)
  - microservices/intelligence/iac/helm/cedar-engine/values.yaml
doc_status: published
---

# Runbook: Cedar engine restart + bundle reload

## Trigger

ONE of:

1. Cedar evaluation timeout >0 (FM-04): bundle bug / oversized input loop.
2. Cedar bundle integrity violation: SHA mismatch at hot-reload.
3. Cedar default-deny drift (FM-05): cross-references `policy-rule-rollback.md`.
4. Post-rollback verification: confirming engine re-loaded the prior bundle.

## Severity

- Single-pod timeout: Sev-3.
- Cluster-wide timeout: Sev-2.
- Integrity violation / default-deny drift: Sev-1.

## Pre-checks

1. Read current bundle SHA: `kubectl -n foundry-guardrails get configmap cedar-bundle-<pack> -o jsonpath='{.metadata.annotations.bundle-sha}'`.
2. Compare to expected SHA from git: `bash microservices/intelligence/iac/cedar/build.sh microservices/intelligence/policy --check-sha`.
3. Identify drift / mismatch source.

## Steps — restart cedar-engine pods (Sev-3 / Sev-2)

| Step | Action | Time |
|---|---|---|
| 1 | Verify HPA scaling state: `kubectl -n foundry-guardrails get hpa cedar-engine` | ≤ 2 min |
| 2 | Rolling restart: `kubectl -n foundry-guardrails rollout restart deployment/cedar-engine` | ≤ 5 min |
| 3 | Verify all pods Ready: `kubectl -n foundry-guardrails get pods -l app=cedar-engine` | ≤ 5 min |
| 4 | Verify bundle loaded: `oya foundry-guardrails cedar-bundle-show --pack <p>` matches expected SHA | ≤ 2 min |
| 5 | Verify evaluation latency: `foundry_guardrails_cedar_evaluation_duration_seconds{quantile="0.99"} < 10ms` | ≤ 5 min |

## Steps — bundle reload (live config update)

| Step | Action | Time |
|---|---|---|
| 1 | Compile bundle: `bash microservices/intelligence/iac/cedar/build.sh microservices/intelligence/policy` | ≤ 30s |
| 2 | Verify default-deny present: `oya gate validate cedar-default-deny-enforced --bundle <path>` | ≤ 5s |
| 3 | Apply via ArgoCD: `argocd app sync foundry-guardrails-cedar-<pack>` | ≤ 30s |
| 4 | Sidecar hot-reload: triggered by ConfigMap change; ≤ 30s per `cedar-engine` `refreshInterval` | ≤ 30s |
| 5 | Verify SHA: `oya foundry-guardrails cedar-bundle-show --pack <p>` | ≤ 2 min |

## Steps — default-deny drift (Sev-1; cross-ref policy-rule-rollback.md)

| Step | Action | Time |
|---|---|---|
| 1 | Engage ops-security; declare Sev-1; open `#inc-sec-<id>` | immediate |
| 2 | ArgoCD auto-rollback if pre-deploy detection | ≤ 5 min |
| 3 | If live-cluster mutation: see `policy-rule-rollback.md` §"Cedar bundle rollback" | per that runbook |

## Verification

- Cedar engine pods Ready.
- Bundle SHA matches expected.
- Evaluation p99 < 10ms.
- Default-deny enforced (LEAN exit 0).
- Audit-chain emit `CedarEngineRestarted` or `CedarBundleReloaded`.

## Post-incident updates

- Postmortem if Sev-2+.
- If FM-04 (timeout): review bundle complexity; consider fragment pre-compilation.
- If FM-05 (drift): control-plane access audit.

## References

- `microservices/intelligence/failure-modes.md` FM-04 + FM-05.
- `microservices/intelligence/runbooks/policy-rule-rollback.md`.
- `microservices/intelligence/iac/helm/cedar-engine/values.yaml`.
- Cedar docs — `docs.cedarpolicy.com`.
