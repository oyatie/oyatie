---
id: ADR-0306
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - council-product
  - council-reliability
  - council-legal
  - ops-sre-reliability
  - ops-incident-response
  - ops-trust-and-safety
  - ops-compliance
  - axis-edge
  - axis-network
  - axis-cell-topology
  - axis-policy-engine
  - axis-audit-chain
  - axis-tenancy
  - axis-data-residency
supersedes: []
amends: []
superseded_by: [ADR-707]
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0044-service-mesh-and-mtls.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0188-passkey-webauthn-as-canonical-auth.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-disaster-recovery-dr-pair-strategy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0253-network-topology-edge-service-mesh.md
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
  - ADR-0304-cross-jurisdiction-conflict-resolution.md
  - ADR-0305-delegated-agent-authority-chain.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/observability.json
  - /specs/microservices/edge-gateway.json
  - /specs/microservices/api-gateway.json
  - /specs/microservices/cell.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/disaster-mode-controls.json
  - /specs/offline-first-sync-schema.json
  - /specs/compliance-pack-schema.json
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
keystone_position: critical-path-cluster-disaster-mode-cell-resilience
purpose: >
  Establish the Disaster-Mode + Cell-Resilience doctrine — a
  substrate-level primitive that addresses disaster-zone surge
  handling (10× normal traffic), offline-first sync (CRDT per
  `oya-collab-crdt-portability-kernel`), progressive enhancement,
  per-cell DR-pair failover (per ADR-0241), per-pack disaster-mode
  rules, cell-isolation per ADR-0248 preserved across degradation,
  graceful per-tenant SLO degradation, and the absolute invariant
  that emergency-services (per ADR-0298) NEVER throttle even during
  the most severe disaster. The bar is: during a mass-casualty
  incident or regional outage, oyatie's substrate absorbs ≥10× normal
  traffic + preserves per-pack data-residency + degrades non-critical
  SLOs gracefully + keeps emergency-services available without
  exception. Per documentation-rigor.md §3.2.5 rows 14 + 22 + 30.
enforcement_status: advisory-until-2026-10-31-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet disaster-mode-surge-capacity-declaration
  - cloud-ci/Rust gate packet disaster-mode-offline-first-coverage
  - cloud-ci/Rust gate packet disaster-mode-dr-pair-failover-config
  - cloud-ci/Rust gate packet disaster-mode-per-pack-overlay
  - cloud-ci/Rust gate packet disaster-mode-cell-isolation-preservation
  - cloud-ci/Rust gate packet disaster-mode-emergency-services-non-throttle
naming_justifications:
  - name: oya-shared-disaster-mode
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.disaster-mode
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate),
      the crate that exposes the disaster-mode coordinator trait +
      surge-handler trait + offline-first sync bridge trait +
      DR-pair failover orchestrator trait + per-cell isolation
      preservation enforcer trait belongs at the shared layer.
      Naming `oya-shared-disaster-mode` keeps the single-concern
      flat layout per ADR-0131 and avoids any "suite" packaging
      per ADR-0132.
  - name: oya-governance-disaster-mode-surge-capacity
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.disaster-mode-surge-capacity
    justification: >
      CI fitness lane per ADR-0212 buildability doctrine; verifies
      every µservice declares a 10× surge-capacity plan + load-shed
      schedule + critical-path preservation in
      `iac/<env>-disaster-mode.yaml`.
  - name: oya-governance-offline-first-coverage
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.offline-first-coverage
    justification: >
      CI fitness lane per ADR-0212; verifies user-facing µservices
      declare CRDT-based offline-first sync via the
      `oya-collab-crdt-portability-kernel` substrate.
  - name: oya-governance-dr-pair-failover
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.dr-pair-failover
    justification: >
      CI fitness lane per ADR-0212; verifies per-cell DR-pair
      configuration per ADR-0241 + per-µservice failover playbook
      in runbook directory.
  - name: oya-governance-disaster-mode-pack-overlay
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.disaster-mode-pack-overlay
    justification: >
      CI fitness lane per ADR-0212; verifies per-pack disaster-mode
      Cedar overlays exist (e.g., HIPAA disaster, GDPR disaster,
      KR-PIPA disaster).
  - name: oya-governance-cell-isolation-preservation
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.cell-isolation-preservation
    justification: >
      CI fitness lane per ADR-0212; verifies cell-isolation per
      ADR-0248 is preserved across DR-pair failover and disaster-
      mode load-shed.
  - name: oya-governance-emergency-services-non-throttle
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.emergency-services-non-throttle
    justification: >
      CI fitness lane per ADR-0212; verifies the disaster-mode
      load-shed schedule + per-pack overlay never throttle
      emergency-services paths per ADR-0298.
  - name: oya-governance-disaster-mode-cell-resilience
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.disaster-mode-cell-resilience
    justification: >
      Aggregate fitness lane per ADR-0212 rolling up the child
      lanes into a single advisory/BLOCKER gate per the keystone-
      bundle 2026-05-20 promotion-gate model.
  - name: X-Oya-Disaster-Mode-Active
    layer: N/A (HTTP response header naming)
    bnf_segments: X-Oya.Disaster-Mode-Active
    justification: >
      Custom HTTP response header indicating disaster-mode is active
      on the serving cell; client libraries consume to enable
      offline-first sync fallback paths.
  - name: X-Oya-DR-Pair-Cell
    layer: N/A (HTTP response header naming)
    bnf_segments: X-Oya.DR-Pair-Cell
    justification: >
      Custom HTTP response header identifying the DR-pair cell the
      request was failed over to, satisfying per-pack data-residency
      audit + transparency.
  - name: X-Oya-Load-Shed-Tier
    layer: N/A (HTTP response header naming)
    bnf_segments: X-Oya.Load-Shed-Tier
    justification: >
      Custom HTTP response header indicating the load-shed tier
      currently active (`tier-0` = full service; `tier-1` = degraded
      reads; `tier-2` = emergency-only).
  - name: DisasterModeActivated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DisasterMode.Activated
    justification: >
      Audit-event-class emitted whenever disaster-mode activates on
      a cell. Registered per ADR-0263.
  - name: DisasterModeDeactivated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DisasterMode.Deactivated
    justification: >
      Audit-event-class emitted whenever disaster-mode deactivates
      on a cell. Registered per ADR-0263.
  - name: DRPairFailoverInitiated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DisasterMode.DRPairFailoverInitiated
    justification: >
      Audit-event-class emitted at DR-pair failover initiation.
      Registered per ADR-0263.
  - name: DRPairFailoverCompleted
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DisasterMode.DRPairFailoverCompleted
    justification: >
      Audit-event-class emitted at DR-pair failover completion.
      Registered per ADR-0263.
  - name: LoadShedTierEscalated
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DisasterMode.LoadShedTierEscalated
    justification: >
      Audit-event-class emitted whenever the load-shed tier
      escalates on a cell. Registered per ADR-0263.
  - name: OfflineFirstSyncReconciled
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DisasterMode.OfflineFirstSyncReconciled
    justification: >
      Audit-event-class emitted when CRDT-based offline-first sync
      reconciles after connectivity restored. Registered per
      ADR-0263.
  - name: CellIsolationPreservationVerified
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: DisasterMode.CellIsolationPreservationVerified
    justification: >
      Audit-event-class emitted on the periodic CI lane verification
      of cell-isolation across DR-pair. Registered per ADR-0263.
  - name: policy/disaster-mode.cedar
    layer: N/A (per-µservice Cedar fragment file)
    bnf_segments: policy.disaster-mode
    justification: >
      Canonical filename for the per-µservice disaster-mode Cedar
      fragment under the µservice's `policy/` directory per
      ADR-0246 + ADR-0243 fragment-lifecycle conventions; single-
      concern naming keeps the policy directory's contract-by-name
      invariant.
  - name: iac/<env>-disaster-mode.yaml
    layer: N/A (per-µservice IaC manifest)
    bnf_segments: iac.<env>.disaster-mode
    justification: >
      Canonical filename for per-µservice + per-env disaster-mode
      IaC manifest declaring per-tier load-shed schedule + DR-pair
      configuration + critical-path preservation rules.
  - name: DR_PAIR_PRIMARY
    layer: N/A (Cell.role enum value per ADR-0248)
    bnf_segments: cell.role.DR_PAIR_PRIMARY
    justification: >
      Cell.role enum extension per ADR-0248 §D-3; identifies the
      primary cell in a DR-pair relationship.
  - name: DR_PAIR_REPLICA
    layer: N/A (Cell.role enum value per ADR-0248)
    bnf_segments: cell.role.DR_PAIR_REPLICA
    justification: >
      Cell.role enum extension per ADR-0248; identifies the replica
      cell in a DR-pair relationship.
  - name: DISASTER_RECOVERY_TENANT
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.DISASTER_RECOVERY_TENANT
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3;
      identifies tenants providing disaster-recovery services
      (911 PSAPs, FEMA partners, Red Cross, regional emergency
      mgmt agencies) that receive priority-allocation during
      disaster-mode.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0306: Disaster-Mode + Cell-Resilience Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **critical-path-cluster-disaster-mode-cell-
resilience** keystone, closing the gap identified in
`docs/standards/documentation-rigor.md` §3.2.5 rows 14 + 22 + 30 of
the critical-path edge-case coverage matrix. The standard already
codifies the row-level handling requirements (disaster-zone surge
handling 10× normal traffic; offline-first sync via CRDT;
progressive enhancement; per-cell DR-pair failover; per-pack
disaster-mode; cell-isolation per ADR-0248 preserved; per-tenant SLO
degrades gracefully; emergency-services NEVER throttled per
ADR-0298); this ADR is the binding ADR the standard's rows 14, 22,
and 30 cite.

Enforcement is `advisory-until-2026-10-31-blocker-thereafter`. The
doctrine accepts in text immediately; the CI lanes promote to
BLOCKER on 2026-10-31 to give per-µservice game-day testing time to
land. Until 2026-10-31, validators emit findings without failing CI;
post-2026-10-31, the lanes block merge.

## Date

2026-05-20.

## Context

### §A. Why disaster-mode is a substrate primitive

Modern hyperscaler platforms treat disaster-mode + cell-resilience as
a *first-class substrate primitive* — wired at the planetary edge,
in every cell's control plane, and as Cedar policy composing with
every other gate. The pattern is unambiguous across the named
industry references:

- **AWS Cell-Based Architecture + Region Isolation + AWS Outposts
  + AWS Wavelength + AWS Local Zones.** Per AWS's 2024 Reliability
  Pillar documentation, every AWS service deploys as cells with
  shuffle-sharding (~10-100 cells per region typical) + per-region
  isolation + per-AZ failover. Per AWS's 2024 published
  Resilience Reports, AWS achieves 99.99%+ availability via cell-
  based architecture. The substrate's disaster-mode primitive is
  operated at AWS scale — substrate, not per-tenant code.
- **Google SRE Workbook + Chubby + Spanner global-isolation.** Per
  Google's "Site Reliability Engineering" 2016 + 2018 books +
  "The Site Reliability Workbook" 2018, Google operates a cell-
  based architecture with per-cell failover + global isolation
  via Spanner; per Google's published Cloud SLAs, Google Cloud
  achieves 99.95%+ availability via cell architecture. Google's
  "Borg" 2015 paper + "Omega" 2013 paper describe the substrate
  primitive in detail.
- **Microsoft Azure Availability Zones + Region-Pair model +
  Microsoft Substrate.** Per Microsoft's 2024 Azure Region-Pair
  documentation, every Azure region has a paired region for DR
  failover; per Microsoft's "Microsoft Substrate" 2024 disclosures,
  the substrate orchestrates per-cell failover + per-region SLO
  preservation.
- **Cloudflare Workers + Cloudflare Pingora migration + Cloudflare
  Anycast.** Per Cloudflare's 2024 Workers + Pingora documentation,
  Cloudflare operates ~300 POPs each with disaster-mode failover
  + Anycast routing. Per Cloudflare's 2024 outage post-mortems
  (April 2023 Atlanta colo outage, June 2023 control-plane
  outage), the disaster-mode primitive is the substrate.
- **Netflix Chaos Engineering + ChAP + AZ-based deployments.** Per
  Netflix's 2014 "Chaos Monkey" paper + 2018 "Chaos Engineering"
  book + 2024 ChAP documentation, Netflix injects failures
  continuously to verify disaster-mode + cell-resilience. The
  substrate primitive is tested, not assumed.
- **Apple iCloud + Apple Maps + Apple Push Notifications.** Per
  Apple's 2024 Platform Security Guide + the Apple Maps SLA, Apple
  operates multi-region cell-based architecture with active-active
  failover; per Apple's published incident post-mortems (e.g.,
  March 2024 iCloud regional outage), the disaster-mode pattern
  is substrate.
- **PSAP integration (911) + FEMA + Red Cross + WHO emergency
  surge.** Per FEMA's 2024 IPAWS documentation + the FCC's 911
  reliability reports + WHO's 2024 Emergency Response Framework,
  emergency-services traffic surges by 10-100× during mass-
  casualty events; the substrate MUST absorb without throttling
  emergency paths.
- **Akamai EdgeWorkers + StackPath + Fastly per-POP failover.**
  Per Akamai's 2024 EdgeWorkers documentation + StackPath's 2024
  technical specs + Fastly's 2024 post-mortems, every Tier-1 CDN
  operates disaster-mode + cell-resilience as substrate primitive.

The corollary: **every internet-facing surface oyatie ships MUST
inherit disaster-mode + cell-resilience from the substrate, not
author it per-µservice.** A µservice that authors its own surge-
handling logic, its own offline-first sync, its own DR-pair
failover, its own load-shedding is duplicating substrate primitives
that the shared `oya-shared-disaster-mode` crate already serves.
That duplication is a `feedback_no_silent_regression` violation;
a `feedback_quality_performance_scalability_bar` violation; and a
`feedback_autonomous_implementation_artifacts` violation.

The ADR-0306 disaster-mode + cell-resilience doctrine closes this
gap.

### §A.1. The disaster landscape 2026 — what the substrate defends against

The 2026 disaster landscape is qualitatively richer than any prior
era:

- **Natural disasters with 10× surge.** Per FEMA's 2024 disaster
  declarations, the US experienced ~95 federally-declared disasters
  in 2024 — wildfires (West Coast), hurricanes (Gulf + Atlantic
  Coast), tornadoes (Midwest), earthquakes (West Coast), severe
  storms (Northeast, Southeast). During acute phase, regional
  cellular networks experience 5-20× normal voice + 10-50× normal
  data traffic per the FCC 2024 Disaster Information Reporting
  System (DIRS).
- **Mass-casualty incidents (active shooter, transport accident,
  building collapse, etc.).** Per the FBI's 2024 active-shooter
  data, ~50+ incidents per year; each triggers regional
  communications surge of 50-200× normal. Per the NTSB's 2024
  reports, major transport incidents (commercial aviation,
  passenger rail, maritime) trigger similar surge.
- **Cyber-incident-driven disaster (ransomware on critical
  infrastructure, DDoS, supply-chain compromise).** Per CISA's
  2024 cybersecurity advisories, ~25 critical-infrastructure
  cyber incidents per year impacting >100k users each; the
  substrate must absorb traffic shifted from compromised
  jurisdictions.
- **Regional infrastructure failure (power grid, telecom backbone,
  submarine cable cut, DNS root server outage).** Per the 2024
  IEEE Spectrum reports, ~30+ major infrastructure failures per
  year; each creates regional outages where the substrate's
  DR-pair takes over.
- **Pandemic + public health emergency.** Per the WHO's 2024
  pandemic-response framework, public health emergencies trigger
  multi-month elevated-traffic regimes (e.g., COVID-19 created
  ~3× sustained traffic for 36 months across multiple
  jurisdictions).
- **War + regional conflict + civil unrest.** Per the UN's 2024
  global conflict tracking, ~50+ active armed conflicts; each
  creates per-jurisdiction traffic disruption + cross-border
  refugee data-portability surge.
- **Climate-driven progressive disaster (sea-level rise, severe
  drought, climate migration).** Per the IPCC's 2024 reports,
  climate-driven multi-decade disaster trajectory; the substrate
  must operate in degraded-grid + offline-first regimes for
  extended periods.

The substrate baseline MUST be sized to this 2026 landscape — not
the 2010 cloud era. The bar is not "scale up under load"; the bar
is "operate across continuously-degraded regimes, preserve per-pack
data-residency under failover, keep emergency-services available,
support offline-first across days-to-weeks of intermittent
connectivity, and degrade gracefully tier-by-tier."

### §A.2. Why the keystone bundle 2026-05-20 requires this as a substrate primitive

The keystone bundle's foundational ADRs intersect disaster-mode +
cell-resilience as follows:

- **ADR-0241 (DR-pair strategy).** ADR-0306 articulates how the
  DR-pair shape per ADR-0241 behaves during active disaster.
  ADR-0241 specifies the topology; ADR-0306 specifies the
  failover orchestration.
- **ADR-0242 (oyatie-is-a-tenant).** The platform's own surfaces
  participate in disaster-mode. No carve-outs.
- **ADR-0243 (Cedar universal gate).** Every disaster-mode decision
  is composable as a Cedar fragment. The disaster-mode-active
  predicate, the load-shed-tier predicate, the DR-pair-target
  predicate compose with every per-µservice Cedar fragment.
- **ADR-0244 (tenant scoping primitive).** Tenants declare their
  disaster-recovery preferences; the substrate honors per-tenant
  failover routing.
- **ADR-0248 (Amazon-shape cellular architecture).** ADR-0306
  preserves cell-isolation during failover; the DR-pair shape is
  the natural extension of the cellular topology.
- **ADR-0251 (compliance packs).** Per-pack disaster-mode rules
  (e.g., HIPAA-disaster, GDPR-disaster, KR-PIPA-disaster) compose
  with the baseline.
- **ADR-0253 (HTTP/3 + QUIC + ECH + PQC).** HTTP/3+QUIC's
  connection migration enables seamless failover across DR-pair
  cells.
- **ADR-0263 (observability emission contract).** Every disaster-
  mode event emits an audit-event-class; the audit-chain remains
  consistent through failover.
- **ADR-0276 (backup portability per GDPR Article 20).** Disaster-
  mode data-portability obligations honored per the per-pack rules.
- **ADR-0292 (minor user doctrine).** Minor users' protections
  preserved during disaster-mode.
- **ADR-0297 (abuse-defence baseline).** Disaster-mode elevates
  abuse-defence sensitivity to account for opportunistic-abuse-
  during-disaster patterns.
- **ADR-0298 (emergency-services bypass doctrine).** ADR-0298
  defines emergency-services; ADR-0306 enforces that emergency-
  services NEVER throttle during disaster-mode. The two ADRs
  compose: ADR-0298 is the path; ADR-0306 is the resilience.
- **ADR-0299 (account-recovery resilience).** Account-recovery
  paths preserved during disaster-mode.
- **ADR-0301 (survivor-safety domestic-abuse mode).** Shelter mode
  preserved during disaster-mode.
- **ADR-0303 (decision-resilience).** Cooling-off + trusted-contact
  preserved during disaster-mode; emergency-services exemption
  honored.
- **ADR-0304 (cross-jurisdiction conflict resolution).** Per-pack
  data-residency preserved during DR-pair failover; cross-pack
  failover forbidden absent multi-pack alignment.
- **ADR-0305 (delegated-agent authority chain).** Delegated agents
  in disaster-mode operate with potentially elevated rate-limits
  + per-pack disaster-mode overlays.

The bundle cannot land without the disaster-mode + cell-resilience
doctrine articulated explicitly. The promotion gate for the 2026-
05-20 bundle is: *the substrate MUST absorb 10× normal traffic
during disaster-mode, preserve per-pack data-residency through
DR-pair failover, support offline-first via CRDT, degrade non-
critical SLOs gracefully tier-by-tier, and keep emergency-services
available without exception.* This ADR is the binding articulation.

### §A.3. What this ADR explicitly does NOT do

- This ADR does not redefine the DR-pair topology — that is
  ADR-0241's scope. This ADR articulates the disaster-mode
  behavior across the topology.
- This ADR does not specify the per-µservice load-shed thresholds
  in detail — each µservice's `iac/<env>-disaster-mode.yaml`
  declares its concrete tier thresholds layered atop the substrate
  baseline.
- This ADR does not specify the CRDT data model — that is the
  `oya-collab-crdt-portability-kernel` substrate. This ADR
  specifies how disaster-mode coordinates with the CRDT bridge.
- This ADR does not redefine Cedar fragment authoring conventions —
  that is ADR-0243 + ADR-0294. This ADR specifies the *content*
  of `policy/disaster-mode.cedar`.
- This ADR does not specify the audit-event-class registry shape —
  that is ADR-0263's scope. This ADR adds eight event classes.
- This ADR does not displace emergency-services routing per
  ADR-0298; the disaster-mode invariant preserves ADR-0298 at
  every tier.
- This ADR does not specify legal-substantive disaster-declaration
  policy; that is per-jurisdiction policy (FEMA, EU Civil
  Protection, KR-PMA). The substrate consumes the per-pack
  disaster-flag.

## Decision

### §B. Six core primitives at three layers

The disaster-mode + cell-resilience baseline is **six core
primitives** (surge handling; offline-first sync; progressive
enhancement; DR-pair failover; per-pack disaster overlay;
emergency-services non-throttle invariant) wired at **three layers**
(Tier-0 shared crate, per-µservice gate, Cedar policy fragment).
The 6×3 matrix produces eighteen cells; each cell has a defined
primitive.

```
                    Tier-0 shared              Per-µservice          Cedar policy
                    -------------              -------------         -------------
Surge handling      Surge-detector +           Per-route load-shed   forbid when
                    10× capacity reserve       tier escalator         load_shed_tier
                                                                       > route_minimum_tier

Offline-first       CRDT bridge +              Per-µservice CRDT     permit but
sync                conflict-free merge        coalescer              emit OfflineSyncEvent

Progressive         Capability-detector +      Per-route capability  forbid when
enhancement         degraded-path router        check                  required_capability
                                                                       not_available
                                                                       AND no_degraded_path

DR-pair failover    DR-pair orchestrator +     Per-µservice          forbid when
                    health-check               failover-readiness     dr_pair_failover_in_
                                                                       progress AND
                                                                       request_class ≠
                                                                       critical

Per-pack disaster   Per-pack disaster-mode    Per-µservice pack     forbid when
overlay             registry                    consultation           pack_disaster_mode_
                                                                       active AND
                                                                       action_forbidden_
                                                                       under_disaster

Emergency-services  Bypass router (per         Per-route emergency   permit always when
non-throttle        ADR-0298)                  path                    emergency_path_
                                                                       attested
                                                                       (NEVER forbid)
```

The six primitives are **interdependent**:

- **Surge handling** addresses the volumetric component. When
  traffic surges to 10× normal, the substrate's surge-detector
  triggers load-shed tier escalation (`tier-0` → `tier-1` →
  `tier-2`).
- **Offline-first sync** addresses the connectivity component.
  When connectivity degrades (rural users, post-disaster, satellite
  internet), the substrate's CRDT bridge enables local-first
  writes that reconcile when connectivity restored.
- **Progressive enhancement** addresses the capability component.
  When client capabilities degrade (JavaScript-disabled, low-power
  mode, accessibility-tool active, low-bandwidth), the substrate's
  capability-detector routes to degraded-path equivalents that
  preserve core workflow.
- **DR-pair failover** addresses the cell-failure component. When
  a cell becomes unhealthy, the substrate's DR-pair orchestrator
  fails over to the paired cell per ADR-0241.
- **Per-pack disaster overlay** addresses the regulatory component.
  Per-pack disaster-mode rules (HIPAA, GDPR, KR-PIPA, etc.)
  compose with the baseline.
- **Emergency-services non-throttle invariant** is the absolute
  ceiling. Per ADR-0298, emergency-services NEVER throttle even
  during the most severe disaster.

The three layers are **complementary**:

- **Tier-0 shared crate** centralizes the surge-detector, the CRDT
  bridge, the DR-pair orchestrator, the per-pack registry, the
  emergency-services bypass router.
- **Per-µservice gate** sees the µservice-local request context
  (route, resource, action, client-capabilities).
- **Cedar policy fragment** composes the substrate + µservice +
  per-tenant + per-pack signals into a single permit/forbid
  decision per ADR-0243 + ADR-0263.

### §B.1. The load-shed tier schedule

The substrate's load-shed schedule is tier-based:

| Tier | Trigger | Service shape | Emergency-services |
|---|---|---|---|
| `tier-0` (normal) | Surge < 2× | Full service; all routes available | available |
| `tier-1` (elevated) | Surge 2-5× | Non-critical writes degraded; reads available; cache TTLs extended | available |
| `tier-2` (degraded) | Surge 5-10× | Writes minimal; reads cached; analytics + reporting deferred | available |
| `tier-3` (critical) | Surge >10× | Writes blocked except emergency-services + critical-class; reads cached | available |
| `tier-4` (disaster) | Cell-level failure | DR-pair failover initiated; client retries to DR-pair | available |

**Per-route tier minimum:**

Each route declares its minimum-acceptable tier:

- `tier-0` minimum: analytics, recommendations, suggestions, social
  feed, marketplace browse.
- `tier-1` minimum: bulk operations, batch exports, scheduled
  workflows.
- `tier-2` minimum: standard CRUD, search, messaging, notes.
- `tier-3` minimum: account-critical operations, billing, tenant-
  admin actions, emergency-mode toggles.
- `tier-4` always-available: emergency-services per ADR-0298,
  shelter-mode per ADR-0301, healthcare break-glass per ADR-0247.

If the current load-shed tier exceeds the route's minimum, the
substrate refuses + returns 503 (Service Unavailable) with
`Retry-After` + `X-Oya-Load-Shed-Tier`.

### §B.2. The DR-pair failover state machine

Per ADR-0241, every cell has a DR-pair. The failover state machine:

```
┌──────────────────┐
│ Idle             │ — primary healthy, replica synced
│   cell.role =    │
│   DR_PAIR_       │
│   PRIMARY        │
└────────┬─────────┘
         │ health-check failure ≥ N consecutive
         │   (N = 3 default, configurable per ADR-0241)
         ▼
┌──────────────────┐
│ FailoverInitiated│ — primary unhealthy; promote replica
│                  │
│ Substrate steps: │
│ 1. Drain         │
│    in-flight     │
│    requests      │
│    (≤ 30s)        │
│ 2. Flip          │
│    DR-pair       │
│    PRIMARY ↔     │
│    REPLICA       │
│ 3. Update edge   │
│    Anycast       │
│    routing       │
│ 4. Notify        │
│    audit-chain   │
└────────┬─────────┘
         │ failover complete (≤ 5 min target)
         ▼
┌──────────────────┐
│ FailoverComplete │ — replica is now primary
│   former-replica │
│   cell.role =    │
│   DR_PAIR_       │
│   PRIMARY        │
└────────┬─────────┘
         │ recovery of former primary
         │ (manual or automated)
         ▼
┌──────────────────┐
│ Resync           │ — former primary catches up
│                  │
│   former-primary │
│   cell.role =    │
│   DR_PAIR_       │
│   REPLICA        │
└──────────────────┘
```

**Failover invariants:**

- **Per-pack data-residency preserved.** DR-pair cells are within
  the same per-pack data-residency boundary (e.g., EU-Sovereign
  cell's DR-pair is another EU-Sovereign cell). Cross-pack
  failover is FORBIDDEN per ADR-0304.
- **Cell-isolation preserved.** Per ADR-0248, the DR-pair cell
  receives traffic only from the failed primary's tenants; not
  from arbitrary other cells' tenants.
- **Audit chain consistent.** The audit-chain Merkle anchor per
  ADR-0028 + ADR-0263 remains consistent through failover; the
  DR-pair appends to the same logical chain.
- **Bounded failover time.** Per ADR-0241, failover target ≤ 5
  minutes (RTO); data loss bound ≤ 30 seconds (RPO).
- **Notification surface.** Failover initiation + completion emit
  `DRPairFailoverInitiated` + `DRPairFailoverCompleted` event
  classes per ADR-0263.

### §B.3. The offline-first sync mechanism

Per the `oya-collab-crdt-portability-kernel` substrate, the
substrate provides CRDT-based offline-first sync. The mechanism:

- **CRDT data model.** Conflict-free Replicated Data Types per
  the kernel substrate. Operations are commutative + idempotent
  + monotonic, enabling offline-first writes that reconcile
  deterministically.
- **Client-side write log.** When offline, the client writes
  to a local log. Each write is tagged with a vector-clock + a
  client-id + a tenant-id.
- **Reconciliation on reconnect.** When connectivity restored,
  the client uploads the write log. The substrate's CRDT bridge
  merges using the CRDT operation's merge function (e.g., LWW
  for last-write-wins; OR-Set for sets; G-Counter for monotonic
  counters; PN-Counter for increment/decrement).
- **Conflict surface for non-CRDT data.** For data classes that
  do not have a CRDT representation (e.g., financial transactions),
  the offline write is queued + the user is informed; the
  reconciliation requires user-conflict-resolution UI.
- **Per-µservice CRDT registry.** Each µservice declares which
  data classes have CRDT representation in
  `iac/<env>-disaster-mode.yaml`.
- **Audit emission.** On reconciliation, the substrate emits
  `OfflineFirstSyncReconciled` per ADR-0263.

### §B.4. The progressive-enhancement degradation hierarchy

The substrate supports progressive enhancement across capability
tiers:

| Capability tier | Client | Substrate behavior |
|---|---|---|
| **Full** | Modern browser, JS, WebAuthn, fonts, images | Default path |
| **JS-degraded** | JS-disabled / failed | Server-rendered fallback; passkey via redirect-form |
| **Low-bandwidth** | <128 Kbps | Minified assets; lazy-loaded images; HTTP/3 multiplexing |
| **Offline-buffered** | Intermittent | Service-worker cache + offline-first sync per §B.3 |
| **Text-only** | Lynx / curl / accessibility | Text response; semantic HTML; no required JS |
| **Voice** | Voice-only client | TTS rendering; ARIA-compliant DOM; audio-CAPTCHA per ADR-0297 |
| **Single-switch** | Single-input device | Time-extended interactions; explicit-confirm always |

The substrate routes the request to the appropriate degraded path
based on client capability declarations + server-side feature
detection.

### §B.5. The emergency-services non-throttle invariant — absolute

The most critical invariant in this ADR. Per ADR-0298, emergency-
services paths NEVER throttle even during the most severe disaster-
mode. This is enforced as:

> **No load-shed tier may include emergency-services routes in its
> shed-set. No DR-pair failover delay may exceed 5 seconds on an
> emergency-services route. No per-pack disaster overlay may
> forbid emergency-services. No surge-handling may queue an
> emergency-services request behind any other request.**

Enforcement mechanisms:

- **Tier-isolation.** Emergency-services routes are tagged
  `tier-4-always-available`; the surge handler never elevates the
  emergency-services tier above `tier-4`.
- **Priority queue.** Emergency-services requests enter a separate
  priority queue with reserved capacity (≥10% of cell's total
  capacity reserved exclusively).
- **DR-pair fast-path.** During DR-pair failover, emergency-
  services requests route via a fast-path that bypasses the
  normal drain (≤ 5s failover for emergency vs ≤ 5 min normal).
- **Per-pack exemption.** Every per-pack disaster-mode overlay
  MUST include the emergency-services exemption at the top of
  the fragment.
- **CI lane enforcement.** `oya-governance-emergency-services-
  non-throttle` BLOCKER from 2026-10-31 verifies every µservice +
  every per-pack overlay enforces the invariant.

This invariant is the highest-priority hard-rule in the substrate.
Any violation is a P0 incident + per-pack regulator notification.

### §B.6. The cell-isolation preservation invariant

Per ADR-0248, cells are isolated. During disaster-mode + failover,
isolation MUST be preserved:

> **A DR-pair failover MUST NOT mix traffic from different cells'
> tenants into a single cell. The failover preserves the source
> cell's tenant boundary; the DR-pair receives only the source
> cell's tenants.**

In practice:

- **DR-pair is per-cell, not per-region.** Each cell has its own
  DR-pair; not a regional shared replica.
- **Shuffle-sharding preserved.** Per ADR-0248, tenant
  shuffle-sharding is preserved across failover.
- **No cross-cell load-shed.** The substrate does not redistribute
  tenants across cells under load; it shedss within the cell.
- **Capacity reserve per cell.** Each cell maintains 10% reserve
  capacity for DR-pair surge during partner-cell failure.

## §C. Consequences

### §C.1. Maintainability dimension

The disaster-mode + cell-resilience baseline is the substrate that
every internet-facing µservice inherits. Maintainability invariants:

- **Per-µservice declaration is configuration.** Each µservice
  declares its disaster-mode posture in `ARCHITECTURE.md
  §disaster-mode` + `iac/<env>-disaster-mode.yaml` +
  `policy/disaster-mode.cedar`. The actual primitive lives in
  `oya-shared-disaster-mode` + `oya-collab-crdt-portability-
  kernel`.
- **Per-tenant tuning is configuration.** Tenants declare
  disaster-recovery preferences via the tenancy substrate.
- **Per-pack disaster overlay is configuration.** Per-pack
  disaster-mode rules in `policy/disaster-mode-overlays/`.
- **Versioning policy.** Cedar fragment per ADR-0294; IaC per
  ADR-0258 SemVer.
- **Game-day testing.** Every µservice MUST execute a quarterly
  disaster-mode game-day per the runbook; results in
  `evidence/disaster-mode-gameday-<date>.json`.
- **Single-concern crate.** The shared crate is single-concern per
  ADR-0131. It does NOT absorb account-recovery (ADR-0299),
  survivor-safety (ADR-0301), or cross-jurisdiction conflict
  (ADR-0304).
- **Tests as inheritance proof.** Every µservice ships contract
  tests against the shared crate's disaster-mode fixtures.
- **Documentation density.** Each µservice's PRD MUST cite its
  load-shed tier minimum, its DR-pair failover playbook, its
  offline-first coverage, its emergency-services exemption.

### §C.2. Observability dimension

Per ADR-0263:

- **Audit-event-classes:**
  - `DisasterModeActivated`
  - `DisasterModeDeactivated`
  - `LoadShedTierEscalated`
  - `LoadShedTierDeescalated`
  - `DRPairFailoverInitiated`
  - `DRPairFailoverCompleted`
  - `DRPairFailoverFailed`
  - `OfflineFirstSyncReconciled`
  - `OfflineFirstSyncConflictDetected`
  - `CellIsolationPreservationVerified`
  - `EmergencyServicesNonThrottleEnforced`
  - `PerPackDisasterOverlayApplied`
  - `ProgressiveEnhancementDegradedPathServed`
- **Metrics:**
  - `oya_disaster_mode_active_gauge` — count of cells in
    disaster-mode. Dimensions: cell_id, disaster_class.
  - `oya_disaster_mode_load_shed_tier_gauge` — current tier per
    cell. Dimensions: cell_id, µservice.
  - `oya_disaster_mode_dr_pair_failover_counter` — failover events.
    Dimensions: source_cell, target_cell, failover_cause.
  - `oya_disaster_mode_dr_pair_failover_latency_histogram` —
    failover duration.
  - `oya_disaster_mode_offline_sync_reconcile_counter` —
    reconciliations. Dimensions: tenant_bucket, µservice.
  - `oya_disaster_mode_offline_sync_conflict_counter` — conflict
    detection events.
  - `oya_disaster_mode_surge_ratio_gauge` — current surge ratio
    per cell.
  - `oya_disaster_mode_emergency_capacity_reserve_gauge` — reserved
    capacity for emergency-services.
  - `oya_disaster_mode_cell_isolation_violation_counter` — MUST be
    0; any non-zero is P0.
- **Dashboards:** Each user-facing µservice ships
  `dashboards/disaster-mode.json` with the canonical 14-panel
  layout.

### §C.3. Scalability dimension

The substrate scales to absorb 10× normal traffic:

- **Cell-level capacity reserve.** Each cell maintains 10% reserve
  for DR-pair surge + 10% reserve for tier-3 emergency capacity =
  20% headroom per cell. Total per-cell capacity sized at 1.25×
  steady-state.
- **Surge absorption.** Tier-1 absorbs 2-5× via cache + CDN edge;
  tier-2 absorbs 5-10× via load-shed + queue.
- **Offline-first reduces server-side load.** Connectivity-
  degraded users write to local logs; server-side reconciliation
  is batched + amortized.
- **Burst capacity.** Per-cell burst headroom 1.5× steady-state;
  burst absorbed by cache + CDN edge.
- **Multi-region capacity.** Per ADR-0240 + ADR-0241, multi-region
  capacity sized at 2× single-region peak so any one region
  can fully absorb its DR-pair's traffic.

### §C.4. Performance dimension

- **DR-pair failover latency target.** ≤ 5 minutes RTO normal;
  ≤ 5 seconds RTO for emergency-services.
- **Failover RPO.** ≤ 30 seconds data loss bound.
- **Load-shed tier transition latency.** ≤ 10 seconds from surge
  detection to tier escalation.
- **Offline-first reconciliation latency.** O(log(write_log_size))
  per client write; p99 ≤ 100 ms for typical workloads.
- **CPU budget under disaster-mode.** Per-request overhead ≤ 20 μs
  CPU including disaster-mode evaluation.
- **Memory budget.** Cell capacity-reserve allocation ≤ 20% per-
  cell memory; offline-first write log ≤ 100 MB per client.

### §C.5. Optimization dimension

- **Pre-computed disaster-class detection.** Surge detection uses
  pre-computed per-cell baseline; hot-path is O(1) ratio
  comparison.
- **Cell-local DR-pair health-check.** Health-check is cell-local
  + cached; no cross-cell hop on the hot-path.
- **CRDT operation batching.** Multiple offline writes batched
  into a single reconciliation call.
- **Progressive-enhancement capability cache.** Client capability
  declarations cached per session; not recomputed per request.
- **Priority queue per cell.** Emergency-services + critical-class
  in a per-cell priority queue with reserved CPU.
- **Pre-warmed DR-pair connections.** mTLS connections to DR-pair
  pre-warmed during normal operation; no handshake delay on
  failover.

### §C.6. Code quality dimension

- **Single ingress trait.** `DisasterModeGate::evaluate_or_shed()`;
  no µservice authors its own surge-handling.
- **No `#[cfg(test)]` bypass.**
- **Mandatory documentation.** Every µservice MUST include
  `compliance.md §disaster-mode-edge-cases` per §3.2.5 rows 14, 22,
  30.
- **Deterministic test fixtures.** The shared crate ships fixtures
  for canonical disaster classes (surge, regional outage, mass-
  casualty, infrastructure failure).
- **No magic numbers.** All tier thresholds, reserve percentages
  declared in `iac/<env>-disaster-mode.yaml`.
- **Audit-event-class registration enforcement.**
- **Property-based test coverage.** ≥ 85% per ADR-0212.
- **Chaos engineering enforcement.** Per ADR-0212, every µservice
  MUST pass quarterly chaos-monkey injection per the
  Netflix Chaos pattern.

## §D. Detailed mechanics

### §D-1. Worked example — disaster row 14 (low-bandwidth / disaster-zone / offline-first)

Scenario: a user in a post-hurricane region with intermittent
satellite internet attempts to use oyatie.

**Step 1 — Connectivity detection.**

Substrate's client SDK detects intermittent connectivity (≥1
heartbeat timeout in 60s window). Service worker enters offline-
first mode.

**Step 2 — Offline writes accumulate.**

User creates notes, sends messages (queued), updates project
status. All writes are CRDT-compatible per the
`oya-collab-crdt-portability-kernel`; written to local IndexedDB.

**Step 3 — Reconciliation on reconnect.**

When connectivity restores, service worker uploads the write log
to the substrate. Substrate's CRDT bridge merges using each
operation's CRDT merge function:

- LWW-Register for note bodies (last-write-wins by wall-clock).
- OR-Set for tags (add-wins).
- G-Counter for view counts (monotonic increment).

Substrate emits `OfflineFirstSyncReconciled` per reconciliation.

**Step 4 — Conflict surface (rare).**

If a non-CRDT data class has a conflict (e.g., two concurrent
balance updates), substrate emits `OfflineFirstSyncConflict
Detected` + the user receives a conflict-resolution UI prompt
per ADR-0276 portability semantics.

**Step 5 — Per-pack honoring.**

Per ADR-0304, reconciliation honors per-pack data-residency. If
the user's data is EU-Sovereign-cell-pinned, the reconciliation
targets EU-Sovereign DR-pair.

### §D-2. Worked example — disaster row 22 (mass-casualty incident / disaster-zone surge)

Scenario: a major incident (active shooter / earthquake / mass
transit accident) triggers a regional communications surge.

**Step 1 — Surge detection.**

Substrate's surge-detector observes 50× normal request rate from
the affected region. Triggers `LoadShedTierEscalated` to
`tier-2`.

**Step 2 — Load-shed activation.**

- Tier-0 routes (analytics, recommendations) refused with 503 +
  Retry-After.
- Tier-1 routes (bulk operations) refused.
- Tier-2 routes (standard CRUD, messaging) available with
  elevated cache TTLs.
- Tier-3 routes (account-critical) available with no degradation.
- Tier-4 routes (emergency-services, shelter-mode, healthcare
  break-glass) available with priority routing.

**Step 3 — Emergency-services priority.**

A user attempting to contact emergency services (911 PSAP
integration) is routed via the per-ADR-0298 bypass; the request
enters the cell's emergency-services priority queue with reserved
capacity.

**Step 4 — Tier-3 escalation.**

Surge continues to 10×; substrate escalates to `tier-3`. Standard
writes blocked except emergency-services + critical-class. The
substrate communicates the tier to clients via
`X-Oya-Load-Shed-Tier` header so client UI may explain
"limited service due to high traffic."

**Step 5 — Per-pack disaster-mode active.**

If the affected jurisdiction has a `pack-us-disaster-fema`
attestation (or per-pack equivalent), the per-pack disaster-mode
overlay activates: e.g., per-pack rate-limits relaxed for
emergency-coordination tenants (FEMA partners, Red Cross, state
emergency mgmt).

**Step 6 — Surge resolves.**

When surge drops below 5× sustained 5 minutes, substrate
deescalates to `tier-2` → `tier-1` → `tier-0`. Each transition
emits `LoadShedTierDeescalated`.

**Step 7 — Post-incident review.**

Substrate generates a per-cell incident summary; audit-chain
preserves the full timeline; per ADR-0263 incident-response runbook
followed.

### §D-3. Worked example — disaster row 30 (service degradation during regional outage)

Scenario: a regional power-grid failure takes out a single cell
(`cell-us-east-1-pop-001`) entirely.

**Step 1 — Health-check failure.**

Substrate's per-cell health-check fires consecutive failures (≥3
in 30s window).

**Step 2 — DR-pair failover initiation.**

Substrate emits `DRPairFailoverInitiated{source_cell=cell-us-east-
1-pop-001, target_cell=cell-us-east-1-pop-002}`. Per ADR-0241,
the DR-pair is co-located within the same per-pack data-residency
boundary.

**Step 3 — In-flight drain.**

Substrate drains in-flight requests at the failed cell over 30
seconds. Requests not completing within the drain window are
retried at the DR-pair.

**Step 4 — Anycast routing update.**

Edge POPs update Anycast routing to direct
`cell-us-east-1-pop-001` traffic to `cell-us-east-1-pop-002`.
HTTP/3 + QUIC connection migration per ADR-0253 enables seamless
client transition.

**Step 5 — Role flip.**

`cell-us-east-1-pop-002` flips from `DR_PAIR_REPLICA` to
`DR_PAIR_PRIMARY`. Audit-chain Merkle anchor updated.

**Step 6 — Cell-isolation preserved.**

Traffic from `cell-us-east-1-pop-001`'s tenants now serves at
`cell-us-east-1-pop-002`. Traffic from other cells' tenants is
NOT mixed in; cell-isolation per ADR-0248 preserved.

**Step 7 — Per-pack data-residency preserved.**

Both cells are within the same per-pack data-residency boundary
(US-Cellular-Pack). No cross-pack failover.

**Step 8 — Emergency-services fast-path.**

During the failover, emergency-services requests bypass the
normal drain + immediately route to the DR-pair (≤ 5s end-to-end).

**Step 9 — Failover complete.**

Substrate emits `DRPairFailoverCompleted{source_cell=...,
target_cell=..., duration_seconds=N, rpo_seconds=N}`. Audit-chain
preserved.

**Step 10 — Resync.**

When the failed cell is recovered (power restored, hardware
replaced), it enters `DR_PAIR_REPLICA` role + catches up via the
substrate's resync protocol. Bidirectional CRDT merge handles
the gap.

### §D-4. Cedar policy fragment — `policy/disaster-mode.cedar`

```cedar
// policy/disaster-mode.cedar
// Per-µservice Cedar fragment per ADR-0306 + ADR-0243 + ADR-0294.

// Default-deny: action refused when load-shed tier above route minimum,
// except emergency-services which always permit.

// Predicate 1: load-shed tier above route minimum
forbid (
  principal,
  action,
  resource
)
when {
  context.current_load_shed_tier > resource.tier_minimum &&
  !context.emergency_path_attested
};

// Predicate 2: DR-pair failover in progress
forbid (
  principal,
  action,
  resource
)
when {
  context.dr_pair_failover_in_progress == true &&
  resource.tier_minimum >= context.failover_drain_tier_cutoff &&
  !context.emergency_path_attested
};

// Predicate 3: cross-pack failover attempt (must NEVER permit)
forbid (
  principal,
  action,
  resource
)
when {
  context.dr_pair_failover_in_progress == true &&
  context.dr_pair_target_cell_pack != context.source_cell_pack
};

// Predicate 4: per-pack disaster overlay forbids
forbid (
  principal,
  action,
  resource
)
when {
  context.applicable_pack_disaster_active == true &&
  context.action_forbidden_under_pack_disaster == true &&
  !context.emergency_path_attested
};

// Predicate 5: cell-isolation violation attempt
forbid (
  principal,
  action,
  resource
)
when {
  context.cross_cell_tenant_mix_attempted == true
};

// Emergency-services bypass per ADR-0298 — ALWAYS PERMIT
permit (
  principal,
  action in [
    Action::"emergency_services_initiate",
    Action::"crisis_hotline_connect",
    Action::"healthcare_break_glass",
    Action::"shelter_mode_activate",
    Action::"emergency_proceed_under_disaster"
  ],
  resource
)
when {
  context.emergency_path_attested == true
};

// Offline-first sync reconciliation — permit per CRDT merge
permit (
  principal,
  action == Action::"reconcile_offline_first_sync",
  resource
)
when {
  context.crdt_compatible_data_class == true &&
  context.write_log_signature_valid == true
};

// Progressive-enhancement degraded path — permit with emit
permit (
  principal,
  action,
  resource
)
when {
  context.client_capability_degraded == true &&
  context.degraded_path_available == true &&
  context.degraded_path_serve_emit == true
};
```

### §D-5. Per-pack disaster overlay — examples

```yaml
# packs/us-hipaa/disaster-mode.yaml
pack_id: pack-us-hipaa
disaster_mode:
  emergency_break_glass:
    relaxed_audit_pre_action: true  # post-hoc audit acceptable per HIPAA §164.512
    documentation_required_post_hoc: ≤72_hours
  patient_communication:
    surge_capacity_relaxed: true  # message-throughput cap relaxed
  data_residency:
    cross_region_emergency_replication: permitted_per_BAA
  emergency_services_exemption: always_permit
  audit_emission_required: always

# packs/eu-gdpr/disaster-mode.yaml
pack_id: pack-eu-gdpr
disaster_mode:
  data_subject_rights:
    relaxed_response_window: ≤30_days_extended  # per GDPR Art. 12(3)
  cross_border_transfer:
    emergency_derogation: permitted_per_Art_49
  data_minimization:
    relaxed_for_emergency_coordination: true  # per Art. 6(1)(d) vital interests
  emergency_services_exemption: always_permit
  audit_emission_required: always

# packs/kr-pipa/disaster-mode.yaml
pack_id: pack-kr-pipa
disaster_mode:
  cross_border_transfer:
    emergency_derogation: permitted_per_PIPA_Art_18
  notice_window:
    relaxed: ≤72_hours
  emergency_services_exemption: always_permit
  audit_emission_required: always

# packs/disaster-fema/disaster-mode.yaml
pack_id: pack-disaster-fema
disaster_mode:
  rate_limit_relaxation_for_emergency_coordinators: 100x
  ipaws_message_priority: highest
  emergency_services_exemption: always_permit
  audit_emission_required: always
```

### §D-6. Per-cell-tier variants

Per ADR-0248:

- **Tier-0 cells (edge POPs).** Tier-0 edge absorbs surge via
  cache + CDN; routes emergency-services on fast-path.
- **Tier-1 cells (regional control planes).** Hosts surge-
  detector + DR-pair orchestrator + per-cell health-check.
- **Tier-2 cells (data plane regions).** Per-µservice Cedar gate;
  load-shed tier evaluation.
- **Tier-3 cells (compliance-isolated).** Per-pack disaster-mode
  overlay; restricted cross-cell traffic.
- **Tier-4 cells (sovereign-cloud).** Sovereign-specific disaster
  protocol; DR-pair within sovereign boundary only.

### §D-7. Observability — metrics, dashboards, audit-event-classes

Per ADR-0263:

**Audit-event-classes:** see §C.2.

**Metrics:** see §C.2.

**Dashboard:** 14-panel canonical layout per §C.2.

### §D-8. Per-tenant audience-type tuning

| Audience type | Disaster-mode behavior |
|---|---|
| `B2C_CONSUMER` | Graceful degradation; offline-first enabled |
| `B2B_TENANT` | Per-tenant SLA preserved within tier limits |
| `SENIOR_PROTECTED` | Trusted-contact alerts preserved; cooling-off honored |
| `MINOR_PII` | Parental notification preserved |
| `HIGH_RISK_USER` | Shelter mode preserved per ADR-0301 |
| `DISASTER_RECOVERY_TENANT` | Priority routing; 100× rate-limit per pack-disaster-fema |
| `SOVEREIGN_GOV_TENANT` | Per-jurisdiction emergency-coordination priority |
| `FRIENDLY_CRAWLER_PARTNER` | Refused during tier ≥ 2 |

### §D-9. Compliance interactions

- **HIPAA §164.512(b) (public health).** Disaster-mode allows
  public-health emergency disclosures.
- **HIPAA §164.510(b) (notification).** Disaster-mode permits
  next-of-kin notification.
- **GDPR Article 6(1)(d) (vital interests).** Disaster-mode
  permits processing necessary to protect vital interests.
- **GDPR Article 49 (derogations).** Disaster-mode permits
  cross-border transfer for important public interest reasons.
- **GDPR Article 12(3) (extended response).** Disaster-mode
  permits ≤30-day extension to data-subject-rights response.
- **KR-PIPA Article 18 (disaster).** Korean disaster-mode
  derogations.
- **PCI-DSS Requirement 9.1.3 (incident response).** Disaster-mode
  incident-response procedures.
- **NIS2 Article 23 (incident reporting).** ≤24h impact
  assessment preserved during disaster-mode.
- **SOC 2 CC7.5 (disaster recovery).** Substrate DR-pair primitive.

## §E. Implementation footprint

### §E.1. New crate

```
oya-shared-disaster-mode/
├── Cargo.toml                            # workspace crate, single-concern
├── src/
│   ├── lib.rs                            # DisasterModeGate trait
│   ├── surge/
│   │   ├── mod.rs                        # surge submodule
│   │   ├── detector.rs                   # per-cell surge detector
│   │   ├── tier_escalator.rs             # tier-0 → tier-3 escalation
│   │   └── reserve_capacity.rs           # 10% reserve enforcement
│   ├── offline_first/
│   │   ├── mod.rs                        # offline-first submodule
│   │   ├── crdt_bridge.rs                # bridge to oya-collab-crdt-portability-kernel
│   │   ├── write_log.rs                  # client-side write log shape
│   │   ├── reconciler.rs                 # per-write-class reconciler
│   │   └── conflict_surface.rs           # non-CRDT conflict UI
│   ├── progressive_enhancement/
│   │   ├── mod.rs                        # progressive-enhancement submodule
│   │   ├── capability_detector.rs        # client capability detection
│   │   ├── degraded_path_router.rs       # per-route fallback router
│   │   └── tier_registry.rs              # capability-tier definitions
│   ├── dr_pair/
│   │   ├── mod.rs                        # DR-pair submodule
│   │   ├── orchestrator.rs               # DR-pair failover state machine
│   │   ├── health_check.rs               # per-cell health-check
│   │   ├── anycast_router.rs             # edge Anycast update
│   │   ├── drain.rs                      # in-flight drain
│   │   └── resync.rs                     # post-recovery resync
│   ├── per_pack_disaster/
│   │   ├── mod.rs                        # per-pack disaster submodule
│   │   ├── registry.rs                   # per-pack disaster registry
│   │   └── overlay.rs                    # per-pack disaster overlay
│   ├── emergency_services/
│   │   ├── mod.rs                        # emergency-services non-throttle submodule
│   │   ├── bypass.rs                     # ADR-0298 bypass router
│   │   ├── priority_queue.rs             # per-cell priority queue
│   │   └── reserve_capacity.rs           # 10% emergency capacity reserve
│   ├── cell_isolation/
│   │   ├── mod.rs                        # cell-isolation preservation submodule
│   │   ├── preservation_enforcer.rs      # per-failover preservation enforcer
│   │   └── violation_detector.rs         # P0 detector for any violation
│   ├── cedar_fragment/
│   │   ├── mod.rs                        # Cedar fragment helpers
│   │   ├── context_builder.rs
│   │   └── evaluator.rs
│   ├── audit/
│   │   ├── mod.rs                        # audit-event-class emission
│   │   ├── event_class.rs
│   │   └── emit.rs
│   ├── observability/
│   │   ├── mod.rs                        # metrics + dashboards
│   │   ├── metrics.rs
│   │   └── tracing.rs
│   ├── chaos/
│   │   ├── mod.rs                        # chaos-engineering injection per Netflix pattern
│   │   ├── monkey.rs                     # ChAP-class fault injection
│   │   └── game_day.rs                   # quarterly game-day driver
│   └── error.rs
├── tests/
│   ├── surge_handling.rs                 # 10× surge absorption test
│   ├── offline_first_crdt.rs             # CRDT reconciliation test
│   ├── offline_first_conflict.rs         # non-CRDT conflict handling
│   ├── progressive_enhancement.rs        # capability-tier router test
│   ├── dr_pair_failover.rs               # state-machine + RTO test
│   ├── dr_pair_cross_pack_block.rs       # cross-pack forbidden test
│   ├── cell_isolation_preservation.rs    # isolation invariant test
│   ├── emergency_services_non_throttle.rs # absolute invariant test
│   ├── per_pack_disaster_overlay.rs      # per-pack overlay tests
│   ├── chaos_property.rs                 # chaos-injection property tests
│   ├── game_day_quarterly.rs             # game-day driver test
│   └── fixtures/
│       ├── disaster_class_fixtures.rs
│       ├── dr_pair_fixtures.rs
│       ├── crdt_fixtures.rs
│       └── per_pack_disaster_fixtures.rs
└── docs/
    ├── README.md
    ├── ARCHITECTURE.md
    ├── usage.md
    ├── game-day-runbook.md
    └── disaster-class-catalog.md
```

### §E.2. New µservice extensions

```
microservices/<name>/
├── policy/
│   ├── disaster-mode.cedar
│   └── disaster-mode-overlays/
│       ├── pack-us-hipaa.cedar
│       ├── pack-eu-gdpr.cedar
│       ├── pack-kr-pipa.cedar
│       ├── pack-disaster-fema.cedar
│       └── pack-disaster-eu-civil-protection.cedar
├── iac/
│   ├── dev-disaster-mode.yaml
│   ├── staging-disaster-mode.yaml
│   └── prod-disaster-mode.yaml
├── docs/
│   ├── ARCHITECTURE.md                   # +§disaster-mode
│   ├── PRD.md                            # +§disaster-mode-edge-cases
│   ├── compliance.md                     # +§disaster-mode per §3.2.5 rows 14, 22, 30
│   └── runbooks/
│       ├── disaster-mode-tier-escalation.md
│       ├── disaster-mode-dr-pair-failover.md
│       ├── disaster-mode-offline-first-reconcile.md
│       ├── disaster-mode-cell-isolation-violation-p0.md
│       └── disaster-mode-quarterly-game-day.md
├── tests/
│   └── disaster_mode_contract.rs
├── dashboards/
│   └── disaster-mode.json
└── slos/
    ├── dr-pair-failover-rto.openslo.yaml
    ├── load-shed-tier-transition-latency.openslo.yaml
    └── emergency-services-non-throttle.openslo.yaml
```

### §E.3. New runbooks

- `disaster-mode-tier-escalation.md`
- `disaster-mode-dr-pair-failover.md`
- `disaster-mode-offline-first-reconcile.md`
- `disaster-mode-cell-isolation-violation-p0.md`
- `disaster-mode-quarterly-game-day.md`
- `disaster-mode-emergency-services-non-throttle-verification.md`
- `disaster-mode-per-pack-overlay-activation.md`

### §E.4. New CI lanes

- `oya-governance-disaster-mode-surge-capacity`
- `oya-governance-offline-first-coverage`
- `oya-governance-dr-pair-failover`
- `oya-governance-disaster-mode-pack-overlay`
- `oya-governance-cell-isolation-preservation`
- `oya-governance-emergency-services-non-throttle`
- `oya-governance-disaster-mode-cell-resilience` (aggregate)

### §E.5. Vendor selection rationale

- **CRDT substrate** — `oya-collab-crdt-portability-kernel` (single
  shared substrate per ADR-0145 inter-microservice communication
  reform).
- **DR-pair orchestration** — per ADR-0241 substrate; no external
  vendor.
- **Health-check** — substrate's per-cell health-check primitive
  per ADR-0044.
- **Anycast routing** — Cloudflare Anycast (Year 1-2); Pingora-
  native Year 3+ per ADR-0253.
- **Audit-chain** — Merkle-sealed per ADR-0028 + ADR-0263.

## §F. Migration

### §F.1. Per-µservice rollout sequenced by criticality

| Wave | Cohort | µservices | Window |
|---:|---|---|---|
| 1 | Critical (always-on) | identity, api-gateway, edge-gateway, observability, audit-chain | 2026-06-01 → 2026-07-15 |
| 2 | High-impact | payments, intelligence, tenancy, governance, foundry, billing | 2026-07-15 → 2026-08-31 |
| 3 | User-facing | messenger, mail, notes, social, marketplace, workflow-studio | 2026-08-31 → 2026-09-30 |
| 4 | Long-tail | comms-email, connect, finops-portal, ontology, ops-dashboard-control-center | 2026-09-30 → 2026-10-31 |

### §F.2. Per-µservice migration playbook

1. Add `oya-shared-disaster-mode` + `oya-collab-crdt-portability-
   kernel` workspace dependencies.
2. Author `policy/disaster-mode.cedar` + per-pack overlays.
3. Author `iac/<env>-disaster-mode.yaml` with tier minimums per
   route + DR-pair declaration.
4. Add `§disaster-mode` to `ARCHITECTURE.md`.
5. Add `§disaster-mode-edge-cases` to `PRD.md` + `compliance.md`.
6. Add `dashboards/disaster-mode.json` + SLOs.
7. Add contract test + chaos-injection test.
8. Execute first quarterly game-day; record evidence.
9. Pass `oya-governance-disaster-mode-cell-resilience`.
10. Soak ≥ 60s; promote.

### §F.3. Per-cell rollout pattern

1. dev cells: full disaster-mode deployment 2026-05-30 → 2026-07-15.
2. staging cells: 2026-07-15 → 2026-08-31.
3. prod cells (non-sovereign): 2026-08-31 → 2026-09-30.
4. prod cells (sovereign + compliance-isolated): 2026-09-30 →
   2026-10-31.

### §F.4. What is NOT migrated

- The CRDT substrate is `oya-collab-crdt-portability-kernel`; this
  ADR depends on it but does not re-implement.
- Per ADR-0241 DR-pair topology is the topology layer.
- Per ADR-0298 emergency-services routing is the routing layer.

### §F.5. Rollback path

- Cell-tier rollback: `oya policy revert disaster-mode-v1`.
- µservice rollback: revert `policy/disaster-mode.cedar`.
- Soft-disable: `disaster_mode_enabled = false` in IaC; substrate
  emits warnings + relies on emergency-services hard-coded bypass.
- Hard-disable: drop workspace dependency (NOT recommended; loss
  of resilience invariant).

## §G. References

### §G.1. Hyperscaler precedents

- AWS Reliability Pillar documentation 2024.
- AWS Cell-Based Architecture documentation 2024.
- AWS Resilience Reports 2024.
- AWS Outposts + AWS Wavelength + AWS Local Zones documentation
  2024.
- Google "Site Reliability Engineering" 2016 (Beyer et al.).
- Google "Site Reliability Workbook" 2018 (Beyer et al.).
- Google "Borg" paper (2015) + "Omega" paper (2013).
- Microsoft Azure Region-Pair documentation 2024.
- Microsoft Substrate disclosure 2024.
- Cloudflare Workers + Pingora documentation 2024.
- Cloudflare 2024 outage post-mortems.
- Netflix "Chaos Engineering" 2018 + Chaos Monkey 2014 paper.
- Netflix ChAP documentation 2024.
- Apple Platform Security Guide 2024.
- Akamai EdgeWorkers + 2024 State of the Internet reports.
- Fastly + StackPath 2024 technical reports.
- Apple iCloud + Maps + Push regional-failover documentation 2024.

### §G.2. Standards + RFCs

- HTTP/3 + QUIC per ADR-0253 (RFC 9114, RFC 9000).
- Connection migration per QUIC RFC 9000 §9.
- TCP retransmission semantics per RFC 9293.
- DNS RFC 1035 + DNSSEC RFC 4033 (for Anycast).
- BGP Anycast practices.
- WebSocket RFC 6455 (reconnect semantics).
- Service Worker spec (W3C).
- IndexedDB spec (W3C) for offline-first.

### §G.3. Legal + compliance

- HIPAA §164.512(b) — public health emergency disclosures.
- HIPAA §164.510(b) — next-of-kin notification.
- GDPR Article 6(1)(d) — vital interests.
- GDPR Article 12(3) — extended response.
- GDPR Article 49 — derogations for important public interest.
- KR-PIPA Article 18 — disaster derogations.
- PCI-DSS Requirement 9.1.3 — incident response.
- NIS2 Article 23 — incident reporting.
- SOC 2 CC7.5 — disaster recovery.
- FEMA IPAWS standards 2024.
- FCC DIRS (Disaster Information Reporting System) 2024.
- WHO Emergency Response Framework 2024.
- EU Civil Protection Mechanism Decision 1313/2013/EU.
- KR-PMA (Public Health Crisis Management Act).

### §G.4. Internal portfolio ADRs

- ADR-0028 Audit Chain (Merkle-sealed).
- ADR-0044 Service Mesh + mTLS.
- ADR-0099 Data Class Registry.
- ADR-0105 Thirteen-Layer Canonical Enum.
- ADR-0131 Per-µservice Flat Layout.
- ADR-0140 Cedar Policy Enforcement.
- ADR-0145 Inter-Microservice Communication Reform.
- ADR-0188 Passkey/WebAuthn Canonical Auth.
- ADR-0212 Buildability Doctrine.
- ADR-0240 Sovereign-Cloud per Regional Pack.
- ADR-0241 Disaster Recovery — DR-Pair Strategy.
- ADR-0242 Oyatie is a Tenant Doctrine.
- ADR-0243 Cedar as Universal Gate.
- ADR-0244 Tenant as Universal Scoping Primitive.
- ADR-0245 Substrate vs Product Layering.
- ADR-0246 Policy Engine Substrate Promotion.
- ADR-0248 Amazon-Shape Cellular Architecture.
- ADR-0250 Build Ahead of Certification Doctrine.
- ADR-0251 Compliance Pack — Cell Certification Levels.
- ADR-0253 Network Topology — Edge + Service Mesh.
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
- ADR-0304 Cross-Jurisdiction Conflict Resolution.
- ADR-0305 Delegated-Agent Authority Chain.

### §G.5. Standards docs

- `docs/standards/documentation-rigor.md` §3.2.5 rows 14, 22, 30.
- `docs/standards/doc-style.md`.
- `docs/templates/adr-template-v2.md`.
- `docs/templates/runbook-template-v2.md`.

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
  cluster-disaster-mode-cell-resilience keystone. Closes
  documentation-rigor.md §3.2.5 rows 14 + 22 + 30. Enforcement
  advisory until 2026-10-31, BLOCKER thereafter.

---

End of ADR-0306.
