---
id: ADR-0362
status: Superseded
planning_impact: true
deciders: council-architecture, founder
date: 2026-05-25
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
amends: [ADR-0132]
related: [ADR-0131, ADR-0132, ADR-0135, ADR-0237, ADR-0238, ADR-0139]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/tenant-rbac-packaging.json, /specs/microservices/tenant-rbac.json]
session_context:
  authored: 2026-05-25
  basis: "Founder directives 2026-05-25 — 'flattened microservice structure has its advantages, and so does grouping them … we can ignore family. ignore all grouping. we can revive it later if there is a need.' Grouping is retired as an architecture artifact now; revivable only via a future ADR for a concrete need a catalog tag + platform µservice cannot cover."
purpose: Retire ALL product grouping (suite / family / bundle / vertical) as an architecture artifact. ADR-0132 set a forward-policy and grandfathered existing suites; this ADR removes that grandfather and the brand-layer carve-out, and makes the no-grouping gate real (ADR-0132 specified it but it was never implemented).
---

# ADR-0362: Full grouping retirement (flat-only catalog)

## Status

Accepted — 2026-05-25. Amends ADR-0132.

## Context

ADR-0132 established a no-grouping **forward-policy** but (a) explicitly **grandfathered** existing grouping wrappers ("Tenant/RBAC Packaging … Tenant RBAC … Foundry … remain as authored until their owning teams refactor them"), (b) explicitly **rejected** retiring grouping as a brand-layer concept, and (c) **specified** a `no-grouping` BLOCKER lane that was **never implemented** (no check crate; not a registered lane — an aspirational-enforcement gap).

Founder directive 2026-05-25 supersedes (a) and (b): *"ignore all grouping. we can revive it later if there is a need."* The genuine value of flat microservices (independent deploy / scale / SLO / blast-radius) is kept; the cost of grouping **artifacts** — a wrapper spec that *binds* shared policy/audit/deploy across children and re-grows a monolith by gravity — is removed. A grouping is at most a presentation/catalog concern, never an architecture artifact, and even that is deferred until a concrete need arises.

## Decision

1. **Flat single-concern µservices are the only architecture unit.** No grouping artifact of any kind — `suite`, `family`, `bundle`, `platform`, `vertical` — may exist as a spec, folder, or binding. This extends ADR-0132's prohibitions to *existing* artifacts, not just new ones.

2. **The grandfather clause is removed.** Existing grouping wrappers (`../tenant-rbac-packaging.json`, `tenant-rbac.json`, and any Foundry/Workspace/Cloud grouping wrappers) are no longer exempted-as-active. They are **demoted to `deprecated`** with a `retirement_ref`, and are tolerated by the gate **only while** they carry that deprecated status — i.e. they are tombstones on a tracked retirement path, not live architecture.

3. **No paper µservices.** Retiring the wrappers does NOT mean manufacturing replacement flat PRDs now (that would trip honest-claims). The real decomposition follows the existing owning ADRs on their SLO-gated cadence: → ADR-0238 (8 flat µservices) via the ADR-0237 strangler; Enterprise → a future `tenant-rbac-governance-council` dissolution ADR (not yet authored).

4. **Revival is ADR-gated.** Grouping may be reintroduced only via a future ADR, and only for a concrete need that a flat **catalog tag** (`product_family: "connect"`) plus a flat **platform µservice** cannot cover (e.g. a shared billing SKU). It may never become a code/deploy/SLO unit or a binding spec.

## Enforcement

The previously-aspirational `no-grouping` lane is **implemented for real** as the `no-grouping` gate (`oya-check-no-grouping`, registered in `oya gate run-all`). It fails on:

- any `specs/microservices/*-suite.json` / `*-family.json` / `*-bundle.json` that is NOT one of the two known retiring wrappers, OR is a retiring wrapper lacking `_meta.status: "Deprecated"` + a `retirement_ref`;
- any new multi-concern `microservices/<bundle>/` folder.

This closes the ADR-0132 aspirational gap (a claimed BLOCKER lane that did not exist).

## Rejected alternatives

- **Full dissolution now** (author all 11 flat PRDs this pass) — rejected: would duplicate the in-flight ADR-0238/0237 strangler, usurp `tenant-rbac-governance-council`'s authority over the Enterprise split, and manufacture paper µservices (honest-claims violation).
- **Keep grouping as a `*-family.json` artifact** — rejected per founder directive; a family artifact carries the same monolith-by-gravity risk as a suite. Grouping survives only as a future catalog tag, ADR-gated.
- **Leave ADR-0132's grandfather in place** — rejected; it left two live grouping wrappers binding shared concerns indefinitely.

## Clean Architecture Impact

Structural/policy only; layer assignments and dependency direction unchanged. ADR-0131 remains the flat-layout authority; this ADR strengthens its suite-prevention sibling from forward-only to full retirement.

## Verification

- `cargo run -p oya-dev-cli -- gate validate no-grouping` — exit 0 (the two wrappers are Deprecated + carry retirement_ref; no other grouping artifacts exist).
- `cargo run -p oya-dev-cli -- gate run-all` — green with the new lane registered.

## References

- ADR-0132: No-grouping forward-policy (amended by this ADR — grandfather + brand carve-out removed; aspirational gate now real).
- ADR-0131: Per-microservice flat layout.
- ADR-0238 / ADR-0237: dissolution + strangler migration (owns the real decomposition).
- ADR-0139: Agentic SLO-gated promotion.
- Founder directive 2026-05-25 (this session).
