# Realignment Review — 2026-05-21 mid-Wave-4-rolling

This is the orchestrator-authored cross-cutting analysis of 48 µservice audits completed during the realignment session. It is durable on disk + sister file `.omc/state/wave-findings-aggregation-2026-05-21.md` holds the per-µservice findings rollup.

## Coverage snapshot

| | Count | % |
|---|---:|---:|
| Audited (as of review) | 48 | 62% |
| In flight at review time | 23 (10 codex + 10 Claude + 3 Claude R2 already done before this review) | 30% |
| Queue remaining post-dispatch | 9 | 12% |
| Total active µservices | 77 | 100% |
| Total findings cataloged | ~858 | |
| Lines of audit content authored | ~75,000+ | |

## Six cross-cutting patterns (with provenance citations)

### Pattern 1 — Tier-retirement candidates scale much higher than projected

Sample tier-retirement candidate counts from agent self-reports:

| µservice | Tier refs found |
|---|---:|
| shorts | 155 |
| mail | 73 |
| community | 56 |
| workflow-studio | 48 |
| workflow-engine | 47 |
| ontology | 45 |
| api-gateway | 43 |
| messenger | 8 files |
| network | 35 |
| intelligence | 29 |
| developer-sdk | 28 |
| application | 27 |
| cloud-billing-tax | 12 |
| cloud-billing | 10 |
| feature-flags | full capability-tiers/ folder |
| marketplace | 5 files |
| crm | ~104 (23 + 81 stamped) |

**Extrapolation**: 48 audited × ~35 avg ≈ **1,680 distinct tier call-sites**. Wave 15J retirement scope = larger than the original 9,300-Bronze-occurrences scope-audit suggested. (Original counted character occurrences; this counts distinct call-sites needing remediation.)

**Recommendation**: Wave 15J will not be feasible as a single manual sweep. Author an `oyatie-tier-retirement` Rust crate that:
- Detects tier-vocabulary patterns (Bronze/Silver/Gold/Platinum + capability_tiers field references + tier_classification field + ADR-0316 citations)
- Suggests replacements in canonical doctrine language (deployment_context overlay / tenant_class overlay)
- Generates a per-µservice retirement-PR per-µservice
- Runs as a CI lane against any future tier-vocabulary regression

### Pattern 2 — Tenant-class adoption gap is UNIVERSAL

All 48 audited µservices flag the gap. Legacy vocabularies observed:

| Legacy vocabulary | µservices using it |
|---|---|
| `{free/paid/starter/pro/enterprise}` | Most common pattern (mail, community, etc.) |
| `{trial/sandbox/production/internal-foundry}` | intelligence |
| `{max_tier}` contract field | ontology |
| `partner_tier` | api-gateway |
| `tier_classification + capability_tiers + criticality_tier` (3 fields) | crm |

**ZERO µservices have the new `{demo_trial, paid}` + composable `billing_components` ⊆ `{revenue_share, per_seat, per_usage}` model adopted.**

**Recommendation**: Author a cross-µservice tenant-class adoption ADR (e.g., ADR-0331) BEFORE Wave 15A starts. Single doctrine source per the new tenant-class memory + a per-µservice plumbing IP rather than 77 ad-hoc fixes. Key elements:
- `tenant_class` claim binding in `cloud-iam` + `identity`
- `billing_components` context attribute (consumed by `cloud-billing`)
- Cedar policy gate templates (e.g., `tenant_class == paid && contract.allow_billing_component(BC)`)
- Per-µservice IP template for converting legacy free/paid/tier vocabulary

### Pattern 3 — Kernel-ahead-of-spec anti-pattern (cloud-billing)

cloud-billing has a **1,030-line hyperscaler-grade Rust kernel** but PRD/ARCHITECTURE/README/contracts/SLOs **ALL ABSENT**. This is the OPPOSITE of the typical substance-gap (where docs exist but kernel is stub).

Other µservices show the typical inversion (docs ahead, kernel behind). Suggests cloud-billing's kernel author bypassed canonical spec authoring.

**Provenance**: `microservices/cloud-billing/` directory inventory + cloud-billing Claude Round 1 orchestrator report; verdict REVISE; 12 P0s tied to missing spec surface.

**Recommendation**: cloud-billing needs a focused **spec-authoring sprint** in Wave 15A, NOT remediation of the kernel. Kernel is already substance-grade. Spec must catch up to kernel.

### Pattern 4 — Industrial-scale template-stamping (crm)

crm contains pervasive stamped-loop content:
- README.md: 169 identical "README evidence row NNN" lines
- ARCHITECTURE.md §H: 90 identical "Architecture trace NN" lines
- competitor-parity-matrix.md: 327 stamped Row entries
- PRD.md §C: 30 stamped user stories

Per ADR-0324 anti-stamping doctrine + ADR-0328 §D-20.111-115 Big-8 P0 elevation, every stamped row = P0. Result: **94 P0s in one µservice** = 78% of all P0s found across the entire realignment so far.

This is the worst µservice quality in the corpus. Lane 2 trace identified surface-wave coordination as the causal pattern; crm is the proof-by-existence of how bad that pattern can get.

**Recommendation**: crm needs **rewrite, not remediation**. Specifically:
- Discard the stamped content (README, ARCHITECTURE §H, parity matrix, PRD §C)
- Rebuild with substantive content authored by a single owner agent
- Use Big-8 family proper counterpart anchors (Salesforce as #1 anchor, not #3; HubSpot CRM as second-anchor; Microsoft Dynamics 365 Sales with current naming)
- Author missing CRM primitives: CPQ, Sales Cadences, Lead/Opportunity AI scoring, Reports primitive, Mobile CRM (Swift+Kotlin), Lead bounded context, Contact bounded context, OpportunityTeam/OpportunitySplit, Quote-to-Cash flow, Custom Objects/Fields extensibility
- Treat crm as its own Wave 15A-crm-rewrite sub-wave (not bundled with general remediation)

### Pattern 5 — Doctrine evolution within session created stale audits

Doctrine drifted (added constraints + retired old ones) DURING the session, with downstream effects on what each audit could detect:

| Cohort | Time window | Doctrine in force |
|---|---|---|
| Wave 2 Batch 2.1 | 22:33 → 23:04 | 5 cross-cutting constraints (no tier-retirement / no tenant-class) |
| Wave 3 Batch 3.1 | 23:09 → 23:40 | Same 5 (added Leptos web mid-flight, but not exposed in audit prompts) |
| Wave 3 Batch 3.2 | 23:42 → 00:15 | 5 + tier-retirement guard (3 deliverables, no tier-deltas) |
| Claude Round 1 | 00:11 → ~01:00 | 5 + tier-retirement + tenant-class model (3 classes initially, then 2 corrected) |
| Wave 4 rolling codex | 00:10 → ~ | Mixed (some used old 4-deliverable template, some used 3-deliverable) |
| Claude Round 2 | ~00:30 → ~01:00 | 5 + tier-retirement + tenant-class (2-class corrected) + Leptos + selective-hydration + Stainless + C/C++ |
| Recovery (messenger) | ~01:30 → ~02:00 | All previous + mobile-app-bundle directive |

**Earlier-audited µservices do NOT reflect later directives.** Their findings still hold (they're more conservative — they didn't try to flag what wasn't in their prompt), but the directives become Wave 15J/K/L retirement candidates anyway.

**Recommendation**: Do NOT re-audit earlier µservices. The findings are valid. Wave 15J/K/L sweep handles the missed directives consistently across ALL µservices in remediation phase. The "stale audit" risk is bounded.

### Pattern 6 — Counterpart-assignment errors detected by agents themselves

| µservice | Wrong counterparts assigned | Correct counterparts | Remediation |
|---|---|---|---|
| `network` | AWS VPC Lattice / GCP Cross-Cloud Network / Azure Virtual WAN (networking infra) | LinkedIn / X / Threads (actual µservice is LinkedIn-class professional network) | Wave 15K: retire `network`, merge into `community` |
| `cell` | AWS Cell-Based / Google Distributed Cloud / Fastly Edge (edge cloud products) | Cellular architecture pattern (not a customer-facing product to compare) | Wave 15L: retire `cell`, absorbed by tenancy + cloud-iac + observability + Rust crate |
| `cloud-billing-tax` | Stripe Tax / Avalara / TaxJar (3 counterparts in dispatch) | Existing benchmark used 5 (added Vertex + Sovos) | Recorded as F-DIM4-01 / F-DIM5-01 disagreement per ADR-0328 §D-5.3 |
| `crm` | Counterpart ORDERING: Salesforce Sales Cloud as anchor (per dispatch) | crm artifacts treat Salesforce as #3 not anchor; HubSpot ABSENT from matrix/README/ARCHITECTURE | Wave 15A-crm-rewrite: pivot to proper Big-8 counterpart ordering |

**Recommendation**: When dispatching audits, the orchestrator MUST verify the µservice's actual purpose matches the assigned counterparts before dispatch. Lane 2 trace identified surface-wave coordination as the cause; this is a related orchestrator-side bug (insufficient pre-dispatch verification).

## P0 Prioritization for Wave 15A

```
crm           : 94 P0s  ← needs rewrite, not remediation (Wave 15A-crm-rewrite sub-wave)
cloud-billing : 12 P0s  ← kernel-ahead-of-spec; spec-authoring sprint
marketplace   :  7 P0s  ← 6-category surfaces + revenue_share completion
identity      :  4 P0s  ← T0 substrate OpenTofu/OS/multi-context gaps
messenger     :  3 P0s  ← iac/terraform/ rename + mobile-bundle coordination + 6-context iac/
──────────────────────
TOTAL         : ~120 P0s across 5 µservices (current state)
```

Crm = ~80% of remediation effort. Spec-authoring sprint for cloud-billing = ~10%. Remaining = ~10%.

## Notable quality outliers

| µservice | Outlier signal | Root cause |
|---|---|---|
| shorts | 155 tier references — worst tier entrenchment | Historical Bronze/Silver/Gold/Platinum creator-monetization tiers baked deep |
| mail | 73 tier refs in onboarding/playbooks/benchmarks | Historical email-tier framing (free Gmail / paid Workspace) |
| workflow-engine + workflow-studio | ~47-48 tier refs each | n8n-class historical pricing tiers carried over from Bominal inheritance |
| crm | 113 findings, 94 P0s | Template-stamping at industrial scale |
| cloud-billing | 12 P0s, kernel hyperscaler-grade | Spec authoring skipped while kernel was built |
| cloud-network | Only 19 total findings | µservice is undersized (limited artifacts); not a quality signal |
| intelligence | Only 13 findings despite complexity | Either mature OR prompt didn't probe deeply enough |
| marketplace | 5 SUPERIOR capabilities preserved (single cross-category ledger, BLAKE3 audit-chain, EU AI Act pack readiness, Cedar default-deny, single event schema) | High-quality kernel design |
| feature-flags | F-COH-010: uses HashiCorp Terraform (not OpenTofu) | Forbidden-engine violation per ADR-0328 §D-16 |
| messenger | F-MSGR-002: directory still named `iac/terraform/` | Forbidden engine name (cosmetic + symbolic) |

## Risks for remaining work

1. **Long-tail Phase 4 verticals** (real-estate, plant-maintenance, etc.) may be stub-only. Full 600+400+300-line audit may be overkill — a leaner "scope present + counterpart anchor" check might suffice.
2. **Wave 15A crm rewrite scope is enormous** — may need its own multi-batch plan.
3. **Cross-µservice tenant_class adoption** needs ONE doctrine ADR + per-µservice plumbing IP, not 77 ad-hoc fixes.
4. **Doctrine evolution within session** means earlier audits used older constraint sets; Wave 15J/K/L sweep must apply final doctrine consistently across ALL µservices.
5. **Wave 14 final aggregation** will need substantial polish — the running tally in `wave-findings-aggregation-2026-05-21.md` is per-cohort; final must be per-µservice + per-phase + per-finding-category + per-remediation-route.

## Recommendations going forward

1. **Author ADR-0329-tier-system-retirement** as the canonical retirement ADR (supersedes ADR-0316). Pair with ADR-0330-tenant-class-replacement-model (codifies {demo_trial, paid} + composable billing_components). Both should land BEFORE Wave 15A.

2. **Author ADR-0331-cross-µservice-tenant-class-adoption** prescribing the per-µservice plumbing IP template (tenant_class claim binding, billing_components context attribute, Cedar policy gates, demo_trial cap-breach flow).

3. **Build `oyatie-tier-retirement` Rust crate** for automated tier-vocabulary detection + suggestion + per-µservice retirement PR generation. The 1,680+ tier call-sites are too many for manual scrubbing.

4. **Wave 15A-crm-rewrite as its own sub-wave**: crm has 94 P0s requiring rewrite-not-remediation. Bundle CRM's reconstruction into a dedicated wave with proper Big-8 counterpart ordering + missing CRM primitives authoring (CPQ, Sales Cadences, etc.).

5. **Wave 15B-cloud-billing-spec-sprint**: focused authoring sprint for cloud-billing's missing PRD/ARCHITECTURE/contracts/SLOs/Cedar — the kernel is already substance-grade, so the work is documentation-around-kernel.

6. **Wave 15K and 15L** as already-defined merge/retirement sub-waves for network→community and cell-retirement.

7. **Phase 4 long-tail leaner audit format**: for stub µservices (real-estate / plant-maintenance / production-planning / quality-management / supply-chain-planning / treasury / warehouse / global-trade / contact-center), consider a 150-line "scope present + counterpart anchor + buildability assessment" instead of the full 600+400+300 audit triple. Saves codex compute for genuine remediation work.

8. **Orchestrator pre-dispatch verification**: before dispatching µservice audits, the orchestrator must read the µservice's PRD (if present) and verify the assigned counterparts match the µservice's actual purpose. Lane 2 trace identified this as a coordination gap; the next-round dispatcher should enforce a pre-dispatch "PRD-counterpart match" check.

9. **Wave 14 aggregation polish**: at the end of all µservice audits, the running aggregation file should be promoted to a canonical Wave 14 deliverable with per-phase rollup + remediation-backlog routing to Wave 15A/B/J/K/L sub-waves.

## Files this review references (durable)

- `.omc/state/wave-findings-aggregation-2026-05-21.md` — per-µservice findings tally
- Per-µservice `microservices/<name>/coherence-audit-2026-05-20.md` — primary findings source
- Per-µservice `microservices/<name>/feature-parity-matrix-2026-05-20.md` — counterpart UNION-coverage
- Per-µservice `microservices/<name>/performance-benchmark-numbers-2026-05-20.md` — per-context + per-tenant-class targets
- Per-µservice `microservices/<name>/capability-tier-deltas-vs-counterparts-2026-05-20.md` — for Wave 2 + Wave 3 B1 only; dropped from Batch 3.2 onward
- ADR-0328 + master-plan-sequencing.json + brief-template.md — canonical doctrine
- 10 constraint memory files at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_*_2026_05_*.md`

## Snapshot of audit dispatch summary at this review point

| Wave | µservices | Mechanism |
|---|---:|---|
| Wave 2 Batch 2.1 | 8 | codex |
| Wave 3 Batch 3.1 | 8 | codex |
| Wave 3 Batch 3.2 | 5 | codex |
| Wave 4-rolling Claude R1 | 3 | Claude |
| Wave 4-rolling codex | 20 | codex (broken dispatcher) |
| Wave 4-rolling Claude R2 | 3 | Claude |
| Wave 4-rolling recovery | 1 | Claude (messenger) |
| **Subtotal audited** | **48** | |
| Wave 4-rolling Round 3 (dispatched at review time) | 20 | 10 codex + 10 Claude (parallel) |
| Long-tail remaining post-R3 | 9 | TBD |
| **TOTAL TARGET** | **77** | |
