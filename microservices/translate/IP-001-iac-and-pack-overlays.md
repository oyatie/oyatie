---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-001-iac-and-pack-overlays
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-translate + ops-iac
acceptance_lanes: [iac-helm-lint, iac-kustomize-build, iac-terraform-validate, cargo-deny, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC + per-pack overlays

## Intent

Stand up the Helm chart, kustomize base, and per-pack overlays (`pack-kr`, `pack-eu`, `pack-jp`, `pack-cn-stub`) for the `translate` µservice. Wire Postgres + Valkey + Meilisearch + S3. Configure Istio mTLS + SPIFFE per `cell`. Bind to per-pack OpenBao endpoint. Emit no live workload — substrate only, ready for IP-002+ to land code.

## ChangeSet boundary

Creates `iac/` subtree only. No source code in this IP.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/translate-router/Chart.yaml` | create |
| `iac/helm/translate-router/values.yaml` | create — replicas + resources + adapters + observability + recording rules + event topics |
| `iac/helm/translate-router/templates/deployment-rest.yaml` | create |
| `iac/helm/translate-router/templates/deployment-worker.yaml` | create |
| `iac/helm/translate-router/templates/deployment-app.yaml` | create |
| `iac/helm/translate-router/templates/deployment-adapter-*.yaml` | create (foundry-runtime + anthropic + openai + google + deepl) |
| `iac/helm/translate-router/templates/deployment-doc-worker.yaml` | create — gVisor RuntimeClass |
| `iac/helm/translate-router/templates/deployment-bulk-worker.yaml` | create |
| `iac/helm/translate-router/templates/deployment-stream-router.yaml` | create |
| `iac/helm/translate-router/templates/service-*.yaml` | create |
| `iac/helm/translate-router/templates/hpa-*.yaml` | create |
| `iac/helm/translate-router/templates/pdb-*.yaml` | create |
| `iac/helm/translate-router/templates/istio-virtualservice.yaml` | create — mTLS STRICT |
| `iac/helm/postgres/values.yaml` | create — per-pack Postgres 16 HA |
| `iac/helm/redis/values.yaml` | create — Valkey 8.1 (Redis wire-compat) sentinel HA |
| `iac/helm/meilisearch/values.yaml` | create — Meilisearch 0.10.0 LTS |
| `iac/kustomize/base/kustomization.yaml` | create — namespace `oya-translate`; commonLabels |
| `iac/kustomize/overlays/pack-kr/kustomization.yaml` | create — namespace `oya-translate-kr`; engine whitelist patch (in-house + Anthropic + Google + DeepL conditional) |
| `iac/kustomize/overlays/pack-eu/kustomization.yaml` | create — engine whitelist patch (in-house + Anthropic + OpenAI + Google + DeepL) |
| `iac/kustomize/overlays/pack-jp/kustomization.yaml` | create — engine whitelist patch (in-house + Anthropic + Google + DeepL) |
| `iac/kustomize/overlays/pack-cn-stub/kustomization.yaml` | create — engine whitelist patch (in-house ONLY; ALL external vendors forbidden) |
| `iac/terraform/oci-resources.tf` | create — per-pack OCI resources (Object Storage buckets + KMS keys) |

## Helm values headline

Per `microservices/translate/capacity-model.md`:

```yaml
image:
  repository: registry.oyatie.dev/translate/router
  tag: ""              # set per-deploy from release-pointer ref

router-rest:
  replicaCount: 4
  resources:
    requests: {cpu: 1000m, memory: 2Gi}
    limits: {cpu: 2000m, memory: 4Gi}

adapters:
  foundry-runtime:    {replicaCount: 4, fragile: false}
  anthropic:          {replicaCount: 2, enabled: true,  fragile: false}
  openai:             {replicaCount: 2, enabled: true,  fragile: false}
  google-translate:   {replicaCount: 2, enabled: true,  fragile: false}
  deepl:              {replicaCount: 2, enabled: true,  fragile: false}

doc-translate-worker:
  replicaCount: 4
  runtimeClassName: gvisor   # ADR-TRANSLATE-0005

residency:
  pack: ""             # set per-deploy overlay
  engineWhitelist: []  # set per-deploy overlay

eventEmission:
  nats:
    endpoint: "oya-foundry-evidence-nats.<pack>.svc.cluster.local:4222"
    subjects:
      translationCompleted: "oya.translate.translation.completed"
      engineRouted:         "oya.translate.engine.routed"
      tmUpdated:            "oya.translate.tm.updated"
      termbaseUpdated:      "oya.translate.termbase.updated"
      qualityEstimated:     "oya.translate.qe.estimated"
      bulkJobStarted:       "oya.translate.bulk.started"
      bulkJobCompleted:     "oya.translate.bulk.completed"
      bulkJobFailed:        "oya.translate.bulk.failed"
      euAiActDisclosure:    "oya.translate.eu-ai-act.disclosure"
      languageDetected:     "oya.translate.langdetect.detected"
      documentTranslated:   "oya.translate.doc.translated"
      streamSessionStarted: "oya.translate.stream.session-started"
      streamSessionEnded:   "oya.translate.stream.session-ended"

recordingRules:
  - name: oya:translate:engine_availability:rolling_15m
    expr: avg_over_time(oya_translate_engine_request_success_ratio[15m])
  - name: oya:translate:translation_request_p99_ms:rolling_5m
    expr: histogram_quantile(0.99, sum by (le) (rate(oya_translate_translation_request_latency_ms_bucket[5m])))
  - name: oya:translate:tm_leverage_p99_ms:rolling_5m
    expr: histogram_quantile(0.99, sum by (le) (rate(oya_translate_tm_leverage_latency_ms_bucket[5m])))
  - name: oya:translate:residency_violation_total
    expr: sum(oya_translate_residency_violation_total)
```

## Pack-cn-stub specific guardrails

The overlay sets `residency.engineWhitelist = ["in-house"]`. The `adapter-anthropic / openai / google / deepl` deployments are scaled to `replicaCount: 0`. The OPA gatekeeper policy `policy/cn-stub-external-vendor-forbid.rego` blocks any helm-render that enables external adapter replicas in the cn-stub namespace.

## Acceptance Gates

```bash
helm lint iac/helm/translate-router/
helm template iac/helm/translate-router/ --values iac/kustomize/overlays/pack-kr/values.yaml > /tmp/translate-kr.yaml
kustomize build iac/kustomize/overlays/pack-kr
kustomize build iac/kustomize/overlays/pack-eu
kustomize build iac/kustomize/overlays/pack-jp
kustomize build iac/kustomize/overlays/pack-cn-stub
tofu validate iac/terraform/
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice translate
```

## Test Plan

| Test | Verifies |
|---|---|
| `tests/iac/helm_lint` | Helm chart lints clean |
| `tests/iac/kustomize_build_per_pack` | Each pack overlay builds to a valid k8s manifest |
| `tests/iac/cn_stub_excludes_external_vendors` | `kustomize build overlays/pack-cn-stub` contains no `adapter-anthropic/openai/google/deepl` non-zero replicas |
| `tests/iac/recording_rules_load` | Per-pack recording rules load into Mimir test instance |

## Halt Conditions

- Helm chart fails lint.
- Any per-pack overlay produces a non-residency-compliant manifest.
- Pack-cn-stub overlay accidentally enables external vendor adapter.

## Next IP

[`IP-002-translate-router-kernel.md`](IP-002-translate-router-kernel.md)
