---
id: ADR-0107
title: `tools/` directory canonical-suffix binding (was: implicit `app` layer — SUPERSEDED by ADR-0105)
status: Superseded
doc_status: superseded
superseded_by: ADR-0105-13-layer-enum-and-check-family-patterns.md
owner: council-architecture
date: 2026-05-15
amends:
  - ADR-0056-rust-clean-architecture-bnf.md
  - ADR-0105-13-layer-enum-and-check-family-patterns.md
relates_to:
  - ADR-0104-ecosystem-expansion-toolchain-and-adapters.md
  - ADR-0106-rename-application-to-usecase.md
sunset_at: 2026-05-15
deprecation_at: 2026-05-15
removal_at: 2026-08-15
sunset_topic: tools-implicit-app-exception-superseded
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0107: `tools/` directory canonical-suffix binding (was: implicit `app` layer — superseded)

## Status
**Superseded by ADR-0105** (2026-05-15 lifecycle transition; see §"Amendment 2026-05-15 — Superseded" below). The original "implicit-app" exception was REMOVED on 2026-05-15 by self-amendment; the canonical `tools/` suffix-binding rule that replaced it has since been fully absorbed by ADR-0105 §"Amendment 2026-05-15 — `tools/` canonical-suffix binding (paired with ADR-0107 supersede)". ADR-0107 retains no unique decision content and exists as a historical record only.

Forensic-retention window: `sunset_at = 2026-05-15`, `removal_at = 2026-08-15` (3-month retention per ADR-0108 default-aware schema). Cite ADR-0105 for all new work.

## Amendment 2026-05-15 — no-exception canonical naming

User directive (2026-05-15): *"for 9 dont allow exceptions. fix our adr and other documents that declare exceptions. stick with canonical and if you need addition to canonical make the edit. it is paramount that everything is in predictable shape for maintainability."*

### Revised decision

1. **The "implicit-app" exception is REMOVED.** Crates under `tools/` are NOT implicitly `app` by directory location. The directory is an organizational hint, not a naming declaration.
2. **Every `tools/` crate MUST end in a canonical layer suffix** drawn from ADR-0105's 13-value enum (`kernel | domain | usecase | app | adapter | infrastructure | cli | rest | grpc | graphql | worker | sdk | api`), or match a documented Adopted Pattern (`oya-check-<feature>` self-layering or `*-adapter-<backend>` backend-qualifier).
3. **Binary tool crates under `tools/` SHALL use the `-app` suffix** as the canonical declaration of composition-root layer. This is the documented binding of `app` to the `[[bin]]` shape per ADR-0056 §"Layer semantics > app".
4. **`oya-tooling-agent-read` retains its name** ONLY because CLAUDE.md declares it a sanctioned coordination primitive whose name is doctrinally fixed by the agent-operating contract. This is a doctrinal lock at the agent-contract layer, NOT a naming-canonical exception. The crate-naming kernel records it as a documented doctrinal bounded-extension tied to ADR-0053 (sanctioned primitives), distinct from the layer-enum canonical surface.

### Effect on the 9 crates originally enumerated

| Crate | Resolution | Rationale |
|---|---|---|
| `tools/oya-governance-portfolio-citation` | RENAME → `*-app` | Binary tool; canonical `-app` suffix per layer enum. |
| `tools/oya-governance-predictable-naming` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-governance-archive-orphan` | RETIRED by ADR-0118 | One-time IP-008 lane removed after M01-P18 replaced the grit-era cutover substrate. |
| `tools/oya-governance-banned-primitives` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-governance-adr-shape` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-governance-authoritative-tracked` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-governance-purpose-audit` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-adapter-substitution-test` | RENAME → `*-app` | Binary tool; canonical `-app`. |
| `tools/oya-tooling-agent-read` | KEEP (doctrinal lock, not naming exception) | CLAUDE.md sanctioned primitive; name fixed at agent-operating-contract layer. Recorded as a doctrinal bounded-extension under ADR-0053, NOT a layer-enum exception. |

All 8 renames execute as C1-shaped atomic commits per ADR-0054 §grit-scaffold-claim-pattern.

## Amendment 2026-05-15 — Superseded (canonical lifecycle transition)

**Lifecycle transition:** `Superseded by self` → `Superseded by ADR-0105`.

**Rationale.** The 2026-05-15 self-supersession amendment above replaced the original "implicit-app" decision with a `tools/` canonical-suffix binding rule. That replacement rule has been fully absorbed by ADR-0105 §"Amendment 2026-05-15 — `tools/` canonical-suffix binding (paired with ADR-0107 supersede)" (commit `1d07b63`), which states verbatim:

> "Every crate under `tools/` MUST end in a canonical layer suffix from the 13-value enum above (or match a documented Adopted Pattern: `oya-check-<feature>` or `*-adapter-<backend>`). The `tools/` directory is an organizational hint; the layer suffix is the naming declaration. For binary-shape (`[[bin]]`) tools, the canonical layer suffix is **`-app`** ..."

ADR-0105 also retroactively rejects the original "Tools/-implicit-app convention" alternative and records the `oya-tooling-agent-read` doctrinal bounded-extension at the agent-operating-contract layer (ADR-0053). Every clause of ADR-0107's self-amendment is now present in ADR-0105. ADR-0107 carries no unique decision content.

**Why supersede rather than amend further.** Per `feedback_no_exceptions_canonical.md`, superseded ADRs are a canonical lifecycle stage — supersession is the prescribed transition when content is fully absorbed by a sibling ADR. Keeping ADR-0107 as a self-referential supersede would split the canonical surface across two ADRs for one rule.

**Forensic retention.** Per ADR-0108 default-aware schema (30 days deprecation → 90 days removal, both anchored at `sunset_at`), this file enters the `DEPRECATED` state on 2026-05-15 and is scheduled for removal on 2026-08-15 (3 months). The sunset frontmatter (`sunset_at`, `deprecation_at`, `removal_at`, `sunset_topic = tools-implicit-app-exception-superseded`) makes the transition machine-readable for the sunset-lifecycle lane.

**Effect on downstream citations.** ADR-0108 §"Lane shape" and ADR-0109 §"Naming justification" cite "ADR-0107 §Amendment 2026-05-15" for the `tools/*-app` binding; those citations remain valid as historical references but new authors SHOULD cite ADR-0105 §"Amendment 2026-05-15 — `tools/` canonical-suffix binding" directly. No textual rewrite is required during the retention window; sweeps occur naturally as documents are touched.

## Original (superseded) decision — for history

(Original §"Decision" content preserved below for traceability. Do NOT cite.)

~~**Any crate under `tools/` is implicitly the `app` layer.**~~ Superseded by 2026-05-15 amendment — implicit-by-location is no longer a sanctioned naming surface. The 9 crates enumerated above are resolved either by explicit rename to canonical layer suffix or by recorded doctrinal bounded-extension (only `oya-tooling-agent-read`).

## Pre-amendment Status (historical)
Originally `Accepted` on 2026-05-15. Superseded the same day by user directive prioritizing predictable canonical naming over directory-implicit conventions.

## Context

The 2026-05-15 crate-naming audit (`specs/crate-naming-audit.json`) flagged 9 crates in `tools/` that lacked an explicit layer suffix:

```
tools/oya-governance-portfolio-citation
tools/oya-governance-predictable-naming
tools/oya-governance-archive-orphan  # retired by ADR-0118
tools/oya-governance-banned-primitives
tools/oya-governance-adr-shape
tools/oya-governance-authoritative-tracked
tools/oya-governance-purpose-audit
tools/oya-adapter-substitution-test
tools/oya-tooling-agent-read
```

Per ADR-0056 §"12-Value Layer Enum" all crates must end in a canonical layer suffix. The 9 above do not. The original audit verdict proposed renaming each to `*-app`.

Two issues with the per-crate rename:
1. **Mechanical cost** — 9 atomic renames (mv + Cargo.toml + workspace.members + every importer), one of which is `oya-tooling-agent-read` (the CLAUDE.md sanctioned primitive whose name is doctrinally fixed).
2. **Convention already exists** — every crate under `tools/` IS a binary. The directory location already declares the layer. The `*-app` suffix would be redundant tautology (`tools/` + `-app` = "binary binary").

ADR-0105 §Adopted Patterns already formalized `oya-check-<feature>` as a self-layering convention (the µservice IS the layer). The same pattern applies to `tools/`: the directory IS the layer.

## Decision

> **SUPERSEDED — historical content preserved for traceability. See §"Amendment 2026-05-15 — no-exception canonical naming" and §"Amendment 2026-05-15 — Superseded" above for the current (post-supersession) state.**

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
| `tools/oya-governance-portfolio-citation` | app | ✓ |
| `tools/oya-governance-predictable-naming` | app | ✓ |
| `tools/oya-governance-archive-orphan` | retired | ADR-0118 |
| `tools/oya-governance-banned-primitives` | app | ✓ |
| `tools/oya-governance-adr-shape` | app | ✓ |
| `tools/oya-governance-authoritative-tracked` | app | ✓ |
| `tools/oya-governance-purpose-audit` | app | ✓ |
| `tools/oya-adapter-substitution-test` | app | ✓ |
| `tools/oya-tooling-agent-read` | app | ✓ |

The audit's residual non-compliant count drops from 18 to 9 (the remaining 9 are the 3 `*-runtime` plus 6 vcs/saas one-offs that need real per-crate decisions).

## Consequences

- **`oya-governance-predictable-naming-kernel`** must be updated to recognize the tools/-implicit-app convention. Until then, the lane would flag these 9 crates as non-compliant.
- **`specs/crate-naming-audit.json`** is amended in this commit to mark the 9 tools/ crates as compliant via ADR-0107.
- **Cargo.toml `[workspace.metadata.oya]`** comment block updated to document the convention alongside the check-family and backend-suffix patterns from ADR-0105.
- **`oya-tooling-agent-read` no longer requires the "naming-exceptions" doctrinal bounded-extension** I added in ADR-0105 (it's now compliant via the standard convention).

## Drivers

- decision-principles.json DP-06 (Bounded scope per doc) — directory IS the layer when the convention is unambiguous.
- decision-principles.json DP-03 (Mechanical prevention over process) — convention-by-location is mechanically checkable.
- forbidden-operations.json FO-01 (No parallel canonical trees) — `tools/` is the canonical-binary-tree; redundant suffix would create a parallel naming axis.

## Alternatives Considered

1. **Per-crate rename to `*-app`.** Rejected: 9 atomic renames (workspace.members + importers + scripts that invoke by name), redundant tautology, breaks `oya-tooling-agent-read` doctrinal name.
2. **Allow tools/ crates to use ANY layer suffix.** Rejected: too permissive; defeats the BNF discipline.
3. **Restrict tools/ to binaries-only (no lib surface).** Rejected: the existing `oya-governance-purpose-audit` has a lib surface (it's both a lib + a bin); same for `oya-tooling-agent-read`. Production crates under `tools/` legitimately expose both surfaces for testing.

## Follow-ups

1. Update `oya-governance-predictable-naming-kernel` (Phase E successor-IP) to:
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
- specs/crate-naming-audit.json (per-crate compliance ledger)
- User directive 2026-05-15: "if they are well defined and serve a purpose, consider adopting them"
