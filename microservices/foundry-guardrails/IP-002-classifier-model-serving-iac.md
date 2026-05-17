---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-002-classifier-model-serving-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, classifier-model-cosign-signed, oya-governance-version-pinning-conformance]
---

# IP-002: ONNX-runtime classifier-model-serving IaC

## Intent

Helm chart for in-cluster ONNX-runtime classifier-serving (4 models per pack: PII/PHI BERT-class, jailbreak classifier, content-safety Llama-Guard-class, AI-slop BERT-small). Cosign-signed model artifacts; per-pack S3 model registry; pod-start integrity verification.

## ChangeSet boundary

One ChangeSet: Helm chart + per-model values + Cosign key references via OpenBao + model registry S3 layout + pod-start integrity-check init container. M01 ships placeholder artifacts (small distilled BERT) per pack; final production models per ADR follow-up.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry-guardrails/iac/helm/classifier-model-serving/Chart.yaml` | create | upstream onnxruntime-server chart pinned |
| `microservices/foundry-guardrails/iac/helm/classifier-model-serving/values.yaml` | create | 4-model spec; replica counts per `capacity-model.md` |
| `microservices/foundry-guardrails/iac/helm/classifier-model-serving/values-pack-kr.yaml` | create | pack-kr overlay |
| `microservices/foundry-guardrails/iac/cosign/keys.yaml` | create | Cosign public-key references (OpenBao-bound) |
| `microservices/foundry-guardrails/iac/cosign/init-verify.sh` | create | pod-start verification script |
| `microservices/foundry-guardrails/iac/model-registry/layout.md` | create | per-pack S3 bucket layout + signed-manifest schema |

## Acceptance Gates

```bash
helm lint microservices/foundry-guardrails/iac/helm/classifier-model-serving
kubectl --dry-run=client apply -k microservices/foundry-guardrails/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate classifier-model-cosign-signed
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- helm-lint + helm-install smoke per chart.
- E2E: kind cluster; deploy 4 placeholder models; verify integrity-check passes; verify pods reach Ready; verify classifier-serving exposes `/v1/classify` endpoint.
- Negative: deploy tampered model; verify integrity-check refuses pod-start + emits `classifier_model_integrity_violation` metric.

## Halt Conditions

- Cosign key not present in OpenBao — block.
- Model artifact size > 1GB — escalate (compute envelope review).
- ONNX runtime version drift from LTS — escalate.

## Next IP

[`IP-003-rule-store-postgres-iac.md`](IP-003-rule-store-postgres-iac.md)

## References

- ADR-0131; `policy/guardrail-enforcement.md`; `capacity-model.md`.
- ONNX Runtime — `onnxruntime.ai`.
- Cosign — `docs.sigstore.dev/cosign/`.
