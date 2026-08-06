---
id: ADR-0016
status: Accepted
doc_status: published
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Wave/plane integration framework

# ADR-0016: Wave and plane integration framework — descriptive wave names (W-Foundation through W-Region-Fan-Out), per-wave gate criteria, status labels (preview / stable / GA), no M0/M1/M2/M3/minimum-shippable-tier vocab

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture` + `tactical-first-vertical-pilot` until first vertical preview ships
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0004, ADR-0010, ADR-0011, ADR-0012, ADR-0017, ADR-0018, ADR-0019

---

## Context

The legacy `M0..M3 / minimum-shippable-tier` milestone vocabulary baked in a date-bound + commercial-launch-bound mental model that no longer matches the optimal-path framing under unconstrained time/resource (PRD §3.1). The 2026-05-09 reframing retired the milestone vocabulary in favor of *descriptive wave names* that name the substrate-or-axis a wave delivers and *industry-standard status labels* (`preview / stable / GA`) for surface maturity. Without an authoritative ADR in this Foundation pack that pins the wave names + gate criteria + the forbidden vocabulary, every roadmap reference becomes a re-litigation point.

A second pressure: each wave is a *gate*, not a date. Gate criteria need to be enumerable (so a council can answer "did W-Foundry-Preview ship?"), composable (waves run in parallel after their prerequisites), and auditable (gate evidence emits to the audit chain per ADR-0003). Without explicit gate criteria, "preview" and "stable" become marketing-shaped, not engineering-shaped.

---

## Decision

We adopt **descriptive wave names**, **per-wave gate criteria**, **`preview / stable / GA` status labels**, and explicitly forbid `M0..M3 / minimum-shippable-tier` vocab (ADR-0018 enforces this in the glossary fitness lane).

### Wave names (canonical)

| Wave | Description |
|---|---|
| **W-Foundation** | Foundation correctness: tenancy + identity kernel (ADR-0002), audit chain (ADR-0003), plane separation (ADR-0004), eventing backbone (ADR-0005), Ontology + property tiers (ADR-0006), Cedar + autonomy ceiling (ADR-0007), Data Use Boundary (ADR-0008), cell architecture (ADR-0009), regional pack architecture (ADR-0010), cross-microservice contract registry (ADR-0011), license policy (ADR-0013), build-vs-buy (ADR-0014), flat-crates (ADR-0015) |
| **W-Foundry-Preview** | Foundry preview: SecretProvider/KMS, multi-provider adapter (Claude/OpenAI/Gemini × subscription + API), capability registry, autonomy ceiling enforcement, evidence emission, RAG endpoint, foundry surfaces (catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, repoctl, scorecards, fitness functions, supply-chain Cosign+Trivy+SBOM) |
| **W-SaaS-Preview** | SaaS platform preview: workflow engine, Ontology property tiers, plugin substrate (signing + sandbox), public REST API stability tier, webhook signing, plugin marketplace catalog |
| **W-Workspace-Preview** | Workspace / Productivity Platform preview: Mail / Docs / Sheets / Slides / Drive / Calendar / Meet / Chat / Forms / Sites / Tasks / Notes / Translate / Recordings |
| **W-Cloud-Preview** | Cloud provider preview: IAM (Cedar + SSO + STS), region/AZ/cell taxonomy, compute (managed k8s + functions), storage (object + block + KMS-shred), network (VPC + LB + DNS + interconnect), billing (per-region tax-invoice via regional pack), observability |
| **W-Search-Preview** | Search preview: pgroonga day-1, KR/JP/EN morphology, inverted index sharding, vector index (pgvector), tenant-private indexes, RAG endpoint to Foundry, per-class data boundary enforcement |
| **W-Vertical-Pilot** | First vertical end-to-end as design-partner pilot |
| **W-Vertical-Fan-Out** | Additional verticals built in parallel |
| **W-Cloud-Stable** | Public cloud-provider GA: marketplace, ISV onboarding, multi-AZ failover automation, FinOps surfaces, regulator-equivalent (CSAP / ISMAP / FedRAMP / GAIA-X / MeitY / LGPD / NDMO / TDRA / IRAP) |
| **W-Search-Stable** | Public web search (crawler + freshness + KG + SERP); sponsored-result slot infrastructure ready (ad serving still off) |
| **W-Ads-Preview** | Internal ad-serving + advertiser console preview; tenant-facing-only at first |
| **W-Ads-Stable** | External ad platform serving advertisers outside the current tenant base |
| **W-AI-Model-Substrate** | In-house model training + inference substrate (long-horizon, post Vertical-Stable) |
| **W-Robotics-Vision-Speech** | Vision + Speech + Robotics intelligence substrate (long-horizon; runs in parallel with verticals that consume) |
| **W-DataCenter-Operations** | DCIM, BMS/BAS, power/cooling/network ops, asset lifecycle, sustainability (long-horizon, post W-Cloud-Stable) |
| **W-Region-Fan-Out** | Adds regional packs in parallel — secondary KR regions, JP-Osaka, US-West, EU-Paris, EU-Stockholm, IN-Mumbai, BR-São Paulo, KSA-Riyadh, UAE-Dubai, ANZ-Sydney, SG-Singapore, … |

### Per-wave gate criteria (illustrative for W-Foundation; all waves follow the same shape)

A wave's gate is met when *all* are true:

1. Each substrate / axis ADR in the wave's scope has Status: Accepted.
2. The wave's CI lanes hard-fail on violations (e.g. for W-Foundation: `oya-governance-cohesion`, `-data-class`, `-rls`, `-architecture`, `-license`, `-build-vs-buy`, `-contracts`).
3. Per-wave evidence pack emitted to the audit chain (ADR-0003); per-wave trust-portal artifact published.
4. Per-wave council review records gate decision; council-architecture chair signs `EVT-WAVE-GATE-PASSED`.

### Status labels (industry standard)

- `preview` = surface is operational with a known set of design partners; SLO not committed externally; per-tenant uplift possible per request.
- `stable` = surface is operational at SLO; public availability open; deprecation governance applies (ADR-0019).
- `GA` = stable + per-pack regulator-equivalent evidence + customer-facing SLA committed.

A surface MAY skip `preview` and ship directly to `stable` (e.g. supply-chain signing / Cosign was external; we adopt and ship stable). A surface that ships at `preview` cannot claim `stable`-tier guarantees on its catalog record.

### No M0/M1/M2/M3/minimum-shippable-tier vocab

The tokens `M0`, `M1`, `M2`, `M3`, `minimum-shippable-tier`, `milestone-zero`, `milestone-one` (and case variants) are **forbidden** in any consolidated doc, ADR, PRD, runbook, README, or PR title from the date this ADR is Accepted. The glossary fitness lane (`oya-governance-glossary`, ADR-0018) detects them and warns at PR open + fails at merge. Legacy ADRs that mention them are forensic only — superseded language stays in the historical record.

### Wave parallelism

Waves run in parallel after their prerequisites:

```
W-Foundation
  ↓
W-Foundry-Preview
  ├─→ W-Cloud-Preview ──┐
  ├─→ W-SaaS-Preview ───┤
  ├─→ W-Workspace-Preview
  ├─→ W-Search-Preview ──┤
  │     ↓                 │
  │   W-Vertical-Pilot   │
  │     ↓                 │
  │   W-Vertical-Fan-Out │
  ↓                       │
W-Cloud-Stable, W-Search-Stable
  ↓                       │
W-Ads-Preview ─→ W-Ads-Stable
                          ↓
                W-Region-Fan-Out (parallel pack onboarding)
```

`W-AI-Model-Substrate`, `W-Robotics-Vision-Speech`, `W-DataCenter-Operations` run as long-horizon tracks in parallel with the main thread once their prerequisites land.

### Boundary

- Applies to: every consolidated doc, ADR, PRD, ROADMAP, RUNBOOKS-INDEX, per-team CHARTER, every PR-author-visible status label.
- Does not apply to: external customer-facing marketing copy (which uses `preview / stable / GA` from this ADR; not the wave names).

---

## Consequences

### Positive

- Engineering-shaped wave names make gate criteria mechanical, not date-bound.
- `preview / stable / GA` aligns with industry expectation; auditors and customers parse the labels without translation.
- Forbidden M0..M3/minimum-shippable-tier vocab eliminates a recurring re-litigation point.
- Per-wave gate criteria + audit emission produces an auditable history of wave landings.

### Negative

- Initial sweep cost — every legacy doc that uses M0..M3/minimum-shippable-tier needs revision (ROADMAP §8 + GLOSSARY §11 retired-terms appendix already absorbs most of this).
- Wave parallelism diagram requires careful per-PR maintenance as waves land out of order.

### Operational

- On-call: `EVT-WAVE-GATE-PASSED` posted to council-architecture; per-wave evidence pack regenerable per ADR-0003.
- Runbooks: `runbooks/wave-gate-evaluation.md`, `runbooks/preview-to-stable-promotion.md`, `runbooks/stable-to-GA-promotion.md`.
- CI: `oya-governance-glossary` (forbidden vocab), `oya-governance-wave-status` (catalog status field matches surface guarantees).

---

## Alternatives considered

### Alternative A — Keep M0..M3/minimum-shippable-tier

- **Pros:** familiar to legacy contributors.
- **Cons:** date-bound; commercial-launch-bound; conflicts with optimal-path framing.
- **Rejected because:** PRD §3.1.

### Alternative B — Numbered waves only (W1..WN)

- **Pros:** simpler ordering.
- **Cons:** number does not communicate substrate-or-axis content; reviewers must look up "what is W3."
- **Rejected because:** descriptive names are self-documenting.

### Alternative C — Track per-microservice status without wave concept

- **Pros:** simpler.
- **Cons:** loses cross-microservice dependency tracking; W-Foundation gating is explicit.
- **Rejected because:** cohesion + dependency clarity.

---

## Open questions

1. **Q1.** Per-pack wave subdivision (W-Region-Fan-Out is a single wave covering multiple packs) — split per pack or keep aggregate? Default: aggregate; per-pack onboarding is a sub-batch. → ADR-0010.
2. **Q2.** "GA" criteria per-microservice — pinned to per-pack regulator-equivalent OR global SLO commit? Default: per-pack regulator-equivalent for the regions in scope. → COMPLIANCE-MATRIX.
3. **Q3.** Cross-wave status (e.g. an axis's data-plane is `stable` while its analytics-plane is `preview`) — per-plane status? Default: per-surface status per the catalog `plane:` field; an axis is "stable" when all its data-plane surfaces are stable. → ADR-0004.

---

## References

- `docs/PRD.md` §3.1 (optimal-path waves; vocabulary update retiring M0..M3/minimum-shippable-tier)
- `docs/ROADMAP.md` §1 (canonical wave sequence), §2 (per-wave gate criteria)
- `docs/GLOSSARY.md` §11 (deprecated terms)
- `docs/CONTRADICTION-LEDGER.md` (resolution batches sequenced by wave)
- ADR-0001 (cohesion), ADR-0011 (cross-microservice contract registry per wave gate), ADR-0012 (axis admission), ADR-0018 (forbidden-vocab fitness lane), ADR-0019 (per-wave evidence emission cadence)
