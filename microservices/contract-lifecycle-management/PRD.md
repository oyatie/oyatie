---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-contract-lifecycle-management
microservice: contract-lifecycle-management
status: wave-4-rolling-remediated
date: 2026-05-21
owner_team: axis-contract-lifecycle-management + council-product
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
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
  - ADR-CLM-001
companion_docs:
  - microservices/contract-lifecycle-management/ARCHITECTURE.md
  - microservices/contract-lifecycle-management/compliance.md
  - microservices/contract-lifecycle-management/manifest.json
  - microservices/contract-lifecycle-management/decisions/ADR-CLM-001-clause-obligation-ledger-and-redline-provenance.md
  - microservices/contract-lifecycle-management/REMEDIATION-NOTES-2026-05-21.md
planned_enforcement_ref: oya-governance-contract-lifecycle-management-doc-suite
---

# PRD-contract-lifecycle-management: Contract Lifecycle Management

## A. Problem

Contract Lifecycle Management closes B2B-leader coverage for Legal operations. The µservice owns the canonical contract packet identity, an append-only clause/redline/obligation ledger, Cedar default-deny gating, ontology projection, audit-chain sealing, and signature-provider portability across the five contract-lifecycle aggregates (contract-intake, clause-library, negotiation, obligation, renewal).

Canonical competitive set (per `feedback_microservice_ownership_coherence_2026_05_20` + audit X-D3 resolution): **Ironclad, DocuSign CLM, Conga CLM**. Each is benchmarked feature-by-feature in `competitor-parity-matrix.md`.

The operational reason for a dedicated flat µservice (per ADR-0131 + ADR-0132): contract state, clause controls, obligations, approvals, renewal risk, and per-jurisdiction signature evidence require legal-domain ownership beyond what `drive`, `workflow-engine`, `ontology`, or `audit-chain` can supply individually. Per ADR-0245, this µservice is product (not substrate); it composes substrate dependencies but owns its own bounded contexts and signature evidence model.

Per ADR-0329 + `feedback_no_capability_profiles_2026_05_20`, the µservice does **not** stratify capability by retired named capability levels tiers. All deployed instances ship the full canonical capability surface. Tenant-class (`demo_trial` / `paid`), deployment context (`oyatie-public-cloud` / `aws-guest` / `oci-guest` / `on-prem` / `colo` / `oyatie-as-cloud-provider`), and jurisdiction pack (`gdpr` / `eidas` / `esign` / `kr-pipa` / `hipaa-baa` / `sox-404` / `sec-17a-4` / etc.) are the canonical axes per ADR-0330 + ADR-0331 + ADR-0251.

## B. Target Users

The µservice serves six canonical personas. Each persona is contract-lifecycle-domain-specific.

- **Marcus Chen, General Counsel of a 600-person B2B SaaS**. Manages 200+ active counterparty contracts. Needs tenant-scoped contract registry, AI-assisted redline review, obligation-extraction confidence bands, segregation-of-duties approval routing, SOX-404 audit evidence, integration with crm and cloud-billing.
- **Yejin Park, Solo Practitioner Lawyer** advising 30 client tenants from her side practice. Needs per-tenant contract isolation, demo_trial onboarding for new clients, KR-PIPA + KR 전자서명법 evidence for Korean contracts, on-prem deployment for sensitive matters.
- **Diana Alvarez, Contracts Manager at a Legal-Ops Agency** serving 8 enterprise tenants. Needs white-label deployment, per-tenant clause-library inheritance, counterparty MDM across tenants (with strict tenant isolation), bulk migration playbooks from Ironclad + DocuSign CLM + Conga CLM.
- **Nadia Singh, Compliance Officer at an EU bank**. Needs GDPR Article 7 consent records, eIDAS QES with Trust List TSA, HSM-resident signing keys (BYOK Thales Luna 7), sovereign-cell residency in Frankfurt + Paris, EU AI Act Annex III boundary declaration.
- **Omar Watkins, SRE accountable for legal-evidence integrity**. Needs WORM storage per SEC 17a-4(f), legal-hold state machine integration, e-discovery export (EDRM XML), HSM key rotation playbook, multi-region failover with RPO ≤ 60s.
- **Hana Mori, External Auditor (Big 4)** auditing a SOX-public-company tenant. Needs auditor-scope Cedar profile, contract approval segregation evidence, period-end SOX-404 export, immutable audit-chain seal verification, 7-year retention WORM.

## C. User Stories — Bespoke per persona × bounded context

Each story is concrete to the persona's domain. Acceptance criteria specify the actual legal-evidence + observability + policy expectations rather than templated checklists. The prior 25 stamped stories (5 personas × 5 contexts with identical acceptance criteria) are replaced.

### US-001 — Contract intake (Marcus Chen)
As Marcus Chen, GC of a 600-person B2B SaaS, I drag-drop a counterparty's MSA onto the contract-intake surface and the µservice (a) OOXML-diffs against my standard MSA template, (b) flags clauses deviating from my playbook per IP-026, (c) extracts obligations with IP-027 confidence bands, (d) proposes redlines via local Llama-3.1-70B with model-id pinning per `legal-dimensions/ai-redlining-prompt-template.md`, (e) seals the intake event into my SOX-404-compliant audit chain.
**Acceptance**: contract-intake.create accepts `.docx` ≤ 250 MiB; OOXML diff p95 ≤ 2s for ≤ 200KB documents; obligations extracted with confidence (≥ 0.85 auto-propose; < 0.85 advisory); audit event `oya.contract.lifecycle.management.contract_intake.created` emitted with tenant_id + tenant_class + jurisdiction_pack + policy_decision_id; Cedar gate verified.

### US-002 — Clause library tenant-scoped inheritance (Yejin Park)
As Yejin Park, solo lawyer with 30 client tenants, I need strict per-tenant clause library isolation with three-tier inheritance (tenant playbook → contract-type playbook → per-deal override), and Cedar-enforced no-cross-tenant-leakage even when I am logged into multiple tenants simultaneously.
**Acceptance**: clause-library.create enforces `tenant_id` from principal claim; tenant-scoped projection per ADR-0244 prevents cross-tenant query; three-tier resolution per `legal-dimensions/clause-library-inheritance.md`; SOX-404 segregation enforced (author ≠ approver).

### US-003 — Negotiation collaboration (Diana Alvarez)
As Diana Alvarez, contracts manager, I need real-time Loro CRDT collaborative editing per `legal-dimensions/redline-collaboration-crdt.md` with my 3 attorneys on a draft MSA, with every edit recorded in IP-029 redline provenance (author, source, timestamp) — and the contract state machine blocks edits the moment the contract enters OutForSignature.
**Acceptance**: negotiation.amend accepts CRDT operation batches; redline events emitted with author/source/timestamp/Cedar decision per IP-029; contract state machine blocks edits in OutForSignature / Signed / Effective states; audit-chain preserves all CRDT operations replayable by content hash.

### US-004 — Obligation tracking + force-majeure suspension (Nadia Singh)
As Nadia Singh, EU bank compliance officer, I need obligation tracking that respects GDPR Article 17 erasure exemptions, suspends obligations during force-majeure (per `legal-dimensions/force-majeure-obligation-suspension.md`), tracks notice-and-cure cure periods (per `legal-dimensions/notice-and-cure-obligation.md`), and emits cure-failed events to my workflow-engine when notice goes unanswered.
**Acceptance**: obligation.create + amend + replay; force-majeure state machine pauses obligation clock; notice-and-cure state machine drives cure-period reminders via calendar substrate; audit-chain preserves all obligation state transitions.

### US-005 — Renewal risk explainability (Omar Watkins)
As Omar Watkins, SRE, I need IP-028 renewal risk scoring with an explainability board so incident response can answer "why was this contract flagged" — the answer must trace to specific clauses, prior counterparty redline behavior, and obligation breach history, and the explanation must be reproducible from the audit chain.
**Acceptance**: renewal.risk-score returns explanation with feature attributions; replay deterministic from contract version hash + model id; renewal risk freshness p95 ≤ 15 min from upstream ledger append.

### US-006 — Audit export (Hana Mori)
As Hana Mori, external auditor, I need an auditor-scope Cedar profile granting read access to all SOX-relevant contracts for the audit period, with evidence bundle export covering (a) signature envelope verification, (b) approval evidence with segregation certification, (c) audit-chain seal verification (BLAKE3 root hash chain), (d) WORM lock attestation, (e) full obligation history.
**Acceptance**: auditor-scope Cedar policy grants tenant-scoped read; signature envelope verification per `legal-dimensions/signature-envelope-canonical.md`; SOX-404 segregation evidence per `legal-dimensions/approval-routing-matrix.md`; WORM evidence per `legal-dimensions/worm-binding-model.md`; full obligation history retained 7y.

### US-007 — Demo_trial onboarding (Yejin Park, new client)
As Yejin Park onboarding a new client to evaluate Oyatie CLM, the client tenant starts on `tenant_class=demo_trial` with caps: max 5 active contracts, max 100 KB document size, AES-only e-signature (no QES), no AI redlining, 30-day retention, TEST_DATA classification. Conversion to `tenant_class=paid` lifts all caps and reclassifies existing demo_trial contracts to LEGAL_PRODUCTION.
**Acceptance**: demo_trial Cedar gates enforce caps; conversion emits `oya.contract.lifecycle.management.tenant.class_converted.demo_trial_to_paid`; OCI Always Free deployment context supported per `feedback_oci_always_free_maximization`.

### US-008 — Counterparty MDM resolution (Marcus Chen)
As Marcus Chen, I track when "Acme Inc." in my contracts becomes "Acme LLC" via corporate reorg, when Acme merges with BetaCo, when BetaCo is acquired by GammaCorp — each name change preserved as a `LegalNameChange` event, active contracts surfaced for review.
**Acceptance**: per `counterparty-mdm/counterparty-mdm.md`, resolution via LEI + company registry + name fuzzy match; merger/dissolution state transitions emit audit events; sanctions screening at every counterparty update + monthly.

### US-009 — Multi-language contract execution (Diana Alvarez)
As Diana Alvarez, I have a French-Canadian client signing a contract whose governing law is Delaware. Per Bill 96 (Québec), the contract must have a French version available even though the governing law is Delaware. The µservice requires the French version and seals both languages into the signature envelope's composite Merkle hash.
**Acceptance**: per `legal-dimensions/multi-language-contract-overlay.md`, Cedar gate blocks signature seal absent French version when counterparty residency is QC; both language versions hash-bound in signature envelope; governing-language clause auto-injected.

### US-010 — Legal hold preservation (Omar Watkins)
As Omar Watkins, when served with a subpoena, I apply a legal hold per `state-machines/legal-hold-state-machine.md` covering all contracts with the named counterparty. All affected contracts move to PRESERVATION_OBLIGATION_ACTIVE; retention destruction is suspended; redline history is preserved; e-discovery export in EDRM XML 1.2 is produced on demand.
**Acceptance**: legal-hold state machine enforced; Cedar gate blocks delete/alter under hold; e-discovery export bundles contract body + all versions + redline + audit events + approval evidence; export hash-verified.

### US-011 — eIDAS QES with HSM custody (Nadia Singh)
As Nadia Singh, my EU bank tenant requires all material contracts signed with eIDAS QES using HSM-resident signing keys (Thales Luna 7 A790, FIPS 140-3 L3) in my Frankfurt cell. Trust List TSA per RFC 3161 from D-Trust Qualified TSA. PAdES-B-LTA applied at seal and renewed at TSA cert expiry -90 days.
**Acceptance**: per `packs/eidas/README.md` + `legal-dimensions/signature-envelope-canonical.md`, QES envelope with PAdES-B-LTA; HSM custody attested via QSCD; D-Trust TSA used; LTA renewal scheduled.

### US-012 — KR-PIPA sovereign deployment (Yejin Park, Korean client)
As Yejin Park onboarding a Korean enterprise client, the client requires sovereign-cell deployment in Seoul, KISA-rooted TSA, KISA-Certified Electronic Signature Service (Yessign), all data residency in Korean cells, and PIPA Article 28 explicit consent for any cross-border transfer.
**Acceptance**: per `packs/kr-pipa/README.md` + `jurisdictions/README.md`, KR-PIPA pack auto-activated; KR sovereign cell selected; KISA TSA used; cross-border Cedar gate enforced.

### US-013 — DocuSign CLM migration (Diana Alvarez)
As Diana Alvarez migrating a 50,000-contract Salesforce-native DocuSign CLM tenant to Oyatie CLM, I use `migration-playbooks/from-docusign-clm.md` with field-level mapping (`vendor-mapping/docusign-clm-field-mapping.md`), bulk export from DocuSign + Salesforce, hash-verified migration validation, and a 48-72-hour cutover window.
**Acceptance**: migration discovery enumerates Workflows / Contracts / Tags / Approvals; migration validates field completeness + signature evidence preservation; cutover preserves original signature evidence (no re-signing); audit chain records `migrated_from_docusign_clm_with_original_signature` annotation.

### US-014 — Approval routing with SOX-404 segregation (Marcus Chen)
As Marcus Chen, for any contract over $1M material value, approval routing per `legal-dimensions/approval-routing-matrix.md` requires Contracts Manager + Procurement + Legal Review + VP Procurement + VP Legal + General Counsel + CFO. Cedar blocks the author from approving their own contract. CFO approval is AES-signed. Full chain becomes audit evidence.
**Acceptance**: matrix N-of-M approval enforced; segregation gate (author ≠ approver) enforced; CFO approval AES-signed; SOX-404 7y retention applied.

### US-015 — Renewal notice cure period (Marcus Chen)
As Marcus Chen, my MSA has "termination notice required 90 days before expiration". The µservice auto-computes the renewal notice deadline per `legal-dimensions/obligation-due-basis-grammar.md` (`contract.expiration_date - 90 days`). At T-90 obligation enters Active; T-30 warning; T-7 escalation; T+0 if no notice issued, transition to Overdue with contract-renewal-by-default audit event.
**Acceptance**: due-basis grammar parses clause; obligation state machine drives reminders via calendar substrate; audit events emitted at each state transition.

### US-016 — FCPA anti-corruption certification (Omar Watkins)
As Omar Watkins overseeing a US-public-company tenant's anti-corruption program, every counterparty contract over $25k in a high-corruption-risk jurisdiction (CPI < 50) requires FCPA certification per `legal-dimensions/fcpa-ukba-detection.md`. The AI clause detector flags risky payment terms; Cedar rejects contract creation absent certification.
**Acceptance**: FCPA overlay activated; clause detector flags risky terms; Cedar gate blocks contract execution absent certification; counterparty sanctions screened against FCPA enforcement database.

### US-017 — Privilege tagging (Hana Mori)
As Hana Mori, when I request e-discovery export of a contract with attorney-client privilege annotations, the export pipeline applies privilege filter per `legal-dimensions/privilege-tagging-overlay.md` — privileged annotations replaced with privilege log entries; full audit trail preserved (I see WHAT was withheld and WHY, but not the content).
**Acceptance**: privilege classification per artefact; e-discovery export applies privilege filter; privilege log generated with redaction basis; FRE 502 clawback supported on inadvertent disclosure.

### US-018 — EU AI Act transparency (Nadia Singh)
As Nadia Singh, my EU bank uses CLM AI for clause suggestion. Per EU AI Act Article 50 (Limited Risk), every AI-suggested clause is marked as AI-generated with model provenance (`model_id`, `model_version`, `prompt_hash`, `inference_timestamp`). Signatory sees AI-suggested marker before sealing.
**Acceptance**: per `legal-dimensions/eu-ai-act-classification-for-clm-ai.md`, AI provenance bound to every suggestion; UI marker visible at sign time; Annex III boundary check applied to employment contracts.

### US-019 — HIPAA BAA flow-down (Marcus Chen for healthcare client)
As Marcus Chen advising a healthcare-tenant client, when the client (covered entity) sub-contracts a BA, the µservice ensures the sub-BA BAA inherits constraints from the upstream BAA per `packs/hipaa-baa/README.md`. The µservice refuses to seal a sub-BAA that weakens upstream constraints.
**Acceptance**: HIPAA pack activated; sub-BA flow-down detected; sub-BAA Cedar-gated against upstream BAA constraint weakening.

### US-020 — Marketplace DealSet binding (Diana Alvarez)
As Diana Alvarez running a marketplace where her clients license clause templates from each other, the marketplace DealSet (per ADR-0314) binds the clause-template purchase to a CLM contract. CLM cross-emits to marketplace on settlement; marketplace settles payment to seller.
**Acceptance**: dealset-contract-bind capability per IP-014; marketplace cross-emit on settlement; payment µservice handles disbursement.

### US-021 — Provider BYOK e-signature (Nadia Singh)
As Nadia Singh, my bank uses our corporate DocuSign account (not Oyatie's). Per `feedback_byok_everywhere_credentials` + ADR-0255, BYOK is the credential mode; the tenant provides DocuSign API credentials; the µservice authenticates as the tenant against DocuSign.
**Acceptance**: `provider_credential_modes.e_signature = "byok"` in manifest; credential sidecar binds DocuSign credentials from OpenBao; provider portability per IP-030.

### US-022 — On-prem deployment (Yejin Park, Korean client)
As Yejin Park, my Korean enterprise client deploys CLM on bare-metal infrastructure (on-prem deployment context). OpenTofu module `iac/on-prem/` provisions the µservice; customer brings their Thales Luna 7 HSM for QES; data never leaves customer-controlled infrastructure.
**Acceptance**: per `feedback_zero_handroll_opentofu_only`, on-prem OpenTofu module provisions everything; HSM BYOK = `byok_required_by_pack` for QES; no Oyatie outbound network dependency for runtime operations.

### US-023 — OCI Always Free demo_trial (Yejin Park)
As Yejin Park giving each new client a 30-day evaluation, the demo_trial tenant runs on OCI Always Free (`iac/oci-guest/always-free/`) at zero cost: 1 OCPU + 4 GB RAM + 50 GB block + 20 GB Autonomous DB + 25 GB egress. Demo_trial limits apply.
**Acceptance**: per `feedback_oci_always_free_maximization`, OCI Always Free module deploys within free-tier limits; demo_trial caps enforced.

### US-024 — Multi-region failover (Omar Watkins)
As Omar Watkins, when primary US-East-1 cell becomes unavailable, the µservice fails over to US-East-2 cell within RTO ≤ 1 hour, RPO ≤ 60 seconds. Cross-region replicas pre-positioned; HSM keys replicated to secondary HSM cluster.
**Acceptance**: per `multi-region.md`, cross-region failover within RTO/RPO; HSM cluster replication validated; failover drill quarterly per `runbooks/`.

### US-025 — Auto-renewal with renegotiation (Marcus Chen)
As Marcus Chen, my SaaS subscription contract has auto-renewal with 90-day renewal notice + 30-day renegotiation window. At T-90 obligation alerts me; T-90 to T-60 renegotiation window open and I author renewal amendment; T-60 renewal locks. Per IP-028, renewal risk score informs renegotiate vs auto-renew.
**Acceptance**: auto-renewal with renegotiation window; renewal risk score per IP-028; renewal amendment as child contract per contract-state-machine.

## D. Functional Requirements

Per ADR-CLM-001, canonical capability surface = 6 capability families × 5 bounded contexts. Functional requirements bespoke per (capability × context). Full enumeration in `contracts/openapi-v1.yaml`; headline requirements:

- **FR-001 contract-intake.create**: requires tenant_id, principal_id, tenant_class, deployment_context, jurisdiction_packs, contract_type from `taxonomies/contract-type-taxonomy.md`, idempotency key, trace context. Cedar default-deny per ADR-0243.
- **FR-002 contract-intake.amend**: same envelope + parent contract reference; redline events per IP-029.
- **FR-003 contract-intake.approve**: SOX-404 segregation (author ≠ approver); approval routing per `legal-dimensions/approval-routing-matrix.md`.
- **FR-004 clause-policy-evaluate**: IP-026 deviation classification; emits ClauseDeviation event with classification.
- **FR-005 obligation.create**: IP-027 confidence-band model; due-basis grammar per `legal-dimensions/obligation-due-basis-grammar.md`.
- **FR-006 obligation.acknowledge**: state machine transition per `state-machines/obligation-state-machine.md`.
- **FR-007 obligation.satisfy**: with performance evidence.
- **FR-008 renewal.risk-score**: IP-028; deterministic with model provenance.
- **FR-009 negotiation.redline-add**: IP-029; provenance preserved.
- **FR-010 signature.envelope-seal**: per `legal-dimensions/signature-envelope-canonical.md`; signature level per jurisdiction pack.
- **FR-011 signature.timestamp-request**: per `legal-dimensions/tsa-binding-model.md`; LOTL / KISA TSA per jurisdiction.
- **FR-012 dealset-contract-bind**: IP-014 marketplace integration.
- **FR-013 legal-hold.apply**: per `state-machines/legal-hold-state-machine.md`.
- **FR-014 legal-hold.release**: requires authority + audit event.
- **FR-015 e-discovery.export**: PRESERVATION_OBLIGATION_ACTIVE; EDRM XML 1.2.
- **FR-016 consent.record**: per `legal-dimensions/gdpr-article-7-consent-records.md`.
- **FR-017 consent.withdraw**: Article 7(3); 72-hour fulfillment.
- **FR-018 esign.consumer-disclose**: per `legal-dimensions/esign-consumer-disclosure-flow.md`.
- **FR-019 baa.execute**: per `packs/hipaa-baa/README.md`.
- **FR-020 force-majeure.declare**: per `legal-dimensions/force-majeure-obligation-suspension.md`.
- **FR-021 notice-and-cure.issue**: per `legal-dimensions/notice-and-cure-obligation.md`.
- **FR-022 counterparty.create**: per `counterparty-mdm/counterparty-mdm.md`; sanctions screening.
- **FR-023 counterparty.merger**: predecessor → successor chain.
- **FR-024 privilege.tag**: per `legal-dimensions/privilege-tagging-overlay.md`.
- **FR-025 privilege.waive**: with audit.
- **FR-026 audit.export.sox404**: per `packs/sox-404/README.md`.
- **FR-027 audit.export.gdpr-article-30**: record of processing activities.
- **FR-028 audit.export.eidas-cert-validation**: signer-cert chain + Trust List membership evidence.
- **FR-029 pack.activate**: tenant-scoped pack activation with Cedar policy compilation.
- **FR-030 pack.deactivate**: requires general-counsel authority + audit.

## E. Non-Functional Requirements

- **Maintainability**: each microservice owns one operational concern per ADR-0131 flat-layout doctrine; tenant-class behavior overlays differentiate tenant-facing affordance. Evidence: ADR-CLM-001 + this PRD + ARCHITECTURE.md + manifest.json + `packs/` + `legal-dimensions/`.
- **Observability**: per ADR-0263, every state transition emits a canonical audit event with mandatory dimensions: tenant_id, tenant_class, principal_id, deployment_context, home_cell, jurisdiction_pack, data_class, audit_event_class, trace_id, policy_decision_id.
- **Scalability**: tenant + cell + deployment_context + jurisdiction_pack + data_class + workload partition. Per ADR-0248 cellular topology, eligible at cell-tier-1, cell-tier-2.
- **Performance**: clause policy eval p95 ≤ 200 ms, p99 ≤ 500 ms; redline event append p95 ≤ 300 ms; obligation extraction completeness ≥ 0.98 vs canonical fixtures; renewal risk freshness p95 ≤ 15 min.
- **Optimization**: cost dimensions per `cost-budget.md` include tenant_id, tenant_class, billing_component (per_seat / per_usage / revenue_share), deployment_context, source_vendor, workflow_template, cell, data_class, jurisdiction_pack. Demo_trial tenants run on OCI Always Free with zero per-tenant cost.
- **Code quality**: Rust-strict per `feedback_rust_strict_only_no_python_2026_05_20`; OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 / BNF v4.1; ADR-0105 layer enum (9 layers); property + replay + authorization + contract tests required before promotion.
- **Availability**: 99.95% monthly for paid tenants (product-critical); 99.5% best-effort for demo_trial (no SLO contractual commitment).
- **Latency**: simple tenant-scoped command p95 ≤ 300 ms; signature delivery p95 ≤ 4s (AES), p95 ≤ 6s (QES with HSM round-trip).
- **Capacity**: partition by tenant + cell + deployment_context + status + data class + source-system id before any cross-tenant aggregation.

### DR posture (ADR-0343)

- Target: RTO <= 3600 s and RPO <= 300 s for contract state, obligation ledger, approval evidence, signature envelope metadata, and legal-hold records, matching `manifest.json#dr`.
- Compliance floors considered: HIPAA-2024 requires 3600 s / 300 s; SOX-404 base requires 14400 s / 3600 s; KR-PIPA general personal information requires 14400 s / 900 s. CLM does not own general-ledger journal entries or resident-registration-number storage in the manifest-backed posture; activating either overlay would tighten the effective target outside this PRD value.
- Failover runbook reference: `runbooks/signature-provider-outage.md`, `multi-region.md`, `iac/dr-failover.yaml`, and IP-022 chaos-drill evidence. The manifest substrate is `postgres_wal_g`, `object_storage_versioned`, and `audit_chain_merkle_seal`; HSM key replication and WORM/evidence immutability must be verified before promotion.
- Multi-region active-active posture: `true` in `manifest.json`; metadata and workflow read replicas can remain active across regions, while contract writes, signatures, obligation state, and legal-hold transitions still require idempotent evidence-preserving commit rules.
- WHY: legal teams retain access to contract evidence and renewal/obligation posture during regional failure without creating a second writable source of legal truth.

### Capacity model (ADR-0340)

- Manifest source: `manifest.json#capacity_model` declares the PRD capacity baseline.
- Per-tenant baseline: reserve 0.10 vCPU, 256 MiB RAM, 6 GB contract metadata/evidence working storage, 6 Postgres connections, 3 Valkey/cache connections, and 16 outbound HTTP slots for drive, KMS/HSM, mail, calendar, intelligence, marketplace, payments, and e-sign providers.
- Scaling dimension: `per_workflow_run`, because drafting, redline, approval, signature, obligation, renewal, and DealSet binding are long-lived workflow runs rather than high-frequency request loops.
- Cell placement class: Tier-3 product cell. Rationale: CLM carries legal evidence and signature integrations, but this manifest class keeps product workflow runtime outside Tier-1/Tier-2 substrate ownership.
- Autoscaling boundaries: contract-intake, clause-policy, obligation, approval, and renewal REST surfaces floor at 2 replicas and scale to 40; AI/OOXML/redline workers floor at 2 and scale to 30; signature and HSM paths use queue admission instead of unbounded replica growth.
- WHY: the model serves large document intake and extraction bursts while keeping signature, approval, legal-hold, and obligation state transitions deterministic.

### Sustainability + cost attribution (ADR-0344)

- Per-call emission claim: every contract intake, clause evaluation, obligation event, approval decision, signature packet, renewal-risk score, DealSet bind, and audit export row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours`.
- Provider routing affected by carbon: yes for OOXML diffing, AI redline suggestions, obligation extraction, renewal-risk scoring, and migration backfills when deadlines permit; no for signature envelope sealing, SOX approval gates, HIPAA BAA constraints, eIDAS/QES evidence, or legal-hold actions.
- Per-tenant cost transparency surface: CLM cost-budget and tenant billing expose cost by contract aggregate, document workload, signature provider, HSM/KMS path, cell, deployment_context, and jurisdiction pack.
- WHY: legal operations can explain expensive AI/document workloads and climate disclosures without delaying evidence-preserving legal actions.

### API versioning posture (ADR-0342)

- Public API version model: date carrier triplet using `Oyatie-Version: YYYY-MM-DD`, URL prefix `/v/<YYYY-MM-DD>/contract-lifecycle-management/...`, and proto3 field `oyatie_version`.
- SDK semver model: CLM SDKs use `major.minor.patch` for contract, clause, obligation, approval, signature, renewal, and audit clients.
- Support window: last N=3 public API dates are supported for at least 180 days.
- Per-tenant pinning supported: yes, for Ironclad, DocuSign CLM, Conga CLM, LinkSquares, Agiloft, and Icertis migrations and legal-review cutovers.
- Internal-mesh exemption: yes. ADR-0145 direct gRPC remains valid for drive, workflow-engine, KMS, marketplace, payments, mail, calendar, and intelligence calls.

## F. UX Flows

Per `state-machines/contract-state-machine.md`:

- **Contract intake flow**: discover source object (OOXML / PDF / text) → counterparty resolution via MDM → preview transform (OOXML diff vs standard template) → request Cedar permit → IP-027 obligation extraction → store as Draft.
- **Clause library flow**: author template → tenant-playbook integration → preview render with sample data → SOX-404 author ≠ approver gate at approval → activate version.
- **Negotiation flow**: Draft → CRDT-collaborative editing → SentForRedline → CounterpartyEdited (IP-029 provenance) → InternalReview → CounterRedline → ConvergedToFinal → ApprovalRouting → Signed.
- **Obligation flow**: Pending (confidence band gating) → Acknowledged → Active (calendar reminders) → Satisfied | Overdue → Cured | Disputed → Resolved.
- **Renewal flow**: T-180 risk score (IP-028) → T-90 alert → T-60 renegotiation window open → T-60 to T-30 renewal amendment authoring → T-0 auto-renewal or termination.

## G. Success Metrics

- **Coverage**: each of Ironclad / DocuSign CLM / Conga CLM has a migration playbook + field-level mapping + bulk migration tooling.
- **Authorization**: 100% of mutations pass through Cedar default-deny.
- **Observability**: 100% of state transitions emit canonical audit events per ADR-0263.
- **Migration**: dry-run rejection reports include source id, transform id, reason, owner, retry plan.
- **Cost**: every async job emits the dimensions in NFR Optimization.
- **Legal-evidence integrity**: every signature envelope verifies; every audit-chain seal verifies; every WORM-locked object remains non-rewriteable; every legal hold blocks deletion.
- **Hyperscaler-grade legal-domain depth**: 20 legal-compliance dimension docs authored per `legal-dimensions/` + `packs/`; intern-buildable substance bar per ADR-0328 + ADR-CLM-001.

## H. Compliance Impact

Compliance packs per `manifest.json` + `packs/README.md`:

- **soc-2** + **iso-27001**: organisational + technical security controls baseline.
- **gdpr**: Article 7 consent + Article 17 erasure + Article 28 DPA + Article 32 security + Chapter V transfer + Article 35 DPIA.
- **sox-404**: § 404 ICFR + § 802 7-year retention + segregation of duties.
- **eidas**: AES + QES envelope; LOTL Trust List ingestion; HSM custody; PAdES-B-LTA archive.
- **esign**: § 7001 + § 7001(c) consumer disclosure flow.
- **kr-pipa**: Article 32 consent + Article 28 cross-border + KISA TSA.
- **hipaa-baa**: BAA contract type + § 164.308(b)(3) written-BAA evidence + sub-BA flow-down.
- **sec-17a-4**: WORM storage + audit-trail-system option + DTP credentials.

Each pack file declares: active triggers, enforced behaviour, retention overlay, residency overlay, Cedar gate fragment, evidence-on-activation, standards references. Higher-restriction-wins on composition.

## I. Open Questions

Per audit § 5:

- **Q-001**: CPQ-CLM bridge ownership (crm vs CLM vs new cpq µservice). Wave 14.
- **Q-002**: E-signature provider boundary (CLM-internal vs separate µservice). Wave 14.
- **Q-003**: Contract repository ownership (CLM vs drive vs cloud-storage). Wave 14.
- **Q-004**: Clause taxonomy ownership (CLM vs governance vs separate clause-library-as-µservice). Wave 14.
- **Q-005**: Legal hold ownership (CLM vs governance vs ediscovery µservice). Wave 14.
- **Q-006**: TSA integration ownership (CLM vs kms). Wave 14.
- **Q-007**: HSM-resident QES key ownership (CLM vs kms). Wave 14.
- **Q-008**: Counterparty MDM ownership (CLM vs crm vs separate MDM). Wave 14.
- **Q-009**: Marketplace DealSet contract binding — in-progress per IP-014.
- **Q-010**: OCI Always Free profile decomposition for CLM demo_trial.

## J. Out of Scope

- Recreating an Ironclad / DocuSign CLM / Conga CLM suite boundary (ADR-0132 no-suite-policy).
- Sharing database tables with adjacent µservices (ADR-0145 direct gRPC).
- Treating vendor labels as canonical object names (ADR-0329).
- Bypassing marketplace DealSet settlement for commercial obligations (ADR-0314).
- Building a separate e-signature provider µservice within CLM (Wave 14 decision).
- Building a separate CPQ µservice within CLM (Wave 14 decision).

## K. Hyperscaler and Industry Precedents

The µservice draws explicit lessons from the canonical top-3 counterparts:

- **Ironclad** (B2B-legal high-velocity SaaS): Workflow data model with structured fields, Approval Workflow with parallel + sequential routing, Jurist AI clause analysis, Repository Search. Lesson: schema-driven approach scales. Oyatie adopts schema-driven contract types per `taxonomies/contract-type-taxonomy.md`.
- **DocuSign CLM** (formerly SpringCM): native eSignature integration, DocuSign Insight AI for search + analytics, multi-folder hierarchy, deep Salesforce integration. Lesson: e-signature provider portability is critical. Oyatie adopts provider-portable signature envelope per IP-030.
- **Conga CLM** (formerly Apttus, Salesforce-native CLM): Agreement / Clause / Schedule / Order Form data model with strong CPQ-CLM bridge, Salesforce-native deployment. Lesson: CRM integration critical for revenue contracts. Oyatie adopts crm ↔ CLM cross-emit via ontology projection per ADR-CLM-001 §C8.

## L. Pack Overlay Applicability

Per `packs/README.md`, canonical overlay roster: soc-2, iso-27001, gdpr, sox-404, eidas, esign, kr-pipa, hipaa-baa, sec-17a-4. Tenants opt into additional jurisdiction packs per `jurisdictions/README.md`.

Each pack declares active triggers, enforced behaviour, retention overlay, residency overlay, Cedar gate fragment, composition with other packs (higher-restriction-wins), and evidence on activation.

## M. Per-Aggregate Trace (5 bespoke traces; the prior 217 stamped trace rows are deleted per Wave 15A remediation)

### Aggregate: contract-intake
Tenant-scoped per ADR-0244; Cedar-gated per ADR-0243; ontology-projected to crm.opportunity + ontology.contract; workflow-orchestrated by workflow-engine for approval routing; audit-chain sealed with HSM-rooted key; pack-aware (gdpr / esign / eidas auto-apply per counterparty jurisdiction); reversible via the contract state machine's Draft state. Key invariants: source-vendor provenance immutable; destructive correction forbidden; counterparty resolved via MDM; jurisdiction pack frozen at contract execution.

### Aggregate: clause-library
Tenant-scoped per ADR-0244; Cedar-gated; ontology-projected as clause-template index; SOX-404 segregation enforced at template activation; audit-chain sealed; pack-aware (hipaa-baa BAA template prohibited modifications, gdpr DPA Article 28 flow-down). Key invariants: three-tier inheritance (tenant-playbook → contract-type playbook → per-deal override); template version Merkle-rooted; fallback positions enumerated.

### Aggregate: negotiation
Tenant-scoped; Cedar-gated; ontology-projected as redline events; workflow-orchestrated via redline turnaround state machine; audit-chain sealed (every CRDT operation + redline event); pack-aware (counterparty privilege filter per `legal-dimensions/privilege-tagging-overlay.md`). Key invariants: edit blocked once OutForSignature; redline provenance preserved (IP-029); collaborative editing via Loro CRDT pre-signature only.

### Aggregate: obligation
Tenant-scoped; Cedar-gated; ontology-projected as obligation register; workflow-orchestrated via obligation state machine; audit-chain sealed; pack-aware (force-majeure suspends, notice-and-cure adjusts due dates). Key invariants: IP-027 confidence bands gate auto-propose; due-basis grammar deterministic; force-majeure suspension pauses but does not extend.

### Aggregate: renewal
Tenant-scoped; Cedar-gated; ontology-projected as renewal risk index; workflow-orchestrated via renewal cadence and renegotiation window; audit-chain sealed; pack-aware (auto-renewal blocked for ESIGN consumer contracts without disclosure). Key invariants: IP-028 explainability board produces feature attributions; renewal risk score deterministic from contract version hash + model id; auto-renewal cure period blocks default-renewal.

## N. Companion docs (canonical surface)

- `ARCHITECTURE.md` — 13-anchor architecture per ADR-0321.
- `compliance.md` — full compliance evidence per pack.
- `manifest.json` — machine-readable spec.
- `decisions/ADR-CLM-001-clause-obligation-ledger-and-redline-provenance.md` — per-µservice ADR.
- `packs/<pack>/README.md` — 9 pack overlay files.
- `legal-dimensions/<dimension>.md` — 16 legal-compliance dimension files.
- `state-machines/<machine>.md` — 4 state machines (contract, obligation, redline, legal-hold).
- `taxonomies/<taxonomy>.md` — 2 taxonomies (clause family, contract type).
- `jurisdictions/` — per-jurisdiction overlays.
- `vendor-mapping/` — Ironclad / DocuSign CLM / Conga CLM field mappings.
- `migration-playbooks/` — Ironclad / DocuSign CLM / Conga CLM migration playbooks.
- `counterparty-mdm/` — Counterparty MDM model.
- `IP-001..IP-030` — 30 implementation plans.
- `REMEDIATION-NOTES-2026-05-21.md` — Wave 15A remediation log.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
