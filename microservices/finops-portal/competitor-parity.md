---
doc_id: finops-portal/competitor-parity
authored: 2026-05-18
status: ready
authority: ADR-0199 in-house roadmap + product positioning
classification: internal
---

# Competitor parity — finops-portal

This is the parity audit vs market-leading FinOps / cost-management
products. The list of competitors is wider than the hyperscaler-
native tools cited in the PRD because the parity bar is set by the
full FinOps tooling ecosystem.

## Competitor set

| Competitor              | Type            | Reference target          |
|-------------------------|-----------------|---------------------------|
| OpenCost (OSS)          | data plane      | data-plane substrate (we adopt it; ADR-0199) |
| Kubecost                | hybrid          | dashboards + chargeback   |
| AWS Cost Explorer       | hyperscaler-native | invoice + drill-down   |
| Google Cloud Billing    | hyperscaler-native | label-driven rollup    |
| Azure Cost Management   | hyperscaler-native | budgets + alerts        |
| Oracle Cost Analysis    | hyperscaler-native | compartment rollup     |
| CloudHealth (VMware)    | SaaS multi-cloud | cross-cloud cost + governance |
| Apptio Cloudability     | SaaS enterprise | mature chargeback formula |
| Cast.ai                 | SaaS opt-tool   | optimization recommendations |
| Vantage                 | SaaS startup    | modern UX + FOCUS-first   |
| Spot.io                 | SaaS opt-tool   | spot/preemptible mgmt     |
| Finout                  | SaaS modern     | shared-resource attribution |

## Feature parity matrix

| Feature                              | OpenCost | Kubecost | AWS CE | GCP Billing | Azure CM | Oracle CA | CloudHealth | Apptio | Cast.ai | Vantage | Spot.io | Finout | **finops-portal** |
|--------------------------------------|----------|----------|--------|-------------|----------|-----------|-------------|--------|---------|---------|---------|--------|-------------------|
| Per-tenant invoice                   | n/a      | n/a      | ◐      | ◐           | ◐        | ◐         | ●           | ●      | n/a     | ●       | n/a     | ●      | **●**             |
| Drill-down by cost-center            | ●        | ●        | ●      | ●           | ●        | ●         | ●           | ●      | ◐       | ●       | ◐       | ●      | **●**             |
| Drill-down by workload-class         | ●        | ●        | ◐      | ◐           | ◐        | ◐         | ●           | ●      | ●       | ●       | ●       | ●      | **●**             |
| FOCUS 1.3 native export              | ◐        | ◐        | ◐      | ◐           | ◐        | ✗         | ◐           | ●      | ✗       | ●       | ✗       | ●      | **●**             |
| Anomaly detection                    | ✗        | ●        | ●      | ●           | ●        | ●         | ●           | ●      | ●       | ●       | ◐       | ●      | **● (via Prometheus)** |
| Anomaly **explanation** (root-cause) | ✗        | ◐        | ◐      | ◐           | ◐        | ✗         | ●           | ●      | ●       | ●       | ◐       | ●      | **●**             |
| Editable cost-allocation policy      | ✗        | ◐        | ◐      | ◐           | ●        | ◐         | ●           | ●      | ◐       | ◐       | ✗       | ●      | **●**             |
| Credit / commitment ledger           | ✗        | ✗        | ●      | ●           | ●        | ●         | ●           | ●      | ●       | ●       | ✗       | ●      | **●**             |
| Signed quarterly regulator emit      | ✗        | ✗        | ✗      | ✗           | ✗        | ✗         | ◐           | ●      | ✗       | ✗       | ✗       | ◐      | **● (differentiated)** |
| Audit-chain integrity (cryptographic)| ✗        | ✗        | ◐      | ◐           | ◐        | ◐         | ◐           | ●      | ✗       | ✗       | ✗       | ✗      | **● (Ed25519 + HSM differentiator)** |
| Per-pack residency overlays          | n/a      | n/a      | ◐      | ◐           | ◐        | ◐         | ◐           | ◐      | ✗       | ✗       | ✗       | ✗      | **● (differentiated)** |
| EU AI Act capability declarations    | ✗        | ✗        | ✗      | ✗           | ✗        | ✗         | ✗           | ✗      | ✗       | ✗       | ✗       | ✗      | **● (differentiated)** |
| Workflow Studio integration          | ✗        | ✗        | ◐      | ◐           | ◐        | ✗         | ◐           | ●      | ●       | ◐       | ●       | ●      | **● (via foundry-eval)** |
| Multi-cloud cost                     | ✗        | ◐        | ✗      | ✗           | ✗        | ✗         | ●           | ●      | ●       | ●       | ◐       | ●      | **● (via OpenCost upstream)** |
| Open source                          | ●        | ◐        | ✗      | ✗           | ✗        | ✗         | ✗           | ✗      | ✗       | ✗       | ✗       | ✗      | **●**             |

Legend: ● strong / ◐ partial / ✗ missing / n/a not applicable.

## Differentiated edges

`finops-portal` reaches **competitive parity** on the hyperscaler-
native surfaces and **differentiates** on:

1. **FOCUS 1.3 native** (most hyperscaler UIs treat FOCUS as a
   bolt-on; we author the export pipeline FOCUS-first in IP-014).
2. **Signed regulator-evidence emit** (Ed25519 + HSM + audit-chain
   seal; no other competitor offers cryptographic-grade
   regulator-evidence as a first-class feature).
3. **Per-pack residency overlays** (KR, EU, US-healthcare, US-
   financial, US-public-sector each as a first-class regulatory
   pack with overlay values; no competitor offers this level of
   built-in residency).
4. **EU AI Act capability declarations** (each capability ships a
   `.capability.yaml` with `eu_ai_act.risk_class` declared per
   ADR-0083 + EU AI Act Art. 6).
5. **Workflow Studio integration** (cost anomalies route into
   workflow runs; humans + agents respond in the same surface).
6. **Open-source substrate** (we adopt OpenCost + Mimir as data
   plane; UX layer is in-house but the substrate is replaceable).

## Honest gaps

`finops-portal` is **behind** competitors on:

1. **Optimization recommendations** (Cast.ai + Spot.io). We
   surface anomalies; we don't yet recommend e.g. rightsizing.
   Roadmap: ADR-0199 §Phase 3 will add a recommendations BC.
2. **Multi-cloud cost depth** (CloudHealth + Apptio). We rely on
   OpenCost's multi-cloud coverage; we add no shim.
3. **Saas-style onboarding** (Vantage). We are a multi-tenant
   platform with per-pack onboarding; tenant self-serve onboarding
   is owned by the `tenancy` µservice, not us.

## Parity verdict

`finops-portal` is **competitive on table-stakes** AND **leading
on regulatory + compliance + cryptographic differentiators**.

## References

- ADR-0199 in-house roadmap.
- PRD §Competitive parity reference.
- FOCUS 1.3 spec: https://focus.finops.org/focus-specification/
