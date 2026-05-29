---
doc_class: Architecture
microservice: contract-lifecycle-management
status: wave-4-rolling-remediated
date: 2026-05-21
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0244
  - ADR-0245
  - ADR-0247
  - ADR-0248
  - ADR-0251
  - ADR-0253
  - ADR-0255
  - ADR-0263
  - ADR-0314
  - ADR-0321
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-CLM-001
companion_docs:
  - microservices/contract-lifecycle-management/PRD.md
  - microservices/contract-lifecycle-management/compliance.md
  - microservices/contract-lifecycle-management/manifest.json
  - microservices/contract-lifecycle-management/REMEDIATION-NOTES-2026-05-21.md
---

# Architecture: Contract Lifecycle Management

## A. Boundary

Contract Lifecycle Management owns contract state, clause controls, obligations, approvals, renewal risk, and per-jurisdiction signature evidence. The µservice does not own tenant identity (identity µservice), Cedar policy engine internals (policy-eval), workflow runtime internals (workflow-engine), ontology storage (ontology), payments rails (payments), marketplace settlement (marketplace), or adjacent product labels.

Per ADR-0329, the µservice does not stratify capability by retired named capability levels tiers. Every deployment ships the full canonical capability surface; tenant_class (demo_trial / paid) × deployment_context (6 contexts) × jurisdiction_pack are the canonical differentiation axes per ADR-0330 + ADR-0331.

## B. Layer Map

Per ADR-0105 9-layer canonical enum:

| Layer | Planned responsibility |
|---|---|
| api | public command/query DTOs and OpenAPI 3.2.0 contract binding |
| rest | HTTP/3-first transport, idempotency enforcement, request validation |
| application | usecase orchestration and transaction boundaries |
| usecase | command handlers, read models, migration dry-runs, replay flows |
| domain | aggregate invariants and state transitions |
| kernel | pure value objects, policy-port traits, deterministic calculations (e.g. OOXML diff, signature envelope construction, due-basis grammar evaluation) |
| adapter | source-system, storage, queue, HSM, TSA, e-signature provider, AI LLM adapters |
| worker | async import, replay, reconciliation, CRDT coordinator, retention enforcer, TSA LTA renewal worker |
| governance | policy, compliance, scorecards, evidence gates, pack-overlay resolver |

## C. Bounded Context Architecture

### contract-intake
- Aggregate root: `contract_intake_document`.
- Invariants: tenant scope required; version monotonic; source-system provenance immutable; destructive correction forbidden; counterparty resolved via MDM before draft creation; jurisdiction pack frozen at contract execution.
- Commands: create, amend, approve, import, export, replay, archive, reverse where applicable.
- Events: created, amended, approved, import-accepted, import-rejected, replayed, exported, archived, reversed.
- Read model: tenant-scoped projection keyed by document id, source-system id, status, data class, region, jurisdiction pack, workflow run.

### clause-library
- Aggregate root: `clause_library_document`.
- Invariants: three-tier inheritance per `legal-dimensions/clause-library-inheritance.md`; template version Merkle-rooted; fallback positions enumerated; SOX-404 segregation enforced at template activation.
- Commands: create, amend, approve, import, export, replay, archive, reverse.
- Events: template-added, template-versioned, template-used, fallback-invoked, prohibited-modification-blocked.

### negotiation
- Aggregate root: `negotiation_document`.
- Invariants: edit blocked once OutForSignature; redline provenance preserved (IP-029); collaborative editing via Loro CRDT pre-signature only.
- Commands: create, amend, approve, import, export, replay, archive.
- Events: redline-added, redline-classified, fallback-invoked, counterparty-edited, converged.

### obligation
- Aggregate root: `obligation_document`.
- Invariants: IP-027 confidence bands gate auto-propose; due-basis grammar deterministic; force-majeure suspension pauses but does not extend.
- Commands: create, acknowledge, satisfy, dispute, resolve, waive, suspend, terminate.
- Events: pending, acknowledged, active, satisfied, overdue, cured, disputed, resolved, waived, suspended, terminated.

### renewal
- Aggregate root: `renewal_document`.
- Invariants: IP-028 explainability board produces feature attributions; renewal risk score deterministic from contract version hash + model id; auto-renewal cure period blocks default-renewal.
- Commands: risk-score, alert, renegotiate, renew, terminate.
- Events: risk-scored, alert-fired, renegotiation-window-opened, renewed, auto-renewed, terminated.

## D. Integration Topology

Per audit X-D5, the integration topology is expanded to include the dependencies the audit identified as missing:

- **identity**: principal authentication; tenant_class principal claim; SPIFFE/SVID workload identity.
- **drive**: contract artefact storage (.docx / PDF blobs); tenant-scoped buckets.
- **workflow-engine**: approval routing; renewal cadence; force-majeure notification workflows.
- **workplace-integration**: email/Slack/Teams notifications.
- **ontology**: legal-entity projection; counterparty ↔ crm.account resolution.
- **audit-chain**: ADR-0263 audit event sealing.
- **marketplace**: DealSet settlement per ADR-0314.
- **payments**: commercial obligation settlement.
- **kms**: HSM-resident QES signing keys; key rotation; TSA certificate management.
- **calendar**: notice-and-cure cure-period reminders; renewal alert scheduling.
- **mail**: signed-URL email delivery; consumer disclosure delivery; counterparty notification.
- **intelligence**: AI clause suggestion via cross-emit per `legal-dimensions/ai-redlining-prompt-template.md`.

Each integration uses ADR-0145 direct gRPC over HTTP/3 (no shared database; no event-bus coupling for synchronous calls).

## E. Failure Modes

Per `failure-modes.md`:

- Source-system import drift: dry-run evidence identifies row, field, transform, data class, rejection reason.
- Cross-tenant reference attempt: Cedar denies before domain command execution; refusal evidence emitted.
- Duplicate command submission: idempotency key returns prior result; duplicate metric incremented.
- Regional outage: writes queue in tenant home cell; reads expose stale-region metadata.
- Audit-chain outage: critical state transitions pause; non-critical reads continue with degraded banner.
- Pack conflict: pack resolver blocks activation; opens workflow-engine remediation task.
- HSM unavailability: signature seal blocks (no provisional QES); audit event records the outage; signatures queued for HSM recovery.
- TSA outage: signature sealed with provisional timestamp; LTA timestamp back-filled within 24 hours.
- AI provider outage: AI redlining suggestions fall back to next-priority provider; UI surfaces "AI provider degraded" banner.
- Counterparty MDM lookup failure: contract creation blocks until counterparty resolved or manually created.
- WORM provider outage: signature packets accumulate in staging; alert raised; alarm fires at 24-hour buffer threshold.

## F. Required ADR-3.2.1 Anchors

The audit identified the §F anchor section as 13 stamped depth-detail blocks (~208 identical-structure bullets) replicating the same template. The remediation replaces those blocks with substantive ADR-3.2.1 anchor content specific to CLM legal-domain concerns.

### F.1 Principals

The µservice authenticates principals via the identity µservice. Mandatory request fields per ADR-0244 + ADR-0321: `tenant_id`, `principal_id`, `tenant_class` ∈ {demo_trial, paid}, `audience_type` ∈ {tenant-b2b-legal, tenant-b2c, internal-operator, auditor, automated-worker}, `home_cell`, `jurisdiction_code`, `audit_event_class`.

Principal roles: contracts_manager, legal_review, general_counsel, compliance_officer, deal_desk, dpo, hipaa_privacy_officer, sox_compliance_officer, auditor, outside_counsel, dtp (designated third party for SEC 17a-4), board_member, ceo, cfo, ai_inference_principal (per ADR-0247 Foundry self-modification under Cedar).

Per ADR-0263, every principal authentication emits `oya.contract.lifecycle.management.principal.authenticated` with tenant_class + jurisdiction_pack dimensions; failed authentications emit `oya.contract.lifecycle.management.principal.authentication.denied`.

External precedent: AWS IAM service-linked roles + Google Cloud service agents. Oyatie's principal model is tenant-scoped (no cross-tenant principal) and adds the `tenant_class` claim that gateway IAM enforces transparently.

### F.2 Cedar gates

Cedar v4.2 LTS default-deny per ADR-0243 evaluated before storage / provider / cross-µservice access. Per-aggregate policies in `policy/contract-obligation-authorization.cedar` + scoped policies in `policy/auditor-scope.cedar`, `policy/ci-scope.cedar`, `policy/abuse-defence.cedar`, `policy/emergency-services-bypass.cedar`.

Tenant-class gates per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`:

- demo_trial blocked from QES signature seal (per `packs/eidas/README.md`).
- demo_trial blocked from AI redlining inference (per `legal-dimensions/ai-redlining-prompt-template.md`).
- demo_trial document-size cap 100 KB.
- demo_trial retention cap 30 days.
- demo_trial maximum 5 active contracts per tenant.

Pack-conditional gates per `packs/<pack>/README.md` examples (full set in policies/):

- `forbid (principal, action == "ContractDelete", resource) when { resource.retention_remaining_days > 0 }`
- `forbid (principal, action == "ContractApprove", resource) when { resource.author_principal_id == principal.principal_id }`
- `forbid (principal, action == "SignaturePacketSeal", resource) when { resource.required_signature_level == "QES" && resource.tsa.lotl_qualified == false }`

External precedent: AWS Verified Permissions Cedar + Google Zanzibar. Oyatie's Cedar usage is tenant-scoped + pack-composable + audit-emitted on deny.

### F.3 Tenant scoping

ADR-0244 tenant-scoping primitive: every row, audit event, signature envelope, redline event, obligation, cost line carries `tenant_id`. Tenant-scoped projection at the storage layer prevents cross-tenant leakage by construction.

Per `feedback_oyatie_is_a_tenant_doctrine`, the Oyatie itself is a reserved-namespace tenant; the µservice's own contracts (Oyatie corporate vendor contracts) flow through CLM as a `tenant_id=oyatie` tenant subject to the same scoping rules.

### F.4 Substrate-product binding

CLM is product (per ADR-0245 substrate-vs-product layering). It composes substrate dependencies (identity, drive, workflow-engine, ontology, audit-chain, kms, calendar, mail, marketplace, payments, workplace-integration, intelligence) but owns its bounded contexts.

Substrate dependencies are called via ADR-0145 direct gRPC; no shared database; no event-bus synchronous coupling. Async events are AsyncAPI 3.1.0 per channel definitions in `contracts/asyncapi-v1.yaml`.

### F.5 Observability

Per ADR-0263 observability emission contract: every state transition emits an audit-chain event with mandatory dimensions:

- tenant_id
- tenant_class
- principal_id
- deployment_context
- home_cell
- jurisdiction_pack
- data_class
- audit_event_class
- trace_id
- policy_decision_id

Audit events route to `audit-chain` substrate; metrics route to OTEL collector; traces route to Tempo/Jaeger; logs route to Loki/Elasticsearch.

CLM-specific audit-event classes: contract.intake, clause.library, negotiation, obligation, renewal, signature, consent, legal_hold, counterparty_mdm, privilege, pack, migration. Full enumeration in audit-event-catalog.md (per audit-chain substrate's catalog convention).

### F.6 Packs and residency

Pack composition per `packs/README.md`: 9 compliance packs + 17+ jurisdiction packs compose multiplicatively. Higher-restriction-wins on every dimension (retention, residency, signature evidence level, consent, breach notification, audit export, UI disclosure, workflow approvals).

Residency: per ADR-0330 deployment_context × jurisdiction_pack composition. KR-PIPA pack triggers Korean cell residency; eu-eidas-qes triggers EU cell; HIPAA-Provider triggers US-only.

### F.7 Detection

Abuse, policy, insider, and anomaly signals route to detection / investigation through ADR-0263 audit events. CLM-specific detection signals:

- Counterparty sanctions match.
- High-velocity signature requests (potential signature-impersonation attack).
- Bulk contract download by single principal.
- Pre-signature redline by principal who is also approver (segregation violation).
- Unusual approval-route bypass attempt.
- AI inference with PII-stripping disabled.
- TSA non-qualified for jurisdiction.

### F.8 Critical-path

Critical paths (cannot be downgraded under load):

- Signature envelope seal.
- Audit-chain event emission.
- Cedar policy evaluation.
- Legal hold preservation enforcement.
- WORM-locked object retention enforcement.

Non-critical paths (may be deferred under load):

- AI redlining suggestions.
- Renewal risk score refresh.
- Counterparty MDM background refresh.
- Dashboard query.

### F.9 Abuse defence

Edge WAF rules in `iac/<context>/edge-waf.yaml` + abuse Cedar fragment in `policy/abuse-defence.cedar`. Abuse-defence detects:

- Signature impersonation attempts.
- Mass-counterparty enumeration.
- Cross-tenant principal token abuse.
- Bulk export by a principal beyond normal envelope.

### F.10 Transport

Per ADR-0253 HTTP/3 + QUIC default everywhere. TLS 1.3 floor. ECH where terminated. PQC hybrid (X25519+ML-KEM-768) where negotiated. gRPC over HTTP/3 for substrate calls.

### F.11 Provider credentials

Per ADR-0255 §D-4 + `feedback_byok_everywhere_credentials`:

- E-signature provider credentials: `provider_credential_modes.e_signature ∈ {platform_default, byok}`. Tenants may bring their own DocuSign / Adobe Sign / HelloSign / OneSpan account credentials.
- HSM QES credentials: `provider_credential_modes.hsm_qes ∈ {platform_default, byok, byok_required_by_pack}`. Sovereign-cell tenants may be `byok_required_by_pack`.
- TSA credentials: `provider_credential_modes.tsa ∈ {platform_default, byok}`.
- AI LLM credentials: `provider_credential_modes.ai_llm ∈ {platform_default, byok}`.

Credentials are bound from OpenBao with ≤60s sidecar TTL.

### F.12 Self-modification

Per ADR-0247, the Foundry pipeline operates on the CLM µservice as a tenant principal `oyatie.foundry.*`. Foundry self-modifications evaluated by Cedar (same default-deny gate as any other principal). Foundry modifications emit audit events.

### F.13 Cellular topology

Per ADR-0248 Amazon-shape cellular architecture. CLM cell tiers eligible: tier-1, tier-2 (per `manifest.json` cell_eligibility). Each cell isolated; shuffle sharding allocates tenants to cells. Cell tier-0 is sovereign-pack-bound (eu-eidas-qes, kr-pipa-sovereign). Cell tier-2 supports demo_trial (no contractual SLO).

Pod runtime: Cloud Hypervisor + Kata containers per ADR-0254. Kubernetes everywhere (except edge nodes).

## G. Data model (canonical)

The canonical contract data model is anchored in ADR-CLM-001. Key aggregates:

- `ContractPacket`: stable legal identity; immutable; versioned via Merkle hash.
- `ClauseVersion`: immutable normalized clause + source-span map.
- `RedlineEvent`: counterparty edits + comments + accept/reject + provenance (IP-029).
- `ObligationFact`: source span + due basis + owner role + confidence band (IP-027).
- `RenewalRiskFact`: projection over obligations + notice dates + counterparty history + clause deviations (IP-028).
- `SignaturePacket`: SignatureEnvelope[] + intent evidence + provider envelope id (IP-030).
- `LegalHoldRecord`: per `state-machines/legal-hold-state-machine.md`.
- `ConsentRecord`: per `legal-dimensions/gdpr-article-7-consent-records.md`.
- `Counterparty`: per `counterparty-mdm/counterparty-mdm.md`.
- `ApprovalEvidence`: per `legal-dimensions/approval-routing-matrix.md`.

## H. State machines

Four state machines anchor the µservice:

- `state-machines/contract-state-machine.md` (Draft → Review → Approved → OutForSignature → Signed → Effective → Amended | Renewed | InDispute → Terminated | Settled).
- `state-machines/obligation-state-machine.md` (Pending → Acknowledged → Active → Satisfied | Overdue → Cured | Disputed | Waived | Suspended | Terminated).
- `state-machines/redline-turnaround-state-machine.md` (InternalDraft → SentForRedline → CounterpartyEdited → InternalReview → CounterRedline → ResubmittedToCounterparty → ConvergedToFinal → ApprovalRouting → Signed).
- `state-machines/legal-hold-state-machine.md` (NORMAL → HOLD_APPLIED → LITIGATION_PARTY_IDENTIFIED → PRESERVATION_OBLIGATION_ACTIVE → HOLD_RELEASED_WITH_AUDIT).

## I. Deployment contexts

Per ADR-0330 + `feedback_multi_context_provider_agnostic_2026_05_20`, the µservice runs in 6 deployment contexts. Each has its own OpenTofu module under `iac/<context>/`:

- `oyatie-public-cloud` (Oyatie's hosted SaaS).
- `aws-guest` (customer's AWS account).
- `oci-guest` (customer's OCI account; demo_trial uses Always Free).
- `on-prem` (customer's bare-metal; air-gap possible).
- `colo` (customer's colocation).
- `oyatie-as-cloud-provider` (Oyatie as IaaS provider for downstream tenants).

## J. OS support matrix

Per `feedback_os_support_matrix_2026_05_20`: 13 supported OSes + linux/amd64 + linux/arm64 + darwin/arm64 (M5+) + Tier-2 linux/ppc64le + linux/s390x. Full list in `manifest.json` `supported_oses` + `arch_matrix` + `package_shapes`.

## K. Tenant-class doctrine

Per ADR-0331 + `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`:

- `tenant_class ∈ {demo_trial, paid}`.
- `paid.billing_components ⊆ {per_seat, per_usage, revenue_share}`.
- `tenant_class` is a principal claim; gateway-IAM-enforced; not in request bodies.
- Cedar gates enforce per-class behavior caps.
- Audit events include tenant_class dimension.

## L. Companion docs

See PRD.md §N for the full companion-doc inventory.

## M. Wave 15A remediation closure

This architecture document was fully rewritten as part of Wave 15A CLM remediation. The prior 13 stamped Content-pass expansion anchor blocks (totaling ~208 identical-structure depth-detail bullets) were replaced with the bespoke § F anchor content above. The `tier `product`` reference (27 file:line citations in the prior version) was eliminated; the canonical category is now `product` (not a tier label) per `manifest.json`. See `REMEDIATION-NOTES-2026-05-21.md` for the full remediation log.
