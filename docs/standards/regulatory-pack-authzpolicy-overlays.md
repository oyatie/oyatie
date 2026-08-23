---
doc_class: Standard
doc_id: STANDARD-regulatory-pack-authzpolicy-overlays
status: Active
owner: council-architecture, council-privacy, ops-security, axis-regional-packs
date: 2026-05-18
authority: ADR-0148 (layered service mesh), ADR-0064 (canonical-base + localization), ADR-0145 (inter-microservice communication reform)
absorbs: ADR-0174-istio-ambient-waypoint-for-regulatory-packs (retired 2026-05-18; substantive content folded here)
related: [ADR-0044, ADR-0064, ADR-0121, ADR-0145, ADR-0146, ADR-0147, ADR-0148, ADR-0150, ADR-0171]
---

# Regulatory-pack AuthorizationPolicy overlays — universal Cilium + Istio Ambient mesh, regulatory deltas on top

## Why this standard exists (not an ADR)

The architectural decision is ADR-0148: every µservice runs on the same two-layer mesh (Cilium L3/L4 + Istio Ambient L7). Regulatory packs do NOT introduce a separate mesh, a separate dataplane, or a separate control plane. They introduce **AuthorizationPolicy CR overlays + EnvoyFilter response-shaping fragments** on the same waypoint that the µservice already runs.

This document is a STANDARD, not an ADR, because it specifies *application* of an existing architectural decision (ADR-0148) to regulatory packs — it does not itself decide anything new. ADR-0174 (the prior framing that proposed a separate regulatory-mesh decision) is retired; its substantive technical content lives here.

## Authority and scope

Authority: ADR-0148 owns the universal mesh; ADR-0064 owns the canonical-base + per-pack-localization pattern; this standard sits at the application layer where those two ADRs meet for regulated traffic. The scope is the set of µservices that handle traffic regulated by one or more of the active packs (eu, kr, us-healthcare, ksa, uae).

## What every regulated µservice ships

A regulated µservice — defined as any µservice whose `manifest.json` lists one or more entries in `regulatory_packs` that intersect with the universally regulated packs (eu, kr, us-healthcare, ksa, uae) — ships the following in addition to the universal mesh shape:

1. **`mesh_layering.ambient_waypoint: true`** in `manifest.json`. Enrolls the µservice in the Tier-3 waypoint.
2. **`iac/helm/<ms>/templates/istio-waypoint.yaml`** Helm template referencing the central `microservices/governance/iac/helm/istio-ambient-waypoint/` chart. The waypoint Helm release is pinned to Istio Ambient 1.29.2 (per ADR-0148 LTS pin; review on LTS-rotation cadence per ADR-0098).
3. **`iac/helm/<ms>/templates/regulatory-authpolicy-<pack>.yaml`** per active pack, gated by pack labels. Each file emits an `AuthorizationPolicy` v1 CR that gates a specific regulatory anchor (GDPR Art. 22, EU AI Act Annex III §4, EU DSA Art. 17, HIPAA §164.502, KR PIPA Art. 17, KSA PDPL Arts. 5/29, UAE PDPL Arts. 22-24).

## Per-pack regulatory anchor → AuthorizationPolicy overlay table

Each row is one AuthorizationPolicy CR fragment that ships as `regulatory-authpolicy-<pack>-<anchor>.yaml` under the µservice's Helm templates, gated by `{{ if .Values.regulatoryPacks.<pack>.enabled }}`.

| Pack | Regulatory anchor | Waypoint enforcement shape |
|---|---|---|
| `pack-eu` | EU AI Act 2024/1689 Annex III §4 (employment) | DENY when `request.headers[content-type]` indicates employment-context AND `principal.tenant_pack ∈ {pack-eu}` AND `action` is an auto-decision; response envelope mutated to carry GDPR Art. 22 right-to-human-review explanation per `microservices/governance/iac/kustomize/components/istio-waypoint-policies/pack-eu-annex-iii-refusal.yaml` |
| `pack-eu` | GDPR Regulation (EU) 2016/679 Art. 22 (automated decision-making) | DENY-WITH-RESPONSE-SHAPE for any automated-decision Action; response includes link to human-review endpoint per pack-eu-gdpr-art-22-routing.yaml |
| `pack-eu` | EU DSA Regulation (EU) 2065/2022 Arts. 14 / 17 / 27 (statement of reasons, transparency reports, recommender systems) | Response mutated to include the DSA Art. 17 statement-of-reasons body skeleton when content moderation Actions occur per pack-eu-dsa-transparency.yaml |
| `pack-kr` | KR PIPA Arts. 17 / 22-2 / 28 (cross-border, alternative pseudonymous, breach) | DENY cross-border egress to non-adequacy-decision destinations; ALLOW only with pseudonymous-processing claim header per pack-kr-pipa-art-17-cross-border.yaml; 통신비밀보호법 Art. 9 lawful-interception envelope honored |
| `pack-us-healthcare` | HIPAA 45 CFR §164.502 (minimum necessary) | Response body scrubbed of PHI fields per business-associate scope per pack-us-healthcare-hipaa-minimum-necessary.yaml; §164.504(e) business-associate-contract scoping enforced |
| `pack-ksa` | KSA PDPL Arts. 5 / 29 + SAMA Cyber Security Framework v1.0 | Sovereign-tier traffic routed via SEV-SNP-capable waypoint pool; SAMA-CSF monitoring header injected on every response per pack-ksa-pdpl-sovereign-routing.yaml |
| `pack-uae` | UAE Federal Decree-Law No. 45 of 2021 (PDPL) Arts. 22-24 + UAE Cybersecurity Council frameworks | Cross-border egress DENY default; data-subject-rights endpoints exempted via explicit allowlist per pack-uae-pdpl-cybersecurity-council.yaml |

## Canonical 7 AuthorizationPolicy fragments

The 7 canonical fragments live at `microservices/governance/iac/kustomize/components/istio-waypoint-policies/`:

- `pack-eu-annex-iii-refusal.yaml` (EU AI Act Annex III §4 employment refusal)
- `pack-eu-gdpr-art-22-routing.yaml` (GDPR Art. 22 automated-decision response shaping)
- `pack-eu-dsa-transparency.yaml` (DSA Arts. 14/17/27 statement-of-reasons + transparency)
- `pack-kr-pipa-art-17-cross-border.yaml` (KR PIPA Art. 17 cross-border + 22-2 alternative pseudonymous)
- `pack-us-healthcare-hipaa-minimum-necessary.yaml` (HIPAA min-necessary scrub)
- `pack-ksa-pdpl-sovereign-routing.yaml` (KSA PDPL + SAMA-CSF sovereign routing)
- `pack-uae-pdpl-cybersecurity-council.yaml` (UAE PDPL cross-border + cybersecurity council)

Per-µservice Helm templates reference these fragments via Kustomize component composition; per-pack `values.yaml` overrides supply namespace selectors, replica counts, and µservice-specific allowlist exemptions.

## Concrete µservice × pack waypoint targets (initial)

These are the µservice × pack pairs that ship Tier-3 waypoint enforcement at PR #143 landing:

- `microservices/intelligence` + `pack-eu`: Annex III §4 high-risk-request routing; rejected requests get DSA-compliant transparency response shaped at waypoint.
- `microservices/tasks` + `pack-eu` / `pack-kr`: employment-context auto-assign refusal at L7 (Cedar denies; waypoint shapes response with GDPR Art. 22 right-to-human-review).
- `microservices/mail` + `pack-eu` / `pack-kr`: KR PIPA Art. 17 cross-border egress check at L7 envelope.
- `microservices/recordings` + `pack-us-healthcare`: HIPAA minimum-necessary egress shaping (PHI scrub for non-authorized callers).
- `microservices/messenger` + `pack-us-healthcare`: PHI in DM channels routed through Cedar-gated waypoint.
- `microservices/anonymous` + `pack-eu` / `pack-kr`: legal-process-disclosure workflow gating at L7.

## RuntimeClass for waypoint pods

Waypoint pods run under the `istio-ambient-waypoint` RuntimeClass (handler `runc`) by default; sovereign-tier packs (pack-ksa-sovereign-tier, pack-uae-cybersecurity-council) upgrade to `kata-clh-sev-snp` per the ADR-0147 ladder. The RuntimeClass CR lives at `microservices/governance/iac/kustomize/components/runtime-classes/istio-ambient-waypoint-runtime-class.yaml`.

## Cedar policy compiler emit path

The governance µservice's Cedar policy compiler reads `microservices/<ms>/policy/tenant-scope.cedar` and emits two artifacts per regulated µservice:

- `CiliumNetworkPolicy` for Tier-1 L4 IDENTITY rules (universal).
- `AuthorizationPolicy` v1 for Tier-3 waypoint enforcement (per-pack regulatory deltas).

Both artifacts derive from the same Cedar source-of-truth; pack-specific Cedar policy fragments (e.g. `policy/pack-eu.cedar`) compile to pack-specific AuthorizationPolicy overlays.

## Cell-µservice scheduling awareness

Regulated-pack cells declare `cell-pack-eu`, `cell-pack-kr`, `cell-pack-us-healthcare`, `cell-pack-ksa`, `cell-pack-uae` tiers requiring Istio Ambient waypoint node-pool affinity. Tenancy refuses to bind a regulated tenant to a non-Ambient-capable cell using cloud-iac cell capability metadata and the `shuffle-sharding` crate. Documented in `microservices/tenancy/ARCHITECTURE.md#cell-assignment` and `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning`.

## Observability

Envoy access logs from each waypoint ship to the observability µservice's Tempo backend per ADR-0153, carrying regulatory anchor annotations:

- `oyatie.regulation` (e.g. `gdpr`, `eu-ai-act`, `hipaa`, `pipa`, `pdpl`, `dsa`)
- `oyatie.regulation.article` (e.g. `art-22`, `annex-iii-4`, `s164-502`)
- `oyatie.regulation.action` (e.g. `deny`, `shape-response`, `redirect-human-review`)

Auditor queries (per-pack, per-anchor) become a Tempo trace filter.

## Compliance gate

A lint gate (`presubmit` (retired CLI `gate validate layered-architecture-discipline`)) refuses to merge a µservice IP that adds a `pack-eu` / `pack-kr` / `pack-us-healthcare` / `pack-ksa` / `pack-uae` regulatory pack overlay without composing the matching waypoint Helm release and matching AuthorizationPolicy fragment. The gate also enforces the layer-boundary discipline (Cilium owns L3/L4 only; Istio Ambient owns L7 only) per ADR-0148.

## Migration from the retired ADR-0174

ADR-0174-istio-ambient-waypoint-for-regulatory-packs.md is retired and deleted; its substantive content (per-pack regulatory anchor tables, EnvoyFilter shapes, Kustomize component layout, RuntimeClass note, cell scheduling subsection) has been absorbed verbatim into this standard. Cross-references previously pointing to ADR-0174 are rewritten to point to **ADR-0148 + this standard**. The numbering note in ADR-0148 documents the absorption.

## LTS-rotation cadence

All version pins in this standard (Istio Ambient 1.29.2, Envoy waypoint binaries, Cedar 4.9.1) review on the LTS-rotation cadence per ADR-0098 (LTS pin policy). Versions current as of 2026-05-18.

## References

- ADR-0148 — service-mesh canonical: Cilium L3/L4 + Istio Ambient L7 layered (this standard's authority)
- ADR-0145 — inter-microservice communication reform (Invariants 1, 2 wired via the waypoint Cedar `ext_authz` filter)
- ADR-0064 — canonical-base + per-pack localization
- ADR-0121 — on-prem K8s stack (Cilium CNI baseline)
- ADR-0146 — distroless non-root container base image
- ADR-0147 — container sandboxing runtime ladder
- ADR-0150 — policy engine separation (Cedar app authz vs Kyverno admission)
- ADR-0171 — multi-cluster federation via ArgoCD ApplicationSets
- EU AI Act Regulation (EU) 2024/1689 — Annex III §4 employment
- GDPR Regulation (EU) 2016/679 — Art. 22 automated decision-making
- EU DSA Regulation (EU) 2065/2022 — Arts. 14, 17, 27
- HIPAA 45 CFR §164.502, §164.504(e), §164.514
- KR PIPA — Arts. 17, 22-2, 28; 통신비밀보호법 Art. 9
- KSA PDPL — Arts. 5, 29; SAMA Cyber Security Framework v1.0
- UAE Federal Decree-Law No. 45 of 2021 (PDPL) — Arts. 22-24
- Istio AuthorizationPolicy v1 — https://istio.io/latest/docs/reference/config/security/authorization-policy/
- Istio Ambient `ext_authz` — https://istio.io/latest/docs/tasks/security/authorization/authz-custom/
