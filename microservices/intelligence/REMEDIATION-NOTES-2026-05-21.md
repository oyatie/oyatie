<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: intelligence
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  tier_references_scrubbed: 46
  ADR_0316_citations_replaced: 4
  cellular_criticality_preserved: 1
-->

## Wave 15-IP-substance scrub (2026-05-21)

Assigned bucket: IP-BUCKET-H.

Scope interpreted as the `intelligence` µservice's stamped short core IPs, not already-long journey IPs.
Detection used line clustering, repeated heading inspection, and source grounding against `PRD.md`, `ARCHITECTURE.md`,
`manifest.json`, `competitor-parity-matrix.md`, policy fragments, SLOs, contracts, and runbooks.

Rewritten in place with bespoke Wave 15 substance sections:

- `IP-005-domain-layer-eval-record.md`
- `IP-006-domain-layer-attribution.md`
- `IP-007-kernel-model-router.md`
- `IP-008-kernel-guardrail-stack.md`
- `IP-009-kernel-audit-tap.md`
- `IP-011-adapter-anthropic.md`
- `IP-012-adapter-openai.md`
- `IP-013-adapter-google-vertex.md`
- `IP-014-adapter-bedrock.md`
- `IP-015-kernel-guardrail-eu-ai-act.md`
- `IP-016-streaming-sse-transport.md`
- `IP-017-streaming-websocket-transport.md`
- `IP-018-multi-modal-audio-video.md`
- `IP-019-library-first-caller-eval.md`
- `IP-020-brand-ux-surface-components.md`
- `IP-021-eval-golden-set.md`
- `IP-022-audit-tap-merkle-seal.md`
- `IP-023-byok-credential-rotation.md`
- `IP-024-minor-protection-wiring.md`
- `IP-025-cn-pipl-pack-adapter.md`

Preserved as already-substantive or outside the stamped 55-line core signature:

- `IP-001-consumer-intelligence-substrate.md`
- `IP-001-domain-layer-dispatch-request.md`
- `IP-002-domain-layer-secret-reference.md`
- `IP-003-domain-layer-refusal-decision.md`
- `IP-004-domain-layer-routing-decision.md`
- `IP-010-usecase-dispatch-flow.md`
- all `IP-journey-*.md` files.

Deleted as duplicative: none.

Counterpart references added across rewritten IPs include OpenAI, Anthropic, Google, Google Vertex, AWS Bedrock,
Azure OpenAI, OpenRouter, Alibaba Qwen, Tencent Hunyuan, CloudTrail, Azure Monitor, and provider-evaluation
counterparts where the IP scope demanded them.

Follow-up: some preserved non-stamped IPs and journey IPs still do not contain the generic counterpart keywords used
by the broad grep check; they were not rewritten because they did not match the stamped short-core signature.

## Wave 15-journey-IP substance pass

Assigned µservice: `intelligence`.

Inventory found 39 long `IP-*.md` files over 200 lines. Template-loop signatures were detected in 16 journey IPs:
`j22`, `j30`, `j31`, `j91` through `j100`, `j108`, `j132`, and `j144`.

Rewritten in place:

- `IP-journey-j22-spam-classification.md`
- `IP-journey-j30-minor-safety-classifier.md`
- `IP-journey-j31-spam-cib-signals.md`
- `IP-journey-j91-us-msb-mtl-overlay.md`
- `IP-journey-j92-br-lgpd-us-parent-dsar.md`
- `IP-journey-j93-in-dpdpa-rbi-overlay.md`
- `IP-journey-j94-sox404-public-company-controls.md`
- `IP-journey-j95-iso27001-soc2-annual-audit.md`
- `IP-journey-j96-ksa-uae-mena-onboarding.md`
- `IP-journey-j97-sg-pdpa-mas-tenant.md`
- `IP-journey-j98-au-privacy-apra-cps234.md`
- `IP-journey-j99-multi-pack-conflict-resolution.md`
- `IP-journey-j100-pack-rollout-first-action.md`
- `IP-journey-j108-ranking-and-metering-model.md`
- `IP-journey-j132-applicant-screening-with-fairness-gate.md`
- `IP-journey-j144-filter-and-drafter-consumer-tier.md`

Substance changes:

- Replaced repeated numbered work/test/appendix loops with journey-specific rows naming the source trigger, actor,
  Cedar or contract probe, state effect, evidence touch, and counterpart equivalence.
- Grounded rows in existing intelligence surfaces: `contracts/openapi/intelligence-v1.yaml`,
  `contracts/proto/intelligence-v1.proto`, `contracts/asyncapi/intelligence-events-v1.yaml`, Cedar policy fragments,
  SLO/dashboards, and existing runbooks.
- Deleted ungrounded generated expansions instead of preserving line count.

Edit accounting from rewrite pass: 271 substantive rows written, 3,375 generated scaffold rows deleted, 109 counterpart
references added.

Verification:

- Re-ran long-IP inventory after rewrite; edited regulatory/step-loop files dropped below the long-IP threshold except
  `j132`, whose remaining length is substantive pre-existing API/fairness detail plus the new journey rows.
- Re-ran repeated opening-token check for remaining long IPs; top repeated table openings are now frontmatter separators
  or unique artifact rows, not 30+ repeated action labels.
- Re-ran signature grep for generic step rows, generic positive-path test rows, numbered implementation/boundary/verification/rigor loops, and numbered appendix loop identifiers; no matches remain in journey IP files.

Follow-up: long journey IPs that were preserved (`j02`, `j03`, `j13`, `j14`, `j16`, `j39`, `j43`, `j44`, `j49`,
`j51`, `j69`, `j70`, `j72`, `j76`, `j77`, `j79`, `j82`, `j83`, `j89`, `j90`, `j123`, `j150`) did not match the
journey-row template-loop signature in this pass.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/intelligence/onboarding/ai-platform-engineer-first-week.md`
- `microservices/intelligence/performance-benchmark-numbers-2026-05-20.md`

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: preserved `manifest.json` RTO 300s / RPO 60s under ADR-0343 because it is stricter than HIPAA, SOC2, and EU-AI high-risk floors. Alternative considered: use only provider-failover runbooks; rejected because BYOK, refusal evidence, and audit taps have recoverable state. Cost: active-active dispatch, hot audit queues, and regulated-cell credential pinning stay in scope.
- Capacity model: set ADR-0340 baseline to 0.5 vCPU/1GiB per 10 advisory streams, 1GiB retrieval/eval artifact storage, four provider connections, and max 20 dispatch / 8 retrieval workers per tenant. Alternative considered: scale only by token count; rejected because retrieval bytes, refusal bursts, and tool calls stress different queues. Cost: admission logic must inspect consent, BYOK, provider, and abuse-defence dimensions.
- Sustainability + cost attribution: added ADR-0344 fields to dispatch, completion, refusal, BYOK, prompt-injection, and audit-tap rows. Alternative considered: provider invoice reconciliation only; rejected because high-risk AI governance needs per-provider and per-refusal emissions evidence. Cost: offline evals may route by carbon, but protected real-time/refusal paths cannot.
- API versioning posture: added ADR-0342 carrier triplet, SDK semver, last-three/180-day support, and tenant pinning for public/admin APIs. Alternative considered: internal-only versioning because tier is `internal`; rejected because tenant advisory clients and admin APIs are public-facing contracts. Cost: public carriers remain stable even while provider-dispatch mesh keeps ADR-0145 direct gRPC.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

D4-BUCKET-1 trigger-based IP doctrine propagation.

- Root IPs scanned: 64
- Trigger A additions: 27
- Trigger B additions: 30
- Trigger C additions: 42
- Trigger D additions: 3
- Root IPs unmatched: 4
- Doctrine sources: ADR-0338, ADR-0342, ADR-0343, ADR-0344, ADR-0345; `specs/compliance-pack-floors.json`.
- Idempotence: skipped any IP section that already existed; no unmatched root IPs were edited.

IP-by-IP changes:
- `microservices/intelligence/IP-001-consumer-intelligence-substrate.md`: added DR posture.
- `microservices/intelligence/IP-001-domain-layer-dispatch-request.md`: added API Versioning.
- `microservices/intelligence/IP-004-domain-layer-routing-decision.md`: added DR posture.
- `microservices/intelligence/IP-005-domain-layer-eval-record.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-006-domain-layer-attribution.md`: added API Versioning, Sustainability emission.
- `microservices/intelligence/IP-007-kernel-model-router.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-008-kernel-guardrail-stack.md`: added Sustainability emission.
- `microservices/intelligence/IP-009-kernel-audit-tap.md`: added Sustainability emission.
- `microservices/intelligence/IP-010-usecase-dispatch-flow.md`: added Sustainability emission.
- `microservices/intelligence/IP-011-adapter-anthropic.md`: added Sustainability emission, Pod runtime tier.
- `microservices/intelligence/IP-012-adapter-openai.md`: added Sustainability emission.
- `microservices/intelligence/IP-013-adapter-google-vertex.md`: added Sustainability emission.
- `microservices/intelligence/IP-014-adapter-bedrock.md`: added Sustainability emission.
- `microservices/intelligence/IP-015-kernel-guardrail-eu-ai-act.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-016-streaming-sse-transport.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-017-streaming-websocket-transport.md`: added API Versioning.
- `microservices/intelligence/IP-018-multi-modal-audio-video.md`: added API Versioning, Sustainability emission.
- `microservices/intelligence/IP-019-library-first-caller-eval.md`: added DR posture.
- `microservices/intelligence/IP-020-brand-ux-surface-components.md`: added API Versioning, Sustainability emission.
- `microservices/intelligence/IP-021-eval-golden-set.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-022-audit-tap-merkle-seal.md`: added Sustainability emission.
- `microservices/intelligence/IP-023-byok-credential-rotation.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j02-code-blue-clinical-summarizer.md`: added API Versioning, Sustainability emission.
- `microservices/intelligence/IP-journey-j03-acute-risk-triage.md`: added API Versioning, Sustainability emission.
- `microservices/intelligence/IP-journey-j100-pack-rollout-first-action.md`: added Sustainability emission.
- `microservices/intelligence/IP-journey-j108-ranking-and-metering-model.md`: added Sustainability emission.
- `microservices/intelligence/IP-journey-j123-audience-and-copy-assist.md`: added API Versioning, DR posture.
- `microservices/intelligence/IP-journey-j13-legal-request-classifier.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j132-applicant-screening-with-fairness-gate.md`: added DR posture.
- `microservices/intelligence/IP-journey-j14-bounded-summary-dispatch.md`: added API Versioning, Sustainability emission.
- `microservices/intelligence/IP-journey-j144-filter-and-drafter-consumer-tier.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j150-brand-safety-and-caption-assist.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/intelligence/IP-journey-j16-speech-intent-assistive-parser.md`: added API Versioning, Sustainability emission.
- `microservices/intelligence/IP-journey-j22-spam-classification.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j30-minor-safety-classifier.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j31-spam-cib-signals.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j39-transcription-summarization.md`: added DR posture.
- `microservices/intelligence/IP-journey-j43-clinical-summary-assist.md`: added DR posture.
- `microservices/intelligence/IP-journey-j44-clinical-transcription.md`: added DR posture.
- `microservices/intelligence/IP-journey-j49-support-reply-assist.md`: added DR posture, Pod runtime tier.
- `microservices/intelligence/IP-journey-j51-po-line-item-extraction.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j69-delegated-agent.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j70-contract-draft.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j72-translation-advice.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j76-risk-and-explanation.md`: added API Versioning.
- `microservices/intelligence/IP-journey-j77-risk-and-explanation.md`: added API Versioning, DR posture.
- `microservices/intelligence/IP-journey-j79-risk-and-explanation.md`: added API Versioning.
- `microservices/intelligence/IP-journey-j82-risk-and-explanation.md`: added API Versioning, DR posture.
- `microservices/intelligence/IP-journey-j83-risk-and-explanation.md`: added API Versioning.
- `microservices/intelligence/IP-journey-j89-risk-and-explanation.md`: added API Versioning.
- `microservices/intelligence/IP-journey-j90-risk-and-explanation.md`: added API Versioning.
- `microservices/intelligence/IP-journey-j91-us-msb-mtl-overlay.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j92-br-lgpd-us-parent-dsar.md`: added Sustainability emission.
- `microservices/intelligence/IP-journey-j93-in-dpdpa-rbi-overlay.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j94-sox404-public-company-controls.md`: added DR posture, Sustainability emission.
- `microservices/intelligence/IP-journey-j95-iso27001-soc2-annual-audit.md`: added Sustainability emission.
- `microservices/intelligence/IP-journey-j96-ksa-uae-mena-onboarding.md`: added Sustainability emission.
- `microservices/intelligence/IP-journey-j97-sg-pdpa-mas-tenant.md`: added Sustainability emission.
- `microservices/intelligence/IP-journey-j98-au-privacy-apra-cps234.md`: added Sustainability emission.
- `microservices/intelligence/IP-journey-j99-multi-pack-conflict-resolution.md`: added Sustainability emission.


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.55 vCPU; baseline_ram_per_tenant: 1024 MiB; storage_per_tenant: 8 GB.
- connections_per_tenant: valkey=2, postgres=4, outbound_http=32.
- scaling_dimension: per_request; cell_placement_class: Tier-1.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.55 vCPU / 1024 MiB / 8 GB reserves CPU and memory for guardrail/eval work while outbound HTTP is intentionally high for provider routing.
- Rejected: per_user sizing was rejected because a few automated callers can generate high provider and guardrail request volume.
- Cost: Tier-1 placement commits Intelligence to substrate nodepool and credential-handling controls without elevating all provider calls to Tier 0.

### Block 2: dr
- rto_p99_seconds: 1800; rpo_p99_seconds: 300; multi_region_active_active: true.
- backup_substrate: postgres_wal_g, openbao_seal_unseal, object_storage_versioned, audit_chain_merkle_seal; failover_runbook: runbooks/model-router-stall-investigation.md; replication_shape: active-active-multi-az-cross-region.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 1800s / RPO 300s follows high-risk AI and health-data floors while preserving audit and credential recovery substrates.
- Rejected: active-passive only was rejected because the architecture declares active-active AI substrate failover.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 1; evidence: microservices/intelligence/PRD.md, microservices/intelligence/ARCHITECTURE.md, microservices/intelligence/IP-002-domain-layer-secret-reference.md, microservices/intelligence/contracts/openapi/intelligence-v1.yaml.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Intelligence owns tenant AI transport, provider credential references, guardrail/audit paths, and high-risk evaluation routing, which ADR-0338 classifies as substrate-touching tenant data at Tier 1.
- Rejected: Tier 2 was rejected because intelligence transport touches tenant prompts, provider credentials, and audit outcomes.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: provider-specific unpinned contracts were rejected because tenants need stable AI routing and audit APIs across model changes.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: postgresql, valkey, cedar, openbao, wasmtime, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: Postgres, Valkey, Cedar, OpenBao, Wasmtime, and OpenTofu cover routing state, cache, policy, credential vaulting, sandbox-adjacent tooling, and IaC.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1, oci-guest/kms@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace, secret, and KMS module declarations are required because provider credentials and tenant encryption references are part of the service surface.
- Rejected: declaring only app namespace modules was rejected because ADR-0338/0343 require credential substrate recovery evidence.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.
