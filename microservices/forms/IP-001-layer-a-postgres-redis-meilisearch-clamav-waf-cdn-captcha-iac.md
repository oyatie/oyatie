---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-001-layer-a-postgres-redis-meilisearch-clamav-waf-cdn-captcha-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-forms + cloud-iac
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance, oya-forms-recaptcha-forbidden-pack-eu-kr-us-hc]
---

# IP-001: Layer-A IaC — Postgres (Citus) + Redis + Meilisearch + ClamAV + WAF + CDN + Captcha sidecar

## Intent

Author Helm + Kustomize manifests for the forms Layer-A substrate: OCI CDN (per-pack edge for form-renderer WASM + design-system primitives), OCI WAF (rate-limit + bot-detection + CSP enforcement), Postgres + Citus 12.x (form-definition + response-store with tenant_id shard key + column-level envelope encryption per ADR-FORMS-0003), Redis 7.2 (rate-limit + session + WAF cache), Meilisearch 0.10.0 (response search; PII-redacted index per AC-09), ClamAV 1.3 + OPSWAT MetaDefender (file-upload scan), Captcha sidecar (hCaptcha primary; Cloudflare Turnstile fallback; Friendly Captcha tertiary per ADR-FORMS-0002).

## ChangeSet boundary

One cohesive ChangeSet: 12 Helm chart bundles + shared Kustomize base + 11 per-pack overlays + 2 Terraform files (CDN edge config + node-library publishers). No Rust code; pure IaC. Per-pack secret references via OpenBao SecretReference.

## Concrete File Targets

See `iac/helm/<chart>/` directory tree under `microservices/forms/iac/`.

## Code Shape

Helm chart skeleton excerpt:

```yaml
citus:
  image:
    tag: "12.1.0"  # LTS pin
  primary:
    replicas: 1
    resources:
      requests: {cpu: 2, memory: 8Gi}
      limits: {cpu: 4, memory: 16Gi}
  worker:
    replicas: 3
  rls:
    enabled: true
  shardKey: tenant_id
  config:
    citus.shard_count: 32
    citus.replication_factor: 2
  columnEncryption:
    enabled: true
    kmsRoot: "openbao:secret/forms/<pack>/kek"
    kekRotationCadence: "quarterly"
    dekRotationCadence: "quarterly"
    cipher: "aes-256-gcm"
    aadFields: [tenant_id, form_id, response_id, field_id, data_class]
```

## Acceptance Gates

```bash
helm lint microservices/forms/iac/helm/response-store-postgres
helm lint microservices/forms/iac/helm/response-cache-redis
helm lint microservices/forms/iac/helm/response-search-meilisearch
helm lint microservices/forms/iac/helm/upload-scan-clamav
helm lint microservices/forms/iac/helm/captcha-sidecar
helm lint microservices/forms/iac/helm/form-cdn
helm lint microservices/forms/iac/helm/form-waf
helm lint microservices/forms/iac/helm/form-rest
helm lint microservices/forms/iac/helm/form-builder-wasm
helm lint microservices/forms/iac/helm/response-collector-rest
helm lint microservices/forms/iac/helm/export-worker
helm lint microservices/forms/iac/helm/bulk-distribute-worker
kubectl --dry-run=client apply -k microservices/forms/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice forms
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
cargo run -p oya-dev-cli -- gate validate forms-recaptcha-forbidden-pack-eu-kr-us-hc
```

## Test Plan

- Per Phase-01 §"Per-IP Test Coverage Threshold" IaC class: ≥ 1 helm-install + helm-test smoke per chart.
- E2E: kind cluster; apply pack-kr overlay; verify all 12 component pods reach `Ready` within 10 min.

## Halt Conditions

- Any chart upstream-version drift from the LTS pin — escalate.
- OpenBao secret-reference resolution failure — block.
- Citus RLS migration fails — block.
- reCAPTCHA accidentally configured in pack-eu/kr/us-hc — block.

## Next IP

[`IP-002-form-field-section-response-domain-kernel.md`](IP-002-form-field-section-response-domain-kernel.md)

## References

- ADR-0131 per-microservice flat layout.
- ADR-FORMS-0001 form-definition schema.
- ADR-FORMS-0002 captcha selection.
- ADR-FORMS-0003 PII column encryption.
- `multi-region.md`, `capacity-model.md`, `threat-model.md`.
- Citus docs, Redis Sentinel docs, Meilisearch docs, ClamAV docs, hCaptcha docs, Cloudflare Turnstile docs.
