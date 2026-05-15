---
purpose: Auto-backfilled purpose for ROADMAP.md
---

# Oyatie — Roadmap

## Constitutional authority — [CONSTITUTION.md](CONSTITUTION.md)


> **Status:** Draft v0.1 skeleton — 2026-05-09. The detailed v2 backlog (1500-2500 leaves) lands when the synthesis agent completes; that output becomes Section 6 of this doc.
> **Owner:** `tactical-first-vertical-pilot` until first vertical preview ships, then rolling cross-axis ownership.
> **Companion:** [`PRD.md`](PRD.md) (the *what*), [`DESIGN.md`](DESIGN.md) (the *how*), [`machine-readable/batches.json`](machine-readable/batches.json) (Foundry batch dispatch manifest).

---

## 1. The optimal-path wave sequence (canonical)

Re-iterating from [PRD.md §3.1](PRD.md). Each wave is a **gate**, not a date. Status: `Foundation → Substrate → Axis-Preview → Axis-Stable → Vertical-Preview → Vertical-Stable → Public-GA → Region-Fan-Out`. Retired milestone vocabulary does not apply.

```
W-Foundation
  ↓ (foundation ADRs Accepted; fitness fns hard-fail)
W-Foundry-Preview
  ↓ (Foundry runs ≥ N capabilities end-to-end with full evidence emission across all 3 providers in both auth modes)
  ├─→ W-Cloud-Preview (parallel) ────────────┐
  ├─→ W-SaaS-Preview (parallel) ─────────────┤
  ├─→ W-Search-Preview (parallel) ───────────┤
  │   ↓ (each axis preview gates its own forward path)        │
  │   ├─→ W-Vertical-Pilot (one design-partner vertical)      │
  │   │   ↓ (vertical pilot proves end-to-end on the stack)   │
  │   │   ↓                                                    │
  │   │   W-Vertical-Fan-Out (all 14 verticals in parallel) ──┤
  │   ↓                                                        │
  │   W-Cloud-Stable, W-Search-Stable (per axis)              │
  │   ↓                                                        │
  └─→ W-Ads-Preview ─→ W-Ads-Stable                           │
                                                              ↓
                                            W-Region-Fan-Out (parallel pack onboarding)
```

## 2. Wave details

> Each wave's gate criteria, capabilities, dependencies, batch dispatch shape.

### 2.1 W-Foundation

**Goal:** every cross-axis contract correct from day one. No product surface ships until this wave is complete.

**Gate (all must be true):**
- Data Use Boundary ADR Accepted
- Tenant kernel + Identity kernel + RBAC/ABAC (Cedar) at flat-crates target
- Audit chain (ADR-0003) hash-chain implemented + emission contract published
- Plane separation enforcement (ADR-0017) — every catalog record declares plane
- Cell architecture ADR + cell-routing primitive
- Object Graph property tiers ADR (ADR-0006..0112) all Accepted
- Eventing backbone (outbox) + topic registry
- Schema-class annotation + lint (`oya-foundry-fitness-data-class`)
- License policy ADR Accepted
- In-house build manifest ADR per axis
- Regional pack architecture ADR (canonical seams + pack contract)
- Cross-axis contract review class ADR (PR labels + CI gate)
- Architectural flattening (ADR-0015) — flat-crates guard passes: every workspace crate is under `crates/oya-*`, every workspace crate has a `registry/catalog/<crate>.yaml` record, retired top-level `modules/` / `services/` / `platform` / `tools` roots stay absent, and the role-boundary graph validates

**Foundry batches:**
- `data-use-boundary` (fanout=1; SHARED-WRITES: PRIVACY-PROGRAM.md, decisions/ADR-0008-data-use-boundary.md)
- `foundation-adrs` (fanout=4; one agent per ADR; SHARED-WRITES: ADR-INDEX.md, decisions/_index.md)
- `tenant-identity-kernel` (fanout=2; SHARED-WRITES: crates/oya-platform-tenant-kernel/, crates/oya-platform-identity-kernel/)
- `audit-chain-impl` (fanout=3; SHARED-WRITES: crates/oya-platform-audit-chain-*)
- `eventing-backbone` (fanout=2; SHARED-WRITES: crates/oya-platform-eventing-*)
- `cedar-policy-substrate` (fanout=2; SHARED-WRITES: crates/oya-platform-policy-cedar-*)
- `cell-architecture` (fanout=1; ADR + crates/oya-platform-cell-*)
- `regional-pack-architecture` (fanout=1; ADR + crates/oya-platform-regional-pack-kernel)
- `flattening-guard-ratchet` (fanout=1; guard/no-legacy-root/catalog-record evidence per ADR-0015)
- `contradiction-resolution-axis-admission` (fanout=8; the 24 HIGH contradictions from rename agent)

### 2.2 W-Foundry-Preview

**Goal:** Foundry preview spans agent runtime + foundry surfaces. Multi-provider × multi-auth verified.

**Gate:**
- SecretProvider + KMS in production
- Anthropic Claude adapter (subscription + API) operational
- OpenAI adapter (subscription + API) operational
- Google Gemini adapter (subscription + API) operational
- Provider-failover routing operational with cost ceiling enforcement
- PTY/process launch backend (no tmux dependency)
- Daemon hardening: hook_bus stale anchor, subscription_router credential shadowing, shutdown checkpoint
- Live provider smoke lane in CI
- Capability registry online with at least 50 capabilities published
- Autonomy ceiling enforcement (Cedar policy + runtime check)
- Evidence chain emission per capability invocation (ADR-0003)
- RAG endpoint exposed to Foundry-internal capabilities
- Foundry surfaces operational: repoctl, catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, plugin substrate trust gates, plugin marketplace authoring

**Foundry batches:** see [`machine-readable/batches.json`](machine-readable/batches.json) (synthesis output).

### 2.3 W-Cloud-Preview (parallel with W-SaaS-Preview, W-Search-Preview)

**Goal:** Cloud provider preview running in ≥2 regional packs in parallel (per [PRD §3.1](PRD.md)).

**Gate:**
- IAM (Cedar + SSO + STS)
- Region/AZ/cell taxonomy + cell-isolation evidence
- Compute (managed k8s + functions)
- Storage (object + block + KMS-shred)
- Network (VPC + LB + DNS + interconnect)
- Billing (per-resource metering + per-region tax-invoice format via regional pack)
- Observability (audit log + SLO dashboards)
- Cloud control-plane API frozen at v1
- ≥ 2 regional packs onboarded with full residency contracts

### 2.4 W-SaaS-Preview (parallel with W-Cloud-Preview, W-Search-Preview)

**Goal:** SaaS platform preview — workflow engine, Object Graph properties, plugin substrate, public REST API stability tier.

### 2.5 W-Search-Preview (parallel with W-Cloud-Preview, W-SaaS-Preview)

**Goal:** Search preview — pgroonga day-1 + KR/JP/EN morphology, vector index (pgvector), tenant-private indexes, RAG endpoint exposed to Foundry, per-class data boundary enforcement.

### 2.6 W-Vertical-Pilot

**Goal:** Run one vertical end-to-end on the foundation+axes preview stack as a design-partner pilot. Council picks vertical (likely `vertical-corporate` given existing depth + KR Group anchor).

### 2.7 W-Vertical-Fan-Out

**Goal:** all 14 verticals built in parallel using Foundry-authored vertical packs.

### 2.8 W-Cloud-Stable, W-Search-Stable

**Goal:** Public GA for cloud + search. Marketplace, ISV onboarding, multi-AZ failover automation, FinOps surfaces, regulator-equivalent (CSAP/ISMAP/FedRAMP/GAIA-X/MeitY/LGPD/NDMO/TDRA/IRAP) for the regions in scope.

### 2.9 W-Ads-Preview, W-Ads-Stable

**Goal:** Ads platform — internal-tenant first (preview), then external-advertiser (stable). Data Use Boundary ADR is the gate.

### 2.10 W-Region-Fan-Out

**Goal:** Add regional packs in parallel — secondary KR regions, JP-Osaka, US-West, EU-Paris, EU-Stockholm, IN-Mumbai, BR-São Paulo, KSA-Riyadh, UAE-Dubai, ANZ-Sydney, SG-Singapore, etc.

## 3. Per-axis sequencing inside each wave

> **TODO v0.2** — fill from synthesis output. Each axis gets its own decomposed batch list: capability-by-capability with dependencies + Foundry fanout target.

## 4. Cross-axis blocking matrix

Which axis blocks which other axis at which wave. Aligns with [DESIGN §10 cross-axis contract surface](DESIGN.md).

| Blocker | Blocked | At wave |
|---|---|---|
| Data Use Boundary ADR | All cloud/search/ads work | W-Foundation → W-Cloud-Preview / W-Search-Preview |
| Tenant kernel | Every other axis | W-Foundation → W-Foundry-Preview |
| Audit chain | Every regulated capability | W-Foundation → W-Foundry-Preview |
| Foundry preview (multi-provider auth) | All agent-driven work | W-Foundry-Preview → W-Cloud-Preview / W-SaaS-Preview / W-Search-Preview |
| Foundry foundry surfaces | All cross-axis CI gates | W-Foundry-Preview → all parallel previews |
| Cell architecture | Cloud + Search + Ads (anything cell-routed) | W-Foundation → W-Cloud-Preview |
| Regional pack architecture | Multi-region launch | W-Foundation → W-Region-Fan-Out |
| Plugin substrate trust gates | Marketplace + customer plugins | W-Foundry-Preview → W-SaaS-Preview |

## 5. Foundry batch shape (dispatch hints)

Per [`DOC-CATALOG.md`](DOC-CATALOG.md), each batch tag carries `fanout=N` + SHARED-WRITES line. The full machine-readable manifest is `machine-readable/batches.json`. Batch tags currently in scope (will be expanded by synthesis):

- `data-use-boundary` (fanout=1)
- `foundation-adrs` (fanout=8)
- `tenant-identity-kernel` (fanout=2)
- `audit-chain-impl` (fanout=3)
- `eventing-backbone` (fanout=2)
- `cedar-policy-substrate` (fanout=2)
- `cell-architecture` (fanout=1)
- `regional-pack-architecture` (fanout=1)
- `flattening-additive-splits` (fanout per approved split; forward-only inside `crates/oya-*` per ADR-0015)
- `foundry-mvp-chain` (sequential, fanout=1; SecretProvider → adapters → daemon → smoke → live pilot)
- `foundry-foundry` (fanout=10; repoctl + catalog + claim-ceiling + foundation-bypass + plane-gated lanes + scorecards + fitness + ADR templates + marketplace + plugin trust)
- `provider-adapter-anthropic-api`, `-anthropic-subscription`, `-openai-api`, `-openai-subscription`, `-gemini-api`, `-gemini-subscription` (fanout=6 across the 6 adapters)
- `cloud-iam`, `cloud-compute-vm`, `cloud-compute-k8s`, `cloud-compute-functions`, `cloud-storage-object`, `cloud-storage-block`, `cloud-network-vpc`, `cloud-network-lb`, `cloud-network-dns`, `cloud-billing-metering`, `cloud-billing-tax`, `cloud-observability`, `cloud-finops` (fanout per batch from cloud greenfield 299 leaves)
- `search-crawler`, `search-parser`, `search-index-inverted`, `search-index-vector`, `search-rank`, `search-qu`, `search-serp`, `search-rag` (fanout per batch from search greenfield 295 leaves)
- `ads-auction`, `ads-targeting`, `ads-attribution`, `ads-console`, `ads-publisher`, `analytics-event`, `analytics-warehouse`, `analytics-streaming`, `analytics-dp` (fanout per batch from ads-analytics greenfield 252 leaves)
- `vertical-corporate`, `vertical-healthcare`, `vertical-industrial`, `vertical-logistics`, `vertical-fintech`, `vertical-legal`, `vertical-retail`, `vertical-education`, `vertical-public-sector`, `vertical-hospitality`, `vertical-construction`, `vertical-real-estate`, `vertical-agriculture`, `vertical-food` (fanout=14 in parallel for vertical-kernels)
- `regional-pack-kr`, `regional-pack-jp`, `regional-pack-us`, `regional-pack-eu`, `regional-pack-in`, `regional-pack-br`, `regional-pack-ksa`, `regional-pack-ae`, `regional-pack-au`, `regional-pack-sg`, ... (fanout=N regions in parallel)
- `contradiction-resolution-axis-admission`, `contradiction-resolution-data-use-boundary-group`, `contradiction-resolution-cloud-sovereignty`, `contradiction-resolution-foundry-external`, `contradiction-resolution-foundry-internal` (fanout per batch from rename agent's 77 contradictions)
- `brand-rename-docs`, `brand-rename-cargo`, `brand-rename-npm`, `brand-rename-urls`, `brand-rename-ui-svelte`, `brand-rename-ui-mobile`, `brand-rename-ui-html`, `brand-rename-config-yaml`, `brand-rename-config-quadlet`, `brand-rename-config-json`, `brand-rename-rust-srv`, `brand-rename-scripts`, `brand-rename-adrs-cosmetic`, `brand-rename-design-system`, `brand-rename-doc-trees`, `brand-rename-canonical-trio` (fanout per batch from rename agent's 17 batches)
- `adr-promotion-burndown` (fanout up to 71 in parallel, but serialization on `decisions/_index.md`; actual fanout per merge-window)
- `registry-hygiene-{clients,policies,systems,prototypes,github,parked,deferred-work,foundation-bypasses,drift-defense,scope-kills}` (fanout=10)

## 6. The detailed v2 backlog

> **TODO v0.2** — Filled in when the synthesis agent completes. Will be 1500-2500 leaves with full per-leaf metadata: axis, band, cost-of-deferral, batch-tag, fanout-class, SHARED-WRITES, BLOCKS / BLOCKED-BY, structural-req, source, description.
>
> Until then: defer to the recon files at `/Users/jasonlee/oyatie/docs/raw/*.md` for the raw leaf inventories.

## 7. Highest-regret deferrals per band

> **TODO v0.2** — Filled in alongside §6.

## 8. Open user-input questions

These need a council decision before the next plan refresh:

1. Foundry rename evaluation (ADR-0006 "no Palantir vocabulary" clause) — keep "Foundry" or rename?
2. Vertical pilot vertical (which one — Corporate, Healthcare, Industrial, Logistics, Fintech)?
3. Which regional packs onboard at W-Cloud-Preview (KR / JP / US / EU candidates)?
4. Connect Personal "no ads, ever" — keep inviolable or carve a P-tier?
5. Healthcare PHI exclusion from any aggregation — confirm hard-deny defaults?
6. Search-engine consumer brand — "Oyatie Search" or separate brand for KR / Naver-class?
7. Cloud customer self-onboarding at W-Cloud-Stable — open self-serve or invite-only pilot?
8. Public ad serving at W-Ads-Stable — KR-first or global?
9. GitHub repo path — confirm `jason931225/oyatie` stays permanent?
10. Crate naming convention — `oyatie-*` or `oya-*`?
11. Defense / drone / public-safety scope — separate legal vehicle or carve out?
12. M&A posture — pursue any KR/JP cloud-native acquisitions?
13. Hardware position — accept GPU-fleet co-investment partner?
14. Founder/council membership — finalize council seat assignments?
15. Per-vertical wave-2 ordering — Manufacturing-MES, Logistics-Spine, Fintech-PG which next?
16. **`repoctl` scope split** — `repoctl` currently bundles engineer + tenant-admin + agent + ops concerns into one binary. Recommended split (council decision pending):

    | Persona | CLI |  Owns |
    |---|---|---|
    | Internal engineer + OSS contributor | `oya dev` | `dev check / push / new / lint / validate / migrate / fmt / bench` |
    | Tenant admin (customer-facing) | `oya admin` | `admin tenant / users / regions / billing / consent / dsr / break-glass` (mirror of REST) |
    | Customer builder (workflow + plugin authoring) | `oya build` | `build workflow new / plugin new / pack new / publish / sign / verify` |
    | Foundry agent (internal) | `oya agent` | `agent doctor / status / probe / dispatch / capability invoke / evidence emit` |
    | Operator (SRE / Ops) | `oya ops` | `ops cell / region / deploy / runbook / drill` |
    | Regional pack maintainer | `oya pack` | `pack build / verify / publish / install / version` |
    | Catalog + capability authoring | `oya catalog` | `catalog scaffold / promote / supersede / validate` |
    | Gates + bypasses + claim-ceiling | `oya gate` | `gate ratchet / bypass create / bypass renew / claim verify` |

    **Recommendation: ratify the split.** Rationale: clean-arch boundary per persona; each CLI loads only the deps relevant to its persona (smaller binary surface, smaller blast radius); each CLI versioned independently; each CLI documented in its own per-product PRD section. Current flat entrypoint: `crates/oya-tooling-cli-dev-runtime` exposes the `repoctl` compatibility binary for `pre-push`. Default migration path: `repoctl <cmd>` continues as a deprecated alias for ~2 waves while persona CLIs land under `crates/oya-tooling-cli-{dev,admin,build,agent,ops,pack,catalog,gate}-*`, then sunsets per ADR-0001 deprecation governance. The historical `tools/repoctl/` root is not live and must not be recreated.

(Plus the 7 in PRIVACY-PROGRAM §2.5.)

## 9. Sources scanned (footer)

- All 9 recon outputs at `/Users/jasonlee/oyatie/docs/raw/`
- All consolidated docs at `docs/`
- ADR-0050 master plan, ADR-0015 flat crates, ADR-0040 readiness pack, ADR-0017 wave framework
- v1 backlog at `~/.claude/plans/look-at-all-outstanding-buzzing-teacup.md`

*Footer regenerated whenever this doc is edited.*
