# ADR-0012: Axis admission protocol — what counts as an axis (vs sub-axis vs vertical vs pack), the current seven, the new-axis admission protocol, and the retire/consolidate protocol

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0010, ADR-0011, ADR-0019

---

## Context

ADR-0001 fixes the seven axes (SaaS, Workspace, Vertical, Foundry, Cloud, Search, Ads/Analytics) as the canonical set, with the cohesion thesis attached to that closed enumeration. But the corpus has historical drift: ADR-0001 enumerated 5–6 arms with no cloud/search/ads/agent-runtime axes; the legacy axis-admission ancestry plus ADR-0001 needed a new-axis horizon; the 2026-05-09 reframing added Workspace; later in the same day Foundry consolidated the prior engineering-platform axis. Without an explicit axis admission protocol, every reframing risks proliferation (every new product becomes a new axis) or contraction (cohesion loses surface area). LEDG-008 captures this as an open BLOCKER.

A second pressure: sub-axes are real (Foundry's robotics-control sub-axis, Cloud's DC-Ops sub-axis), verticals are real (14 verticals at fan-out), packs are real (per-locale per ADR-0010). Without a clean test that distinguishes axis vs sub-axis vs vertical vs pack, every council debate re-litigates classification.

---

## Decision

We adopt an **axis admission protocol** with explicit definitions, an enumerated current axis set, a multi-step new-axis admission process, a sub-axis test, and a retire/consolidate procedure.

### Definitions

- **Axis** = a top-level bounded context that satisfies all of:
  - Owns at least one cross-axis contract row in the registry (ADR-0011).
  - Has its own bounded-context kernel + own clean-architecture stack (ADR-0015 roles).
  - Owns at least one user-visible product surface OR is a substrate consumed by all others.
  - Has a council-ratified owning team in `teams/`.
  - Passes the *cohesion-invariant compliance* check (ADR-0001 forbidden-pattern set).

- **Sub-axis** = a scoped extension inside an axis that has its own crate cluster + per-substrate eval but does NOT own a cross-axis contract row independently. Examples: Foundry's `robotics-control`, Cloud's `dcops`.

- **Vertical** = a per-industry SaaS+plug-in cluster inside the Vertical axis. Examples: healthcare, fintech, industrial, logistics, public-sector. Verticals share the Vertical axis kernel + adopt regulatory packs (ADR-0010); each vertical does NOT own a cross-axis contract row.

- **Pack** = a per-locale plug-in (regional pack per ADR-0010) supplying seam impls for a market. Packs are NEVER axes — they plug into seams that axes own.

### The current seven axes

| # | Axis | Owning bounded context |
|---|---|---|
| 1 | SaaS multi-tenant platform | `crates/oya-platform-*`, `crates/oya-saas-*` |
| 2 | Workspace / Productivity Suite | `crates/oya-workspace-*` |
| 3 | Vertical industry cloud | `crates/oya-vertical-*` |
| 4 | Foundry (AI agent runtime + control plane + engineering platform) | `crates/oya-foundry-*` |
| 5 | Cloud provider | `crates/oya-cloud-*` |
| 6 | Search engine | `crates/oya-search-*` |
| 7 | Advertising + analytics | `crates/oya-ads-*`, `crates/oya-analytics-*` |

### New-axis admission protocol

A proposal to admit an 8th (or higher) axis must include:

1. **Council proposal document** — written by the proposing team; lives at `docs/decisions/ADR-NNNN-axis-admission-<name>.md`.
2. **Justification against the four-criterion definition** — the proposal explicitly answers: (a) which cross-axis contract row(s) does the axis own; (b) which kernel + stack; (c) which user-visible surface OR substrate role; (d) which owning team.
3. **Cross-axis contract enumeration** — the new axis's contracts with each of the existing axes, drafted to ADR-0011 registry shape.
4. **Bounded-context boundary statement** — what is in the axis vs what stays in adjacent axes.
5. **Owning team charter** under `teams/<axis-id>/CHARTER.md` per `RACI-OWNERSHIP.md`.
6. **Cohesion-invariant compliance evidence** — fitness-lane runs of `oya-foundry-fitness-cohesion` against the proposed axis's kernels.
7. **Founder ratification** — final sign-off via `council-architecture` chair + Founder.

### Sub-axis vs axis test

A surface is a *sub-axis* (not an axis) when:

- It does NOT own an independent cross-axis contract row in the registry (ADR-0011).
- Its contracts to other axes route through its parent axis's contracts.
- Its catalog records cite the parent-axis kernel, not a new substrate.

The default for any new surface is sub-axis. Promotion from sub-axis to axis follows the new-axis admission protocol.

### Vertical vs axis test

A surface is a *vertical* (not an axis) when:

- It plugs into the Vertical axis's kernel + adopters (`crates/oya-vertical-kernel-*` + `crates/oya-vertical-<industry>-*`).
- Its regulator bindings are per-tenant via the Vertical axis's `Tenant.regulatory_packs` (ADR-0002 + ADR-0010), not a cross-axis contract.
- Its surface composes Workspace + Foundry + Cloud + Search + (sometimes) Ads, all consumed via the existing axis contracts.

### Pack vs axis test

A surface is a *pack* (not an axis or sub-axis) when:

- It plugs into a seam published by ADR-0010.
- It supplies per-locale impls for regulatory, tax, IdP, payment, address, content safety, ad policy, industry data model, or vendor partners.
- It does not modify the kernel.

### Retire / consolidate protocol

An axis MAY be retired or consolidated into another axis when:

1. **Council proposal** with rationale (e.g. cohesion benefit, user-visible posture change).
2. **Per-contract migration plan** — every cross-axis contract row owned by the retiring axis migrates to the absorbing axis with versioning + deprecation per ADR-0019.
3. **Crate-tree migration plan** — flat-crates renames per ADR-0015.
4. **Brand surface migration plan** — per ADR-0017.
5. **Founder ratification.**

The 2026-05-09 Foundry consolidation (prior engineering-platform axis absorbed into Foundry) is the canonical example; LEDG-022 records the resolution.

### Boundary

- Applies to: every proposal to add, retire, merge, or split a top-level axis.
- Does not apply to: per-vertical or per-pack additions (those follow ADR-0010 + the Vertical axis's per-vertical onboarding); sub-axis additions inside an axis (default behavior of the parent axis team).

---

## Consequences

### Positive

- Closes LEDG-008 (axis admission contract) at the protocol level.
- Stops axis proliferation at the council door — every new axis has to defend itself against the four-criterion test.
- Verticals + packs + sub-axes have clear "this is not an axis" tests; classification debates end.
- Retire / consolidate path is explicit; the Foundry consolidation precedent is generalized.

### Negative

- Higher friction for organic growth (e.g. a future Robotics-as-axis would face the full protocol). This is intentional.
- Council bottleneck on axis-shaped decisions; mitigated by per-quarter council cadence and pre-routed proposal triage.

### Operational

- On-call: not applicable (architectural).
- Runbooks: `runbooks/axis-admission-proposal.md`, `runbooks/axis-retire-consolidate.md`, `runbooks/sub-axis-promotion.md`.
- CI: `oya-foundry-fitness-axis-admission` validates that every crate cluster claiming axis status has an admission ADR; sub-axes whose contracts try to register independently in ADR-0011 are rejected.
- Council cadence: axis-admission proposals reviewed quarterly; emergency review for substrate-affecting proposals.

---

## Alternatives considered

### Alternative A — Open admission (any team can propose any axis at any time)

- **Pros:** maximal flexibility.
- **Cons:** axis proliferation observed in legacy corpus (ADR-0001 / 0185 / 0231 drift).
- **Rejected because:** ADR-0001 cohesion needs a closed axis set.

### Alternative B — Static axis set (no admission protocol; only this ADR can change it)

- **Pros:** minimum drift.
- **Cons:** future markets / surfaces cannot enter; council loses tool to expand scope.
- **Rejected because:** Workspace was admitted on 2026-05-09; the protocol must support repeats.

### Alternative C — Per-PR axis-classification check, no protocol

- **Pros:** less ceremony.
- **Cons:** PRs cannot adjudicate axis-shape decisions; council is the right level.
- **Rejected because:** decision rights mismatch.

---

## Open questions

1. **Q1.** Defense / drone scope (LEDG-017) — separate axis (`Defense`) or a sub-axis of Vertical with a dedicated regional pack `oya-pack-defense`? Default: Vertical sub-axis + pack. → council.
2. **Q2.** Lifestyle / consumer scope (LEDG-017) — in-scope as a Vertical (Cellar / Dining as verticals) or anti-scope at the axis level? Default: anti-scope at PRD level; founder reconsiders. → PRD §3.3.
3. **Q3.** Future Robotics axis — ever? Default: NO; robotics is a Foundry sub-axis + per-vertical consumer. → ADR-0001 + Foundry.
4. **Q4.** Future Hardware axis — anti-scope per PRD §3.2; reconsider only if KR partner GPU co-investment lands. → PRD §8.
5. **Q5.** AI-Model-Substrate (long-horizon) — sub-axis of Foundry or sibling? Default: sub-axis of Foundry per DESIGN §3.0.1. → Foundry council.

---

## References

- `docs/PRD.md` §1 (axis count = 6 per Foundry consolidation; this ADR re-asserts 7 with Workspace per the user brief)
- `docs/DESIGN.md` §1 (axis enumeration + bounded contexts)
- `docs/CONTRADICTION-LEDGER.md` LEDG-008 (axis admission), LEDG-017 (lifestyle / defense scope), LEDG-022 (Foundry external/internal — consolidation precedent)
- ADR-0001 (cohesion thesis — closed axis set), ADR-0010 (regional pack architecture — packs ≠ axes), ADR-0011 (cross-axis contract registry — axis test), ADR-0015 (flat-crates target — bounded context kernel naming), ADR-0019 (doc-update protocol — axis-admission ADRs)
