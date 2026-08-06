---
id: ADR-0707
title: "Live trust, safety, and resilience substrate doctrines"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-0300, ADR-0301, ADR-0302, ADR-0306]
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: []
milestone: W0
deliverables:
  - id: ADR-0707-D1
    description: "Live apex source-of-truth for topic trust_safety: Live trust, safety, and resilience substrate doctrines."
    exit_criteria: "docs/decisions/ADR-0707-trust-safety-live-apex.md is Accepted with planning_impact true; member ADRs listed in supersedes are archived under docs/adr-archive/."
    verified_by: "oya-ci-required"
---
# ADR-0707: Live trust, safety, and resilience substrate doctrines

## Status

**Accepted** — live consolidated source-of-truth entry for topic `trust_safety` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **4** Accepted ADRs in the `trust_safety` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

1. **This ADR is the live reading entry** for topic `trust_safety` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-300** (ADR-0300-whistleblower-press-freedom-anonymity): ### §B. Decision summary **Decision 1: Per-tenant anonymous-submission surface via sealed- sender envelope.** Every tenant with the `publisher-source- protection` pack overlay OR a per-pack reporter-privilege overlay gets a per-tenant SecureDrop-class submission surface. The surface: (i) accepts only Tor onion-service v3 ingress OR clearnet ingress
- **ADR-301** (ADR-0301-survivor-safety-domestic-abuse-mode): ### §B. Decision summary **Decision 1: Silent shelter mode.** Per-account `SHELTER_MODE_ ACTIVE` lifecycle_state per ADR-0244 §D-3. Activation is silent (no notification to other shared-credential principals; no per- tenant-admin alert). The shelter-mode state is the foundational primitive on which all other decisions compose. **Decision 2: Hide-fr
- **ADR-302** (ADR-0302-deceased-user-inheritance-doctrine): ### §B. Decision summary **Decision 1: Per-account Legacy-Contact pre-designation (Apple- class).** Every account can pre-designate ≤5 Legacy-Contacts in the account preferences. Each Legacy-Contact: - Receives an access-key fragment at designation time (Shamir- shared per ADR-0247 break-glass + per-Legacy-Contact threshold of 2-of-N). - After the 
- **ADR-306** (ADR-0306-disaster-mode-cell-resilience): ### §B. Six core primitives at three layers The disaster-mode + cell-resilience baseline is **six core primitives** (surge handling; offline-first sync; progressive enhancement; DR-pair failover; per-pack disaster overlay; emergency-services non-throttle invariant) wired at **three layers** (Tier-0 shared crate, per-µservice gate, Cedar policy frag

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-302 residual

**ADR-0302-deceased-user-inheritance-doctrine** — ### §B. Decision summary **Decision 1: Per-account Legacy-Contact pre-designation (Apple- class).** Every account can pre-designate ≤5 Legacy-Contacts in the account preferences. Each Legacy-Contact: - Receives an access-key fragment at designation time (Shamir- shared per ADR-0247 break-glass + per-Legacy-Contact threshold of 2-of-N). - After the user's death, submits the death certificate (per-

### ADR-301 residual

**ADR-0301-survivor-safety-domestic-abuse-mode** — ### §B. Decision summary **Decision 1: Silent shelter mode.** Per-account `SHELTER_MODE_ ACTIVE` lifecycle_state per ADR-0244 §D-3. Activation is silent (no notification to other shared-credential principals; no per- tenant-admin alert). The shelter-mode state is the foundational primitive on which all other decisions compose. **Decision 2: Hide-from-shared-device option.** Per-account notificatio

### ADR-306 residual

**ADR-0306-disaster-mode-cell-resilience** — ### §B. Six core primitives at three layers The disaster-mode + cell-resilience baseline is **six core primitives** (surge handling; offline-first sync; progressive enhancement; DR-pair failover; per-pack disaster overlay; emergency-services non-throttle invariant) wired at **three layers** (Tier-0 shared crate, per-µservice gate, Cedar policy fragment). The 6×3 matrix produces eighteen cells; eac

### ADR-300 residual

**ADR-0300-whistleblower-press-freedom-anonymity** — ### §B. Decision summary **Decision 1: Per-tenant anonymous-submission surface via sealed- sender envelope.** Every tenant with the `publisher-source- protection` pack overlay OR a per-pack reporter-privilege overlay gets a per-tenant SecureDrop-class submission surface. The surface: (i) accepts only Tor onion-service v3 ingress OR clearnet ingress with metadata-minimization + ECH-enabled TLS, (ii
