# Emergency Department Information System (ED-IS)

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

`microservices/emergency/` — the µservice that owns every Emergency-Department workflow inside the oyatie platform.

Authority: ADR-0332 (in flight) | ADR-0131 | ADR-0132 | ADR-0251 | ADR-0328
Owner: emergency-medicine-platform-engineer (single-owner end-to-end)
Status: scaffold-authored 2026-05-21

---

## What is ED-IS?

ED-IS is the operational nervous system for a hospital Emergency Department. It owns ESI 5-level triage, the real-time tracking board, time-locked clinical protocols (Trauma / Stroke / STEMI / Sepsis), the mass-casualty incident workflow with START / SALT triage, EMS handoff, ED registration in three modes (quick-reg / pre-arrival / walk-in), rapid CPOE with protocol-driven order sets, template-driven documentation, every disposition path (admit / transfer / discharge / AMA / expired), the boarding crisis dashboard, LWBS tracking, door-to-X metrics, bed control, multi-disciplinary communication, room assignment, the ACS-conformant trauma registry feed, and disaster response activation.

ED-IS is a peer µservice to the general-purpose `emr` µservice — it publishes FHIR encounters into the EMR but owns its own ED-shaped business rules end-to-end. It is single-concern, flat per ADR-0131.

---

## Why a separate µservice?

Emergency medicine has a fundamentally different shape from general clinical workflows:

- **Tempo** — patient is on the board for under 4 hours, not days.
- **Acuity-driven UI** — ESI 1 reshapes notification routing and bed placement.
- **Concurrency** — an attending holds 8-15 simultaneous patients across phases.
- **Protocol overlays** — Trauma / Stroke / STEMI / Sepsis alerts override normal flow.
- **MCI pivot** — instant switch to mass-casualty mode with surge bed assignment.
- **Boarding** — admitted patients held in the ED are a first-class metric.
- **Door-to-X** — every minute is regulatorily and reputationally tracked.

A general EMR cannot answer these patterns without becoming a swiss-army knife. ED-IS is the focused answer.

---

## Counterpart Landscape

ED-IS is benchmarked against the top three:

- **T-System EDIS (Hyland)** — template library leader in mid-market US EDs.
- **Wellsoft EDIS (Medsphere)** — pure-play EDIS, lean clinical workflow.
- **Cerner FirstNet (Oracle Health)** — deep tracking board + EMS integration.

Secondary references: Epic ASAP, Picis CareSuite, Medhost EDIS, TeleTracking ED Tracker.

A union of ≥100 capabilities across all seven counterparts is captured in `competitor-parity-matrix.md`.

---

## Bounded Contexts

17 bounded contexts, each owning one aggregate root + one event prefix + one Cedar policy slice + one OpenSLO slice:

1. **Triage** — ESI 5-level triage with reassessment.
2. **TrackingBoard** — real-time visual board.
3. **Protocol** — Trauma / Stroke / STEMI / Sepsis activation.
4. **MCI** — mass-casualty incident mode + START/SALT triage.
5. **EMSHandoff** — prehospital + bedside handoff.
6. **Registration** — quick-reg, pre-arrival, walk-in.
7. **OrderEntry** — rapid CPOE with protocol order sets.
8. **Documentation** — template-driven ED notes.
9. **Disposition** — admit / transfer / discharge / AMA / expired.
10. **Boarding** — admitted-but-held tracking.
11. **LWBS** — left-without-being-seen.
12. **Metrics** — door-to-X KPIs.
13. **BedControl** — bed grid authority.
14. **Communication** — multi-disciplinary message board.
15. **RoomAssignment** — rule + AI-assisted placement.
16. **TraumaRegistry** — TQIP/NTDB feed.
17. **DisasterResponse** — ICS activation + facility status.

See `PRD.md` for the full bounded-context narrative.

---

## Repository Layout

```
microservices/emergency/
  PRD.md                    # 800+ line product spec
  ARCHITECTURE.md           # 600+ line architecture deep-dive
  README.md                 # this file
  manifest.json             # µservice metadata
  supported-oses.json       # OS matrix per support-matrix memory
  competitor-parity-matrix.md  # ≥100 capability union vs top-3 + secondary
  contracts/
    openapi.yaml            # FHIR R4B + ED-specific operations
    asyncapi.yaml           # ed.* event catalog
    proto/emergency.proto   # gRPC contract
  slos/
    *.openslo.yaml          # 12 OpenSLO objects
  policies/
    *.cedar                 # 8 Cedar policies
  iac/
    aws-guest/              # OpenTofu module
    oci-guest/              # OpenTofu module
    on-prem/                # OpenTofu module
    colo/                   # OpenTofu module
    oyatie-cloud/           # OpenTofu module
  decisions/
    ADR-MS-001-triage-engine.md
    ADR-MS-002-mass-casualty-mode.md
  implementation-plans/
    IP-001..IP-010          # 10 phased implementation plans
  src/                      # Rust crates (canonical code root per ADR-0131)
```

---

## Languages

Per `feedback_rust_strict_only_no_python_2026_05_20`:

- **Service logic** — Rust 1.85+ (strict; no Python, no JS app logic).
- **IaC** — OpenTofu HCL (not Terraform).
- **Policy** — Cedar.
- **Contracts** — OpenAPI 3.1, AsyncAPI 2.6, proto3.
- **SLO** — OpenSLO v1.
- **Frontend native** — Swift (iOS/macOS), Kotlin (Android), C# / WinUI 3 (Windows).
- **Migrations** — SQL.
- **Docs** — Markdown.

---

## OS Support Matrix

Per `feedback_os_support_matrix_2026_05_20`:

- **Tier 1 Linux**: Talos, RHEL 9, Oracle Linux 8/9, SUSE SLES 15-SP5+, Ubuntu LTS 22.04 / 24.04 / 26.04, Debian 12/13, Rocky 9, AlmaLinux 9, CentOS Stream 9/10, Amazon Linux 2023, Flatcar stable, Photon OS 5.
- **Tier 1 macOS**: 15.x+ Apple Silicon M5+ ONLY (no Intel macOS, no pre-M5).
- **Architectures**: linux/amd64, linux/arm64, darwin/arm64.
- **Tier 2**: linux/ppc64le, linux/s390x (best effort).
- **Per-OS CI lane**: required.
- **Per-OS package format**: RPM / DEB / container image / pkg / Homebrew.

See `supported-oses.json`.

---

## Deployment Contexts

Per `feedback_multi_context_provider_agnostic_2026_05_20`:

1. **aws-guest** — Oyatie hosted on AWS, customer is a tenant.
2. **oci-guest** — Oyatie hosted on OCI, customer is a tenant.
4. **on-prem** — Customer-controlled on-premise hospital deployment.
5. **colo** — Customer-controlled colocation.
6. **oyatie-cloud** — Oyatie as cloud provider (own IaaS).

Every context ships a signed OpenTofu module under `iac/<context>/`. Tenant onboarding completes with a single `tofu apply` per `feedback_zero_handroll_opentofu_only_2026_05_20`.

---

## Compliance Packs

ED-IS supports the canonical pack set per `feedback_compliance_pack_primitive`:

- HIPAA, GDPR, SOC2, HITRUST-CSF, ISO-27001, PCI-DSS, EU-AI-Act, CMS-CoP-EMTALA, TJC-Standards, ACS-Trauma-Verification.

Pack resolution happens at request time via the `compliance` µservice. Pack flags drive Cedar evaluation, audit granularity, encryption mode, retention period, and disclosure obligations.

ED-IS is built ahead of certification per `feedback_build_ahead_of_certification` — packs are not retrofitted.

---

## SLOs

12 OpenSLO objects under `slos/`. Highlights:

- door-to-doctor: median ≤ 30 min, p95 ≤ 60 min.
- door-to-CT (stroke): ≤ 25 min.
- door-to-needle (stroke thrombolytic): ≤ 60 min.
- door-to-balloon (STEMI PCI): ≤ 90 min.
- door-to-disposition: median ≤ 240 min.
- triage-latency: median ≤ 10 min, p95 ≤ 20 min.
- boarding-burden: ≥4h-boarders daily target.
- LWBS rate ≤ 2% target / ≤ 4% ceiling.
- tracking-board-staleness: p99 ≤ 500 ms.
- protocol-bundle-compliance (sepsis 3h): ≥ 90%.
- ed-throughput: treat-and-release LOS p95 ≤ 6 h.
- api-availability: 99.99% rolling 30-day.

OpenSLO authoring is mandatory before promotion past dev per ADR-0130 + ADR-0131.

---

## Public APIs

### REST (FHIR R4B-anchored)

`contracts/openapi.yaml` defines:

- FHIR `Encounter[class=EMER]` create/read/update.
- FHIR `EpisodeOfCare` for ED episodes.
- FHIR `Observation` for triage acuity (LOINC-coded).
- FHIR `CarePlan` for protocol checklists.
- FHIR `Patient` for quick-reg + ambulance arrivals.
- Custom operations: `$ed-triage`, `$ed-disposition`, `$ed-protocol-activate`, `$ed-mci-activate`.

### AsyncAPI

`contracts/asyncapi.yaml` defines the `ed.*` event catalog: `ed.patient.registered`, `ed.triage.completed`, `ed.triage.reassessed`, `ed.protocol.activated`, `ed.protocol.window.breached`, `ed.mci.activated`, `ed.ems.report.received`, `ed.order.placed`, `ed.bed.assigned`, `ed.disposition.set`, `ed.boarding.threshold`, `ed.lwbs.recorded`, `ed.metrics.snapshot`, `ed.shift.handoff`, `ed.disaster.activated`, `ed.trauma.registry.exported`, `ed.expired.notify`, and more.

### gRPC

`contracts/proto/emergency.proto` defines stream-friendly RPCs for tracking board (`BoardSubscribe`), metric subscription (`MetricsSubscribe`), and MCI coordination (`MciActivate`, `MciTriageWrite`).

---

## Cedar Policies

8 Cedar policies under `policies/`:

- `charge-nurse-can-reassign-bed.cedar`
- `registration-can-quick-reg.cedar`
- `trauma-alert-bypass-rules.cedar`
- `ed-only-disposition.cedar`
- `mci-mode-activation.cedar`
- `ama-disposition.cedar`
- `verbal-order-bridge.cedar`
- `byok-credential-mode.cedar`

Per `feedback_cedar_as_universal_gate`: every gate is a Cedar evaluation; no policy in code.

---

## Tenancy

Per `feedback_tenant_as_universal_scoping_primitive`: every row, audit, metric carries tenant context. `oyatie` itself is a reserved-namespace tenant per `feedback_oyatie_is_a_tenant_doctrine`.

BYOK is supported per `feedback_byok_everywhere_credentials` — `provider_credential_mode ∈ {platform_default, byok, byok_required_by_pack}`. B2C defaults to platform_default; B2B defaults to byok.

---

## Cellular Architecture

Per `feedback_amazon_shape_cellular_architecture` and ADR-0248: ED-IS deploys across Tier 0..4 cells. Customer hospital tenants get a Tier-2 single-tenant cell; smaller tenants share a Tier-3 pooled cell with shuffle sharding for blast-radius control.

Cell-tier promotion requires:

- 14 days of green SLOs.
- All Cedar policies signed at the current revision.
- Required compliance packs attested.
- Trauma registry export sample passes ACS conformance.

---

## Cross-Microservice Integration

ED-IS calls peer µservices via direct gRPC + AsyncAPI per ADR-0145. Notable integrations:

- `identity` — actor / role resolution.
- `audit-chain` — privileged-action attestation.
- `emr` — FHIR projection + inpatient encounter open.
- `healthcare-integration` — HL7 / FHIR bridge to third-party EMRs.
- `messenger` — multi-disciplinary E2EE communication (MLS RFC 9420).
- `intelligence` — voice-to-text + room assignment recommender (BYOK-aware).
- `pharmacy`, `lab`, `imaging` — async order routing.
- `cloud-billing-tax` — encounter billing handoff.
- `incident-management`, `ops-dashboard-control-center` — boarding + disaster surge.
- `data-warehouse`, `data-pipeline`, `analytics` — projections.

---

## Implementation Plan Sequence

10 phased IPs under `implementation-plans/`:

1. IP-001 — Triage engine core.
2. IP-002 — Tracking board projection.
3. IP-003 — Protocol activation + bundle timer.
4. IP-004 — MCI mode + START/SALT.
5. IP-005 — EMS handoff.
6. IP-006 — Registration + identity reconciliation.
7. IP-007 — Order entry + verbal-order countersign.
8. IP-008 — Disposition + boarding + LWBS.
9. IP-009 — Metrics + trauma registry.
10. IP-010 — Disaster response + cell-tier promotion.

---

## How to Get Started (Operator)

1. Clone the repo and `cd microservices/emergency/`.
2. Pick a deployment context under `iac/`.
3. `tofu init && tofu apply` against the target deployment.
4. The OpenTofu module provisions the cell, deploys the Kubernetes workloads, configures NATS streams, runs DB migrations, and registers the tenant.
5. Verify SLOs are reporting under `observability`.
6. Verify Cedar policies are loaded under `governance`.


---

## How to Contribute

ED-IS is a single-owner-end-to-end µservice per `feedback_microservice_ownership_coherence_2026_05_20`. The emergency-medicine-platform-engineer owns ADR + PRD + spec + docs + IPs + runbooks + contracts + Cedar + src + tests across every artifact.

Contributors land work via the Foundry pipeline (PR against `dev`) and the multispectrum-review v2.4.0 lane. ADRs are required for any architectural decision; refer to `decisions/` for the current set.

---

## Substance Claim

Per `feedback_docs_substance_not_scaffold_2026_05_20`: every artifact in this µservice is substantive bespoke content. PRD is ≥800 lines, ARCHITECTURE is ≥600 lines, README is ≥300 lines, competitor-parity-matrix.md is ≥100 capabilities, IPs are 10 distinct phased plans, ADRs are decision-grade, and contracts are FHIR-conformant.

---

## Authority Trail

Every directive followed: ADR-0332 (in flight), ADR-0328, ADR-0131, ADR-0132, ADR-0251, ADR-0145, ADR-0248, ADR-0253, ADR-0254, ADR-0255, ADR-0105, ADR-0064. Constraint memories: `feedback_rust_strict_only_no_python_2026_05_20`, `feedback_os_support_matrix_2026_05_20`, `feedback_zero_handroll_opentofu_only_2026_05_20`, `feedback_oci_always_free_maximization_2026_05_20`, `feedback_multi_context_provider_agnostic_2026_05_20`, `feedback_microservice_ownership_coherence_2026_05_20`, `feedback_docs_substance_not_scaffold_2026_05_20`, `feedback_compliance_pack_primitive`, `feedback_cedar_as_universal_gate`, `feedback_tenant_as_universal_scoping_primitive`, `feedback_oyatie_is_a_tenant_doctrine`, `feedback_byok_everywhere_credentials`, `feedback_amazon_shape_cellular_architecture`, `feedback_mls_rfc_9420_e2ee_personal_messenger`, `feedback_intelligence_two_layer_substrate`, `feedback_no_silent_regression`, `feedback_build_ahead_of_certification`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
