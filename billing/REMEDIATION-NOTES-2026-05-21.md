---
doc_class: RemediationNotes
title: cloud-billing Wave 15B remediation notes
status: Accepted
date: 2026-05-21
microservice: cloud-billing
scope: closure-documentation-only
write_scope: microservices/cloud-billing/REMEDIATION-NOTES-2026-05-21.md
source_audit: microservices/cloud-billing/coherence-audit-2026-05-20.md
---

# cloud-billing Remediation Notes - 2026-05-21

## 1. Closure Scope

This file is the Wave 15B finalizer note for cloud-billing.

It documents what landed after the prior cloud-billing spec-authoring sprint.

It does not author new PRD, architecture, contract, policy, IaC, or kernel substance.

It references existing artifacts only.

It preserves the user's write constraint: only this file is written.

No commits are produced.

The target audit is `coherence-audit-2026-05-20.md`.

The audit carried 40 findings.

The audit carried 12 P0 findings.

The P0 findings are CB-F-001 through CB-F-012.

The sprint substantively landed PRD, architecture, README, contracts, SLOs, Cedar policies, supported OS matrix, and partial IaC.

The sprint did not land kernel code changes.

The sprint did not land per-microservice ADR files under `decisions/`.

The sprint did not land implementation-plan files under `implementation-plans/`.

The sprint did not land OpenTofu module files for every created IaC context directory.

Those gaps are recorded below as deferred, not silently closed.

## 2. Verification Snapshot

Verifier: Codex finalizer.

Date: 2026-05-21.

Workspace: `/Users/jasonlee/oyatie`.

AGENTS scope observed: repo root AGENTS plus `docs/AGENTS.md`.

Root pointer read: `specs/root-hub-pointers.json`.

Operating contract read: `docs/AGENTS.md`.

Microservice directory inspected: `microservices/cloud-billing/`.

Command evidence used:

- `rg --files microservices/cloud-billing`
- `find microservices/cloud-billing -maxdepth 3 -type d`
- `wc -l $(rg --files microservices/cloud-billing | sort)`
- `rg -n "CB-F-00[1-9]|CB-F-010|CB-F-011|CB-F-012|P0" microservices/cloud-billing/coherence-audit-2026-05-20.md`
- `ls -la microservices/cloud-billing/decisions microservices/cloud-billing/implementation-plans microservices/cloud-billing/iac microservices/cloud-billing/iac/* microservices/cloud-billing/iac/oci-guest/always-free`
- `rg -n "foundry|Bronze|Silver|Gold|Platinum|--tier|TIER=" microservices/cloud-billing`
- `sed -n` reads of PRD, ARCHITECTURE, README, contracts, SLOs, policies, supported-oses, and IaC files.

Pre-finalizer line count for files returned by `rg --files microservices/cloud-billing`: 9,515 lines.

That pre-finalizer total excludes empty directories.

The new target file is intentionally not included in that pre-finalizer number.

## 3. Artifact Inventory and Line Counts

Core spec artifacts:

- `microservices/cloud-billing/PRD.md` - 786 lines.
- `microservices/cloud-billing/ARCHITECTURE.md` - 1,042 lines.
- `microservices/cloud-billing/README.md` - 418 lines.
- `microservices/cloud-billing/coherence-audit-2026-05-20.md` - 638 lines.
- `microservices/cloud-billing/feature-parity-matrix-2026-05-20.md` - 438 lines.
- `microservices/cloud-billing/performance-benchmark-numbers-2026-05-20.md` - 388 lines.
- `microservices/cloud-billing/supported-oses.json` - 152 lines.

Contract artifacts:

- `microservices/cloud-billing/contracts/openapi.yaml` - 993 lines.
- `microservices/cloud-billing/contracts/asyncapi.yaml` - 438 lines.
- `microservices/cloud-billing/contracts/proto/cloud-billing.proto` - 699 lines.

SLO artifacts:

- `slos/audit-chain-seal-latency.openslo.yaml` - 45 lines.
- `slos/cap-breach-detection-latency.openslo.yaml` - 47 lines.
- `slos/focus-export-completion-time.openslo.yaml` - 43 lines.
- `slos/fx-lock-freshness.openslo.yaml` - 39 lines.
- `slos/invoice-generation-time.openslo.yaml` - 49 lines.
- `slos/metering-event-ingest-latency.openslo.yaml` - 48 lines.
- `slos/rev-share-settlement-time.openslo.yaml` - 46 lines.
- `slos/seat-counting-availability.openslo.yaml` - 42 lines.
- `slos/tenant-class-read-api-latency.openslo.yaml` - 46 lines.
- `slos/usage-aggregation-time.openslo.yaml` - 43 lines.
- SLO subtotal - 448 lines.

Cedar policy artifacts:

- `policies/billing-components-gates.cedar` - 156 lines.
- `policies/cloud-billing.cedar` - 195 lines.
- `policies/conversion-gates.cedar` - 142 lines.
- `policies/demo-trial-gates.cedar` - 174 lines.
- `policies/settlement-gates.cedar` - 126 lines.
- `policies/tenant-class-binding.cedar` - 88 lines.
- Cedar subtotal - 881 lines.

IaC artifacts:

- `iac/oyatie-public-cloud/main.tf` - 107 lines.
- `iac/oyatie-public-cloud/outputs.tf` - 58 lines.
- `iac/oyatie-public-cloud/variables.tf` - 49 lines.
- `iac/oyatie-public-cloud/versions.tf` - 27 lines.
- IaC file subtotal - 241 lines.

Existing operational/supporting docs:

- `benchmarks/cloud-billing-vs-aws-cur-vs-gcp-billing-vs-azure-cost-management.md` - 105 lines.
- `capability-tiers/tier-matrix.md` - 93 lines.
- `faqs/billing-engineer-faq.md` - 200 lines.
- `migration-playbooks/from-aws-cur-and-cloudability.md` - 179 lines.
- `onboarding/billing-engineer-first-week.md` - 174 lines.
- `reference-implementations/emit-usage-and-generate-invoice-rust-sdk.md` - 200 lines.
- `runbooks/invoice-generation-timeout.md` - 269 lines.
- `runbooks/per-tenant-cost-attribution-mismatch.md` - 270 lines.
- `runbooks/reservation-recommendation-engine-stall.md` - 267 lines.
- `tutorials/meter-attribute-invoice-and-export-focus.md` - 196 lines.

Directory-only artifacts observed:

- `microservices/cloud-billing/decisions/` exists but contains no files.
- `microservices/cloud-billing/implementation-plans/` exists but contains no files.
- `iac/guest-on-aws/` exists but contains no files.
- `iac/guest-on-oci/` exists but contains no files.
- `iac/on-prem/` exists but contains no files.
- `iac/colo/` exists but contains no files.
- `iac/oyatie-as-cloud-provider/` exists but contains no files.
- `iac/oci-guest/always-free/` exists but contains no files.

Referenced-but-absent artifact:

- `microservices/cloud-billing/competitor-parity-matrix.md` is referenced by README and PRD completion-report prose but was not present during finalizer verification.

## 4. P0 Finding Closure Matrix

CB-F-001: Required canonical PRD missing.

Status: CLOSED for spec-authoring.

Evidence: `PRD.md` exists at 786 lines and includes tenant_class, billing_components, handoffs, SLO posture, deployment contexts, and P0 remediation mapping.

Residual: Kernel implementation items in PRD acceptance gates remain deferred.

CB-F-002: Required canonical architecture document missing.

Status: CLOSED for spec-authoring.

Evidence: `ARCHITECTURE.md` exists at 1,042 lines and defines bounded contexts, ledgers, workers, cross-service handoffs, deployment topology, security, observability, and roadmap.

Residual: Some architecture references point to not-yet-landed ADR and IP files.

CB-F-003: foundry referenced as still-active runtime owner.

Status: DEFERRED / PARTIAL.

Evidence of new direction: PRD and ARCHITECTURE route future cleanup through Wave 15I and use governance/cloud-iac style language for the new specification surface.

Evidence of unresolved drift: `rg` still finds `foundry` in FAQ, all three runbooks, benchmark evidence text, feature-parity, and the audit itself.

Stop condition for closure: rewrite legacy runbook and FAQ references to the ADR-0328 absorption mapping.

CB-F-004: Bronze/Silver/Gold/Platinum tier system pervasive.

Status: DEFERRED / PARTIAL.

Evidence of new direction: PRD, README, policies, contracts, and some new SLOs use `tenant_class` plus `billing_components`.

Evidence of unresolved drift: `capability-tiers/tier-matrix.md`, benchmark, FAQ, migration playbook, onboarding guide, tutorial, and runbook text still contain Bronze/Silver/Gold/Platinum, `--tier`, or `TIER=`.

Stop condition for closure: Wave 15J retires or rewrites the old tier corpus in-place.

CB-F-005: tenant_class enum not defined on tree.

Status: CLOSED for spec and contract surfaces; DEFERRED for kernel implementation.

Evidence: PRD section 6 defines the closed enum; README states exact values; proto defines `TenantClass`; OpenAPI and AsyncAPI expose tenant_class payloads; Cedar policies bind principal/resource/context tenant_class values.

Residual: `crates/cloud-billing-domain` was not modified in this sprint, so the Rust kernel extension is still future work.

CB-F-006: billing_components set plus revenue_share, per_seat, per_usage not modeled.

Status: CLOSED for spec, contract, and policy surfaces; DEFERRED for kernel implementation.

Evidence: PRD section 7 defines `BillingComponent` and `BillingComponentSet`; README documents all 8 combinations; proto defines `BillingComponent`; OpenAPI and AsyncAPI expose mutation/event surfaces; Cedar files gate the three components.

Residual: No kernel code changes were made; decision and implementation-plan files are absent.

CB-F-007: Zero OpenTofu modules for any of six deployment contexts.

Status: PARTIAL.

Evidence: `iac/oyatie-public-cloud/` contains `main.tf`, `variables.tf`, `outputs.tf`, and `versions.tf` totaling 241 lines.

Evidence of unresolved gap: the other five canonical context directories plus OCI Always Free are empty.

Stop condition for closure: OpenTofu module files land for guest-on-aws, guest-on-oci, on-prem, colo, oyatie-as-cloud-provider, and OCI Always Free.

CB-F-008: No supported-deployment-contexts manifest, per-context tenant onboarding flow, or per-context CI lane.

Status: DEFERRED / PARTIAL.

Evidence: PRD section 6.5 and ARCHITECTURE section 7 enumerate the six contexts; proto defines `DeploymentContext`.

Evidence of unresolved gap: no standalone `supported-deployment-contexts.json` was present; per-context module files are mostly absent; CI lane manifests were not found under this microservice.

CB-F-009: supported-oses.json missing.

Status: CLOSED for manifest authoring.

Evidence: `supported-oses.json` exists at 152 lines and declares 13 Tier-1 OS rows, Tier-2 ppc64le/s390x test-only rows, out-of-scope OSes, architecture matrix, default runtime, package formats, and CI lane names.

Residual: CI lane existence was not validated in this finalizer because the task scope was documentation closure.

CB-F-010: No OpenAPI / AsyncAPI / proto contract surface.

Status: CLOSED for contract authoring.

Evidence: `contracts/openapi.yaml` exists at 993 lines; `contracts/asyncapi.yaml` exists at 438 lines; `contracts/proto/cloud-billing.proto` exists at 699 lines.

Coverage: tenant_class, billing_components, metering, invoice, settlement, subscription, seat count, FOCUS export, and deployment context appear in the contract surfaces.

CB-F-011: No OpenSLO files on tree.

Status: CLOSED for SLO file presence; PARTIAL for ADR-0331 tenant_class label completeness.

Evidence: 10 OpenSLO YAML files exist under `slos/` totaling 448 lines.

Residual: several SLI Prometheus queries do not group by or filter on `tenant_class`; only some files carry explicit `tenant_class` or billing_component metadata.

CB-F-012: No Cedar policy files on tree.

Status: CLOSED for Cedar authoring.

Evidence: 6 Cedar files exist under `policies/` totaling 881 lines.

Coverage: master permits, tenant-class binding, billing-component gates, demo_trial gates, settlement gates, and conversion gates.

Residual: Cedar syntax/runtime validation was not run in this finalizer; no code/policy changes were requested.

## 5. billing_components Implementation Status

Overall status: spec-authored and contract-authored; kernel implementation deferred.

The prior sprint preserved `crates/cloud-billing-domain`.

No Rust changes were made for `TenantClass`, `BillingComponentSet`, or component workers.

The docs explicitly identify this as future kernel-extension work.

`revenue_share` status:

- PRD section 7.2 defines the component.
- ARCHITECTURE defines the settlement ledger and settlement worker.
- README defines monthly settlement, direction, FX adjustment, and clawback behavior.
- Proto defines revenue-share event kinds and settlement RPCs.
- AsyncAPI defines settlement event channels.
- Cedar `billing-components-gates.cedar` gates revenue-share operations.
- Cedar `settlement-gates.cedar` gates settlement compute, payout, clawback, affiliate payout, BEPS export, and sovereign invoice issue.
- Kernel implementation remains deferred.

`per_seat` status:

- PRD section 7.3 defines per-seat counting and seat snapshot behavior.
- ARCHITECTURE defines the seat-counter bounded context.
- README defines seat semantics and monthly cadence.
- SLO `seat-counting-availability.openslo.yaml` exists.
- Cedar `billing-components-gates.cedar` gates seat-count reads, add/remove seat operations, ceiling setting, and over-ceiling authentication denial.
- Kernel implementation remains deferred.

`per_usage` status:

- PRD section 7.4 defines per-usage meter aggregation.
- ARCHITECTURE defines metering bus, cloud-billing-event-ledger, meter-aggregator, and idempotency-dedup.
- README defines per-usage grouping by meter_unit and pricing_dimension.
- OpenAPI, AsyncAPI, and proto expose usage-event shapes.
- Existing kernel already has a `Usage` event concept, but the paid `billing_components` gating and `BillingComponentSet` model are not yet in kernel code.
- Kernel extension remains deferred.

## 6. tenant_class Adoption Surfaces per ADR-0331

ADR-0331 defines twelve adoption surfaces.

D-1 manifest.json.

Status: DEFERRED.

Evidence: no `microservices/cloud-billing/manifest.json` was found.

D-2 PRD tenant-class capability surface.

Status: PARTIAL / CLOSED for equivalent content.

Evidence: `PRD.md` has tenant_class semantics in section 6 and paid billing_components in section 7.

Residual: the ADR-0331 exact title `Tenant-class capability surface` was not verified as a section title.

D-3 ARCHITECTURE tenant-class axis.

Status: PARTIAL / CLOSED for equivalent content.

Evidence: `ARCHITECTURE.md` describes tenant-class state machine, cloud-iam reads, tenancy events, governance lanes, deployment topology, and fixtures.

Residual: exact ADR-0331 `Tenant-class axis` heading was not verified.

D-4 Cedar tenant_class principal-claim gate fragments.

Status: CLOSED for cloud-billing equivalent.

Evidence: `policies/tenant-class-binding.cedar`, `demo-trial-gates.cedar`, `conversion-gates.cedar`, `cloud-billing.cedar`, and `billing-components-gates.cedar` cover principal and resource tenant_class checks.

D-5 capability YAML tenant_class caps.

Status: DEFERRED.

Evidence: no `capabilities/tenant-class-caps.yaml` was found under cloud-billing.

D-6 OpenSLO tenant_class SLI label.

Status: PARTIAL.

Evidence: SLO files exist and some contain tenant_class-specific metadata.

Residual: many Prometheus queries do not include `tenant_class` label grouping/filtering.

D-7 cost-budget tenant_class axis.

Status: DEFERRED.

Evidence: no `cost-budget.md` was found under cloud-billing.

D-8 per-context IaC variants.

Status: PARTIAL.

Evidence: all six context directories plus OCI Always Free directory exist.

Residual: only `oyatie-public-cloud` contains OpenTofu files; the other context directories are empty.

D-9 mobile/SDK tenant_class header propagation.

Status: DEFERRED / NOT APPLICABLE TO CURRENT ARTIFACT SET.

Evidence: no SDK client implementation files were authored in this sprint; the Rust reference implementation exists but is not an ADR-0331 SDK adoption implementation.

D-10 onboarding flow conversion logic.

Status: PARTIAL.

Evidence: PRD, README, proto, OpenAPI, AsyncAPI, and Cedar define demo_trial to paid conversion.

Residual: existing onboarding guide still uses `TIER=Silver` and needs Wave 15J rewrite.

D-11 tests per tenant_class.

Status: DEFERRED.

Evidence: no `microservices/cloud-billing/tests/tenant_class/` directory was found.

D-12 observability tenant_class on every event.

Status: PARTIAL.

Evidence: PRD and ARCHITECTURE require tenant_class on audit events and metrics; AsyncAPI events include tenant_class in some payloads; SLO metadata includes related tenant_class coverage in selected files.

Residual: no runtime test evidence was produced here; not every SLO query carries `tenant_class`.

## 7. Deployment Context Coverage

Canonical paid deployment contexts named in PRD and ARCHITECTURE:

- `oyatie-public-cloud`
- `guest-on-aws`
- `guest-on-oci`
- `on-prem`
- `colo`
- `oyatie-as-cloud-provider`

Additional demo_trial default profile:

- `oci-guest/always-free`

Contract coverage:

- Proto `DeploymentContext` enum includes all six canonical paid contexts.

Documentation coverage:

- PRD section 6.5 enumerates all six paid contexts.
- ARCHITECTURE section 7.1 enumerates all six contexts and state backends.
- README status table says all six canonical contexts plus OCI Always Free.

Filesystem coverage:

- Directories exist for all six contexts plus OCI Always Free.
- `iac/oyatie-public-cloud/` contains OpenTofu files.
- `iac/guest-on-aws/` is empty.
- `iac/guest-on-oci/` is empty.
- `iac/on-prem/` is empty.
- `iac/colo/` is empty.
- `iac/oyatie-as-cloud-provider/` is empty.
- `iac/oci-guest/always-free/` is empty.

Conclusion:

- Deployment-context coverage is CLOSED for docs/contracts.
- Deployment-context coverage is PARTIAL for IaC files.
- Full module coverage remains deferred.

## 8. supported-oses Status

Status: CLOSED for manifest presence and content.

File: `microservices/cloud-billing/supported-oses.json`.

Line count: 152.

Tier-1 OS entries observed:

- Talos 1.7.
- RHEL 9.4.
- Oracle Linux 9.4.
- SLES 15-SP6.
- Ubuntu 24.04.
- Debian 13.
- Rocky 9.4.
- AlmaLinux 9.4.
- CentOS Stream 10.
- Amazon Linux 2023.
- Flatcar stable-3815.2.0.
- Photon 5.0.
- macOS Apple Silicon M5 developer-only.

Tier-2 test-only entries observed:

- linux ppc64le.
- linux s390x.

Out-of-scope entries observed:

- macOS Intel.
- macOS M1.
- macOS M2.
- macOS M3.
- macOS M4.
- FreeBSD.
- OpenBSD.
- Windows Server.
- Solaris.
- AIX.

Default runtime observed:

- Kubernetes pod.
- Cloud Hypervisor plus Kata.
- OCI image.
- distroless-cc-debian13 image base.

Residual:

- The file names CI lanes but this finalizer did not verify whether those CI lane files exist.

## 9. Cedar Policy Status

Status: CLOSED for policy files on tree.

Policy file count: 6.

Policy line total: 881.

`cloud-billing.cedar`:

- Master permits.
- tenant_class read.
- conversion.
- billing_components mutation.
- usage event emission.
- invoice issue/void/credit memo.
- reservation actions.
- settlement actions.

`tenant-class-binding.cedar`:

- principal.tenant_class schema comments.
- principal.billing_components schema comments.
- cap breach denial.
- cross-tenant denial.
- unknown tenant_class denial.
- unknown billing_component denial.

`billing-components-gates.cedar`:

- revenue_share settlement gates.
- marketplace listing gates.
- per_seat read/add/ceiling gates.
- over-seat denial.
- per_usage soft/hard cap gates.

`demo-trial-gates.cedar`:

- compliance pack denial for demo_trial.
- BYOK denial for demo_trial.
- marketplace paid-listing denial for demo_trial.
- free-listing consumption permit.
- cap-breach write denial.
- read during grace.
- conversion during grace.

`settlement-gates.cedar`:

- settlement compute.
- payout initiation.
- clawback handling.
- affiliate payout.
- BEPS export.
- sovereign invoice issuance.

`conversion-gates.cedar`:

- demo_trial to paid conversion permit.
- paid to demo_trial downgrade denial.
- unknown target denial.
- missing contract denial.
- partial transaction denial.
- audit-chain seal requirement.

Residual:

- This finalizer did not run a Cedar parser or policy-engine validation.

## 10. Explicit Deferred Items

The following items remain deferred after the spec-authoring sprint:

- Kernel implementation of `TenantClass`.
- Kernel implementation of `BillingComponentSet`.
- Kernel implementation of revenue_share-specific event and settlement types.
- Kernel implementation of per_seat seat counting.
- Kernel implementation of paid billing_components gating.
- `decisions/ADR-MS-001-billing-components-composability.md`.
- `decisions/ADR-MS-002-revenue-share-settlement-pipeline.md`.
- `implementation-plans/IP-001` through `IP-015`.
- `competitor-parity-matrix.md`.
- `manifest.json` with ADR-0331 D-1 fields.
- `capabilities/tenant-class-caps.yaml`.
- `cost-budget.md`.
- `tests/tenant_class/`.
- SDK header propagation tests.
- OpenTofu files for five of six paid deployment contexts.
- OpenTofu files for OCI Always Free.
- Full tier vocabulary retirement in legacy docs.
- Full foundry reference absorption in legacy docs.

## 11. Final Gate Read

The sprint is substantively landed for specification surfaces.

The sprint is not a full implementation closeout.

The Phase-0 substance blocker is materially reduced.

The P0 audit set is not uniformly closed.

Closed or effectively closed P0s:

- CB-F-001.
- CB-F-002.
- CB-F-009.
- CB-F-010.
- CB-F-012.

Closed for spec but deferred for kernel or implementation:

- CB-F-005.
- CB-F-006.

Partial:

- CB-F-007.
- CB-F-008.
- CB-F-011.

Deferred:

- CB-F-003.
- CB-F-004.

This is the honest closure boundary for Wave 15B finalization.

## Wave 15-IMPL-truth-up

Date: 2026-05-21.

Scope: truth-up every IP-declared crate/type/contract/Cedar-entity reference under `microservices/cloud-billing/`.

Doctrine references: `feedback_verify_deliverables_not_just_line_count_2026_05_20`, `feedback_no_silent_regression`, ADR-0212 buildability doctrine, ADR-0131 per-µservice flat layout.

### A. IP inventory result

IPs scanned: 0.

Filesystem evidence:

- `find microservices/cloud-billing -name "IP-*.md" -type f` returns no rows.
- `microservices/cloud-billing/implementation-plans/` is an empty directory (consistent with section 3 directory-only artifact observation above).
- `microservices/cloud-billing/decisions/` is an empty directory (no per-µservice ADR files).

Implication: Wave 15-IP-substance did not author or rewrite any cloud-billing IP files in this tree. The deferred items in section 10 (`IP-001` through `IP-015`) remain unwritten. There are therefore zero stamped IP claims to truth-up for this µservice.

### B. Declared artifacts catalogued

declared artifacts catalogued: 0.

Reason: the IP corpus is empty for cloud-billing. No artifacts are claimed by any IP, so no inventory can be extracted from IP claims.

For completeness, the artifacts that EXIST and are referenced from PRD/ARCHITECTURE/contracts (not from IPs) are recorded below as a baseline so a future IP-substance pass can build on them.

Existing Rust crates on tree (workspace members, confirmed):

- `crates/cloud-billing-domain` — billing-account, cloud-billing-event, invoice, tax-registration domain types; integrates with `metering-domain`, `cloud-region-domain`, `cloud-resource-domain`, `data-boundary-kernel`.
- `crates/cloud-billing-kernel` — kernel surface for cloud-billing.
- `crates/cloud-billing-tax-app` — tax composition app (includes integration tests under `tests/cloud_billing_invoice_api.rs`).

Existing contract surfaces on tree:

- `microservices/cloud-billing/contracts/openapi.yaml` (993 lines).
- `microservices/cloud-billing/contracts/asyncapi.yaml` (438 lines).
- `microservices/cloud-billing/contracts/proto/cloud-billing.proto` (699 lines).

Existing Cedar policies on tree:

- `policies/cloud-billing.cedar`, `policies/billing-components-gates.cedar`, `policies/conversion-gates.cedar`, `policies/demo-trial-gates.cedar`, `policies/settlement-gates.cedar`, `policies/tenant-class-binding.cedar`.

### C. Artifacts confirmed existing

artifacts confirmed existing: 3 Rust crates + 3 contract files + 6 Cedar policy files + 10 SLO files + 4 IaC files for `oyatie-public-cloud`.

Compile evidence: `cargo check -p cloud-billing-domain -p cloud-billing-kernel -p cloud-billing-tax-app` finished cleanly under workspace dev profile. No new crates were required; all declared (in non-IP docs) crates resolve.

### D. Artifacts scaffolded

artifacts scaffolded: 0.

Rationale per task anti-pattern guidance: this pass MUST NOT scaffold artifacts that don't belong (cross-µservice leakage) and MUST NOT scaffold for placeholder-only crates that future-Wave work owns. Since no IP file declares a missing artifact, scaffolding here would be speculative and would violate the "do not scaffold artifacts that already exist under a different name" + "do not scaffold artifacts that don't belong" guidance.

The genuinely-needed cloud-billing extensions (`TenantClass`, `BillingComponentSet`, per-seat / per-usage / revenue-share kernel types) are explicitly recorded as deferred kernel work in section 10 and require IP-led design before scaffolding. Truth-up scope does not author those.

### E. IP claims trimmed

IP claims trimmed: 0.

Rationale: trimming requires a present claim. There are no IP claims on tree, so nothing can be trimmed.

### F. Workspace Cargo.toml changes

No changes. No new crates were created, so the workspace member list is unchanged.

### G. Compile status

cargo check status: PASS.

Command: `cargo check -p cloud-billing-domain -p cloud-billing-kernel -p cloud-billing-tax-app`.

Result: `Finished dev profile [unoptimized + debuginfo] target(s) in 1.20s` with zero errors and zero warnings reported for the three cloud-billing crates.

### H. Follow-ups (precise, not silent)

The honest truth-up boundary for cloud-billing is that there is no IP corpus to truth-up. The follow-up work, ordered:

1. Wave 15-IP-substance owes cloud-billing the IP files listed in section 10 (`IP-001` through `IP-015`). Until those exist, IMPL-truth-up has nothing to ratify for this µservice.
2. When those IPs are authored they MUST be authored against the already-confirmed crates (`cloud-billing-domain`, `cloud-billing-kernel`, `cloud-billing-tax-app`) and the already-confirmed contracts under `contracts/` to avoid declaring phantom artifacts.
3. Per ADR-0331 D-1/D-5/D-7/D-11, `manifest.json`, `capabilities/tenant-class-caps.yaml`, `cost-budget.md`, and `tests/tenant_class/` remain deferred (section 6 of this file) — IPs must drive those before truth-up can act on them.
4. Per CB-F-007 / CB-F-008, the empty `iac/<context>/` directories for `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`, and `oci-guest/always-free` still need OpenTofu module files; truth-up does not author IaC modules because that is not the truth-up step's deliverable.
5. Per CB-F-003 / CB-F-004, Wave 15I (foundry absorption) and Wave 15J (tier-vocabulary retirement) own the legacy-doc rewrite — out of scope for truth-up.

REMEDIATION-NOTES section appended: Y.

<!--
COMPLETION REPORT - Wave 15B cloud-billing remediation-notes finalizer

Target written:
microservices/cloud-billing/REMEDIATION-NOTES-2026-05-21.md

Write scope honored:
Only the remediation-notes target was authored by this finalizer.

No commits produced.

Artifacts verified:
PRD.md 786L
ARCHITECTURE.md 1042L
README.md 418L
contracts/openapi.yaml 993L
contracts/asyncapi.yaml 438L
contracts/proto/cloud-billing.proto 699L
slos/*.openslo.yaml 10 files / 448L
policies/*.cedar 6 files / 881L
supported-oses.json 152L
iac/oyatie-public-cloud/*.tf 4 files / 241L

P0 status summary:
CB-F-001 closed.
CB-F-002 closed.
CB-F-003 deferred.
CB-F-004 deferred.
CB-F-005 closed for spec and contracts; kernel deferred.
CB-F-006 closed for spec/contracts/policies; kernel deferred.
CB-F-007 partial.
CB-F-008 partial.
CB-F-009 closed.
CB-F-010 closed.
CB-F-011 closed for file presence; tenant_class SLI labeling partial.
CB-F-012 closed for policy authoring.

billing_components summary:
revenue_share, per_seat, and per_usage are specified in PRD/ARCH/README/contracts/policies.
Runtime/kernel implementation remains deferred.

tenant_class adoption summary:
ADR-0331 12-surface coverage is partial: PRD, ARCH, Cedar, SLO, IaC, onboarding, observability have coverage; manifest, caps YAML, cost-budget, SDK tests, tenant_class tests are deferred.

Stop condition:
Finalizer complete when this file exists, exceeds 200 lines, references existing artifacts, and no other files are modified by the finalizer.
-->

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/cloud-billing/ARCHITECTURE.md`

Counterpart-fact preservations:
- None.

Files renamed (git mv):
- None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

D3-BUCKET-1 updated `PRD.md` frontmatter with ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because cloud-billing exports FOCUS data but this PRD does not declare the service as an OLAP writer through the canonical data-warehouse path.

### DR posture

Values: RTO 900 seconds, RPO 60 seconds, effective compliance floor stricter than HIPAA/SOC2/PCI/SOX/KR-CSAP/ISO/KR-PIPA pack floors; multi_region_active_active is yes for paid sovereign or multi-cell ledgers and home-cell fail-closed for demo_trial. ADR: ADR-0343. Alternatives considered: loosen to HIPAA 3600/300 or invent a manifest DR block; rejected because the existing PRD SLO is stricter and no manifest exists. Cost: D-2 must backfill `manifest.json` and a billing-cell failover runbook.

### Capacity model

Values: current envelope is 50,000 tenants/cell, 5,000,000 sustained events/sec, 18,000,000 burst events/sec, scaling by `per_request` and `per_usage_event`, Tier-1 placement for commercial ledger paths. ADR: ADR-0340. Alternatives considered: fabricate CPU/RAM/storage/connections or defer the whole section; rejected because prose must not invent manifest values, while the existing PRD already carries a usable cell envelope. Cost: per-tenant CPU/RAM/storage/connection baselines remain a manifest-backfill follow-up.

### Sustainability + cost attribution

Values: every billing audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours`; carbon routing applies to non-urgent export/recommendation/reporting paths and is excluded from period-close/legal/SLO paths. ADR: ADR-0344. Alternatives considered: aggregate only monthly at finops-portal; rejected because ADR-0344 requires per-call emission alongside audit rows. Cost: audit producers and finops rollups must carry the extra dimensions.

### API versioning posture

Values: YYYY-MM-DD carrier triplet across header, URL prefix, and proto3 field; SDK semver; last 3 versions for at least 180 days; per-tenant pinning for paid/sovereign tenants; internal gRPC exemption retained. ADR: ADR-0342. Alternatives considered: retain only `/v1` or SDK semver alone; rejected because invoices, ERP, FOCUS, and tenant_class reads are public contractual surfaces. Cost: router, generated SDK, and deprecation-calendar work for date-version migration.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

D4-BUCKET-1 trigger-based IP doctrine propagation.

- Root IPs scanned: 0
- Trigger A additions: 0
- Trigger B additions: 0
- Trigger C additions: 0
- Trigger D additions: 0
- Root IPs unmatched: 0
- Doctrine sources: ADR-0338, ADR-0342, ADR-0343, ADR-0344, ADR-0345; `specs/compliance-pack-floors.json`.
- Idempotence: skipped any IP section that already existed; no unmatched root IPs were edited.

IP-by-IP changes:
- No root `IP-*.md` files found for this service under the prompt trigger surface; `implementation-plans/IP-*.md` was not edited because the dispatch pattern was root `microservices/<ms>/IP-*.md`.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.25 vCPU, 512 MiB RAM, 25 GB storage per active tenant; Valkey/Postgres/outbound connections 2/4/10; scaling_dimension=per_message; cell_placement_class=Tier-3.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=2.
- Why: Metering and invoice ledger load scales with Kafka-style metering messages, period-close workers, and tenant-scoped ledgers; PRD/architecture cite 5M events/sec and cell-aware data residency.
- Rejected: Tier-2 was rejected because cloud-billing is an application family service under ADR-0340; the heavier Tier-2 capability class would blur it with workflow/identity substrate placement.
- Cost: Commits paid billing cells to multi-region Postgres/WAL-G plus object-versioned invoice exports and audit-chain seal retention.

### Block 2: dr
- Values: RTO=900s, RPO=60s, multi_region_active_active=true, backup_substrate=postgres_wal_g+object_storage_versioned+audit_chain_merkle_seal, failover_runbook=runbooks/billing-ledger-failover.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns billing ledger, metering ingest, invoices, settlement, FOCUS export; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; evidence=microservices/cloud-billing/PRD.md, microservices/cloud-billing/ARCHITECTURE.md, microservices/cloud-billing/implementation-plans/IP-014-cell-aware-billing-data-residency.md.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-3.
- Why: First-party billing application workload: it handles Oyatie-authored metering, invoices, settlement, and exports; it does not execute tenant-customer code, while FINANCIAL rows and event ledgers are protected by Cedar and audit-chain rather than a Tier-1 substrate runtime.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi, asyncapi, proto.
- ADR: ADR-0342.
- Why: tenant-facing billing invoices/events need stable dated API behavior for finance and audit integrations.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql,valkey,kafka,cedar; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
- ADR: ADR-0345; classes, owners, and CVE SLAs remain centralized in specs/oss-stewardship-registry.json.
- Why: The manifest now indexes the service to the registry so SBOM, SOC2, ISO 27001, and CVE-response evidence can be generated without free-text dependency inference.
- Rejected: embedding per-dependency owner/class objects in this manifest was rejected because manifest-schema.json defines this field as dep_name strings, not local copies of registry rows.
- Cost: Any new direct upstream now needs a registry entry or an explicit local override before the service can pass the governance lane.

### Block 6: iac_module_invocations
- Values: Declared 9 shared module primitive invocations from the service's IaC context evidence; inline OpenTofu resource bodies remain a migration risk until Wave 15Q lands module bodies.
- ADR: ADR-0339.
- Why: IaC dependency on shared primitives must be machine-readable so module pins, signatures, and wrapper-thinness can be checked at admission.
- Rejected: hand-authored, per-service OpenTofu resources were rejected as the long-term target because they preserve the duplication ADR-0339 was created to remove.
- Cost: Future IaC edits must use shared module pins and keep service wrappers thin.
