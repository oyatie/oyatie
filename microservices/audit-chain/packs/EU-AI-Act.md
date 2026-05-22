---
doc_class: CompliancePackOverlay
pack_id: EU-AI-ACT-2024-HIGH-RISK
microservice: audit-chain
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# audit-chain EU AI Act Compliance Pack Overlay

## Pack Identity
- Full pack name: EU AI Act audit-chain AI evidence overlay.
- Citing jurisdiction: European Union harmonised AI regulation.
- Version: EU-AI-ACT-2024-HIGH-RISK-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2024/1689/oj
- Cited law: Regulation (EU) 2024/1689.
- Covered audit-chain surface: AI risk decisions, model-touch events, human review, technical-documentation hashes, drift events, incident evidence, conformity proofs, and evidence exports.
- Pack activation means audit-chain preserves AI Act evidence without raw prompts, raw training data, or unnecessary personal data.
- The overlay seals model governance decisions and downstream AI touchpoints.
- Data classes include `AUDIT_CHAIN_EUAI_EVIDENCE`, `AUDIT_CHAIN_EUAI_MODEL_TOUCH`, and `AUDIT_CHAIN_EUAI_INCIDENT_PROOF`.
- High-risk evidence requires stronger retention and traceability.
- ADR-0064 keeps EU AI Act evidence behavior in a pack overlay.
- ADR-0251 supplies high-risk cell, provider BYOK, and pack retention constraints.
- ADR-0263 supplies model-touch observability linkage.
- PCI-DSS is omitted because audit-chain does not authorize payments.
- Payment AI evidence remains tokenized unless payment service activates PCI scope.

## Data Model Deltas
- Add `audit_event.eu_ai_event_class`.
- Add `audit_event.model_registry_id`.
- Add `audit_event.risk_tier`.
- Add `audit_event.annex_iii_context`.
- Add `audit_event.model_touch_trace_hash`.
- Add `audit_event.prompt_hash`.
- Add `audit_event.output_hash`.
- Add `audit_event.human_review_id`.
- Add `audit_event.fundamental_rights_assessment_id`.
- Add `audit_event.technical_documentation_hash`.
- Add `audit_event.conformity_assessment_ref`.
- Add `audit_event.model_card_ref`.
- Add `merkle_leaf.eu_ai_pack_version`.
- Add `seal_batch.high_risk_ai_cell_id`.
- Add `retention_rule.eu_ai_evidence_floor_iso8601`.
- Add `export_job.eu_ai_act_manifest_hash`.
- Add `incident.serious_ai_incident_bundle_hash`.
- Add `drift_event.detector_version`.
- Add `threshold_change.rollback_plan_id`.
- Add `query_session.ai_evidence_scope`.
- Add `signature.eu_ai_key_attestation_ref`.
- Add `audit_shadow.audit_chain_eu_ai_event_id`.
- Add `tenant_audit_chain_config.eu_ai_act_retention_profile`.
- Add `tenant_audit_chain_config.ai_evidence_export_scope`.

## Cedar Policy Deltas
- Policy `EUAI-audit-chain-ingest-01`: require model registry id for AI evidence event.
- Policy `EUAI-audit-chain-ingest-02`: forbid raw prompt or raw output payload.
- Policy `EUAI-audit-chain-risk-01`: require risk tier for AI decision event.
- Policy `EUAI-audit-chain-review-01`: require human review id for high-risk action.
- Policy `EUAI-audit-chain-doc-01`: require technical documentation hash for high-risk system.
- Policy `EUAI-audit-chain-fra-01`: require fundamental-rights assessment id when applicable.
- Policy `EUAI-audit-chain-conformity-01`: require conformity ref before high-risk launch event.
- Policy `EUAI-audit-chain-card-01`: require model card ref before model activation event.
- Policy `EUAI-audit-chain-drift-01`: require detector version for drift event.
- Policy `EUAI-audit-chain-threshold-01`: require rollback plan for threshold change.
- Policy `EUAI-audit-chain-incident-01`: create serious AI incident bundle on candidate confirmation.
- Policy `EUAI-audit-chain-export-01`: require compliance approval for AI evidence export.
- Policy `EUAI-audit-chain-export-02`: require manifest hash before export release.
- Policy `EUAI-audit-chain-retention-01`: forbid purge before AI evidence retention floor.
- Policy `EUAI-audit-chain-route-01`: require EU/high-risk eligible cell for AI evidence.
- Policy `EUAI-audit-chain-key-01`: require key attestation for AI evidence signatures.
- Policy `EUAI-audit-chain-query-01`: restrict AI evidence query by approved scope.
- Policy `EUAI-audit-chain-replay-01`: require prompt-free replay manifest.
- Policy `EUAI-audit-chain-support-01`: forbid support view of raw prompt material.
- Policy `EUAI-audit-chain-admin-01`: require elevated ACR for AI retention changes.
- Policy `EUAI-audit-chain-webhook-01`: require approved evidence sink for AI exports.
- Policy `EUAI-audit-chain-pack-01`: defer deactivation while AI evidence is retained.
- Policy `EUAI-audit-chain-audit-01`: require self-audit seal for AI evidence policy change.
- Policy `EUAI-audit-chain-sample-01`: require high-risk AI trace sampling proof.

## API Contract Deltas
- `POST /events` requires model registry id for AI event classes.
- `POST /events` rejects raw prompt and raw output markers.
- `POST /events` requires risk tier.
- `POST /events` requires human review id for high-risk action.
- `POST /technical-doc-hashes` stores documentation hash event.
- `POST /fundamental-rights-assessments` stores assessment linkage event.
- `POST /conformity` stores conformity reference event.
- `POST /model-cards` stores model card reference event.
- `POST /drift-events` requires detector version.
- `POST /threshold-changes` requires rollback plan id.
- `POST /serious-ai-incidents` creates evidence bundle.
- `POST /exports/ai-evidence` requires compliance approval.
- `GET /exports/ai-evidence/{id}` returns manifest hash.
- `DELETE /events/{id}` returns retention conflict before AI evidence floor.
- `POST /replication/plan` validates high-risk eligible target cell.
- `POST /keys/attest` stores AI evidence key attestation.
- `GET /events` requires AI evidence query scope.
- `POST /replay` requires prompt-free replay manifest.
- `POST /webhooks` requires approved evidence sink.
- `POST /pack/deactivate` returns retained AI evidence count.

## Workflow Deltas
- AI event ingest validates model registry and risk tier.
- Prompt and output payload detector rejects raw model content.
- High-risk action workflow requires human review id.
- Technical-documentation workflow seals documentation hash.
- Fundamental-rights workflow seals assessment linkage.
- Conformity workflow seals declaration and assessment refs.
- Model-card workflow seals card ref before activation.
- Drift workflow records detector version.
- Threshold workflow requires rollback plan.
- Serious AI incident workflow bundles model, trace, and review evidence.
- Export workflow builds EU AI Act evidence manifest.
- Retention workflow blocks purge before AI evidence floor.
- Replication workflow validates high-risk eligible target cell.
- Key attestation workflow verifies signing key before use.
- Query workflow scopes AI evidence fields.
- Replay workflow uses prompt-free manifest.
- Webhook workflow validates approved evidence sink.
- Pack activation workflow scans existing AI event classes.
- Pack deactivation waits for retained AI evidence.
- Self-audit workflow seals every AI evidence policy change.

## SLO Deltas
- AI evidence event ingest validation p99 must stay <= 100 ms.
- Raw prompt rejection p99 must complete <= 30 seconds.
- Human review evidence linkage p99 must stay <= 500 ms.
- Technical documentation seal p99 target is <= 5 minutes.
- Conformity evidence seal p99 target is <= 5 minutes.
- Drift event seal p99 target is <= 1 second.
- Threshold rollback event seal p99 target is <= 1 second.
- Serious AI incident bundle creation p99 target is <= 10 minutes.
- AI evidence export manifest p99 target is <= 30 minutes.
- Retention conflict response p99 must stay <= 300 ms.
- High-risk route validation p99 must stay <= 100 ms.
- Key attestation lookup p99 must stay <= 200 ms.
- Prompt-free replay throughput target is >= 10k events per minute.
- AI evidence query authorization p99 must stay <= 100 ms.
- EU AI audit-chain dashboard lag target is <= 5 minutes.
- Evidence integrity verification cadence is daily.

## Audit-event class additions
- `AuditChainEuAiEventIngested` records model id and tier.
- `AuditChainEuAiRawPromptRejected` records detector verdict.
- `AuditChainEuAiRiskDecisionSealed` records risk tier.
- `AuditChainEuAiHumanReviewLinked` records review id.
- `AuditChainEuAiTechnicalDocSealed` records documentation hash.
- `AuditChainEuAiFraLinked` records assessment id.
- `AuditChainEuAiConformitySealed` records conformity ref.
- `AuditChainEuAiModelCardSealed` records card ref.
- `AuditChainEuAiDriftEventSealed` records detector version.
- `AuditChainEuAiThresholdChangeSealed` records rollback plan.
- `AuditChainEuAiSeriousIncidentBundled` records bundle hash.
- `AuditChainEuAiEvidenceManifestCreated` records manifest hash.
- `AuditChainEuAiPurgeRefused` records retention floor.
- `AuditChainEuAiReplicationBlocked` records target cell.
- `AuditChainEuAiKeyAttested` records key ref.
- `AuditChainEuAiQueryScoped` records scope id.
- `AuditChainEuAiReplayPromptFree` records replay id.
- `AuditChainEuAiWebhookRefused` records sink id.
- `AuditChainEuAiPolicyChanged` records policy bundle.
- `AuditChainEuAiPackDeactivationDeferred` records retained count.

## Failure Modes specific to this pack
- AI event lacks model registry id; recovery is reject ingest.
- Raw prompt is submitted; recovery is quarantine and reject seal.
- Risk tier missing; recovery is reject event.
- High-risk event lacks human review id; recovery is reject event.
- Technical documentation hash missing; recovery is block high-risk launch evidence.
- Conformity ref missing; recovery is reject launch event.
- Model card ref missing; recovery is reject activation event.
- Drift event lacks detector version; recovery is reject event.
- Threshold change lacks rollback plan; recovery is reject event.
- Serious incident bundle misses trace; recovery is rebuild from observability ref.
- Export manifest mismatch appears; recovery is revoke and rebuild.
- Retention purge requested early; recovery is refuse purge.
- Replication target not high-risk eligible; recovery is block plan.
- Key attestation expires; recovery is pause AI evidence signing.
- Query lacks approved scope; recovery is deny query.
- Replay includes prompt material; recovery is halt replay.
- Webhook sink unapproved; recovery is disable sink.
- Pack deactivation requested with retained evidence; recovery is defer.
- Audit-chain backpressure appears; recovery is fail-closed for AI evidence events.
- Model registry drift appears; recovery is reconcile sealed refs with compliance registry.

## Cross-µservice coordination
- `tenancy` provides EU AI Act pack roster and eligible cell placement.
- `identity` provides reviewer and governance owner roles.
- `compliance` provides risk classification, model inventory, and incident workflows.
- `observability` provides model-touch traces and drift events.
- `policy-engine` loads all `EUAI-audit-chain-*` fragments.
- `workflow-engine` runs incident, export, and evidence workflows.
- `model-registry` supplies model cards and version refs.
- `foundry-runtime` or AI gateway emits prompt-free model-touch hashes.
- `mail` emits AI mail evidence classes.
- `drive` emits AI drive evidence classes.
- `calendar` emits AI calendar evidence classes.
- `storage` provides high-risk evidence backend proof.
- `cloud-kms` or OpenBao provides key attestation.
- `incident-response` consumes serious AI evidence bundles.
- `admin-console` renders AI evidence status.
- `legal` defines AI evidence export rules.
- `support` cannot view raw prompt material.
- `release-engine` gates model rollout on sealed evidence.
- `pack-registry` signs this EU AI Act audit-chain overlay.
