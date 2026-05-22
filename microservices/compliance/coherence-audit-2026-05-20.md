# Compliance µservice ownership-coherence audit

Audit date: 2026-05-20.
Target µservice: `microservices/compliance/`.
Sole audit owner: this report author.
Write scope: compliance µservice only.
Inventory source: recursive file listing of `microservices/compliance/` on 2026-05-21.
Existing files seen before audit deliverables: 201.
Existing lines audited before audit deliverables: 43,432.
Chat-history matches processed: 15 targeted matches around compliance, substance, multi-context, OpenTofu, and ownership directives.
Public counterpart sources checked: Vanta, Drata, OneTrust official public product/help pages.
Classification scale: P0 hard contradiction in P0-priority µservice; P1 in-scope canonical violation; P2 inconsistency or missing proof; P3 advisory hardening.

## Five-citation anchor block

1. Canonical direction: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` §D-15..§D-20, especially multi-context deployment at lines 1730-2240, OpenTofu at lines 2241-2645, OS support at lines 2646-3044, Rust-strict policy at lines 3045-3490, OCI Always Free at lines 3491-3828, and audit decision tree at lines 3829-4232.
2. Machine-readable control surface: `specs/master-plan-sequencing.json` deployment contexts at lines 704-746, `iac_substrate` at lines 747-776, `supported_oses` at lines 777-816, `language_policy` at lines 817-856, and `oci_always_free` at lines 857-868.
3. µservice PRD read: `microservices/compliance/PRD.md` lines 1-127.
4. µservice architecture read: `microservices/compliance/ARCHITECTURE.md` lines 1-220 sampled directly, with full-file line count 1,281 and repeated section structure audited by search.
5. Documentation-rigor anchor: `docs/standards/documentation-rigor.md` lines 40-81 for retroactive applicability and mandatory artifact bar, plus lines 133-173 for intern-buildability and engineering-rigor checks.

## §1 µservice purpose summary

The compliance µservice is intended to own Oyatie's compliance-evidence, regulator-evidence, DSAR, breach-clock, pack-overlay, and auditor-facing assurance surface.
The PRD narrows the first product story to SOC 2 Type II, GDPR DSAR, HIPAA, PCI readiness, auditor self-service, and cross-tenant isolation.
The README widens the product story to pack registry, DPIA orchestration, breach notification workflow, regulator-audit evidence, cell-certification attestation, and compliance-control mapping.
The architecture document widens it further into principals, Cedar gates, tenant scoping, substrate binding, pack invariants, failure modes, SLO evidence, dashboards, runbooks, and IaC evidence.
The capability-tier matrix frames the service as a displacement substrate for Drata, Vanta, Hyperproof, AuditBoard, and LogicGate, but the batch assignment asks for union coverage against Vanta, Drata, and OneTrust.
The service's strongest existing purpose is not generic GRC alone; it is compliance-pack enforcement plus tamper-evident evidence generation across Oyatie's own cellular platform.
The service's strongest differentiator is deterministic pack-overlay precedence with audit-chain evidence, which is present in `capability-tiers/tier-matrix.md` lines 11-13 and in `decisions/ADR-COMP-001-pack-overlay-precedence-conflict-resolution.md`.
The service's weakest purpose boundary is the stale PRD non-goal at `PRD.md` lines 31-36, because the rest of the corpus now assigns control mapping, regulator engagement, DPIA, breach, and pack-registry ownership here.
The service is not deployable from current docs across the six canonical contexts because its IaC is root-level Kubernetes/Helm plus a Terraform-named module, not per-context OpenTofu.
The service is not OS-certified from current docs because it lacks `supported-oses.json` and per-OS package/CI declarations.
The service passes a narrow language-file scan: no forbidden source files with Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, F#, C#, Kotlin, or Swift extensions were found under the service path.
The service fails OCI demo_trial reconciliation because no `iac/oci-guest/always-free/` directory exists and the tier matrix does not state demo_trial-on-OCI equals Always Free.
The actual buildable product should be reframed as: compliance-pack registry and evidence automation for Oyatie tenants, with DSAR, breach, regulator, auditor, third-party assurance, and control mapping surfaces, deployed through OpenTofu across all six contexts.
The product should keep OneTrust scope in view: privacy operations, DSR automation, consent/preferences, third-party lifecycle, IT risk, compliance automation, and AI governance appear in OneTrust's current public product taxonomy.
The product should keep Vanta scope in view: compliance automation, risk, third-party risk, audit prep, trust center, and questionnaire automation appear in Vanta's current public surface.
The product should keep Drata scope in view: enterprise GRC, compliance automation, trust center, AI questionnaire assistance, and third-party risk management appear in Drata's current public surface.
The audit treats all six deployment contexts as in scope because neither PRD nor manifest declares a narrower, justified set.
The audit treats missing current implementation code as a buildability gap, not as proof that the service is intentionally documentation-only.
The audit treats legacy “Terraform” references as drift from the ADR-0328 OpenTofu doctrine even when the HCL could later be migrated.
The audit treats `No forbidden source files found` as a source-tree result only; documentation examples still need review before implementation prompts reuse them.

## §2 Inventory snapshot

Inventory note: size is line count from `wc -l`; role and coherence are audit classifications.

| File | Size | Role | Coherent with purpose? |
|---|---:|---|---|
| ARCHITECTURE.md | 1281 | architecture walkthrough | partial: broad and useful, but repeats scaffold-like generated expansions and omits canonical deployment/OS fields |
| AUDIT-FINDINGS-2026-05-20.json | 50 | prior audit summary | partial: useful inventory signal, but records Terraform in IaC roster |
| CHANGELOG.md | 43 | change history | partial: mentions Terraform module and therefore carries OpenTofu drift |
| IP-001-evidence-collector-bootstrap.md | 91 | implementation plan | yes: evidence collector aligns with core purpose |
| IP-002-soc2-control-mapping.md | 79 | implementation plan | yes: SOC 2 mapping aligns with compliance evidence |
| IP-003-gdpr-dsar-automation-pipeline.md | 113 | implementation plan | yes: DSAR aligns with PRD and OneTrust parity |
| IP-004-hipaa-min-necessary-log-substrate.md | 94 | implementation plan | yes: HIPAA logging aligns with pack purpose |
| IP-005-audit-chain-seal-coverage.md | 71 | implementation plan | yes: tamper evidence is core differentiator |
| IP-006-evidence-storage-seaweedfs.md | 72 | implementation plan | yes: evidence storage aligns with retention purpose |
| IP-007-auditor-readonly-portal.md | 88 | implementation plan | yes: auditor portal aligns with PRD |
| IP-008-pii-scrubber.md | 75 | implementation plan | yes: privacy evidence support |
| IP-009-retention-tier-policy.md | 58 | implementation plan | yes: pack retention support |
| IP-010-attestation-aggregator.md | 60 | implementation plan | yes: attestation aggregation fits evidence pipeline |
| IP-011-cross-microservice-evidence-fan-in.md | 58 | implementation plan | partial: cross-service dependency exists but no handoff doc |
| IP-012-evidence-replay.md | 54 | implementation plan | yes: replay aligns with auditability |
| IP-013-audit-anomaly-detection.md | 52 | implementation plan | yes: anomaly detection supports compliance evidence |
| IP-014-manual-evidence-upload-flow.md | 71 | implementation plan | yes: manual evidence upload appears in OpenAPI |
| IP-015-regulatory-pack-evidence-overlay.md | 76 | implementation plan | yes: pack overlay aligns strongly |
| IP-016-pack-registry-kernel.md | 32 | implementation plan | partial: relevant but thin for intern buildability |
| IP-017-pack-registry-domain.md | 27 | implementation plan | partial: relevant but thin |
| IP-018-dpia-orchestration-usecase.md | 35 | implementation plan | partial: relevant but thin |
| IP-019-breach-notification-workflow.md | 36 | implementation plan | partial: relevant but thin |
| IP-020-regulator-audit-evidence-rest.md | 35 | implementation plan | partial: relevant but thin |
| IP-021-cell-certification-attestation-worker.md | 31 | implementation plan | partial: relevant but thin |
| IP-022-compliance-control-mapping-domain.md | 30 | implementation plan | partial: relevant but thin |
| IP-023-pack-registry-grpc.md | 21 | implementation plan | partial: relevant but thin |
| IP-024-dpia-orchestration-adapter-postgres.md | 23 | implementation plan | partial: relevant but thin |
| IP-025-breach-notification-async-emit.md | 28 | implementation plan | partial: relevant but thin |
| IP-026-control-mapping-rest-and-sdk.md | 28 | implementation plan | partial: relevant but thin |
| IP-journey-j01-emergency-911-dispatch-pack-overlay.md | 412 | journey implementation plan | yes: emergency compliance overlay |
| IP-journey-j02-healthcare-code-blue-ehr-break-glass-privacy-officer.md | 833 | journey implementation plan | yes: regulated healthcare overlay |
| IP-journey-j13-higher-restriction-policy.md | 802 | journey implementation plan | yes: higher-restriction policy is central |
| IP-journey-j20-kr-pipa-notification-clock.md | 803 | journey implementation plan | yes: KR-PIPA breach clock aligns |
| IP-journey-j43-hipaa-cell-overlay.md | 420 | journey implementation plan | yes |
| IP-journey-j44-hipaa-consult-overlay.md | 420 | journey implementation plan | yes |
| IP-journey-j45-patient-record-overlay.md | 420 | journey implementation plan | yes |
| IP-journey-j46-rx-overlay.md | 420 | journey implementation plan | yes |
| IP-journey-j47-healthcare-billing-overlay.md | 420 | journey implementation plan | yes |
| IP-journey-j48-kr-fss-overlay.md | 420 | journey implementation plan | yes |
| IP-journey-j61-hipaa-pack.md | 430 | journey implementation plan | yes |
| IP-journey-j63-trial-pack.md | 430 | journey implementation plan | yes |
| IP-journey-j64-hipaa-boundary.md | 430 | journey implementation plan | yes |
| IP-journey-j65-gdpr-pack.md | 430 | journey implementation plan | yes |
| IP-journey-j66-tax-pack.md | 430 | journey implementation plan | partial: tax belongs at boundary with tax service |
| IP-journey-j67-warrant-pack.md | 430 | journey implementation plan | yes |
| IP-journey-j68-per-pack-attestation.md | 430 | journey implementation plan | yes |
| IP-journey-j71-kr-fss-report.md | 430 | journey implementation plan | yes |
| IP-journey-j73-publisher-pack.md | 430 | journey implementation plan | partial: needs marketplace/governance handoff |
| IP-journey-j74-pack-overlay-verification.md | 430 | journey implementation plan | yes |
| IP-journey-j75-incident-pack.md | 430 | journey implementation plan | yes |
| IP-journey-j76-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j77-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j78-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j79-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j80-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j81-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j82-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j83-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j84-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j85-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j86-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j87-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j88-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j89-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j90-pack-overlay-regulator.md | 430 | journey implementation plan | yes |
| IP-journey-j91-us-msb-mtl-overlay.md | 400 | journey implementation plan | partial: payments/regulatory boundary needs handoff |
| IP-journey-j92-br-lgpd-us-parent-dsar.md | 400 | journey implementation plan | yes |
| IP-journey-j93-in-dpdpa-rbi-overlay.md | 400 | journey implementation plan | yes |
| IP-journey-j94-sox404-public-company-controls.md | 400 | journey implementation plan | yes |
| IP-journey-j95-iso27001-soc2-annual-audit.md | 400 | journey implementation plan | yes |
| IP-journey-j96-ksa-uae-mena-onboarding.md | 400 | journey implementation plan | yes |
| IP-journey-j97-sg-pdpa-mas-tenant.md | 400 | journey implementation plan | yes |
| IP-journey-j98-au-privacy-apra-cps234.md | 400 | journey implementation plan | yes |
| IP-journey-j99-multi-pack-conflict-resolution.md | 400 | journey implementation plan | yes |
| IP-journey-j100-pack-rollout-first-action.md | 400 | journey implementation plan | yes |
| IP-journey-j101-pack-attestation.md | 865 | journey implementation plan | yes |
| IP-journey-j104-pack-attestation.md | 863 | journey implementation plan | yes |
| IP-journey-j105-pack-attestation.md | 863 | journey implementation plan | yes |
| IP-journey-j106-pack-attestation.md | 860 | journey implementation plan | yes |
| IP-journey-j118-data-sharing-pack-overlay.md | 430 | journey implementation plan | yes |
| IP-journey-j119-kyb-aml-bid-screening.md | 430 | journey implementation plan | partial: KYB/AML likely shared with identity/payments |
| IP-journey-j122-tax-withholding-overlay.md | 430 | journey implementation plan | partial: tax boundary needs handoff |
| IP-journey-j125-overlay-union-and-pack-delta.md | 430 | journey implementation plan | yes |
| IP-journey-j126-fedramp-conmon-pack-overlay.md | 425 | journey implementation plan | yes |
| IP-journey-j129-judicial-process-pack.md | 425 | journey implementation plan | yes |
| IP-journey-j130-whistleblower-protection-pack.md | 425 | journey implementation plan | partial: cites missing dashboard |
| IP-journey-j131-multi-jurisdiction-pack-overlay.md | 425 | journey implementation plan | yes |
| IP-journey-j132-eu-ai-act-and-multi-jurisdiction-overlays.md | 425 | journey implementation plan | yes |
| IP-journey-j133-rif-compliance-and-litigation-hold.md | 425 | journey implementation plan | yes |
| IP-journey-j135-per-jurisdiction-investigation-overlay.md | 425 | journey implementation plan | yes |
| IP-journey-j137-corporate-internal-audit-sox-controls-test-pack-overlay.md | 425 | journey implementation plan | yes |
| IP-journey-j141-internal-audit-personal-tenant-boundary-pack-overlay.md | 425 | journey implementation plan | yes |
| IP-journey-j143-dlp-scrub-bot-principal.md | 425 | journey implementation plan | yes |
| PHASE-01-EVIDENCE-PIPELINE-BOOTSTRAP.md | 84 | phase plan | yes |
| PHASE-02-PACK-OVERLAY-BREACH-DPIA-BUILD.md | 58 | phase plan | yes |
| PRD.md | 127 | product requirements | partial: strong seed but stale against later control/GRC scope |
| README.md | 39 | service index | partial: broad purpose but thin deployment/build guidance |
| backfill-replay.md | 37 | replay plan | partial: relevant but thin |
| benchmarks/drata-vanta-onetrust-auditboard-vs-oyatie.md | 126 | prior benchmark comparison | partial: useful, but does not satisfy current performance target report |
| capabilities/auditor-engagement-read.cedar | 23 | Cedar capability | yes |
| capabilities/breach-declare.cedar | 29 | Cedar capability | yes |
| capabilities/compliance-admin-upload.cedar | 22 | Cedar capability | yes |
| capabilities/dsar-subject-self-service.cedar | 28 | Cedar capability | yes |
| capabilities/pack-overlay-subscribe.cedar | 20 | Cedar capability | yes |
| capability-tiers/tier-matrix.md | 184 | tier matrix | partial: rich tiering, missing OCI demo_trial Always Free |
| capacity-model.md | 55 | capacity model | partial: thin compared with tier matrix |
| catalog/api-asyncapi.yaml | 12 | catalog component | partial: useful but minimal |
| catalog/api-rest.yaml | 12 | catalog component | partial: useful but minimal |
| catalog/auditor-portal-frontend.yaml | 14 | catalog component | partial: frontend catalog lacks allowed frontend stack details |
| catalog/component-info.yaml | 30 | Backstage catalog | yes |
| catalog/oya-compliance-breach-notification-workflow-usecase.yaml | 17 | catalog component | partial |
| catalog/oya-compliance-cell-certification-attestation-worker.yaml | 14 | catalog component | partial |
| catalog/oya-compliance-control-mapping-domain.yaml | 14 | catalog component | partial |
| catalog/oya-compliance-dpia-orchestration-usecase.yaml | 15 | catalog component | partial |
| catalog/oya-compliance-pack-registry-domain.yaml | 16 | catalog component | partial |
| catalog/oya-compliance-pack-registry-kernel.yaml | 14 | catalog component | partial |
| catalog/oya-compliance-regulator-audit-evidence-rest.yaml | 15 | catalog component | partial |
| competitor-parity-matrix.md | 64 | prior parity sketch | partial: missing current top-3 union coverage depth |
| compliance.md | 968 | regulatory mapping | partial: rich, but references missing policy JSON files |
| contracts/asyncapi.yaml | 126 | async contract | yes |
| contracts/compliance.proto | 20 | gRPC contract | partial: too thin for pack registry, DSAR, regulator, and tier surfaces |
| contracts/dsar-export-format.json | 47 | export schema | yes |
| contracts/openapi.yaml | 216 | REST contract | partial: core DSAR/evidence only; misses current expanded purpose |
| cost-budget.md | 60 | cost plan | partial: lacks per-context OCI Always Free budget split |
| dashboards/audit-chain-seal-health.json | 12 | dashboard | yes |
| dashboards/breach-notification-sla.json | 33 | dashboard | yes |
| dashboards/dsar-pipeline.json | 43 | dashboard | yes |
| dashboards/evidence-coverage.json | 12 | dashboard | yes |
| dashboards/pack-overlay-coverage.json | 27 | dashboard | yes |
| dashboards/regulator-engagement-activity.json | 33 | dashboard | yes |
| decisions/ADR-COMP-001-pack-overlay-precedence-conflict-resolution.md | 204 | service ADR | yes: strong canonical service decision |
| decisions/ADR-compliance-001-evidence-retention-policy.md | 27 | service ADR | partial: relevant but thin |
| decisions/ADR-compliance-002-dsar-sla.md | 23 | service ADR | partial: relevant but thin |
| decisions/ADR-compliance-003-auditor-access-cedar-policy.md | 18 | service ADR | partial: relevant but thin |
| decisions/ADR-compliance-004-cross-tenant-kernel-invariant.md | 23 | service ADR | partial: relevant but thin |
| decisions/ADR-compliance-005-replace-drata-vanta-with-in-house.md | 27 | service ADR | partial: relevant but thin |
| dpia.md | 72 | DPIA doc | partial: relevant but lacks OneTrust-depth privacy operations |
| failure-modes.md | 52 | failure-mode list | partial: useful but not tied to all expanded capabilities |
| faqs/compliance-engineer-faq.md | 197 | onboarding FAQ | yes |
| iac/ech-config.yaml | 17 | IaC/security config | partial: useful, not per-context OpenTofu |
| iac/edge-waf.yaml | 48 | IaC/security config | partial: useful, not per-context OpenTofu |
| iac/helm/evidence-collector/Chart.yaml | 10 | Helm chart | partial: Helm is useful after OpenTofu composition |
| iac/helm/evidence-collector/README.md | 49 | Helm chart doc | partial: not canonical context module |
| iac/helm/evidence-collector/values.yaml | 92 | Helm values | partial: not canonical context module |
| iac/k8s-deployment.yaml | 56 | Kubernetes manifest | partial: not per-context OpenTofu |
| iac/k8s-network-policy.yaml | 52 | Kubernetes manifest | partial: not per-context OpenTofu |
| iac/openbao-policy.hcl | 32 | secret policy | yes: HCL policy aligns with OpenBao |
| iac/pqc-cert.yaml | 16 | cert config | partial: not per-context OpenTofu |
| iac/secret-bindings.yaml | 34 | secret bindings | yes |
| iac/terraform-module.tf | 52 | IaC module | no: Terraform naming and block violate OpenTofu direction |
| incident-response.md | 66 | incident response | partial: useful but not per-context runbook matrix |
| manifest.json | 180 | machine-readable service manifest | partial: lacks deployment contexts, OS matrix, OCI profile |
| migration-playbooks/from-onetrust.md | 205 | migration playbook | yes: directly relevant to top-3 counterpart |
| multi-region.md | 48 | multi-region note | partial: not per-context deployment support |
| onboarding/compliance-engineer-first-week.md | 298 | onboarding | yes |
| packs/EU-AI-Act.md | 207 | compliance pack doc | yes |
| packs/GDPR.md | 207 | compliance pack doc | yes |
| packs/HIPAA.md | 207 | compliance pack doc | yes |
| packs/KR-PIPA.md | 207 | compliance pack doc | yes |
| packs/SOC2.md | 207 | compliance pack doc | yes |
| policy/abuse-defence.cedar | 57 | Cedar policy | yes |
| policy/action-authorization.cedar | 70 | Cedar policy | yes |
| policy/auditor-scope.cedar | 37 | Cedar policy | yes |
| policy/ci-scope.cedar | 26 | Cedar policy | yes |
| policy/data-residency.cedar | 32 | Cedar policy | yes |
| policy/data-residency.md | 98 | policy explainer | yes |
| policy/pack-overlay-authorization.cedar | 32 | Cedar policy | yes |
| reference-implementations/pack-publish-and-conflict-rust-sdk.md | 317 | Rust reference implementation doc | yes |
| runbooks/audit-seal-verify-failure.md | 271 | runbook | yes |
| runbooks/breach-notification-72h-clock-at-risk.md | 271 | runbook | yes |
| runbooks/certification-evidence-pipeline-stall.md | 271 | runbook | yes |
| runbooks/cross-tenant-dsar-leak-suspected.md | 271 | runbook | yes |
| runbooks/dsar-backlog-overflow.md | 271 | runbook | yes |
| runbooks/engagement-cedar-revoke-failed.md | 271 | runbook | yes |
| runbooks/evidence-collector-degraded.md | 271 | runbook | yes |
| runbooks/manual-evidence-upload-rejected.md | 271 | runbook | yes |
| runbooks/pack-overlay-conflict-resolution.md | 271 | runbook | yes |
| runbooks/phi-access-anomaly.md | 271 | runbook | yes |
| runbooks/regulator-engagement-grant-revoke.md | 271 | runbook | yes |
| runbooks/regulator-evidence-export-failure.md | 271 | runbook | yes |
| runbooks/seaweedfs-evidence-bucket-loss.md | 271 | runbook | yes |
| scorecards/gdpr.json | 16 | scorecard | partial: relevant but thin |
| scorecards/hipaa.json | 16 | scorecard | partial: relevant but thin |
| scorecards/overrides.json | 52 | scorecard overrides | yes |
| scorecards/pci-dss.json | 19 | scorecard | partial: relevant but PCI is deferred in PRD |
| scorecards/soc2-type-2.json | 19 | scorecard | yes |
| sdk-plan.md | 60 | SDK plan | partial: contains non-Rust example risk in prose |
| slos/audit-chain-seal-verify-success.openslo.yaml | 29 | OpenSLO | yes |
| slos/auditor-portal-availability.openslo.yaml | 28 | OpenSLO | yes |
| slos/auditor-portal-latency.openslo.yaml | 28 | OpenSLO | yes |
| slos/breach-notify-authority-72h.openslo.yaml | 17 | OpenSLO | yes |
| slos/cross-tenant-isolation-violations.openslo.yaml | 33 | OpenSLO | yes |
| slos/dsar-backlog-depth.openslo.yaml | 28 | OpenSLO | yes |
| slos/dsar-completion-time.openslo.yaml | 33 | OpenSLO | yes |
| slos/evidence-coverage-rollup.openslo.yaml | 28 | OpenSLO | yes |
| slos/evidence-emission-lag.openslo.yaml | 28 | OpenSLO | yes |
| slos/manual-upload-success.openslo.yaml | 28 | OpenSLO | yes |
| slos/pack-publish-soak-respected.openslo.yaml | 17 | OpenSLO | yes |
| slos/phi-anomaly-detection-fidelity.openslo.yaml | 30 | OpenSLO | yes |
| threat-model.md | 104 | threat model | partial: useful but not mapped to all expanded surfaces |
| tutorials/resolve-multi-pack-erasure-conflict.md | 245 | tutorial | yes |

## §3 9-dimension audit

### §3.1 Dimension 1 — internal coherence

D1-01 Evidence: `PRD.md` lines 16-20 define an in-house evidence pipeline, auditor relationship, and tamper-evident evidence chain.
D1-02 Evidence: `README.md` lines 18-21 expands the service to DPIA, breach notification, regulator audit evidence, cell certification, and compliance-control mapping.
D1-03 Evidence: `capability-tiers/tier-matrix.md` line 11 assigns evidence collection, SOC 2 control mapping, GDPR DSR, HIPAA logging, regulator evidence, DPIA, breach notification, pack registry, and control mapping.
D1-04 Contradiction probe: `PRD.md` line 34 says this is not a GRC platform, while `README.md` lines 18-21 and `tier-matrix.md` line 11 assign control mapping and GRC-displacement responsibilities.
D1-05 Severity: P1 because scope contradiction changes what product an implementer would build.
D1-06 Contradiction probe: `PRD.md` lines 24-28 cover SOC 2, GDPR, HIPAA, PCI; `manifest.json` lines 129-144 adds ISO27001, KR-PIPA, IL5, IL6, FedRAMP, EU AI Act, CN PIPL, and duplicate pack IDs.
D1-07 Severity: P2 because pack breadth grew without PRD refresh, but the expanded corpus is coherent with the compliance-pack doctrine.
D1-08 Contradiction probe: `PRD.md` line 116 excludes Drata/Vanta migration wizard; the service includes `migration-playbooks/from-onetrust.md`, which is not the same vendor but signals migration scope growth.
D1-09 Severity: P3 because OneTrust migration is useful for this batch, but migration ownership should be explicit.
D1-10 Broken local reference: `compliance.md` line 42 says RoPA register is at `policy/ropa.json`; that file is absent.
D1-11 Severity: P1 because GDPR Art. 30 coverage is claimed as present while the named evidence file is missing.
D1-12 Broken local reference: `compliance.md` line 83 names `policy/iso-27001-annex-a-coverage.json`; that file is absent.
D1-13 Severity: P1 because ISO mapping is claimed with a specific missing artifact.
D1-14 Broken local reference: `competitor-parity-matrix.md` line 29 cites `iso-27001-annex-a-coverage.json` as Phase 1.5 evidence; the file is absent.
D1-15 Severity: P2 because the matrix labels it future-ish, but still uses a concrete missing filename.
D1-16 Broken local reference: `IP-journey-j130-whistleblower-protection-pack.md` line 46 names `dashboards/whistleblower-pack-status.json`; the dashboard is absent.
D1-17 Severity: P2 because the journey may be planned, but the artifact table makes it look concrete.
D1-18 Internal reference resolves: `ARCHITECTURE.md` lines 45 and 99 cite `contracts/asyncapi.yaml`, `contracts/compliance.proto`, `contracts/dsar-export-format.json`, and `contracts/openapi.yaml`; all four exist.
D1-19 Internal reference resolves: `ARCHITECTURE.md` lines 46 and 100 cite Cedar policy files; the named policy files exist.
D1-20 Internal reference resolves: `ARCHITECTURE.md` lines 48 and 102 cite SLO files; the SLO directory contains the named OpenSLO documents.
D1-21 Internal reference resolves: `ARCHITECTURE.md` lines 49 and 103 cite runbooks; the runbooks directory contains the named runbooks.
D1-22 Internal reference partial: `ARCHITECTURE.md` line 85 cites deployment evidence as root-level IaC; those files exist but are not canonical per-context OpenTofu.
D1-23 Internal reference partial: `PRD.md` lines 86-90 name implementation crates and `clients/auditor-portal/`; no `src/` or client directory exists under this service.
D1-24 Severity: P2 because crate names may be planned elsewhere, but an intern cannot build from this path alone.
D1-25 Internal reference partial: `manifest.json` lines 95-110 registers IP-001 through IP-015, and those files exist.
D1-26 Internal reference partial: `manifest.json` lines 122-128 bounded contexts map to capability Cedar files, and those files exist.
D1-27 Internal reference mismatch: `manifest.json` lines 118-121 says capability tiers are `T2` and `T3`, while `capability-tiers/tier-matrix.md` defines retired customer-class names.
D1-28 Severity: P2 because tier vocabulary is not aligned between manifest and tier matrix.
D1-29 Internal scope mismatch: `PRD.md` line 27 defers PCI until payments lands, while `manifest.json` line 133 and `scorecards/pci-dss.json` keep PCI in active pack evidence.
D1-30 Severity: P2 because substrate-ready PCI is useful, but active pack wording should distinguish deferred launch.
D1-31 Internal dependency gap: `manifest.json` lines 165-174 depends on nine microservices, but no `cross-microservice-handoffs.md` exists.
D1-32 Severity: P1 because microservice ownership directive requires handoff coherence for cross-boundary dependencies.
D1-33 Internal contract gap: `contracts/openapi.yaml` lines 13-139 has DSAR and evidence endpoints only; it lacks pack registry, DPIA, breach, regulator, control mapping, and trust-center/questionnaire endpoints named elsewhere.
D1-34 Severity: P2 because expanded scope is not represented in REST.
D1-35 Internal contract gap: `contracts/compliance.proto` lines 5-20 has only `RecordEvidence`.
D1-36 Severity: P2 because gRPC cannot express the broader pack and regulator control surface.
D1-37 Internal SLO mismatch: `PRD.md` line 94 says DSAR export p99 target is 5 days and statutory cap is 30 days; `capability-tiers/tier-matrix.md` line 53 says DSAR fulfillment p99 <= 30d for demo_trial and line 119 says paid on-prem-connected p99 <= 14d.
D1-38 Severity: P2 because tier-specific SLOs are plausible but the PRD should name tier deltas.
D1-39 Internal cost mismatch: `PRD.md` line 101 says $1,500/month for a 32-service moderate fleet; tier matrix line 122 says paid dedicated-cloud is about $520k/cell/year and paid on-prem-connected about $980k/cell/year.
D1-40 Severity: P1 because cost scale changes product economics by orders of magnitude.
D1-41 Internal product mismatch: `PRD.md` line 103 compares Drata baseline at about $25k/year; top-3 batch requires OneTrust and Drata/Vanta union, not one baseline.
D1-42 Severity: P3 because prior PRD is older than this audit batch.
D1-43 Internal evidence strength: runbooks are consistently 271 lines and cover important operational failure modes.
D1-44 Internal evidence weakness: many IP-016 through IP-026 plans are 21-36 lines and do not meet intern-buildability for expanded critical surfaces.
D1-45 Internal evidence strength: `reference-implementations/pack-publish-and-conflict-rust-sdk.md` is 317 lines and aligns with Rust-strict direction.
D1-46 Internal evidence risk: `sdk-plan.md` includes non-Rust SDK discussion in docs; no forbidden source files exist, but implementation prompts must keep backend Rust-only.
D1-47 Internal inventory conclusion: the compliance corpus is broad and mostly on-purpose, but the PRD/manifest/contracts lag the current service scope.
D1-48 Internal contradiction count used in findings: 6 direct contradictions or stale-claim risks.
D1-49 Internal broken-reference count used in findings: 4 concrete missing local artifact references.
D1-50 Internal coherence decision: partial pass; remediation should refresh PRD, manifest, contracts, cost model, and missing evidence files before implementation.

### §3.2 Dimension 2 — outbound cross-references

D2-01 Outbound ADR references in `PRD.md` lines 120-125 include ADR-0131, ADR-0145, ADR-0170, ADR-0181, ADR-0183, and ADR-0209.
D2-02 Outbound standard reference in `PRD.md` line 126 cites `docs/standards/compliance-evidence-automation.md`.
D2-03 Outbound implementation reference in `PRD.md` line 127 cites `oya-shared-compliance-evidence-kernel`.
D2-04 Outbound ADR references in `ARCHITECTURE.md` lines 15-25 include ADR-0209, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0250, ADR-0251, ADR-0253, ADR-0263, ADR-0276, and ADR-0293.
D2-05 Outbound planned enforcement in `ARCHITECTURE.md` line 26 cites `oya-governance-adr-adherence-matrix`.
D2-06 Outbound cross-service dependency evidence: `manifest.json` lines 165-174 lists `audit-chain`, `tenancy`, `identity`, `observability`, `cell`, `intelligence`, `detection`, `network`, and `cloud-iac`.
D2-07 Outbound dependency concern: no local handoff document defines payload, ownership, failure semantics, or reverse commitments for those dependencies.
D2-08 Outbound policy references resolve locally for Cedar files cited by `ARCHITECTURE.md` lines 46 and 100.
D2-09 Outbound SLO references resolve locally for OpenSLO files cited by `ARCHITECTURE.md` lines 48 and 102.
D2-10 Outbound runbook references resolve locally for named runbooks cited by `ARCHITECTURE.md` lines 49 and 103.
D2-11 Outbound IaC references resolve locally for root-level files cited by `ARCHITECTURE.md` line 85, but those files are not canonical context modules.
D2-12 Reverse reference from repo registry: `registry/microservices.json` line 34 includes compliance in Wave 7 metadata.
D2-13 Reverse reference from user journey corpus: `docs/user-journeys/j108-supplier-rating-and-marketplace-discovery/story.md` references compliance attestations and a compliance overlay in marketplace discovery.
D2-14 Reverse reference from user journey corpus: `docs/user-journeys/j54-quote-to-contract-to-payment-saas/story.md` references compliance packs as the determinant for cross-region continuity.
D2-15 Reverse reference from schema corpus: `specs/capability-tier-schema.json` references compliance pack overlays as tier signals.
D2-16 Reverse reference consequence: compliance is a substrate-like service for tenant readiness, not a narrow auditor portal.
D2-17 Orphan risk: PRD says control management out of scope, but multiple external docs and service docs treat compliance packs/control mapping as cross-system substrate.
D2-18 Missing reverse reference: current `manifest.json` dependency list does not record the reverse consumers that rely on compliance pack decisions.
D2-19 Missing cross-service file: `cross-microservice-handoffs.md` is absent from the service root.
D2-20 Missing implementation-plans directory: root IP files exist, but the requested `implementation-plans/IP-*.md` path shape does not exist.
D2-21 Path-shape impact: tools expecting `implementation-plans/` will miss current IP documents unless the root flat-layout exception is codified.
D2-22 ADR-COMP-001 is a strong outbound authority for pack precedence; it should be lifted into manifest authority chain.
D2-23 ADR-0328 cross-cutting constraints are not cited inside the service docs because they were created after many service artifacts.
D2-24 The five Claude feedback files are canonical for this audit and confirm the six-context, OpenTofu, OS, Rust, and OCI doctrines.
D2-25 Chat line 10131 reinforces docs of substance, not thin scaffold.
D2-26 Chat line 14247 reinforces AWS/OCI, on-prem, colo, and Oyatie-as-cloud-provider deployment directions.
D2-27 Chat line 14334 reinforces zero-handroll OpenTofu.
D2-28 Chat line 791 reinforces compliance packs and cell certification as first-class architecture.
D2-29 Chat line 948 records a task to draft ADR-0251 around Compliance Pack and Cell Certification Levels.
D2-30 External counterpart citation: Vanta platform page lines 196-200 names compliance, risk, third-party risk, audit, trust center, and questionnaire automation.
D2-31 External counterpart citation: Vanta Trust Center page lines 174-199 names automated access, CRM/contract integration, and real-time control evidence.
D2-32 External counterpart citation: Vanta TPRM page lines 342-348 names AI-powered assessments, continuous monitoring, and actionable insights.
D2-33 External counterpart citation: Drata compliance page lines 218-224 names enterprise GRC, compliance automation, trust center, questionnaire assistance, TPRM, and frameworks.
D2-34 External counterpart citation: Drata TPRM page lines 118-150 names agentic document collection, gap flagging, criteria generation, vendor source sync, risk summaries, risk register, directory, and reporting.
D2-35 External counterpart citation: Drata Trust Center page lines 55-66 names buyer-ready trust library, controlled access, and AI-powered search.
D2-36 External counterpart citation: OneTrust products page lines 257-356 names consent, privacy operations, DSR, AI governance, compliance automation, IT risk, and third-party lifecycle.
D2-37 External counterpart citation: OneTrust DSR page lines 237-244 names secure portal and privacy automation.
D2-38 External counterpart citation: OneTrust TPRM page lines 213-254 names third-party lifecycle, inventory, assessment, monitoring, mitigation, reporting.
D2-39 External counterpart citation: OneTrust AI governance page lines 251-259 names AI inventory, ownership, lifecycle, and dependencies.
D2-40 Outbound counterpart conclusion: top-3 public surfaces create a union wider than current REST/proto and wider than old PRD.
D2-41 Outbound docs conclusion: reverse references make compliance a cross-platform policy and evidence authority.
D2-42 Orphan reference conclusion: missing local policy JSON files are immediate broken links.
D2-43 Missing reverse reference conclusion: consumers need a reverse-reference section in `cross-microservice-handoffs.md`.
D2-44 Wrong-direction reference: `ARCHITECTURE.md` lines 52-53 uses AWS IAM and Google Cloud service agents for principals, but compliance product counterparts should be Vanta/Drata/OneTrust for product parity.
D2-45 Severity: P3 for wrong comparison direction, because architecture precedent and product parity are different lenses.
D2-46 Wrong-direction reference: README line 34 lists AWS Audit Manager as a hyperscaler precedent; useful, but not part of this batch top-3 bar.
D2-47 Severity: P3 because AWS Audit Manager can stay as precedent but not substitute for OneTrust.
D2-48 Outbound audit verdict: references are numerous and mostly meaningful, but cross-service handoff and reverse-reference mechanics are under-specified.
D2-49 Required remediation: add a handoff artifact that maps dependencies to APIs, events, data classes, failure semantics, and reverse consumers.
D2-50 Required remediation: add top-3 counterpart source anchors to parity and tier docs so product comparisons do not drift.

### §3.3 Dimension 3 — substance bar (intern-buildability)

D3-01 A cold intern can understand the intended value proposition from `PRD.md` lines 16-20.
D3-02 A cold intern can identify users and jobs from `PRD.md` lines 38-47.
D3-03 A cold intern can see initial REST operations from `PRD.md` lines 60-70 and `contracts/openapi.yaml` lines 13-139.
D3-04 A cold intern can see initial success metrics from `PRD.md` lines 48-59.
D3-05 A cold intern cannot build the service because no `src/` directory exists under `microservices/compliance/`.
D3-06 A cold intern cannot run unit tests because no `tests/` directory exists under `microservices/compliance/`.
D3-07 A cold intern cannot follow per-context deployment because no canonical `iac/<context>/` directories exist.
D3-08 A cold intern cannot validate OS support because no `supported-oses.json` exists.
D3-09 A cold intern cannot compile per ADR-0328 §D-20.104 because no service-local Cargo package or workspace target is declared here.
D3-10 A cold intern can read policy intent because Cedar files exist under `policy/` and `capabilities/`.
D3-11 A cold intern can inspect operational procedures because 13 runbooks exist and are each 271 lines.
D3-12 A cold intern can inspect SLO intent because 12 OpenSLO files exist.
D3-13 A cold intern cannot generate a regulator export from docs alone because `contracts/openapi.yaml` lacks regulator export endpoints.
D3-14 A cold intern cannot build pack registry APIs from `contracts/compliance.proto`, which only records evidence.
D3-15 A cold intern cannot implement OneTrust-level DSR flow because current OpenAPI does not model identity verification, legal hold checks, redaction, secure portal, or consent integration.
D3-16 A cold intern cannot implement Vanta/Drata questionnaire automation because no questionnaire, knowledge-base, or trust-answer surface is present in contracts.
D3-17 A cold intern cannot implement Vanta/Drata TPRM because no vendor inventory, assessment, monitoring, residual-risk, or evidence request endpoints are present.
D3-18 A cold intern cannot reconcile cost because `PRD.md` line 101 and `tier-matrix.md` line 122 disagree on scale.
D3-19 A cold intern cannot know launch scope because PRD Phase 1 defers or excludes some features now present in tier and pack docs.
D3-20 A cold intern can use `reference-implementations/pack-publish-and-conflict-rust-sdk.md` as an implementation model for pack publish and conflict resolution.
D3-21 Weak section: `IP-023-pack-registry-grpc.md` is 21 lines and too thin for production gRPC implementation.
D3-22 Weak section: `IP-024-dpia-orchestration-adapter-postgres.md` is 23 lines and too thin for database schema and migration semantics.
D3-23 Weak section: `IP-026-control-mapping-rest-and-sdk.md` is 28 lines and too thin for REST/SDK implementation.
D3-24 Weak section: `decisions/ADR-compliance-003-auditor-access-cedar-policy.md` is 18 lines and too thin for auditor access invariants.
D3-25 Weak section: `contracts/compliance.proto` is 20 lines and too thin for a service with pack, DSAR, breach, regulator, and auditor domains.
D3-26 Missing data model: no explicit relational schema file exists for packs, controls, evidence, DSAR, breach clocks, regulator access, trust center, questionnaire answers, or third-party vendors.
D3-27 Missing API model: no endpoint for pack publish, pack activation, hotfix, effective-policy projection, conflict report, or legal hold.
D3-28 Missing API model: no endpoint for trust-center public/gated resources, access approval, NDA collection, or analytics.
D3-29 Missing API model: no endpoint for questionnaire import, approved-answer reuse, reviewer assignment, or buyer-answer export.
D3-30 Missing API model: no endpoint for vendor intake, inherent risk rubric, assessment rules, risk findings, residual-risk approval, or reassessment schedule.
D3-31 Missing failure semantics: runbooks exist, but contracts do not define error codes for pack conflict, legal hold denial, DSR redaction failure, or regulator export refusal.
D3-32 Missing CI lane spec: no CI matrix lists Tier-1 OS, OpenTofu plan validation, Rust build invocation, or contract checks for this service.
D3-33 Missing IaC lane spec: no `tofu init`, `tofu plan`, `tofu apply`, cosign module verification, or state-backend evidence per context.
D3-34 Missing SDK boundary: `sdk-plan.md` exists, but no generated Rust crate path or language exception record exists.
D3-35 Missing storage boundary: SeaweedFS, PostgreSQL/Citus, Kafka, Valkey, and ClickHouse appear in tier docs, but no schema and retention model ties them together.
D3-36 Missing tenant onboarding path: no tenant variables per ADR-0328 §D-15 are documented in service IaC.
D3-37 Missing per-tenant proof: no audit event class list maps all public operations to tamper-evident records.
D3-38 Missing privacy proof: `dpia.md` exists, but OneTrust-like consent, notice, RoPA, data map, TIA, and privacy incident flows are incomplete.
D3-39 Missing third-party proof: no vendor-risk register equivalent exists despite Vanta/Drata/OneTrust parity pressure.
D3-40 Missing AI governance proof: EU AI Act packs exist, but no AI inventory, ownership, model/dataset/agent lifecycle, or monitoring surface exists.
D3-41 Missing deployment proof: no context support matrix in manifest.
D3-42 Missing OS proof: no package formats for RPM, DEB, pkg/Homebrew, Talos extension, Flatcar extension, Photon package, or container image.
D3-43 Missing source proof: no Rust modules under service path.
D3-44 Missing test proof: no test files under service path.
D3-45 Positive substance: long journey IPs contain detailed jurisdictional coverage and should be mined into canonical contracts.
D3-46 Positive substance: runbooks are operationally substantive and cover DSAR, breach, regulator, phi anomaly, and pack conflict risks.
D3-47 Positive substance: Cedar files give real authorization boundary material.
D3-48 Positive substance: OpenAPI 3.2.0 is current and useful for the initial DSAR/evidence slice.
D3-49 Intern-buildability verdict: partial fail; an intern can understand the service, but cannot build or deploy it from current local artifacts.
D3-50 Remediation: add service-local Rust crate map, schemas, expanded contracts, context IaC modules, OS manifest, CI matrix, and missing policy evidence files.

### §3.4 Dimension 4 — canonical-direction alignment

D4-01 Multi-context constraint: ADR-0328 §D-15 and master plan lines 704-746 require six deployment contexts.
D4-02 Compliance status: drifted-fixable because no deployment-context manifest field exists and no per-context IaC directories exist.
D4-03 Evidence: `manifest.json` lines 1-180 has no `deployment_contexts` or equivalent support declaration.
D4-04 Evidence: `iac/` contains only root-level files and Helm, not `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-iaas/`.
D4-05 Severity: P1 because all six contexts are in scope unless explicitly justified otherwise.
D4-06 OpenTofu constraint: ADR-0328 §D-16 and master plan lines 747-776 require OpenTofu and forbid Terraform/Pulumi/CloudFormation.
D4-07 Compliance status: incoherent because `iac/terraform-module.tf` line 1 names Terraform and line 5 uses a `terraform` block.
D4-08 Evidence: `AUDIT-FINDINGS-2026-05-20.json` line 37 includes “terraform” in the IaC roster.
D4-09 Evidence: `CHANGELOG.md` line 24 mentions a terraform module.
D4-10 Severity: P1 because Terraform spelling and binary direction are explicitly superseded.
D4-11 OpenTofu forbidden patterns scan: no `null_resource`, `local-exec`, `remote-exec`, `tfstate`, Pulumi, or CloudFormation hits were found in compliance path.
D4-12 OpenTofu signing gap: no sigstore/cosign module signing wiring exists in service IaC.
D4-13 OpenTofu state gap: no state backend per context is declared.
D4-14 OpenTofu tenant gap: no per-tenant `tofu plan`/`tofu apply` variables exist in service docs.
D4-15 OS constraint: ADR-0328 §D-17 and master plan lines 777-816 require 13 Tier-1 OSes, two Tier-2 test-only architectures, and out-of-scope declarations.
D4-16 Compliance status: drifted-fixable because no `supported-oses.json` exists.
D4-17 Evidence: recursive file inventory contains no `supported-oses.json`.
D4-18 Severity: P2 because absence of manifest/package/CI lane is explicitly P2 for non-P0 service, unless deployment docs claim support without proof.
D4-19 Rust-strict constraint: ADR-0328 §D-18 and master plan lines 817-856 require Rust backend and forbid Python/JS/TS/Ruby/Go/Java/Scala/Groovy/PHP/F# unless exception exists.
D4-20 Compliance status: aligned at file-extension level because no forbidden source files were found under service path.
D4-21 Evidence: forbidden extension scan returned zero files.
D4-22 Documentation risk: `sdk-plan.md` contains non-Rust SDK planning, but no source file; keep implementation backend Rust.
D4-23 Build invocation gap: no local crate documents `cargo build --workspace --release --all-features --locked`.
D4-24 Frontend allowance: no `frontend/ios`, `frontend/android`, or `frontend/windows` code exists, so frontend language exception is not currently exercised.
D4-25 OCI Always Free constraint: ADR-0328 §D-19 and master plan lines 857-868 require `iac/oci-guest/always-free/` and demo_trial-on-OCI Always Free reconciliation.
D4-26 Compliance status: drifted-fixable because no `iac/oci-guest/always-free/` exists.
D4-27 Evidence: `capability-tiers/tier-matrix.md` lines 15-56 defines demo_trial with three 8-vCPU/32GiB nodes, not OCI Always Free.
D4-28 Severity: P1 because demo_trial-on-OCI cannot be claimed with the current hardware envelope.
D4-29 Documentation substance constraint: brief-template lines 169-187 say line count is not quality; service has many large journey IPs but some thin core IP/ADR files.
D4-30 Compliance status: partial; broad corpus is substantive, but core machine-readable deployability is missing.
D4-31 Microservice ownership directive: memory file requires one owner to read full service path and chat history; this audit did that.
D4-32 Verification directive: memory file requires deliverables be verified beyond line count; this report will use line count plus content scans.
D4-33 Docs-substance directive: memory file rejects scaffold; this audit identifies generated repetition in `ARCHITECTURE.md` but also uses concrete evidence.
D4-34 Chat directive alignment: line 14247 confirms all deployment shapes; service docs do not satisfy it.
D4-35 Chat directive alignment: line 14334 confirms OpenTofu; service IaC does not satisfy it.
D4-36 Chat directive alignment: line 10131 demands substance; service docs are uneven.
D4-37 Chat directive alignment: line 791 confirms compliance packs; service tier/IP docs align strongly.
D4-38 Chat directive alignment: line 948 records Compliance Pack + Cell Certification ADR task; service `ARCHITECTURE.md` lines 20-21 cites ADR-0251.
D4-39 Canonical-direction verdict for multi-context: drifted-fixable/P1.
D4-40 Canonical-direction verdict for OpenTofu: incoherent/P1.
D4-41 Canonical-direction verdict for OS support: drifted-fixable/P2.
D4-42 Canonical-direction verdict for Rust-strict file scan: aligned/P3 build invocation gap.
D4-43 Canonical-direction verdict for OCI Always Free: drifted-fixable/P1.
D4-44 Canonical-direction verdict for substance: partial/P2.
D4-45 Canonical-direction verdict for ownership coherence: this audit satisfies the process, but service artifacts need remediation.
D4-46 Canonical fix order: first service manifest fields, then per-context IaC, then OS manifest, then contract expansion.
D4-47 Canonical fix risk: editing IaC before purpose/contracts are refreshed could bake stale product scope into deployment.
D4-48 Canonical fix evidence: every remediation should cite ADR-0328 §D-15..§D-20 and master-plan fields.
D4-49 Canonical residual risk: compliance is important enough that missing context deployability blocks honest hyperscaler maturity claims.
D4-50 Canonical decision: not ready for Wave 14 green aggregation without P1 remediation tickets.

### §3.5 Dimension 5 — industry-counterpart parity

D5-01 Top-3 counterpart set for this batch: Vanta, Drata, OneTrust.
D5-02 Vanta headline surface from public docs: compliance automation, risk, third-party risk, audit prep, trust center, questionnaire automation.
D5-03 Vanta evidence: Vanta homepage lines 196-200 lists those platform categories.
D5-04 Vanta Trust Center evidence: Vanta page lines 174-199 describes gated document access, CRM/contract integrations, continuous controls, and real-time evidence.
D5-05 Vanta questionnaire evidence: Vanta help lines 33-41 describes AI-assisted questionnaire automation, past questionnaire import, document upload, and policy sync.
D5-06 Vanta integration evidence: Vanta help lines 31-36 describes continuous evidence collection, control monitoring, and real-time posture.
D5-07 Vanta access review evidence: Vanta help lines 62-76 and 124-132 describe access review creation and user data imports.
D5-08 Vanta TPRM evidence: Vanta TPRM page lines 342-348 lists AI-powered assessments, continuous monitoring, and actionable insights.
D5-09 Drata headline surface from public docs: enterprise GRC, compliance automation, trust center, AI questionnaire assistance, third-party risk management, integrated risk.
D5-10 Drata compliance evidence: Drata compliance page lines 218-224 lists products and frameworks.
D5-11 Drata TPRM evidence: Drata page lines 118-150 names document collection, gap flagging, AI criteria generation, vendor sync, risk summaries, risk register, directory, and reporting.
D5-12 Drata Trust Center evidence: Drata page lines 55-66 names trust library, controlled access, self-service, and AI-powered search.
D5-13 Drata risk evidence: Drata internal-risk page lines 34-38 names risk register, owners, remediation status, and audit-ready reporting.
D5-14 OneTrust headline surface from public docs: consent/preferences, privacy automation, DSR, DataGuidance, AI governance, compliance automation, IT risk, and third-party management.
D5-15 OneTrust product evidence: OneTrust products page lines 257-356 lists consent, privacy operations, DSR, compliance automation, IT risk, and third-party lifecycle.
D5-16 OneTrust DSR evidence: OneTrust DSR page lines 237-244 describes secure portal and privacy automation.
D5-17 OneTrust TPRM evidence: OneTrust TPRM page lines 213-254 describes third-party lifecycle, inventory, assessments, monitoring, mitigation, reporting.
D5-18 OneTrust AI evidence: OneTrust AI governance page lines 251-259 describes AI inventory, ownership, lifecycle, and dependencies.
D5-19 Oyatie present capability: tamper-evident evidence pipeline in `PRD.md` lines 16-20.
D5-20 Oyatie present capability: DSAR export/delete/rectify/status in `PRD.md` lines 64-67 and OpenAPI lines 14-92.
D5-21 Oyatie present capability: evidence coverage/artifact/manual upload in OpenAPI lines 93-139.
D5-22 Oyatie present capability: pack overlays and deterministic precedence in tier matrix lines 27-39 and 150-155.
D5-23 Oyatie present capability: breach notification pack and runbook coverage.
D5-24 Oyatie present capability: regulator portal/evidence through tier matrix lines 68 and 100.
D5-25 Oyatie present capability: DPIA orchestration in tier matrix line 70 and `dpia.md`.
D5-26 Oyatie present capability: audit-chain seal on decisions in tier matrix line 181.
D5-27 Oyatie present capability: per-pack legal hold in tier matrix line 72.
D5-28 Oyatie present capability: EU AI Act pack and pipeline in tier matrix lines 104-105 and 135.
D5-29 Gap: Vanta/Drata trust-center buyer workflow is broader than Oyatie auditor portal; Oyatie lacks public/gated trust-center API, NDA automation, access approval analytics, CRM integration.
D5-30 Gap: Vanta/Drata questionnaire automation is absent from current contracts and PRD.
D5-31 Gap: Vanta/Drata TPRM vendor inventory, assessment, residual risk, and continuous monitoring are absent.
D5-32 Gap: Vanta/Drata internal risk register, risk treatment, Jira/task remediation, and risk posture dashboard are absent.
D5-33 Gap: OneTrust consent/preferences and CMP are absent from contracts and docs except generic consent references.
D5-34 Gap: OneTrust privacy operations data map, RoPA, transfer impact, privacy notices, and personal-data discovery are incomplete.
D5-35 Gap: OneTrust AI governance inventory/lifecycle is not represented except EU AI Act pack references.
D5-36 Gap: OneTrust third-party due diligence and risk exchange equivalent is absent.
D5-37 Gap: access reviews are included as evidence artifacts but not modeled as a user-facing access-review workflow.
D5-38 Gap: policy template drafting and approval workflows are not first-class.
D5-39 Gap: integrations catalog for evidence collection is not specified.
D5-40 Gap: customer-assurance analytics and revenue influence metrics are absent.
D5-41 Additive Oyatie surface: compliance packs are versioned, signed, and Cedar-evaluated across platform cell placement.
D5-42 Additive Oyatie surface: higher-restriction-wins conflict resolution is deterministic and audit-chain sealed.
D5-43 Additive Oyatie surface: cell certification attestation ties compliance packs to placement eligibility.
D5-44 Additive Oyatie surface: per-pack regulator-attested publishing is stronger than typical SaaS trust-center sharing.
D5-45 Additive Oyatie surface: cross-jurisdictional transfer evidence appears stronger than Vanta/Drata surface and overlaps OneTrust.
D5-46 Additive Oyatie surface: sovereign-pack residency and air-gap tiering go beyond public SMB compliance tools.
D5-47 Headline finding: partial union coverage.
D5-48 Reason: Oyatie is ahead on platform-native pack enforcement but behind on customer-trust, questionnaire, TPRM, consent, and risk-management product surfaces.
D5-49 Severity: P1 for parity gap because batch explicitly requires top-3 union coverage.
D5-50 Remediation: add trust-center, questionnaire, third-party risk, consent/privacy operations, and AI governance capability slices or explicitly delegate them with handoffs.

### §3.6 Dimension 6 — multi-context deployment support

D6-01 Context: `oyatie-public-cloud`; support status: not proven.
D6-02 Evidence: no `iac/oyatie-public-cloud/` directory exists.
D6-03 Required content: namespace, workload, storage, event, policy, secret, and audit-chain wiring for Oyatie-managed public cloud.
D6-04 Current substitute: root-level Kubernetes/Helm files exist but do not carry context identity.
D6-05 Verdict: missing IaC, P1.
D6-06 Context: `guest-on-aws`; support status: not proven.
D6-07 Evidence: no `iac/guest-on-aws/` directory exists.
D6-08 Current anti-pattern risk: `iac/terraform-module.tf` defaults regions to AWS-style `us-east-1`, `eu-central-1`, and `ap-northeast-2` at line 23 without context abstraction.
D6-09 Required content: AWS provider module composed through OpenTofu and cloud-iac, with portable state backend.
D6-10 Verdict: missing IaC, P1.
D6-11 Context: `guest-on-oci`; support status: not proven.
D6-12 Evidence: no `iac/oci-guest/` directory exists.
D6-13 Evidence: no `iac/oci-guest/always-free/` directory exists.
D6-14 Required content: OCI provider module plus Always Free profile.
D6-15 Verdict: missing IaC and missing Always Free subprofile, P1.
D6-16 Context: `on-prem`; support status: not proven.
D6-17 Evidence: no `iac/on-prem/` directory exists.
D6-18 Required content: bare-metal/Kubernetes substrate inputs, local storage, local secrets, local audit-chain endpoints.
D6-19 Verdict: missing IaC, P1.
D6-20 Context: `colo`; support status: not proven.
D6-21 Evidence: no `iac/colo/` directory exists.
D6-22 Required content: colo provider networking, rack/cell placement, HA storage assumptions, offline regulator evidence path.
D6-23 Verdict: missing IaC, P1.
D6-24 Context: `oyatie-as-cloud-provider`; support status: not proven.
D6-25 Evidence: no `iac/oyatie-iaas/` directory exists.
D6-26 Required content: modules using Oyatie cloud-* providers and tenant/customer isolation boundaries.
D6-27 Verdict: missing IaC, P1.
D6-28 Manifest gap: `manifest.json` lines 1-180 has no supported context declaration.
D6-29 Cost gap: `cost-budget.md` does not split costs by six contexts.
D6-30 Capacity gap: `capacity-model.md` does not split capacity by six contexts.
D6-31 SLO gap: SLO files do not annotate context-specific targets.
D6-32 Runtime placement gap: no file states Kubernetes Pods/controllers as default runtime placement for server workloads within each context.
D6-33 Cloud-vendor API grep: no direct AWS/OCI SDK source files exist because no source files exist.
D6-34 Cloud-vendor API risk: HCL module uses HashiCorp providers directly and AWS-region defaults without canonical context layer.
D6-35 Tenant onboarding gap: no per-context tenant variables required by ADR-0328 §D-15.
D6-36 Admission gap: no per-context admission-gate artifacts.
D6-37 CI gap: no per-context plan/apply verification lanes.
D6-38 State gap: no per-context state backend.
D6-39 Signing gap: no context module signature verification.
D6-40 Documentation gap: README does not tell operators which contexts are supported.
D6-41 N/A analysis: no context is correctly N/A because compliance is a platform service consumed by managed, guest, on-prem, colo, and provider-mode tenants.
D6-42 demo_trial placement issue: current demo_trial requires 3x 8-vCPU nodes, incompatible with OCI Always Free as stated.
D6-43 Sovereignty issue: paid compliance_pack air-gap claims require on-prem/colo/provider-mode proof, absent.
D6-44 Regulator issue: regulator export and pack residency need per-context evidence location, absent.
D6-45 Cross-cell issue: multi-region notes exist but not context-specific.
D6-46 Security issue: OpenBao policies exist but not context-specific secret backends.
D6-47 Observability issue: dashboards exist but not context-specific telemetry endpoints.
D6-48 Multi-context verdict: fail/P1 across all six contexts.
D6-49 Remediation: add six context directories or a service-specific justified N/A matrix with cloud-iac ownership.
D6-50 Stop condition for green: every context has OpenTofu module files, signed plan evidence, state backend, tenant variables, and CI lane reference.

### §3.7 Dimension 7 — OpenTofu IaC coverage

D7-01 Required canonical engine: OpenTofu, not Terraform.
D7-02 Current IaC inventory: `iac/ech-config.yaml`.
D7-03 Current IaC inventory: `iac/edge-waf.yaml`.
D7-04 Current IaC inventory: `iac/helm/evidence-collector/Chart.yaml`.
D7-05 Current IaC inventory: `iac/helm/evidence-collector/README.md`.
D7-06 Current IaC inventory: `iac/helm/evidence-collector/values.yaml`.
D7-07 Current IaC inventory: `iac/k8s-deployment.yaml`.
D7-08 Current IaC inventory: `iac/k8s-network-policy.yaml`.
D7-09 Current IaC inventory: `iac/openbao-policy.hcl`.
D7-10 Current IaC inventory: `iac/pqc-cert.yaml`.
D7-11 Current IaC inventory: `iac/secret-bindings.yaml`.
D7-12 Current IaC inventory: `iac/terraform-module.tf`.
D7-13 Missing directory: `iac/oyatie-public-cloud/`.
D7-14 Missing directory: `iac/guest-on-aws/`.
D7-15 Missing directory: `iac/oci-guest/`.
D7-16 Missing directory: `iac/oci-guest/always-free/`.
D7-17 Missing directory: `iac/on-prem/`.
D7-18 Missing directory: `iac/colo/`.
D7-19 Missing directory: `iac/oyatie-iaas/`.
D7-20 Required module file missing per context: `main.tf`.
D7-21 Required module file missing per context: `variables.tf`.
D7-22 Required module file missing per context: `outputs.tf`.
D7-23 Required module file missing per context: `versions.tf`.
D7-24 Required module file missing per context: `README.md`.
D7-25 Terraform reference found: `iac/terraform-module.tf` line 1 names a Terraform module.
D7-26 Terraform reference found: `iac/terraform-module.tf` line 5 uses `terraform {`.
D7-27 Terraform reference found: `AUDIT-FINDINGS-2026-05-20.json` line 37 includes terraform in IaC roster.
D7-28 Terraform reference found: `CHANGELOG.md` line 24 mentions terraform module.
D7-29 Pulumi reference scan: none in service path.
D7-30 CloudFormation reference scan: none in service path.
D7-31 Forbidden pattern scan: no `null_resource` found.
D7-32 Forbidden pattern scan: no `local-exec` found.
D7-33 Forbidden pattern scan: no `remote-exec` found.
D7-34 Forbidden pattern scan: no hand-edited `tfstate` found.
D7-35 Forbidden pattern scan: no SSH provisioner found.
D7-36 Unsigned module gap: no sigstore/cosign evidence for service OpenTofu modules.
D7-37 State backend gap: no context-specific state backend documented.
D7-38 Tenant onboarding gap: no `tofu init`, `tofu plan`, or `tofu apply` command path.
D7-39 Provider lock risk: `iac/terraform-module.tf` uses HashiCorp Kubernetes and Helm providers at lines 8-9 without OpenTofu registry/signing notes.
D7-40 OpenBao provider line 10 is useful but still inside Terraform-named module.
D7-41 Helm chart can remain as a lower-level deployment artifact after OpenTofu composes it.
D7-42 Kubernetes manifests can remain as rendered artifacts, but cannot substitute for context modules.
D7-43 ECH/WAF/PQC configs are useful security surfaces, but not complete infrastructure.
D7-44 OpenTofu severity: P1 due canonical explicitness.
D7-45 Documentation severity: P2 for missing module README and state backend descriptions.
D7-46 Supply-chain severity: P2 for missing sigstore verification wiring.
D7-47 Tenant-launch severity: P1 because zero-handroll onboarding is not present.
D7-48 IaC verdict: fail.
D7-49 Remediation: replace Terraform-named module with OpenTofu context modules and cloud-iac-owned state/signing.
D7-50 Stop condition for green: no `Terraform` references except historical rejected notes, all six context modules present, no forbidden provisioners, cosign and state backend documented.

### §3.8 Dimension 8 — OS support matrix

D8-01 Required manifest: `supported-oses.json` or equivalent supported_oses field.
D8-02 Current status: absent.
D8-03 Tier-1 Talos Linux: not declared.
D8-04 Tier-1 RHEL: not declared.
D8-05 Tier-1 Oracle Linux: not declared.
D8-06 Tier-1 SUSE Linux Enterprise Server: not declared.
D8-07 Tier-1 Ubuntu LTS: not declared.
D8-08 Tier-1 Debian: not declared.
D8-09 Tier-1 Rocky Linux: not declared.
D8-10 Tier-1 AlmaLinux: not declared.
D8-11 Tier-1 CentOS Stream: not declared.
D8-12 Tier-1 Amazon Linux: not declared.
D8-13 Tier-1 Flatcar Container Linux: not declared.
D8-14 Tier-1 VMware Photon OS: not declared.
D8-15 Tier-1 macOS Apple Silicon M5+: not declared.
D8-16 Tier-2 ppc64le: not declared as test-only.
D8-17 Tier-2 s390x: not declared as test-only.
D8-18 Out-of-scope Intel macOS: not explicitly denied.
D8-19 Out-of-scope pre-M5 Apple Silicon: not explicitly denied.
D8-20 Out-of-scope FreeBSD: not explicitly denied.
D8-21 Out-of-scope OpenBSD: not explicitly denied.
D8-22 Out-of-scope Windows Server: not explicitly denied.
D8-23 Out-of-scope Solaris: not explicitly denied.
D8-24 Package format RPM: not declared.
D8-25 Package format DEB: not declared.
D8-26 Package format macOS `.pkg`: not declared.
D8-27 Package format Homebrew: not declared.
D8-28 Package format Talos extension: not declared.
D8-29 Package format Flatcar extension: not declared.
D8-30 Package format container image: implied by Kubernetes files but not OS-matrixed.
D8-31 CI lane Talos: absent.
D8-32 CI lane RHEL: absent.
D8-33 CI lane Oracle Linux: absent.
D8-34 CI lane SLES: absent.
D8-35 CI lane Ubuntu LTS: absent.
D8-36 CI lane Debian: absent.
D8-37 CI lane Rocky: absent.
D8-38 CI lane Alma: absent.
D8-39 CI lane CentOS Stream: absent.
D8-40 CI lane Amazon Linux: absent.
D8-41 CI lane Flatcar: absent.
D8-42 CI lane Photon: absent.
D8-43 CI lane macOS M5+: absent.
D8-44 Architecture coverage x86_64: not declared.
D8-45 Architecture coverage aarch64: not declared.
D8-46 Architecture coverage ppc64le/s390x test-only: not declared.
D8-47 Service runtime likely containerized, but ADR-0328 requires explicit per-service OS support proof.
D8-48 Severity: P2 for missing OS manifest and package docs.
D8-49 Risk: without OS matrix, on-prem/colo and customer self-managed installs cannot be honestly supported.
D8-50 Remediation: add `supported-oses.json`, package formats, and CI lane references before claiming cross-context deployability.

### §3.9 Dimension 9 — Rust-strict language coverage

D9-01 Forbidden extension scan included `.py`.
D9-02 Forbidden extension scan included `.js`.
D9-03 Forbidden extension scan included `.ts`.
D9-04 Forbidden extension scan included `.tsx`.
D9-05 Forbidden extension scan included `.rb`.
D9-06 Forbidden extension scan included `.go`.
D9-07 Forbidden extension scan included `.java`.
D9-08 Forbidden extension scan included `.scala`.
D9-09 Forbidden extension scan included `.groovy`.
D9-10 Forbidden extension scan included `.php`.
D9-11 Forbidden extension scan included `.fs`.
D9-12 Forbidden extension scan included `.fsx`.
D9-13 Additional frontend-allowance scan included `.cs`.
D9-14 Additional frontend-allowance scan included `.kt`.
D9-15 Additional frontend-allowance scan included `.swift`.
D9-16 Result: zero matching forbidden source files under `microservices/compliance/`.
D9-17 Authorized file type present: `.md` documentation.
D9-18 Authorized file type present: `.yaml` and `.openslo.yaml`.
D9-19 Authorized file type present: `.json`.
D9-20 Authorized file type present: `.proto`.
D9-21 Authorized file type present: `.cedar`.
D9-22 Authorized file type present: `.hcl` OpenBao policy.
D9-23 Authorized file type present: `.tf` HCL syntax, but canonical naming must be OpenTofu rather than Terraform.
D9-24 No generated SDK output directories were found.
D9-25 No Rust source files were found either; this is not a language violation but is a buildability gap.
D9-26 No frontend path exists under `frontend/ios`.
D9-27 No frontend path exists under `frontend/android`.
D9-28 No frontend path exists under `frontend/windows`.
D9-29 No Swift/Kotlin/WinUI3 code is therefore present.
D9-30 OpenAPI file uses `openapi: 3.2.0` at `contracts/openapi.yaml` line 1, aligned with documentation-rigor.
D9-31 Proto file uses `proto3` at `contracts/compliance.proto` line 1, aligned with protocol allowance.
D9-32 AsyncAPI file exists and is authorized YAML.
D9-33 OpenSLO files exist and are authorized YAML.
D9-34 Cedar files exist and are authorized policy files.
D9-35 `sdk-plan.md` contains non-Rust SDK concepts in prose; this is documentation scope, not source code.
D9-36 Rust build invocation required: `cargo build --workspace --release --all-features --locked`.
D9-37 Current service docs do not provide that invocation tied to a compliance crate.
D9-38 Current service docs do not identify a Rust package name in Cargo metadata under this path.
D9-39 Current service docs do not include `Cargo.lock` evidence under this path.
D9-40 Current service docs do not define Rust codegen for OpenAPI/AsyncAPI/proto.
D9-41 Current service docs do not define Rust-only validation tooling for contracts.
D9-42 Current service docs do not define Rust-only migration tooling for schemas.
D9-43 Language verdict: source-file scan aligned.
D9-44 Build-system verdict: incomplete.
D9-45 IaC language nuance: HCL is allowed; Terraform product/binary references are not.
D9-46 Frontend verdict: no unauthorized frontend code.
D9-47 Severity: P3 for Rust source absence if service is not in implementation phase; P2 if Wave 14 expects buildable artifact.
D9-48 Severity: P1 would apply if forbidden source files appear later without ADR exception.
D9-49 Remediation: add Rust crate plan, Cargo build target, codegen commands, and Rust-only validation lane.
D9-50 Stop condition for green: zero forbidden source files, Rust crate present, canonical cargo build documented, and any frontend code confined to allowed platform paths.

## §4 Findings summary

| Severity | Dimension | Short description | Citation | Remediation hint |
|---|---|---|---|---|
| P1 | 1 | PRD says not GRC while later docs assign control mapping and broad GRC displacement | `PRD.md:31-36`; `README.md:18-21`; `tier-matrix.md:11` | Refresh PRD purpose and non-goals around pack/control ownership |
| P1 | 1 | Claimed RoPA register file missing | `compliance.md:42` | Add `policy/ropa.json` or downgrade claim |
| P1 | 1 | Claimed ISO coverage file missing | `compliance.md:83` | Add `policy/iso-27001-annex-a-coverage.json` or downgrade claim |
| P1 | 1 | Cost model contradicts tier matrix by orders of magnitude | `PRD.md:101`; `tier-matrix.md:122` | Reconcile fleet/month vs cell/year economics |
| P1 | 2 | Cross-service dependencies have no handoff artifact | `manifest.json:165-174`; inventory absence | Add `cross-microservice-handoffs.md` |
| P1 | 4 | Six deployment contexts not declared or implemented | ADR-0328 §D-15; `manifest.json:1-180`; `iac/` inventory | Add context manifest and six OpenTofu modules |
| P1 | 4 | Terraform references violate OpenTofu doctrine | `iac/terraform-module.tf:1`; `iac/terraform-module.tf:5` | Rename/rewrite as OpenTofu module set |
| P1 | 4 | OCI Always Free profile absent and demo_trial oversized | ADR-0328 §D-19; `tier-matrix.md:15-56` | Add `iac/oci-guest/always-free/` and demo_trial profile |
| P1 | 5 | Top-3 union parity is partial | Vanta, Drata, OneTrust public source lines cited in §3.5 | Add trust center, questionnaire, TPRM, consent, risk, AI governance slices |
| P1 | 6 | `oyatie-public-cloud` deployment not proven | missing `iac/oyatie-public-cloud/` | Add context module |
| P1 | 6 | `guest-on-aws` deployment not proven | missing `iac/guest-on-aws/` | Add context module |
| P1 | 6 | `guest-on-oci` deployment not proven | missing `iac/oci-guest/` | Add context module |
| P1 | 6 | `on-prem` deployment not proven | missing `iac/on-prem/` | Add context module |
| P1 | 6 | `colo` deployment not proven | missing `iac/colo/` | Add context module |
| P1 | 6 | `oyatie-as-cloud-provider` deployment not proven | missing `iac/oyatie-iaas/` | Add context module |
| P1 | 7 | OpenTofu tenant onboarding absent | ADR-0328 §D-16; IaC inventory | Add `tofu` plan/apply flow through cloud-iac |
| P2 | 1 | Tier vocabulary mismatch `T2/T3` vs customer-class ladder | `manifest.json:118-121`; `tier-matrix.md:15-146` | Align tier vocabulary |
| P2 | 1 | PCI deferred in PRD but active in manifest/scorecards | `PRD.md:27`; `manifest.json:133`; `scorecards/pci-dss.json` | Mark PCI substrate-ready vs launched |
| P2 | 1 | OpenAPI misses expanded domains | `contracts/openapi.yaml:13-139`; `README.md:18-21` | Add pack, DPIA, breach, regulator, trust endpoints |
| P2 | 1 | Proto too thin for service scope | `contracts/compliance.proto:5-20` | Add service definitions for pack/evidence/regulator domains |
| P2 | 1 | Whistleblower dashboard reference missing | `IP-journey-j130-whistleblower-protection-pack.md:46` | Add dashboard or mark planned |
| P2 | 3 | No service-local source tree | inventory absence | Add Rust crate or declare external crate ownership |
| P2 | 3 | No service-local tests | inventory absence | Add tests or CI references |
| P2 | 3 | Core IPs and service ADRs are thin | IP-016..IP-026 line counts; ADR-compliance line counts | Expand intern-buildable plans |
| P2 | 4 | `supported-oses.json` absent | inventory absence; ADR-0328 §D-17 | Add OS manifest |
| P2 | 7 | sigstore/cosign module signing missing | IaC inventory absence | Add module signing evidence |
| P2 | 7 | per-context state backend missing | IaC inventory absence | Add state backend per context |
| P2 | 8 | OS package formats absent | inventory absence | Add RPM/DEB/pkg/Homebrew/extensions/container mapping |
| P2 | 9 | canonical Rust build invocation not tied to service | ADR-0328 §D-18; source absence | Add Cargo build target and command |
| P3 | 2 | Product parity sometimes uses hyperscaler precedents instead of top-3 SaaS counterparts | `README.md:32-34`; `ARCHITECTURE.md:52-53` | Separate architecture precedents from product parity |
| P3 | 5 | OneTrust migration playbook exists but migration scope not in PRD | `PRD.md:116`; `migration-playbooks/from-onetrust.md` | Add migration policy |
| P3 | 9 | SDK prose includes non-Rust discussion but no source files | `sdk-plan.md` | Keep backend Rust-only and document exceptions |

Severity counts:
P0: 0.
P1: 16.
P2: 13.
P3: 3.

## §5 Open questions for Wave 14 aggregation

1. Should compliance own trust-center and questionnaire automation directly, or should those be split to a customer-assurance µservice with compliance as evidence source?
2. Should third-party risk management live in compliance, procurement, security, or a dedicated vendor-risk µservice?
3. Should consent/preference management live in compliance or in a separate consent µservice already reserved by prior architecture discussion?
4. Should OneTrust-style AI governance be owned by compliance, intelligence, governance, or a joint pack-overlay boundary?
5. Should PCI remain substrate-ready but launch-deferred until payments lands, or should PCI pack evidence be built as a compliance-only precondition now?
6. Should `implementation-plans/` be created despite the flat-layout PRD/ADR style, or should root IP files be accepted as the service-local implementation-plan convention?
7. Should `ARCHITECTURE.md` generated repetition be simplified into fewer higher-signal sections after canonical gaps are fixed?
8. Should the service declare no public/customer trust-center surface at demo_trial and move it to paid tenant_class, or should public trust proof exist at every tier?
9. Should OCI Always Free demo_trial use a reduced compliance surface, or should demo_trial be renamed because the current hardware envelope cannot fit Always Free?
10. Should cross-microservice handoffs be machine-readable JSON plus human notes, given user direction toward machine-optimized artifacts in Oyatie?

<!-- ORCHESTRATOR REPORT
  µservice: compliance
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/compliance/coherence-audit-2026-05-20.md: 806 lines
    - /Users/jasonlee/oyatie/microservices/compliance/feature-parity-matrix-2026-05-20.md: 405 lines
    - /Users/jasonlee/oyatie/microservices/compliance/performance-benchmark-numbers-2026-05-20.md: 306 lines
    - /Users/jasonlee/oyatie/microservices/compliance/capability-tier-deltas-vs-counterparts-2026-05-20.md: 368 lines
  inventory_files_seen: 201
  inventory_lines_read: 43432
  chat_history_matches_processed: 15
  findings_p0: 0
  findings_p1: 16
  findings_p2: 13
  findings_p3: 3
  top_3_counterparts_confirmed: Vanta / Drata / OneTrust
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1885
-->
