## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/forms/AUDIT-FINDINGS-2026-05-18.json
- microservices/forms/IP-001-layer-a-postgres-valkey-meilisearch-clamav-waf-cdn-captcha-iac.md
- microservices/forms/IP-006-postgres-citus-adapter-with-column-encryption.md
- microservices/forms/IP-007-valkey-adapter.md
- microservices/forms/PHASE-01-FORMS-FOUNDATION.md
- microservices/forms/PRD.md
- microservices/forms/capacity-model.md
- microservices/forms/catalog/oya-forms-valkey-adapter.yaml
- microservices/forms/coherence-audit-2026-05-20.md
- microservices/forms/dashboards/response-pipeline.json
- microservices/forms/feature-parity-matrix-2026-05-20.md
- microservices/forms/iac/helm/form-rest/values.yaml
- microservices/forms/iac/helm/response-cache-valkey/Chart.yaml
- microservices/forms/iac/helm/response-cache-valkey/values.yaml
- microservices/forms/iac/kustomize/base/kustomization.yaml
- microservices/forms/iac/kustomize/base/openbao-secret-references.yaml
- microservices/forms/manifest.json
- microservices/forms/policy/data-residency.md
- microservices/forms/runbooks/spam-flood-throttle.md

Counterpart-fact preservations:
- None; every Redis hit in this bucket described Oyatie-owned substrate vocabulary.

Files renamed (git mv):
- microservices/forms/IP-001-layer-a-postgres-redis-meilisearch-clamav-waf-cdn-captcha-iac.md -> microservices/forms/IP-001-layer-a-postgres-valkey-meilisearch-clamav-waf-cdn-captcha-iac.md
- microservices/forms/IP-007-redis-adapter.md -> microservices/forms/IP-007-valkey-adapter.md
- microservices/forms/catalog/oya-forms-redis-adapter.yaml -> microservices/forms/catalog/oya-forms-valkey-adapter.yaml
- microservices/forms/iac/helm/response-cache-redis/ -> microservices/forms/iac/helm/response-cache-valkey/
