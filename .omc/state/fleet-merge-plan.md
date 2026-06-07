# Fleet merge plan (base=Forgejo dev 7ea5b5ff259a)

Counts: {'REVIEW': 48, 'DISPOSE': 179, 'MERGE': 33}  total=260

## MERGE (33)

| branch | ahead | files | clean | last_commit | reason |
|---|--|--|--|--|--|
| feat/cloud-g005-billing-lifecycle-20260601 | 2 | 3 | True | 2026-06-01 | clean feature/fix |
| feat/cloud-g006-storage-replay-20260601 | 1 | 3 | True | 2026-06-01 | clean feature/fix |
| feat/cloud-g004-network-security-20260601 | 1 | 2 | True | 2026-06-01 | clean feature/fix |
| feat/cloud-g002-iac-plan-diff-20260601 | 1 | 3 | True | 2026-06-01 | clean feature/fix |
| fix/ci-rustup-preflight-20260601 | 1 | 1 | True | 2026-06-01 | clean feature/fix |
| feat/cloud-g001-resource-state-20260601 | 1 | 2 | True | 2026-06-01 | clean feature/fix |
| fix/final-review-pr47-20260601 | 2 | 4 | True | 2026-06-01 | clean feature/fix |
| feat/oya-ci-tide | 1 | 0 | True | 2026-05-31 | clean feature/fix |
| feat/trusted-surface-integrity | 1 | 10 | True | 2026-05-31 | clean feature/fix |
| feat/oya-cli-retire-increment | 1 | 3 | True | 2026-05-31 | clean feature/fix |
| land/ssot-projection | 2 | 186 | True | 2026-05-31 | clean feature/fix |
| feat/object-store-colossus-seams | 1 | 7 | True | 2026-05-31 | clean feature/fix |
| fix/gate-trunk-source-95 | 1 | 3 | True | 2026-05-31 | clean feature/fix |
| feat/oya-release-scaffold | 1 | 8 | True | 2026-05-31 | clean feature/fix |
| fix/dogfood-boundary-purity | 1 | 37 | True | 2026-05-31 | clean feature/fix |
| feat/scm-trait-scaffold | 1 | 7 | True | 2026-05-31 | clean feature/fix |
| adr-0517-cloud-native-scm | 1 | 1 | True | 2026-05-31 | clean feature/fix |
| adr-0518-oya-release | 1 | 1 | True | 2026-05-31 | clean feature/fix |
| adr-0488-linker | 1 | 1 | True | 2026-05-31 | clean feature/fix |
| feat/adr-ssot-projection | 1 | 9 | True | 2026-05-31 | clean feature/fix |
| chore/adr-citation-mold-linker-20260531 | 1 | 13 | True | 2026-05-30 | clean feature/fix |
| chore/openapi-workspace-route-parity-20260530 | 1 | 3 | True | 2026-05-30 | clean feature/fix |
| chore/loop-recovery-workspace-paths-20260530 | 1 | 6 | True | 2026-05-30 | clean feature/fix |
| chore/supply-chain-trivy-evidence-20260530 | 1 | 3 | True | 2026-05-30 | clean feature/fix |
| chore/glossary-mvp-token-20260530 | 1 | 4 | True | 2026-05-30 | clean feature/fix |
| chore/data-class-invoice-totals-20260530 | 1 | 14 | True | 2026-05-30 | clean feature/fix |
| chore/active-artifact-paths-20260530 | 1 | 3 | True | 2026-05-30 | clean feature/fix |
| chore/stage0-application-shell-path-20260530 | 1 | 3 | True | 2026-05-30 | clean feature/fix |
| chore/marketplace-doc-set-license-20260530 | 1 | 3 | True | 2026-05-30 | clean feature/fix |
| chore/pre-push-contract-dev-cli-path-20260530 | 1 | 4 | True | 2026-05-30 | clean feature/fix |
| chore/intelligence-vendor-lockin-paths-20260530 | 1 | 2 | True | 2026-05-30 | clean feature/fix |
| feat/intelligence-provider-pool-kernel-20260530 | 1 | 9 | True | 2026-05-30 | clean feature/fix |
| ci/rust-ci-clang-mold | 7 | 1 | True | 2026-05-30 | clean feature/fix |

## REVIEW (48)

| branch | ahead | files | clean | last_commit | reason |
|---|--|--|--|--|--|
| feat/oyatie-erp-hr-payroll-cloud-parity-20260601 | 17 | 314 |  | 2026-06-01 | large (ahead=17,files=314) |
| feat/oyatie-erp-hr-payroll-20260601 | 16 | 295 |  | 2026-06-01 | large (ahead=16,files=295) |
| feat/oya-ci-tide-review-semantics-20260601 | 2 | 4 | False | 2026-06-01 | conflicts with dev |
| feat/controller-oci-musl | 25 | 80 | False | 2026-05-31 | conflicts with dev |
| feat/emit-rust-test-targets | 1 | 682 |  | 2026-05-31 | large (ahead=1,files=682) |
| fix/buck2-cutover-tail | 1 | 5 |  | 2026-05-31 | structural/refactor |
| fix/libs-buck2-test-3crates | 1 | 6 |  | 2026-05-31 | structural/refactor |
| scratch/gate-diagnostics-20260530 | 13 | 65 |  | 2026-05-30 | non-standard name |
| chore/aspirational-enforcement-libs-20260530 | 1 | 6 | False | 2026-05-30 | conflicts with dev |
| chore/buck2-ffi-doc-catalog-20260530 | 1 | 4 |  | 2026-05-30 | structural/refactor |
| chore/buck2-ci-resume-20260530 | 3 | 3 |  | 2026-05-30 | structural/refactor |
| chore/ci-forge-clone-fix | 1 | 2 | False | 2026-05-30 | conflicts with dev |
| chore/trusted-surface-integrity | 1 | 8 | False | 2026-05-29 | conflicts with dev |
| feat/cd-obs-sla-observability-api-summary-read-request-validation | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-cph-kernel-failure-reason-tier-reachability | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-cursor-pagination-inmemory-reference-impl | 1 | 5 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-workflow-studio-policy-preview-rationale-rollup | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-intel-archmap-reachability-cycles | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-intel-api-semver-sunset-date-validation | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-obs-hyperscaler-metrics-multiwindow-burn-rate | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-marketplace-entitlement-suspend-reinstate-transitions | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-obs-cloud-domain-audit-topic-rollup | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-webauthn-packtier-attestation-policy-gate | 1 | 4 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-managed-k8s-cluster-lifecycle-provisioning-state-machine | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-tenant-quota-inmemory-five-axis-impl | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-cloud-capacity-committed-use-amortization | 1 | 4 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-cloud-finops-savings-portfolio-rollup | 1 | 4 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-workload-identity-decision-precedence-evaluator | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-obs-cloud-kernel-signal-headroom-report | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-intelligence-guardrails-fp-budget-headroom-weighted-aggregate | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-messenger-mention-fanout-usecase | 1 | 4 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-payroll-retro-adjustment-net-delta-kernel-2 | 1 | 4 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-ontology-kernel-link-instance-cardinality-enforcement | 1 | 4 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-route-policy-ranked-candidate-slate | 1 | 3 |  | 2026-05-29 | cd-experiment slice |
| feat/cd-pooling-openai-apikey-pool | 1 | 10 |  | 2026-05-29 | cd-experiment slice |
| feat/adr-scm-cicd-governance-2026-05-29 | 1 | 2 | False | 2026-05-29 | conflicts with dev |
| feat/cd-intel-guardrails-shadow-mode-fp-budget-kernel | 1 | 4 |  | 2026-05-29 | cd-experiment slice |
| feat/adr-0493-oya-ml-2026-05-28 | 1 | 5 | False | 2026-05-28 | conflicts with dev |
| fix/flattened-clean-architecture-taxonomy-2026-05-28 | 2 | 3814 |  | 2026-05-28 | large (ahead=2,files=3814) |
| chore/adr-0384-llm-gateway-oauth-subscription-pool-2026-05-28 | 1 | 4 | False | 2026-05-28 | conflicts with dev |
| chore/adr-0383-observability-reconciliation-2026-05-28 | 1 | 4 | False | 2026-05-28 | conflicts with dev |
| feature/managed-k8s-control-plane-host-2026-05-27 | 1 | 44 | False | 2026-05-27 | conflicts with dev |
| feature/analytics-build-2026-05-27 | 1 | 24 | False | 2026-05-27 | conflicts with dev |
| feature/managed-k8s-tenant-quota-2026-05-27 | 1 | 35 | False | 2026-05-27 | conflicts with dev |
| feature/llm-gateway-gapclose-2026-05-27 | 1 | 3 | False | 2026-05-27 | conflicts with dev |
| feature/salvage-live-postgres-rls-harness-2026-05-27 | 1 | 3 | False | 2026-05-27 | conflicts with dev |
| feature/adr-0376-managed-k8s-product-surface-2026-05-27 | 1 | 6 | False | 2026-05-27 | conflicts with dev |
| agent/backbone-microservices-20260523T081210Z | 30 | 382 |  | 2026-05-25 | structural/refactor |

## DISPOSE (179)

| branch | ahead | files | clean | last_commit | reason |
|---|--|--|--|--|--|
| chore/remove-context-injection-hooks |  |  |  | 2026-05-31 | already in dev |
| wave-5-0 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-1-2 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-5-2 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-3-1 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-4-1 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-4-2 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-3-0 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-5-1 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-4-0 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-1-1 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-1-0 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-0-2 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-2-1 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-0-1 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-2-0 |  |  |  | 2026-05-31 | scratch/ephemeral |
| wave-0-0 |  |  |  | 2026-05-31 | scratch/ephemeral |
| worktree-wf_e99f6a3a-031-5 |  |  |  | 2026-05-31 | already in dev |
| worktree-wf_e99f6a3a-031-6 |  |  |  | 2026-05-31 | dup of worktree-wf_e99f6a3a-031-5 |
| worktree-wf_fec208e2-8c4-11 |  |  |  | 2026-05-31 | dup of worktree-wf_e99f6a3a-031-5 |
| chore/adr-citation-mold-linker-20260530 |  |  |  | 2026-05-30 | already in dev |
| worktree-agent-a822b5af458d60951 |  |  |  | 2026-05-30 | scratch/ephemeral |
| worktree-agent-a4fd51f383a9e5e73 |  |  |  | 2026-05-30 | already in dev |
| ci/seaweedfs-volmax |  |  |  | 2026-05-30 | already in dev |
| chore/ci-go-live-phase1 |  |  |  | 2026-05-30 | already in dev |
| chore/buck2-firstparty |  |  |  | 2026-05-30 | already in dev |
| chore/buck2-thirdparty |  |  |  | 2026-05-29 | already in dev |
| chore/buck2-native-ffi |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | already in dev |
| chore/security-guardrails |  |  |  | 2026-05-29 | already in dev |
| chore/hooks-hygiene |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_66122331-818-2 |  |  |  | 2026-05-29 | dup of (detached) |
| chore/cloud-oya-split |  |  |  | 2026-05-29 | already in dev |
| chore/canonical-archb-green |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | dup of chore/canonical-archb-green |
| chore/canonical-catalog-backfill |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_65d11eb4-8a2-1 |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_65d11eb4-8a2-2 |  |  |  | 2026-05-29 | dup of worktree-wf_65d11eb4-8a2-1 |
| worktree-wf_65d11eb4-8a2-3 |  |  |  | 2026-05-29 | dup of worktree-wf_65d11eb4-8a2-1 |
| worktree-wf_f3811957-c47-1 |  |  |  | 2026-05-29 | already in dev |
| drain-363 |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_f3811957-c47-2 |  |  |  | 2026-05-29 | dup of drain-363 |
| drain-365 |  |  |  | 2026-05-29 | already in dev |
| drain-390 |  |  |  | 2026-05-29 | already in dev |
| drain-362 |  |  |  | 2026-05-29 | already in dev |
| drain-364 |  |  |  | 2026-05-29 | already in dev |
| drain-389 |  |  |  | 2026-05-29 | already in dev |
| drain-366 |  |  |  | 2026-05-29 | already in dev |
| drain-374 |  |  |  | 2026-05-29 | already in dev |
| drain-404 |  |  |  | 2026-05-29 | already in dev |
| drain-399 |  |  |  | 2026-05-29 | already in dev |
| drain-388 |  |  |  | 2026-05-29 | already in dev |
| drain-411 |  |  |  | 2026-05-29 | already in dev |
| drain-408 |  |  |  | 2026-05-29 | already in dev |
| drain-414 |  |  |  | 2026-05-29 | already in dev |
| drain-413 |  |  |  | 2026-05-29 | already in dev |
| drain-412 |  |  |  | 2026-05-29 | already in dev |
| drain-415 |  |  |  | 2026-05-29 | already in dev |
| drain-419 |  |  |  | 2026-05-29 | already in dev |
| drain-416 |  |  |  | 2026-05-29 | already in dev |
| drain-420 |  |  |  | 2026-05-29 | already in dev |
| drain-423 |  |  |  | 2026-05-29 | already in dev |
| drain-417 |  |  |  | 2026-05-29 | already in dev |
| drain-425 |  |  |  | 2026-05-29 | already in dev |
| drain-422 |  |  |  | 2026-05-29 | already in dev |
| drain-421 |  |  |  | 2026-05-29 | already in dev |
| drain-424 |  |  |  | 2026-05-29 | already in dev |
| drain-427 |  |  |  | 2026-05-29 | already in dev |
| drain-426 |  |  |  | 2026-05-29 | already in dev |
| drain-433 |  |  |  | 2026-05-29 | already in dev |
| drain-432 |  |  |  | 2026-05-29 | already in dev |
| drain-429 |  |  |  | 2026-05-29 | already in dev |
| drain-428 |  |  |  | 2026-05-29 | already in dev |
| drain-434 |  |  |  | 2026-05-29 | already in dev |
| drain-430 |  |  |  | 2026-05-29 | already in dev |
| drain-431 |  |  |  | 2026-05-29 | already in dev |
| drain-435 |  |  |  | 2026-05-29 | already in dev |
| drain-437 |  |  |  | 2026-05-29 | already in dev |
| drain-436 |  |  |  | 2026-05-29 | already in dev |
| feat/cd-finops-anomaly-to-recommendation-derivation |  |  |  | 2026-05-29 | already in dev |
| feat/cd-obs-tracing-client-w3c-traceparent-parser |  |  |  | 2026-05-29 | already in dev |
| feat/cd-tenant-quota-kernel-pressure-band-classifier |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-intel-archmap-reachability-cycles |
| feat/cd-ulid-crockford-timestamp-validation-hardening |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-intel-api-semver-sunset-date-validation |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-obs-hyperscaler-metrics-multiwindow-burn-rate |
| feat/cd-capacity-reservation-commitment-expiry-cancel-transitions |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-capacity-reservation-commitment-expiry-cancel-transitions |
| feat/cd-intel-attribution-claim-fanout-cap |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-intel-attribution-claim-fanout-cap |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-marketplace-entitlement-suspend-reinstate-transitions |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-obs-cloud-domain-audit-topic-rollup |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-managed-k8s-cluster-lifecycle-provisioning-state-machine |
| feat/cd-workflow-exec-sla-escalation-bucket-projection |  |  |  | 2026-05-29 | already in dev |
| feat/cd-intelligence-autonomy-ceiling-batch-most-restrictive-resolve |  |  |  | 2026-05-29 | already in dev |
| feat/cd-cloud-resource-domain-transition-graph-introspection |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-messenger-mention-fanout-usecase |
| feat/cd-workflow-studio-canvas-reachability-cycle-analysis |  |  |  | 2026-05-29 | already in dev |
| feat/cd-cloud-network-dns-zone-name-canonicalization |  |  |  | 2026-05-29 | dup of feat/cd-workflow-studio-canvas-reachability-cycle-analysis |
| worktree-wf_d77ac0c9-20d-17 |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_d77ac0c9-20d-14 |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_d77ac0c9-20d-1 |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_d77ac0c9-20d-4 |  |  |  | 2026-05-29 | already in dev |
| feat/cd-usage-window-burn-rate-forecast |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_d77ac0c9-20d-2 |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_d77ac0c9-20d-3 |  |  |  | 2026-05-29 | dup of worktree-wf_d77ac0c9-20d-2 |
| worktree-wf_d77ac0c9-20d-5 |  |  |  | 2026-05-29 | dup of worktree-wf_d77ac0c9-20d-2 |
| worktree-wf_d77ac0c9-20d-9 |  |  |  | 2026-05-29 | dup of worktree-wf_d77ac0c9-20d-2 |
| feat/cd-community-vote-ledger-hot-controversy-ranking |  |  |  | 2026-05-29 | already in dev |
| feat/cd-social-collab-consent-reconciliation-audit |  |  |  | 2026-05-29 | already in dev |
| feat/cd-mail-domain-thread-conversation-grouping-kernel |  |  |  | 2026-05-29 | already in dev |
| feat/cd-email-comms-kernel-inbound-dmarc-alignment-disposition |  |  |  | 2026-05-29 | already in dev |
| feat/cd-messenger-reaction-tally-kernel |  |  |  | 2026-05-29 | already in dev |
| feat/cd-hr-leave-carryover-forfeiture-kernel |  |  |  | 2026-05-29 | already in dev |
| feat/cd-ontology-query-engine-domain-reverse-direction-traversal |  |  |  | 2026-05-29 | already in dev |
| feat/cd-sla-obs-kernel-fleet-rollup-summary |  |  |  | 2026-05-29 | already in dev |
| (detached) |  |  |  | 2026-05-29 | dup of feat/cd-route-policy-ranked-candidate-slate |
| feat/cd-billing-kernel-invoice-total-tax-aggregator |  |  |  | 2026-05-29 | already in dev |
| feat/cd-wf-studio-dsl-emitter-domain-node-typology-semantics |  |  |  | 2026-05-29 | already in dev |
| feat/cd-oidc-issuer-verification-key-grace-selection |  |  |  | 2026-05-29 | already in dev |
| feat/cd-network-domain-routetable-longest-prefix-resolve |  |  |  | 2026-05-29 | already in dev |
| feat/cd-wf-engine-state-machine-domain-ordered-batch-fold |  |  |  | 2026-05-29 | already in dev |
| feat/cd-iac-domain-changeset-apply-approval-gate |  |  |  | 2026-05-29 | already in dev |
| feat/cd-metering-domain-window-rollup-kernel |  |  |  | 2026-05-29 | already in dev |
| feat/cd-payroll-retro-adjustment-net-delta-kernel |  |  |  | 2026-05-29 | already in dev |
| feat/cd-cedar-policy-version-diff-impact-report |  |  |  | 2026-05-29 | dup of feat/cd-payroll-retro-adjustment-net-delta-kernel |
| feat/cd-obs-domain-error-budget-window-kernel |  |  |  | 2026-05-29 | dup of feat/cd-payroll-retro-adjustment-net-delta-kernel |
| feat/cd-pooling-quota-fairness-reserve-reconcile |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_0e191f72-283-1 |  |  |  | 2026-05-29 | already in dev |
| feat/cd-pooling-seat-observability-otel |  |  |  | 2026-05-29 | already in dev |
| worktree-agent-a9542763ac7c6849b |  |  |  | 2026-05-29 | already in dev |
| feat/cd-pooling-sse-passthrough-streaming |  |  |  | 2026-05-29 | already in dev |
| feat/cd-pooling-anthropic-oauth-refresh-runtime |  |  |  | 2026-05-29 | already in dev |
| feat/cd-pooling-429-retry-after-rotation |  |  |  | 2026-05-29 | already in dev |
| feat/cd-pooling-openbao-secret-fetch |  |  |  | 2026-05-29 | already in dev |
| feat/cd-pooling-kernel-cooldown-window |  |  |  | 2026-05-29 | already in dev |
| feat/cd-pooling-hyper-client-transport |  |  |  | 2026-05-29 | already in dev |
| feat/pooling-bind-binary-2026-05-29 |  |  |  | 2026-05-29 | already in dev |
| worktree-wf_874f7838-ca6-1 |  |  |  | 2026-05-29 | already in dev |
| feat/cd-gate-run-all-affected-scope |  |  |  | 2026-05-29 | already in dev |
| feat/cd-cloud-iac-opentofu-plan-changeset-model |  |  |  | 2026-05-29 | already in dev |
| feat/cd-tenant-quota-usage-headroom-projection |  |  |  | 2026-05-29 | already in dev |
| feat/cd-identity-token-issue-otel-and-error-taxonomy |  |  |  | 2026-05-29 | already in dev |
| feat/cd-control-plane-drain-failure-taxonomy |  |  |  | 2026-05-29 | already in dev |
| feat/cd-intel-capability-registry-affected-target-index |  |  |  | 2026-05-29 | already in dev |
| feat/cd-wire-quality-lanes-into-aggregator |  |  |  | 2026-05-29 | already in dev |
| feat/cd-intel-autonomy-ceiling-tenant-tier-policy |  |  |  | 2026-05-29 | already in dev |
| feat/cd-cedar-policy-publish-rest-edge |  |  |  | 2026-05-29 | already in dev |
| feat/cd-workload-oidc-eddsa-ed25519-verification |  |  |  | 2026-05-29 | already in dev |
| feat/cd-cedar-domain-obligations-and-decision-annotations |  |  |  | 2026-05-29 | already in dev |
| worktree-agent-a698018b3358afaee |  |  |  | 2026-05-29 | already in dev |
| feat/adr-0506-aws-lc-rs-2026-05-28 |  |  |  | 2026-05-28 | already in dev |
| feat/adr-0507-webauthn-rs-2026-05-28 |  |  |  | 2026-05-28 | already in dev |
| feat/adr-0508-opensk-2026-05-28 |  |  |  | 2026-05-28 | already in dev |
| fix/bespoke-adrs-adr0509-align-2026-05-28 |  |  |  | 2026-05-28 | already in dev |
| worktree-agent-a6a252bd8926f6df3 |  |  |  | 2026-05-28 | already in dev |
| worktree-agent-a0ceeba502c5764a6 |  |  |  | 2026-05-28 | already in dev |
| dev |  |  |  | 2026-05-28 | dup of worktree-agent-a0ceeba502c5764a6 |
| worktree-agent-a27709e9510c25a1f |  |  |  | 2026-05-28 | already in dev |
| worktree-agent-a26b5cd60663753ee |  |  |  | 2026-05-28 | already in dev |
| worktree-agent-a5cac8f428b8acd48 |  |  |  | 2026-05-28 | already in dev |
| worktree-agent-a5ceeb141926fdc09 |  |  |  | 2026-05-28 | dup of worktree-agent-a5cac8f428b8acd48 |
| worktree-agent-a7cf1d25ad2adef0a |  |  |  | 2026-05-28 | dup of worktree-agent-a5cac8f428b8acd48 |
| worktree-agent-a908bfce2b1f97154 |  |  |  | 2026-05-28 | dup of worktree-agent-a5cac8f428b8acd48 |
| worktree-agent-ae034b6be71d6b96c |  |  |  | 2026-05-28 | dup of worktree-agent-a5cac8f428b8acd48 |
| worktree-agent-ae51991c2d3a7e014 |  |  |  | 2026-05-28 | dup of worktree-agent-a5cac8f428b8acd48 |
| worktree-agent-af4387e1372b9142e |  |  |  | 2026-05-28 | dup of worktree-agent-a5cac8f428b8acd48 |
| worktree-agent-afb9ad32f0b374a40 |  |  |  | 2026-05-28 | dup of worktree-agent-a5cac8f428b8acd48 |
| chore/jenkins-forgejo-ci-cutover-2026-05-28 |  |  |  | 2026-05-28 | already in dev |
| feature/managed-k8s-sla-observability-2026-05-27 |  |  |  | 2026-05-27 | already in dev |
| feature/managed-k8s-cluster-lifecycle-2026-05-27 |  |  |  | 2026-05-27 | already in dev |
| feature/wave-3-board-team-leader-2026-05-27 |  |  |  | 2026-05-27 | already in dev |
| feature/managed-k8s-unify-2026-05-27 |  |  |  | 2026-05-27 | already in dev |
| feature/identity-workload-gapclose-2026-05-27 |  |  |  | 2026-05-27 | already in dev |
| worktree-agent-a143c6532f5b76084 |  |  |  | 2026-05-27 | already in dev |
| worktree-agent-a3260c13e0cf2e450 |  |  |  | 2026-05-27 | dup of worktree-agent-a143c6532f5b76084 |
| worktree-agent-ae906facdb920654e |  |  |  | 2026-05-27 | dup of worktree-agent-a143c6532f5b76084 |

