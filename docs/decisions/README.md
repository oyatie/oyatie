---
doc_status: published
---
# Oyatie ADR Pack

> **Status:** Draft v0.1 — 2026-05-09. The new self-contained ADR pack per user directive 2026-05-09: "I want a consolidated ADR pack. that means no reference to our old ADRs."
> **Authority:** This pack is the authoritative decisions ledger going forward. Legacy `decisions/ADR-####-*.md` files are RETIRED — they remain in the directory for forensic / git-blame integrity but are NOT referenced from active consolidated docs.
> **Location:** All new ADRs live at `docs/decisions/ADR-####-<slug>.md`.
> **Owner:** `crew-adr-promotion` + `council-architecture`.

---

## 1. Pack contents (50 ADRs target; ~21 currently authored)

### Foundation (ADR-0001..0019)

| # | Slug | Status | Owner |
|---|---|---|---|
| 0001 | cohesion-thesis-one-product-seven-axes | Proposed | council-architecture |
| 0002 | tenant-and-identity-kernel | Proposed | platform-tenancy-identity |
| 0003 | audit-chain-and-evidence-emission | Proposed | platform-audit-evidence |
| 0004 | plane-separation-control-data-analytics | Proposed | council-architecture |
| 0005 | eventing-backbone-outbox-pattern | Proposed | platform-eventing-og |
| 0006 | object-graph-and-property-tier-model | Proposed | platform-eventing-og |
| 0007 | cedar-authorization-policy-and-persona-tier | Proposed | platform-tenancy-identity + council-privacy |
| 0008 | data-use-boundary | Proposed | council-privacy |
| 0009 | cell-architecture-per-tenant-per-region | (in flight) | cloud + council-architecture |
| 0010 | regional-pack-architecture | (in flight) | regional-packs |
| 0011 | cross-microservice-contract-registry | (in flight) | council-architecture + foundry |
| 0012 | axis-admission-protocol | (in flight) | council-architecture + founder |
| 0013 | product-license-policy | (in flight) | council-architecture + ops-security |
| 0014 | build-vs-buy-policy | (in flight) | council-architecture |
| 0015 | architectural-flattening-target | (in flight) | council-architecture |
| 0016 | wave-and-plane-integration-framework | (in flight) | council-architecture |
| 0017 | brand-naming-and-repo-layout | (in flight) | council-architecture + founder |
| 0018 | glossary-and-terminology-canon | (in flight) | council-architecture |
| 0019 | doc-catalog-and-update-protocol | (in flight) | council-architecture |

### Foundry (ADR-0020..0027)

| # | Slug | Status | Owner |
|---|---|---|---|
| 0020 | foundry-multi-provider-adapter-model | Proposed | foundry |
| 0021 | foundry-capability-registry-and-mcp-gateway | Proposed | foundry |
| 0022 | autonomy-ceiling-runtime-enforcement | Proposed | foundry + council-privacy |
| 0023 | foundry-sandbox-wasmtime-firecracker | Proposed | foundry + ops-security |
| 0024 | foundry-eval-harness-and-replay | Proposed | foundry |
| 0025 | foundry-as-engineering-platform | Proposed | foundry |
| 0026 | in-house-ai-model-substrate-roadmap | Proposed | foundry + cloud |
| 0027 | robotics-vision-speech-sub-substrates | Proposed | foundry + vertical-industrial |

### Axis + cross-cutting (ADR-0028..0050)

| # | Slug | Status | Owner |
|---|---|---|---|
| 0028 | cloud-provider-architecture | Proposed | cloud |
| 0029 | workspace-productivity-suite-architecture | Proposed | axis-workspace |
| 0030 | search-engine-architecture | Proposed | axis-search |
| 0031 | ads-and-analytics-architecture | Proposed | axis-ads-analytics |
| 0032 | dcim-software-for-own-dc-ops | Proposed | cloud |
| 0033 | vertical-industry-cloud-pack-architecture | (in flight) | per-vertical |
| 0034 | per-vertical-data-class-overrides | (in flight) | council-privacy + per-vertical |
| 0035 | workflow-engine-state-machine-and-dag-hybrid | (in flight) | foundry + axis-saas |
| 0036 | plugin-substrate-wasm-and-trust | (in flight) | axis-saas + foundry + ops-security |
| 0037 | public-api-stability-tiers-and-deprecation | (in flight) | platform-api-sdk |
| 0038 | trust-framework-and-dsr-cascade-and-proof-of-erasure | (in flight) | council-privacy + platform-audit-evidence |
| 0039 | supply-chain-security-trivy-cosign-sbom-signed-commits | (in flight) | ops-security |
| 0040 | progressive-delivery-canary-blue-green-metric-gated-rollback | (in flight) | ops-sre-reliability |
| 0041 | gitops-trunk-based-and-release-branch-cut-at-tag | (in flight) | ops-sre-reliability + foundry |
| 0042 | observability-stack-otel-and-in-house-ui | (in flight) | ops-sre-reliability |
| 0043 | secrets-management-openbao-and-hsm-per-cell | (in flight) | ops-security + cloud |
| 0044 | service-mesh-istio-ambient-and-envoy-gateway | (in flight) | cloud |
| 0045 | database-tier-strategy | (in flight) | cloud + platform-eventing-og |
| 0046 | vector-store-strategy | (in flight) | platform-eventing-og + axis-search |
| 0047 | search-backend-strategy | (in flight) | axis-search |
| 0048 | korean-morphology-and-multilingual-tokenization | (in flight) | axis-search + regional-packs |
| 0049 | cross-region-replication-and-residency | (in flight) | cloud + regional-packs + council-privacy |
| 0050 | automation-first-pipeline | (in flight) | foundry |

## 2. Citation rules (effective 2026-05-09)

**Active consolidated docs MUST cite ADRs from this pack ONLY** (`ADR-0001..0051`, plus future pack ADRs after their files exist). Citations to legacy `decisions/ADR-####-*.md` files are forbidden in active docs. CI lane `oya-foundry-fitness-adr-citation` enforces.

**Forensic mention** of legacy ADR numbers is allowed only in:
- `ADR-CONSOLIDATION-PLAN.md` (the meta-doc that documents the consolidation)
- `ADR-LEGACY-REGRESSION-MAPPING.md` (the legacy-to-new-pack evidence map)
- `CONTRADICTION-LEDGER.md` (forensic ledger; legacy refs noted as "(legacy)")
- `decisions/RETIRED.md` (the retirement note)

## 3. Pack governance

- New ADR proposed via PR to `docs/decisions/`
- Status: Proposed → Accepted → (rare) Superseded by newer pack ADR
- Per-ADR Owner team approves; council co-signs for cross-cutting
- Per-PR ADR template adherence checked by `oya-foundry-fitness-adr-template`
- Per-ADR supersession back-link required

## 4. Per-ADR template

See [`docs/templates/adr-template.md`](../templates/adr-template.md).

## 5. Sources

- User directive 2026-05-09: "I want a consolidated ADR pack. that means no reference to our old ADRs."
- [`docs/ADR-CONSOLIDATION-PLAN.md`](../ADR-CONSOLIDATION-PLAN.md)
- [`docs/ADR-INDEX.md`](../ADR-INDEX.md) — to be regenerated as the index of THIS pack (the prior 127-ADR index will be retained as `ADR-INDEX-LEGACY.md` for forensics)
- [`docs/raw/codex-verdict.md`](../raw/codex-verdict.md) §16-§18 (license + build-vs-buy + ADR consolidation feedback)
