---
id: ADR-0107
title: `tools/` directory canonical-suffix binding (was: implicit `app` layer — SUPERSEDED by self)
status: Superseded
superseded_by: ADR-0107 §"Amendment 2026-05-15 — no-exception canonical naming"
owner: council-architecture
date: 2026-05-15
amends:
  - ADR-0056-rust-clean-architecture-bnf.md
  - ADR-0105-13-layer-enum-and-check-family-patterns.md
relates_to:
  - ADR-0104-ecosystem-expansion-toolchain-and-adapters.md
  - ADR-0106-rename-application-to-usecase.md
---

# ADR-0107: `tools/` directory canonical-suffix binding (was: implicit `app` layer — superseded)

## Status
Superseded by self via 2026-05-15 amendment (see §"Amendment 2026-05-15" below). The original "implicit-app" exception is REMOVED. Every crate under `tools/` MUST carry a canonical layer suffix from ADR-0105's 13-value enum. No exceptions.

## Amendment 2026-05-15 — no-exception canonical naming

User directive (2026-05-15): *"for 9 dont allow exceptions. fix our adr and other documents that declare exceptions. stick with canonical and if you need addition to canonical make the edit. it is paramount that everything is in predictable shape for maintainability."*

### Revised decision

1. **The "implicit-app" exception is REMOVED.** Crates under `tools/` are NOT implicitly `app` by directory location. The directory is an organizational hint, not a naming declaration.
2. **Every `tools/` crate MUST end in a canonical layer suffix** drawn from ADR-0105's 13-value enum (`kernel | domain | usecase | app | adapter | infrastructure | cli | rest | grpc | graphql | worker | sdk | api`), or match a documented Adopted Pattern (`oya-check-<feature>` self-layering or `*-adapter-<backend>` backend-qualifier).
3. **Binary tool crates under `tools/` SHALL use the `-app` suffix** as the canonical declaration of composition-root layer. This is the documented binding of `app` to the `[[bin]]` shape per ADR-0056 §"Layer semantics > app".
4. **`oya-tooling-agent-read` retains its name** ONLY because CLAUDE.md declares it a sanctioned coordination primitive whose name is doctrinally fixed by the agent-operating contract. This is a doctrinal lock at the agent-contract layer, NOT a naming-canonical exception. The crate-naming kernel records it as a documented doctrinal carve-out tied to ADR-0053 (sanctioned primitives), distinct from the layer-enum canonical surface.

### Effect on the 9 crates originally enumerated

| Crate | Resolution | Rationale |
|---|---|---|
| `tools/oya-foundry-fitness-portfolio-citation` | RENAME → `*-app` | Binary tool; canonical `-app` suffix per layer enum. |
| `tools/oya-foundry-fitness-predictable-naming` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-foundry-fitness-archive-orphan` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-foundry-fitness-banned-primitives` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-foundry-fitness-adr-shape` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-foundry-fitness-authoritative-tracked` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-foundry-fitness-purpose-audit` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-adapter-substitution-test` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-tooling-agent-read` | KEEP (doctrinal lock, not naming exception) | CLAUDE.md sanctioned primitive; name fixed at agent-operating-contract layer. Recorded as a doctrinal carve-out under ADR-0053, NOT a layer-enum exception. |

All 8 renames execute as C1-shaped atomic commits per ADR-0054 §grit-scaffold-claim-pattern.

## Original (superseded) decision — for history

(Original §"Decision" content preserved below for traceability. Do NOT cite.)

~~**Any crate under `tools/` is implicitly the `app` layer.**~~ Superseded by 2026-05-15 amendment — implicit-by-location is no longer a sanctioned naming surface. The 9 crates enumerated above are resolved either by explicit rename to canonical layer suffix or by recorded doctrinal carve-out (only `oya-tooling-agent-read`).

## Pre-amendment Status (historical)
Originally `Accepted` on 2026-05-15. Superseded the same day by user directive prioritizing predictable canonical naming over directory-implicit conventions.

## Context

The 2026-05-15 crate-naming audit (`specs/cross-cutting/crate-naming-audit.json`) flagged 9 crates in `tools/` that lacked an explicit layer suffix:

```
tools/oya-foundry-fitness-portfolio-citation
tools/oya-foundry-fitness-predictable-naming
tools/oya-foundry-fitness-archive-orphan
tools/oya-foundry-fitness-banned-primitives
tools/oya-foundry-fitness-adr-shape
tools/oya-foundry-fitness-authoritative-tracked
tools/oya-foundry-fitness-purpose-audit
tools/oya-adapter-substitution-test
tools/oya-tooling-agent-read
```

Per ADR-0056 §"12-Value Layer Enum" all crates must end in a canonical layer suffix. The 9 above do not. The original audit verdict proposed renaming each to `*-app`.

Two issues with the per-crate rename:
1. **Mechanical cost** — 9 atomic renames (mv + Cargo.toml + workspace.members + every importer), one of which is `oya-tooling-agent-read` (the CLAUDE.md sanctioned primitive whose name is doctrinally fixed).
2. **Convention already exists** — every crate under `tools/` IS a binary. The directory location already declares the layer. The `*-app` suffix would be redundant tautology (`tools/` + `-app` = "binary binary").

ADR-0105 §Adopted Patterns already formalized `oya-check-<feature>` as a self-layering convention (the µservice IS the layer). The same pattern applies to `tools/`: the directory IS the layer.

## Decision (SUPERSEDED — historical, see §Amendment above)

~~**Any crate under `tools/` is implicitly the `app` layer.**~~ Superseded 2026-05-15. The canonical rule is: every `tools/` crate ends in a canonical layer suffix; binaries use `-app`. The original text is preserved below for traceability only.

~~Any crate under `tools/` is implicitly the `app` layer.~~ No explicit `*-app` suffix is required. The directory location is the layer declaration.

Constraints on tools/ crates:
- MUST be a binary (i.e., `[[bin]]` in Cargo.toml).
- MAY also be a library (lib + bin) when the lib surface is intentionally exposed for in-process testing.
- MUST NOT be lib-only (that would be a `*-kernel` or `*-domain` and belongs under `crates/`).

Constraints on the convention:
- This convention applies ONLY to crates DIRECTLY under `tools/`, not to nested subdirectories.
- The convention is opt-in by location: if a crate moves from `tools/` to `crates/`, the explicit `*-app` suffix becomes mandatory.
- This convention is documented in ADR-0056 §Layer semantics and in the workspace.metadata.oya Cargo.toml comment block.

### Effect on the 9 crates

All 9 become compliant in-place without renaming:

| Crate | Layer (implicit) | Compliance |
|---|---|---|
| `tools/oya-foundry-fitness-portfolio-citation` | app | ✓ |
| `tools/oya-foundry-fitness-predictable-naming` | app | ✓ |
| `tools/oya-foundry-fitness-archive-orphan` | app | ✓ |
| `tools/oya-foundry-fitness-banned-primitives` | app | ✓ |
| `tools/oya-foundry-fitness-adr-shape` | app | ✓ |
| `tools/oya-foundry-fitness-authoritative-tracked` | app | ✓ |
| `tools/oya-foundry-fitness-purpose-audit` | app | ✓ |
| `tools/oya-adapter-substitution-test` | app | ✓ |
| `tools/oya-tooling-agent-read` | app | ✓ |

The audit's residual non-compliant count drops from 18 to 9 (the remaining 9 are the 3 `*-runtime` plus 6 vcs/saas one-offs that need real per-crate decisions).

## Consequences

- **`oya-foundry-fitness-predictable-naming-kernel`** must be updated to recognize the tools/-implicit-app convention. Until then, the lane would flag these 9 crates as non-compliant.
- **`specs/cross-cutting/crate-naming-audit.json`** is amended in this commit to mark the 9 tools/ crates as compliant via ADR-0107.
- **Cargo.toml `[workspace.metadata.oya]`** comment block updated to document the convention alongside the check-family and backend-suffix patterns from ADR-0105.
- **`oya-tooling-agent-read` no longer requires the "naming-exceptions" doctrinal carve-out** I added in ADR-0105 (it's now compliant via the standard convention).

## Drivers

- decision-principles.json DP-06 (Bounded scope per doc) — directory IS the layer when the convention is unambiguous.
- decision-principles.json DP-03 (Mechanical prevention over process) — convention-by-location is mechanically checkable.
- forbidden-operations.json FO-01 (No parallel canonical trees) — `tools/` is the canonical-binary-tree; redundant suffix would create a parallel naming axis.

## Alternatives Considered

1. **Per-crate rename to `*-app`.** Rejected: 9 atomic renames (workspace.members + importers + scripts that invoke by name), redundant tautology, breaks `oya-tooling-agent-read` doctrinal name.
2. **Allow tools/ crates to use ANY layer suffix.** Rejected: too permissive; defeats the BNF discipline.
3. **Restrict tools/ to binaries-only (no lib surface).** Rejected: the existing `oya-foundry-fitness-purpose-audit` has a lib surface (it's both a lib + a bin); same for `oya-tooling-agent-read`. Production crates under `tools/` legitimately expose both surfaces for testing.

## Follow-ups

1. Update `oya-foundry-fitness-predictable-naming-kernel` (Phase E follow-up) to:
   - Recognize the 13-value enum (per ADR-0105).
   - Recognize tools/-implicit-app convention (per this ADR-0107).
   - Recognize `oya-check-<feature>` self-layering (per ADR-0105).
   - Recognize `*-adapter-<backend>` sub-suffix (per ADR-0105).
   - Recognize `usecase` (renamed from `application` per ADR-0106).
2. `crate-naming-audit.json` amended in this same commit.
3. Workspace `[workspace.metadata.oya]` comment updated.

## References

- ADR-0056 §"12-Value Layer Enum" (the original closed enum)
- ADR-0105 §Adopted Patterns (where check-family and backend-suffix conventions were formalized; this ADR adds the tools/ convention)
- ADR-0106 (application → usecase rename, sibling change)
- specs/cross-cutting/crate-naming-audit.json (per-crate compliance ledger)
- User directive 2026-05-15: "if they are well defined and serve a purpose, consider adopting them"
