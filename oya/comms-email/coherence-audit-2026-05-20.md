# comms-email ownership-coherence audit — 2026-05-20

µservice: `comms-email`
Audit owner: single-agent ownership audit
Counterpart set: SendGrid / Postmark / Mailgun
Deployable-context assumption: all 6 contexts unless evidence proves otherwise
Deliverable scope: 3 documents only; retired capability-profile delta deliverable intentionally omitted
Evidence stance: file-line citations, canonical direction citations, and chat-history citations only
Stop condition: publish the 3 required reports with line-count verification and no new capability-profile scaffold

## §1 Purpose

This audit evaluates whether `microservices/comms-email/` is coherent as a µservice-owned product surface.
The service is intended to be Oyatie's canonical email substrate, not a loose mail-provider wrapper.
The original PRD defines a transactional-email substrate for all µservices that currently create bespoke SES clients.
Evidence: `PRD.md:10-17`.
The same PRD requires DKIM, SPF, DMARC, adapter abstraction, audit-chain emission, tenant domains, rate ceilings, and suppression.
Evidence: `PRD.md:21-34`.
The newer README broadens purpose to transactional plus marketing email.
Evidence: `README.md:13-15`.
The newer README also names inbound receiving, list management, unsubscribe handling, and reputation monitoring.
Evidence: `README.md:17-21`.
Chat history confirms Wave 3-B originally described `comms-email` as "transactional + marketing email substrate".
Evidence: `.claude/.../8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:6713-6716`.
Chat history also confirms the expected counterpart set includes SendGrid, Postmark, and Mailgun.
Evidence: `.claude/.../8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290-16311`.
The audit therefore treats the actual product as email delivery, templates, suppressions, webhooks, deliverability, inbound handling, list handling, and reputation telemetry.
The audit does not treat push notifications, BIMI logos, or an in-house Rust-native MTA as current scope.
Evidence: `PRD.md:57-65`.
The audit flags contradictions where newer artifacts already moved inbound or list workflows into implementation plans.
Evidence: `IP-016-inbound-receiver-kernel.md:11-23`, `IP-018-list-management-usecase.md:11-24`.
The audit uses the 2026-05-20 doctrine that capability profiles are retired.
Evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_customer_class_ladders_2026_05_20.md:10-24`.
The audit uses the replacement tenant-class model from the prompt: `demo_trial`, `paid`, and `revenue_share`.
The audit checked whether that tenant-class vocabulary exists in this µservice path.
Evidence: `rg tenant_class|demo_trial|revenue_share|Always Free|always-free|paid microservices/comms-email`.
The only `paid` hit in the service is an API pricing phrase, not tenant-class semantics.
Evidence: `ARCHITECTURE.md:919`.
The audit treats old T0/T1/T2/T3 capability records and old named commercial tiers as retirement candidates, not as future design.
Evidence: `manifest.json:44-48`, `tenant_class model in ADR-0330:9-14`.
The audit evaluates five canonical constraints: six deployment contexts, OpenTofu IaC, OS support, Rust-strict implementation language, and OCI Always Free profile.
The six deployment contexts are canonical in `master-plan-sequencing.json`.
Evidence: `specs/master-plan-sequencing.json:704-745`.
OpenTofu is the canonical IaC engine and Terraform/Pulumi/CloudFormation/ARM are forbidden.
Evidence: `specs/master-plan-sequencing.json:747-775`.
Each µservice must declare supported OS coverage at the service level.
Evidence: `specs/master-plan-sequencing.json:777-815`.
Backend code must be Rust; non-Rust backend code is forbidden except data, policy, proto, OpenTofu, docs, and front-end allowlist cases.
Evidence: `specs/master-plan-sequencing.json:817-855`.
OCI Always Free has a required service module path when the service claims the guest-on-oci profile.
Evidence: `specs/master-plan-sequencing.json:857-867`.
The audit read the service inventory recursively and counted 136 files and 14,645 current lines before authoring this report.
Evidence: `find microservices/comms-email -type f | sort | wc -l`, `find microservices/comms-email -type f | sort | xargs wc -l | tail -1`.
The audit read chat history for `comms-email` rather than trusting prior self-report.
Evidence: `.claude/.../8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:7011`.
The audit did not create a commit and did not touch any other µservice.

## §2 Inventory

Inventory count: 136 files seen before writing this audit.
Inventory line count read baseline: 14,645 service lines before new deliverables.
Inventory method: recursive `find` plus line-numbered reads for core artifacts and representative evidence.

### §2.1 Top-level product and plan files

- `ARCHITECTURE.md` — accepted architecture walkthrough with principals, Cedar, cell, abuse, and repeated old tier vocabulary.
- `AUDIT-FINDINGS-2026-05-20.json` — prior audit status record; contains old `tier_before` and `tier_after` fields.
- `CHANGELOG.md` — service-local change chronology.
- `IP-001-ses-adapter-impl.md` — SES adapter implementation plan for AWS-backed delivery.
- `IP-002-postal-adapter-impl.md` — Postal adapter implementation plan; contains old sovereign tier vocabulary.
- `IP-003-smtp-fallback-adapter-impl.md` — SMTP fallback implementation plan.
- `IP-004-mailgun-adapter-impl.md` — Mailgun adapter implementation plan.
- `IP-005-dkim-key-rotation-pipeline.md` — DKIM key rotation plan; references old Tier-B DNS module vocabulary.
- `IP-006-mjml-template-renderer.md` — MJML renderer plan using Rust-friendly rendering path.
- `IP-007-liquid-substitution-engine.md` — Liquid substitution plan for per-send variables.
- `IP-008-webhook-delivery-pipeline.md` — webhook normalization and retry plan.
- `IP-009-bounce-complaint-handler.md` — bounce and complaint handling plan.
- `IP-010-suppression-list.md` — suppression policy and storage plan; references storage tier language.
- `IP-011-per-tenant-from-domain-onboarding.md` — from-domain onboarding plan.
- `IP-012-audit-chain-emission.md` — audit-chain emission plan.
- `IP-013-multi-region-routing.md` — multi-region provider-routing plan.
- `IP-014-sovereign-pack-postal-only-enforcement.md` — Postal-only sovereign pack enforcement; contains old tier-check command language.
- `IP-015-in-house-relay-roadmap-phase-2.md` — in-house relay roadmap marker, explicitly future.
- `IP-016-inbound-receiver-kernel.md` — inbound receiver kernel plan, contradicting PRD deferred inbound.
- `IP-017-inbound-receiver-domain.md` — inbound domain plan with quarantine and audit behavior.
- `IP-018-list-management-usecase.md` — list import, segment, and double-opt-in plan, contradicting PRD non-goal.
- `IP-019-unsubscribe-handler-domain.md` — one-click unsubscribe and preference-center domain plan.
- `IP-020-reputation-monitor-worker.md` — reputation-monitoring worker plan.
- `IP-021-bounce-handler-domain.md` — bounce handler domain plan.
- `IP-022-template-rendering-mjml-engine.md` — template-rendering MJML engine plan.
- `IP-023-inbound-receiver-rest.md` — inbound REST endpoint plan.
- `IP-024-list-management-rest.md` — list and segment REST endpoint plan.
- `IP-025-reputation-monitor-rest-and-dashboard.md` — reputation dashboard REST plan.
- `IP-026-unsubscribe-async-emit.md` — cross-region unsubscribe event plan.
- `IP-journey-j100-pack-rollout-first-action.md` — pack rollout journey overlay.
- `IP-journey-j91-us-msb-mtl-overlay.md` — US MSB/MTL journey overlay.
- `IP-journey-j92-br-lgpd-us-parent-dsar.md` — Brazil LGPD and US parent DSAR overlay.
- `IP-journey-j93-in-dpdpa-rbi-overlay.md` — India DPDPA/RBI overlay, with repeated merchant KYC tiering vocabulary.
- `IP-journey-j94-sox404-public-company-controls.md` — SOX 404 control overlay.
- `IP-journey-j95-iso27001-soc2-annual-audit.md` — ISO/SOC2 audit overlay.
- `IP-journey-j96-ksa-uae-mena-onboarding.md` — KSA/UAE onboarding overlay.
- `IP-journey-j97-sg-pdpa-mas-tenant.md` — Singapore PDPA/MAS overlay.
- `IP-journey-j98-au-privacy-apra-cps234.md` — Australia privacy/APRA overlay.
- `IP-journey-j99-multi-pack-conflict-resolution.md` — multi-pack conflict-resolution overlay.
- `PHASE-01-COMMS-EMAIL-SUBSTRATE.md` — Phase 1 plan with cloud-hosted and sovereign old tier language.
- `PRD.md` — original product requirements; still the clearest transactional substrate contract.
- `README.md` — newer reference surface broadening product to marketing, inbound, lists, and reputation.
- `backfill-replay.md` — replay/backfill operational doc.
- `capacity-model.md` — capacity, storage, latency, and throughput assumptions.
- `competitor-parity-matrix.md` — existing provider comparison; now stale on inbound and tier language.
- `compliance.md` — long compliance answer set with repeated old tier/product metadata.
- `cost-budget.md` — provider cost budget; contains old sovereign tier budget language.
- `dpia.md` — GDPR impact assessment.
- `failure-modes.md` — failure mode and detection/runbook mapping.
- `incident-response.md` — severity ladder and incident procedures.
- `manifest.json` — machine-readable service manifest; missing deployment contexts and tenant classes.
- `multi-region.md` — multi-region behavior reference.
- `sdk-plan.md` — SDK planning doc; contains generic Tier 1 docs reference.
- `threat-model.md` — STRIDE threat model.

### §2.2 Benchmarks, capabilities, and catalog

- `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md` — old benchmark report using retired named commercial tiers.
- `capabilities/T0-transactional-send.json` — capability record with `"tier": "T0"`.
- `capabilities/T1-bounce-handle.json` — capability record with `"tier": "T1"`.
- `capabilities/T1-webhook-delivery-event.json` — capability record with `"tier": "T1"`.
- `capabilities/T2-list-manage.json` — capability record with `"tier": "T2"` and list actions.
- `capabilities/T2-tenant-domain-mgmt.json` — capability record with `"tier": "T2"`.
- `capabilities/T3-inbound-receive.json` — capability record with `"tier": "T3"` and inbound actions.
- `tenant_class model in ADR-0330` — retired capability-profile matrix and the densest retirement candidate.
- `catalog/bounded-contexts.json` — 16 context entries; omits inbound/list/reputation names found in IPs.

### §2.3 Contracts

- `contracts/asyncapi.yaml` — event channels for delivery, bounces, suppressions, DKIM, and domain state.
- `contracts/comms_email.proto` — gRPC service with send, domain, and DKIM operations; includes a Go package option.
- `contracts/openapi.yaml` — REST contract for messages, bounces, suppressions, webhooks, and domains.

### §2.4 Dashboards

- `dashboards/deliverability.json` — deliverability dashboard evidence.
- `dashboards/dkim-rotation.json` — DKIM rotation dashboard evidence.
- `dashboards/reputation-monitoring.json` — reputation dashboard evidence.
- `dashboards/send-pipeline.json` — send pipeline dashboard evidence.
- `dashboards/webhook-and-audit.json` — webhook and audit dashboard evidence.

### §2.5 Decisions

- `decisions/ADR-CME-001-per-tenant-signing-key-custody-with-rotation-cadence.md` — proposed service ADR for DKIM custody.
- `decisions/SVC-ADR-001-dkim-cadence.md` — accepted DKIM rotation cadence.
- `decisions/SVC-ADR-002-suppression-list-policy.md` — accepted suppression removal policy.
- `decisions/SVC-ADR-003-webhook-retry-policy.md` — accepted webhook retry and DLQ policy.
- `decisions/SVC-ADR-004-tenant-domain-onboard-flow.md` — accepted domain onboarding state machine.
- `decisions/SVC-ADR-005-mjml-liquid-canonical.md` — accepted MJML plus Liquid templating decision.

### §2.6 IaC and runtime assets

- `iac/ech-config.yaml` — ECH config evidence.
- `iac/edge-waf.yaml` — edge WAF evidence.
- `iac/helm/postal/Chart.yaml` — Postal chart metadata; contains old sovereign tier annotation.
- `iac/helm/postal/templates/deployment.yaml` — Postal Helm deployment template.
- `iac/helm/postal/values.yaml` — Postal chart values; contains old sovereign tier comment.
- `iac/helm/ses-adapter/values.yaml` — SES adapter Helm values.
- `iac/k8s-deployment.yaml` — Kubernetes deployment for outbound sender.
- `iac/k8s-network-policy.yaml` — Kubernetes network policy.
- `iac/openbao-policy.hcl` — OpenBao policy.
- `iac/packs/eu/overlay.yaml` — EU overlay.
- `iac/packs/kr/overlay.yaml` — KR overlay.
- `iac/packs/ksa/overlay.yaml` — KSA overlay.
- `iac/packs/uae/overlay.yaml` — UAE overlay.
- `iac/packs/us-healthcare/overlay.yaml` — US healthcare overlay.
- `iac/pqc-cert.yaml` — PQC certificate config.
- `iac/secret-bindings.yaml` — secret binding config.
- `iac/terraform-module.tf` — Terraform-named module; violates OpenTofu naming and context layout doctrine.

### §2.7 Policy, onboarding, migration, reference, runbooks, scorecards, SLOs, source, tutorials

- `faqs/deliverability-engineer-faq.md` — deliverability FAQ with old named commercial tiers.
- `migration-playbooks/from-sendgrid-and-mailgun.md` — migration guide from SendGrid, Mailgun, Postmark, and SES.
- `onboarding/deliverability-engineer-first-week.md` — deliverability engineer onboarding; cites non-existent ADR names and old tier docs.
- `policy/abuse-defence.cedar` — abuse defense policy.
- `policy/action-authorization.cedar` — action authorization policy.
- `policy/auditor-scope.cedar` — auditor scope policy.
- `policy/ci-scope.cedar` — CI scope policy.
- `policy/comms-email-send.cedar` — send policy.
- `policy/comms-email-suppression-list.cedar` — suppression policy.
- `policy/comms-email-tenant-domain-mgmt.cedar` — domain management policy.
- `policy/comms-email-webhook-ingest.cedar` — webhook ingest policy.
- `policy/data-residency.cedar` — residency policy.
- `policy/dual-context.md` — dual-context policy note.
- `policy/pack-overlay-authorization.cedar` — pack overlay authorization policy.
- `policy/residency.md` — residency policy note with application/admission tier language.
- `reference-implementations/send-transactional-rust-sdk.md` — Rust SDK reference implementation.
- `runbooks/blacklist-recovery.md` — blacklist recovery runbook.
- `runbooks/bounce-storm-mitigation.md` — bounce storm runbook.
- `runbooks/dkim-key-rotation.md` — DKIM rotation runbook.
- `runbooks/dmarc-policy-tune.md` — DMARC tuning runbook.
- `runbooks/inbound-receiver-quarantine-release.md` — inbound quarantine release runbook.
- `runbooks/per-tenant-from-domain-onboard.md` — domain onboarding runbook.
- `runbooks/postal-failover.md` — Postal failover runbook with old sovereign tier language.
- `runbooks/reputation-drop-circuit-breaker-engaged.md` — reputation circuit breaker runbook.
- `runbooks/ses-failover.md` — SES failover runbook.
- `runbooks/webhook-replay.md` — webhook replay runbook.
- `scorecards/compliance.json` — compliance scorecard.
- `scorecards/operational-excellence.json` — operational excellence scorecard.
- `scorecards/overrides.json` — override scorecard with old tier target.
- `scorecards/reliability.json` — reliability scorecard.
- `scorecards/security.json` — security scorecard.
- `slos/audit-chain-emit-lag-p99.openslo.yaml` — audit emit lag SLO.
- `slos/deliverability-rate.openslo.yaml` — receiver delivery SLO.
- `slos/dkim-signing-rate.openslo.yaml` — DKIM signing SLO.
- `slos/dmarc-alignment-rate.openslo.yaml` — DMARC alignment SLO.
- `slos/from-domain-onboarding-time.openslo.yaml` — domain onboarding SLO.
- `slos/send-latency-p99.openslo.yaml` — send p99 latency SLO.
- `slos/send-success-rate.openslo.yaml` — provider accept success SLO.
- `slos/suppression-lookup-latency-p99.openslo.yaml` — suppression lookup SLO.
- `slos/webhook-success-rate.openslo.yaml` — webhook-to-audit SLO.
- `src/README.md` — says service code lives in workspace crates and that this directory is a docs root.
- `tutorials/send-1m-transactional-campaign-with-warmup.md` — campaign warmup tutorial with old named commercial tier prerequisite.

## §3 9-dimension audit

### §3.1 Dimension 1 — Product purpose and ownership coherence

Assessment: partial pass with a P1 product-scope contradiction.
The PRD centers on transactional email sent on behalf of other µservices.
Evidence: `PRD.md:10-17`.
The PRD declares marketing-class campaign management, lists, A/B tests, and segments as non-goals.
Evidence: `PRD.md:92-98`.
The README calls the product transactional plus marketing email.
Evidence: `README.md:13-15`.
The README lists inbound receiving, list management, unsubscribe handling, and reputation monitoring.
Evidence: `README.md:17-21`.
The implementation-plan set adds inbound receiver kernel/domain/REST.
Evidence: `IP-016-inbound-receiver-kernel.md:11-23`, `IP-017-inbound-receiver-domain.md:11-24`, `IP-023-inbound-receiver-rest.md:11-21`.
The implementation-plan set adds list management and list REST.
Evidence: `IP-018-list-management-usecase.md:11-24`, `IP-024-list-management-rest.md:11-22`.
The implementation-plan set adds unsubscribe handling and cross-region unsubscribe emission.
Evidence: `IP-019-unsubscribe-handler-domain.md:11-26`, `IP-026-unsubscribe-async-emit.md:11-21`.
The implementation-plan set adds reputation monitoring.
Evidence: `IP-020-reputation-monitor-worker.md:11-25`, `IP-025-reputation-monitor-rest-and-dashboard.md:11-24`.
The manifest remains closer to the older transactional substrate and only lists seven bounded contexts.
Evidence: `manifest.json:18-26`.
The catalog lists 16 bounded-context entries, but still omits the newer inbound/list/unsubscribe/reputation contexts by those exact names.
Evidence: `catalog/bounded-contexts.json:4-101`.
This means a reader cannot tell whether inbound and list management are Phase 1 scope, Phase 2 scope, or accidental drift.
The service owner is consistently `oya-substrate-comms` across PRD and manifest.
Evidence: `PRD.md:5`, `manifest.json:4`.
The ownership stance is good, but the owned product boundary is not coherent.
Finding linkage: F-01.

### §3.2 Dimension 2 — Artifact inventory, completeness, and substance

Assessment: strong document breadth with a P2 executable-evidence gap.
The directory has 136 files, including PRD, architecture, contracts, policies, SLOs, dashboards, runbooks, DPIA, compliance, failure modes, capacity, and migration material.
Evidence: recursive inventory in §2.
The service has 9 OpenSLO files with concrete targets.
Evidence: `slos/send-latency-p99.openslo.yaml:12-44`, `slos/webhook-success-rate.openslo.yaml:12-36`.
The failure-mode file maps provider, deliverability, substrate, tenant, and catastrophic cases to response patterns.
Evidence: `failure-modes.md:5-140`.
The incident-response file defines severity triggers, DKIM compromise, blacklist, bounce storm, webhook backlog, and SES quota response.
Evidence: `incident-response.md:5-109`.
The DPIA includes processing purpose, personal data categories, lawful basis, transfers, retention, and risk assessment.
Evidence: `dpia.md:7-105`.
The compliance file maps CAN-SPAM, GDPR, CCPA, HIPAA, KR PIPA, KSA/UAE, SOC2, and PCI.
Evidence: `compliance.md:8-117`.
The source directory is not an implementation root; it points to future workspace crates.
Evidence: `src/README.md:1-11`.
The source directory states that 16 bounded-context crates land in follow-up implementation PRs.
Evidence: `src/README.md:45-47`.
No `tests/` directory exists under `microservices/comms-email/`.
Evidence: `find microservices/comms-email -path '*/tests/*' -o -name '*test*' -o -name '*spec*' | sort`.
This does not invalidate a documentation audit, but it limits executable verification.
The service has enough substantive documentation for audit reporting.
It does not yet have enough local implementation evidence to prove runtime behavior.
Finding linkage: F-08.

### §3.3 Dimension 3 — Counterpart fit and union-coverage pressure

Assessment: partial pass; the counterpart set is right, but current parity docs lag product breadth.
Chat history identifies SendGrid, Postmark, and Mailgun as the top-three audit counterpart set.
Evidence: `.claude/.../8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290-16311`.
The existing competitor parity matrix covers SES, SendGrid, Mailgun, Postmark, and Postal.
Evidence: `competitor-parity-matrix.md:6-15`.
That matrix covers transactional API, DKIM, SPF, DMARC, tenant domains, suppressions, webhooks, multi-region, self-hosting, templates, audit chain, and idempotency.
Evidence: `competitor-parity-matrix.md:18-39`.
The same matrix says inbound email is deferred.
Evidence: `competitor-parity-matrix.md:57-64`.
The README and IPs no longer keep inbound deferred.
Evidence: `README.md:17-21`, `IP-016-inbound-receiver-kernel.md:11-23`.
Postmark has official outbound/inbound/broadcast separation and message streams.
Evidence: `https://postmarkapp.com/manual`, lines 235-243 and `https://postmarkapp.com/message-streams`, lines 167-205.
SendGrid has official Mail Send, dynamic templates, event activity, unsubscribe mechanisms, and inbound parse.
Evidence: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 298-352; `https://www.twilio.com/docs/sendgrid/for-developers/parsing-email/inbound-email`, lines 112-151.
Mailgun has official batch sending, webhooks, routes, mailing lists, and suppressions.
Evidence: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/batch-sending`, lines 111-150; `https://documentation.mailgun.com/docs/mailgun/user-manual/receive-forward-store/routes`, lines 104-115.
The existing parity matrix is no longer the right comparison base for the broadened product.
The separate feature parity deliverable updates the union matrix without creating tier deltas.
Finding linkage: F-09.

### §3.4 Dimension 4 — Canonical-direction alignment

Assessment: fail on five constraint alignment, with strong partials in policy/SLO/compliance.
The canonical deployment-context list contains six contexts.
Evidence: `specs/master-plan-sequencing.json:704-745`.
The service has no `iac/oyatie-public-cloud/` directory.
Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
The service has no `iac/guest-on-aws/` directory.
Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
The service has no `iac/oci-guest/` directory.
Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
The service has no `iac/oci-guest/always-free/` directory.
Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
The service has no `iac/on-prem/` directory.
Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
The service has no `iac/colo/` directory.
Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
The service has no `iac/oyatie-iaas/` directory.
Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
The manifest `iac` block names only Helm and pack overlays.
Evidence: `manifest.json:86-89`.
Canonical IaC uses OpenTofu, not Terraform.
Evidence: `specs/master-plan-sequencing.json:747-775`.
The service includes `iac/terraform-module.tf`.
Evidence: `iac/terraform-module.tf:1-10`.
The Terraform-named file uses `terraform { required_version = ">= 1.7.0" }`.
Evidence: `iac/terraform-module.tf:1-4`.
The Terraform-named file uses HashiCorp Kubernetes and Helm providers.
Evidence: `iac/terraform-module.tf:5-9`.
The service has no `supported-oses.json` file.
Evidence: recursive inventory in §2.
Canonical OS support requires per-service OS manifests.
Evidence: `specs/master-plan-sequencing.json:777-815`.
The service has no forbidden backend language files under the µservice path.
Evidence: `find microservices/comms-email -type f \( -name '*.py' -o -name '*.js' -o -name '*.ts' -o -name '*.tsx' -o -name '*.rb' -o -name '*.go' -o -name '*.java' -o -name '*.scala' -o -name '*.groovy' -o -name '*.php' -o -name '*.fs' \) -print | sort`.
The `.proto` file includes a Go package option, but no Go source file exists in the µservice path.
Evidence: `contracts/comms_email.proto:1-17`.
OCI Always Free requires a per-service module path.
Evidence: `specs/master-plan-sequencing.json:857-867`.
The service has no OCI Always Free module path.
Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
The service has useful Kubernetes, Helm, OpenBao, PQC, WAF, ECH, and pack overlays.
Evidence: `iac/k8s-deployment.yaml:1-35`, `iac/helm/postal/values.yaml:15-62`.
Those overlays do not satisfy context-specific OpenTofu evidence.
Finding linkage: F-02, F-03, F-04, F-05.

#### §3.4.T — Tenant-class adoption candidates

Default severity: P2 documentation gap unless tied to deployment-gating behavior.
Candidate: `tenant_class model in ADR-0330:2` declares `CapabilityTierMatrix`.
Candidate: `tenant_class model in ADR-0330:9` titles the file as a capability profile matrix.
Candidate: `tenant_class model in ADR-0330:11` says tiers differ on send envelope and deliverability surface.
Candidate: `tenant_class model in ADR-0330:13` defines a named preview tier.
Candidate: `tenant_class model in ADR-0330:30` assigns raw bounce codes to that named tier.
Candidate: `tenant_class model in ADR-0330:49` defines a named production-default tier.
Candidate: `tenant_class model in ADR-0330:51` says it adds to the previous tier.
Candidate: `tenant_class model in ADR-0330:81` defines a named multi-region tier.
Candidate: `tenant_class model in ADR-0330:83` says it adds to the previous tier.
Candidate: `tenant_class model in ADR-0330:110` compares cost delta between named tiers.
Candidate: `tenant_class model in ADR-0330:114` defines a named sovereign-pack tier.
Candidate: `tenant_class model in ADR-0330:116` says it adds to the previous tier.
Candidate: `tenant_class model in ADR-0330:127` reuses the named multi-region latency baseline.
Candidate: `tenant_class model in ADR-0330:129` reuses the named multi-region SLO posture.
Candidate: `tenant_class model in ADR-0330:133` describes what does not differ across tiers.
Candidate: `tenant_class model in ADR-0330:136` says enforcement is on at every tier.
Candidate: `tenant_class model in ADR-0330:142` defines a migration path across named tiers.
Candidate: `tenant_class model in ADR-0330:144` says only send-tier degrades on downgrade.
Candidate: `manifest.json:44-48` stores capability records with `tier` fields.
Candidate: `manifest.json:60-64` stores provider records with `tier` fields.
Candidate: `manifest.json:96-101` stores `capability_profiles`.
Candidate: `manifest.json:118` stores `service_classification`.
Candidate: `manifest.json:145` stores `criticality_tier`.
Candidate: `capabilities/T0-transactional-send.json:2-3` stores `T0` and `tier`.
Candidate: `capabilities/T1-bounce-handle.json:2-3` stores `T1` and `tier`.
Candidate: `capabilities/T1-webhook-delivery-event.json:2-3` stores `T1` and `tier`.
Candidate: `capabilities/T2-list-manage.json:2-3` stores `T2` and `tier`.
Candidate: `capabilities/T2-tenant-domain-mgmt.json:2-3` stores `T2` and `tier`.
Candidate: `capabilities/T3-inbound-receive.json:2-3` stores `T3` and `tier`.
Candidate: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:13` uses an old named hardware tier.
Candidate: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:21` labels benchmark rows by old named tier.
Candidate: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:36` labels MTA latency rows by old named tier.
Candidate: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:49` labels inbox-rate rows by old named tier.
Candidate: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:58` states a PRD target at old named tier.
Candidate: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:87` uses another old named cost tier.
Candidate: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:106` shows `--tenant-class` in the benchmark command.
Candidate: `faqs/deliverability-engineer-faq.md:13` maps HSM compliance to old named tier levels.
Candidate: `faqs/deliverability-engineer-faq.md:15` asks why IP pools differ across old named tiers.
Candidate: `faqs/deliverability-engineer-faq.md:17` describes early-trial tenants through old named tiers.
Candidate: `faqs/deliverability-engineer-faq.md:69` references an old named service tier.
Candidate: `faqs/deliverability-engineer-faq.md:106` references tenants on a tier.
Candidate: `PRD.md:54` uses sovereign tier for Postal.
Candidate: `PRD.md:107` uses sovereign tier in a success metric.
Candidate: `PRD.md:136` uses `Tier` as a provider matrix column.
Candidate: `PRD.md:151` uses sovereign-tier for KSA/UAE packs.
Candidate: `PRD.md:165` uses storage tier language.
Candidate: `ARCHITECTURE.md:43` uses service-owner tier metadata.
Candidate: `ARCHITECTURE.md:202` says `Tier-substrate`.
Candidate: `ARCHITECTURE.md:312-313` uses Tier-1 and Tier-3 placement.
Candidate: `ARCHITECTURE.md:919` says paid bulk API tier.
Candidate: `ARCHITECTURE.md:1041-1051` uses cell tier and T0/T1/T2/T3 placement.
Candidate: `contracts/openapi.yaml:59` says Tier-A invariant.
Candidate: `contracts/openapi.yaml:70` says Tier-A 4-INV pattern.
Candidate: `contracts/openapi.yaml:90` says Tier-A invariant.
Candidate: `contracts/openapi.yaml:186` says Tier-A 4-INV.
Candidate: `contracts/openapi.yaml:229` says Tier-A 4-INV.
Candidate: `cost-budget.md:40-45` uses sovereign-tier budget language.
Candidate: `capacity-model.md:80-82` says sovereign tier remains steady.
Candidate: `threat-model.md:131-132` says sovereign-tier datacenter.
Candidate: `runbooks/postal-failover.md:4-5` uses sovereign-tier severity.
Candidate: `runbooks/postal-failover.md:33-55` uses sovereign-tier paths and anti-patterns.
Candidate: `tutorials/send-1m-transactional-campaign-with-warmup.md:15` requires an old named tier cell.
Candidate: `scorecards/overrides.json:4` stores `tier_target`.
Candidate: `iac/terraform-module.tf:32` labels a cell tier.
Candidate: `iac/helm/postal/Chart.yaml:4-5` describes sovereign tier.
Candidate: `iac/helm/postal/Chart.yaml:31` stores `oyatie.io/sovereign-tier`.
Candidate: `iac/helm/postal/values.yaml:1` comments sovereign tier.
Candidate: `onboarding/deliverability-engineer-first-week.md:16` references non-existent deliverability tier ADR.
Candidate: `reference-implementations/send-transactional-rust-sdk.md:244` says lower-tier path.
Candidate: `IP-002-postal-adapter-impl.md:12-20` uses sovereign tier.
Candidate: `IP-005-dkim-key-rotation-pipeline.md:59` references Tier-B OpenTofu DNS.
Candidate: `IP-014-sovereign-pack-postal-only-enforcement.md:45` references `oya-check-iac-tier-discipline`.
Candidate: `IP-journey-j93-in-dpdpa-rbi-overlay.md:43-67` repeats merchant KYC tiering in journey tasks.
Retirement conclusion: the path still contains a large retired-tier corpus and needs a Wave 15J cleanup pass.

#### §3.4.C — Tenant-class adoption gaps

The target tenant-class vocabulary for this audit is `demo_trial`, `paid`, and `revenue_share`.
No service artifact declares a `tenant_class` field.
Evidence: `rg tenant_class microservices/comms-email`.
No service artifact declares `demo_trial`.
Evidence: `rg demo_trial microservices/comms-email`.
No service artifact declares `revenue_share`.
Evidence: `rg revenue_share microservices/comms-email`.
No service artifact declares OCI Always Free profile behavior under `demo_trial`.
Evidence: `rg 'Always Free|always-free' microservices/comms-email`.
The word `paid` appears only in an architecture phrase about a paid bulk API tier, not tenant-class licensing.
Evidence: `ARCHITECTURE.md:919`.
The service already has per-tenant rate ceilings.
Evidence: `PRD.md:29-31`, `capacity-model.md:14-19`.
The service already has per-tenant budget caps.
Evidence: `cost-budget.md:25-30`.
The service already has tenant-scoped domains, credentials, suppression, policy, and audit.
Evidence: `manifest.json:134-145`, `decisions/ADR-CME-001-per-tenant-signing-key-custody-with-rotation-cadence.md:33-42`.
The service therefore has the substrate primitives needed for tenant classes.
The gap is naming, schema, policy, and performance overlay adoption.
Finding linkage: F-06.

### §3.5 Dimension 5 — API, contracts, and event coherence

Assessment: partial pass; core contract is real but not aligned to broadened product.
The OpenAPI contract has `/v1/messages` for sending.
Evidence: `contracts/openapi.yaml:27-55`.
The OpenAPI contract has `/v1/bounces`.
Evidence: `contracts/openapi.yaml:56-85`.
The OpenAPI contract has `/v1/suppressions`.
Evidence: `contracts/openapi.yaml:87-115`.
The OpenAPI contract has `/v1/webhooks/{provider}`.
Evidence: `contracts/openapi.yaml:117-139`.
The OpenAPI contract has `/v1/tenants/{tenant_id}/from-domains`.
Evidence: `contracts/openapi.yaml:140-162`.
The OpenAPI schema captures idempotency and audit headers.
Evidence: `contracts/openapi.yaml:166-186`.
The AsyncAPI contract defines delivery event, bounce classified, suppression inserted, DKIM rotated, and from-domain state channels.
Evidence: `contracts/asyncapi.yaml:18-46`.
The AsyncAPI delivery event includes provider enum SES, Postal, Mailgun, and SMTP.
Evidence: `contracts/asyncapi.yaml:84-105`.
The proto exposes send, domain, and DKIM operations.
Evidence: `contracts/comms_email.proto:11-17`.
No contract exposes inbound message retrieval from `IP-023`.
Evidence: `IP-023-inbound-receiver-rest.md:15-21`, `contracts/openapi.yaml:27-162`.
No contract exposes list-management routes from `IP-024`.
Evidence: `IP-024-list-management-rest.md:15-22`, `contracts/openapi.yaml:27-162`.
No contract exposes reputation routes from `IP-025`.
Evidence: `IP-025-reputation-monitor-rest-and-dashboard.md:16-24`, `contracts/openapi.yaml:27-162`.
No contract exposes unsubscribe event channel from `IP-026`.
Evidence: `IP-026-unsubscribe-async-emit.md:16-21`, `contracts/asyncapi.yaml:18-46`.
The contracts are coherent for the older transactional substrate.
The contracts are incomplete for the broadened service.
Finding linkage: F-07.

### §3.6 Dimension 6 — SLO, capacity, performance, and operational readiness

Assessment: strong operational doc set; benchmark model must be re-cut without retired tiers.
The capacity model defines p50, p99, and peak sends per second per cluster.
Evidence: `capacity-model.md:12-19`.
The capacity model defines Phase 1 replica CPU and memory needs.
Evidence: `capacity-model.md:21-29`.
The capacity model defines storage needs for suppression, idempotency, and audit buffer.
Evidence: `capacity-model.md:30-36`.
The capacity model defines p99 budgets for preflight, render, suppression, DKIM, provider call, and total send.
Evidence: `capacity-model.md:45-56`.
The capacity model claims 2x headroom at 10k sends/s peak.
Evidence: `capacity-model.md:58-64`.
The SLO set includes p99 send latency <= 500 ms.
Evidence: `slos/send-latency-p99.openslo.yaml:12-44`.
The SLO set includes send success >= 99.9%.
Evidence: `slos/send-success-rate.openslo.yaml:12-40`.
The SLO set includes deliverability >= 99.5%.
Evidence: `slos/deliverability-rate.openslo.yaml:12-34`.
The SLO set includes DKIM signed rate >= 99.99%.
Evidence: `slos/dkim-signing-rate.openslo.yaml:12-34`.
The SLO set includes DMARC alignment >= 99%.
Evidence: `slos/dmarc-alignment-rate.openslo.yaml:12-35`.
The SLO set includes webhook audit success >= 99.99%.
Evidence: `slos/webhook-success-rate.openslo.yaml:12-36`.
The old benchmark report has useful metrics but labels Oyatie rows with retired named tiers.
Evidence: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:17-111`.
The performance deliverable rewrites targets as one industry-leader target set with deployment-context and tenant-class overlays.
Finding linkage: F-10.

### §3.7 Dimension 7 — Security, privacy, compliance, and abuse posture

Assessment: pass with caveats tied to product-scope drift.
The threat model names six trust boundaries.
Evidence: `threat-model.md:7-15`.
The threat model identifies DKIM private keys, provider credentials, suppression lists, audit entries, templates, and webhook secrets.
Evidence: `threat-model.md:16-24`.
The threat model mitigates DKIM exfiltration, tampering, skipped rotation, and tenant cross-spoofing.
Evidence: `threat-model.md:27-37`.
The threat model covers from-domain spoofing, shared-provider cross-tenant leakage, DKIM downgrade, bounce storm, and SES quota exhaustion.
Evidence: `threat-model.md:75-124`.
The DPIA identifies recipient email, names, links, account references, behavior telemetry, and PHI in packs.
Evidence: `dpia.md:47-57`.
The DPIA retention model excludes message-body storage by the substrate and keeps audit chain by ADR-0145.
Evidence: `dpia.md:72-83`.
The compliance doc maps CAN-SPAM unsubscribe handling.
Evidence: `compliance.md:8-17`.
The compliance doc maps GDPR lawful basis, erasure, access, and residency.
Evidence: `compliance.md:18-55`.
The compliance doc maps HIPAA BAA providers and PHI handling.
Evidence: `compliance.md:65-75`.
The compliance doc still says inbound email ingestion ADR is deferred.
Evidence: `compliance.md:111-117`.
That is inconsistent with inbound implementation plans and runbooks.
Evidence: `IP-016-inbound-receiver-kernel.md:11-23`, `runbooks/inbound-receiver-quarantine-release.md`.
The security surface is substantive.
The compliance posture must be re-opened if inbound/list/marketing remain in scope.
Finding linkage: F-11.

### §3.8 Dimension 8 — Cross-microservice dependencies and handoffs

Assessment: partial pass; dependency list exists, but handoff file is missing.
The manifest depends on tenancy, identity, observability, cell, audit-chain, intelligence, detection, and cloud-iac.
Evidence: `manifest.json:134-142`.
The architecture says cross-service dependencies include tenancy, identity, policy-engine, observability, audit-chain, cloud-secrets, cell, and cloud-iac.
Evidence: `ARCHITECTURE.md:51`, `ARCHITECTURE.md:154-160`, `ARCHITECTURE.md:1072-1077`.
The PRD says all other µservices consume the email substrate through one trait.
Evidence: `PRD.md:10-17`, `PRD.md:40-42`.
The migration playbook maps source-provider webhooks into Oyatie event fields.
Evidence: `migration-playbooks/from-sendgrid-and-mailgun.md:138-160`.
The explicit `cross-microservice-handoffs.md` file is absent.
Evidence: `test -e microservices/comms-email/cross-microservice-handoffs.md`.
The missing file matters because this service sits on the boundary of tenancy, identity, audit-chain, cloud-iac, and multiple caller µservices.
The existing architecture contains dependency prose but not a single handoff contract.
Finding linkage: F-12.

### §3.9 Dimension 9 — Verification, buildability, and audit readiness

Assessment: partial pass; audit docs are rich, but executable verification is not yet local.
The reference implementation is Rust and demonstrates send, idempotency, event subscription, and error handling.
Evidence: `reference-implementations/send-transactional-rust-sdk.md:13-248`.
The source README says the live code lands in workspace crates and follow-up implementation PRs.
Evidence: `src/README.md:45-47`.
The service path has no forbidden backend language files.
Evidence: forbidden-language `find` command output.
The service path has no local tests.
Evidence: tests `find` command output.
The service has SLO query definitions but no service-local CI or test harness in this path.
Evidence: recursive inventory in §2.
The benchmark report references a harness path that is not present in this µservice directory.
Evidence: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:99-111`.
The docs can support planning and ownership.
The docs cannot, by themselves, prove production behavior.
Finding linkage: F-08 and F-10.

## §4 Findings table

| ID | Severity | Finding | Evidence | Required correction |
|---|---|---|---|---|
| F-01 | P1 | Product scope contradicts itself: PRD says transactional-only/deferred inbound/list; README/IPs define marketing, inbound, list, unsubscribe, and reputation. | `PRD.md:57-65`, `PRD.md:92-98`, `README.md:13-21`, `IP-016-inbound-receiver-kernel.md:11-23`, `IP-018-list-management-usecase.md:11-24` | Decide the service boundary and update PRD, README, manifest, contracts, compliance, and parity docs together. |
| F-02 | P1 | Six deployment-context IaC modules are absent. | `specs/master-plan-sequencing.json:704-745`, `manifest.json:86-89`, `find microservices/comms-email/iac -maxdepth 3 -type d` | Add or explicitly mark N/A for all canonical context modules. |
| F-03 | P1 | IaC uses Terraform naming and block semantics instead of OpenTofu context layout. | `specs/master-plan-sequencing.json:747-775`, `iac/terraform-module.tf:1-10` | Replace with OpenTofu module layout under canonical context paths. |
| F-04 | P1 | OS support manifest is missing. | `specs/master-plan-sequencing.json:777-815`, recursive inventory in §2 | Add `supported-oses.json` with supported, test-only, and out-of-scope entries. |
| F-05 | P1 | OCI Always Free profile module is absent. | `specs/master-plan-sequencing.json:857-867`, `find microservices/comms-email/iac -maxdepth 3 -type d` | Add `iac/oci-guest/always-free/` or declare context N/A with evidence. |
| F-06 | P2 | Tenant-class model is not adopted. | `ARCHITECTURE.md:919`, `rg tenant_class|demo_trial|revenue_share|Always Free|always-free|paid microservices/comms-email` | Add `tenant_class` semantics and overlays for `demo_trial`, `paid`, and `revenue_share`. |
| F-07 | P1 | Contracts only match old transactional surface, not inbound/list/reputation/unsubscribe plans. | `contracts/openapi.yaml:27-162`, `contracts/asyncapi.yaml:18-46`, `IP-023-inbound-receiver-rest.md:15-21`, `IP-024-list-management-rest.md:15-22` | Align OpenAPI/AsyncAPI/proto with the decided product boundary. |
| F-08 | P2 | Local implementation and test evidence are not present in the µservice path. | `src/README.md:1-11`, `src/README.md:45-47`, test find output | Link or land crate/test evidence so audit claims can be executed. |
| F-09 | P2 | Existing competitor parity doc lags counterpart union coverage. | `competitor-parity-matrix.md:57-64`, `README.md:17-21`, official SendGrid/Postmark/Mailgun docs cited in feature parity report | Replace parity baseline with current union-coverage matrix. |
| F-10 | P2 | Benchmark report uses retired named tiers and old `--tenant-class` command. | `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:13-111` | Replace with single target set and deployment-context plus tenant-class overlays. |
| F-11 | P2 | Compliance/DPIA posture still treats inbound as future while IPs/runbooks make it active scope. | `compliance.md:111-117`, `dpia.md:104-105`, `IP-016-inbound-receiver-kernel.md:11-23` | Reopen compliance/DPIA for inbound/list/marketing if scope is retained. |
| F-12 | P2 | Cross-microservice handoff contract is missing despite many dependencies. | `manifest.json:134-142`, `ARCHITECTURE.md:154-160`, missing `cross-microservice-handoffs.md` check | Add handoff contract or merge equivalent machine-readable handoff fields into manifest. |
| F-13 | P2 | Retired tier corpus remains in service docs, capability records, benchmarks, FAQs, runbooks, IaC labels, and scorecards. | §3.4.T candidate list | Execute Wave 15J retirement cleanup without reintroducing tier deltas. |
| F-14 | P3 | Proto contains a `go_package` option even though service backend policy is Rust-strict. | `contracts/comms_email.proto:1-17`, forbidden-language find output | Confirm whether codegen metadata is allowed; if not, replace with Rust-oriented generation metadata. |

Finding counts: P0 = 0, P1 = 6, P2 = 7, P3 = 1.

## §5 Open questions

1. Is `comms-email` still transactional-only, or is it now the full application-email substrate including inbound, lists, unsubscribe, reputation, and bulk sending?
2. If marketing/list management remains in scope, which µservice owns campaign orchestration, A/B testing, segmentation, and WYSIWYG editing?
3. Should inbound receiving be Phase 1 or Phase 2, given PRD/compliance deferral and active IP/runbook surfaces?
4. Should `catalog/bounded-contexts.json` be expanded to match IP-016 through IP-026, or should those IPs move to a future-scope folder?
5. Which canonical machine-readable field should replace `capability_profiles` in `manifest.json` for uniform-quality tenant classes?
6. Does `demo_trial` for this service mean reduced usage ceilings only, or also constrained provider choices under the OCI Always Free profile?
7. How should `revenue_share` tenants express at-cost substrate usage without product-quality degradation?
8. Is the old `iac/terraform-module.tf` salvageable by renaming and restructuring as OpenTofu, or should it be replaced per context?
9. Which deployment contexts genuinely support self-hosted Postal, and which use provider adapters only?
10. Does on-prem/colo require explicit DKIM/SPF/DMARC/abuse/egress evidence beyond the current Postal chart?
11. Should the proto keep a `go_package` option for external tooling, or should generator metadata be Rust-first?
12. Where are the workspace crates named in `src/README.md`, and how should this service path link to their tests?
13. What is the expected OS matrix for Postal, SES adapter, and future in-house relay across Linux, BSD, Windows, and macOS support classes?
14. Should the old benchmark report be retired or kept as historical evidence after the performance deliverable lands?
15. Should the service retain Postmark as only a migration/counterpart source, or add a Postmark adapter to match counterpart parity?
16. Should SendGrid remain a rejected canonical provider while still supporting migration inventory?
17. How should pack overlays reconcile regional provider availability, especially KR where current compliance says Postal-only?
18. Which canonical file owns tenant-class usage caps, billing hooks, and usage-based/rate ceilings?
19. Should `cross-microservice-handoffs.md` be created despite the user's current no-shared-docs directive, or should the handoff live in `manifest.json` only?
20. What verifier should replace old `oya-check-iac-tier-discipline` references after Wave 15J retirement?

<!-- ORCHESTRATOR REPORT
  µservice: comms-email
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/comms-email/coherence-audit-2026-05-20.md (639 lines)
    - /Users/jasonlee/oyatie/microservices/comms-email/feature-parity-matrix-2026-05-20.md (430 lines)
    - /Users/jasonlee/oyatie/microservices/comms-email/performance-benchmark-numbers-2026-05-20.md (405 lines)
  inventory_files_seen: 136
  inventory_lines_read: 14645
  chat_history_matches_processed: 11
  findings_p0: 0
  findings_p1: 6
  findings_p2: 7
  findings_p3: 1
  customer_class_ladder_retirement_candidates_found: 68; cites: tenant_class model in ADR-0330:2,9,11,13,30,49,51,81,83,110,114,116,127,129,133,136,142,144; manifest.json:44-48,60-64,96-101,118,145; capabilities/T0-transactional-send.json:2-3; capabilities/T1-bounce-handle.json:2-3; capabilities/T1-webhook-delivery-event.json:2-3; capabilities/T2-list-manage.json:2-3; capabilities/T2-tenant-domain-mgmt.json:2-3; capabilities/T3-inbound-receive.json:2-3; benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:13,21,36,49,58,87,106; faqs/deliverability-engineer-faq.md:13,15,17,69,106; PRD.md:54,107,136,151,165; ARCHITECTURE.md:43,202,312-313,919,1041-1051; contracts/openapi.yaml:59,70,90,186,229; cost-budget.md:40-45; capacity-model.md:80-82; threat-model.md:131-132; runbooks/postal-failover.md:4-5,33-55; tutorials/send-1m-transactional-campaign-with-warmup.md:15; scorecards/overrides.json:4; iac/terraform-module.tf:32; iac/helm/postal/Chart.yaml:4-5,31; iac/helm/postal/values.yaml:1; onboarding/deliverability-engineer-first-week.md:16; reference-implementations/send-transactional-rust-sdk.md:244; IP-002-postal-adapter-impl.md:12-20; IP-005-dkim-key-rotation-pipeline.md:59; IP-014-sovereign-pack-postal-only-enforcement.md:45; IP-journey-j93-in-dpdpa-rbi-overlay.md:43-67
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/revenue_share/OCI Always Free profile semantics found, only an unrelated paid bulk API phrase at ARCHITECTURE.md:919
  top_3_counterparts_confirmed: SendGrid / Postmark / Mailgun
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1474
-->
