---
doc_class: Runbook
title: RAG Corpus Drift Detection
status: Accepted
date: 2026-05-20
microservice: intelligence
severity: sev2
audience: sre, ai-safety-engineer, knowledge-operator
owner_team: axis-intelligence + knowledge-platform + ops-sre-reliability
doc_status: published
---

# Runbook: RAG Corpus Drift Detection

## Operator Contract
- Runbook id: intelligence-rag-corpus-drift-detection.
- Primary namespace: `intelligence`.
- Owning rotation: PagerDuty `oya-intelligence-primary`.
- Knowledge secondary: PagerDuty `oya-knowledge-platform-primary`.
- Incident channel: `#inc-intelligence`.
- Customer channel: `#support-rag-quality`.
- Protected surface: RAG corpus ingestion, chunking, embeddings, citation freshness, retrieval quality, adversarial text handling.
- Safety invariant: do not serve stale or adversarial corpus chunks as instructions.
- Evidence invariant: preserve corpus version, chunk ids, embedding model, and retrieval traces.
- Privacy invariant: redacted chunk exports only unless evidence store is used.
- Stop condition: drifted corpus is quarantined or reindexed, retrieval quality is back in SLO, and citations point to current corpus version.
- Evidence event: `EVT_INTELLIGENCE_RAG_CORPUS_DRIFT_INCIDENT`.
- Handoff API: `https://intelligence.internal.oyatie.dev/v1/intelligence/rag/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-quality?orgId=1&var-cell=prod-us-east-1`.
- Corpus dashboard: `https://grafana.dev.oyatie.internal/d/intelligence-substrate/corpus-freshness?orgId=1&var-pack=canonical-base`.
- Loki query: `{namespace="intelligence",runbook="rag-corpus-drift-detection"}`.
- Related catalog: `microservices/intelligence/catalog/oya-intelligence-guardrails-kernel.yaml`.
- Related dashboard: `microservices/intelligence/dashboards/intelligence-overview.json`.
- Related failure mode: `microservices/intelligence/failure-modes.md`.

## Trigger Conditions
- Alert `IntelligenceRagCorpusDriftDetected` fires.
- Alert `IntelligenceRagRetrievalQualityBurn` fires.
- Alert `IntelligenceCitationFreshnessHighAge` fires.
- Alert `IntelligenceEmbeddingIndexLagHigh` fires.
- Alert `IntelligenceRagAdversarialChunkDetected` fires.
- Metric `oya_intelligence_rag_corpus_drift_score` exceeds threshold.
- Metric `oya_intelligence_rag_hit_quality_ratio` drops below SLO.
- Metric `oya_intelligence_rag_citation_age_seconds` exceeds pack policy.
- Metric `oya_intelligence_embedding_index_lag_seconds` exceeds 1800.
- Metric `oya_intelligence_rag_adversarial_chunk_total` is non-zero.
- Metric `oya_intelligence_rag_chunk_tombstone_miss_total` increases.
- Metric `oya_intelligence_rag_corpus_version_skew_total` increases.
- Metric `oya_intelligence_rag_retrieval_empty_result_ratio` exceeds 0.05.
- Customer reports stale answer with citation.
- AI safety reports RAG text treated as instruction.
- Corpus source connector changed schema.
- Embedding model changed but index was not rebuilt.
- Tenant pack updates policy documents but retrieval cites prior version.
- Audit-chain lacks `intelligence.rag.corpus.indexed` after corpus ingest.
- Prompt fence bypass detection implicates RAG chunk.

## Symptoms
- Answers cite old policy or outdated product copy.
- Answers cite tombstoned documents.
- Retrieval returns empty result for known corpus content.
- Retrieval quality drops while provider latency is healthy.
- Embedding index has mixed model versions.
- Corpus version differs between index shards.
- `rag_corpus_version_skew=true` appears in logs.
- `adversarial_chunk_detected=true` appears in guardrail logs.
- `citation_freshness_status=expired` appears in answer trace.
- Chunk sanitizer strips content and retrieval score collapses.
- Ingest connector reports success but index worker lags.
- Corpus tombstone event exists but chunk remains retrievable.
- Tenant pack policy update is not visible in answers.
- RAG answer uses content from wrong tenant or pack.
- Prompt fence sees RAG chunk as instruction-like.
- Search recall drops for a specific language or locale.
- Retrieval is good for keyword search but poor for embedding search.
- Customer impact is answer quality, trust, and compliance.
- Severity rises to Sev1 if stale citations drive customer decisions.
- Severity rises to Sev0 if cross-tenant content is retrieved.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-intelligence-rag-drift-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export PACK=canonical-base`.
3. Acknowledge page: `pd incident ack --service intelligence --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-intelligence --severity sev2`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="rag")'`.
6. Query drift score: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_rag_corpus_drift_score{tenant_id="'$TENANT'",pack="'$PACK'"}'`.
7. Query hit quality: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_rag_hit_quality_ratio{tenant_id="'$TENANT'",pack="'$PACK'"}'`.
8. Query citation age: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_rag_citation_age_seconds{tenant_id="'$TENANT'",pack="'$PACK'"}'`.
9. Query index lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_intelligence_embedding_index_lag_seconds{tenant_id="'$TENANT'",pack="'$PACK'"}'`.
10. Query adversarial chunks: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_intelligence_rag_adversarial_chunk_total[5m])'`.
11. Open RAG dashboard: `open "https://grafana.dev.oyatie.internal/d/intelligence-substrate/rag-quality?orgId=1&var-tenant=$TENANT&var-pack=$PACK"`.
12. Open corpus dashboard: `open "https://grafana.dev.oyatie.internal/d/intelligence-substrate/corpus-freshness?orgId=1&var-tenant=$TENANT&var-pack=$PACK"`.
13. Read RAG logs: `kubectl -n intelligence logs deploy/intelligence-rag-api --since=60m | rg "corpus|chunk|embedding|retrieval|citation"`.
14. Read indexer logs: `kubectl -n intelligence logs deploy/intelligence-rag-indexer --since=60m | rg "index|embedding|tombstone|version|lag"`.
15. Inspect corpus status: `oya ops intelligence rag corpus status --tenant $TENANT --pack $PACK --cell $CELL --output json`.
16. Inspect corpus version: `oya ops intelligence rag corpus version --tenant $TENANT --pack $PACK --output yaml`.
17. Inspect index status: `oya ops intelligence rag index status --tenant $TENANT --pack $PACK --cell $CELL --output json`.
18. Inspect embedding model: `oya ops intelligence rag embedding-model status --tenant $TENANT --pack $PACK --output json`.
19. Inspect shard skew: `oya ops intelligence rag index shard-skew --tenant $TENANT --pack $PACK --output table`.
20. Query retrieval trace: `oya ops intelligence rag trace --tenant $TENANT --pack $PACK --query "<known-query>" --output json`.
21. Query citation trace: `oya ops intelligence rag citation trace --tenant $TENANT --pack $PACK --answer-id <answer-id> --output json`.
22. Inspect suspicious chunk redacted: `oya ops intelligence rag chunk inspect --tenant $TENANT --chunk <chunk-id> --redacted --output json`.
23. Check tombstones: `oya ops intelligence rag tombstone status --tenant $TENANT --pack $PACK --output table`.
24. Check source connector: `oya ops intelligence rag source status --tenant $TENANT --pack $PACK --source all --output table`.
25. Check source schema: `oya ops intelligence rag source schema-diff --tenant $TENANT --pack $PACK --output json`.
26. Check sanitizer: `oya ops intelligence rag sanitizer status --tenant $TENANT --pack $PACK --output json`.
27. Run retrieval eval: `oya ops intelligence eval run --pack $PACK --suite rag-retrieval-quality --tenant $TENANT --output json`.
28. Run adversarial chunk eval: `oya ops intelligence eval run --pack $PACK --suite rag-adversarial-corpus --tenant $TENANT --output json`.
29. Query indexed events: `oya audit-chain query --event-class intelligence.rag.corpus.indexed --tenant $TENANT --since 7d`.
30. Query tombstone events: `oya audit-chain query --event-class intelligence.rag.chunk.tombstoned --tenant $TENANT --since 7d`.
31. Query answer events: `oya audit-chain query --event-class intelligence.rag.answer.cited --tenant $TENANT --since 24h`.
32. Check support cases: `oya support cases list --tag rag-quality --tenant $TENANT --since 7d`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice intelligence --runbook rag-corpus-drift-detection --output evidence/incidents/$INCIDENT_ID.json`.
34. Export redacted corpus diff: `oya ops intelligence rag corpus diff --tenant $TENANT --pack $PACK --redacted --output evidence/incidents/$INCIDENT_ID-corpus-diff.json`.
35. Export retrieval eval: `oya ops intelligence eval export --suite rag-retrieval-quality --incident $INCIDENT_ID --output evidence/incidents/$INCIDENT_ID-rag-eval.json`.

### Diagnostic Decision Tree
```text
1. Is cross-tenant content retrieved?
   |-- yes: raise Sev0, quarantine index, and page security/privacy.
   |-- no: continue quality triage.
2. Is drift from stale corpus source?
   |-- yes: refresh source and reindex.
   |-- no: inspect embedding model, tombstones, and sanitizer.
3. Is adversarial chunk detected?
   |-- yes: quarantine chunk and coordinate prompt fence bypass detection.
   |-- no: inspect retrieval quality regression.
4. Is embedding model mixed across shards?
   |-- yes: rebuild index with one approved model version.
   |-- no: inspect source schema and citation projection.
5. Does retrieval eval pass after reindex?
   |-- yes: release quarantine and watch citation freshness.
   |-- no: keep incident open and page knowledge platform.
```

## Mitigation
1. Quarantine affected corpus: `oya ops intelligence rag corpus quarantine --tenant $TENANT --pack $PACK --reason $INCIDENT_ID --dry-run`.
2. Confirm quarantine if cross-tenant or adversarial content exists: `oya ops intelligence rag corpus quarantine --tenant $TENANT --pack $PACK --reason $INCIDENT_ID --confirm`.
3. Quarantine chunk: `oya ops intelligence rag chunk quarantine --tenant $TENANT --chunk <chunk-id> --reason $INCIDENT_ID`.
4. Hold RAG answers for affected pack: `oya flags set oya.intelligence.rag.answer_hold=true --tenant $TENANT --pack $PACK --reason $INCIDENT_ID`.
5. Force citation freshness warning: `oya flags set oya.intelligence.rag.citation_stale_warning=true --tenant $TENANT --pack $PACK --reason $INCIDENT_ID`.
6. Hold corpus deploys: incident hold PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
7. Refresh corpus source dry-run: `oya ops intelligence rag source refresh --tenant $TENANT --pack $PACK --source all --dry-run`.
8. Refresh corpus source confirmed: `oya ops intelligence rag source refresh --tenant $TENANT --pack $PACK --source all --confirm $INCIDENT_ID`.
9. Rebuild index dry-run: `oya ops intelligence rag index rebuild --tenant $TENANT --pack $PACK --embedding-model approved-current --dry-run`.
10. Rebuild index confirmed: `oya ops intelligence rag index rebuild --tenant $TENANT --pack $PACK --embedding-model approved-current --confirm $INCIDENT_ID`.
11. Apply tombstones: `oya ops intelligence rag tombstone apply --tenant $TENANT --pack $PACK --confirm $INCIDENT_ID`.
12. Run retrieval eval before release: `oya ops intelligence eval run --pack $PACK --suite rag-retrieval-quality --tenant $TENANT --expect pass`.
13. Notify support: `oya notify support --incident $INCIDENT_ID --template rag-quality-degraded`.
14. Notify tenant admin when stale answer was visible: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template rag-citation-stale`.
15. Notify AI safety when adversarial chunk is involved: `oya notify ai-safety --incident $INCIDENT_ID --category rag-adversarial-corpus`.
16. Emit mitigation audit: `oya audit-chain emit --event-class EVT_INTELLIGENCE_RAG_CORPUS_DRIFT_INCIDENT --incident $INCIDENT_ID --field mitigation=corpus-quarantine-or-reindex`.
17. Keep raw corpus exports redacted unless evidence storage is required.
18. Keep answer hold until retrieval eval passes.
19. Keep citation warnings until freshness is green.
20. Keep source connector owner in bridge for schema drift.

## Resolution
1. Patch source connector if ingest skipped or malformed content.
2. Patch chunker if document boundaries or instruction text were mishandled.
3. Patch sanitizer if adversarial instructions were not neutralized.
4. Patch embedding indexer if shard model versions diverged.
5. Patch tombstone projection if deleted chunks remained retrievable.
6. Patch citation freshness if answers cited old corpus version.
7. Add regression fixture for stale policy document citation.
8. Add regression fixture for adversarial RAG instruction chunk.
9. Run RAG tests: `cargo test -p oya-intelligence-rag-api rag_corpus -- --nocapture`.
10. Run eval tests: `cargo test -p oya-governance-eval-domain rag_retrieval -- --nocapture`.
11. Run production gate: `cargo run -p oya-dev-cli -- gate validate intelligence-rag-corpus --production-snapshot --cell $CELL`.
12. Verify retrieval eval: `oya ops intelligence eval run --pack $PACK --suite rag-retrieval-quality --tenant $TENANT --expect pass`.
13. Release answer hold: `oya flags set oya.intelligence.rag.answer_hold=false --tenant $TENANT --pack $PACK --reason resolved-$INCIDENT_ID`.
14. Unhold deploys: recovery PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
15. Seal audit: `oya audit-chain emit --event-class EVT_INTELLIGENCE_RAG_CORPUS_DRIFT_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `IntelligenceRagCorpusDriftDetected` is green.
- `oya_intelligence_rag_hit_quality_ratio` is back within SLO.
- `oya_intelligence_rag_citation_age_seconds` is below pack policy.
- `oya_intelligence_embedding_index_lag_seconds` is below 300.
- No cross-tenant chunk is retrievable.
- Adversarial chunks are quarantined or sanitized.
- Index shards use one approved embedding model version.
- Retrieval eval passes.
- Audit-chain contains indexed, tombstoned, mitigation, and resolution events.
- Support has updated customer-visible cases.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: intelligence-rag-corpus-drift-detection
microservice: intelligence
event_class: EVT_INTELLIGENCE_RAG_CORPUS_DRIFT_INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# RAG Corpus Drift Detection postmortem

## Summary
- Which tenant, pack, corpus source, chunk ids, and embedding model drifted.
- Whether stale, adversarial, tombstoned, or cross-tenant content was involved.
- Whether customer-visible answers cited drifted content.

## Timeline
- Drift detected:
- Corpus quarantined:
- Index rebuilt:
- Eval passed:
- Audit sealed:

## Customer Impact
- Answers affected:
- Citations affected:
- Tenants affected:
- Safety/privacy posture:

## Root Cause
- Source connector:
- Chunker:
- Sanitizer:
- Embedding index:
- Tombstone:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Source contract update:
```

## Escalation Path
- Page `oya-intelligence-primary` for RAG corpus drift.
- Page `oya-knowledge-platform-primary` for source, chunking, and index ownership.
- Page `oya-ai-safety-primary` when adversarial chunks or prompt-fence bypass are implicated.
- Page `oya-privacy-primary` for cross-tenant or personal data retrieval.
- Page `oya-audit-chain-primary` when indexing or answer citation events are missing.
- Notify `#inc-intelligence` with tenant, pack, and corpus source.
- Notify `#support-rag-quality` before tenant communication.
- Notify `#compliance-review` when stale regulated corpus was cited.
- Escalate to executive incident commander for cross-tenant retrieval.
- Keep raw corpus evidence out of incident chat.

## Cross-µservice Coordination
- `audit-chain`: seal corpus indexed, tombstoned, answer cited, mitigation, and resolution events.
- `cloud-iam`: verify source connector principal and tenant scope.
- `cloud-kms`: verify embedding/index encryption keys and BYOK boundaries.
- `cloud-network`: verify source connector reachability.
- `tenancy`: verify pack, locale, and tenant corpus isolation.
- `ai-safety`: own adversarial chunk and prompt fence linkage.
- `privacy`: own cross-tenant or personal data exposure review.
- `support`: manage stale answer and citation cases.
- `observability`: attach RAG and corpus dashboards.
- `foundry`: pause corpus processing deploys while hold is active.
- `workflow-engine`: pause workflows relying on affected corpus answers.
- `comms-email`: send approved quality and all-clear notices.

## Runbook Maintenance
- Add new corpus source drift signatures after every incident.
- Keep eval set names aligned with CI.
- Keep redaction and evidence-storage boundaries explicit.
- Review this runbook after every embedding model change.
- Add every new source connector to Diagnostic Steps.
