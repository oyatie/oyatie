---
doc_class: FAQ
microservice: detection
persona: detection-engineer + ml-fairness-engineer
date: 2026-05-20
doc_status: published
---

# Detection Engineer FAQ — detection

## Why Flink for streaming and Spark for batch — why not Apache Beam unified?

Per ADR-DET-001-streaming-vs-batch-substrate-split.md. Apache Beam is portable across runners (Flink, Spark, Dataflow) but its state-management abstraction is the LCD of those runners — Flink's `OperatorState` + `KeyedState` is richer than Beam's. We give up portability we don't need; we get Flink's full state model. We similarly give up Beam's unified-batch-streaming model for Spark's mature AQE + Iceberg integration.

## Why Cedar rules over OPA / Drools?

Per ADR-0307 §"Cedar over OPA" + ADR-DET-001. Cedar's policies are statically analyzable: the validator can prove termination + bounded resource use BEFORE the rule deploys. OPA's Rego is Turing-complete (you can author a rule that loops). For streaming detection where a runaway rule has measurable customer impact (a $1B/year payments stream stalling for 30 s = $1k+ of dropped transactions), bounded resource use is load-bearing.

The trade-off: Cedar lacks Rego's recursion + arbitrary aggregation. We added `rule-of-N` + `temporal-aggregate` extensions for the windowed aggregates fraud detection needs.

## A model card says `fairness_audit_signoff_id: pending` — can I deploy it to production?

No. Per ADR-0308 § "Model card requirements" + IP-019. A model cannot promote to production until:

1. Drift baseline distribution captured (`drift_baseline_distribution_id`).
2. Fairness audit run AND PASSED for ALL packs the model will serve.
3. ADR or fairness-board signoff on any pack-specific exemption (extremely rare; requires written justification).

The CI gate `detection-model-card-acceptance` enforces this; a missing signoff blocks the deploy.

## The drift dashboard shows divergence > threshold but the metrics dashboard shows normal performance. Should I roll back?

Not automatically. Drift > threshold is a SIGNAL not a VERDICT. The diagnostic path (per `runbooks/model-drift-alert.md`):

1. Is the drift on a feature the model gives high weight to? Low weight → likely benign.
2. Is the drift correlated with a known event (new tenant onboarded; product launch; holiday-shopping spike)? If so, expected.
3. Are downstream metrics (false-positive rate, false-negative rate, mitigation-action rate) drifting in the same direction? If so, the drift is meaningful.
4. Is the affected tenant population disproportionately a protected class? If so, escalate to fairness review BEFORE deciding.

The default action on confirmed drift: human-review hold; retrain on the new window only after the underlying event is understood.

## A tenant says they want EU AI Act Annex III §1 conformity but they're a B2C consumer product. Which obligations apply?

EU AI Act Annex III §1 covers biometric identification. §3 covers educational scoring. §4 covers employment-related. The detection µservice's models touch:

- §1 if biometric (face/voice/fingerprint matching) is in scope — only relevant if the tenant integrates with `identity` for biometric verification.
- §4 for hiring-related detection (only relevant for the `network` µservice's recruiter ranker, not detection's fraud detection).
- §5 (essential private services) for fraud detection — payment-fraud + AML are explicitly carve-out in Annex III §5 footnote-c if fraud-prevention purpose can be demonstrated. We claim the carve-out + document the demonstration in the per-tenant compliance dossier.

A B2C consumer product's fraud detection is typically §5-carved-out; consult legal per pack.

## Why is the rule engine called "Cedar" rather than just "policy"?

Cedar (https://www.cedarpolicy.com/) is AWS's open-source policy language we adopted per ADR-0090 and extended per ADR-DET-001. It's the same policy substrate as the rest of the platform's Authorization Service (audit-chain, IAM, KMS) — using it for detection rules unifies the policy emission + audit-chain shape across the platform.

## A streaming job's checkpoint stalled — what's the recovery path?

Per `runbooks/flink-checkpoint-stalled.md`:

1. Check the TaskManager logs for the stalled checkpoint's barrier-id. Identify the operator that's slow to align.
2. Common cause: state-backend back-pressure (RocksDB compaction blocking the write). Check `flink_taskmanager_status_jvm_thread_blocked_count`.
3. If RocksDB: trigger a state-backend tuning (`taskmanager.state.checkpoints.dir` on a faster disk; raise `state.backend.rocksdb.thread.num`).
4. If application-state-explosion: the rule may have unbounded state (rare, since Cedar bounds at validate-time). Check for an in-flight rule that has a `rule-of-N` with `window_seconds > 86400` — those are operationally suspect.

If checkpoint hasn't recovered in 10 min, fail-over the job (Flink high-availability mode re-elects a new JobManager).

## When does a SAR-candidate pipeline emit to FinCEN?

Per AML-sanctions detection family + 31 CFR § 1020.320. A SAR-candidate emitted by the pipeline goes to:

1. Tenant's BSA officer queue (per the tenant's BSA program).
2. Tenant's BSA officer reviews within 30 days of detection.
3. If the BSA officer confirms, the SAR is filed with FinCEN within 30 days (60 days if no suspect identified per § 1020.320(b)(3)).

The detection µservice emits CANDIDATES; we do NOT auto-file with FinCEN. The tenant's BSA officer decides. Per ADR-0310 the case-management subsystem tracks SAR-candidate → BSA review → FinCEN filing → audit-chain evidence.

## Why do we have 8 detection families instead of N specialized µservices?

Per ADR-0307. The 8 families share enough infrastructure (feature store, rules engine, model registry, investigation case-management, fairness audit, replay) that splitting them into 8 µservices would duplicate the substrate 8× without benefit. The decision boundary: a new family is added when (a) its features are share-worthy with at least one existing family, (b) its mitigation action class fits in our existing taxonomy. A family that needs distinct substrate (e.g., biometric-spoofing detection with image-tensor primitives) would warrant a separate µservice.

## A tenant in pack-us-financial wants explainable-AI for every decision. Can we provide that?

Per ADR-0308 § "Explainability" + EU AI Act Annex IV obligations. Yes, with caveats:

- We emit SHAP-class explanations (feature contributions) on every decision; for the streaming job, this adds ~ 50 ms tail per decision (we run SHAP in async post-decision so the streaming critical path is unaffected).
- The explanation is stored in the audit-chain evidence bundle.
- Per GDPR Art. 22 + EU AI Act Art. 86, the user has the right to a "meaningful information about the logic involved" — SHAP feature contributions qualify; raw model weights do not.

For US tenants under ECOA Reg B + Fair Credit Reporting Act, the explanation must additionally itemize the specific principal reasons for adverse action (the "Reg B reason codes"). We map SHAP contributions to FCRA / ECOA-compliant reason codes in the post-decision flow.

## What's the boundary between detection and observability + analytics?

- `detection`: scores events, applies mitigations, manages investigations, audits fairness. Tenant-facing for tenant-fraud-control.
- `observability`: SRE telemetry. Internal-facing.
- `analytics`: tenant-facing OLAP for business dashboards.

A tenant query for "show me my fraud-mitigation-action rate last week" hits `analytics` (which reads MVs that detection emits). A tenant query for "show me the SAR-candidate pipeline status" hits `detection` directly (case-management surface). An SRE query for "show me Flink TaskManager memory pressure" hits `observability`.
