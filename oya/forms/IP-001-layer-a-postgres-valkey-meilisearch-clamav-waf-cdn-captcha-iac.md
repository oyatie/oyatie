---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-001-layer-a-postgres-valkey-meilisearch-clamav-waf-cdn-captcha-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-forms + cloud-iac
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance, oya-forms-recaptcha-forbidden-pack-eu-kr-us-hc]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Layer-A IaC — Postgres (Citus) + Valkey + Meilisearch + ClamAV + WAF + CDN + Captcha sidecar

## Intent

Author Helm + Kustomize manifests for the forms Layer-A substrate: OCI CDN (per-pack edge for form-renderer WASM + design-system primitives), OCI WAF (rate-limit + bot-detection + CSP enforcement), Postgres + Citus 12.x (form-definition + response-store with tenant_id shard key + column-level envelope encryption per ADR-FORMS-0003), Valkey 8.1 (RESP3 wire-compatible) (rate-limit + session + WAF cache), Meilisearch 0.10.0 (response search; PII-redacted index per AC-09), ClamAV 1.3 + OPSWAT MetaDefender (file-upload scan), Captcha sidecar (hCaptcha primary; Cloudflare Turnstile fallback; Friendly Captcha tertiary per ADR-FORMS-0002).

## ChangeSet boundary

One cohesive ChangeSet: 12 Helm chart bundles + shared Kustomize base + 11 per-pack overlays + 2 OpenTofu files (CDN edge config + node-library publishers). No Rust code; pure IaC. Per-pack secret references via OpenBao SecretReference.

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
helm lint microservices/forms/iac/helm/response-cache-valkey
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
- Citus docs, Valkey Sentinel docs, Meilisearch docs, ClamAV docs, hCaptcha docs, Cloudflare Turnstile docs.

## Counterpart Benchmark

- Counterpart: Salesforce Web-to-Lead public intake, HubSpot Forms embed infrastructure, Slack workflow form intake, and ServiceNow catalog item forms.
- Defensible parity claim: Layer-A must provision the complete response path before any authoring or submitter workflow can claim production readiness.
- Differentiator: Oyatie binds cache, search, captcha, WAF, CDN, Citus, OpenBao, ClamAV, and pack overlays as one auditable substrate.
- Grep counterpart names: Salesforce Web-to-Lead; HubSpot Forms; Slack workflow form intake; ServiceNow catalog item forms.

## Foundation A-G Substance

- A. Product scope: Layer-A provisions the Forms runtime substrate for authoring, rendering, collection, export, and distribution.
- B. Domain model: the substrate must not define form semantics; it hosts the storage, cache, search, scan, captcha, WAF, and CDN boundaries.
- C. Contracts: Helm and Kustomize outputs must align with manifest, OpenAPI, AsyncAPI, proto, SLO, and catalog service names.
- D. Policy: pack overlays enforce captcha provider bans, data residency, RLS, and OpenBao secret-reference shape before rollout.
- E. Operations: chart smoke tests, kind install, pack-kr overlay apply, and secret-reference checks are the deployment stop condition.
- F. Observability: pod readiness, chart drift, shard readiness, WAF posture, captcha sidecar health, and scan service health are emitted.
- G. Promotion: no downstream IP can claim done until Layer-A charts lint, dry-run, install, and pass readiness gates.

## Remediation Notes

- Added grep-recognized counterpart names for foundation IP parity verification.
- Preserved the existing Layer-A substrate detail and acceptance gates.
