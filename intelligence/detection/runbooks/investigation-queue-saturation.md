---
doc_class: Runbook
shape: Reference
status: Proposed
date: 2026-05-21
owner_team: axis-detection
microservice: detection
related_adrs:
  - ADR-0307-detection-substrate-streaming-batch
  - ADR-0308-ml-model-lifecycle-ai-act-compliance
  - ADR-0309-detection-fairness-audit-civil-rights
  - ADR-0310-investigation-case-management
  - ADR-0263-observability-emission-contract
  - ADR-0105-13-layer-enum-and-check-family-patterns
  - ADR-0131-per-microservice-flat-layout
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0703-cas-cache-live-apex.md
planned_enforcement_ref: oya-governance-detection-baseline
bnf_version: v4.1
layer_enum: layer_5_shared_substrate
---

# Investigation Queue Saturation

## A Trigger conditions

1. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 1 in <= 5 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
2. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 2 in <= 6 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
3. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 3 in <= 7 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
4. Command/API: oya detection rules-engine inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 4 in <= 8 minutes; emit ADR-0263 audit tag DetectionRulePromoted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
5. Command/API: oya detection composite-scorer inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 5 in <= 9 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
6. Command/API: oya detection graph-store-community-detection inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 6 in <= 10 minutes; emit ADR-0263 audit tag DetectionGraphClusterFound.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
7. Command/API: oya detection investigation-bridge inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 7 in <= 11 minutes; emit ADR-0263 audit tag InvestigationCaseOpened.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
8. Command/API: oya detection sandbox-replay inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 8 in <= 12 minutes; emit ADR-0263 audit tag DetectionReplayCompleted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
9. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 9 in <= 13 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
10. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 10 in <= 14 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
11. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 11 in <= 15 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
## B Pre-checks

1. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 1 in <= 5 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
2. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 2 in <= 6 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
3. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 3 in <= 7 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
4. Command/API: oya detection rules-engine inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 4 in <= 8 minutes; emit ADR-0263 audit tag DetectionRulePromoted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
5. Command/API: oya detection composite-scorer inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 5 in <= 9 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
6. Command/API: oya detection graph-store-community-detection inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 6 in <= 10 minutes; emit ADR-0263 audit tag DetectionGraphClusterFound.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
7. Command/API: oya detection investigation-bridge inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 7 in <= 11 minutes; emit ADR-0263 audit tag InvestigationCaseOpened.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
8. Command/API: oya detection sandbox-replay inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 8 in <= 12 minutes; emit ADR-0263 audit tag DetectionReplayCompleted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
9. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 9 in <= 13 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
10. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 10 in <= 14 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
11. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 11 in <= 15 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
## C Procedure

1. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 1 in <= 5 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
2. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 2 in <= 6 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
3. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 3 in <= 7 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
4. Command/API: oya detection rules-engine inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 4 in <= 8 minutes; emit ADR-0263 audit tag DetectionRulePromoted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
5. Command/API: oya detection composite-scorer inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 5 in <= 9 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
6. Command/API: oya detection graph-store-community-detection inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 6 in <= 10 minutes; emit ADR-0263 audit tag DetectionGraphClusterFound.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
7. Command/API: oya detection investigation-bridge inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 7 in <= 11 minutes; emit ADR-0263 audit tag InvestigationCaseOpened.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
8. Command/API: oya detection sandbox-replay inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 8 in <= 12 minutes; emit ADR-0263 audit tag DetectionReplayCompleted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
9. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 9 in <= 13 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
10. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 10 in <= 14 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
11. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 11 in <= 15 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
## D Verification

1. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 1 in <= 5 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
2. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 2 in <= 6 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
3. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 3 in <= 7 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
4. Command/API: oya detection rules-engine inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 4 in <= 8 minutes; emit ADR-0263 audit tag DetectionRulePromoted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
5. Command/API: oya detection composite-scorer inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 5 in <= 9 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
6. Command/API: oya detection graph-store-community-detection inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 6 in <= 10 minutes; emit ADR-0263 audit tag DetectionGraphClusterFound.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
7. Command/API: oya detection investigation-bridge inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 7 in <= 11 minutes; emit ADR-0263 audit tag InvestigationCaseOpened.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
8. Command/API: oya detection sandbox-replay inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 8 in <= 12 minutes; emit ADR-0263 audit tag DetectionReplayCompleted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
9. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 9 in <= 13 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
10. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 10 in <= 14 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
11. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 11 in <= 15 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
## E Rollback

1. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 1 in <= 5 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
2. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 2 in <= 6 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
3. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 3 in <= 7 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
4. Command/API: oya detection rules-engine inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 4 in <= 8 minutes; emit ADR-0263 audit tag DetectionRulePromoted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
5. Command/API: oya detection composite-scorer inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 5 in <= 9 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
6. Command/API: oya detection graph-store-community-detection inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 6 in <= 10 minutes; emit ADR-0263 audit tag DetectionGraphClusterFound.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
7. Command/API: oya detection investigation-bridge inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 7 in <= 11 minutes; emit ADR-0263 audit tag InvestigationCaseOpened.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
8. Command/API: oya detection sandbox-replay inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 8 in <= 12 minutes; emit ADR-0263 audit tag DetectionReplayCompleted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
9. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 9 in <= 13 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
10. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 10 in <= 14 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
11. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 11 in <= 15 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
## F Post-incident

1. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 1 in <= 5 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
2. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 2 in <= 6 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
3. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 3 in <= 7 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
4. Command/API: oya detection rules-engine inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 4 in <= 8 minutes; emit ADR-0263 audit tag DetectionRulePromoted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
5. Command/API: oya detection composite-scorer inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 5 in <= 9 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
6. Command/API: oya detection graph-store-community-detection inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 6 in <= 10 minutes; emit ADR-0263 audit tag DetectionGraphClusterFound.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
7. Command/API: oya detection investigation-bridge inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 7 in <= 11 minutes; emit ADR-0263 audit tag InvestigationCaseOpened.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
8. Command/API: oya detection sandbox-replay inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 8 in <= 12 minutes; emit ADR-0263 audit tag DetectionReplayCompleted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
9. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 9 in <= 13 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
10. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 10 in <= 14 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
11. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 11 in <= 15 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
## G References

1. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 1 in <= 5 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
2. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 2 in <= 6 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
3. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 3 in <= 7 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
4. Command/API: oya detection rules-engine inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 4 in <= 8 minutes; emit ADR-0263 audit tag DetectionRulePromoted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
5. Command/API: oya detection composite-scorer inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 5 in <= 9 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
6. Command/API: oya detection graph-store-community-detection inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 6 in <= 10 minutes; emit ADR-0263 audit tag DetectionGraphClusterFound.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
7. Command/API: oya detection investigation-bridge inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 7 in <= 11 minutes; emit ADR-0263 audit tag InvestigationCaseOpened.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
8. Command/API: oya detection sandbox-replay inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 8 in <= 12 minutes; emit ADR-0263 audit tag DetectionReplayCompleted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
9. Command/API: oya detection streaming-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 9 in <= 13 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
10. Command/API: oya detection batch-pipeline inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 10 in <= 14 minutes; emit ADR-0263 audit tag DetectionSignalEmitted.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
11. Command/API: oya detection feature-store inspect --tenant <tenant_id> --trace <trace_id>.
   Timing budget: complete step 11 in <= 15 minutes; emit ADR-0263 audit tag DetectionFeatureMaterialized.
   If this step fails: freeze the affected version, open InvestigationCaseOpened, and move to rollback section.
