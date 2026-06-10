# 00 — ENUM CATALOG (the refined, closed fixed-vocabularies of the architecture)

> **Founder 2026-06-07: "refine the enums."** Every fixed vocabulary in the oyatie architecture, refined to ONE canonical **closed set**, drift resolved, source-cited, on-canon. This is the backbone the architecture design doc, the dashboard, the org-taxonomy, and the doc-organization all draw from. **Generated-validated + drift-gated** (D-DOCTRINE): the per-record enum membership is machine-checked; an off-enum value is a gate violation. `PENDING:<wf>` = value set converges from a running workflow (A=org-taxonomy, B=deep-research-facets, C=doc-org); marked, not invented.
>
> **Refinement principles:** closed (no open-ended "other" unless explicit) · one definition per concept (no competing enums) · on-canon (no `foundry`/`cli`/flat-`crates`/`kafka`) · drift-resolved (e.g. 6→7, application→usecase) · each enum names its **enforcement** seam (the CaC/CaaS pipeline gate, never an `oya` CLI — Δ2/Δ3).

---

## §1 — STRUCTURAL enums

### `realm` — the two overarching categories (founder 2026-06-07)
`oya-product` · `cloud`
*Every microservice belongs to exactly one realm. (D-PURESPLIT: `oya/`=products, `cloud/`=platform.)*

### `vertical_stack_layer` — the platform stack, kernel → pods (component-boundaries.md; ADR-0014/0017/0018)
`hardware` (L0) · `kernel` (L1 — framekernel, the Capsule isolation engine) · `node-os` (L2 — Talos-class) · `container-runtime` (L3 — CRI/OCI→Capsule adapter) · `container-platform` (L4–L8 — image supply-chain: content-store · snapshotter · image-spec · distribution/registry · builder/SBOM) · `kubernetes` (L9 — orchestration; schedules Capsules) · `cloud-substrate` (the substrate workloads running on k8s) · `product-pod` (oya products running as pods on the cloud, dogfooding substrates)
*Two axes meet here: L0–L9 = the owned infra stack (the 5 components); `cloud-substrate`/`product-pod` = the workload bands above it. The container-platform's INTERNAL L0–L9 sub-layering is a nested detail per ADR-0017 (`PENDING: read ADR-0017 for the exact ctrd L0–L9 split`).*

### `clean_arch_layer` — hexagonal layer per crate (ADR-0056 BNF v4.1 → REFINED via ADR-0105/0106 + D-CLOUD-NATIVE)
`kernel` · `domain` · `usecase` · `app` · `adapter` · `infrastructure` · `rest` · `grpc` · `graphql` · `worker` · `sdk`
**Refinements applied:** (1) `application` → **`usecase`** (ADR-0106 rename; 22 catalog stragglers still say `application` — complete the sweep). (2) **`cli` DROPPED** from the enum (D-CLOUD-NATIVE: cloud-native-not-CLI; the `oya-check-*`/`cli`-layer family is superseded, Δ2 — checks re-home to the CaC pipeline, not `*-cli` crates). (3) `port` is the trait seam (lives in `kernel`/`domain`), `adapter` its impl — the ports/adapters mobility pair, not separate layer tokens. *Was 12 (ADR-0056), refined to 11.*
*Enforced: the architecture/BNF gate (last-token-must-be-a-layer-value), re-homed to the CaC pipeline.*

### `capsule_isolation_strength` — the unified isolation primitive (ADR-0014/0018)
`native` · `sandbox` · `microvm` · `confidential`
*One Capsule trait + four adapters; "container/VM/pod/OS-instance" are Capsules of differing strength, selected per-workload via `RuntimeClass`.*

### `plane` — control/data/analytics separation (ADR-0017 §2)
`control` · `data` · `analytics`
*Declared at `registry/catalog/<crate>.yaml: plane:`; cross-plane change triggers cross-plane review. Strict invariant: control-plane never reads the data-plane store directly.*

---

## §2 — ORGANIZATION enums (the taxonomy spine; realm → category → group → microservice)

### `hyperscaler_category` — `PENDING:A` (org-taxonomy workflow), refined against these closed candidate sets:
- **cloud substrates:** `compute` · `storage` · `data` · `networking` · `identity-security` (IAM/IDP/KMS/secrets) · `eventing-integration` · `observability` · `containers-k8s` · `governance-policy` · `developer-cicd` · `ai-intelligence` · `billing-finops` · `tenancy-cells`
- **oya products:** `workspace-collaboration` · `workforce` (HR/payroll/accounting) · `healthcare` · `fintech-payments` · `crm-cx` · `sales-commerce` · `industrial-trade-logistics` · `data-analytics` · `search` · `ads-analytics` · `developer-platform` · `security-compliance`
*Supersedes DESIGN.md's stale "7 product axes" (which had a `Foundry` axis [→ ai-intelligence/governance], a `Cloud provider` product-axis [→ the `cloud` realm], and flat `crates/` paths). The 6-vs-7 contradiction is resolved by replacing the flat axis-list with realm × hyperscaler_category.*

### `group_kind` — the tier-2 unit
`product` · `substrate`

### `microservice_status` — build state (`PENDING:A` confirm)
`built` · `spec-only` · `stub`

### `maturity` — the maturity-spread signal — `PENDING:A` (close the value set from the real catalog)

### `edge_kind` — interrelationship edges
`orchestrates` · `serves` · `substrate-call` · `dogfood` (product-pod → cloud substrate, `wired|intended`) · `cloud-on-cloud` (substrate → substrate) · `port-adapter` · `depends`
*The Interrelationship facet's vocabulary; `dogfood` is the products-as-pods-on-cloud consumption edge. `cloud-on-cloud` added 2026-06-07 (ralplan close-out) — substrate→substrate edges (e.g. cloud-storage→cloud-kms) must NOT be counted as product dogfood; it was previously free-text in `_taxonomy.json` `note` only and would have failed enum-membership as a gate target.*

---

## §3 — DOCUMENTATION enums (the 4-tuple per-doc record; D-DOCORG, ADR-0388/0019)

### `doc_axis` — canonical doc-TYPE (ADR-0388, closed; `no-shadow-docs`)
`DECISIONS` · `PLANS`(auto-gen) · `INDEX`(auto-gen) · `SPECS-MS` · `SPECS-CRATE` · `RUNBOOKS` · `IPS` · `IDEAS`(transient, 14-day promote-or-archive)
*Allowed `docs/` subdirs are themselves a closed set: decisions · ideas · ideas/archive · conventions · machine-readable · products · site.*

### `diataxis_quadrant` (Diátaxis)
`tutorial` · `how-to` · `reference` · `explanation`

### `doc_catalog_tier` — governance weight (DOC-CATALOG)
`0` · `1` · `2` · `3` · `cross-cutting`

### `adr_status` (ADR-0388, exact case)
`Accepted` · `Proposed` · `Superseded` · `Deprecated` · `Rejected`

> **The unified per-doc record** = `{doc_axis × diataxis_quadrant × hyperscaler_category(subject) × doc_catalog_tier}` + `owner_team` + `agent_authoring_allowed` + `generated|manual`. Manual docs bind to the **ADR-0019 DOC-UPDATE-PROTOCOL**. Enforced by the doc-axis gate (CaC pipeline). Auto-gen axes (PLANS/INDEX/catalog/contracts) are generated; even if the generator fails to run, the gate blocks the drift.

---

## §4 — GOVERNANCE enums (D12 namespaced — "tier" is split, never bare)

### `autonomy_tier` (agent autonomy ceiling)
`T1` · `T2` · `T3` · `T4`

### `eu_ai_act_risk_tier`
`0` (minimal) · `1` (limited) · `2` (high) · `3` (unacceptable) · `4` (`PENDING:` confirm whether GPAI/systemic is a distinct value vs an orthogonal flag)

### `tenant_class` (retired `tenant-tier`, ADR-0329)
`PENDING:` close the value set (e.g. internal-dogfood · standard · regulated · sovereign — confirm vs source)

### `dr_tier` · `storage_tier` — `PENDING:` enumerate + close (currently open in D12; refinement = pin the value sets)

### `data_class` (ADR-0083 data-classification)
`PENDING:` close the value set (seen: `INTERNAL_ONLY` — enumerate the full ladder: PUBLIC · INTERNAL_ONLY · CONFIDENTIAL · RESTRICTED/PII · …)

---

## §5 — ARCHITECTURE-FACET enum (the design-doc views) — `PENDING:B` (deep-research converge)

Candidate closed set (idea-refine divergent, to be validated/pruned by the deep-research against 42010 / 4+1 / C4 / arc42 / Well-Architected):
`system-context` · `stack-deployment` · `organization-hierarchy` · `interrelationship` · `runtime-flows` · `data-architecture` · `security-isolation` · `reliability-failure-domains` · `performance-scaling` · `cost-finops` · `operational-observability` · `sustainability` · `ports-adapters-ratchet` · `multitenancy-dogfooding` · `compliance-owed-depth` · `governance-policy` · `program-sequencing` · `decisions-adrs` · `risk-open-questions`
*B decides which are first-class dashboard views vs doc-sections vs overlays.*

---

## §6 — REFINEMENTS APPLIED (the drift this catalog resolves)

| Enum | Stale state | Refined to |
|---|---|---|
| clean_arch_layer | `application` + `app` + `cli` (ADR-0056, 12) | `usecase`(was application) + `app`; **`cli` dropped** (cloud-native) → 11 |
| product taxonomy | DESIGN.md "7 axes" incl. **Foundry** axis + **Cloud-provider** product-axis + flat `crates/` | `realm` × `hyperscaler_category` (foundry→ai-intelligence/governance; cloud→realm; pure-split) |
| axes_count | catalog `6` vs DESIGN `7` (D-DOCORG drift) | resolved — flat axis-list retired for realm×category |
| `tier` (bare) | overloaded | namespaced (autonomy/eu_ai_act_risk/dr/storage/tenant_class) — D12 |
| doc-axis gate / auto-gen | `oya gate validate doc-axis`, `oya gen masterplan`, `oya-governance-*` (CLI) | re-homed to CaC/CaaS pipeline (Δ2/Δ3); ADR-0388/0019 = AMEND |

## §7 — ENFORCEMENT (every enum is gate-checked; no enum is advisory)

Each closed set is validated in the **CaC/CaaS pipeline** (GitHub-Actions live / oya-ci shadow), never an `oya` CLI: BNF/architecture gate (clean_arch_layer) · doc-axis gate (doc enums + `no-shadow-docs`) · brand-residue gate (forbids the eradicated terms so they can't re-enter any enum) · cross-artifact-agreement gate (status/edge enums + reciprocal edges). **Two-layer defense:** the generator writes enum-valid records; the gate blocks any off-enum value even if the generator never ran (D-DOCTRINE; founder 2026-06-07 defense-in-depth).
