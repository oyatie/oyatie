---
id: ADR-0105
title: 13-value canonical layer enum + check-family + backend-suffix patterns (amends ADR-0056)
status: Accepted
planning_impact: true
doc_status: published
owner: council-architecture
date: 2026-05-15
amends: [ADR-0056]
supersedes: [ADR-0107]
amended_by: [ADR-0106, ADR-0107, ADR-0565, ADR-0632]
relates_to:
  - ADR-0056-rust-clean-architecture-bnf.md
  - ADR-0104-ecosystem-expansion-toolchain-and-adapters.md
---

# ADR-0105: 13-value canonical layer enum + check-family + backend-suffix patterns (amends ADR-0056)

## Status
Accepted (amends ADR-0056 §"12-Value Layer Enum")

> **F-0029 RECONCILIATION (ratified 2026-06-07, door:one-way).** This ADR added `api` (12 ⇒ 13).
> The third inner value shown below as `application` was subsequently renamed to **`usecase`** by
> ADR-0106; the reconciled canonical enum is therefore: `kernel`, `domain`, `usecase`, `app`,
> `adapter`, `infrastructure`, `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, `api` (13 product
> values) + the governance-only `check` family. `runtime` is **not** canonical (→ `app`). The
> tables below are read through this banner; `application` ⇒ `usecase` wherever it appears as a layer.

## ADR-0632 product-protocol reconciliation

The layer enum is a crate-shape vocabulary, not an exposure allowlist. Public contracts are HTTPS REST documented by OpenAPI 3.2.0, signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, and WebSocket. GraphQL, public gRPC, gRPC-Web, and Connect are forbidden. The `grpc` layer is internal-only gRPC/proto3 over HTTP/2; the historical `graphql` token remains naming provenance only and cannot authorize a GraphQL crate or owned API surface under ADR-0565.

## Context

The 2026-05-15 crate-naming audit (`specs/crate-naming-audit.json`) classified 264 workspace.members against ADR-0056's 12-value enum and found three classes of non-compliance:

1. **21 crates use `-api` suffix** — a coherent, well-defined pattern for protocol-neutral contract-surface layers (cloud-*, identity, tenancy, ontology, policy-cedar, regional-pack, foundry-dashboard, *-compat). Renaming all 21 to `-rest`/`-grpc`/`-graphql` would lose the protocol-neutral framing that justified the suffix.

2. **36 crates use `oya-check-<feature>` form** — fitness-check µservice family with implicit self-layering. The crate IS a check; the layer is "lib + optional bin"; the feature name is the layer in spirit.

3. **13 crates use `*-adapter-<backend>` form** — backend qualifier on a canonical `adapter` layer (e.g., `oya-intelligence-account-adapter-inmemory`, `oya-cloud-*-adapter-{aws,fake,oci}` — though the latter were deleted per ADR-0104). The pattern is well-formed: layer is `adapter`; backend is a qualifier.

User directive 2026-05-15: *"if they are well defined and serve a purpose, consider adopting them. otherwise stick to canonical layer types."*

All three patterns are well-defined, multi-crate, purposeful. ADR-0056 §"12-Value Layer Enum (closed)" explicitly states "Adding a layer value is a **1-ADR action**." This ADR is that action.

## Decision

### Amendment 1 — Extend the canonical enum from 12 to 13 values: add `api`.

The 13-value canonical layer enum:

| Group | Values |
|---|---|
| Inner / pure (4) | `kernel`, `domain`, `application`, `app` |
| Outer / external (2) | `adapter`, `infrastructure` |
| Presentation / entry-point (7) | `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, **`api`** |

**`api` semantics.** Protocol-neutral contract-surface layer: declares typed inputs, typed outputs, and error variants without committing to HTTP/gRPC/GraphQL at the type level. Depends on `kernel` only. Distinguished from:

- `rest` / `grpc` / `graphql` — protocol-specific handler/route implementations (consume `api`)
- `sdk` — client library for *external* consumers (consume `api` types as wire schemas)
- `application` — use cases that orchestrate domain (consume `api` is OK; emit `api` is not)

`api` is *what* the µservice promises to expose; the protocol layers (`rest`/`grpc`/`graphql`) are *how* it exposes it. A µservice MAY ship an `api` crate without any protocol-specific layer (consumers are in-process); it MAY ship multiple protocol layers atop one `api`.

**Inward-only flow for `api`:** depends only on `kernel`. Producer of types. NOT a consumer of `domain`, `application`, or `adapter` — those depend on `api`, not the other way around.

### Amendment 2 — Formalize the `oya-check-<feature>` self-layering convention.

A crate matching `^oya-check-[a-z][a-z0-9-]*$` is a **fitness-check µservice**. Its layer is implicit: lib + optional bin, dual-purpose, scope-bounded to a single check. The feature name (e.g., `brand-residue`, `data-class`, `statelessness`) names the check; no explicit `-kernel`/`-app` suffix is required.

Constraints on the family:

- One crate per check. Splitting a check into multiple crates requires standard `-kernel` / `-app` suffixes (then it's no longer `check-family`).
- Pure logic + optional CLI entry. NO outbound I/O beyond `std::fs` for the file-walking runner and `std::process` for the binary.
- Consumers: `oya-dev-cli gate validate <name>` is the canonical caller. Direct importers must be other `check-family` crates only.

### Amendment 3 — Formalize the `*-adapter-<backend>` sub-suffix pattern.

A crate matching `*-adapter-<backend>$` where `<backend>` ∈ `{fake, inmemory, aws, oci, gcp, azure, postgres, redis, sqlite, ...}` is an **adapter for a specific backend**. The layer is `adapter`; the backend is a qualifier.

Constraints:

- The `<backend>` token MUST be in the recognized-backend set (currently open per CODEOWNERS; future tightening: ADR-required to add a new backend name).
- A `*-adapter-<backend>` MUST implement at least one port trait from the corresponding `*-kernel`.
- Backend MAY be a software-implementation name (`postgres`, `sqlite`) OR a cloud-vendor name (`aws`, `oci`, `gcp`, `azure`) OR a test-double marker (`fake`, `inmemory`).
- `fake` and `inmemory` are honest test-doubles with explicit "NOT FOR PRODUCTION" doc-comments (see `oya-intelligence-account-adapter-inmemory` per commit `c7fda53`).

## Consequences

- **264 workspace.members** are now classified as: 245 canonical-suffix + 21 `api` (newly canonical) + 36 check-family (newly canonical) + 13 backend-suffix (newly canonical) + 41 - (21 + 36 + 13) = -29 ← (math fix: most non-compliant rows were already in canonical-suffix; the 41 non-compliant from the audit had 21 in the `api` group; the residual 20 are real cases needing per-crate rename, not pattern-formalization).
- **Updated non-compliant count post-amendment:** 18 crates (audit non-compliant 41 minus 21 `api` adoptions, minus 2 already-removed `tooling-agent-read` exception). The remaining 18 are:
  - 3 × `*-runtime` (per ADR-0056 §"Concrete migration" lines 283-289; scheduled rename to `*-app`)
  - 7 × tools/`oya-governance-<feature>` (decide: tools/-implicit-app convention, or add `-app` suffix per crate)
  - 8 × one-off drift (vcs-*-{controller,mergequeue,ratchet,gate,adapters}, saas-plugin-marketplace, adapter-substitution-test, oya-governance-purpose-audit)
- **`oya-governance-predictable-naming-kernel` lane** must be updated to enforce the 13-value enum + adopted patterns. Until then, the lane is too strict (it would flag legitimate `*-api` and `oya-check-*` crates).
- **`specs/crate-naming-audit.json`** is updated in this same commit to mark the 21 `api`, 36 check-family, and 13 backend-suffix crates as compliant.

## Drivers

- audit finding (2026-05-15): "52 of 297 crates already violate the closed enum the BNF claims to enforce"
- user directive (2026-05-15): "audit enums layers as well" + "if they are well defined and serve a purpose, consider adopting them"
- `decision-principles.json` DP-06 (Bounded scope per doc) — each pattern is bounded + clearly scoped
- `forbidden-operations.json` FO-01 (No parallel canonical trees) — formalizing patterns prevents drift

## Alternatives Considered

1. **Rename all 21 `*-api` to `*-rest`/`*-grpc`/`*-graphql`.** Rejected: `api` carries protocol-neutral semantics that `rest` does not. Force-renaming loses meaning + creates 21 atomic-rename commits with no semantic gain.

2. **Keep `*-api` as non-compliant, narrow ADR-0056 instead.** Rejected: 21 crates is too widespread to be "drift"; it's a coherent pattern that the BNF should recognize.

3. **Adopt `*-api` informally without ADR amendment.** Rejected: violates ADR-0056's own "1-ADR action" rule for enum extensions.

4. **Tools/-implicit-app convention** (any crate under `tools/` is implicitly layer `app`). Considered; scheduled-for-distinct-tracked-work to a separate ADR. The current 7 tools/-prefixed non-compliant crates can be renamed to `*-app` per the existing convention without needing a new pattern.

## Follow-ups

1. Update `specs/crate-naming-audit.json` to reflect the 13-value enum + adopted patterns. **Done in this commit.**
2. Update `oya-governance-predictable-naming-kernel` to recognize the 13-value enum + adopted patterns. Tracked separately.
3. Per-crate rename for the 18 remaining non-compliant entries (3 runtime + 7 fitness-tool + 8 one-offs). Each is its own C1-shaped atomic commit.
4. Update `Cargo.toml [workspace.metadata.oya]` comment block (line 266) which references the "Layer enum (12 canonical values)" — bump to 13.

## Amendment 2026-05-15 — `tools/` canonical-suffix binding (paired with ADR-0107 supersede)

User directive (2026-05-15): *"for 9 dont allow exceptions. fix our adr and other documents that declare exceptions. stick with canonical and if you need addition to canonical make the edit."*

### Binding

Every crate under `tools/` MUST end in a canonical layer suffix from the 13-value enum above (or match a documented Adopted Pattern: `oya-check-<feature>` or `*-adapter-<backend>`). The `tools/` directory is an organizational hint; the layer suffix is the naming declaration.

For binary-shape (`[[bin]]`) tools, the canonical layer suffix is **`-app`** (composition-root binary per ADR-0056 §"Layer semantics > app"). This is the documented binding of `app` to the `[[bin]]` shape; no enum extension is required because the existing `app` value already covers binary tools.

For lib+bin dual-shape tools, the suffix follows the lib intent: dual-purpose check µservices use `oya-check-<feature>` (Adopted Pattern); other dual-shape tools take the suffix of the dominant surface.

### Effect on Alternatives Considered

The fourth "Alternatives Considered" entry below (the original "Tools/-implicit-app convention" deferral) is REJECTED retroactively. ADR-0107's implicit-app exception is superseded; tools/ crates take explicit canonical suffixes.

### Doctrinal carve-out (NOT a naming exception)

`oya-tooling-agent-read` retains its name because CLAUDE.md declares it a sanctioned coordination primitive (ADR-0053). The carve-out is at the agent-operating-contract layer, not the layer-enum surface. The predictable-naming fitness kernel records this as a doctrinal-lock entry tied to ADR-0053, distinct from layer-enum compliance.

### Crate-naming kernel update

`oya-governance-predictable-naming-kernel` is updated in the same commit series to:
- REMOVE the tools/-implicit-app shortcut (no `declared_role = None` pass for tools/ crates).
- ADD a `DOCTRINAL_CARVE_OUTS` allowlist limited to `oya-tooling-agent-read` (citation: ADR-0053 + CLAUDE.md sanctioned primitives).
- Require canonical suffix for every other `tools/` crate.

## Amendment 2026-05-15 — `ALLOWED_DEPENDENCY_ROLES` reconciliation with the 13-value canonical enum

### Context

The architecture-boundaries gate ports `scripts/check-architecture-boundaries.sh`'s Python `ALLOWED_DEPENDENCY_ROLES` table verbatim into `crates/oya-dev-cli/src/commands/gate/architecture_boundaries.rs::allowed_dependency_roles()` (Wave 2 of the shell/python replacement program; audit row B-2 in `evidence/audits/shell-python-replacement-audit-2026-05-15.md`). The ported table contains 11 role keys:

```
kernel, domain, application, app, api, worker, adapter, rest, infrastructure, test, runtime
```

These pre-date the 13-value canonical enum defined in this ADR and amended by ADR-0106. The gap is:

| Legacy role (in `ALLOWED_DEPENDENCY_ROLES`) | Canonical 13-value equivalent | Notes |
|---|---|---|
| `application` | `usecase` (ADR-0106 rename) | 22 catalog records still use `application` as of 2026-05-15 |
| `runtime` | `app` (ADR-0105 §"Concrete migration" / ADR-0056) | 6 catalog records still use `runtime` |
| `test` | (no canonical value; `cfg(test)` is the canonical exemption per `oya-governance-predictable-naming-kernel`) | 4 catalog records still use `test` |
| `kernel`, `domain`, `app`, `adapter`, `rest`, `infrastructure`, `worker`, `api` | Same | Already canonical |
| Missing from legacy: `cli`, `grpc`, `graphql`, `sdk`, `usecase` | — | No catalog records use these yet (zero edges to validate); adding requires the catalog migration below |

### Decision (Choice (a) — staged migration, canonical `usecase` active for new records)

The architecture-boundaries gate keeps legacy `application`, `runtime`, and `test` rows only as transitional compatibility for existing catalog records. As of the 2026-05-16 gate remediation, ADR-0106's canonical `usecase` role is active in the dependency matrix:

- `usecase` may depend on `kernel` and `domain` only.
- `app` may depend inward on `kernel`, `domain`, `application` (legacy), `usecase`, `adapter`, and `rest`.
- `app -> app` remains forbidden. Do not make one deployable composition root depend on another deployable composition root; shared orchestration belongs in `usecase`.

The remaining reconciliation is staged in three successor-IP changes:

1. **Migrate legacy `application` catalog records → `usecase`** (paired with active workspace-crate renames from ADR-0106). Update `registry/catalog/<name>.yaml` `role:` lines in lockstep with each crate rename.
2. **Migrate legacy `runtime` catalog records → `app`** (paired with the rename plan in ADR-0056 §"Concrete migration"). Each `*-runtime` crate renames to `*-app`; the catalog record's `role:` flips at the same time.
3. **Remove legacy `test` catalog records** OR retain the `test` row in the dependency matrix as a cfg(test) exemption marker. The honest path is removal — test-only crates take canonical layer suffixes; the `test` role is not in the canonical enum.

After those successor-IPs land, `ALLOWED_DEPENDENCY_ROLES` drops `application`, `runtime`, and `test`, then adds the remaining canonical entry-point roles (`cli`, `grpc`, `graphql`, `sdk`) as real catalog records require them. The Rust source-of-truth lives at `crates/oya-dev-cli/src/commands/gate/architecture_boundaries.rs::allowed_dependency_roles()` and is reviewed under the same ADR-0107 / canonical-app-layer surface.

### Until then

The architecture-boundaries gate is the source of truth for inter-crate edges; the legacy role names in its matrix are **transitional**, not canonical. New catalog records MUST use canonical names (`usecase`, `app`, plus a canonical-suffix crate-rename) from ADR-0106 onward. The predictable-naming kernel (lib.rs `ALLOWED_ROLES`) already rejects `application`/`runtime`/`test` for new crates; existing legacy records are grandfathered via the dependency-roles table.

### Traceability

Tracked as a successor-IP entry in `evidence/audits/shell-python-replacement-audit-2026-05-15.md` so the migration sequence sits next to the rest of the Wave 2 work.

## References

- ADR-0056 §"12-Value Layer Enum (closed)" — the enum this ADR amends
- ADR-0104 — ecosystem-expansion principle (why some adapters stay scheduled-for-distinct-tracked-work)
- specs/crate-naming-audit.json — per-crate classification table
- 2026-05-15 user directives
