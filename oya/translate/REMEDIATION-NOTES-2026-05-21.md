<!-- WAVE 15J SCRUB COMPLETION REPORT
  µservice: translate
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  prd_md_tier_references_scrubbed: 4
  architecture_md_tier_references_scrubbed: 15
  compliance_md_pack_tier_references_scrubbed: 0
  total_files_modified: 34
  total_lines_changed: 749
  ADR_0316_citations_replaced_with_0329_0330_0331: 5
  cellular_tier_references_preserved: 2 (per ADR-0248)
  halt_cleanly: yes
-->

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/translate/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/translate/IP-001-iac-and-pack-overlays.md`
- `microservices/translate/IP-010-bulk-translate-stack.md`
- `microservices/translate/IP-014-router-rest-worker-app.md`
- `microservices/translate/PHASE-01-TRANSLATE-PLATFORM.md`
- `microservices/translate/PRD.md`
- `microservices/translate/iac/helm/translate/templates/networkpolicy.yaml`
- `microservices/translate/iac/helm/translate/values.yaml`
- `microservices/translate/multi-region.md`

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 26.
- Trigger A matched: 2.
- Trigger B matched: 11.
- Trigger C matched: 16.
- Trigger D matched: 2.
- IPs unmatched: 5.

### IP changes
- `microservices/translate/IP-001-iac-and-pack-overlays.md` — added DR posture.
- `microservices/translate/IP-002-translate-router-kernel.md` — added DR posture, Sustainability emission.
- `microservices/translate/IP-003-translate-router-domain.md` — added DR posture, Sustainability emission.
- `microservices/translate/IP-004-translate-router-usecase-and-api.md` — added Sustainability emission.
- `microservices/translate/IP-008-language-detection-stack.md` — added DR posture.
- `microservices/translate/IP-009-document-translation-stack.md` — added Pod runtime tier.
- `microservices/translate/IP-011-real-time-stream-stack.md` — added DR posture.
- `microservices/translate/IP-012-engine-adapter-foundry-runtime.md` — added Sustainability emission.
- `microservices/translate/IP-014-router-rest-worker-app.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/translate/IP-015-hg-translate-gate-registration.md` — added DR posture, Pod runtime tier.
- `microservices/translate/IP-journey-j100-pack-rollout-first-action.md` — added Sustainability emission.
- `microservices/translate/IP-journey-j72-translation-memory.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/translate/IP-journey-j91-us-msb-mtl-overlay.md` — added DR posture, Sustainability emission.
- `microservices/translate/IP-journey-j92-br-lgpd-us-parent-dsar.md` — added Sustainability emission.
- `microservices/translate/IP-journey-j93-in-dpdpa-rbi-overlay.md` — added DR posture, Sustainability emission.
- `microservices/translate/IP-journey-j94-sox404-public-company-controls.md` — added DR posture, Sustainability emission.
- `microservices/translate/IP-journey-j95-iso27001-soc2-annual-audit.md` — added Sustainability emission.
- `microservices/translate/IP-journey-j96-ksa-uae-mena-onboarding.md` — added Sustainability emission.
- `microservices/translate/IP-journey-j97-sg-pdpa-mas-tenant.md` — added Sustainability emission.
- `microservices/translate/IP-journey-j98-au-privacy-apra-cps234.md` — added Sustainability emission.
- `microservices/translate/IP-journey-j99-multi-pack-conflict-resolution.md` — added Sustainability emission.

### Unmatched IPs
- `microservices/translate/IP-005-translation-memory-stack.md` — no trigger match; no doctrine section added.
- `microservices/translate/IP-006-termbase-and-glossary-stack.md` — no trigger match; no doctrine section added.
- `microservices/translate/IP-007-quality-estimation-stack.md` — no trigger match; no doctrine section added.
- `microservices/translate/IP-010-bulk-translate-stack.md` — no trigger match; no doctrine section added.
- `microservices/translate/IP-013-engine-adapters-external.md` — no trigger match; no doctrine section added.

### Follow-up
- `microservices/translate/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: preserved existing router RTO 600s and TM/termbase RPO 300s under ADR-0343 because they satisfy HIPAA RPO and sit inside EU-AI high-risk RTO floor. Alternative considered: one generic 1800s/300s high-risk target; rejected because the router already claims faster restart. Cost: engine failover, TM restore, and caption-stream recovery must be pack-aware.
- Capacity model: set ADR-0340 baseline to 1 vCPU/2GiB router-rest, 1GiB TM/termbase per project, four engine connections per language-pair pool, and max eight router/adaptor plus four document workers per tenant. Alternative considered: size by total tenant seats; rejected because segments, language pairs, documents, and streams drive separate pressure. Cost: admission must protect real-time translation from bulk localization.
- Sustainability + cost attribution: added ADR-0344 fields to translation, TM, termbase, bulk, QE, and engine-routing audit rows. Alternative considered: calculate spend from vendor invoices; rejected because residency and Art. 50 evidence require per-call provider/model attribution. Cost: carbon routing can optimize batch jobs but cannot defer medical, emergency, or high-risk real-time calls.
- API versioning posture: adopted ADR-0342 carrier triplet, SDK semver, tenant pinning, and 180-day support for REST/webhook/proto clients. Alternative considered: version only the standalone Translate API; rejected because shared-substrate callers need the same stable public contract. Cost: product and substrate integrations carry three live date contracts.


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.35 vCPU; baseline_ram_per_tenant: 768 MiB; storage_per_tenant: 16 GB.
- connections_per_tenant: valkey=2, postgres=4, outbound_http=24.
- scaling_dimension: per_request; cell_placement_class: Tier-3.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.35 vCPU / 768 MiB / 16 GB reserves local TM, glossary, document staging, and high outbound MT provider concurrency per tenant.
- Rejected: per_message was rejected because document and batch requests vary too widely to map cleanly to individual message count.
- Cost: Tier-3 placement avoids substrate isolation cost while keeping enough app-cell capacity for regulated translation content.

### Block 2: dr
- rto_p99_seconds: 2100; rpo_p99_seconds: 300; multi_region_active_active: false.
- backup_substrate: postgres_wal_g, object_storage_versioned, valkey; failover_runbook: runbooks/tm-corruption-restore.md; replication_shape: active-passive-cross-region-continuous.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 2100s follows the service's 35 minute recovery target; RPO 300s preserves translation memory and document status without claiming unsupported active-active writes.
- Rejected: active-active was rejected because the documented translate recovery model is active-passive with TM restore.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 2; evidence: microservices/translate/PRD.md, microservices/translate/ARCHITECTURE.md, microservices/translate/IP-002-translate-router-kernel.md, microservices/translate/contracts/openapi/translate.yaml.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Translate is a first-party translation and localization application; it handles tenant content but does not execute tenant-customer code or own key-management substrate responsibilities.
- Rejected: Tier 0 was rejected because document sandboxing is not tenant-customer code execution under ADR-0338.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: unpinned translation contracts were rejected because TM and localization clients need stable request/response shapes.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: postgresql, valkey, cedar, openbao, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: Postgres, Valkey, Cedar, OpenBao, and OpenTofu cover translation memory, cache, policy, provider-secret references, and IaC.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1, oci-guest/dns@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace, secret, and DNS module declarations match public translation APIs and external provider secret use.
- Rejected: declaring provider-specific IaC modules was rejected because ADR-0339 centralizes reusable cloud primitives only.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.
