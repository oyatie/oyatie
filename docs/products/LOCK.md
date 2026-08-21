---
doc_class: ProductLock
product: oyatie
repo: oyatie/oyatie
status: published
date: 2026-08-21
owner: Product
doc_status: published
---

# Oyatie product lock

Live product authority for `oyatie/oyatie`. Quality cannot sign off without this bar.
`oya-ci-required` green is merge admission, not product-complete.

## Problem

Agents are the primary producers and have no honest done-when. Merge admission is
real. Product-complete currently accepts planned-maturity theater (axis PRD AC
rows that only hit `planned_maturity.rs`) or stays blank. The May 7-axis
first-production list (mail, ads, public search, datacenters) is a later Discover
bet, not this lock.

Cell and console journeys (#2076, #2086, #2085) are operator work and stay
**NotMeasured** until they have a fail-closed test. Test-count deltas on
capability lanes are not product progress.

## Locked product

Oyatie on this repo is the **Agentic Delivery Fabric**: owned SCM + CI + CD on
one substrate. Agents produce the tree. Sub-standard output cannot enter `dev`.

Cloud, SaaS, Search, Ads, and Workspace stay target / non-claim until this bar
is green.

## Done-when

All four must pass. **NotMeasured** and **Errored** are honest. "Shipped" is
not a pass.

| AC-ID | Given | When | Then | Evidence |
|---|---|---|---|---|
| P-LOCK-01 | More than one product north star is published | A reviewer reads published product docs | Only this lock is the live product claim | `docs/PRD.md` and `docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md` are Discover / draft / non-claim |
| P-LOCK-02 | A SHA is on `dev` with green `oya-ci-required` | Product-complete is asked | Status is NotMeasured unless a packet cites an AC-ID whose test is a runtime path, not `planned_maturity.rs` | Packet under `evidence/` or explicit NotMeasured |
| P-LOCK-03 | A change violates a required gate | Merge is attempted | It stays unmerged | Check URL on the PR, not prose |
| P-LOCK-04 | An agent authors a well-formed change | Gates pass | It can enter `dev` through the owned fabric | Recorded agent-authored merged PR + `oya-ci-required` URL |

## Non-claims

- This lock does not complete the fabric. P-LOCK-04 is NotMeasured until the first recorded agent-authored merge after this lock lands.
- Axis PRD AC rows that only hit `planned_maturity.rs` are not product acceptance.
- `docs/SPEC.md` "stable" labels are not product-complete.
- SLO catalog rows without a serving SLI are NotMeasured.

## Competing docs (Discover only)

- [`../PRD.md`](../PRD.md) — 7-axis ecosystem thesis
- [`../PRD-OYATIE-FROM-SCRATCH-CANONICAL.md`](../PRD-OYATIE-FROM-SCRATCH-CANONICAL.md)
- Per-product PRDs in this directory — planning / target / non-claim
