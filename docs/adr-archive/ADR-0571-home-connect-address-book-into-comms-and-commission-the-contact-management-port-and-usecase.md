---
id: ADR-0571
title: "Home the connect address-book domain into the comms capability and commission the contact-management port + usecase (wave1 strangler MOVE + cloud-agnostic core slice)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-22
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: [ADR-701]
amends: []
depends_on: [ADR-0510, ADR-0532, ADR-0533, ADR-0536, ADR-0537, ADR-0538, ADR-0555, ADR-0562, ADR-0563]
related: [ADR-0029, ADR-0083, ADR-0105, ADR-0131, ADR-0139, ADR-0245, ADR-0280, ADR-0515, ADR-0567, ADR-0569]
related_specs:
  - /specs/capability-registry.json
  - /specs/reachability-registry.json
milestone: W2
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Comms address-book into capability tree

# ADR-0571: Home the connect address-book into `comms` + commission the contact-management port + usecase

## Status

**Proposed - 2026-06-22 (authored for founder sign-off; door: two-way — a single-crate strangler
relocation into an already-established capability plus two additive crates behind an already-shaped
clean-architecture seam; reversible by the codemod inverse plan + deleting the two new crates,
without unwinding any SSOT; the producer remains the sole face generator).**

## Context

The capability-first reorg (ADR-0562/0563) homes each capability's crates under
`<capability>/{core,ports,adapters,facade}` via the deterministic reorg codemod. The `comms`
communications plane was established by the twelfth strangler move (ADR-0562 §10.16, #757), which
homed `oya/{mail,messenger,meet,contact-center}` into `comms/{core,ports,adapters,facade}` and
seeded the `comms/*/*` workspace-members glob, the `comms` closed-registry capability slug +
allowed-top-level-dir, and the breadth-unlimited `comms/OWNERS` reachability anchor (ADR-0555).

`oya/connect` carried exactly one crate — `oya-address-book-domain`, a 705-line pure domain kernel
for the W-Workspace Org Address Book adjunct surface (ADR-0029): per-tenant + per-user contact-card
validation plus consent-gated cross-tenant directory exposure, with NO CardDAV / identity-lookup /
search-indexing adapters. The address book is a contacts/directory concern of the multi-channel
communications plane (`comms` charter: email/mail, messenger, meet, notifications, contact-center),
so its home is `comms`, not a standalone app vertical. `oya/connect` itself has no governance md
tree (no OWNERS, no docs), but the moved domain crate DOES carry one gate-bound catalog record
(`registry/catalog/oya-address-book-domain.yaml` — catalog-liveness + slo-coverage key on the file
stem == crate id), which co-moves and is re-keyed to the live crate id. Its sole dependency is
`libs/oya-data-boundary-kernel` (downward, safe), and no other crate depends on it — the cleanest
single-crate lift in the surface.

Two seams stay open at the domain rung: there is no typed contact-management API surface, and no
application usecase wiring the domain through a port. The persistence/identity/cloud adapters are
deliberately DEFERRED (clean architecture; ports designed for the owned stack), so this slice
advances the cloud-agnostic core only.

## Decision

### D1 — MOVE: home the address-book domain into `comms/core`

Relocate `oya/connect/crates/oya-address-book-domain` →
`comms/core/connect-address-book-domain` via the reorg codemod (history-preserving `git mv` +
package/lib-name de-brand per ADR-0532/0533: cargo `connect-address-book-domain` == path-tail, the
`connect-` discriminator retained to name the address-book's product origin and keep the leaf unique
within `comms`). The move requires ZERO members-array edit (the `comms/*/*` glob already covers
`comms/core`), ZERO capability-registry edit (`comms` is a registered capability; POST-move the path
itself is the namespace per the membership lint), ZERO reachability-registry edit (the
breadth-unlimited `comms/OWNERS` seed covers the whole `comms` subtree), ZERO OWNERS edit. The
emptied `oya/connect` dir tracks no files (git tracks no empty dirs), so no orphan or inert path
remains; the `oya/connect` app_products membership-lint entry becomes inert-but-harmless (a PRE-move
path with no crates, the established pattern for every prior absorbed dir).

Per the single-plan materialization invariant (codemod #65), this decision's PR commits exactly one
move-plan at `specs/reorg/connect-move-plan.json` and removes the now-satisfied
`specs/reorg/intelligence-move-plan.json` (move-22a already applied; its crates no longer exist at
the old path) — the same single-plan swap every prior move PR performed (e.g. move-12 removed
`console-move-plan.json` when it added `comms-move-plan.json`).

### D2 — BUILD: commission `comms/ports/connect-address-book-api` (contact-management port)

A cloud-agnostic inbound port crate that owns the typed contact-management request/receipt DTOs, a
FAIL-CLOSED `AuthorizedContactContext` (refuses to proceed without a verified principal, a
`tenant:`-scoped subject, a PDP policy-decision reference, an idempotency key, and an
audit-correlation id — `validate()` returns a hard `Missing*` error, never a silent allow), and the
OUTBOUND `ContactStore` / `DirectoryGrantStore` port traits the deferred cloud/persistence/identity
adapters implement. The crate has no persistence/identity/transport dependency — the trait shapes
model the owned-stack destination (they speak tenant/contact/consent, not a concrete store, so they
do not change at cutover). Fail-closed authz on any new surface is mandatory per the founder
new-HTTP-surfaces doctrine; this port crate carries the authorization gate the usecase enforces.

### D3 — BUILD: commission `comms/core/connect-address-book-usecase` (contact-management usecase)

The application layer wiring the pure domain through the port. It depends only on
`connect-address-book-domain` + `connect-address-book-api` (no persistence/identity/transport).
Every entry point is fail-closed: it calls `AuthorizedContactContext::validate()` FIRST and enforces
principal/tenant binding (rejecting cross-tenant and cross-principal writes) before any domain
construction, then returns a traceable receipt carrying the audit/idempotency/PDP references.
Persistence, identity, and cloud adapters are DEFERRED behind the port traits.

### D4 — Faces + lock

The Cargo.lock and the cloud-ci generated faces (accounting-registry, scm-facts, gate-baseline,
move-manifest) are regenerated by the canonical producers (`buck2-build-green != CI-green`), never
hand-edited.

### D5 — Ownership + justification manifest (ADR-0555 D2)

Owner: the breadth-unlimited `comms/OWNERS` = `axis-cloud-platform` (ADR-0562 §10.16) resolves the
owner for every file under the `comms/` subtree, including the moved + new crates. The crate `.rs`
sources, `Cargo.toml`, and `BUCK` files are reachable via the `comms/*/*` member glob (ADR-0538) +
the breadth-unlimited `comms/OWNERS` reachability anchor (ADR-0555). The committed move-plan
`specs/reorg/connect-move-plan.json` is reachable via the existing `specs/reorg/` ADR-0563 prefix.
No NEW catalog record is minted for the two commissioned crates (the gate-tool default; neither
catalog-liveness nor slo-coverage requires every live crate to carry a record). The MOVED domain
crate's catalog record co-moves as a non-crate artifact of the move-plan: it is re-keyed
`registry/catalog/oya-address-book-domain.yaml` → `registry/catalog/connect-address-book-domain.yaml`
to track the de-branded live crate id (both gates key on the file stem == crate id), and the
rename-aware move-manifest (ADR-0563) relabels its path-keyed baseline disposition so it is NOT
net-new. The MOVED domain crate's source files are likewise co-moved and inherit their baseline
disposition, so they are not net-new. The net-new files commissioned by this decision:

`comms/ports/connect-address-book-api/BUCK`,
`comms/ports/connect-address-book-api/Cargo.toml`,
`comms/ports/connect-address-book-api/src/lib.rs`,
`comms/core/connect-address-book-usecase/BUCK`,
`comms/core/connect-address-book-usecase/Cargo.toml`,
`comms/core/connect-address-book-usecase/src/lib.rs`,
`specs/reorg/connect-move-plan.json`.

## Precedent

- **ADR-0562 §10.16 / #757 (comms move-12)**: the established `comms` capability home, its members
  glob, closed-registry slug, and breadth-unlimited OWNERS reachability anchor that this lift reuses
  with zero new infrastructure.
- **ADR-0569 / ADR-0567 commissioning pattern**: a net-new crate is born-accounted via a D5
  ownership + justification manifest that lists every commissioned file path.
- **ADR-0532/0533 de-brand**: cargo name == de-branded path-tail; drop the `oya-` vendor prefix.
- **Clean architecture / ports designed for the owned stack**: domain + usecase pure, ports defined,
  cloud/persistence/identity adapters deferred behind the port traits.

## Rejected alternatives

- **Home the address-book under `app/connect/` as an app-product vertical.** Rejected: the
  capability-registry tentatively maps `oya/connect` to the `app_products` axis, but the
  address-book is a single contacts/directory DOMAIN, not a multi-capability tenant composition. A
  domain kernel belongs in a capability `core/`, and contacts/directory is squarely the `comms`
  communications plane (the registry's `comms` boundary_note already flags directory/contacts as a
  comms-plane concern). `app/` membership is reserved for composition roots that wire 2+
  capabilities, which this crate is not.
- **Define the persistence port traits in the usecase (core) instead of the ports crate.** Rejected:
  the port is the inbound capability boundary; keeping the outbound store traits in `ports/` keeps
  the dependency direction clean (usecase → port-traits ← adapters) and matches the established
  `comms/ports/*-api` face placement.
- **Sign-off door admission for the net-new files.** Rejected: the one-way sign-off door is the
  founder-authority backstop for born-unjustified residue, not the default path. A commission ADR
  that lists the file paths is the construction-over-reaction discipline (the ADR-0569 precedent),
  so the files are born-justified, not door-exempted.
