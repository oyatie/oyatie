---
doc_class: Runbook
title: Classifier-model rollback
microservice: foundry-guardrails
severity: "Sev-1 (cluster-wide outage) / Sev-2 (single-model)"
status: Accepted
owner_team: axis-foundry-guardrails + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-01, FM-02, FM-03, FM-14)
  - microservices/intelligence/incident-response.md
  - microservices/intelligence/iac/helm/classifier-model-serving/values.yaml
doc_status: published
---

# Runbook: Classifier-model rollback

## Trigger

ONE of:

1. **Cosign signature mismatch at pod-start** (FM-14): pod refuses start; integrity violation metric fires.
2. **Recall regression** in production (FM-02 / FM-03): classifier-model A/B in shadow shows worse recall than prior version; OR post-promote enforce reveals regression.
3. **Cluster-wide pool outage** (FM-03): coordinated multi-AZ failure; emergency rollback to prior version.
4. **Sev-1 jailbreak success retraining**: retrained model needs to be deployed via shadow→enforce; this runbook executes the canonical promote/rollback cycle.

## Severity

- Single-model rollback in HA pool: Sev-2.
- Cluster-wide pool outage: Sev-1.
- Cosign integrity violation: Sev-1 (security).

## Pre-checks

1. Confirm the prior known-good model SHA: read `foundry_guardrails_classifier_model_active_version{model_id="<m>",pack="<p>"}` (historical).
2. Confirm prior SHA exists in per-pack S3 model registry + Cosign signature still valid.
3. Confirm rollback target's shadow-vs-enforce delta history was acceptable (when it was the enforce version).

## Steps — single-model rollback

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC; declare Sev-1/2 | ≤ 5 min |
| 2 | Invoke rollback: `oya foundry-guardrails classifier-rollback --model-id <m> --to-sha <prior-sha> --pack <pack> --reason <rfc>` | ≤ 2 min |
| 3 | CLI: <br> a. verifies prior SHA + Cosign signature; <br> b. signs the rollback PATCH with operator Ed25519 (JIT via OpenBao); <br> c. updates Helm values via ArgoCD: `image.tag=<prior-sha>`; <br> d. ArgoCD rolling-restarts pods; <br> e. emits `ClassifierModelDeployed{status:rollback}` event | ≤ 1 min |
| 4 | Verify pod-start: `kubectl -n foundry-guardrails get pods -l app=classifier-model-serving,model=<m>` (Ready) | ≤ 5 min |
| 5 | Verify inference: synthetic test via `oya foundry-guardrails classifier-test --model-id <m>` returns expected | ≤ 1 min |
| 6 | Verify burn-rate recovers; `foundry_guardrails_classifier_request_duration_seconds{model="<m>",quantile="0.99"}` < 80ms | ≤ 5 min |
| 7 | CommsLead: tenant comms if Sev-1 | ≤ 30 min |
| 8 | Postmortem within 5 BD | – |

## Steps — cluster-wide pool outage rollback

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-sre-reliability + axis-foundry-guardrails | ≤ 5 min |
| 2 | If integrity violation: engage ops-security immediately | ≤ 5 min |
| 3 | Determine cause: failed deploy / artifact corruption / signing key issue / OCI outage | ≤ 10 min |
| 4 | If deploy failure: roll back all models to last-good Helm release | ≤ 5 min |
| 5 | If signing key issue: rotate Cosign key via OpenBao + redeploy with new signature | ≤ 15 min |
| 6 | If OCI outage: invoke DR failover (DR-pair packs) per `multi-region.md` §"DR Failover" | ≤ 15 min |
| 7 | If pack lacks DR pair: emergency-bypass entitlement for high-trust tenants (heuristic-only) per `policy/guardrail-enforcement.md` § "Fail-closed posture" | ≤ 5 min |
| 8 | Post-recovery: verify integrity check passes; SLI green ≥ 30 min | ≤ 30 min |

## Rollback (of the rollback)

If the rollback target itself is bad (rare; indicates regression debt):
1. Identify the prior-prior known-good SHA.
2. Repeat steps 2-6 against that SHA.
3. Escalate to ExecSponsor — accumulated classifier debt.

## Shadow→Enforce Promotion (the inverse path)

Promoting a new classifier from shadow to enforce uses this runbook in reverse:
1. Verify shadow phase ≥ 7d (≥ 14d for pack-us-healthcare PHI models).
2. Verify shadow-vs-enforce delta acceptable (< 5% absolute decision change OR signed-off as intentional).
3. Verify `cargo run -p oya-dev-cli -- gate validate shadow-enforce-promotion-readiness --model <m>` exit 0.
4. Update Helm values: `status: enforce`; `image.tag=<new-sha>`.
5. ArgoCD rolling-restart; same verification steps as rollback (4-6).

## Verification

After completion:
- Pods Ready; integrity check passes.
- `foundry_guardrails_classifier_request_duration_seconds{model="<m>",quantile="0.99"} < 80ms` for ≥ 30 min.
- `foundry_guardrails_classifier_model_active_version{model_id="<m>"}` reflects target SHA.
- `ClassifierModelDeployed` event in audit-chain seal log.
- Status page reflects "Resolved" with rollback timestamp (if tenant-facing).

## Post-incident updates

- Postmortem published.
- If FM-02 (recall regression): training-set + shadow-mode-duration policy reviewed.
- If FM-14 (integrity): supply-chain audit; Cosign key state reviewed.

## References

- ADR-0114 (canary observability rollback precedent).
- `microservices/intelligence/failure-modes.md`.
- `microservices/intelligence/policy/guardrail-enforcement.md` §"Shadow→Enforce".
- `microservices/intelligence/iac/helm/classifier-model-serving/values.yaml`.
- Cosign — `docs.sigstore.dev/cosign/`.
