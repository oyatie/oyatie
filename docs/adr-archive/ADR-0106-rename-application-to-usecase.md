---
id: ADR-0106
title: Rename `application` layer to `usecase` (amends ADR-0105)
status: Superseded
superseded_by: [ADR-703]
planning_impact: true
doc_status: published
owner: council-architecture
date: 2026-05-15
amends:
  - ADR-0056-rust-clean-architecture-bnf.md
  - ADR-0105-13-layer-enum-and-check-family-patterns.md
relates_to:
  - ADR-0061-application-b2b-unified-shell.md
---

# ADR-0106: Rename `application` layer to `usecase` (amends ADR-0105)

## Status
Accepted

> **F-0029 RECONCILIATION (ratified 2026-06-07, door:one-way).** This ADR is the terminal link of the
> layer-enum amendment chain (ADR-0056 base 12 → ADR-0105 +`api` ⇒ 13 → ADR-0106 `application` →
> `usecase`). The enum it states below is the **canonical SSOT**: 13 product values (`kernel`,
> `domain`, `usecase`, `app`, `adapter`, `infrastructure`, `cli`, `rest`, `grpc`, `graphql`, `worker`,
> `sdk`, `api`) + the governance-only `check` family; `application` retired, `runtime` non-canonical
> (→ `app`). ADR-0056 and ADR-0105 carry the matching banner; this ratification closes this ADR's
> Follow-ups #1–#2 (the in-place ADR text is now consistent). The implied BNF crate renames
> (`*-application` remainder, `*-runtime` → `*-app`, the `oya-cloud-ci-*` gate-prefix family) are
> executed as the follow-on BNF-rename work, not reopened here.

## Context

ADR-0056 §"12-Value Layer Enum" defined two adjacent layers:

| Layer | Role | Cargo shape |
|---|---|---|
| `application` | Port-only use-case orchestration; depends on `kernel` + `domain`; no concrete adapters | `[lib]` |
| `app` | Composition root; instantiates concrete adapters into a deployable | `[[bin]]` |

Both names mean "an application" in plain English. The semantic gap is real (lib of use-cases vs deployable binary) but the naming carries no signal — readers conflate them.

The 2026-05-15 mass-rename attempt that consolidated `app` → `application` produced `oya-application-application` (µservice "application" + layer "application"), which crystallized the confusion. Reverted.

User question (2026-05-15): *"is there a better way for distinction?"*

## Decision

Rename the `application` layer to **`usecase`**. The `app` layer is unchanged. The canonical enum size stays at 13 (per ADR-0105); only the spelling changes.

### Why `usecase`

- **Clean Architecture canonical name.** Uncle Bob's "Clean Architecture" book names the port-only orchestration ring "Use Cases." The new name borrows from a 30-year-old, well-known model.
- **Visually + semantically distinct from `app`.** No reader confuses "usecase" with "deployable binary." The lib/bin distinction reads at a glance.
- **Smaller blast.** 6 crates rename (`*-application` → `*-usecase`) vs 10 (`*-app` → some other name). Cheaper to ship.
- **Resolves the `oya-application-app` µservice collision naturally.** That crate stays `oya-application-app` (the µservice is "application", the layer is "app"). Now the layer named "application" doesn't exist, so the µservice name standing alone is unambiguous.

### Updated 13-value canonical layer enum

| Group | Values |
|---|---|
| Inner / pure (4) | `kernel`, `domain`, **`usecase`** (was `application`), `app` |
| Outer / external (2) | `adapter`, `infrastructure` |
| Presentation / entry-point (7) | `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, `api` |

### Layer semantics (only `usecase` changes; quoted from ADR-0056 with rename)

- **`usecase`** — Use cases / application services orchestrating `domain` via port-trait bounds. No concrete adapters. (Was `application`.)
- All other layer semantics unchanged.

### Concrete rename

Six workspace crates renamed:

| Old | New |
|---|---|
| `oya-dsr-application` | `oya-dsr-usecase` |
| `oya-identity-application` | `oya-identity-usecase` |
| `oya-intelligence-eval-application` | `oya-intelligence-eval-usecase` |
| `oya-ops-workspace-shell-application` | `oya-ops-workspace-shell-usecase` |
| `oya-ops-docs-portal-application` | `oya-ops-docs-portal-usecase` |
| `oya-audit-chain-application` | `oya-audit-chain-usecase` |

Plus 7 importer files updated (Cargo.toml deps + Rust `use` statements). Cargo.toml `[workspace.members]` updated. Cargo.toml `[workspace.metadata.oya]` enum-comment updated.

## Consequences

- **`*-application` is no longer a canonical-suffix pattern.** `oya-governance-predictable-naming-kernel` enforcement must be updated to accept `*-usecase` and reject `*-application`. A grace period of 1 wave is reasonable; document the cite.
- **5 non-workspace-member `*-application` crates exist on disk** (`oya-cloud-billing-application`, `oya-cloud-billing-tax-application`, `oya-cloud-cell-application`, `oya-eventing-application`, `oya-metering-application`). These are audit finding #6's "16 crates exist on disk but aren't in `[workspace.members]`" — invisible to `cargo check --workspace`. They are NOT renamed in this commit because they're not part of the active workspace. Track in the successor-IP sweep that addresses #6 directly (decision: add to workspace or delete).
- **Decision-principles.json + forbidden-operations.json + ADR-0061 + a few other doctrinal docs** reference "application" as a layer. Sweep separately (Phase E4 cite sweep).
- **Documentation rewrite**: `docs/standards/clean-architecture.md` + `ADR-0056` §Layer semantics + ADR-0105 should be updated to use `usecase`. This commit edits Cargo.toml comments inline; the canonical doc updates are tracked as successor-IP so the rename ships atomically.

## Drivers

- User directive (2026-05-15): "application and app is the same thing? ... is there a better way for distinction?"
- decision-principles.json DP-04 (Dual audience for instructions) — layer names should be readable to human + agent.
- decision-principles.json DP-06 (Bounded scope per doc) — each layer has a single clear role; the name should carry it.

## Alternatives Considered

1. **Keep `application` + `app` as-is.** Rejected: user explicitly asked for a clearer distinction.
2. **Collapse both into a single `application` layer.** Attempted, reverted. Produced `oya-application-application` µservice-layer collision; loses real lib-vs-bin role.
3. **Rename `app` → `bin`** (match Cargo's `[[bin]]`). Rejected: "bin" reads too generic. 10 crates affected (larger blast). The lib-vs-bin distinction is already in Cargo manifest shape; the LAYER name should carry the architectural role (`app` carries "composition root").
4. **Rename µservice "application" → "shell" (per ADR-0061)** instead of touching the layer enum. Considered. Resolves only the one ugly case but doesn't fix the broader `app`/`application` confusion across the workspace.
5. **`application` → `service`.** Rejected: "service" is overloaded (cloud-services, micro-services, web-services).

## Follow-ups

1. Update `docs/standards/clean-architecture.md` to use `usecase`.
2. Update `ADR-0056` §"12-Value Layer Enum (closed)" + §Layer semantics text to use `usecase`. (ADR-0105 added `api`; ADR-0106 renames `application` to `usecase`.)
3. Update `specs/decision-principles.json` + `forbidden-operations.json` + `decision-rights.json` + `governance-amendment.json` if they reference `application` as a layer.
4. Update `oya-governance-predictable-naming-kernel` to recognize `usecase` + reject `application` post-grace-period.
5. Decide what to do with the 5 disk-but-not-workspace `*-application` crates (audit #6 successor-IP).
6. Update the cargo build/test scripts if any reference `application` by name.

## References

- ADR-0056-rust-clean-architecture-bnf.md (the 12-value enum being amended)
- ADR-0105-13-layer-enum-and-check-family-patterns.md (added `api`)
- ADR-0061-application-b2b-unified-shell.md (µservice "application", which keeps its name)
- specs/crate-naming-audit.json (will be amended)
- User directive thread 2026-05-15
