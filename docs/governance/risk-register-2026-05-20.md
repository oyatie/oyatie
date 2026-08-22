---
purpose: "Oyatie canonical risk register and threat-landscape catalogue snapshot"
doc_status: published
doc_class: Governance-Risk-Register
owner_team: council-architecture
snapshot_date: 2026-05-20
review_cadence: weekly for severe/extreme, monthly for elevated, quarterly for watch
---

# Oyatie Risk Register and Threat-Landscape Catalogue - 2026-05-20

## Section 1. Risk Methodology

This register is the dated governance snapshot for strategic, operational, compliance, technical, commercial, reputational, and social risks known on 2026-05-20.
It is scoped to Oyatie's agentic-primary, tenant-scoped, Cedar-gated, audit-chain-backed, cell-oriented platform architecture.
It complements `docs/RISK-REGISTER.md`, `docs/SECURITY-PROGRAM.md`, `docs/PRIVACY-PROGRAM.md`, `docs/COMPLIANCE-MATRIX.md`, `specs/masterplan.json`, and `registry/microservices.json`.
It does not amend any ADR.
It does not replace machine-readable risk registers when PHASE-5/PHASE-6 retirement promotes this content into JSON.
It records threats in enough detail for governance review, runbook routing, monitoring, and control-evidence follow-up.
It treats legal and regulatory material as risk intelligence, not legal advice.
It uses official regulator source snapshots where current enforcement posture matters.

### Likelihood Scale - OYA-L5

1. Rare - credible only under exceptional conditions or targeted nation-state pressure.
2. Unlikely - plausible within a year, but current controls and threat surface keep probability low.
3. Possible - reasonable scenario during normal growth, active development, or regulator attention.
4. Likely - expected without active mitigation, monitoring, and owner accountability.
5. Almost Certain - already recurring, already materialized elsewhere, or structurally probable.

### Impact Scale - OYA-I5

1. Limited - localized inconvenience, no regulated data, no customer trust impact.
2. Moderate - single-team disruption, limited tenant effect, recoverable without external notice.
3. Major - multi-service incident, material customer friction, contractual remediation likely.
4. Severe - regulated breach, major revenue impact, serious customer loss, or regulator engagement.
5. Critical - existential trust loss, business-line prohibition, broad tenant harm, or insolvency path.

### Score Bands

Score = likelihood x impact.
1-4 is Watch.
5-9 is Managed.
10-14 is Elevated.
15-19 is Severe.
20-25 is Extreme.
Velocity is recorded separately because a low-likelihood event can still demand weekly review when warning time is short.
Extreme and Severe risks are reviewed weekly by the named owner and monthly by `council-architecture`.
Elevated risks are reviewed monthly by the owner role.
Managed and Watch risks are reviewed quarterly unless indicators trip.

### Named Risk Owner Roles

`council-architecture` owns architecture doctrine, ADR alignment, cross-risk prioritization, and acceptance decisions.
`council-privacy` owns data-use boundary, DSR, consent, transparency, transfer, and privacy breach risks.
`ops-security` owns secrets, key management, supply chain, vulnerability, incident response, and red-team controls.
`ops-sre-reliability` owns SLOs, incident runbooks, cell recovery, backup restore, capacity, and resilience drills.
`ops-compliance` owns regulator-watch, jurisdiction packs, evidence exports, audit responses, and compliance mappings.
`ops-finops` owns capital efficiency, cloud cost, quota exhaustion, price pressure, and tenant margin risk.
`axis-foundry` owns agentic development pipeline, capability governance, model/eval drift, guardrails, and evidence packs.
`axis-cloud` owns cloud-cell, KMS/HSM, network, compute, storage, Kubernetes, and hyperscaler invariants.
`axis-messenger`, `axis-community`, and `axis-mail` jointly own messenger, community, mail, MLS, DLP, moderation, and cross-tenant communication controls.
`axis-workspace` owns documents, drive, calendar, task, recording, and collaborative editing surfaces.
`vertical-healthcare` owns HIPAA/PHI feature paths and healthcare compliance runbooks.
`regional-packs` owns jurisdiction overlays, sovereign residency, regulator publication feeds, and pack deltas.
`gtm-sales-se` owns GTM motion, customer migration, proof-of-value, pricing, churn, and deal concentration.
`gtm-partnerships` owns vendor/partner ledgers, insurance transfer, processor terms, and ecosystem claims.
`comms-trust` owns public trust, incident narrative, trust portal, content disputes, and social-media escalation.

### Named Review Cadence

Weekly Risk Council: every Monday before roadmap sequencing; covers Extreme, Severe, active incidents, and new regulator alerts.
Monthly Compliance Review: first Thursday of each month; covers GDPR, PIPA, HIPAA, CCPA/CPRA, EU AI Act, DORA, export-control, and sovereign overlays.
Quarterly Board Risk Review: quarter close; ratifies acceptance, transfer, insurance posture, capital reserve, and owner changes.
Release Gate Review: before any wave gate, stable promotion, regional-pack launch, high-risk capability promotion, or major customer go-live.
Incident Close Review: within five business days of Sev-1 or Sev-2 closure; updates this register when a risk materializes or controls fail.

### Current Regulator-Source Snapshot Used

EU AI Act Service Desk timeline: prohibitions and AI literacy applied on 2025-02-02, GPAI governance on 2025-08-02, and most high-risk/transparency enforcement begins 2026-08-02.
European Commission DORA transposition page: DORA transposition deadline was 2025-01-17 and infringement proceedings were pending against multiple member states in the 2026 snapshot.
EDPB CEF 2026: coordinated enforcement focus shifted to GDPR transparency and information obligations across participating DPAs.
EDPB Meta transfer decision: the 2023 Meta/Facebook transfer case remains the named signal for Chapter V cross-border transfer risk.
Korea PIPC actions: Google/Meta behavioral-ad sanctions, OpenAI breach notification recommendations, Temu cross-border transfer sanctions, and KAB/TELUS AI SQLi sanctions are the named PIPA enforcement signals.
HHS OCR 2026 MMG Fusion settlement: business associate breach and risk-analysis enforcement remains the named HIPAA enforcement signal.
California OAG/CPPA enforcement actions: Sephora, Healthline, Disney, Jam City, and General Motors actions are named CCPA/CPRA signals for opt-out, sharing, sensitive data, and minimization risk.

## Section 2. Strategic Risks

### STR-001 - Regulator-imposed prohibition on workforce or credit AI functionality
- risk-ID: STR-001
- name: Regulator-imposed prohibition on workforce or credit AI functionality
- category: Strategic
- description: Oyatie may build agentic or decision-support functions that a regulator later classifies as prohibited, high-risk, or unlawfully manipulative in employment, credit, education, health, or public-service contexts.
- threat landscape: EU AI Act prohibited-practice guidance, AI Act Annex III high-risk enforcement from 2026-08-02, Korea PIPC AI scrutiny, and US sector regulators create a moving boundary around automated recommendations.
- affected microservices: foundry, workflow-engine, tasks, analytics, vertical-healthcare, vertical-fintech, messenger, community, mail, application.
- likelihood: Likely (4/5)
- impact: Critical (5/5)
- score: 20 Extreme
- velocity: Fast when enforcement guidance lands or a large customer requests a sensitive workflow.
- owner: ops-compliance
- owner role: Chief Compliance Officer delegate with council-privacy co-owner.
- review cadence: Weekly until EU AI Act high-risk obligations are fully mapped; monthly afterward.
- status: Open - mitigation in progress.
- acceptance posture: Avoid for prohibited practices; reduce for high-risk deployer/provider obligations.
- transfer posture: E&O insurance only covers defense cost; product prohibition is not transferable.
- microservice mitigations: `intelligence-guardrails-autonomy-tier-gate-kernel`, `check-high-risk-auto-decision-refusal`, `regional-pack-api`, and `intelligence-evidence-regulator-export-framework-profiles`.
- Cedar policies: `policy.ai_high_risk_deployer_gate`, `policy.prohibited_practice_refusal`, `policy.human_oversight_required`, `policy.regional_pack_ai_act_overlay`.
- monitoring: `microservices/intelligence/dashboards/runtime-autonomy-tier-mix.json`, `microservices/intelligence/dashboards/guardrails-jailbreak-attempt-rate.json`, `registry/dashboards/compliance-pack-attestation-lag.yaml`.
- named indicators: high-risk workflow count, T3/T4 autonomy attempt rate, AI Act pack attestation age, denied prohibited-practice requests, regulator publication delta.
- early-warning trigger: new regulator guidance names a category already present in a customer journey or capability registry row.
- control evidence: AI risk-class registry entry, evidence pack export, human-oversight attestation, and capability eval result.
- runbook reference: `docs/runbooks/compliance-pack-emergency-suspension.md`.
- incident class: compliance-product-prohibition.
- customer communication: trust portal statement plus per-tenant capability suspension notice.
- regulator action reference: EU AI Act timeline and AI Act prohibited-practices guidance snapshot.
- ADR reference: ADR-0144, ADR-0308, ADR-0309, ADR-0022, ADR-0129.
- residual risk: Medium after per-capability classification, deployer duties, and region overlays are enforced.
- checkpoint: Require `EU-AI-ACT-PACK-CURRENT` before any high-risk launch.
- escalation: council-architecture plus outside counsel when a capability enters Annex III territory.

### STR-002 - Hyperscaler dependency and maturity-claim mismatch
- risk-ID: STR-002
- name: Hyperscaler dependency and maturity-claim mismatch
- category: Strategic
- description: Oyatie's ambition to match hyperscaler operational maturity may outrun actual cloud-cell, autoscaling, observability, control-plane, and failover evidence.
- threat landscape: Large customers compare Oyatie against AWS, Azure, GCP, OCI, and sovereign-cloud alternatives; unsupported maturity claims create sales, legal, and reputational exposure.
- affected microservices: cloud-cell, cloud-compute, cloud-iam, cloud-network, cloud-storage, observability, ops, foundry.
- likelihood: Possible (3/5)
- impact: Critical (5/5)
- score: 15 Severe
- velocity: Medium; risk rises at enterprise proof-of-concept, RFP, and trust-portal publication moments.
- owner: council-architecture
- owner role: Architecture Council risk owner with axis-cloud accountable for evidence.
- review cadence: Weekly while hyperscaler gates remain active; monthly after gate closure.
- status: Open - gated by honest-claims evidence.
- acceptance posture: Reduce; do not accept unsupported maturity claims.
- transfer posture: Contract disclaimers reduce damages but cannot transfer trust loss.
- microservice mitigations: `cloud-cell-app`, `shared-hyperscaler-metrics-kernel`, `platform-cell-kernel`, `check-hyperscaler-maturity-claims`.
- Cedar policies: `policy.cloud_mutation_cell_scope`, `policy.hyperscaler_claim_publish_gate`, `policy.ops_console_internal_private`.
- monitoring: `specs/hyperscaler-gates.json`, `registry/dashboards/cell-routing-and-shuffle-sharding.yaml`, `registry/dashboards/golden-signals-per-microservice.yaml`.
- named indicators: HG gate pass count, failover drill age, autoscaler error rate, public claim queue, trust-portal maturity assertions.
- early-warning trigger: sales collateral or docs claim provider-grade resilience before gate evidence exists.
- control evidence: chaos drill report, p99 cloud-cell SLO, OTel coverage, runbook drill completion, and gate artifact.
- runbook reference: `docs/runbooks/ops/dr-drill-runbook.md`.
- incident class: strategic-claim-governance.
- customer communication: qualify beta/preview wording and provide evidence-backed capability scope.
- regulator action reference: DORA ICT resilience expectations for financial-sector customers.
- ADR reference: ADR-0123, ADR-0128, ADR-0134, ADR-0198, ADR-0202, ADR-0248.
- residual risk: Medium because hyperscaler parity is a moving target, not a one-time certification.
- checkpoint: No hyperscaler-equivalent public claim without `governance-honest-claims` evidence.
- escalation: Architecture Council blocks publication when claim evidence is stale.

### STR-003 - Key personnel attrition in governance, security, and kernel domains
- risk-ID: STR-003
- name: Key personnel attrition in governance, security, and kernel domains
- category: Strategic
- description: Loss of a small number of domain owners could stall doctrine, Cedar policy, audit-chain, KMS, tenant kernel, or regulatory-pack execution.
- threat landscape: The project has unusually dense governance and architecture context; replacement cost is high because ownership spans docs, specs, code, runbooks, and regulator reasoning.
- affected microservices: foundry, policy, audit-chain, tenancy, identity, secrets, data-boundary, regional-pack.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; notice periods may be short and undocumented knowledge can vanish immediately.
- owner: council-architecture
- owner role: Founder or Chief Architect delegate.
- review cadence: Monthly until RACI, CODEOWNERS, and on-call backups are complete.
- status: Open - reducing through owner assignment and runbook coverage.
- acceptance posture: Accept residual specialist scarcity; reduce single-person dependency.
- transfer posture: Recruiting partners and contractor bench transfer schedule pressure only.
- microservice mitigations: `intelligence-evidence-evidence-pack-builder-usecase`, `check-raci-completeness`, `check-codeowners-mirror`, `intelligence-supervisor-agent-fleet-lifecycle-domain`.
- Cedar policies: `policy.owner_handoff_required`, `policy.break_glass_dual_control`, `policy.capability_owner_backup_required`.
- monitoring: RACI freshness, CODEOWNERS mirror, runbook orphan check, capability owner backup count.
- named indicators: ownerless capability rows, stale runbooks, no backup reviewer, missed risk review, unresolved ADR owner.
- early-warning trigger: an owner leaves, changes role, or cannot approve release-gate evidence.
- control evidence: RACI row, CODEOWNERS mirror, runbook handover, second reviewer sign-off.
- runbook reference: `docs/runbooks/on-call-handover.md`.
- incident class: governance-capacity-loss.
- customer communication: no external notice unless delivery commitment or support SLA changes.
- regulator action reference: not regulator-driven; indirectly affects audit response readiness.
- ADR reference: ADR-0019, ADR-0110, ADR-0212, ADR-0305.
- residual risk: Medium while architecture remains broad and hiring is incomplete.
- checkpoint: Each Extreme/Severe risk must have primary and backup owners.
- escalation: Board risk review if any critical owner lacks backup for two reviews.

### STR-004 - Capital-market downturn and runway compression
- risk-ID: STR-004
- name: Capital-market downturn and runway compression
- category: Strategic
- description: Weak capital markets or delayed enterprise conversion can compress runway before foundational platform, compliance, and trust controls reach sellable maturity.
- threat landscape: Tenant RBAC infrastructure buyers elongate procurement during downturns while investors demand faster revenue and lower cloud burn.
- affected microservices: foundry, cloud-billing, finops, ops, application, marketplace, vertical-corporate.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; cash pressure compounds over months but can abruptly alter roadmap choices.
- owner: ops-finops
- owner role: Finance lead or interim founder finance owner.
- review cadence: Monthly, with weekly review when runway drops below the board threshold.
- status: Open - accepted residual macro risk.
- acceptance posture: Accept macro exposure; reduce burn and sequence revenue proof.
- transfer posture: Non-dilutive funding, cloud credits, customer prepay, and insurance do not transfer product-market risk.
- microservice mitigations: `cloud-billing-tax-app`, `cloud-finops-api`, `metering-domain`, `intelligence-providers-router-domain`.
- Cedar policies: `policy.cost_budget_enforced`, `policy.provider_cost_ceiling`, `policy.tenant_margin_guard`, `policy.discount_approval_dual_control`.
- monitoring: `microservices/intelligence/dashboards/providers-provider-cost-per-tenant.json`, `registry/dashboards/capability-tier-sla-conformance.yaml`, FinOps monthly close.
- named indicators: runway months, CAC payback, cloud cost per active tenant, provider quota spend, discount depth, deferred compliance work.
- early-warning trigger: sales pressure proposes bypassing foundation gates to close a deal.
- control evidence: cost attribution report, budget exception ledger, priced capability tier, board risk minutes.
- runbook reference: `docs/runbooks/finops-monthly-close.md`.
- incident class: strategic-financial-pressure.
- customer communication: avoid unsupported roadmap commitments in procurement answers.
- regulator action reference: none; macro downturn can reduce compliance staffing and increase residual risk.
- ADR reference: ADR-0174, ADR-0199, ADR-0217, ADR-0254.
- residual risk: Medium because market timing is not controllable.
- checkpoint: No foundation-bypass accepted solely to preserve short-term revenue.
- escalation: Board risk review when runway, margin, or burn indicators breach thresholds.

### STR-005 - AI moat erosion and model commoditization
- risk-ID: STR-005
- name: AI moat erosion and model commoditization
- category: Strategic
- description: Foundation model capabilities may commoditize faster than Oyatie's workflow, policy, evidence, and tenant-governance moat matures.
- threat landscape: Model vendors, OSS models, incumbent suites, and customer-built agents can imitate surface features while undercutting price.
- affected microservices: foundry, workflow-engine, workflow-studio, ontology, capability-registry, evidence, guardrails.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Fast; model capability jumps can land without warning.
- owner: axis-foundry
- owner role: Foundry product and engineering owner.
- review cadence: Monthly, escalating after frontier model releases or major competitor launches.
- status: Open - reduce through durable workflow/evidence differentiation.
- acceptance posture: Accept model commoditization; reduce dependence on model novelty.
- transfer posture: Not transferable; partnership and provider abstraction reduce provider-specific exposure.
- microservice mitigations: `intelligence-eval-eval-runner-domain`, `intelligence-evidence-evidence-pack-builder-domain`, `intelligence-guardrails-output-validator-kernel`, `intelligence-providers-router-domain`.
- Cedar policies: `policy.model_provider_abstraction_required`, `policy.eval_gate_before_model_cutover`, `policy.evidence_pack_required`.
- monitoring: `microservices/intelligence/dashboards/eval-parity-trend.json`, `microservices/intelligence/dashboards/providers-provider-error-rate.json`, `microservices/intelligence/dashboards/evidence-pack-assembly-rate.json`.
- named indicators: eval parity delta, per-capability pass rate, model vendor feature overlap, churn due to "good enough" customer agents.
- early-warning trigger: a customer replaces an Oyatie proof-of-concept with generic agents plus spreadsheets.
- control evidence: capability-specific eval set, evidence export, audited run ledger, tenant-specific Cedar decision logs.
- runbook reference: `docs/runbooks/foundry-model-cutover.md`.
- incident class: strategic-competitive-differentiation.
- customer communication: emphasize governed execution, audit chain, residency, and policy controls rather than raw model novelty.
- regulator action reference: EDPB AI model opinion and EU AI Act GPAI obligations inform model-governance positioning.
- ADR reference: ADR-0026, ADR-0255, ADR-0316, ADR-0021.
- residual risk: Medium; durable value must come from governed workflows and evidence.
- checkpoint: Every AI capability must declare non-model moat evidence.
- escalation: Product council if a capability is model-wrapper-only.

### STR-006 - Jurisdictional fragmentation blocks global product coherence
- risk-ID: STR-006
- name: Jurisdictional fragmentation blocks global product coherence
- category: Strategic
- description: Divergent privacy, AI, sector, residency, security, and export rules may force region-specific behavior that conflicts with the canonical-base plus localization-pack doctrine.
- threat landscape: EU, Korea, US states, India, Brazil, Middle East, Australia, and sector regulators increasingly impose conflicting data, AI, cloud, and audit requirements.
- affected microservices: regional-pack, residency, policy, data-boundary, tenancy, analytics, search, messenger, community, mail, foundry.
- likelihood: Likely (4/5)
- impact: Severe (4/5)
- score: 16 Severe
- velocity: Medium; conflicts emerge as regional packs and enterprise deals expand.
- owner: regional-packs
- owner role: Regional Compliance Pack lead with council-privacy co-owner.
- review cadence: Monthly regulator-pack review; weekly during new-region launch.
- status: Open - actively mitigated through pack overlays.
- acceptance posture: Reduce; accept delayed region launch when conflict is unresolved.
- transfer posture: Local counsel transfers interpretation support, not compliance accountability.
- microservice mitigations: `regional-pack-api`, `platform-residency-kernel`, `policy-cedar-domain`, `dsr-domain`.
- Cedar policies: `policy.region_pack_overlay_required`, `policy.residency_deny_by_default`, `policy.cross_jurisdiction_conflict_hold`.
- monitoring: `registry/dashboards/compliance-pack-attestation-lag.yaml`, regulator publication feed health, per-pack stale status.
- named indicators: conflict count, stale pack age, blocked tenant-region pair, counsel memo lag, residency exception count.
- early-warning trigger: two active regional packs require incompatible retention, transfer, or audit evidence on the same tenant flow.
- control evidence: conflict-resolution record, pack overlay diff, legal memo pointer, audit-chain event.
- runbook reference: `docs/runbooks/regulatory-change-response.md`.
- incident class: jurisdiction-conflict.
- customer communication: region availability matrix with explicit unsupported combinations.
- regulator action reference: PIPC cross-border transfer guidance and EU GDPR transfer enforcement.
- ADR reference: ADR-0010, ADR-0049, ADR-0240, ADR-0304, ADR-0064.
- residual risk: Medium because law and customer topology will keep diverging.
- checkpoint: No regional launch without conflict-resolution row.
- escalation: council-architecture and outside counsel when packs conflict.

### STR-007 - Platform trust backlash from agentic autonomy
- risk-ID: STR-007
- name: Platform trust backlash from agentic autonomy
- category: Strategic
- description: Customers, regulators, workers, or the public may reject Oyatie if autonomous agents appear to act without accountable human authority, explainability, or reversal.
- threat landscape: Agentic AI incidents, workplace surveillance concerns, prompt injection, and rogue automation narratives can turn a product strength into a trust liability.
- affected microservices: foundry, guardrails, workflow-engine, messenger, community, mail, tasks, audit-chain, policy, application.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast after a visible incident or viral customer complaint.
- owner: axis-foundry
- owner role: Foundry Trust lead with comms-trust co-owner.
- review cadence: Monthly, weekly after any autonomy incident or public-risk launch.
- status: Open - mitigated through autonomy ceilings and evidence-first controls.
- acceptance posture: Reduce; do not accept opaque T3/T4 autonomy in regulated flows.
- transfer posture: E&O and cyber coverage help response cost but not trust recovery.
- microservice mitigations: `intelligence-supervisor-autonomy-policy-enforcement-kernel`, `intelligence-supervisor-kill-switch-circuit-breaker-domain`, `audit-chain-domain`.
- Cedar policies: `policy.autonomy_ceiling_runtime_enforced`, `policy.human_approval_required_for_t4`, `policy.kill_switch_authorized_operator`.
- monitoring: `microservices/intelligence/dashboards/supervisor-autonomy-violation-rate.json`, `microservices/intelligence/dashboards/supervisor-kill-switch-coverage.json`.
- named indicators: autonomy violation attempts, kill-switch coverage gaps, customer override rate, human approval latency, public complaint volume.
- early-warning trigger: a tenant asks for unsupervised agent authority beyond declared capability tier.
- control evidence: autonomy decision log, Cedar evaluation trace, kill-switch drill, customer approval receipt.
- runbook reference: `docs/runbooks/foundry/autonomy-ceiling-breach-attempt.md`.
- incident class: autonomy-trust.
- customer communication: publish capability tier, approval chain, and rollback path.
- regulator action reference: EU AI Act GPAI/systemic-risk and high-risk obligations inform control wording.
- ADR reference: ADR-0022, ADR-0305, ADR-0293, ADR-0139.
- residual risk: Medium; trust depends on repeated evidence and customer education.
- checkpoint: T3/T4 capability cannot promote without supervised rollback drill.
- escalation: immediate SEV-1 bridge if agent action harms a tenant or end user.

## Section 3. Operational Risks

### OPS-001 - Per-cell cascade failure
- risk-ID: OPS-001
- name: Per-cell cascade failure
- category: Operational
- description: A failure in one cell can spread through shared control-plane, broker, identity, KMS, or observability dependencies and degrade multiple tenants or regions.
- threat landscape: Cell architecture reduces blast radius only when shared planes are backpressure-aware, rate-limited, and fail-closed per tenant and region.
- affected microservices: cloud-cell, cell-domain, cloud-network, cloud-compute, observability, eventing, tenancy, ops.
- likelihood: Possible (3/5)
- impact: Critical (5/5)
- score: 15 Severe
- velocity: Fast during overload, bad deploy, broker failure, or control-plane mutation storm.
- owner: ops-sre-reliability
- owner role: SRE cell-resilience owner with axis-cloud accountable.
- review cadence: Weekly until cell game days pass; monthly after stable drill evidence.
- status: Open - mitigated by shuffle sharding and evacuation runbooks.
- acceptance posture: Reduce; accept only documented residual single-cell impact.
- transfer posture: Business interruption insurance transfers some financial loss, not tenant harm.
- microservice mitigations: `cloud-cell-app`, `cell-domain`, `platform-cell-kernel`, `shared-tenant-quota-kernel`.
- Cedar policies: `policy.cell_mutation_scope`, `policy.cell_evacuation_dual_control`, `policy.cross_cell_fail_closed`.
- monitoring: `registry/dashboards/cell-routing-and-shuffle-sharding.yaml`, `registry/dashboards/golden-signals-per-microservice.yaml`, `specs/design-system/cloud-cell-topology-map.json`.
- named indicators: cross-cell error correlation, broker lag by cell, evacuation queue age, shared control-plane saturation, noisy-tenant quarantine count.
- early-warning trigger: two cells fail the same golden signal within a five-minute window.
- control evidence: cell game-day report, isolation test, failover drill, per-tenant blast-radius record.
- runbook reference: `docs/runbooks/cell-evacuation.md`.
- incident class: sev1-cell-cascade.
- customer communication: status-page per affected cell and tenant.
- regulator action reference: DORA operational resilience expectations for financial-sector tenants.
- ADR reference: ADR-0009, ADR-0248, ADR-0306, ADR-0152, ADR-0165.
- residual risk: Medium after per-cell isolation and recovery evidence are fresh.
- checkpoint: Run quarterly `cell-isolation-evidence` drill.
- escalation: SEV-1 bridge when more than one cell or regulated tenant is affected.

### OPS-002 - Audit-chain integrity breach
- risk-ID: OPS-002
- name: Audit-chain integrity breach
- category: Operational
- description: Evidence records may be missing, tampered, replayed, truncated, or emitted outside canonical audit-chain paths.
- threat landscape: Regulated customers depend on audit-chain integrity for DSR, HIPAA, SOX, DORA, and incident-response evidence; adversaries target logs to hide misuse.
- affected microservices: audit-chain, evidence, foundry, analytics, eventing, vertical-healthcare, ops.
- likelihood: Possible (3/5)
- impact: Critical (5/5)
- score: 15 Severe
- velocity: Fast when a breach or audit requires immediate evidence reconstruction.
- owner: ops-security
- owner role: Security Evidence owner with audit-chain service owner accountable.
- review cadence: Weekly until chain-seal coverage reaches target; monthly afterward.
- status: Open - reducing through seal checks and replay drills.
- acceptance posture: Reduce; do not accept silent evidence gaps on regulated paths.
- transfer posture: Cyber insurance transfers response cost only.
- microservice mitigations: `audit-chain-domain`, `platform-audit-chain-kernel`, `intelligence-evidence-evidence-pack-builder-adapter-audit-chain-bridge`.
- Cedar policies: `policy.audit_event_emit_required`, `policy.evidence_pack_read_scope`, `policy.audit_chain_replay_operator`.
- monitoring: `registry/dashboards/audit-event-emission-throughput.yaml`, `microservices/intelligence/dashboards/evidence-pack-assembly-rate.json`, `microservices/intelligence/dashboards/evidence-regulator-export-status.json`.
- named indicators: emission gap count, hash-chain verification failure, replay divergence, evidence export failure, per-capability missing event rate.
- early-warning trigger: audit-chain check fails for any regulated capability invocation.
- control evidence: chain integrity check, evidence replay, sealed event sample, missing-event exception ledger.
- runbook reference: `docs/runbooks/cross-axis/audit-chain-integrity-failure.md`.
- incident class: sev1-evidence-integrity.
- customer communication: regulator-ready evidence status and corrected export timeline.
- regulator action reference: HIPAA, GDPR accountability, DORA incident reporting, and SOX audit trail obligations.
- ADR reference: ADR-0003, ADR-0209, ADR-0263, ADR-0162.
- residual risk: Low after immutable storage, replay checks, and signed evidence packs are enforced.
- checkpoint: Every regulated capability must emit `EVT-*` proof.
- escalation: Security Council and Privacy Council when PHI/PII evidence is incomplete.

### OPS-003 - MLS key compromise in messenger
- risk-ID: OPS-003
- name: MLS key compromise in messenger
- category: Operational
- description: Messaging Layer Security keys, group secrets, device credentials, or recovery flows may be compromised, exposing tenant or personal communications.
- threat landscape: Endpoint compromise, malicious device enrollment, key-backup weakness, and cross-tenant group confusion can defeat end-to-end security claims.
- affected microservices: messenger, community, mail, identity, secrets, kms, tenancy, audit-chain.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast; compromise can spread across groups before detection.
- owner: axis-messenger, axis-community, axis-mail
- owner role: Security owner with ops-security co-owner.
- review cadence: Monthly; weekly after MLS protocol, client, or device-management changes.
- status: Open - mitigated through MLS conformance and key-health monitoring.
- acceptance posture: Reduce; accept only endpoint compromise residual risk with clear user controls.
- transfer posture: Cyber insurance and customer terms transfer cost, not confidentiality harm.
- microservice mitigations: `messenger-domain`, `cloud-kms-domain`, `secrets-domain`, `identity-domain`.
- Cedar policies: `policy.mls_group_membership_tenant_bound`, `policy.device_enrollment_step_up`, `policy.recovery_key_dual_control`.
- monitoring: `registry/dashboards/mls-key-delivery-health.yaml`, device enrollment anomaly dashboard, key rotation age.
- named indicators: unknown device joins, MLS welcome failure rate, group epoch divergence, recovery-key unwrap spike, tenant boundary mismatch.
- early-warning trigger: MLS delivery health drops while device enrollment spikes.
- control evidence: MLS conformance test, key transparency sample, device roster audit, forced rotation proof.
- runbook reference: `docs/runbooks/per-cell-hsm-rotation.md`.
- incident class: messaging-confidentiality.
- customer communication: affected group and device-level key-rotation notice.
- regulator action reference: GDPR confidentiality, PIPA safeguards, HIPAA ePHI when healthcare workflows use messenger, community, or mail services.
- ADR reference: ADR-0029, ADR-0188, ADR-0043, ADR-0299.
- residual risk: Medium because endpoint compromise remains partially outside platform control.
- checkpoint: No messenger stable release without MLS dashboard green.
- escalation: SEV-1 if regulated group contents are exposed.

### OPS-004 - Multi-tenant data leak
- risk-ID: OPS-004
- name: Multi-tenant data leak
- category: Operational
- description: Tenant-scoped data may leak through API authorization, query filters, search indexes, analytics, caches, object storage, exports, or support tooling.
- threat landscape: Multi-tenant SaaS breach patterns often arise from missing tenant predicates, broken object-level authorization, RLS gaps, and cross-tenant support access.
- affected microservices: tenancy, identity, policy, search, analytics, drive, mail, tasks, marketplace, audit-chain.
- likelihood: Possible (3/5)
- impact: Critical (5/5)
- score: 15 Severe
- velocity: Fast if exposed through a high-volume API or index.
- owner: council-privacy
- owner role: Privacy Engineering owner with ops-security co-owner.
- review cadence: Weekly until tenant-isolation tests and dashboards are complete.
- status: Open - mitigated by tenant universal-scoping primitive.
- acceptance posture: Reduce; no accepted silent cross-tenant access.
- transfer posture: Cyber insurance transfers response cost only.
- microservice mitigations: `platform-tenant-kernel`, `policy-cedar-domain`, `http-tenant-middleware-domain`, `search-domain`.
- Cedar policies: `policy.tenant_scope_required`, `policy.support_access_break_glass`, `policy.cross_tenant_denied_by_default`.
- monitoring: `registry/dashboards/tenant-isolation-health.yaml`, policy deny-rate dashboards, cross-tenant access negative tests.
- named indicators: missing tenant_id annotation, cross-tenant deny spike, support access anomaly, search index tenant mismatch, object ACL drift.
- early-warning trigger: any test reads tenant B data using tenant A principal.
- control evidence: tenant isolation test, data-class annotation, Cedar decision trace, audit-chain event.
- runbook reference: `docs/runbooks/cell-isolation-breach.md`.
- incident class: sev1-tenant-data-leak.
- customer communication: breach notification workflow and trust portal update.
- regulator action reference: GDPR/PIPA/CCPA/HIPAA breach notification regimes.
- ADR reference: ADR-0002, ADR-0008, ADR-0244, ADR-0095, ADR-0162.
- residual risk: Medium until every microservice has enforced tenant middleware and negative tests.
- checkpoint: New API cannot release without tenant-isolation proof.
- escalation: Privacy Council and Security Council immediately.

### OPS-005 - Recovery key envelope compromise
- risk-ID: OPS-005
- name: Recovery key envelope compromise
- category: Operational
- description: Tenant recovery keys, backup envelopes, Shamir shares, or BYOK materials may be exposed, coerced, lost, or misused.
- threat landscape: Recovery paths are attractive because they bypass normal authentication, device possession, and real-time user approval controls.
- affected microservices: secrets, kms, identity, tenancy, messenger, community, mail, audit-chain, cloud-storage.
- likelihood: Unlikely (2/5)
- impact: Critical (5/5)
- score: 10 Elevated
- velocity: Fast when recovery is requested under duress.
- owner: ops-security
- owner role: Key Management owner with council-privacy co-owner.
- review cadence: Monthly; weekly after key-management changes or high-risk tenant onboarding.
- status: Open - mitigating through envelope separation and dual control.
- acceptance posture: Reduce; accept only documented lost-share residual risk.
- transfer posture: Insurance covers breach cost; custody accountability remains internal.
- microservice mitigations: `cloud-kms-domain`, `secrets-domain`, `platform-audit-chain-kernel`, `identity-domain`.
- Cedar policies: `policy.recovery_key_unwrap_m_of_n`, `policy.byok_rotation_tenant_duress`, `policy.key_custodian_separation`.
- monitoring: key unwrap count, failed M-of-N requests, duress flag, recovery request geovelocity, HSM health.
- named indicators: unusual recovery frequency, custodian mismatch, envelope age, missing attestation, failed share validation.
- early-warning trigger: recovery request lacks independent custodian approval or occurs during tenant dispute.
- control evidence: key ceremony log, HSM attestation, dual-control Cedar decision, sealed audit event.
- runbook reference: `docs/runbooks/shamir-share-loss-or-coercion.md`.
- incident class: key-custody.
- customer communication: tenant security contact notification with legal hold if needed.
- regulator action reference: GDPR security of processing and HIPAA ePHI safeguards when regulated data is encrypted.
- ADR reference: ADR-0043, ADR-0299, ADR-0312, ADR-0189.
- residual risk: Low after HSM-backed dual control and customer-managed keys.
- checkpoint: Recovery flow must pass quarterly tabletop.
- escalation: Security Council plus legal if coercion or lawful access is alleged.

### OPS-006 - Cross-tenant Cedar policy escape
- risk-ID: OPS-006
- name: Cross-tenant Cedar policy escape
- category: Operational
- description: A malformed, stale, overly broad, or wrongly scoped Cedar policy could authorize an action across tenant, region, data-class, or autonomy boundaries.
- threat landscape: Policy sprawl, fragment composition errors, emergency overrides, and schema drift can turn a formally secure policy language into an unsafe authorization plane.
- affected microservices: policy, tenancy, foundry, application, messenger, community, mail, marketplace, analytics, tasks.
- likelihood: Possible (3/5)
- impact: Critical (5/5)
- score: 15 Severe
- velocity: Fast when a bad policy is published globally.
- owner: ops-security
- owner role: Policy Security owner with policy microservice owner accountable.
- review cadence: Weekly until fragment soak and rollback are proven; monthly afterward.
- status: Open - mitigating through fragment discipline and anomaly rollback.
- acceptance posture: Reduce; no accepted cross-tenant allow without explicit contract.
- transfer posture: Not meaningfully transferable.
- microservice mitigations: `policy-cedar-domain`, `policy-cedar-api`, `check-cedar-fragment-coverage`, `intelligence-guardrails-cedar-engine`.
- Cedar policies: `policy.cedar_publish_review_required`, `policy.tenant_boundary_invariant`, `policy.fragment_soak_required`, `policy.emergency_policy_rollback`.
- monitoring: `registry/dashboards/cedar-policy-evaluation-latency.yaml`, policy deny-rate dashboards, fragment anomaly alerts.
- named indicators: allow-rate spike, deny-rate collapse, policy evaluation latency spike, fragment publish without soak, schema mismatch.
- early-warning trigger: a new fragment changes allow decisions for more tenants than declared.
- control evidence: Cedar simulation diff, soak result, policy provenance, rollback drill.
- runbook reference: `docs/runbooks/cedar-fragment-emergency-rollback.md`.
- incident class: authorization-escape.
- customer communication: affected permission surface and remediation window.
- regulator action reference: data protection accountability when policy escape reaches personal data.
- ADR reference: ADR-0007, ADR-0183, ADR-0191, ADR-0243, ADR-0294.
- residual risk: Medium until policy explosion controls and per-tenant diffing are automated.
- checkpoint: No policy fragment publishes without simulated blast-radius.
- escalation: SEV-1 when cross-tenant access is permitted.

### OPS-007 - Residency router misroutes regulated data
- risk-ID: OPS-007
- name: Residency router misroutes regulated data
- category: Operational
- description: Data, audit logs, backups, search indexes, or evidence exports may be routed to a region or subprocesser not allowed by tenant, sector, or sovereign pack.
- threat landscape: Region failover, analytics replication, object-storage lifecycle policies, and vendor support workflows can bypass residency intent.
- affected microservices: residency, tenancy, cloud-storage, analytics, search, audit-chain, regional-pack.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast during failover, bulk export, or disaster recovery.
- owner: regional-packs
- owner role: Residency Compliance owner with axis-cloud accountable.
- review cadence: Monthly; weekly during new-region or sovereign pack launch.
- status: Open - mitigated by residency deny-by-default policies.
- acceptance posture: Reduce; accept only formally waived and documented non-sensitive residual flows.
- transfer posture: subprocesser terms transfer remedies, not regulator accountability.
- microservice mitigations: `platform-residency-kernel`, `residency-domain`, `cloud-region-domain`, `cloud-storage-domain`.
- Cedar policies: `policy.residency_region_allowlist`, `policy.sovereign_child_tenant_lock`, `policy.failover_residency_hold`.
- monitoring: residency egress detector, `registry/dashboards/compliance-pack-attestation-lag.yaml`, cross-region replication audit.
- named indicators: disallowed region writes, backup object geography mismatch, failover hold count, subprocesser route mismatch, export pack warning.
- early-warning trigger: DR drill attempts to restore a tenant into a non-approved region.
- control evidence: residency manifest, cell-routed session log, legal basis record, audit-chain event.
- runbook reference: `docs/runbooks/tenant-data-residency-violation.md`.
- incident class: residency-breach.
- customer communication: regulator notification clock and tenant DPO packet.
- regulator action reference: PIPA cross-border transfer, GDPR Chapter V, sovereign-cloud contract obligations.
- ADR reference: ADR-0049, ADR-0164, ADR-0240, ADR-0313.
- residual risk: Medium because failover pressure competes with residency constraints.
- checkpoint: Residency route validation before every region failover.
- escalation: Privacy Council and Regional Pack lead.

### OPS-008 - Incident runbook drift and incomplete evidence
- risk-ID: OPS-008
- name: Incident runbook drift and incomplete evidence
- category: Operational
- description: Runbooks may exist but be stale, undiscoverable, untested, or missing required regulator, customer, and audit-chain evidence steps.
- threat landscape: Documentation drift is amplified by fast microservice growth, generated artifacts, and many specialized incident classes.
- affected microservices: ops, foundry, audit-chain, compliance, all customer-facing microservices.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; drift becomes visible during incident response.
- owner: ops-sre-reliability
- owner role: Runbook owner with doc-catalog reviewer.
- review cadence: Weekly runbook-index review; quarterly game-day validation.
- status: Open - reducing through runbook-index and drill evidence.
- acceptance posture: Reduce; accept minor prose drift only when critical command path is valid.
- transfer posture: Not transferable.
- microservice mitigations: `intelligence-evidence-evidence-pack-builder-usecase`, `check-runbook-index-resolves`, `check-doc-catalog`.
- Cedar policies: `policy.incident_commander_runbook_ack`, `policy.evidence_pack_required_for_close`, `policy.regulator_notice_dual_control`.
- monitoring: runbook freshness, orphan runbook count, incident evidence completeness, game-day pass rate.
- named indicators: broken runbook link, stale owner, failed drill, missing evidence pack, postmortem action reopened.
- early-warning trigger: incident commander cannot find the authoritative runbook in two minutes.
- control evidence: drill record, runbook index row, incident evidence pack, postmortem closure.
- runbook reference: `docs/runbooks/ops/sev-1-bridge-procedure.md`.
- incident class: operations-governance.
- customer communication: not external unless incident response misses an SLA.
- regulator action reference: DORA incident process and HIPAA breach documentation expectations.
- ADR reference: ADR-0019, ADR-0263, ADR-0241, ADR-0152.
- residual risk: Medium while microservice catalogue is expanding.
- checkpoint: Every Severe/Extreme risk must have at least one discoverable runbook.
- escalation: SRE lead when runbook-index validation fails.

### OPS-009 - KMS, OpenBao, or HSM outage blocks tenant operations
- risk-ID: OPS-009
- name: KMS, OpenBao, or HSM outage blocks tenant operations
- category: Operational
- description: Key-management infrastructure may become unavailable, preventing encryption, decryption, signing, BYOK rotation, recovery, webhook signing, or evidence sealing.
- threat landscape: Per-cell HSM dependencies, cloud KMS quotas, OpenBao availability, certificate rotation, and KCMVP/sovereign constraints create hard operational choke points.
- affected microservices: kms, secrets, cloud-iam, audit-chain, messenger, community, mail, storage, identity, regional-pack.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast; key unavailability can stop core tenant operations immediately.
- owner: axis-cloud
- owner role: Cloud KMS owner with ops-security backup.
- review cadence: Monthly; weekly for sovereign and regulated cells.
- status: Open - mitigated by per-cell failover and key rotation drills.
- acceptance posture: Reduce; accept short degraded read-only mode when documented.
- transfer posture: vendor SLA credits transfer limited cost only.
- microservice mitigations: `cloud-kms-domain`, `secrets-domain`, `platform-audit-chain-kernel`, `cloud-iam-domain`.
- Cedar policies: `policy.kms_rotation_operator`, `policy.hsm_break_glass_dual_control`, `policy.key_material_region_bound`.
- monitoring: HSM health, unwrap latency, signing failure rate, KMS quota, OpenBao seal status.
- named indicators: decrypt error spike, HSM quorum loss, OpenBao sealed unexpectedly, key rotation missed, certificate expiry.
- early-warning trigger: p95 key unwrap latency breaches SLO for regulated cell.
- control evidence: key rotation drill, HSM attestation, backup restore, runbook execution.
- runbook reference: `docs/runbooks/cloud/kms-emergency-rotation.md`.
- incident class: crypto-control-plane.
- customer communication: degraded-mode notice for affected tenants.
- regulator action reference: HIPAA Security Rule, DORA resilience, and PIPA safeguard expectations.
- ADR reference: ADR-0043, ADR-0161, ADR-0254, ADR-0164.
- residual risk: Medium where sovereign hardware lead time limits redundancy.
- checkpoint: Per-cell HSM rotation drill every quarter.
- escalation: SEV-1 if data access or evidence sealing is unavailable for regulated tenants.

### OPS-010 - Webhook DLQ and outbox replay storm
- risk-ID: OPS-010
- name: Webhook DLQ and outbox replay storm
- category: Operational
- description: Retried webhooks, dead-letter queues, or outbox relays may replay stale events, duplicate side effects, or overwhelm downstream services.
- threat landscape: Distributed workflows, customer integrations, evidence export, and cross-tenant events depend on idempotency and bounded retries.
- affected microservices: eventing, workflow-engine, foundry, marketplace, tenant integrations, audit-chain.
- likelihood: Possible (3/5)
- impact: Major (3/5)
- score: 9 Managed
- velocity: Fast when backlog drains after an outage.
- owner: ops-sre-reliability
- owner role: Eventing owner.
- review cadence: Monthly; weekly after outbox or event schema changes.
- status: Open - mitigated by idempotency and DLQ runbooks.
- acceptance posture: Reduce; accept duplicate delivery only when idempotency holds.
- transfer posture: not transferable.
- microservice mitigations: `eventing-domain`, `intelligence-run-domain`, `workflow-engine-domain`, `marketplace-domain`.
- Cedar policies: `policy.webhook_retry_scope`, `policy.dlq_replay_operator`, `policy.idempotency_key_required`.
- monitoring: DLQ depth, outbox lag, replay error rate, duplicate command count, downstream throttle.
- named indicators: backlog age, replay batch size, idempotency reject count, customer webhook 5xx, audit emission lag.
- early-warning trigger: DLQ depth and retry rate rise together after downstream recovery.
- control evidence: idempotency test, replay dry-run, event schema version check, DLQ drain report.
- runbook reference: `docs/runbooks/webhook-delivery-failure.md`.
- incident class: eventing-replay.
- customer communication: integration status notice and replay window disclosure.
- regulator action reference: DORA incident and audit integrity for financial workflows.
- ADR reference: ADR-0149, ADR-0153, ADR-0169, ADR-0154.
- residual risk: Low after replay dry-run controls.
- checkpoint: No DLQ replay without dry-run diff.
- escalation: SRE lead when regulated evidence events are delayed.

### OPS-011 - Control-plane mutation against wrong tenant or environment
- risk-ID: OPS-011
- name: Control-plane mutation against wrong tenant or environment
- category: Operational
- description: Operators, agents, or automation may apply cloud, policy, feature, billing, or data mutations to the wrong tenant, cell, region, or environment.
- threat landscape: Agentic operations and multi-environment control planes raise risk of target confusion, stale context, and overbroad credentials.
- affected microservices: ops, foundry, tenancy, feature-flags, cloud-iac, policy, billing.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast; wrong mutation can immediately affect many users.
- owner: axis-foundry
- owner role: Agentic Operations owner with SRE co-owner.
- review cadence: Weekly until admission gates and dry-run evidence are complete.
- status: Open - mitigating through target confirmation and Cedar-scoped claims.
- acceptance posture: Reduce; no accepted production mutation without typed target evidence.
- transfer posture: not transferable.
- microservice mitigations: `intelligence-cloud-mutation-domain`, `feature-flags-domain`, `cloud-iac-domain`, `tenant-cli`.
- Cedar policies: `policy.cloud_mutation_tenant_target`, `policy.production_change_dual_control`, `policy.agent_claim_scope_bound`.
- monitoring: cloud mutation audit, feature flag blast radius, agent claim mismatch, environment selector drift.
- named indicators: dry-run target mismatch, tenant selector ambiguity, environment variable drift, mutation rollback count.
- early-warning trigger: operator command target differs from Oya VCS claim scope.
- control evidence: dry-run artifact, Cedar permit, change bundle, audit-chain event, rollback proof.
- runbook reference: `docs/runbooks/cloud-mutation.md` if promoted; interim `docs/runbooks/release-rollback.md`.
- incident class: control-plane-misfire.
- customer communication: affected tenant change notice if mutation reaches customer state.
- regulator action reference: DORA change-management expectations for regulated financial tenants.
- ADR reference: ADR-0110, ADR-0113, ADR-0223, ADR-0202, ADR-0295.
- residual risk: Medium while agentic control surfaces mature.
- checkpoint: Production mutations require dry-run plus scoped policy permit.
- escalation: SEV-1 if customer data, security, or availability changes.

### OPS-012 - Backup restore or disaster-recovery integrity failure
- risk-ID: OPS-012
- name: Backup restore or disaster-recovery integrity failure
- category: Operational
- description: Backups may be incomplete, unrecoverable, out of residency bounds, missing key material, or inconsistent across event, database, object, audit, and search stores.
- threat landscape: Restore is often untested until ransomware, region outage, operator error, or regulator evidence request requires proof.
- affected microservices: cloud-storage, database, audit-chain, search, analytics, tenancy, kms, workflow-engine.
- likelihood: Possible (3/5)
- impact: Critical (5/5)
- score: 15 Severe
- velocity: Medium; restore failure materializes during high-pressure outage.
- owner: ops-sre-reliability
- owner role: Disaster Recovery owner with axis-cloud co-owner.
- review cadence: Monthly restore drill; weekly during new storage tier rollout.
- status: Open - reducing through RPO/RTO and DR drills.
- acceptance posture: Reduce; accept only documented RPO/RTO per tenant tier.
- transfer posture: cyber and business interruption insurance transfer cost only.
- microservice mitigations: `cloud-storage-domain`, `platform-audit-chain-kernel`, `cloud-cell-app`, `search-domain`.
- Cedar policies: `policy.restore_operator_dual_control`, `policy.backup_region_allowed`, `policy.rpo_rto_tier_enforced`.
- monitoring: backup age, restore drill pass, RPO lag, key availability, cross-store reconciliation.
- named indicators: failed restore, missing audit segment, object checksum mismatch, backup out of region, untested tenant tier.
- early-warning trigger: any regulated tenant exceeds declared RPO.
- control evidence: restore drill, checksum manifest, key unwrap test, audit replay, tenant sign-off.
- runbook reference: `docs/runbooks/dr-drill-playbook.md`.
- incident class: disaster-recovery.
- customer communication: RPO/RTO disclosure and post-incident recovery report.
- regulator action reference: DORA operational resilience and HIPAA contingency plan expectations.
- ADR reference: ADR-0152, ADR-0241, ADR-0184, ADR-0306.
- residual risk: Medium because complete restore spans many state stores.
- checkpoint: Restore drill evidence required before stable launch.
- escalation: Board risk review when restore fails for regulated tenant class.

## Section 4. Compliance and Regulatory Risks

### REG-001 - GDPR transparency and information enforcement
- risk-ID: REG-001
- name: GDPR transparency and information enforcement
- category: Compliance-Regulatory
- description: Oyatie may fail to provide clear, complete, timely, and role-specific GDPR Articles 12-14 notices for complex agentic, cross-tenant, and AI-enabled processing.
- threat landscape: EDPB CEF 2026 focuses on transparency and information obligations, increasing likelihood of regulator or customer scrutiny.
- affected microservices: privacy, consent, analytics, foundry, messenger, community, mail, workspace, regional-pack, trust-portal.
- likelihood: Likely (4/5)
- impact: Severe (4/5)
- score: 16 Severe
- velocity: Medium; enforcement inquiries can start with transparency gaps in public notices or DSR responses.
- owner: council-privacy
- owner role: Data Protection Officer delegate.
- review cadence: Monthly Compliance Review; weekly during EU customer onboarding.
- status: Open - mitigated through notice inventory and data-use boundary.
- acceptance posture: Reduce; no accepted unclear processing notice for regulated flows.
- transfer posture: privacy counsel supports interpretation; controller accountability remains.
- microservice mitigations: `dsr-domain`, `data-boundary-kernel`, `regional-pack-api`, `intelligence-evidence-regulator-export`.
- Cedar policies: `policy.notice_required_before_processing`, `policy.data_class_notice_match`, `policy.dsr_export_allowed_scope`.
- monitoring: DSR SLA, notice coverage, consent change delta, privacy policy diff review, pack attestation age.
- named indicators: unnotified data class, DSR clarification spike, customer DPA redline, regulator inquiry, privacy notice stale age.
- early-warning trigger: a microservice adds a data class without notice mapping.
- control evidence: data inventory, privacy notice map, DSR response sample, DPIA, audit-chain event.
- runbook reference: `docs/runbooks/privacy-council-data-class-review.md`.
- incident class: privacy-transparency.
- customer communication: updated privacy notice and DPA exhibit.
- regulator action reference: EDPB Coordinated Enforcement Framework 2026.
- ADR reference: ADR-0008, ADR-0156, ADR-0209, ADR-0272.
- residual risk: Medium because processing surfaces change rapidly.
- checkpoint: Every data class must map to notice, legal basis, retention, and DSR behavior.
- escalation: DPO when any notice gap reaches production.

### REG-002 - GDPR cross-border transfer and SCC failure
- risk-ID: REG-002
- name: GDPR cross-border transfer and SCC failure
- category: Compliance-Regulatory
- description: EU personal data may be transferred, accessed, stored, supported, or replicated outside allowed regions without valid transfer mechanism, supplementary measures, or customer-specific restriction.
- threat landscape: EDPB and Irish DPC Meta transfer enforcement remains the named signal for high-impact Chapter V transfer failures.
- affected microservices: residency, analytics, search, foundry, support tooling, cloud-storage, audit-chain, messenger, community, mail.
- likelihood: Possible (3/5)
- impact: Critical (5/5)
- score: 15 Severe
- velocity: Fast during support access, provider failover, evidence export, or backup restore.
- owner: council-privacy
- owner role: DPO delegate with regional-packs co-owner.
- review cadence: Monthly; weekly during EU tenant launch or subprocesser change.
- status: Open - mitigated by residency and transfer manifests.
- acceptance posture: Reduce; do not accept silent EU transfer outside approved mechanism.
- transfer posture: subprocesser SCCs transfer contractual remedies only.
- microservice mitigations: `platform-residency-kernel`, `cloud-region-domain`, `intelligence-evidence-regulator-export-framework-profiles`, `dsr-domain`.
- Cedar policies: `policy.eu_transfer_mechanism_required`, `policy.support_access_region_bound`, `policy.scc_subprocessor_check`.
- monitoring: transfer manifest drift, region access logs, subprocesser roster, residency egress detector, export destination.
- named indicators: US access to EU data, missing SCC, stale transfer impact assessment, export job route mismatch, support session from disallowed country.
- early-warning trigger: new provider adapter routes EU tenant data through a non-approved subprocesser.
- control evidence: transfer impact assessment, SCC exhibit, residency manifest, access log, audit-chain event.
- runbook reference: `docs/runbooks/cross-pack-tenant-residency.md`.
- incident class: privacy-transfer.
- customer communication: DPA transfer exhibit update and affected tenant notice if needed.
- regulator action reference: EDPB Binding Decision 1/2023 on Meta transfers.
- ADR reference: ADR-0049, ADR-0276, ADR-0240, ADR-0304.
- residual risk: Medium due to provider, support, and disaster-recovery complexity.
- checkpoint: No EU tenant go-live without transfer route test.
- escalation: Privacy Council plus legal counsel on Chapter V conflict.

### REG-003 - Korea PIPA foreign-operator and cross-border enforcement
- risk-ID: REG-003
- name: Korea PIPA foreign-operator and cross-border enforcement
- category: Compliance-Regulatory
- description: Oyatie may fail Korean PIPA duties around consent, foreign operator notices, cross-border entrustment, breach notice, domestic agent, or behavioral data processing.
- threat landscape: PIPC has sanctioned Google/Meta for behavioral advertising, OpenAI for breach notification and safeguards, Temu for cross-border transfer, and KAB/TELUS AI for SQLi safeguards.
- affected microservices: regional-pack-kr, consent, analytics, foundry, messenger, community, mail, marketplace, tenancy, trust-portal.
- likelihood: Likely (4/5)
- impact: Severe (4/5)
- score: 16 Severe
- velocity: Medium; enforcement can follow public launch, breach, or cross-border flow discovery.
- owner: regional-packs
- owner role: Korea Pack owner with council-privacy co-owner.
- review cadence: Monthly Korean regulator watch; weekly for KR customer onboarding.
- status: Open - mitigated by KR pack and PIPC watch.
- acceptance posture: Reduce; delay KR feature launch when PIPA mapping is incomplete.
- transfer posture: local counsel supports interpretation; accountability remains internal.
- microservice mitigations: `regional-pack-api`, `platform-residency-kernel`, `data-boundary-kernel`, `intelligence-evidence-regulator-export`.
- Cedar policies: `policy.kr_pipa_cross_border_notice`, `policy.kr_behavioral_data_consent`, `policy.kr_domestic_agent_required`.
- monitoring: regulator publication feed, KR pack attestation lag, consent opt-in rate, cross-border transfer manifest, vulnerability backlog.
- named indicators: missing Korean notice, unlisted overseas processor, breach notice late, behavior-based analytics without consent, SQLi critical finding.
- early-warning trigger: a KR tenant flow sends personal data to non-KR processor without pack overlay.
- control evidence: KR PIPA mapping, local notice, processor roster, consent receipt, breach drill evidence.
- runbook reference: `docs/runbooks/regulator-publication-feed-health.md`.
- incident class: kr-pipa-compliance.
- customer communication: Korean-language tenant notice and DPO support packet.
- regulator action reference: PIPC Google/Meta, OpenAI, Temu, and KAB/TELUS sanctions.
- ADR reference: ADR-0010, ADR-0064, ADR-0240, ADR-0304.
- residual risk: Medium because KR scope is broad and fast-moving.
- checkpoint: KR regional pack must be current before KR production use.
- escalation: Korea Pack lead and outside Korean counsel.

### REG-004 - HIPAA business associate PHI safeguard failure
- risk-ID: REG-004
- name: HIPAA business associate PHI safeguard failure
- category: Compliance-Regulatory
- description: Healthcare workflows may process PHI without sufficient risk analysis, access control, audit controls, breach notification, or business associate obligations.
- threat landscape: HHS OCR 2026 MMG Fusion settlement highlights business-associate breach and risk-analysis enforcement for software vendors.
- affected microservices: vertical-healthcare, messenger, community, mail, workflow-engine, analytics, identity, audit-chain, evidence, cloud-storage.
- likelihood: Possible (3/5)
- impact: Critical (5/5)
- score: 15 Severe
- velocity: Fast after PHI breach, ransomware, or unsupported healthcare deployment.
- owner: vertical-healthcare
- owner role: Healthcare Compliance owner with ops-security co-owner.
- review cadence: Monthly HIPAA controls review; weekly before healthcare go-live.
- status: Open - mitigated by healthcare pack and audit evidence.
- acceptance posture: Reduce; do not accept PHI processing without BAA and security controls.
- transfer posture: cyber insurance and BAAs transfer defined cost/risk, not OCR accountability.
- microservice mitigations: `shared-compliance-evidence-kernel`, `platform-audit-chain-kernel`, `dsr-domain`, `cloud-storage-domain`.
- Cedar policies: `policy.hipaa_phi_minimum_necessary`, `policy.healthcare_break_glass`, `policy.baa_required_for_phi`.
- monitoring: PHI data-class coverage, access audit completeness, breach clock, healthcare pack attestation, risk-analysis freshness.
- named indicators: PHI flow without data_class, access log gap, unencrypted export, missing BAA, risk analysis older than review cadence.
- early-warning trigger: healthcare customer enables workflow before HIPAA pack passes.
- control evidence: risk analysis, BAA record, audit log, breach drill, encryption evidence.
- runbook reference: `docs/runbooks/vertical-healthcare/phi-leak-suspected.md`.
- incident class: hipaa-phi.
- customer communication: covered-entity breach support packet and OCR notification workflow.
- regulator action reference: HHS OCR MMG Fusion 2026 settlement.
- ADR reference: ADR-0008, ADR-0209, ADR-0298, ADR-0312.
- residual risk: Medium until healthcare scope is fully bounded.
- checkpoint: PHI-capable workflows require healthcare compliance gate.
- escalation: HIPAA counsel and Privacy Council within breach clock.

### REG-005 - CCPA/CPRA opt-out, sharing, and minimization enforcement
- risk-ID: REG-005
- name: CCPA/CPRA opt-out, sharing, and minimization enforcement
- category: Compliance-Regulatory
- description: California consumers may not receive effective opt-out, GPC honoring, data-sharing disclosures, sensitive data limits, or deletion behavior across Oyatie surfaces.
- threat landscape: California OAG/CPPA enforcement against Sephora, Healthline, Disney, Jam City, and General Motors shows active focus on opt-out, sharing, children, sensitive health/location data, and minimization.
- affected microservices: consent, analytics, ads-analytics, marketplace, messenger, community, mail, workspace, trust-portal, regional-pack-us.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; risk rises with public consumer or personal-tenant surfaces.
- owner: council-privacy
- owner role: US Privacy owner.
- review cadence: Monthly; weekly before personal tenant or ads/analytics launch.
- status: Open - mitigated through consent cascade and GPC support.
- acceptance posture: Reduce; do not accept broken opt-out on sale/share flows.
- transfer posture: privacy counsel and insurance cover cost, not statutory compliance.
- microservice mitigations: `consent-domain`, `ads-analytics-domain`, `dsr-domain`, `data-boundary-kernel`.
- Cedar policies: `policy.ccpa_do_not_sell_share`, `policy.gpc_honor_required`, `policy.sensitive_data_minimization`.
- monitoring: opt-out propagation SLA, GPC request pass rate, deletion cascade, sharing inventory, data-minimization audit.
- named indicators: opt-out not propagated across devices, ad-tech sharing without contract, sensitive data in analytics, minor consent gap.
- early-warning trigger: a consumer opt-out does not reach downstream analytics within SLA.
- control evidence: consent receipt, GPC test, processor contract, deletion proof, data-map row.
- runbook reference: `docs/runbooks/consent-withdrawal-cascade.md`.
- incident class: ccpa-privacy-rights.
- customer communication: updated privacy notice and request-status response.
- regulator action reference: California OAG privacy enforcement actions through 2026.
- ADR reference: ADR-0272, ADR-0008, ADR-0031, ADR-0209.
- residual risk: Medium while consumer-facing surfaces expand.
- checkpoint: No ads/analytics launch without GPC and opt-out tests.
- escalation: Privacy Council when opt-out propagation fails.

### REG-006 - EU AI Act prohibited practice and high-risk compliance
- risk-ID: REG-006
- name: EU AI Act prohibited practice and high-risk compliance
- category: Compliance-Regulatory
- description: Oyatie may provide or deploy AI systems in EU contexts without proper classification, technical documentation, human oversight, transparency, post-market monitoring, or prohibited-practice refusal.
- threat landscape: EU AI Act provisions apply progressively, with prohibitions already applicable and high-risk/transparency obligations starting in 2026 for many systems.
- affected microservices: foundry, guardrails, workflow-engine, tasks, vertical-fintech, vertical-healthcare, analytics.
- likelihood: Likely (4/5)
- impact: Severe (4/5)
- score: 16 Severe
- velocity: Fast if a feature crosses from assistive workflow into high-risk decisioning.
- owner: ops-compliance
- owner role: AI Compliance owner.
- review cadence: Weekly through 2026-08-02 obligations; monthly afterward.
- status: Open - mitigating through risk-class registry.
- acceptance posture: Avoid prohibited practices; reduce high-risk obligations.
- transfer posture: not transferable except for counsel and conformity-assessment support.
- microservice mitigations: `specs/capabilities/eu-ai-act-risk-class-registry.json`, `check-high-risk-auto-decision-refusal`, `intelligence-guardrails-output-validator-kernel`.
- Cedar policies: `policy.eu_ai_act_classification_required`, `policy.prohibited_ai_refusal`, `policy.high_risk_human_oversight`.
- monitoring: risk-class coverage, high-risk capability count, human oversight SLA, transparency notice coverage, model eval drift.
- named indicators: Annex III match, automated decision appeal count, high-risk system without technical file, transparency notice missing.
- early-warning trigger: a customer configures AI for credit, employment, education, healthcare, law enforcement, or essential services.
- control evidence: AI risk file, technical documentation, eval set, oversight trace, post-market monitoring report.
- runbook reference: `docs/runbooks/foundry/capability-eval-regression.md`.
- incident class: eu-ai-act.
- customer communication: deployer obligation packet and capability classification.
- regulator action reference: EU AI Act Service Desk timeline and FAQ.
- ADR reference: ADR-0144, ADR-0308, ADR-0309, ADR-0022.
- residual risk: Medium because classification guidance and product use cases evolve.
- checkpoint: AI capability cannot promote without classification.
- escalation: AI Compliance owner and outside EU counsel.

### REG-007 - DORA ICT third-party and operational resilience exposure
- risk-ID: REG-007
- name: DORA ICT third-party and operational resilience exposure
- category: Compliance-Regulatory
- description: Financial-sector customers may require DORA-aligned ICT risk management, incident reporting, resilience testing, subcontractor control, and exit planning that Oyatie cannot yet evidence.
- threat landscape: DORA has applied since 2025-01-17, with EU implementation and enforcement activity creating procurement and audit pressure.
- affected microservices: cloud, foundry, observability, incident, audit-chain, provider adapters, marketplace, ops.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; triggered by financial customer audits or incident.
- owner: ops-compliance
- owner role: Financial Services Compliance owner.
- review cadence: Monthly; weekly for EU financial customer onboarding.
- status: Open - mitigated through DORA workflow templates and evidence export.
- acceptance posture: Reduce; accept delayed financial-sector go-live if evidence incomplete.
- transfer posture: contractual liability caps and insurance transfer some cost only.
- microservice mitigations: `intelligence-evidence-regulator-export`, `observability-domain`, `cloud-cell-app`, `provider-router-domain`.
- Cedar policies: `policy.dora_financial_tenant_controls`, `policy.ict_subprocessor_review_required`, `policy.incident_report_dual_control`.
- monitoring: resilience drill age, incident reporting clock, critical ICT provider roster, subcontractor change notice, exit-plan test.
- named indicators: customer DORA questionnaire gap, untested DR, subcontractor no-notice change, incident-report SLA breach.
- early-warning trigger: financial customer marks Oyatie as critical ICT provider without evidence pack.
- control evidence: resilience test, incident report template, subcontractor register, exit drill, audit-chain proof.
- runbook reference: `docs/runbooks/ops/regulator-notification-procedure.md`.
- incident class: dora-operational-resilience.
- customer communication: financial-sector evidence pack and regulator-ready incident report.
- regulator action reference: European Commission DORA transposition and enforcement snapshot.
- ADR reference: ADR-0241, ADR-0152, ADR-0180, ADR-0209.
- residual risk: Medium until financial services pack is certified.
- checkpoint: DORA evidence pack before EU financial production.
- escalation: Compliance Council and legal.

### REG-008 - Sovereign data-residency conflict
- risk-ID: REG-008
- name: Sovereign data-residency conflict
- category: Compliance-Regulatory
- description: Sovereign-cloud, public-sector, defense, critical-infrastructure, or national-security customers may require local hosting, local operations, local keys, or air-gapped controls that conflict with shared platform economics.
- threat landscape: Sovereign cloud demands can collide with hyperscaler operations, support access, AI provider calls, telemetry aggregation, and cross-region DR.
- affected microservices: sovereign-cloud, cloud-cell, kms, observability, foundry, residency, support tooling, regional-pack.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; emerges during RFP and certification.
- owner: axis-cloud
- owner role: Sovereign Cloud owner with regional-packs co-owner.
- review cadence: Monthly; weekly during sovereign customer pursuit.
- status: Open - mitigated through sovereign overlays and air-gapped doctrine.
- acceptance posture: Reduce; accept slower feature availability in sovereign cells.
- transfer posture: local hosting partner transfers operations only when contractually bound.
- microservice mitigations: `platform-residency-kernel`, `cloud-cell-app`, `cloud-kms-domain`, `intelligence-providers-router-domain`.
- Cedar policies: `policy.sovereign_cell_no_external_provider`, `policy.local_operator_required`, `policy.telemetry_redaction_required`.
- monitoring: sovereign cell egress, local key custody, provider-call deny count, telemetry export attempts, certification status.
- named indicators: external AI call from sovereign cell, nonlocal support access, telemetry aggregate with personal data, HSM nonlocal.
- early-warning trigger: sovereign tenant requests feature backed by external provider or global telemetry.
- control evidence: sovereign overlay manifest, egress deny logs, local key attestation, air-gap drill.
- runbook reference: `docs/runbooks/cross-pack-tenant-residency.md`.
- incident class: sovereign-residency.
- customer communication: sovereign feature matrix and unsupported control disclosure.
- regulator action reference: national sovereign-data and public-sector certification obligations.
- ADR reference: ADR-0164, ADR-0240, ADR-0251, ADR-0313.
- residual risk: Medium because sovereign requirements vary by country.
- checkpoint: No sovereign label without overlay evidence.
- escalation: Architecture Council before accepting sovereign exceptions.

### REG-009 - Export control, sanctions, and restricted capability misuse
- risk-ID: REG-009
- name: Export control, sanctions, and restricted capability misuse
- category: Compliance-Regulatory
- description: Oyatie capabilities, AI models, cryptography, cloud infrastructure, marketplace flows, or customer workflows may be used by sanctioned parties or in restricted jurisdictions.
- threat landscape: Export-control and sanctions rules can apply to AI, encryption, cloud, marketplace, financial, and dual-use operational capabilities.
- affected microservices: marketplace, identity, tenancy, foundry, cloud, payments, regional-pack, policy.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast if a sanctioned actor signs up or a customer exports restricted capability.
- owner: ops-compliance
- owner role: Sanctions and Export Control owner.
- review cadence: Monthly; weekly after geopolitical or sanctions-list changes.
- status: Open - mitigated through screening and region deny policies.
- acceptance posture: Avoid sanctioned use; reduce ambiguous dual-use exposure.
- transfer posture: screening vendors transfer data support only.
- microservice mitigations: `marketplace-domain`, `tenancy-domain`, `intelligence-guardrails-rule-store`, `regional-pack-api`.
- Cedar policies: `policy.sanctions_screen_required`, `policy.export_control_region_deny`, `policy.restricted_capability_hold`.
- monitoring: screening match rate, geolocation mismatch, restricted capability attempts, denied region signups, manual review backlog.
- named indicators: sanctions hit, VPN/geovelocity mismatch, dual-use workflow request, restricted export destination, blocked payment.
- early-warning trigger: tenant requests high-capability AI or encryption export into a restricted region.
- control evidence: screening receipt, manual review decision, region deny log, audit-chain event.
- runbook reference: `docs/runbooks/marketplace-listing-takedown.md`.
- incident class: sanctions-export.
- customer communication: account hold notice limited by legal instructions.
- regulator action reference: OFAC/BIS and allied export-control regimes.
- ADR reference: ADR-0013, ADR-0314, ADR-0249, ADR-0304.
- residual risk: Medium due to changing lists and indirect use.
- checkpoint: Restricted capability registry must be current.
- escalation: Legal and compliance before unblocking.

### REG-010 - US state privacy patchwork drift
- risk-ID: REG-010
- name: US state privacy patchwork drift
- category: Compliance-Regulatory
- description: US state privacy rules beyond California may impose distinct notice, opt-out, universal opt-out mechanism, sensitive data, profiling, and appeal requirements.
- threat landscape: State privacy laws continue to proliferate and may diverge from Oyatie's CCPA-centered baseline.
- affected microservices: consent, privacy, regional-pack-us, analytics, ads-analytics, application, DSR.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; rules change on state effective dates and AG guidance.
- owner: council-privacy
- owner role: US State Privacy owner.
- review cadence: Monthly regulator watch.
- status: Open - mitigated by pack overlay model.
- acceptance posture: Reduce; accept only staged state rollout with explicit unsupported states.
- transfer posture: counsel support only.
- microservice mitigations: `regional-pack-api`, `dsr-usecase`, `consent-domain`, `data-boundary-kernel`.
- Cedar policies: `policy.us_state_privacy_overlay`, `policy.profiling_opt_out_required`, `policy.sensitive_data_appeal_right`.
- monitoring: state pack freshness, opt-out signal tests, sensitive data request count, profiling appeal SLA.
- named indicators: new state law effective date, UOOM failure, profiling appeal missed, sensitive data category unmapped.
- early-warning trigger: sales enables a state not covered by the US privacy overlay.
- control evidence: state matrix, opt-out test, DSR proof, data-map row.
- runbook reference: `docs/runbooks/regulatory-change-response.md`.
- incident class: us-privacy-state.
- customer communication: state support matrix.
- regulator action reference: state AG and privacy agency enforcement landscape.
- ADR reference: ADR-0010, ADR-0272, ADR-0008.
- residual risk: Medium because state variance is ongoing.
- checkpoint: US pack review before consumer launch.
- escalation: Privacy Council for unsupported state.

### REG-011 - APAC privacy and financial regulator divergence
- risk-ID: REG-011
- name: APAC privacy and financial regulator divergence
- category: Compliance-Regulatory
- description: Australia, Singapore, Japan, India, and regional financial regulators may require security, privacy, outsourcing, breach, or data-localization controls not captured by a generic APAC pack.
- threat landscape: APAC markets combine privacy statutes, financial outsourcing guidance, critical infrastructure rules, and cross-border transfer controls.
- affected microservices: regional-pack, tenancy, residency, analytics, vertical-fintech, messenger, community, mail, foundry.
- likelihood: Possible (3/5)
- impact: Major (3/5)
- score: 9 Managed
- velocity: Medium; risk rises when a first tenant in a jurisdiction signs.
- owner: regional-packs
- owner role: APAC Pack owner.
- review cadence: Monthly while region is active; quarterly for watch-only regions.
- status: Open - monitored.
- acceptance posture: Accept unsupported region delay; reduce when entering market.
- transfer posture: local counsel and reseller terms transfer limited interpretation risk.
- microservice mitigations: `regional-pack-api`, `platform-residency-kernel`, `intelligence-evidence-regulator-export`.
- Cedar policies: `policy.apac_pack_required`, `policy.financial_outsourcing_evidence_required`, `policy.local_breach_clock`.
- monitoring: regulator publication feed, pack attestation lag, APAC tenant count, counsel memo freshness.
- named indicators: first APAC regulated tenant, outsourcing questionnaire, data-localization request, breach-report clock.
- early-warning trigger: enterprise RFP requires APRA CPS 234, MAS TRM, or similar controls.
- control evidence: regional pack, outsourcing control mapping, breach drill, evidence export.
- runbook reference: `docs/runbooks/regulatory-change-response.md`.
- incident class: apac-regulatory-drift.
- customer communication: regional support matrix and control addendum.
- regulator action reference: APAC privacy and financial regulator guidance by local pack.
- ADR reference: ADR-0010, ADR-0240, ADR-0304.
- residual risk: Medium until each market has explicit pack.
- checkpoint: No APAC regulated customer go-live on generic pack.
- escalation: Regional Pack lead and compliance counsel.

### REG-012 - India DPDPA and RBI localization conflict
- risk-ID: REG-012
- name: India DPDPA and RBI localization conflict
- category: Compliance-Regulatory
- description: Indian privacy, digital personal data, payment, banking, and localization expectations may require controls incompatible with global telemetry, provider routing, or backup posture.
- threat landscape: India market entry often combines privacy notice, consent manager, significant data fiduciary obligations, RBI controls, and local storage expectations.
- affected microservices: regional-pack, residency, payments, marketplace, cloud-storage, analytics, consent.
- likelihood: Possible (3/5)
- impact: Major (3/5)
- score: 9 Managed
- velocity: Medium; triggered by Indian fintech or large enterprise customers.
- owner: regional-packs
- owner role: India Pack owner.
- review cadence: Quarterly until market active; monthly after first customer.
- status: Watch - not accepted for production without pack.
- acceptance posture: Accept delayed India rollout; reduce with dedicated pack.
- transfer posture: local counsel and payment partners transfer limited operational support.
- microservice mitigations: `regional-pack-api`, `platform-residency-kernel`, `marketplace-domain`, `consent-domain`.
- Cedar policies: `policy.in_dpdp_consent_required`, `policy.in_rbi_data_localization_hold`, `policy.india_region_feature_gate`.
- monitoring: India pack status, local storage route, payment processor coverage, consent artifact availability.
- named indicators: India lead in pipeline, payment data flow, cross-border backup, consent language gap.
- early-warning trigger: customer asks to process payment or employee data in India.
- control evidence: India pack, local counsel memo, data-flow map, processor DPA.
- runbook reference: `docs/runbooks/regulatory-change-response.md`.
- incident class: india-regulatory.
- customer communication: India availability and limitations statement.
- regulator action reference: DPDPA and RBI obligations through local counsel.
- ADR reference: ADR-0010, ADR-0049, ADR-0240, ADR-0304.
- residual risk: Medium if market entry is accelerated.
- checkpoint: No India regulated use before pack acceptance.
- escalation: Compliance Council.

### REG-013 - Children, minors, and age-assurance enforcement
- risk-ID: REG-013
- name: Children, minors, and age-assurance enforcement
- category: Compliance-Regulatory
- description: Personal tenant, education, collaboration, marketplace, or content features may collect, infer, or expose minors' data without correct consent, age assurance, default privacy, or safety controls.
- threat landscape: COPPA, state minor safety laws, EU age-appropriate design, CCPA minor consent, and platform moderation expectations increasingly target youth data and content.
- affected microservices: identity, messenger, community, mail, marketplace, content-moderation, ads-analytics, application, privacy.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast after a media report or regulator complaint.
- owner: council-privacy
- owner role: Minor Safety and Privacy owner.
- review cadence: Monthly; weekly before personal or education launch.
- status: Open - reduce before personal tenant expansion.
- acceptance posture: Avoid behavioral advertising to minors; reduce other exposure.
- transfer posture: parental consent processors transfer limited verification support.
- microservice mitigations: `identity-domain`, `messenger-domain`, `community-social-domain`, `mail-domain`, `ads-analytics-domain`, `intelligence-guardrails-content-safety-rule-engine-kernel`.
- Cedar policies: `policy.minor_age_gate`, `policy.minor_no_sale_share`, `policy.guardian_consent_required`, `policy.default_private_minor`.
- monitoring: age assurance failure, minor content report, consent expiration, ad sharing deny count, account recovery risk.
- named indicators: under-16 signal, guardian dispute, minor data in ads, social-media complaint, safety escalation.
- early-warning trigger: personal tenant or education workflow collects minor data.
- control evidence: age-assurance record, guardian consent, content safety decision, data minimization audit.
- runbook reference: `docs/runbooks/breach-notification-council-escalation.md`.
- incident class: minors-privacy-safety.
- customer communication: guardian/school notice as required by law.
- regulator action reference: CCPA minor consent enforcement and youth privacy statutes.
- ADR reference: ADR-0292, ADR-0272, ADR-0301.
- residual risk: Medium because age signals are uncertain.
- checkpoint: Minor-capable flows require privacy council approval.
- escalation: Privacy Council plus Legal.

## Section 5. Technical Risks

### TECH-001 - Per-microservice technical debt accumulation
- risk-ID: TECH-001
- name: Per-microservice technical debt accumulation
- category: Technical
- description: Rapid microservice expansion may create inconsistent architecture, missing tests, duplicate policy logic, stale docs, unowned runbooks, and uneven SLO posture.
- threat landscape: The registry lists core microservices plus a large planned backlog; debt compounds when new services copy incomplete patterns.
- affected microservices: all, especially planned backlog services in `registry/microservices.json`.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; debt is cumulative and then blocks releases.
- owner: council-architecture
- owner role: Architecture Quality owner.
- review cadence: Monthly; weekly for active service waves.
- status: Open - mitigated by flat layout and doc/catalog gates.
- acceptance posture: Accept limited preview debt with owner and deadline; reduce structural drift.
- transfer posture: not transferable.
- microservice mitigations: `check-flat-crates`, `check-doc-catalog`, `check-adr-citation`, `governance-gate-catalog-domain`.
- Cedar policies: `policy.service_promotion_requires_owner`, `policy.preview_debt_expiry`, `policy.capability_publish_evidence_required`.
- monitoring: service coverage, missing runbooks, test coverage, stale ADR refs, unowned catalog rows.
- named indicators: new service without SLO, duplicate Cedar fragment, missing incident response, skipped tests, unresolved TODO in canonical spec.
- early-warning trigger: planned backlog service promotes without the standard service pack.
- control evidence: service catalog row, SLO, runbook, ADR, tests, multispectrum evidence.
- runbook reference: `docs/runbooks/flat-crates-move-pr.md`.
- incident class: architecture-debt.
- customer communication: none unless feature commitments slip.
- regulator action reference: indirect; debt affects security and compliance evidence.
- ADR reference: ADR-0056, ADR-0058, ADR-0131, ADR-0212.
- residual risk: Medium while service count grows quickly.
- checkpoint: service promotion checklist required.
- escalation: Architecture Council when debt would create public claim drift.

### TECH-002 - Dependency vulnerability or supply-chain attack
- risk-ID: TECH-002
- name: Dependency vulnerability or supply-chain attack
- category: Technical
- description: A Rust crate, JS package, container image, GitHub Action, model artifact, plugin, or build tool may introduce malware, CVE exposure, license violations, or compromised provenance.
- threat landscape: Supply-chain attacks target CI, package registries, container base images, signing keys, and transitive dependencies.
- affected microservices: foundry, all build pipelines, plugin runtime, marketplace, cloud, application.
- likelihood: Likely (4/5)
- impact: Severe (4/5)
- score: 16 Severe
- velocity: Fast; malicious package can land in one PR.
- owner: ops-security
- owner role: Supply Chain Security owner.
- review cadence: Weekly vulnerability review; release-gate check on every promotion.
- status: Open - mitigated through SBOM, signing, deny checks.
- acceptance posture: Reduce; accept time-boxed low CVEs only with documented waiver.
- transfer posture: cyber insurance plus vendor indemnity for select processors.
- microservice mitigations: `check-license-policy`, `governance-license-policy-kernel`, `check-sbom-attestation`, `plugin-sandbox`.
- Cedar policies: `policy.dependency_allowlist_required`, `policy.critical_cve_release_block`, `policy.plugin_signature_required`.
- monitoring: cargo deny, cargo audit, Trivy, SBOM diff, SLSA provenance, signing coverage.
- named indicators: critical CVE, unsigned image, forbidden license, dependency typosquat, GitHub Action pin drift.
- early-warning trigger: critical CVE affects runtime or build path.
- control evidence: SBOM, Cosign signature, vulnerability scan, dependency review, waiver record.
- runbook reference: `docs/runbooks/supply-chain-compromise.md`.
- incident class: supply-chain-security.
- customer communication: security advisory and patched version notice when affected.
- regulator action reference: DORA ICT risk, HIPAA security, GDPR security of processing.
- ADR reference: ADR-0039, ADR-0013, ADR-0092, ADR-0280, ADR-0181.
- residual risk: Medium because transitive dependencies and CI remain active targets.
- checkpoint: No release with unwaived critical supply-chain finding.
- escalation: Security Council.

### TECH-003 - Contract version drift across APIs and events
- risk-ID: TECH-003
- name: Contract version drift across APIs and events
- category: Technical
- description: REST, gRPC, AsyncAPI, event schemas, SDKs, and cross-axis contracts may drift between providers and consumers.
- threat landscape: A platform with many services and customer integrations can silently break workflows if contracts are regenerated, versioned, or deprecated inconsistently.
- affected microservices: workflow-engine, foundry, marketplace, messenger, community, mail, analytics, cloud, tasks, SDKs.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; release can break consumers instantly.
- owner: platform-api-sdk
- owner role: API Contract owner.
- review cadence: Weekly contract review during active API work.
- status: Open - mitigated by schema registry and version model.
- acceptance posture: Reduce; accept backward-compatible additive changes under stability tier.
- transfer posture: not transferable.
- microservice mitigations: `schema-registry`, `sdk-release`, `check-openapi-contract-binding`, `eventing-domain`.
- Cedar policies: `policy.contract_breaking_change_review`, `policy.sdk_release_provenance`, `policy.event_schema_version_required`.
- monitoring: contract diff, SDK generation failure, event consumer lag, semver check, compatibility test.
- named indicators: removed field, enum narrowing, unversioned event, SDK snapshot drift, consumer deserialization error.
- early-warning trigger: contract diff marks breaking change without migration runbook.
- control evidence: contract compatibility report, SDK regen evidence, deprecation notice, consumer test.
- runbook reference: `docs/runbooks/contract-breaking-change.md`.
- incident class: contract-drift.
- customer communication: deprecation notice and migration window.
- regulator action reference: indirect; regulated workflows may miss evidence if events break.
- ADR reference: ADR-0037, ADR-0154, ADR-0166, ADR-0258, ADR-0177.
- residual risk: Medium during rapid service generation.
- checkpoint: No breaking contract without versioned migration plan.
- escalation: API Council.

### TECH-004 - Ontology schema evolution breaks typed entity layer
- risk-ID: TECH-004
- name: Ontology schema evolution breaks typed entity layer
- category: Technical
- description: Ontology, knowledge graph, entity types, edge types, action types, and projections may evolve incompatibly with workflows, analytics, search, and customer imports.
- threat landscape: The ontology is central to cross-product semantics; schema mistakes can corrupt meaning even when storage remains valid.
- affected microservices: ontology, workflow-engine, search, analytics, marketplace, tasks, application.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; materializes during migration, import, or projection generation.
- owner: ontology-team
- owner role: Ontology Schema owner.
- review cadence: Monthly; weekly during ontology migration.
- status: Open - mitigated by version/deprecation handshake.
- acceptance posture: Reduce; accept only versioned and reversible migrations.
- transfer posture: not transferable.
- microservice mitigations: `specs/microservices/ontology.json`, `specs/ontology-projection-schema.json`, `registry/kg-audit/index.json`, `ontology-domain`.
- Cedar policies: `policy.ontology_schema_publish_review`, `policy.entity_type_deprecation_window`, `policy.projection_scope_required`.
- monitoring: KG audit, projection compatibility, schema version adoption, migration rollback test, import error rate.
- named indicators: unversioned type change, edge invariant failure, projection mismatch, stale semantic query, import rejection.
- early-warning trigger: workflow references an ontology type removed or renamed without handshake.
- control evidence: schema diff, compatibility migration, KG audit, projection tests.
- runbook reference: `docs/runbooks/og-schema-rollback.md`.
- incident class: ontology-schema.
- customer communication: migration guide for customer-owned entity types.
- regulator action reference: indirect where ontology encodes regulated categories.
- ADR reference: ADR-0006, ADR-0055, ADR-0122, ADR-0257, ADR-0130.
- residual risk: Medium because ontology is shared across many products.
- checkpoint: Schema evolution requires deprecation handshake.
- escalation: Architecture Council when canonical types change.

### TECH-005 - CRDT divergence in collaborative editing
- risk-ID: TECH-005
- name: CRDT divergence in collaborative editing
- category: Technical
- description: Collaborative documents, workflow canvases, tasks, whiteboards, or other shared editors may diverge across clients, replicas, or offline sessions.
- threat landscape: CRDTs reduce conflict but can still diverge through version skew, corrupt updates, permission changes, or unsupported migration.
- affected microservices: workflow-studio, workspace-docs, whiteboard, tasks, drive, messenger, community, mail.
- likelihood: Possible (3/5)
- impact: Major (3/5)
- score: 9 Managed
- velocity: Medium; users notice when state fails to converge.
- owner: axis-workspace
- owner role: Collaborative Editing owner.
- review cadence: Monthly; weekly during editor or CRDT library upgrades.
- status: Open - mitigated by replay and divergence runbooks.
- acceptance posture: Reduce; accept local merge conflicts only with recovery path.
- transfer posture: not transferable.
- microservice mitigations: `workflow-studio-domain`, `workspace-docs-domain`, `tasks-task-store-domain`, CRDT portability trait.
- Cedar policies: `policy.collab_edit_permission_snapshot`, `policy.crdt_replay_operator`, `policy.document_recovery_scope`.
- monitoring: convergence test, update rejection rate, replay divergence, offline merge failure, document recovery count.
- named indicators: client state hash mismatch, unresolved merge, user reports missing edits, replay drift, CRDT version skew.
- early-warning trigger: canary replay diverges on same operation log.
- control evidence: deterministic replay, state hash report, restore artifact, client version matrix.
- runbook reference: `docs/runbooks/workspace/doc-crdt-divergence.md`.
- incident class: collaborative-state.
- customer communication: affected document restoration notice.
- regulator action reference: indirect when records retention or eDiscovery data changes.
- ADR reference: ADR-0142, ADR-0185, ADR-0204, ADR-0252.
- residual risk: Low to Medium depending on offline editing scope.
- checkpoint: CRDT library upgrade requires replay suite.
- escalation: Workspace owner if divergence affects regulated records.

### TECH-006 - Cedar policy explosion and evaluation latency
- risk-ID: TECH-006
- name: Cedar policy explosion and evaluation latency
- category: Technical
- description: Number, composition, and specificity of Cedar policies may grow until evaluation latency, authoring complexity, false positives, and debugging cost degrade user workflows.
- threat landscape: Universal gate doctrine creates pressure to encode every exception in policy, risking unreadable and slow authorization.
- affected microservices: policy, foundry, application, tenancy, workflow-engine, marketplace, analytics.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; performance and maintainability degrade gradually.
- owner: ops-security
- owner role: Cedar Platform owner.
- review cadence: Monthly; weekly during major policy pack launch.
- status: Open - mitigated through fragment discipline and latency dashboard.
- acceptance posture: Reduce; accept complexity only with benchmark and ownership.
- transfer posture: not transferable.
- microservice mitigations: `policy-cedar-domain`, `intelligence-guardrails-cedar-engine`, `check-cedar-fragment-coverage`.
- Cedar policies: `policy.cedar_fragment_registry_required`, `policy.policy_benchmark_required`, `policy.policy_deprecation_window`.
- monitoring: `registry/dashboards/cedar-policy-evaluation-latency.yaml`, fragment count, allow/deny entropy, policy authoring backlog.
- named indicators: p99 eval latency breach, fragment duplication, authoring exceptions, false-positive support tickets, rollback frequency.
- early-warning trigger: policy evaluation p99 breaches customer request budget.
- control evidence: benchmark report, fragment registry, rollback drill, authoring review.
- runbook reference: `docs/runbooks/cedar-policy-rollback.md`.
- incident class: policy-performance.
- customer communication: latency incident update if user-facing.
- regulator action reference: indirect; authz failure can cause privacy breach.
- ADR reference: ADR-0243, ADR-0183, ADR-0191, ADR-0294.
- residual risk: Medium because policy count scales with product scope.
- checkpoint: Policy packs require benchmark and blast-radius diff.
- escalation: Security Council if latency or correctness gates fail.

### TECH-007 - Performance regression in core request paths
- risk-ID: TECH-007
- name: Performance regression in core request paths
- category: Technical
- description: New services, policies, telemetry, AI calls, database queries, or workflow orchestration may degrade p95/p99 latency and throughput.
- threat landscape: Agentic workflows and policy-heavy request paths can add hidden fanout and cold-start costs.
- affected microservices: application, policy, foundry, workflow-engine, analytics, search, messenger, community, mail, cloud.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; regressions often appear under tenant-scale load.
- owner: ops-sre-reliability
- owner role: Performance owner.
- review cadence: Weekly during active launch; monthly steady-state.
- status: Open - mitigated through SLOs and performance evidence.
- acceptance posture: Reduce; accept preview degradation only with customer-visible label.
- transfer posture: not transferable.
- microservice mitigations: `observability-domain`, `shared-hyperscaler-metrics-kernel`, `check-perf-evidence`, service-specific benchmark sets.
- Cedar policies: `policy.perf_gate_required_for_promotion`, `policy.slo_burn_release_block`, `policy.expensive_query_budget`.
- monitoring: golden signals, SLO burn, policy eval latency, DB p99, queue lag, provider latency.
- named indicators: p99 regression, error budget burn, cold start spike, N+1 query, provider roundtrip rise.
- early-warning trigger: canary p99 exceeds release threshold for two windows.
- control evidence: benchmark, load test, canary report, rollback plan.
- runbook reference: `docs/runbooks/error-budget-exhaustion.md`.
- incident class: performance-regression.
- customer communication: incident status if SLO breach is external.
- regulator action reference: DORA resilience for financial tenants.
- ADR reference: ADR-0062, ADR-0180, ADR-0114, ADR-0040.
- residual risk: Medium during rapid feature addition.
- checkpoint: Performance evidence required for stable promotion.
- escalation: Release manager blocks rollout.

### TECH-008 - Capacity exhaustion and quota bypass
- risk-ID: TECH-008
- name: Capacity exhaustion and quota bypass
- category: Technical
- description: Tenants, providers, cells, queues, vector stores, GPUs, storage, or external APIs may exhaust capacity or bypass quota controls.
- threat landscape: AI workloads, search indexing, evidence storage, analytics queries, and provider quotas have spiky resource profiles.
- affected microservices: cloud-capacity, foundry, analytics, search, storage, marketplace, workflow-engine.
- likelihood: Likely (4/5)
- impact: Severe (4/5)
- score: 16 Severe
- velocity: Fast when a large tenant, runaway workflow, or provider failure changes load.
- owner: ops-finops
- owner role: Capacity and Cost owner with SRE co-owner.
- review cadence: Weekly during launch; monthly afterward.
- status: Open - mitigated by per-tenant quotas and autoscaling.
- acceptance posture: Reduce; accept graceful throttling rather than uncontrolled spend.
- transfer posture: cloud credits and provider SLAs only partly transfer cost.
- microservice mitigations: `cloud-capacity-domain`, `shared-tenant-quota-kernel`, `intelligence-providers-router-domain`, `analytics-domain`.
- Cedar policies: `policy.tenant_quota_enforced`, `policy.provider_quota_fallback`, `policy.capacity_emergency_scale_operator`.
- monitoring: quota utilization, provider rate limits, queue lag, vector DB capacity, GPU pool saturation, cost anomaly.
- named indicators: 80 percent quota sustained, provider 429 spike, queue delay, cost anomaly, cell saturation.
- early-warning trigger: tenant consumption breaks forecast by more than threshold.
- control evidence: quota config, autoscaler event, throttling decision, cost report.
- runbook reference: `docs/runbooks/capacity-scaling-emergency.md`.
- incident class: capacity-exhaustion.
- customer communication: throttling or capacity expansion notice.
- regulator action reference: DORA resilience when financial tenant service is affected.
- ADR reference: ADR-0155, ADR-0178, ADR-0198, ADR-0199.
- residual risk: Medium because AI and analytics workloads are bursty.
- checkpoint: Tenant quota pack required before onboarding large tenant.
- escalation: FinOps and SRE.

### TECH-009 - Event schema drift and outbox loss
- risk-ID: TECH-009
- name: Event schema drift and outbox loss
- category: Technical
- description: Events may be lost, duplicated, malformed, or version-skewed across outbox, brokers, audit-chain, workflow state, and external subscribers.
- threat landscape: Event-driven systems trade central coupling for schema/version and replay discipline.
- affected microservices: eventing, audit-chain, workflow-engine, foundry, marketplace, analytics, messenger, community, mail.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast when consumer deployments lag producers.
- owner: platform-api-sdk
- owner role: Event Schema owner.
- review cadence: Monthly; weekly during event migration.
- status: Open - mitigated by schema registry and topic rollback.
- acceptance posture: Reduce; accept duplicate events only with idempotency proof.
- transfer posture: not transferable.
- microservice mitigations: `eventing-domain`, `schema-registry`, `audit-chain-domain`, `workflow-engine-domain`.
- Cedar policies: `policy.event_schema_publish_review`, `policy.outbox_replay_operator`, `policy.audit_event_required`.
- monitoring: broker lag, schema compatibility, consumer error, outbox gap, audit emission lag.
- named indicators: deserialization error, unknown event version, outbox sequence gap, replay divergence, consumer DLQ.
- early-warning trigger: schema registry accepts producer version not supported by critical consumers.
- control evidence: schema compatibility report, replay test, topic rollback drill.
- runbook reference: `docs/runbooks/topic-schema-rollback.md`.
- incident class: event-schema.
- customer communication: integration notice if external subscribers affected.
- regulator action reference: audit and incident evidence implications.
- ADR reference: ADR-0154, ADR-0153, ADR-0166, ADR-0149.
- residual risk: Medium until all critical events have compatibility tests.
- checkpoint: Event schema changes require consumer matrix.
- escalation: API Council.

### TECH-010 - AI model lifecycle and eval drift
- risk-ID: TECH-010
- name: AI model lifecycle and eval drift
- category: Technical
- description: Model upgrades, adapter changes, prompt changes, fine-tunes, LoRA adapters, provider outages, or eval-set gaps may degrade correctness, safety, cost, or compliance.
- threat landscape: AI providers change behavior; in-house and external models need repeatable promotion gates.
- affected microservices: foundry, guardrails, eval, providers, evidence, workflow-engine, tasks.
- likelihood: Likely (4/5)
- impact: Severe (4/5)
- score: 16 Severe
- velocity: Fast; provider model changes can occur with little warning.
- owner: axis-foundry
- owner role: Model Lifecycle owner.
- review cadence: Weekly during model changes; monthly otherwise.
- status: Open - mitigated through eval replay and model cutover runbooks.
- acceptance posture: Reduce; accept residual nondeterminism only under monitored confidence bands.
- transfer posture: provider terms transfer limited SLA risk only.
- microservice mitigations: `intelligence-eval-eval-runner-domain`, `intelligence-providers-router-domain`, `intelligence-guardrails-output-validator-kernel`, `intelligence-evidence-capability-invocation-recorder-kernel`.
- Cedar policies: `policy.model_cutover_eval_required`, `policy.provider_fallback_allowed`, `policy.high_risk_model_change_hold`.
- monitoring: eval pass rate, parity trend, guardrail false positive, provider error, token cost, jailbreak attempts.
- named indicators: eval regression, output drift, cost spike, provider 5xx, fairness degradation, high-risk refusal miss.
- early-warning trigger: eval parity drops below threshold on regulated capability.
- control evidence: eval report, replay artifact, model card, rollback test, audit-chain evidence.
- runbook reference: `microservices/intelligence/runbooks/eval-eval-set-rollback.md`.
- incident class: ai-model-lifecycle.
- customer communication: capability-specific release note and rollback notice if visible.
- regulator action reference: EU AI Act, EDPB AI model opinion, sector fairness expectations.
- ADR reference: ADR-0026, ADR-0308, ADR-0139, ADR-0255.
- residual risk: Medium because models are probabilistic and provider-controlled.
- checkpoint: Model promotion blocked without eval evidence.
- escalation: Foundry Council.

### TECH-011 - Observability blind spots and sampling misconfiguration
- risk-ID: TECH-011
- name: Observability blind spots and sampling misconfiguration
- category: Technical
- description: Logs, metrics, traces, audit events, and exemplars may be missing, oversampled, undersampled, privacy-leaking, or disconnected from tenant, cell, and capability context.
- threat landscape: Without observability, incidents, compliance evidence, and SLO governance become speculative; with bad observability, privacy risk increases.
- affected microservices: observability, audit-chain, ops, all product services.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; blind spots matter most during incidents.
- owner: ops-sre-reliability
- owner role: Observability owner with council-privacy reviewer.
- review cadence: Monthly; weekly during new service launch.
- status: Open - mitigated by OTel and emission contract.
- acceptance posture: Reduce; accept only documented sampling gaps.
- transfer posture: not transferable.
- microservice mitigations: `observability-domain`, `shared-hyperscaler-metrics-adapter-prometheus`, `platform-audit-chain-app`.
- Cedar policies: `policy.telemetry_data_class_filter`, `policy.trace_sampling_profile_required`, `policy.incident_observability_override`.
- monitoring: trace coverage, log redaction, metric cardinality, audit emission lag, dashboard freshness.
- named indicators: missing tenant labels, high-cardinality explosion, PII in logs, trace sampling zero for critical path, stale dashboard.
- early-warning trigger: new microservice lacks golden signals or tenant labels.
- control evidence: dashboard row, OTel config, redaction test, trace sample, audit emission.
- runbook reference: `docs/runbooks/cross-plane-call-introduction.md`.
- incident class: observability-gap.
- customer communication: none unless incident evidence is affected.
- regulator action reference: DORA, HIPAA audit controls, GDPR accountability.
- ADR reference: ADR-0042, ADR-0186, ADR-0210, ADR-0263.
- residual risk: Medium during rapid service addition.
- checkpoint: Golden signals before stable promotion.
- escalation: SRE lead.

### TECH-012 - Kubernetes autoscaler and cell scheduling failure
- risk-ID: TECH-012
- name: Kubernetes autoscaler and cell scheduling failure
- category: Technical
- description: Karpenter, node pools, pod disruption budgets, topology spread, network policy, or resource requests may schedule workloads into unavailable, insecure, or overloaded cells.
- threat landscape: Kubernetes-first server workloads require strong scheduling, capacity, and network isolation discipline.
- affected microservices: cloud-compute, cloud-cell, foundry runtime, workflow-engine, analytics, messenger, community, mail, marketplace.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast during scale-up or node failure.
- owner: axis-cloud
- owner role: Kubernetes Platform owner.
- review cadence: Monthly; weekly during cluster or autoscaler changes.
- status: Open - mitigated by node autoscaling and cell topology maps.
- acceptance posture: Reduce; accept degraded scheduling only in preview.
- transfer posture: cloud provider SLA transfers limited node failure cost.
- microservice mitigations: `cloud-compute-domain`, `cloud-cell-app`, `shared-hyperscaler-metrics-kernel`, `intelligence-runtime-invocation-orchestrator-domain`.
- Cedar policies: `policy.workload_cell_affinity_required`, `policy.regulated_workload_node_selector`, `policy.k8s_emergency_drain_operator`.
- monitoring: pending pods, node pressure, autoscaler failures, topology skew, network policy drops, regulated workload placement.
- named indicators: pod unschedulable, cross-cell placement, node quota failure, PDB deadlock, Karpenter provisioning delay.
- early-warning trigger: regulated workload schedules outside approved node pool.
- control evidence: scheduling simulation, node pool policy, chaos drill, placement audit.
- runbook reference: `microservices/intelligence/runbooks/supervisor-kubernetes-operator-restart.md`.
- incident class: k8s-platform.
- customer communication: degraded service notice if external SLO breached.
- regulator action reference: DORA and sovereign placement implications.
- ADR reference: ADR-0198, ADR-0121, ADR-0148, ADR-0253.
- residual risk: Medium because scheduling and capacity interact dynamically.
- checkpoint: topology and PDB tests before stable launch.
- escalation: Cloud Platform owner.

### TECH-013 - Database hot partition and RPO/RTO failure
- risk-ID: TECH-013
- name: Database hot partition and RPO/RTO failure
- category: Technical
- description: Tenant growth, workflow hotspots, analytics joins, time-series cardinality, or bad partition keys may overload storage and undermine recovery objectives.
- threat landscape: Multi-tenant databases fail through hotspots, slow migrations, replica lag, and backup inconsistency before full outage.
- affected microservices: database tier, analytics, workflow-engine, tasks, marketplace, audit-chain, search, tenancy.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; can become fast during high-volume tenant event.
- owner: ops-sre-reliability
- owner role: Data Platform owner.
- review cadence: Monthly; weekly before high-volume tenant migration.
- status: Open - mitigated by database tier strategy and quotas.
- acceptance posture: Reduce; accept tiered RPO/RTO by contract.
- transfer posture: managed database SLA transfers limited infrastructure risk.
- microservice mitigations: `cloud-data-domain`, `analytics-domain`, `audit-chain-domain`, `shared-tenant-quota-kernel`.
- Cedar policies: `policy.database_migration_approval`, `policy.tenant_hot_partition_throttle`, `policy.rpo_rto_tier_required`.
- monitoring: replica lag, partition skew, query p99, migration duration, backup verification, RPO lag.
- named indicators: hot tenant, index bloat, replica delay, long migration lock, failed PITR test.
- early-warning trigger: top tenant consumes disproportionate write or query capacity.
- control evidence: capacity model, partition report, restore test, migration dry-run.
- runbook reference: `docs/runbooks/analytics-warehouse-reconciliation.md`.
- incident class: data-platform-resilience.
- customer communication: maintenance or degraded analytics notice.
- regulator action reference: DORA resilience and HIPAA contingency where applicable.
- ADR reference: ADR-0045, ADR-0172, ADR-0193, ADR-0194, ADR-0152.
- residual risk: Medium as tenant data volume grows.
- checkpoint: High-volume tenant migration requires load test.
- escalation: Data Platform owner and SRE.

## Section 6. Customer and Commercial Risks

### COM-001 - Oyatie creates customer lock-in perception
- risk-ID: COM-001
- name: Oyatie creates customer lock-in perception
- category: Customer-Commercial
- description: Customers may believe Oyatie makes it too hard to export workflows, evidence, data, policies, or integrations despite the open integration and migration-out doctrine.
- threat landscape: Enterprise buyers are sensitive to lock-in after platform-suite consolidation and AI-agent dependency.
- affected microservices: data export, DSR, workflow-engine, ontology, marketplace, application, trust-portal.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; usually appears during procurement and renewal.
- owner: gtm-sales-se
- owner role: Customer Trust Sales Engineering owner.
- review cadence: Monthly; per enterprise procurement.
- status: Open - mitigated by migration-out and portability formats.
- acceptance posture: Reduce; accept platform dependency only when exit is evidenced.
- transfer posture: contractual exit clauses transfer some customer concern.
- microservice mitigations: `dsr-usecase`, `workflow-engine-domain`, `ontology-domain`, `intelligence-evidence-evidence-pack-builder-api`.
- Cedar policies: `policy.tenant_export_authorized`, `policy.portability_export_scope`, `policy.migration_out_evidence_required`.
- monitoring: export success rate, migration-out requests, customer security questionnaire objections, data-portability SLA.
- named indicators: RFP lock-in objection, failed export, custom format gap, competitor FUD, renewal legal redline.
- early-warning trigger: customer asks for full tenant export before contract signature.
- control evidence: export demo, schema mapping, data-portability receipt, migration runbook.
- runbook reference: `docs/runbooks/dsr-cascade-proof-of-erasure.md`.
- incident class: commercial-portability.
- customer communication: migration-out policy and export evidence.
- regulator action reference: GDPR Article 20 portability and consumer deletion/export regimes.
- ADR reference: ADR-0216, ADR-0276, ADR-0173, ADR-0001.
- residual risk: Medium because deep workflow integration is valuable and sticky.
- checkpoint: Enterprise contract includes exit evidence attachment.
- escalation: GTM and Architecture Council for custom exit commitments.

### COM-002 - Competitors claim Oyatie is lock-in or closed AI infrastructure
- risk-ID: COM-002
- name: Competitors claim Oyatie is lock-in or closed AI infrastructure
- category: Customer-Commercial
- description: Incumbents and AI vendors may frame Oyatie as proprietary lock-in, compliance theater, or unnecessary governance overhead.
- threat landscape: Competitive narratives can slow RFPs, board approvals, analyst coverage, and partner adoption.
- affected microservices: trust-portal, evidence, workflow-engine, marketplace, foundry, developer portal.
- likelihood: Possible (3/5)
- impact: Major (3/5)
- score: 9 Managed
- velocity: Medium; risk spikes during competitive sales cycles.
- owner: gtm-sales-se
- owner role: Competitive Strategy owner.
- review cadence: Monthly; per major competitive deal.
- status: Open - monitored.
- acceptance posture: Accept competitive narrative as market reality; reduce with evidence.
- transfer posture: not transferable.
- microservice mitigations: `intelligence-evidence-evidence-pack-builder-api`, `developer-portal`, `marketplace-domain`, migration export tooling.
- Cedar policies: `policy.trust_portal_public_evidence`, `policy.customer_export_demo_allowed`, `policy.competitive_claim_review_required`.
- monitoring: RFP objection tracker, win/loss notes, trust portal views, export demo pass rate.
- named indicators: competitor document names lock-in, deal delay, analyst question, procurement objection.
- early-warning trigger: three active deals repeat the same lock-in concern.
- control evidence: portability demo, public ADR reference, customer migration story, export artifact.
- runbook reference: `docs/runbooks/partner-contract-renewal.md`.
- incident class: commercial-narrative.
- customer communication: evidence-backed portability and open integration response.
- regulator action reference: data portability and interoperability expectations.
- ADR reference: ADR-0173, ADR-0216, ADR-0213, ADR-0314.
- residual risk: Low to Medium.
- checkpoint: Competitive FAQ kept current.
- escalation: GTM Council.

### COM-003 - Churn from failed migration or onboarding
- risk-ID: COM-003
- name: Churn from failed migration or onboarding
- category: Customer-Commercial
- description: Customers may churn if migration, onboarding, identity federation, data import, workflow mapping, or change management fails to deliver promised value.
- threat landscape: Enterprise SaaS adoption fails when migration cost and organizational friction exceed early value.
- affected microservices: application, identity, connectors, workflow-engine, ontology, DSR, trust-portal.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; manifests over onboarding milestones.
- owner: gtm-sales-se
- owner role: Customer Success Engineering owner.
- review cadence: Weekly for active design partners; monthly portfolio review.
- status: Open - mitigated by onboarding runbooks and proof-of-value gates.
- acceptance posture: Reduce; accept staged rollout rather than big-bang migration.
- transfer posture: implementation partners transfer delivery load only.
- microservice mitigations: `connector-*`, `identity-domain`, `workflow-engine-domain`, `ontology-domain`.
- Cedar policies: `policy.tenant_onboarding_stage_gate`, `policy.connector_scope_limited`, `policy.migration_data_validation_required`.
- monitoring: onboarding milestone age, import error rate, identity federation failure, active user adoption, value metric.
- named indicators: stalled onboarding, repeated connector failure, customer admin inactivity, migration rollback, low weekly active use.
- early-warning trigger: design partner misses two onboarding milestones.
- control evidence: onboarding checklist, data import reconciliation, identity federation test, acceptance report.
- runbook reference: `docs/runbooks/design-partner-onboarding.md`.
- incident class: customer-onboarding.
- customer communication: recovery plan and milestone reset.
- regulator action reference: indirect when regulated data import fails.
- ADR reference: ADR-0187, ADR-0190, ADR-0215, ADR-0217.
- residual risk: Medium because enterprise change management is hard.
- checkpoint: Proof-of-value criteria before expansion.
- escalation: Customer Success Council.

### COM-004 - Big-tenant concentration
- risk-ID: COM-004
- name: Big-tenant concentration
- category: Customer-Commercial
- description: A small number of large tenants may dominate revenue, roadmap pressure, support load, and architecture exceptions.
- threat landscape: Enterprise platform companies can become custom-service organizations when large tenants dictate features, timelines, and exceptions.
- affected microservices: all customer-facing services, especially workflow-engine, cloud, messenger, community, mail, analytics, marketplace.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; rises with first major logos.
- owner: gtm-sales-se
- owner role: Revenue Concentration owner with council-architecture review.
- review cadence: Monthly.
- status: Open - managed.
- acceptance posture: Accept early concentration within board-approved thresholds; reduce custom forks.
- transfer posture: credit insurance or payment terms transfer limited cash risk.
- microservice mitigations: productized pack overlays, capability tiers, marketplace extension model, tenant quota controls.
- Cedar policies: `policy.customer_exception_expiry`, `policy.custom_feature_pack_gate`, `policy.big_tenant_quota_review`.
- monitoring: revenue concentration, custom exception count, support hours by tenant, roadmap override requests.
- named indicators: one tenant over threshold, customer-specific code request, support load concentration, pricing concessions.
- early-warning trigger: requested exception violates canonical-base doctrine.
- control evidence: exception ledger, productization plan, board approval, sunset date.
- runbook reference: `docs/runbooks/tenant-escalation-management.md`.
- incident class: customer-concentration.
- customer communication: productized roadmap and exception boundary.
- regulator action reference: not direct.
- ADR reference: ADR-0001, ADR-0064, ADR-0316, ADR-0242.
- residual risk: Medium in early revenue stage.
- checkpoint: No per-customer fork without Architecture Council approval.
- escalation: Board risk review if concentration threshold exceeded.

### COM-005 - GTM motion stalls under horizontal plus vertical complexity
- risk-ID: COM-005
- name: GTM motion stalls under horizontal plus vertical complexity
- category: Customer-Commercial
- description: The platform may be too broad for buyers to understand, delaying repeatable positioning, sales qualification, onboarding, and reference customer creation.
- threat landscape: Oyatie spans workspace, cloud, workflow, marketplace, compliance, AI, and vertical packs; breadth can confuse ICP and sales sequence.
- affected microservices: application, workflow-engine, foundry, vertical-corporate, healthcare, marketplace, trust-portal.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; seen in pipeline conversion and sales-cycle length.
- owner: gtm-sales-se
- owner role: GTM Strategy owner.
- review cadence: Monthly pipeline risk review.
- status: Open - mitigated by first-deliverable sequencing.
- acceptance posture: Reduce; accept narrower wedge messaging.
- transfer posture: channel partners can help distribution but not narrative clarity.
- microservice mitigations: Tenant RBAC view plus Tenant RBAC view first-deliverable package, trust portal evidence packs, vertical templates.
- Cedar policies: `policy.gtm_claim_evidence_required`, `policy.vertical_pack_claim_gate`, `policy.demo_tenant_data_boundary`.
- monitoring: sales cycle days, proof-of-value pass rate, demo abandonment, buyer persona confusion, content engagement.
- named indicators: many demos no pilot, RFP scope creep, buyer cannot describe product, delayed champion.
- early-warning trigger: more than one quarter without repeatable design-partner conversion.
- control evidence: ICP doc, demo script, proof-of-value checklist, design partner feedback.
- runbook reference: `docs/runbooks/design-partner-feedback-session.md`.
- incident class: gtm-positioning.
- customer communication: clearer wedge narrative and use-case-specific evidence.
- regulator action reference: not direct.
- ADR reference: ADR-0217, ADR-0061, ADR-0321, ADR-0316.
- residual risk: Medium while product breadth remains high.
- checkpoint: first-deliverable packaging before broad fanout.
- escalation: GTM Council.

### COM-006 - Pricing pressure and cloud-cost pass-through
- risk-ID: COM-006
- name: Pricing pressure and cloud-cost pass-through
- category: Customer-Commercial
- description: Customers may resist prices required to cover AI provider cost, compliance overhead, dedicated cells, sovereign hosting, evidence retention, and support.
- threat landscape: AI feature commoditization and hyperscaler credits create pressure while Oyatie carries higher governance costs.
- affected microservices: cloud-billing, metering, foundry providers, analytics, storage, marketplace, finops.
- likelihood: Likely (4/5)
- impact: Major (3/5)
- score: 12 Elevated
- velocity: Medium; worsens with provider price changes or large tenant usage.
- owner: ops-finops
- owner role: Pricing and Margin owner with GTM co-owner.
- review cadence: Monthly FinOps and pricing review.
- status: Open - managed.
- acceptance posture: Accept competitive pressure; reduce through metering and tiering.
- transfer posture: provider commitments and reserved capacity transfer some cost volatility.
- microservice mitigations: `cloud-billing-domain`, `metering-domain`, `cloud-finops-api`, `intelligence-providers-router-domain`.
- Cedar policies: `policy.discount_approval_dual_control`, `policy.cost_budget_by_tenant`, `policy.provider_cost_ceiling`.
- monitoring: gross margin, cost per tenant, provider token cost, storage retention cost, discount depth, overage disputes.
- named indicators: margin below threshold, high discount rate, provider price hike, tenant overage complaint, quota bypass.
- early-warning trigger: customer usage exceeds plan economics for two billing cycles.
- control evidence: cost attribution, price pack, tenant budget, discount approval.
- runbook reference: `docs/runbooks/cost-anomaly-response.md`.
- incident class: pricing-margin.
- customer communication: transparent usage and tier adjustment.
- regulator action reference: not direct.
- ADR reference: ADR-0174, ADR-0199, ADR-0178, ADR-0316.
- residual risk: Medium.
- checkpoint: No unlimited AI tier without cost ceiling.
- escalation: FinOps and GTM leadership.

## Section 7. Reputational and Social Risks

### REP-001 - News cycle after AI autonomy incident
- risk-ID: REP-001
- name: News cycle after AI autonomy incident
- category: Reputational-Social
- description: A customer-visible agentic action could be framed as rogue AI, unsafe automation, discriminatory decisioning, or unaccountable workplace control.
- threat landscape: AI incidents spread quickly through press, social platforms, regulators, and enterprise security communities.
- affected microservices: foundry, workflow-engine, tasks, messenger, community, mail, guardrails, audit-chain, trust-portal.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast; narrative can outrun facts.
- owner: comms-trust
- owner role: Trust Communications owner with Foundry owner.
- review cadence: Monthly tabletop; immediate after autonomy incident.
- status: Open - mitigated by evidence and kill switch.
- acceptance posture: Reduce; accept residual media risk only with evidence-ready posture.
- transfer posture: crisis communications retainer supports response only.
- microservice mitigations: `intelligence-supervisor-kill-switch-circuit-breaker-domain`, `intelligence-evidence-evidence-pack-builder-api`, `audit-chain-domain`.
- Cedar policies: `policy.autonomy_incident_freeze`, `policy.public_statement_review`, `policy.customer_fact_pack_access`.
- monitoring: autonomy violation dashboard, social listening, trust portal incident views, customer support spike.
- named indicators: viral post, press inquiry, regulator question, customer board escalation, autonomy incident.
- early-warning trigger: public complaint mentions Oyatie AI made a harmful decision.
- control evidence: decision trace, human approval record, rollback log, customer fact pack.
- runbook reference: `docs/runbooks/trust-portal-publish-procedure.md`.
- incident class: public-ai-trust.
- customer communication: evidence-first incident narrative.
- regulator action reference: EU AI Act transparency and high-risk AI obligations.
- ADR reference: ADR-0022, ADR-0305, ADR-0139, ADR-0293.
- residual risk: Medium because public narratives are hard to control.
- checkpoint: public statement cannot exceed evidence.
- escalation: SEV-1 communications bridge.

### REP-002 - Social-media privacy incident
- risk-ID: REP-002
- name: Social-media privacy incident
- category: Reputational-Social
- description: A privacy complaint, screenshot, DSR dispute, cross-tenant access allegation, or support mishandling may spread publicly before investigation completes.
- threat landscape: Privacy claims are reputationally damaging even when facts are incomplete.
- affected microservices: privacy, DSR, support, messenger, community, mail, workspace, trust-portal, audit-chain.
- likelihood: Possible (3/5)
- impact: Major (3/5)
- score: 9 Managed
- velocity: Fast.
- owner: comms-trust
- owner role: Social Incident owner with council-privacy.
- review cadence: Monthly tabletop.
- status: Open - monitored.
- acceptance posture: Reduce through fast evidence gathering.
- transfer posture: PR retainer and insurance help response cost only.
- microservice mitigations: `dsr-domain`, `intelligence-evidence-evidence-pack-builder-api`, `audit-chain-domain`, support access controls.
- Cedar policies: `policy.support_access_break_glass`, `policy.privacy_incident_fact_pack`, `policy.public_response_approval`.
- monitoring: social sentiment, support escalations, DSR SLA, privacy complaint rate, trust portal hits.
- named indicators: public complaint, journalist inquiry, DSR SLA miss, screenshot with personal data, support access anomaly.
- early-warning trigger: social post alleges data leak or ignored deletion request.
- control evidence: DSR trace, access log, audit-chain event, support transcript.
- runbook reference: `docs/runbooks/breach-notification-council-escalation.md`.
- incident class: public-privacy.
- customer communication: fact-specific privacy response.
- regulator action reference: GDPR, CCPA, PIPA complaint pathways.
- ADR reference: ADR-0008, ADR-0209, ADR-0272.
- residual risk: Low to Medium.
- checkpoint: privacy fact pack within first response window.
- escalation: Privacy Council.

### REP-003 - ESG scope-3 supplier issue
- risk-ID: REP-003
- name: ESG scope-3 supplier issue
- category: Reputational-Social
- description: Suppliers, hyperscalers, data centers, AI providers, hardware vendors, or subcontractors may create emissions, labor, human-rights, or sustainability controversy attributed to Oyatie.
- threat landscape: Enterprise buyers increasingly assess supplier sustainability, carbon reporting, and responsible sourcing across SaaS vendors.
- affected microservices: vendor ledger, cloud, procurement, marketplace, trust-portal, analytics.
- likelihood: Possible (3/5)
- impact: Moderate (2/5)
- score: 6 Managed
- velocity: Medium; risk rises with public sustainability claims.
- owner: gtm-partnerships
- owner role: Vendor Governance owner.
- review cadence: Quarterly vendor review; monthly for high-risk supplier.
- status: Watch - managed through vendor ledger.
- acceptance posture: Accept residual supply-chain ESG exposure; reduce with transparency.
- transfer posture: supplier contractual warranties and audit rights transfer some risk.
- microservice mitigations: vendor partner ledger, supplier due diligence workflow, analytics reporting, trust portal disclosures.
- Cedar policies: `policy.supplier_esg_review_required`, `policy.scope3_claim_evidence`, `policy.vendor_contract_expiry_block`.
- monitoring: vendor review age, scope-3 data completeness, supplier controversy alert, contract audit rights.
- named indicators: supplier press controversy, missing carbon data, customer ESG questionnaire gap, contract lacking audit clause.
- early-warning trigger: customer RFP requires supplier emissions evidence not available.
- control evidence: supplier attestation, contract clause, emissions report, due diligence packet.
- runbook reference: `docs/runbooks/partner-contract-renewal.md`.
- incident class: vendor-esg.
- customer communication: supplier transparency response.
- regulator action reference: ESG disclosure rules when applicable to customers.
- ADR reference: ADR-0014, ADR-0173, ADR-0254.
- residual risk: Medium for opaque supplier chains.
- checkpoint: no ESG claim without evidence.
- escalation: Vendor Governance owner.

### REP-004 - AI-generated content and IP dispute
- risk-ID: REP-004
- name: AI-generated content and IP dispute
- category: Reputational-Social
- description: Generated documents, workflow text, code, images, summaries, or recommendations may be accused of copyright infringement, hallucination, plagiarism, or improper training-data reuse.
- threat landscape: AI IP disputes are active across jurisdictions and customers expect indemnity, provenance, and content controls.
- affected microservices: foundry, workflow-studio, docs, content generation, marketplace, messenger, community, mail.
- likelihood: Possible (3/5)
- impact: Major (3/5)
- score: 9 Managed
- velocity: Medium; disputes appear after generated content is published or relied upon.
- owner: comms-trust
- owner role: AI Content Risk owner with legal.
- review cadence: Monthly; per major content-generating capability.
- status: Open - mitigated by provenance and disclaimers.
- acceptance posture: Reduce; accept bounded output risk with review controls.
- transfer posture: AI provider indemnity and E&O transfer partial risk.
- microservice mitigations: `intelligence-guardrails-output-validator-kernel`, `intelligence-evidence-capability-invocation-recorder-kernel`, `marketplace-domain`.
- Cedar policies: `policy.generated_content_review_required`, `policy.ip_sensitive_output_hold`, `policy.model_provider_terms_checked`.
- monitoring: content dispute tickets, generated output similarity alerts, provider indemnity coverage, customer edits.
- named indicators: takedown request, IP complaint, hallucinated citation, output similarity hit, customer legal escalation.
- early-warning trigger: generated content is used externally without review in regulated or public context.
- control evidence: prompt/output record, provider terms, review approval, provenance note.
- runbook reference: `docs/runbooks/foundry/guardrails-rule-store-restore.md`.
- incident class: ai-content-ip.
- customer communication: correction, takedown, or provenance response.
- regulator action reference: EU AI Act GPAI transparency and copyright-related obligations.
- ADR reference: ADR-0026, ADR-0255, ADR-0308.
- residual risk: Medium.
- checkpoint: Public generated artifacts require provenance and review path.
- escalation: Legal and Trust Communications.

### REP-005 - Content moderation failure
- risk-ID: REP-005
- name: Content moderation failure
- category: Reputational-Social
- description: Messenger, community, mail, marketplace, comments, files, or generated content may host abuse, illegal content, harassment, misinformation, or unsafe material without timely action.
- threat landscape: Multi-tenant collaboration and marketplace surfaces can inherit platform moderation obligations and public trust risk.
- affected microservices: messenger, community, mail, marketplace, foundry guardrails, storage, trust-portal.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Fast; harmful content can spread immediately.
- owner: comms-trust
- owner role: Trust and Safety owner.
- review cadence: Monthly; weekly before community or public marketplace launch.
- status: Open - mitigated by moderation and takedown controls.
- acceptance posture: Reduce; accept private enterprise content boundary only where contractually controlled.
- transfer posture: moderation vendors and insurance transfer some operational cost only.
- microservice mitigations: `intelligence-guardrails-content-safety-rule-engine-kernel`, `marketplace-domain`, `community-social-domain`, `messenger-domain`, `mail-domain`, storage abuse controls.
- Cedar policies: `policy.content_takedown_authorized`, `policy.abuse_report_triage`, `policy.public_marketplace_listing_review`.
- monitoring: abuse report queue, takedown SLA, classifier false negatives, marketplace listing flags, repeat offender count.
- named indicators: illegal content report, customer harassment complaint, public listing abuse, law enforcement request.
- early-warning trigger: abuse queue breaches SLA or public content is reported externally.
- control evidence: moderation decision, takedown log, appeal record, audit-chain event.
- runbook reference: `docs/runbooks/marketplace-listing-takedown.md`.
- incident class: content-moderation.
- customer communication: takedown and appeal notices.
- regulator action reference: platform safety and illegal-content regimes by jurisdiction.
- ADR reference: ADR-0314, ADR-0249, ADR-0301, ADR-0292.
- residual risk: Medium for public/social surfaces.
- checkpoint: public content surfaces require moderation readiness.
- escalation: Trust and Safety Council.

### REP-006 - Labor, works council, and algorithmic management backlash
- risk-ID: REP-006
- name: Labor, works council, and algorithmic management backlash
- category: Reputational-Social
- description: Workforce analytics, task assignment, productivity scoring, screening, scheduling, or monitoring may be perceived as surveillance or unfair algorithmic management.
- threat landscape: Worker privacy, works councils, labor regulators, civil-rights groups, and media scrutiny focus on automated workplace decisioning.
- affected microservices: tasks, analytics, workflow-engine, workplace-integration, messenger, community, mail, foundry, regional-pack.
- likelihood: Possible (3/5)
- impact: Severe (4/5)
- score: 12 Elevated
- velocity: Medium; can become fast after employee complaint.
- owner: council-privacy
- owner role: Workplace Privacy owner with comms-trust.
- review cadence: Monthly; weekly before workforce analytics or hiring features.
- status: Open - mitigated by transparency, works-council pack, and high-risk AI gates.
- acceptance posture: Avoid emotion recognition and prohibited profiling; reduce other workforce AI.
- transfer posture: customer terms do not transfer platform design responsibility.
- microservice mitigations: `tasks-domain`, `analytics-domain`, `workplace-integration-policy`, `intelligence-guardrails-output-validator-kernel`.
- Cedar policies: `policy.workplace_monitoring_notice`, `policy.worker_analytics_minimization`, `policy.hiring_ai_human_review_required`.
- monitoring: workforce feature usage, appeal rate, fairness audit, works-council approval, denied prohibited-practice requests.
- named indicators: employee complaint, works-council objection, fairness metric drift, high-risk AI classification, press inquiry.
- early-warning trigger: customer asks for scoring, ranking, emotion inference, or hidden monitoring.
- control evidence: DPIA, worker notice, fairness audit, human review trace, works-council approval.
- runbook reference: `docs/runbooks/privacy-council-data-class-review.md`.
- incident class: workforce-ai-trust.
- customer communication: workplace AI obligation packet.
- regulator action reference: EU AI Act workplace/education restrictions and GDPR transparency.
- ADR reference: ADR-0309, ADR-0144, ADR-0008, ADR-0319.
- residual risk: Medium because workplace AI is socially sensitive.
- checkpoint: Workforce AI requires Privacy Council review.
- escalation: Privacy Council and Legal.

## Section 8. Risk Heatmap

Matrix name: OYA-L5 x OYA-I5 snapshot on 2026-05-20.

| Impact \\ Likelihood | Rare 1 | Unlikely 2 | Possible 3 | Likely 4 | Almost Certain 5 |
|---|---|---|---|---|---|
| Critical 5 | - | OPS-005 | STR-002, OPS-001, OPS-002, OPS-004, OPS-012, REG-002, REG-004 | STR-001 | - |
| Severe 4 | - | - | STR-003, STR-004, STR-007, OPS-009, OPS-011, REG-007, REG-008, REG-009, REG-013, TECH-004, TECH-009, TECH-011, TECH-012, TECH-013, COM-003, COM-004, REP-001, REP-005, REP-006 | STR-006, REG-001, REG-003, REG-006, TECH-002, TECH-008, TECH-010 | - |
| Major 3 | - | - | OPS-010, REG-011, REG-012, TECH-005, COM-002, REP-002, REP-004 | STR-005, OPS-008, REG-005, REG-010, TECH-001, TECH-003, TECH-006, TECH-007, COM-001, COM-005, COM-006 | - |
| Moderate 2 | - | - | REP-003 | - | - |
| Limited 1 | - | - | - | - | - |

Extreme risks: STR-001.
Severe risks: STR-002, STR-006, OPS-001, OPS-002, OPS-004, OPS-006, OPS-012, REG-001, REG-002, REG-003, REG-004, REG-006, TECH-002, TECH-008, TECH-010.
Elevated risks: STR-003, STR-004, STR-005, STR-007, OPS-003, OPS-005, OPS-007, OPS-008, OPS-009, OPS-011, REG-005, REG-007, REG-008, REG-009, REG-010, REG-013, TECH-001, TECH-003, TECH-004, TECH-006, TECH-007, TECH-009, TECH-011, TECH-012, TECH-013, COM-001, COM-003, COM-004, COM-005, COM-006, REP-001, REP-005, REP-006.
Managed risks: OPS-010, REG-011, REG-012, TECH-005, COM-002, REP-002, REP-003, REP-004.
Watch risks: none in this snapshot because the requested catalogue is intentionally focused on material risks.

## Section 9. Mitigation Mapping

| Risk ID | Named microservice mitigations | Named Cedar policies | Named monitoring | Named runbook |
|---|---|---|---|---|
| STR-001 | foundry guardrails, EU AI Act check, regional-pack API | prohibited_practice_refusal, high_risk_human_oversight | runtime autonomy tier, compliance pack attestation | compliance-pack-emergency-suspension |
| STR-002 | cloud-cell, hyperscaler metrics, honest-claims gate | hyperscaler_claim_publish_gate | cell routing, hyperscaler gates | ops/dr-drill-runbook |
| STR-003 | evidence pack builder, RACI check, CODEOWNERS mirror | owner_handoff_required | RACI freshness, backup owner coverage | on-call-handover |
| STR-004 | cloud billing, metering, provider router | cost_budget_enforced | provider cost per tenant, FinOps close | finops-monthly-close |
| STR-005 | eval runner, evidence builder, output validator | eval_gate_before_model_cutover | eval parity trend, provider health | foundry-model-cutover |
| STR-006 | regional-pack API, residency kernel, DSR | cross_jurisdiction_conflict_hold | compliance pack attestation, publication feed | regulatory-change-response |
| STR-007 | autonomy policy enforcement, kill switch, audit chain | autonomy_ceiling_runtime_enforced | autonomy violation rate, kill-switch coverage | autonomy-ceiling-breach-attempt |
| OPS-001 | cloud-cell app, cell kernel, tenant quotas | cell_mutation_scope | cell-routing dashboard, golden signals | cell-evacuation |
| OPS-002 | audit-chain domain, evidence bridge | audit_event_emit_required | audit throughput, evidence export status | audit-chain-integrity-failure |
| OPS-003 | messenger, KMS, secrets, identity | mls_group_membership_tenant_bound | MLS key delivery health | per-cell-hsm-rotation |
| OPS-004 | tenancy kernel, Cedar policy, tenant middleware | tenant_scope_required | tenant isolation health | cell-isolation-breach |
| OPS-005 | KMS, secrets, audit chain, identity | recovery_key_unwrap_m_of_n | key unwrap and HSM health | shamir-share-loss-or-coercion |
| OPS-006 | Cedar domain, fragment coverage, guardrails engine | cedar_publish_review_required | policy latency, fragment anomaly | cedar-fragment-emergency-rollback |
| OPS-007 | residency kernel, region domain, storage domain | residency_region_allowlist | residency egress, pack attestation | tenant-data-residency-violation |
| OPS-008 | evidence builder, runbook index, doc catalog check | incident_commander_runbook_ack | runbook freshness, game-day pass rate | ops/sev-1-bridge-procedure |
| OPS-009 | KMS, secrets, audit chain, cloud IAM | hsm_break_glass_dual_control | HSM health, unwrap latency | cloud/kms-emergency-rotation |
| OPS-010 | eventing, run ledger, workflow engine | dlq_replay_operator | DLQ depth, outbox lag | webhook-delivery-failure |
| OPS-011 | cloud mutation, feature flags, tenant CLI | production_change_dual_control | mutation audit, claim mismatch | release-rollback |
| OPS-012 | storage, audit chain, search, cell app | restore_operator_dual_control | restore drill, RPO lag | dr-drill-playbook |
| REG-001 | DSR, data boundary, regional pack | notice_required_before_processing | notice coverage, DSR SLA | privacy-council-data-class-review |
| REG-002 | residency kernel, export profiles, DSR | eu_transfer_mechanism_required | transfer manifest, access logs | cross-pack-tenant-residency |
| REG-003 | KR pack, residency, data boundary | kr_pipa_cross_border_notice | KR publication feed, consent receipts | regulator-publication-feed-health |
| REG-004 | healthcare pack, audit chain, evidence | hipaa_phi_minimum_necessary | PHI coverage, breach clock | phi-leak-suspected |
| REG-005 | consent, ads analytics, DSR | ccpa_do_not_sell_share | GPC test, opt-out SLA | consent-withdrawal-cascade |
| REG-006 | AI Act registry, high-risk refusal check | eu_ai_act_classification_required | high-risk capability count | capability-eval-regression |
| REG-007 | evidence export, observability, cloud cell | dora_financial_tenant_controls | resilience drill, ICT roster | regulator-notification-procedure |
| REG-008 | sovereign overlays, cloud cell, KMS | sovereign_cell_no_external_provider | sovereign egress, local key custody | cross-pack-tenant-residency |
| REG-009 | marketplace, tenancy, guardrails | sanctions_screen_required | screening match, region deny | marketplace-listing-takedown |
| REG-010 | US state pack, DSR, consent | us_state_privacy_overlay | state pack freshness | regulatory-change-response |
| REG-011 | APAC pack, residency, evidence export | apac_pack_required | pack attestation, counsel freshness | regulatory-change-response |
| REG-012 | India pack, residency, marketplace | in_dpdp_consent_required | India pack status, local route | regulatory-change-response |
| REG-013 | identity, messenger, community, mail, content safety | minor_age_gate | age assurance, minor reports | breach-notification-council-escalation |
| TECH-001 | flat crates check, doc catalog, gate catalog | service_promotion_requires_owner | service coverage, missing runbooks | flat-crates-move-pr |
| TECH-002 | license check, SBOM, plugin sandbox | critical_cve_release_block | Trivy, SBOM, provenance | supply-chain-compromise |
| TECH-003 | schema registry, SDK release, eventing | contract_breaking_change_review | contract diff, SDK regen | contract-breaking-change |
| TECH-004 | ontology spec, projection schema, KG audit | ontology_schema_publish_review | KG audit, projection compatibility | og-schema-rollback |
| TECH-005 | workflow studio, docs, tasks CRDT controls | crdt_replay_operator | convergence, replay divergence | workspace/doc-crdt-divergence |
| TECH-006 | Cedar domain, fragment coverage, engine | policy_benchmark_required | Cedar latency, false positives | cedar-policy-rollback |
| TECH-007 | observability, metrics, perf check | perf_gate_required_for_promotion | golden signals, SLO burn | error-budget-exhaustion |
| TECH-008 | capacity domain, tenant quotas, provider router | tenant_quota_enforced | quota utilization, provider limits | capacity-scaling-emergency |
| TECH-009 | eventing, schema registry, audit chain | event_schema_publish_review | broker lag, schema compatibility | topic-schema-rollback |
| TECH-010 | eval runner, provider router, output validator | model_cutover_eval_required | eval pass rate, provider health | eval-eval-set-rollback |
| TECH-011 | observability domain, audit chain | telemetry_data_class_filter | trace coverage, redaction tests | cross-plane-call-introduction |
| TECH-012 | compute, cell app, runtime orchestrator | workload_cell_affinity_required | pending pods, topology skew | supervisor-kubernetes-operator-restart |
| TECH-013 | data domain, analytics, quota kernel | database_migration_approval | replica lag, partition skew | analytics-warehouse-reconciliation |
| COM-001 | DSR, workflow export, ontology export | tenant_export_authorized | export success, portability SLA | dsr-cascade-proof-of-erasure |
| COM-002 | evidence builder, developer portal | trust_portal_public_evidence | RFP objections, demo pass | partner-contract-renewal |
| COM-003 | connectors, identity, workflow, ontology | tenant_onboarding_stage_gate | onboarding milestones, import errors | design-partner-onboarding |
| COM-004 | capability tiers, quotas, pack overlays | customer_exception_expiry | revenue concentration, exception count | tenant-escalation-management |
| COM-005 | first-deliverable package, trust portal | gtm_claim_evidence_required | sales cycle, proof-of-value pass | design-partner-feedback-session |
| COM-006 | billing, metering, provider router | discount_approval_dual_control | margin, provider cost | cost-anomaly-response |
| REP-001 | kill switch, evidence builder, audit chain | autonomy_incident_freeze | social listening, autonomy incidents | trust-portal-publish-procedure |
| REP-002 | DSR, evidence builder, audit chain | privacy_incident_fact_pack | complaint rate, DSR SLA | breach-notification-council-escalation |
| REP-003 | vendor ledger, analytics reporting | supplier_esg_review_required | supplier review age | partner-contract-renewal |
| REP-004 | output validator, invocation recorder | generated_content_review_required | content disputes, similarity alerts | guardrails-rule-store-restore |
| REP-005 | content safety engine, marketplace, messenger, community, mail | content_takedown_authorized | abuse queue, takedown SLA | marketplace-listing-takedown |
| REP-006 | tasks, analytics, workplace policy, guardrails | workplace_monitoring_notice | fairness audit, appeal rate | privacy-council-data-class-review |

## Section 10. Acceptance and Transfer Decisions

Accepted residual macro risks: STR-004 capital-market downturn, STR-005 model commoditization, COM-002 competitive lock-in narrative, COM-006 pricing pressure, REP-003 ESG supplier opacity.
Accepted residual operational limits: OPS-003 endpoint compromise residual after MLS controls, OPS-005 lost-share residual after dual-control key ceremonies, OPS-012 tier-specific RPO/RTO limits when contractually declared.
Accepted product-timing limits: REG-011 APAC pack breadth, REG-012 India rollout delay, COM-005 narrow wedge messaging before broad fanout.
Explicitly not accepted: STR-001 prohibited AI practices, OPS-004 multi-tenant data leak, OPS-006 cross-tenant Cedar escape, REG-002 unlawful EU transfers, REG-004 PHI processing without HIPAA controls, TECH-002 unwaived critical supply-chain vulnerability.
Transferred partially through insurance: data breach response cost, cyber incident cost, business interruption, E&O defense, D&O governance exposure, supplier contractual indemnity.
Not transferable: product prohibition, trust collapse, regulator enforcement discretion, customer churn, platform architecture debt, factual evidence gaps, and unsupported public claims.
Transfer owner: `gtm-partnerships` for policy and vendor contract posture; `ops-security` for cyber policy; `council-architecture` for non-transferable acceptance decisions.
Acceptance renewal: all accepted residual risks expire quarterly unless re-ratified in Board Risk Review.
Exception rule: a customer-specific contract cannot accept a risk that this register marks "explicitly not accepted" without Architecture Council and legal approval.
Evidence rule: every accepted or transferred risk must point to the current evidence artifact, insurance binder, contract clause, or board minute.

## Section 11. Owner Assignments

| Owner role | Primary risks | Cadence | Backup role |
|---|---|---|---|
| council-architecture | STR-002, STR-003, TECH-001, architecture acceptance | Weekly severe review, quarterly board review | Founder or Chief Architect delegate |
| council-privacy | OPS-004, REG-001, REG-002, REG-005, REG-010, REG-013, REP-006 | Monthly privacy council, weekly severe risks | Data Protection Officer delegate |
| ops-security | OPS-002, OPS-005, OPS-006, TECH-002, key and supply-chain risks | Weekly security review | Security Engineering lead |
| ops-sre-reliability | OPS-001, OPS-008, OPS-010, OPS-012, TECH-007, TECH-011, TECH-013 | Weekly SRE review | Incident Commander rotation |
| ops-compliance | STR-001, REG-006, REG-007, REG-009 | Monthly compliance review, weekly AI Act/DORA readiness | Compliance Operations lead |
| ops-finops | STR-004, COM-006, TECH-008 | Monthly FinOps close | Finance owner |
| axis-foundry | STR-005, STR-007, TECH-010, OPS-011 | Weekly Foundry governance | Foundry Runtime owner |
| axis-cloud | OPS-009, REG-008, TECH-012 | Monthly platform review | Cloud SRE lead |
| axis-messenger / axis-community / axis-mail | OPS-003, REP-005 support | Monthly security review | Messaging, Community, and Mail Security leads |
| regional-packs | STR-006, REG-003, REG-011, REG-012 | Monthly regulator watch | Compliance Pack maintainer |
| vertical-healthcare | REG-004 | Monthly healthcare control review | Privacy Engineering |
| gtm-sales-se | COM-001, COM-002, COM-003, COM-004, COM-005 | Monthly commercial risk review | Customer Success lead |
| gtm-partnerships | REP-003, transfer ledger | Quarterly vendor review | Legal operations |
| comms-trust | REP-001, REP-002, REP-004, REP-005 | Monthly trust tabletop, immediate incident bridge | Trust and Safety lead |

## Section 12. Cross-References to ADRs

ADR-0001 cohesion thesis: COM-004 and COM-001 rely on no per-customer forks and one-product architecture.
ADR-0002 tenant and identity kernel: OPS-004 and tenant-scope risks depend on universal tenant identity.
ADR-0003 audit chain and evidence emission: OPS-002 and REG-007 depend on complete evidence emission.
ADR-0007 Cedar authorization policy: OPS-006 and TECH-006 depend on policy correctness and scope.
ADR-0008 data use boundary: REG-001, REG-004, REG-005, and REP-006 depend on data-class discipline.
ADR-0009 cell architecture per tenant per region: OPS-001 and OPS-012 depend on cell isolation and recovery.
ADR-0010 regional pack architecture: STR-006 and regional compliance risks depend on localization packs.
ADR-0013 product license policy: TECH-002 and REG-009 depend on acceptable dependency and product licensing.
ADR-0014 build vs buy policy: REP-003 and supplier posture use vendor-selection boundaries.
ADR-0019 doc catalog and update protocol: OPS-008 and ownership/cadence depend on doc lifecycle.
ADR-0021 Foundry capability registry and MCP gateway: STR-005 and capability governance rely on registry discipline.
ADR-0022 autonomy ceiling runtime enforcement: STR-001, STR-007, REG-006, and REP-001 rely on runtime gates.
ADR-0026 in-house AI model substrate roadmap: STR-005 and TECH-010 rely on provider abstraction and model lifecycle.
ADR-0031 ads and analytics architecture: REG-005 and privacy risks depend on data-use boundaries in adtech.
ADR-0037 public API stability and deprecation: TECH-003 and customer migration risks depend on stable contracts.
ADR-0039 supply chain security: TECH-002 depends on Trivy, Cosign, SBOM, and signed artifacts.
ADR-0040 progressive delivery rollback: TECH-007 depends on canary and metric-gated rollback.
ADR-0042 observability stack: TECH-011 depends on OTel and in-house UI coverage.
ADR-0043 secrets management OpenBao and HSM per cell: OPS-005 and OPS-009 rely on key custody controls.
ADR-0045 database tier strategy: TECH-013 depends on database tiering and recovery controls.
ADR-0049 cross-region replication and residency: REG-002 and OPS-007 depend on transfer and failover controls.
ADR-0056 Rust clean architecture BNF: TECH-001 depends on bounded microservice structure.
ADR-0058 flat microservice catalog: TECH-001 depends on flat layout and catalogue discipline.
ADR-0061 application B2B shell: COM-005 depends on clear product surface.
ADR-0064 canonical base and localization packs: STR-006 and COM-004 depend on no region forks.
ADR-0069 active machine-readable artifact contract: all machine-readable evidence follow-up depends on this contract.
ADR-0123 hyperscaler maturity claim gate: STR-002 depends on honest claim governance.
ADR-0128 hyperscaler architecture invariants: STR-002 and TECH-012 depend on hyperscaler-pattern evidence.
ADR-0131 per-microservice flat layout: TECH-001 depends on service consistency.
ADR-0134 portfolio hyperscaler remediation backlog: STR-002 depends on remediation evidence.
ADR-0139 agentic SLO-gated promotion: STR-007 and TECH-010 depend on promotion gates.
ADR-0142 CRDT portability trait: TECH-005 depends on portable collaborative-state semantics.
ADR-0144 EU AI Act graduated risk tier model: STR-001 and REG-006 depend on risk classification.
ADR-0148 service mesh Cilium/Istio layering: TECH-012 depends on network isolation.
ADR-0152 RPO/RTO canonical: OPS-012 and TECH-013 depend on declared recovery objectives.
ADR-0153 outbox pattern: OPS-010 and TECH-009 depend on reliable event relay.
ADR-0154 event schema versioning: OPS-010, TECH-003, and TECH-009 depend on event compatibility.
ADR-0155 per-tenant resource quotas: TECH-008 depends on capacity and quota controls.
ADR-0156 PII registry canonical: REG-001 and privacy notice controls depend on inventory.
ADR-0162 per-tenant audit log slicing: OPS-002 and OPS-004 depend on tenant-scoped audit evidence.
ADR-0164 sovereign cloud air-gapped: REG-008 depends on sovereign overlay constraints.
ADR-0165 chaos engineering substrate: OPS-001 and OPS-012 depend on game-day evidence.
ADR-0166 schema registry: TECH-003 and TECH-009 depend on schema compatibility.
ADR-0173 vendor lock-in avoidance and stack ownership: COM-001 and COM-002 depend on portability posture.
ADR-0174 FinOps cost attribution and chargeback: STR-004 and COM-006 depend on cost evidence.
ADR-0178 layered throttling tiers: TECH-008 and COM-006 depend on quota enforcement.
ADR-0180 SLO composition inheritance arithmetic: REG-007 and TECH-007 depend on SLO evidence.
ADR-0183 policy engine separation: OPS-006 and TECH-006 depend on Cedar/Kyverno separation.
ADR-0186 observability backplane layering: TECH-011 depends on observability architecture.
ADR-0187 canonical OIDC IdP Zitadel primary: COM-003 depends on identity onboarding.
ADR-0189 step-up authentication ACR classes: OPS-005 depends on recovery and key access step-up.
ADR-0190 SCIM provisioning enterprise tenants: COM-003 depends on enterprise onboarding.
ADR-0191 edge authz tier vs origin Cedar PDP: OPS-006 and TECH-006 depend on authz placement.
ADR-0193 OLAP warehouse ClickHouse: TECH-013 and analytics capacity depend on warehouse design.
ADR-0198 K8s node autoscaling Karpenter: TECH-012 and STR-002 depend on autoscaler evidence.
ADR-0199 per-tenant cost attribution FinOps: STR-004, TECH-008, and COM-006 depend on cost attribution.
ADR-0202 GitOps IaC cluster lifecycle: STR-002, OPS-011, and TECH-012 depend on IaC controls.
ADR-0209 compliance evidence automation: REG-001, REG-004, REG-007, and OPS-002 depend on evidence automation.
ADR-0212 buildability doctrine: STR-003 and TECH-001 depend on buildable, testable gates.
ADR-0216 open integration and migration-out policy: COM-001 and COM-002 depend on exit posture.
ADR-0217 vertical slice rollout order: STR-004 and COM-005 depend on first-deliverable sequencing.
ADR-0223 Oya git drop-in surface: OPS-011 depends on sanctioned primitive transition.
ADR-0240 sovereign cloud per regional pack: STR-006, REG-008, and REG-002 depend on regional pack sovereignty.
ADR-0241 DR business continuity portfolio policy: OPS-012 and REG-007 depend on continuity controls.
ADR-0243 Cedar as universal gate: OPS-006 and TECH-006 depend on universal authz gate discipline.
ADR-0244 tenant as universal scoping primitive: OPS-004 depends on tenant-scoped everything.
ADR-0248 Amazon-shape cellular architecture: STR-002 and OPS-001 depend on cell blast-radius doctrine.
ADR-0251 compliance pack cell certification levels: REG-008 and regulated packs depend on certification tiers.
ADR-0254 deployment model spectrum: STR-004, OPS-009, and REP-003 depend on deployment posture.
ADR-0255 intelligence as two-layer AI substrate: STR-005, TECH-010, and REP-004 depend on AI substrate boundaries.
ADR-0257 ontology object type versioning: TECH-004 depends on deprecation handshake.
ADR-0272 cookie consent per-purpose analytics opt-in: REG-005 depends on consent controls.
ADR-0276 backup portability format GDPR Article 20: COM-001 and REG-002 depend on export and portability.
ADR-0292 minor user doctrine: REG-013 and REP-005 depend on minor safety boundaries.
ADR-0293 Foundry meta trust root: STR-007 and REP-001 depend on trusted agent supervision.
ADR-0294 Cedar fragment soak anomaly rollback: OPS-006 and TECH-006 depend on policy rollback.
ADR-0295 bootstrap CI SPIFFE kill switch: OPS-011 and TECH-002 depend on CI trust controls.
ADR-0298 emergency services bypass life safety: REG-004 and safety workflows depend on controlled break-glass.
ADR-0299 account recovery resilience: OPS-003 and OPS-005 depend on recovery controls.
ADR-0301 survivor safety domestic abuse mode: REG-013 and REP-005 depend on safety-sensitive design.
ADR-0304 cross-jurisdiction conflict resolution: STR-006 and REG-002/003/008/009 depend on conflict process.
ADR-0305 delegated agent authority chain: STR-007 and REP-001 depend on authority traceability.
ADR-0306 disaster mode cell resilience: OPS-001 and OPS-012 depend on cell resilience.
ADR-0308 ML model lifecycle AI Act compliance: STR-001, REG-006, TECH-010, and REP-004 depend on model lifecycle.
ADR-0309 detection fairness audit civil rights: STR-001 and REP-006 depend on fairness evidence.
ADR-0312 court warrant scoped piercing: OPS-005 and REG-004 depend on scoped lawful-access controls.
ADR-0313 conglomerate tenant hierarchy sovereign children: STR-006 and REG-008 depend on tenant hierarchy.
ADR-0314 marketplace as universal deal settlement: REG-009, REP-005, and COM-002 depend on marketplace controls.
ADR-0316 capability tier over product fragmentation: STR-005, COM-004, COM-006, and TECH-001 depend on capability tiering.
ADR-0319 front/middle/back office information barrier: REP-006 depends on information barrier controls.
ADR-0321 B2B SaaS industry leader coverage: COM-005 depends on realistic market coverage.

## Checkpoint

Risk catalogue count: 57.
Register owner: `council-architecture`.
Next required review: next Weekly Risk Council after 2026-05-20.
Required follow-up artifact: machine-readable mirror candidate under the active artifact contract when PHASE-5/PHASE-6 work schedules this Markdown projection for retirement.
Clean halt condition: document authored, line count verified, Oya VCS verify/done/promote run with `risks_catalogued:57` and exact `register_lines` evidence.
