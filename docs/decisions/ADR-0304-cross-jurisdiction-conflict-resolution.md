---
id: ADR-0304
status: Accepted
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-legal
  - council-data-governance
  - ops-sre-reliability
  - ops-compliance
  - axis-policy-engine
  - axis-audit-chain
  - axis-tenancy
  - axis-data-residency
supersedes: []
amends: []
superseded_by: []
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0272-cookie-consent-per-purpose.md
  - ADR-0276-backup-portability-gdpr-art-20.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-bypass-doctrine.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0300-whistleblower-press-freedom-anonymity.md
  - ADR-0301-survivor-safety-domestic-abuse-mode.md
  - ADR-0302-deceased-user-inheritance-doctrine.md
  - ADR-0303-cognitive-impairment-decision-resilience.md
  - ADR-0305-delegated-agent-authority-chain.md
  - ADR-0306-disaster-mode-cell-resilience.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/governance.json
  - /specs/compliance-pack-schema.json
  - /specs/jurisdiction-conflict-resolution.json
  - /specs/data-residency-pack-floor.json
related_memory:
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_canonical_base_localization
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_amazon_shape_cellular_architecture
  - feedback_compliance_pack_primitive
  - feedback_naming_justification
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: critical-path-cluster-cross-jurisdiction-conflict-resolution
purpose: >
  Establish the Cross-Jurisdiction Conflict Resolution doctrine — a
  substrate-level primitive that enforces per-pack data-residency
  hard-stops, encodes per-tenant jurisdictional preference + per-pack
  regulator floors, applies the higher-restriction-pack-wins rule,
  requires multi-pack alignment for cross-border transfers, and
  produces a per-request transparency report citing every applicable
  pack. The bar is: when EU GDPR conflicts with US CLOUD Act, when
  KR-PIPA conflicts with US subpoena, when CN-PIPL conflicts with EU
  GDPR, the substrate resolves deterministically by giving the user
  the highest applicable protection floor + auditing the choice. Per
  documentation-rigor.md §3.2.5 row 23.
enforcement_status: advisory-until-2026-10-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet cross-jurisdiction-data-residency-hardstop
  - cloud-ci/Rust gate packet cross-jurisdiction-pack-precedence-graph
  - cloud-ci/Rust gate packet cross-jurisdiction-multi-pack-alignment
  - cloud-ci/Rust gate packet cross-jurisdiction-transparency-report-coverage
  - cloud-ci/Rust gate packet cross-jurisdiction-conflict-audit-emission
naming_justifications:
  - name: oya-shared-jurisdiction-conflict
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.jurisdiction-conflict
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the per-pack precedence-graph evaluator
      trait + multi-pack-alignment validator trait + transparency-
      report builder trait + data-residency-floor enforcer trait
      belongs at the shared layer. Naming
      `oya-shared-jurisdiction-conflict` keeps the single-concern
      flat layout per ADR-0131 and avoids any "suite" packaging per
      ADR-0132.
  - name: oya-governance-data-residency-hardstop
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.data-residency-hardstop
    justification: >
      CI fitness lane per ADR-0212 buildability doctrine; verifies
      every µservice with data-residency-restricted packs in its
      pack roster declares a hard-stop guard preventing cross-border
      data egress. Lane naming follows the canonical
      `oya-governance-<concern>` shape consistent with ADR-0297
      sibling lanes.
  - name: oya-governance-pack-precedence-graph
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.pack-precedence-graph
    justification: >
      CI fitness lane per ADR-0212; verifies the substrate's
      precedence graph is acyclic + total-orderable + that every
      pair of overlapping packs has a resolved precedence rule.
  - name: oya-governance-multi-pack-alignment
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.multi-pack-alignment
    justification: >
      CI fitness lane per ADR-0212; verifies that cross-border
      data transfers have multi-pack alignment recorded + audited.
  - name: oya-governance-transparency-report
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.transparency-report
    justification: >
      CI fitness lane per ADR-0212; verifies the transparency-report
      surface enumerates every applicable pack per consequential
      request and links to the per-pack legal-cite.
  - name: oya-governance-jurisdiction-conflict
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.jurisdiction-conflict
    justification: >
      Aggregate fitness lane per ADR-0212; rolls up the child lanes
      into a single advisory/BLOCKER gate per the keystone-bundle
      2026-05-20 promotion-gate model.
  - name: X-Oya-Pack-Cites
    layer: N/A (HTTP response header naming)
    bnf_segments: X-Oya.Pack-Cites
    justification: >
      Custom HTTP response header carrying the comma-separated list
      of applicable pack-ids consulted on this request; namespace
      prefix `X-Oya-` reserves the platform's header surface and
      makes the per-request transparency surface debuggable for
      operators.
  - name: X-Oya-Residency-Cell
    layer: N/A (HTTP response header naming)
    bnf_segments: X-Oya.Residency-Cell
    justification: >
      Custom HTTP response header identifying the cell-id that
      processed the request, satisfying the per-pack residency-
      audit surface; cell-id is per ADR-0248 cell-naming convention.
  - name: JurisdictionConflictResolved
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: JurisdictionConflict.Resolved
    justification: >
      Audit-event-class emitted whenever a conflict between two or
      more applicable packs is resolved by the substrate. Registered
      in ADR-0263 central registry per §3.2.2 consistency invariant.
  - name: DataResidencyHardStopEnforced
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: JurisdictionConflict.DataResidencyHardStopEnforced
    justification: >
      Audit-event-class emitted when a request is refused due to a
      data-residency hard-stop. Registered per ADR-0263.
  - name: CrossBorderTransferDenied
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: JurisdictionConflict.CrossBorderTransferDenied
    justification: >
      Audit-event-class emitted when a cross-border data transfer
      request is denied due to lack of multi-pack alignment.
      Registered per ADR-0263.
  - name: TransparencyReportEmitted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: JurisdictionConflict.TransparencyReportEmitted
    justification: >
      Audit-event-class emitted whenever the per-request transparency
      report is generated. Registered per ADR-0263.
  - name: policy/jurisdiction-conflict.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.jurisdiction-conflict
    justification: >
      Canonical filename for the per-µservice Cedar fragment under
      the µservice's `policy/` directory per ADR-0246 + ADR-0243
      fragment-lifecycle conventions; single-concern naming keeps
      the policy directory's contract-by-name invariant.
  - name: iac/<env>-pack-residency.yaml
    layer: N/A (per-µservice IaC manifest)
    bnf_segments: iac.<env>.pack-residency
    justification: >
      Canonical filename for per-µservice + per-env data-residency
      IaC manifest declaring per-pack cell-pinning + cross-border
      transfer policy.
  - name: precedence_graph
    layer: N/A (per-pack registry attribute)
    bnf_segments: pack.precedence_graph
    justification: >
      Each compliance pack carries a `precedence_graph` directed-
      acyclic-graph (DAG) declaration mapping it to every other
      pack with a higher-or-lower restriction. Used by the
      substrate's higher-restriction-wins algorithm.
---

# ADR-0304: Cross-Jurisdiction Conflict Resolution Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-cluster-cross-jurisdiction-conflict-
resolution** keystone, closing the gap identified in
`docs/standards/documentation-rigor.md` §3.2.5 row 23 of the
critical-path edge-case coverage matrix. The standard already
codifies row 23 handling requirements (per-pack data-residency
hard-stop; per-tenant jurisdictional preference + per-pack regulator
floor; higher-restriction pack wins; cross-border transfer requires
multi-pack alignment; transparency-report cites all applicable packs);
this ADR is the binding ADR the standard's row 23 cites.

Enforcement is `advisory-until-2026-10-15-blocker-thereafter`. The
doctrine accepts in text immediately; the CI lanes promote to
BLOCKER on 2026-10-15 to give per-µservice pack roster authoring
time to land. Until 2026-10-15, validators emit findings without
failing CI; post-2026-10-15, the lanes block merge.

## Date

2026-05-20.

## Context

### §A. Why cross-jurisdiction conflict resolution is a substrate primitive

Modern hyperscaler-class multi-region platforms treat cross-jurisdiction
conflict resolution as a *first-class substrate primitive* — wired into
the per-µservice request path, in the per-tenant policy engine, and
composing with every Cedar gate. The pattern is unambiguous across
the named industry references:

- **Microsoft Azure Sovereign Regions** (Azure Government, Azure
  China 21Vianet, Azure Germany Berlin, Azure for European
  Sovereign Cloud). Per Microsoft's "Sovereign Cloud" 2024
  documentation, every Microsoft global service that crosses
  sovereign boundaries undergoes per-jurisdiction conflict
  evaluation at the control-plane gate. The pattern's depth: per-
  tenant sovereignty declaration, per-resource cell-pinning, hard-
  stop on cross-border egress, transparency-report per regulator
  request. Microsoft published ~3,300 government data requests
  served per H1-2024; each carries per-pack precedence resolution.
- **AWS Outposts + AWS Local Zones + AWS Wavelength + AWS GovCloud**.
  Per AWS's 2024 GovCloud documentation, every workload tagged
  `Sovereignty=GovCloud` enforces hard-stop on cross-region egress
  to non-GovCloud regions. AWS GovCloud is operated as a substrate-
  isolated control plane with separate IAM, separate audit, and
  separate data plane. Per AWS Transparency Report H1-2024, AWS
  received ~10,200 government information requests; the conflict-
  resolution layer is the substrate boundary.
- **Google Cloud Assured Workloads** (FedRAMP High, US-Sovereign,
  IL5, EU-Sovereign, KR-K-FSI). Per Google Cloud Assured Workloads
  2024 documentation, every workload tagged with a regulatory
  framework enforces per-resource-type compliance constraints +
  hard-stop on cross-jurisdiction transfer. Google publishes the
  per-region data-localization matrix as substrate primitive, not
  per-tenant configuration.
- **Salesforce Hyperforce** + **Workday Sovereign Cloud** + **Oracle
  Sovereign Cloud Region (OSC)** + **Box Zones** + **Slack EKM**.
  Per their respective 2024 documentation, every Tier-1 enterprise
  SaaS now ships sovereign-cloud + per-jurisdiction-conflict-
  resolution as a baseline. The pattern is universal: substrate
  primitive, not per-tenant configuration.
- **Cloudflare Data Localization Suite** (Cloudflare Regional
  Services + Customer Metadata Boundary + Geo Key Manager). Per
  Cloudflare's 2024 documentation, every Cloudflare zone enforces
  per-region data-pinning + per-jurisdiction key-management. The
  substrate primitive serves the conflict resolution; customer
  zones declare their pack roster.

The corollary: **every internet-facing surface oyatie ships MUST
inherit cross-jurisdiction conflict resolution from the substrate, not
author it per-µservice.** A µservice that authors its own per-pack
cell-pinning logic, its own cross-border egress checks, its own
transparency-report generator, its own pack-precedence evaluator is
duplicating substrate primitives that the shared
`oya-shared-jurisdiction-conflict` crate already serves. That
duplication is a `feedback_no_silent_regression` violation (every
µservice's conflict handling drifts independently); it is a
`feedback_quality_performance_scalability_bar` violation (a substrate
primitive sees signal across every µservice's cross-jurisdiction
requests that a single µservice cannot); and it is a
`feedback_autonomous_implementation_artifacts` violation (intern-
buildable means one substrate, not 46 µservice-private
implementations).

The ADR-0304 cross-jurisdiction conflict resolution doctrine closes
this gap.

### §A.1. The cross-jurisdiction conflict landscape 2026

The 2026 cross-jurisdiction conflict landscape is qualitatively
different from any prior era:

- **EU GDPR (Regulation 2016/679) vs US CLOUD Act (18 USC §2701-13).**
  The 2018 US CLOUD Act allows US law enforcement to compel U.S.-
  based providers to disclose data held overseas. EU GDPR Articles
  6, 44-49 require lawful basis + Standard Contractual Clauses
  (SCCs) for cross-border transfers. The EU-US Data Privacy
  Framework (DPF, 2023-07-10) provides a limited resolution; per
  the EU CJEU's Schrems II ruling (Case C-311/18), additional
  safeguards may be required. Per Microsoft's 2018 Microsoft v.
  Ireland Supreme Court case, the conflict is real, recurrent,
  and unresolved by treaty.
- **KR-PIPA Article 28 + Article 39-12 vs US subpoena**. Korean
  PIPA (개인정보 보호법) restricts cross-border personal-data
  transfer unless the receiving jurisdiction provides equivalent
  protection. Per KCC's 2024 Cross-Border Transfer Standard, KR-
  PIPA requires SCC + explicit user consent for transfers to non-
  adequate jurisdictions. A US subpoena cannot override KR-PIPA
  obligations without bilateral treaty (MLAT) processing.
- **CN-PIPL Article 38 + Article 53 vs EU GDPR**. China's PIPL
  (个人信息保护法, effective 2021-11-01) restricts cross-border
  personal-data transfer outside China except via PIPL Article 38
  (CAC security assessment) or Article 53 (CAC certification).
  EU GDPR's adequacy decision for China is absent; transfers between
  China and the EU require multi-pack alignment.
- **CN-DSL + CN-CSL data-export rules**. China's Data Security Law
  (数据安全法, 2021-09-01) and Cybersecurity Law (网络安全法,
  2017-06-01) impose additional restrictions on "important data"
  and Critical Information Infrastructure Operator (CIIO) data.
  Cross-border transfer requires CAC security assessment.
- **GDPR Article 17 (right-to-erasure) vs US litigation-hold
  (Federal Rules of Civil Procedure §37(e))**. EU residents may
  demand erasure; US courts may impose litigation holds. Multi-
  pack alignment requires balancing.
- **EU AI Act + UK AI bill + CN AI rules + KR-AI guidance**. The
  emerging AI regulatory landscape adds per-jurisdiction model-
  training + model-serving restrictions; the substrate must enforce
  per-pack model-training-region floors.
- **GDPR + UK-GDPR + CCPA + VCDPA + CPRA + KR-PIPA + JP-APPI**.
  At least 130 jurisdictions now have data-protection regulations;
  the substrate's pack registry covers the top-50 by tenant
  population coverage.
- **OECD AI Principles + Council of Europe AI Convention 2024**.
  Emerging international frameworks add overlay protections.
- **Sectoral regulation overlay**: HIPAA + COPPA + GLBA + FCRA +
  SOX in the US; FCA + PRA + BoE in the UK; BaFin in Germany;
  FSC + FSS in Korea; FSA in Japan. Sectoral packs compose with
  jurisdictional packs.

The substrate baseline MUST be sized to this 2026 landscape — not
the 2010 landscape that earlier privacy laws were designed against.
The bar is not "follow GDPR"; the bar is "operate per-pack precedence
graph evaluation across 130+ jurisdictional regimes + 50+ sectoral
overlays, deterministically, with audit + transparency at every
consequential request."

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate primitive

The keystone bundle's foundational ADRs intersect cross-jurisdiction
conflict as follows:

- **ADR-0240 (Sovereign-Cloud per Regional Pack).** Cells are
  jurisdiction-pinned per ADR-0240; ADR-0304 articulates how
  cell-pinned data behaves when requests cross the boundary.
- **ADR-0242 (oyatie-is-a-tenant).** The platform's own surfaces
  participate in the conflict resolution as a tenant. Even
  oyatie's internal mutations cross-jurisdiction-evaluate.
- **ADR-0243 (Cedar universal gate).** Every cross-jurisdiction
  decision composes as a Cedar fragment. The pack-precedence
  evaluator emits context attributes; Cedar fragments compose.
- **ADR-0244 (tenant scoping primitive).** Each tenant's data lives
  in a specific cell; the tenant's pack roster declares the
  applicable jurisdictions; cross-tenant data flow is forbidden by
  default; cross-jurisdiction within a tenant requires multi-pack
  alignment.
- **ADR-0248 (Amazon-shape cellular architecture).** Cells are
  per-jurisdiction; cell-pinning honors per-pack data-residency
  floor. The cellular topology IS the conflict-resolution boundary.
- **ADR-0251 (compliance packs).** Each pack carries a precedence
  graph DAG; the substrate consumes the DAG; the higher-restriction
  pack wins per the higher-restriction-wins rule.
- **ADR-0263 (observability emission contract).** Every conflict
  resolution emits an audit-event-class; every transparency-report
  emission is a row in the audit chain.
- **ADR-0276 (backup portability per GDPR Art. 20).** Cross-border
  backup portability requires multi-pack alignment per ADR-0304.
- **ADR-0297 (abuse-defence baseline).** Per-cell abuse-defence
  decisions consult per-pack residency floor for the cell.
- **ADR-0298 (emergency-services bypass).** Emergency-services
  bypass cross-jurisdiction conflict resolution; life-safety
  paramount per ADR-0298.
- **ADR-0303 (decision-resilience).** Per-pack cooling-off floor
  composes with the substrate per ADR-0303 §D-9 compliance
  interactions.
- **ADR-0306 (disaster-mode + cell resilience).** Disaster-mode
  preserves per-pack residency floor; cross-cell failover honors
  per-pack data-residency hard-stop.

The bundle cannot land without the cross-jurisdiction conflict
resolution doctrine articulated explicitly. The promotion gate for
the 2026-05-20 bundle is: *the substrate MUST resolve cross-pack
conflicts deterministically, honor the higher-restriction floor,
and emit a transparency report on every consequential request.*
This ADR is the binding articulation.

### §A.3. What this ADR explicitly does NOT do

- This ADR does not enumerate every per-jurisdiction pack — that
  is the per-pack repository under `packs/<jurisdiction>-<scope>/`.
  This ADR specifies the precedence-graph + multi-pack-alignment
  + transparency-report shape.
- This ADR does not specify per-tenant UI for jurisdiction
  preference declaration — that is the tenancy substrate per
  ADR-0244.
- This ADR does not redefine Cedar fragment authoring conventions
  — that is ADR-0243 + ADR-0294. This ADR specifies the *content*
  of `policy/jurisdiction-conflict.cedar`.
- This ADR does not specify legal-substantive interpretation of
  any regulatory regime — that is the per-pack legal-council axis
  responsibility. This ADR specifies the *mechanical-substrate*
  layer.
- This ADR does not displace MLAT processing for cross-border
  law-enforcement requests; the substrate may surface a request
  to the legal-council for MLAT routing, never substitute the
  treaty process.

## Decision

### §B. Five core primitives at three layers

The cross-jurisdiction conflict resolution baseline is **five core
primitives** (data-residency hard-stop; per-tenant jurisdictional
preference; per-pack regulator floor; higher-restriction-wins
precedence; transparency-report) wired at **three layers** (Tier-0
shared crate, per-µservice gate, Cedar policy fragment). The 5×3
matrix produces fifteen cells; each cell has a defined primitive.

```
                    Tier-0 shared          Per-µservice           Cedar policy
                    -------------          -------------          -------------
Data-residency      Cell-pin registry +    Per-route cell-pin +   forbid when
hard-stop           per-pack DAG            cross-border egress    cell_id ≠
                                            check                  required_cell

Per-tenant          Tenant.pack_roster +   Per-tenant policy      forbid when
preference          tenant.audience_type    consultation           preferred_pack ∉
                                                                    applicable_set
                                                                    AND ¬higher_floor_overrides

Per-pack            Per-pack precedence    Per-µservice pack      forbid when
regulator floor     graph DAG              roster + floor          floor_violated
                                            consultation

Higher-restriction- Precedence-graph       Tree-walk evaluator    forbid when
wins precedence     evaluator              over applicable set     decision violates
                                                                    higher pack

Transparency        Per-request report     Per-µservice report    permit but emit
report              builder + signer        emission                TransparencyReport
                                                                    EmittedEvent
```

The five primitives are **interconnected** but **separable** in
implementation:

- **Data-residency hard-stop** is the strongest primitive — it
  refuses requests where data would egress beyond the per-pack
  residency floor. No higher-restriction-pack-wins rule can
  override; this is the floor.
- **Per-tenant jurisdictional preference** allows the tenant to
  declare which jurisdiction(s) its users primarily inhabit;
  the substrate routes to the corresponding cell when consistent
  with per-pack data-residency.
- **Per-pack regulator floor** is the per-pack legal-minimum;
  for each pack, a list of rules that the substrate MUST satisfy
  regardless of any tenant preference.
- **Higher-restriction-wins precedence** is the conflict-resolution
  algorithm — when multiple packs apply with conflicting rules,
  the higher-restriction rule wins.
- **Transparency-report** is the audit + accountability surface —
  every consequential request emits a report enumerating which
  packs applied + which rule was honored.

The three layers are **complementary**:

- **Tier-0 shared crate** centralizes the per-pack precedence
  graph, the cell-pin registry, the transparency-report builder.
  Per-µservice paths import the crate.
- **Per-µservice gate** sees the µservice-local request context
  (the user's tenant, the requested resource, the action). The
  µservice contributes the per-request context.
- **Cedar policy fragment** composes the substrate + µservice +
  per-tenant + per-pack signals into a single permit/forbid
  decision per ADR-0243 + ADR-0263.

### §B.1. The higher-restriction-wins rule — full mechanics

The higher-restriction-wins rule is the deterministic conflict
resolution algorithm. Given a set of applicable packs `P = {p1,
p2, ..., pn}`, a request action `a`, and a request resource `r`,
the substrate evaluates:

```
applicable_rules = ∪ {rule ∈ pack.rules : rule.applies_to(a, r)
                                          for pack in P}

resolved_decision = max{r.restriction_level : r ∈ applicable_rules}

if resolved_decision = FORBID:
    refuse the request
elif resolved_decision = PERMIT_WITH_COOLING_OFF:
    cool-off per ADR-0303
elif resolved_decision = PERMIT_WITH_GUARDIAN_CO_SIGN:
    guardianship co-sign per ADR-0303 §D-4
elif resolved_decision = PERMIT_WITH_AUDIT:
    permit + emit audit
elif resolved_decision = PERMIT_WITH_TRANSPARENCY_REPORT:
    permit + emit transparency report
elif resolved_decision = PERMIT:
    permit
```

The restriction-level enum is total-ordered: `FORBID > PERMIT_WITH_
COOLING_OFF > PERMIT_WITH_GUARDIAN_CO_SIGN > PERMIT_WITH_AUDIT >
PERMIT_WITH_TRANSPARENCY_REPORT > PERMIT`. When two rules at the
same restriction-level disagree on enforcement details (e.g., one
pack mandates 30-day cool-off, another mandates 60-day), the
**longer/stronger** value wins (60-day wins).

The precedence-graph DAG encodes per-pack inter-relations:

```yaml
# packs/eu-gdpr/precedence.yaml
pack_id: pack-eu-gdpr
restriction_axis_max:
  data_subject_rights: 9  # right-to-erasure, right-to-portability, etc.
  cross_border_transfer: 9  # SCC + adequacy + Art.49 derogations
  sensitive_data: 8  # Art.9 special-category
  consent_grain: 8  # purpose-specific
restriction_axis_floor:
  cross_border_transfer: SCC_OR_ADEQUACY
  data_subject_rights: REQUIRED
  ai_transparency: ARTICLE_22

# packs/us-cloud-act/precedence.yaml
pack_id: pack-us-cloud-act
restriction_axis_max:
  govt_data_requests: 9  # warrants for US-controlled data
restriction_axis_floor:
  govt_data_requests: COMPLY_WITH_VALID_PROCESS

# Conflict resolution: when EU-GDPR + US-CLOUD-Act both apply to
# the same data: 
# - EU-GDPR forbids cross-border transfer without SCC.
# - US-CLOUD-Act compels disclosure under valid US process.
# Higher-restriction-wins: the SUBSTRATE refuses the US-CLOUD-Act
# disclosure and routes the request through MLAT or
# Microsoft v. Ireland-style legal challenge.
```

### §B.2. Worked examples — three canonical cross-jurisdiction conflicts

#### §B.2.a. GDPR × CLOUD Act

Scenario: a tenant operating in Germany has user data stored in
the EU-Sovereign cell. A US law enforcement agency issues a CLOUD
Act subpoena to oyatie (US-incorporated) demanding the data.

Applicable packs:
- `pack-eu-gdpr` (EU jurisdiction, tenant residence).
- `pack-us-cloud-act` (oyatie US-corporate-jurisdiction).
- `pack-de-bdsg` (German federal data-protection law).

Pack precedence:
- GDPR Article 48 forbids transfer pursuant to non-EU court order
  unless via international agreement (MLAT).
- BDSG §3 enforces GDPR + adds German-specific protections.
- CLOUD Act compels disclosure of US-controlled data under valid
  US process.

Substrate decision:
1. Check data-residency hard-stop. The data is pinned to EU-
   Sovereign cell per pack-eu-gdpr.
2. Apply higher-restriction-wins. GDPR Article 48 is the higher
   restriction.
3. Refuse the CLOUD Act demand at the substrate level. Emit
   `CrossBorderTransferDenied` event class.
4. Surface the request to the legal-council axis for MLAT routing
   per the EU-US judicial cooperation agreement.
5. Issue a transparency report citing pack-eu-gdpr Article 48 +
   pack-us-cloud-act + the MLAT routing decision.
6. Notify the tenant via the tenancy substrate that a transparency
   event occurred (warrant-canary preserved per ADR-0297 + ADR-
   0300).

#### §B.2.b. KR-PIPA × US subpoena

Scenario: a tenant operating in South Korea has user data stored
in the KR-Pack cell (per ADR-0240). A US plaintiff issues a US
subpoena seeking the data.

Applicable packs:
- `pack-kr-pipa` (KR jurisdiction, tenant residence).
- `pack-us-subpoena` (US plaintiff source).

Pack precedence:
- PIPA Article 39-12 forbids cross-border personal-data transfer
  without explicit user consent + receiving-jurisdiction
  equivalent-protection.
- US subpoena (FRCP §45) compels production where the producing
  party is subject to US jurisdiction.

Substrate decision:
1. Check data-residency hard-stop. Data is pinned to KR-Pack cell.
2. PIPA Article 39-12 is the higher restriction.
3. Refuse the US subpoena demand at the substrate.
4. Surface the request to legal-council for KR-US MLAT routing.
5. Emit `JurisdictionConflictResolved` + transparency report.

#### §B.2.c. CN-PIPL × EU GDPR

Scenario: a tenant operating in the European Union has users who
are Chinese citizens. The tenant must comply with both EU GDPR
(for EU operations) and CN PIPL (for Chinese-citizen users).

Applicable packs:
- `pack-eu-gdpr`.
- `pack-cn-pipl`.

Pack precedence:
- PIPL Article 38 + 53 require CAC security assessment for cross-
  border personal-data transfer outside China.
- GDPR Article 6 requires lawful basis; GDPR Article 44-49 require
  adequacy or SCC for transfer to non-adequate jurisdictions.
- Neither pack has adequacy decision for the other.

Substrate decision:
1. Check data-residency hard-stop. The substrate routes Chinese-
   citizen user data through the CN-Pack cell (per the user's
   citizenship pack assignment); EU-citizen user data through the
   EU-Sovereign cell.
2. Within the tenant's cross-citizen aggregations (e.g., a global
   user-base dashboard), the substrate enforces multi-pack
   alignment: aggregations must use anonymized data per both packs'
   anonymization standard, OR the aggregation is refused.
3. Cross-citizen messaging between an EU citizen + a CN citizen
   uses E2EE per ADR-0255 + ADR-0300; the metadata layer is
   minimized per both packs.
4. Emit transparency report on every cross-pack data flow.

### §B.3. The transparency-report invariant — every consequential request

Every consequential request that involves multi-pack evaluation
emits a `TransparencyReportEmittedEvent` audit-event-class. The
report contains:

- `request_id` (correlation key).
- `applicable_pack_ids` (the set of packs consulted).
- `precedence_decision` (which pack's rule won the conflict, with
  pack-specific legal-cite).
- `cell_id` (which cell processed the request, satisfying per-pack
  data-residency).
- `data_class` (per ADR-0099 data-class registry).
- `cross_border_egress` (boolean).
- `multi_pack_alignment_present` (boolean).
- `legal_council_routing_required` (boolean; true for MLAT).
- `tenant_notification_sent` (boolean; warrant-canary preserved).
- `audit_chain_anchor` (per ADR-0028 Merkle-sealed anchor).

The report is queryable by the tenant via the tenancy substrate's
transparency-report surface per ADR-0244. The tenant sees the
aggregated reports for their own surfaces; cross-tenant
visibility is blocked.

Per ADR-0263, the report emission is mandatory (it is the audit
surface; not emitting is a compliance failure).

## §C. Consequences

### §C.1. Maintainability dimension

The cross-jurisdiction conflict resolution baseline scales across
130+ jurisdictional packs + 50+ sectoral packs. Maintainability
invariants:

- **Per-pack precedence DAG is data, not code.** Each pack's
  `precedence.yaml` declares its restriction-axis maxima +
  per-rule precedence; the substrate consumes the DAG via the
  per-pack registry. No code change required to add a new pack.
- **Per-tenant pack roster is configuration.** Tenants declare
  their applicable pack roster via the tenancy substrate per
  ADR-0244. No code change.
- **Per-µservice pack roster overlay.** Each µservice declares
  its baseline pack roster via the manifest; per-tenant overrides
  compose at request-time.
- **Versioning policy.** Per-pack precedence DAGs follow
  semantic-versioning per ADR-0258; major version bumps require
  legal-council axis sign-off per ADR-0294.
- **Pack-conflict regression CI.** The substrate's per-pack
  conflict-resolution test set includes >1000 worked examples
  across canonical pack-pair conflicts; the CI lane
  `oya-governance-pack-precedence-regression` blocks any
  PR that breaks an existing conflict resolution.
- **Documentation density.** Each µservice's PRD MUST cite which
  packs apply, which conflict patterns are anticipated, and
  which transparency-report surfaces are exposed.

### §C.2. Observability dimension

Per ADR-0263 observability emission contract:

- **Audit-event-classes (registered in ADR-0263 registry):**
  - `JurisdictionConflictResolved` — every multi-pack conflict
    resolution. Carries: request_id, applicable_pack_ids[],
    precedence_decision_pack_id, conflict_axis, resolution_rule.
  - `DataResidencyHardStopEnforced` — every hard-stop. Carries:
    request_id, requested_cell, required_cell, applicable_pack,
    cross_border_attempted (boolean).
  - `CrossBorderTransferDenied` — every cross-border denial.
    Carries: request_id, source_cell, target_cell, conflicting
    packs[], denial_reason.
  - `TransparencyReportEmitted` — every transparency-report
    emission. Carries: request_id, report_id, applicable_pack_ids[],
    tenant_notification_sent (boolean).
  - `PackPrecedenceGraphUpdated` — every per-pack precedence-graph
    update. Carries: pack_id, version_before, version_after,
    council_sign_off_attestation.
  - `MLATRoutingInitiated` — every legal-council MLAT routing.
    Carries: request_id, requesting_jurisdiction, receiving_
    jurisdiction, legal_basis.
- **Metrics (per ADR-0263 cardinality budget):**
  - `oya_jurisdiction_conflict_resolved_counter` — total
    resolutions. Dimensions: applicable_pack_pair_id, precedence_
    decision_pack_id.
  - `oya_jurisdiction_data_residency_hard_stop_counter` — total
    hard-stops. Dimensions: required_cell_pack, source_µservice.
  - `oya_jurisdiction_cross_border_denied_counter` — total denials.
    Dimensions: source_pack, target_pack, denial_reason_class.
  - `oya_jurisdiction_transparency_report_emitted_counter` —
    total report emissions. Dimensions: tenant_bucket, applicable_
    pack_pair_id.
  - `oya_jurisdiction_mlat_routing_initiated_counter` — total
    MLAT routings. Dimensions: requesting_jurisdiction, receiving_
    jurisdiction.
- **Dashboards:** every µservice with multi-pack exposure ships
  `dashboards/jurisdiction-conflict.json` with the canonical
  9-panel layout (conflict-resolution rate, data-residency
  hard-stop count, cross-border denial rate, transparency-report
  emission rate, MLAT routing volume, per-pack-pair-id breakdown,
  per-µservice exposure, per-tenant exposure, audit-chain anchor
  rate). Dashboard naming follows ADR-0263.

### §C.3. Scalability dimension

The substrate scales horizontally to 130+ packs + 50+ sectoral
overlays:

- **Cell-pinning state is per-tenant.** State volume is O(tenants
  × per-tenant pack roster size); bounded ≤ 50 packs per tenant
  typical; ≤ 500 bytes per pack-tenant pair; per-cell ≤ 50 MB for
  100k tenants — within budget.
- **Per-pack precedence DAG is shared cache.** O(packs²) at worst-
  case for pairwise conflicts; ≤ 100 KB per pack-pair-cache entry;
  per-cell ≤ 5 GB for 130 × 130 pack pair cache — within budget.
- **Hot-path performance.** Conflict-resolution check is a cell-
  local hash lookup against the cached precedence DAG; O(1) per
  rule + O(applicable_packs) tree-walk. Target p99 latency ≤ 5 ms.
- **Transparency-report emission latency.** Async to the request-
  response path; written to ADR-0263 audit-chain via the central
  emission contract.
- **Cross-cell coherence.** Per-pack precedence DAGs are versioned
  + propagated via the substrate's policy-engine substrate per
  ADR-0246 ≥60s soak + signed publication. Cell-to-cell coherence
  is eventual-consistent with bounded staleness ≤ 120s per
  ADR-0294.
- **Burst capacity.** Per-pack registry caches the precedence DAG
  in-memory; reads are O(1); no external service dependency on
  the hot-path.

### §C.4. Performance dimension

- **Conflict-resolution evaluation latency.** p50 ≤ 1 ms; p99 ≤
  5 ms; p99.9 ≤ 20 ms. Measured at the per-µservice gate.
- **Transparency-report build + emission latency.** p50 ≤ 50 ms
  (async to request); p99 ≤ 200 ms; p99.9 ≤ 2 s.
- **Per-pack precedence DAG load latency at startup.** ≤ 1 s per
  cell for 130 packs.
- **CPU budget per request.** ≤ 30 μs CPU including the
  precedence-graph walk + the data-residency check. The substrate
  fits within the 90-μs total Cedar evaluation budget per ADR-0243.
- **Memory budget.** ≤ 1 KB per active tenant pack-roster lookup;
  ≤ 5 GB per cell for the per-pack precedence DAG; ≤ 100 MB per
  cell for the active per-tenant transparency-report queue.

### §C.5. Optimization dimension

The substrate provides optimizations that per-µservice implementations
would miss:

- **Per-tenant pack-roster pre-computation.** The substrate
  pre-computes the applicable-pack set per tenant at tenant-
  registration time; the hot-path is a single hash lookup.
- **Pre-computed pack-pair precedence cache.** The substrate
  pre-computes the precedence decision for every pack-pair at
  registration time + caches; the hot-path is O(applicable_packs)
  tree-walk against the pre-computed cache.
- **MLAT routing batching.** Multiple law-enforcement requests
  for the same tenant + same jurisdiction within a 24-hour window
  are batched + surfaced as a single legal-council notification.
- **Transparency-report aggregation.** Per-tenant transparency-
  report queries are aggregated daily into per-tenant transparency
  dashboards; the per-request emissions feed the dashboard.
- **Cross-pack rule deduplication.** When packs share rules
  (e.g., GDPR + UK-GDPR share most rules), the substrate
  deduplicates the rule evaluation.
- **Cell-pin route hint cache.** Per-tenant cell-pin route hints
  are cached at the edge gateway per ADR-0149; requests route
  directly to the correct cell without cross-cell hop.

### §C.6. Code quality dimension

- **Single ingress trait.** Per-µservice integration uses one
  trait `JurisdictionConflictGate::resolve_or_deny()`; no µservice
  authors its own per-pack logic.
- **No `#[cfg(test)]` bypass paths.** The substrate's Cedar gate
  evaluates in test as in prod.
- **Mandatory documentation block.** Every µservice with multi-
  pack exposure MUST include a `compliance.md §cross-jurisdiction-
  edge-cases` section per the §3.2.5 row-coverage requirement.
- **Deterministic test fixtures.** The shared crate ships test
  fixtures for canonical conflict scenarios (GDPR × CLOUD-Act,
  PIPA × subpoena, PIPL × GDPR, HIPAA × PIPA, KOSA × GDPR, etc.).
- **Pack-versioning enforcement.** Per-pack precedence DAGs are
  versioned per ADR-0258 SemVer + signed per ADR-0294 fragment-
  lifecycle.
- **Property-based test coverage.** The shared crate ships
  property-based tests (proptest crate) for the precedence-graph
  acyclicity + total-orderability + higher-restriction-wins
  determinism.
- **Audit-event-class registration enforcement.** New event
  classes added must be registered in the ADR-0263 central
  registry.

## §D. Detailed mechanics

### §D-1. Data-residency hard-stop — full mechanics

The data-residency hard-stop is the strongest substrate primitive
— no other rule can override.

**Trigger conditions:**

- **Cross-cell read.** A µservice on cell A reads data pinned to
  cell B where B's pack roster includes a residency-restricted
  pack (e.g., pack-eu-gdpr, pack-cn-pipl, pack-kr-pipa,
  pack-ru-pdl, pack-in-dpdpa).
- **Cross-cell write.** A write request targets a cell whose pack
  roster restricts cross-jurisdiction writes.
- **Cross-cell replication.** A backup or DR-replica targets a
  cell outside the source's pack-allowed cell set.
- **Cross-cell streaming.** A real-time stream feeds data from
  cell A into cell B where the pack roster disallows it.
- **Cross-cell aggregation.** A multi-cell aggregation query
  consumes data from cells whose pack rosters disagree.

**Enforcement:**

The substrate checks the per-pack `cell_pin` declaration before
each request:

1. Identify the data-class per ADR-0099 (PHI, PII, payment-PAN,
   biometric-data, financial-record, etc.).
2. Identify the applicable pack roster per the data's tenant + the
   pack's data-class coverage.
3. Identify the target cell per the µservice's routing decision.
4. Check `target_cell.pack_roster ⊇ data.pack_roster` (the target
   cell's pack roster must encompass the data's pack roster).
5. If not, emit `DataResidencyHardStopEnforced` + return 451
   (Unavailable For Legal Reasons) per RFC 7725 OR 403 Forbidden
   per RFC 7231 (mediated by tenant preference).

**Exceptions (the only override paths):**

- Emergency-services per ADR-0298. A user dialing 911 may have
  PII traverse cell boundaries to reach the local PSAP.
- Court-order via MLAT. A duly-processed MLAT request through the
  legal-council axis may override; the override is audited +
  transparency-reported.
- User-explicit consent per pack-allowed paths (e.g., GDPR Art.
  49 derogations). The user's explicit consent + pack-allowed
  derogation enable specific cross-jurisdiction flows.

**State machine:**

```
┌──────────┐ data read/write request    ┌──────────────────────┐
│ Idle      │ ──────────────────────────▶│ ResidencyCheckActive │
│           │                            │ - data_class          │
│           │                            │ - applicable_packs[]  │
│           │                            │ - source_cell         │
│           │                            │ - target_cell         │
│           │ ◀───────────────────────── │                       │
│           │  pack_roster_satisfied     └─────────┬─────────────┘
│           │                                       │
│           │                                       │ pack_roster_violated
│           │                                       ▼
│           │                              ┌────────────────────┐
│           │ ◀── HardStopEnforced ────────│ HardStopEmitted    │
│           │                              │ (return 451 / 403) │
│           │                              └────────────────────┘
└──────────┘
```

### §D-2. Per-pack precedence graph — full mechanics

The per-pack precedence graph is a directed acyclic graph (DAG)
encoding pairwise precedence between packs across restriction axes.

**DAG structure:**

```yaml
# packs/eu-gdpr/precedence.yaml
pack_id: pack-eu-gdpr
version: 1.4.2
restriction_axes:
  data_subject_rights:
    level: 9
    floor:
      right_to_erasure: required_within_30d_per_art_17
      right_to_portability: required_per_art_20
      right_to_object: required_per_art_21
      right_to_access: required_per_art_15
  cross_border_transfer:
    level: 9
    floor:
      lawful_basis: required_per_art_6
      adequacy_or_scc: required_per_art_44_49
      bcrs_permitted: yes
      court_order_outside_eu: forbidden_per_art_48
  sensitive_data:
    level: 8
    floor:
      art_9_special_category: explicit_consent_or_art_9_2
  consent_grain:
    level: 8
    floor:
      purpose_specific: required_per_art_4_11
      freely_given: required
      withdrawable: required
  ai_transparency:
    level: 7
    floor:
      art_22_automated_decisions: human_oversight_required
precedence_relations:
  - over: pack-us-cloud-act
    on_axis: cross_border_transfer
    rule: art_48_disallows_compliance_without_mlat
  - over: pack-us-subpoena
    on_axis: cross_border_transfer
    rule: art_44_requires_adequacy_or_scc
  - parity_with: pack-uk-gdpr
    on_all_axes: substantive_equivalence_per_adequacy_2021
  - under: pack-hipaa
    on_axis: phi_handling
    rule: hipaa_supersedes_for_phi_in_us_jurisdiction
  - parity_with: pack-eu-aiact
    on_axis: ai_transparency
    rule: aiact_extends_gdpr_art_22
```

**Precedence-graph evaluation:**

Given a set `P` of applicable packs + a request `(action, resource,
data_class)`:

1. For each axis on which any `p ∈ P` declares a floor, collect
   the set of floor values.
2. For each axis, compute the higher-restriction floor (max
   level + max value).
3. The resolved decision is the conjunction of all axis floors.

Example: for the GDPR × CLOUD-Act conflict on `cross_border_
transfer`:

- pack-eu-gdpr declares `level=9, floor.court_order_outside_eu =
  forbidden_per_art_48`.
- pack-us-cloud-act declares `level=9, floor.govt_request =
  comply_with_valid_process`.

The substrate evaluates the precedence relation: pack-eu-gdpr
`over: pack-us-cloud-act on_axis: cross_border_transfer`. The
GDPR rule wins; the CLOUD-Act demand is refused at the substrate.

### §D-3. Multi-pack alignment — full mechanics

Multi-pack alignment is required for cross-border transfers that
cross more than one pack's residency floor.

**Alignment criteria:**

A cross-border transfer is **aligned** if all of the following hold:

- **Lawful basis present in source pack.** The source pack's
  lawful-basis requirement is satisfied (e.g., GDPR Art. 6 lawful
  basis).
- **Adequacy or equivalent in target pack.** The target pack's
  receiving-jurisdiction-adequacy is satisfied (e.g., GDPR
  adequacy decision, EU-US Data Privacy Framework).
- **Standard contractual clauses if applicable.** If the target
  pack lacks adequacy, SCCs per the source pack's requirements
  must be in place.
- **User explicit consent if required.** Some packs (KR-PIPA Art.
  39-12) require explicit user consent.
- **CAC security assessment if required.** CN-PIPL Art. 38 requires
  CAC assessment for transfers ≥ 100k subjects or sensitive data.
- **Audit trail present.** The transfer is logged in the
  audit-chain per ADR-0028 + ADR-0263.

**Non-aligned transfer denial:**

If any criterion fails, the substrate denies the transfer + emits
`CrossBorderTransferDenied` + may surface the request to the
legal-council axis for human review.

**Tenant-attested alignment:**

Tenants may pre-declare cross-border alignment via the tenancy
substrate: e.g., a multi-national tenant operating in both EU + US
may declare its global SCC arrangement; the substrate consumes the
declaration + uses it on the hot-path.

### §D-4. Cedar policy fragment — `policy/jurisdiction-conflict.cedar`

The canonical Cedar fragment:

```cedar
// policy/jurisdiction-conflict.cedar
// Per-µservice Cedar fragment per ADR-0304 + ADR-0243 +
// ADR-0294 fragment-lifecycle.

// Default-deny: cross-jurisdiction request refused unless
// precedence + alignment satisfied.

forbid (
  principal,
  action in [
    Action::"read_data",
    Action::"write_data",
    Action::"replicate_data",
    Action::"stream_data",
    Action::"aggregate_cross_cell"
  ],
  resource
)
when {
  // Predicate 1: data-residency hard-stop
  context.target_cell_pack_roster_does_not_satisfy_data_pack_roster ||
  // Predicate 2: cross-border transfer not aligned
  (context.cross_border_transfer == true &&
   !context.multi_pack_alignment_present) ||
  // Predicate 3: higher-restriction pack forbids
  context.higher_restriction_pack_forbids == true
};

// Emergency-services exception per ADR-0298
permit (
  principal,
  action in [
    Action::"emergency_services_data_route",
    Action::"crisis_hotline_session_route"
  ],
  resource
)
when {
  context.emergency_path_attested == true
};

// MLAT-routed exception
permit (
  principal,
  action == Action::"comply_mlat_routed_request",
  resource
)
when {
  context.mlat_routing_completed == true &&
  context.legal_council_attestation_present == true &&
  context.transparency_report_will_emit == true
};

// User-explicit-consent derogation (e.g., GDPR Art. 49)
permit (
  principal,
  action == Action::"transfer_under_derogation",
  resource
)
when {
  context.user_explicit_consent_attested == true &&
  context.pack_allows_user_derogation == true &&
  context.user_consent_audit_present == true
};
```

**Cedar context attributes:**

- `context.target_cell_pack_roster_does_not_satisfy_data_pack_roster`
  — boolean; computed by §D-1.
- `context.cross_border_transfer` — boolean.
- `context.multi_pack_alignment_present` — boolean; computed by
  §D-3.
- `context.higher_restriction_pack_forbids` — boolean; computed by
  §D-2 precedence-graph evaluation.
- `context.emergency_path_attested` — boolean per ADR-0298.
- `context.mlat_routing_completed` — boolean.
- `context.legal_council_attestation_present` — boolean.
- `context.transparency_report_will_emit` — boolean.
- `context.user_explicit_consent_attested` — boolean.
- `context.pack_allows_user_derogation` — boolean.
- `context.user_consent_audit_present` — boolean.

### §D-5. Per-cell-tier variants

Per ADR-0248:

- **Tier-0 cells (edge POPs).** Edge enforces per-IP geo-location
  routing to the correct cell pack (e.g., a request from a German
  IP routes to the EU-Sovereign cell). The edge does NOT process
  consequential data.
- **Tier-1 cells (regional control planes).** Hosts the substrate's
  per-pack precedence-graph cache + the cell-pin registry.
- **Tier-2 cells (data plane regions).** Hosts µservice instances
  + Cedar fragment evaluation.
- **Tier-3 cells (compliance-isolated).** Per-pack hard-stop at
  the cell boundary; cross-cell traffic forbidden except via
  pre-attested cross-pack alignment.
- **Tier-4 cells (sovereign-cloud).** Maximum hard-stop;
  cross-cell traffic forbidden absolutely except emergency-
  services per ADR-0298.

### §D-6. Per-tenant audience-type tuning

| Audience type | Default pack roster | Cross-pack default |
|---|---|---|
| `B2C_CONSUMER` | per-user-geo | refused without explicit consent |
| `B2B_TENANT` | per-tenant-HQ + per-user-geo | per-tenant SCC if declared |
| `HIGH_RISK_USER` (activist) | refused | refused absolutely |
| `MINOR_PII` | per-pack child-protection + per-geo | refused (parental consent required) |
| `SOVEREIGN_GOV_TENANT` | per-jurisdiction-sovereign-pack | refused absolutely |
| `FRIENDLY_CRAWLER_PARTNER` | per-pack data-class-allowed | per-pack defined |

### §D-7. Observability — metrics, dashboards, audit-event-classes

Per ADR-0263:

**Audit-event-classes:**

- `JurisdictionConflictResolved`
- `DataResidencyHardStopEnforced`
- `CrossBorderTransferDenied`
- `TransparencyReportEmitted`
- `PackPrecedenceGraphUpdated`
- `MLATRoutingInitiated`
- `MLATRoutingCompleted`
- `UserConsentDerogationGranted`

**Metrics:**

| Metric | Dimensions | Cardinality bound |
|---|---|---:|
| `oya_jurisdiction_conflict_resolved_counter` | pack_pair_id, decision_pack | 5K |
| `oya_jurisdiction_data_residency_hard_stop_counter` | cell_pack, µservice | 2K |
| `oya_jurisdiction_cross_border_denied_counter` | source_pack, target_pack | 2K |
| `oya_jurisdiction_transparency_report_emitted_counter` | tenant_bucket, pack_pair | 5K |
| `oya_jurisdiction_mlat_initiated_counter` | requesting_juris, receiving_juris | 200 |
| `oya_jurisdiction_user_derogation_granted_counter` | pack_id, audience_type | 500 |

Aggregate ≤ 15K per cell.

**Dashboard:** 9-panel canonical layout per §C.2.

### §D-8. Per-tenant jurisdictional preference

Tenants declare their primary jurisdiction via the tenancy substrate
per ADR-0244. The substrate uses the preference to:

- Route the tenant's user data to the preferred-jurisdiction cell
  when consistent with per-pack data-residency.
- Apply the preferred-jurisdiction pack's defaults to ambiguous
  pack-conflict decisions.
- Surface transparency reports in the preferred-jurisdiction
  language.
- Pre-compute the applicable-pack set for the tenant.

The preference does NOT override per-pack regulator floors; the
higher-restriction-wins rule still applies.

### §D-9. Compliance interactions

- **EU GDPR Article 48.** Refuses non-EU court-order disclosure
  without MLAT; the substrate routes to legal-council.
- **US CLOUD Act.** The substrate complies via MLAT when the data
  is foreign-pack-pinned; complies directly when the data is
  US-pack-pinned + the process is valid.
- **KR-PIPA Article 39-12.** Cross-border transfer requires
  explicit consent + receiving-jurisdiction adequacy.
- **CN-PIPL Article 38.** CAC security assessment for transfers
  out of China.
- **EU AI Act Article 27 + Article 53.** AI-system data-residency
  + cross-border-training-data alignment.
- **HIPAA + GDPR.** HIPAA-protected PHI in EU is double-protected;
  higher-restriction-wins applies on every axis.
- **PCI-DSS + GDPR.** Cross-border PCI data flows require both
  GDPR + PCI compliance; multi-pack alignment.
- **EU AADC + COPPA + KOSA.** Cross-jurisdiction minor-PII
  protection compounds per ADR-0292 + this ADR.

## §E. Implementation footprint

### §E.1. New crate

```
oya-shared-jurisdiction-conflict/
├── Cargo.toml                            # workspace crate, single-concern
├── src/
│   ├── lib.rs                            # JurisdictionConflictGate trait
│   ├── precedence_graph/
│   │   ├── mod.rs                        # precedence-graph submodule
│   │   ├── dag.rs                        # DAG validator + walker
│   │   ├── axis.rs                       # restriction-axis enum + levels
│   │   └── relation.rs                   # over/under/parity-with relations
│   ├── data_residency/
│   │   ├── mod.rs                        # data-residency submodule
│   │   ├── cell_pin_registry.rs          # per-tenant cell-pin registry
│   │   ├── hard_stop.rs                  # hard-stop enforcer
│   │   └── pack_roster_satisfaction.rs
│   ├── multi_pack_alignment/
│   │   ├── mod.rs                        # alignment submodule
│   │   ├── validator.rs                  # alignment criteria validator
│   │   ├── scc.rs                        # SCC handling
│   │   └── derogation.rs                 # user-derogation handling
│   ├── transparency_report/
│   │   ├── mod.rs                        # transparency-report submodule
│   │   ├── builder.rs                    # per-request builder
│   │   ├── signer.rs                     # signed report
│   │   └── tenant_query.rs               # per-tenant query surface
│   ├── mlat_routing/
│   │   ├── mod.rs                        # MLAT routing submodule
│   │   ├── council_notifier.rs           # legal-council notifier
│   │   └── jurisdiction_registry.rs      # per-jurisdiction MLAT registry
│   ├── pack_registry/
│   │   ├── mod.rs                        # per-pack registry submodule
│   │   ├── loader.rs                     # YAML pack loader
│   │   └── cache.rs                      # in-memory cache
│   ├── cedar_fragment/
│   │   ├── mod.rs                        # Cedar fragment helpers
│   │   ├── context_builder.rs            # request-context builder
│   │   └── evaluator.rs                  # invokes ADR-0243 Cedar
│   ├── audit/
│   │   ├── mod.rs                        # audit-event-class emission
│   │   ├── event_class.rs
│   │   └── emit.rs
│   ├── observability/
│   │   ├── mod.rs                        # metrics + dashboards
│   │   ├── metrics.rs
│   │   └── tracing.rs
│   └── error.rs                          # canonical errors
├── tests/
│   ├── precedence_graph_property.rs      # property-based tests
│   ├── gdpr_cloudact_conflict.rs         # worked example test
│   ├── kr_pipa_subpoena_conflict.rs      # worked example test
│   ├── cn_pipl_gdpr_conflict.rs          # worked example test
│   ├── hipaa_gdpr_conflict.rs            # worked example test
│   ├── multi_pack_alignment.rs           # alignment criteria tests
│   ├── data_residency_hardstop.rs        # hard-stop tests
│   ├── transparency_report.rs            # report-builder tests
│   └── fixtures/
│       ├── pack_fixtures.rs
│       ├── tenant_fixtures.rs
│       └── conflict_scenarios.rs
└── docs/
    ├── README.md
    ├── ARCHITECTURE.md
    ├── usage.md
    ├── pack-authoring.md                 # how to add a new pack
    └── conflict-scenarios.md
```

### §E.2. New µservice extensions

Every µservice with multi-pack exposure extends with:

```
microservices/<name>/
├── policy/
│   ├── jurisdiction-conflict.cedar       # Cedar fragment per §D-4
│   └── jurisdiction-conflict-overlays/
│       └── <pack>.cedar                  # per-pack overlays
├── iac/
│   ├── dev-pack-residency.yaml
│   ├── staging-pack-residency.yaml
│   └── prod-pack-residency.yaml
├── docs/
│   ├── ARCHITECTURE.md                   # +§jurisdiction-conflict
│   ├── PRD.md                            # +§cross-jurisdiction-edge-cases
│   ├── compliance.md                     # +§cross-jurisdiction per §3.2.5 row 23
│   └── runbooks/
│       ├── jurisdiction-conflict-mlat-routing.md
│       ├── jurisdiction-conflict-pack-precedence-stale.md
│       └── jurisdiction-conflict-transparency-report-stuck.md
├── tests/
│   └── jurisdiction_conflict_contract.rs
├── dashboards/
│   └── jurisdiction-conflict.json
└── slos/
    └── jurisdiction-conflict-latency.openslo.yaml
```

### §E.3. New runbooks

- `jurisdiction-conflict-mlat-routing.md` — resolve MLAT routing
  failures.
- `jurisdiction-conflict-pack-precedence-stale.md` — resolve stale
  pack precedence cache.
- `jurisdiction-conflict-transparency-report-stuck.md` — resolve
  transparency-report emission failures.
- `jurisdiction-conflict-cross-border-denied-investigation.md` —
  investigate denied cross-border transfers.

### §E.4. New CI lanes

- `oya-governance-data-residency-hardstop` — verifies hard-stop
  declarations per µservice.
- `oya-governance-pack-precedence-graph` — verifies DAG
  acyclicity + total-orderability + conflict-pair coverage.
- `oya-governance-multi-pack-alignment` — verifies cross-border
  alignment recorded.
- `oya-governance-transparency-report` — verifies report
  emission coverage.
- `oya-governance-jurisdiction-conflict` — aggregate roll-up.

### §E.5. Vendor selection rationale

- **Cell-pinning** — per ADR-0240 + ADR-0248 cell-tiered substrate;
  no vendor outside the substrate.
- **MLAT routing** — surfaces to legal-council axis; manual
  processing per US-EU MLAT treaty (2003), US-KR MLAT (1993),
  etc.
- **Pack-yaml validation** — JSON-schema + custom validators;
  shared via the per-pack repository.
- **Audit-chain** — Merkle-sealed per ADR-0028 + ADR-0263.

## §F. Migration

### §F.1. Per-µservice rollout sequenced by jurisdictional-exposure

| Wave | Cohort | µservices | Window |
|---:|---|---|---|
| 1 | High-jurisdiction-exposure | identity, tenancy, governance, billing, finops-portal | 2026-05-30 → 2026-07-15 |
| 2 | Medium-jurisdiction-exposure | payments, marketplace, ontology, intelligence, foundry | 2026-07-15 → 2026-08-31 |
| 3 | Cross-border-data-emitting | mail, comms-email, notes, social, workflow-studio, connect | 2026-08-31 → 2026-09-30 |
| 4 | Edge + observability | api-gateway, observability, edge-gateway, ops-dashboard-control-center | 2026-09-30 → 2026-10-15 |

### §F.2. Per-µservice migration playbook

1. Add `oya-shared-jurisdiction-conflict` workspace dependency.
2. Author `policy/jurisdiction-conflict.cedar`.
3. Author `iac/<env>-pack-residency.yaml`.
4. Update `ARCHITECTURE.md` + `PRD.md` + `compliance.md`.
5. Add `dashboards/jurisdiction-conflict.json` + SLOs.
6. Add contract test.
7. Pass `oya-governance-jurisdiction-conflict`.
8. Soak ≥ 60s; promote.

### §F.3. Per-pack rollout

The top-30 packs land first:

- pack-eu-gdpr, pack-uk-gdpr, pack-us-ccpa, pack-us-cpra,
  pack-kr-pipa, pack-jp-appi, pack-cn-pipl, pack-au-privacy-act,
  pack-ca-pipeda, pack-br-lgpd, pack-in-dpdpa, pack-ru-pdl,
  pack-us-hipaa, pack-us-coppa, pack-us-finra, pack-us-sox,
  pack-us-pci-dss, pack-us-glba, pack-us-fcra, pack-us-cloud-act,
  pack-eu-aiact, pack-eu-aadc, pack-eu-dora, pack-eu-nis2,
  pack-eu-eidas2, pack-eu-dma, pack-eu-dsa, pack-kr-fsc,
  pack-jp-fsa, pack-uk-fca.

### §F.4. What is NOT migrated

- Per-tenant UI for pack-roster declaration is the tenancy
  substrate per ADR-0244.
- Per-pack legal-substantive interpretation is the per-pack
  legal-council axis.
- MLAT bilateral treaties are the legal-council's domain.

### §F.5. Rollback path

- Cell-tier rollback: `oya policy revert jurisdiction-conflict-v1`.
- µservice rollback: revert `policy/jurisdiction-conflict.cedar`.
- Soft-disable: emergency feature flag in the per-µservice IaC
  manifest; cross-pack denial reverts to permit-with-audit.
- Hard-disable: drop the workspace dependency; substrate
  primitive becomes per-µservice responsibility (not recommended
  except in disaster scenarios).

## §G. References

### §G.1. Hyperscaler precedents

- Microsoft Azure Sovereign Cloud documentation 2024.
- Microsoft Transparency Report H1-2024.
- AWS GovCloud + AWS Transparency Report H1-2024.
- Google Cloud Assured Workloads documentation 2024.
- Salesforce Hyperforce documentation 2024.
- Workday Sovereign Cloud documentation 2024.
- Oracle Sovereign Cloud Region documentation 2024.
- Box Zones + Slack EKM documentation 2024.
- Cloudflare Data Localization Suite documentation 2024.
- Microsoft v. Ireland US Supreme Court (2018).

### §G.2. Standards + RFCs

- EU GDPR (Regulation 2016/679) — Articles 6, 17, 21, 22, 44-49.
- UK Data Protection Act 2018 + UK-GDPR.
- US CLOUD Act (18 USC §2701-13).
- US Federal Rules of Civil Procedure §45 (subpoena).
- US Federal Rules of Civil Procedure §37(e) (litigation hold).
- US Stored Communications Act (18 USC §2701-13).
- KR-PIPA (개인정보 보호법) — Articles 28, 39-12.
- CN-PIPL (个人信息保护法) — Articles 38, 53.
- CN-DSL (数据安全法).
- CN-CSL (网络安全法).
- JP-APPI (個人情報保護法).
- AU Privacy Act 1988 (amended 2022).
- CA-PIPEDA + Quebec Law 25.
- BR-LGPD (Lei Geral de Proteção de Dados, 2018).
- IN-DPDPA (Digital Personal Data Protection Act, 2023).
- US-CCPA + CPRA.
- HIPAA + HITECH.
- PCI-DSS v4.0.
- EU AI Act (Regulation 2024/1689).
- RFC 7725 — 451 Unavailable For Legal Reasons.
- RFC 7231 §6.5.3 — 403 Forbidden.
- RFC 3339 — Timestamps.

### §G.3. Legal + compliance

- EU-US Data Privacy Framework (DPF, 2023-07-10).
- CJEU Case C-311/18 (Schrems II).
- CJEU Case C-362/14 (Schrems I).
- US-EU MLAT (2003).
- US-KR MLAT (1993).
- US-JP MLAT (1980, amended 2003).
- Hague Convention on Service Abroad of Judicial and
  Extrajudicial Documents (1965).
- EU Commission adequacy decisions (UK, Switzerland, Andorra,
  Argentina, Canada, Faroe Islands, Guernsey, Israel, Isle of
  Man, Japan, Jersey, New Zealand, Republic of Korea, Uruguay).

### §G.4. Internal portfolio ADRs

- ADR-0028 Audit Chain (Merkle-sealed).
- ADR-0099 Data Class Registry.
- ADR-0105 Thirteen-Layer Canonical Enum.
- ADR-0131 Per-µservice Flat Layout.
- ADR-0140 Cedar Policy Enforcement.
- ADR-0145 Inter-Microservice Communication Reform.
- ADR-0212 Buildability Doctrine.
- ADR-0240 Sovereign-Cloud per Regional Pack.
- ADR-0242 Oyatie is a Tenant Doctrine.
- ADR-0243 Cedar as Universal Gate.
- ADR-0244 Tenant as Universal Scoping Primitive.
- ADR-0245 Substrate vs Product Layering.
- ADR-0246 Policy Engine Substrate Promotion.
- ADR-0248 Amazon-Shape Cellular Architecture.
- ADR-0250 Build Ahead of Certification Doctrine.
- ADR-0251 Compliance Pack — Cell Certification Levels.
- ADR-0258 API Versioning + SemVer Policy.
- ADR-0263 Observability Emission Contract.
- ADR-0272 Cookie Consent per Purpose.
- ADR-0276 Backup Portability per GDPR Article 20.
- ADR-0292 Minor User Doctrine.
- ADR-0294 Cedar Fragment Lifecycle.
- ADR-0297 Abuse-Defence Baseline.
- ADR-0298 Emergency-Services Bypass Doctrine.
- ADR-0299 Account-Recovery Resilience.
- ADR-0300 Whistleblower + Press-Freedom Anonymity.
- ADR-0301 Survivor-Safety Domestic-Abuse Mode.
- ADR-0302 Deceased-User Inheritance Doctrine.
- ADR-0303 Cognitive-Impairment Decision-Resilience.
- ADR-0305 Delegated-Agent Authority Chain.
- ADR-0306 Disaster-Mode + Cell Resilience.

### §G.5. Standards docs

- `docs/standards/documentation-rigor.md` §3.2.5 row 23.
- `docs/standards/doc-style.md`.
- `docs/templates/adr-template-v2.md`.

### §G.6. Auto-memory feedback (related)

- feedback_quality_performance_scalability_bar
- feedback_clean_architecture_requirements
- feedback_no_silent_regression
- feedback_autonomous_implementation_artifacts
- feedback_canonical_base_localization
- feedback_oyatie_is_a_tenant_doctrine
- feedback_cedar_as_universal_gate
- feedback_amazon_shape_cellular_architecture
- feedback_compliance_pack_primitive
- feedback_naming_justification

## §H. Change log

- **2026-05-20** — Initial proposal. Bundled with keystone-bundle
  2026-05-20 foundational doctrine synthesis as the critical-path-
  cluster-cross-jurisdiction-conflict-resolution keystone. Closes
  documentation-rigor.md §3.2.5 row 23. Enforcement advisory until
  2026-10-15, BLOCKER thereafter.

---

End of ADR-0304.
