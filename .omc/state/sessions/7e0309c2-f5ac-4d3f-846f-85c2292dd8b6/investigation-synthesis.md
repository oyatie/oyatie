# Investigation Synthesis — "What is the problem? What are we trying to achieve?"

**Authoritative inputs**: `consensus-masterplan-2026-05-13.md` (Accepted, b4eb035) + `ralplan-ops-portal-2026-05-13.md` v7 (Accepted, critic r2 codex `br2nkyycu`) + `ralplan-docs-portal-2026-05-13.md` v6/v7 (Accepted, critic r2 codex `br2nkyycu`)

## §1 Masterplan structure (verified)

**Authority chain**: `docs/MASTERPLAN.md → ADR-0063 (doc-suite coverage) → ADR-0064 (canonical-base + localization-packs) → consensus-masterplan-2026-05-13.md`

**Horizon**: M02-substrate → M03-first-tenant → M04-healthcare-kr → M05-connect-personal → M06-fintech-kr → M07-industrial-kr → M08-enterprise-breadth → M09-us-expansion → M10-eu-expansion → M11-healthcare-intl → M12-hyperscaler-maturity

**Three pillars**:
1. Canonical global base + localization seams/adapters/packs. Korea = pack #1 (foundational; M01–M07 ship canonical + KR lockstep).
2. Documentation-suite coverage CI-enforced via `lean-a5-documentation` (report-only at HEAD → BLOCKER at M02-P22).
3. Workflow + Ontology = sole inter-µservice adapter layer (per ADR-0059). Products never call each other directly.

**5 products user named** (corporate / connect / ops / workflow / ontology) map onto M03-first-tenant phases:
- corporate = Application B2B shell + SaaS administration (M02-P19 → M03 application phases)
- connect = M03-P04 Connect Pro Mail + M03-P05 Connect Pro Messenger
- ops = `ops.oyatie.com` 20-BC console; Wave 1 = docs BC ships across M03-P04..P08
- workflow = Workflow Studio (M03-P06/P07)
- ontology = Ontology Object/Link/Action/Function browser (Wave 3 of ops at M03-P07)

**Real authoring backlog at HEAD** (per masterplan §Consequences): **1,136 violations** = 933 impl-plan section gaps + 119 canonical artifacts + 72 pack overlays + 12 milestone artifacts. Dispatched to executors in "autopilot Phase 2".

## §2 Ops Wave 1 actual scope (verified)

Per `ralplan-ops-portal-2026-05-13.md` §3 (Option α — RECOMMENDED) + companion `ralplan-docs-portal-2026-05-13.md`:

**Wave 1 = docs BC alone**, shipped as `oya-ops-docs-*` crate family. NOT workspace-shell. NOT a transport-pattern layer-stack across multiple BCs.

| Deliverable | Count | Detail |
|---|---|---|
| Crates | 24 | `oya-ops-docs-{portal,generator,manifest,search,cross-ref,live-diff}-{kernel,domain,application,adapter,rest,worker,cli,app}` |
| Extractors | 16 | G1 hot (5): cargo_metadata, markdown-frontmatter, pack.yaml, phase-spec-frontmatter, lanes.yaml. G2 warm (4): rustdoc-JSON, openapi, proto, async-graphql. G3 warm (4): SQL-migrations, cargo-machete, cargo-udeps, cargo-deny. G4 cold (3): ICM, grit, GH Actions. |
| Watch daemon | 1 | `oya-ops-docs-watch` — SSE fan-out, per-cell, hot/warm/cold scheduler |
| Leptos MVP pages | 13 | `/`, `/microservices`, `/microservices/<id>`, `/decisions`, `/decisions/<id>`, `/milestones`, `/milestones/<id>`, `/phases/<m>/<p>`, `/endpoints`, `/dep-graph`, `/dead-code`, `/live`, `/manifest` |
| CI lanes (separate binaries) | 4 | lean-a5 (existing `oya-check-documentation`), lean-a6 (NEW `oya-check-docs-generated`), lean-a7 (NEW `oya-check-endpoint-coverage`), lean-a8 (NEW `oya-check-dead-code` BLOCKER day 1) |
| Cedar fragments (M02-P20 prereq) | 4 | `ops-public.cedar`, `ops-internal-public.cedar`, `ops-internal-private.cedar`, `ops-system-only.cedar` |
| pgroonga + pgvector | 1 set | Full-text + semantic search indexes |

**Wave 2 (NOT Wave 1)**: overview + dashboards + tech-stack + architecture BCs at M03-P06 IP-X1..X4. Owns root `/` landing surface.

**Wave 1 sub-authority** (extends §1 chain): `+ ADR-0061 + ADR-0065 + ADR-0066`. **Decision**: Option B-prime (no new phase IDs); 8 new IPs total inside existing M02-P19/P20/P21/P22 + M03-P04/P05/P06/P08 phases.

## §3 Current code state (origin/main HEAD `4d6bf91`)

20 session commits authored:

| Crate family | Count | Wave assignment per masterplan |
|---|---|---|
| `oya-ops-workspace-shell-*` (kernel/adapter/application/rest/runtime) | 5 | **Not in any Wave** — workspace-shell ≠ docs BC; closest fit is M02-P19 Application shell substrate |
| `oya-ops-docs-portal-*` (kernel/adapter/application/rest; no runtime) | 4 | Wave 1 substrate (partial; rest layer's routes don't match Wave 1 surface set) |
| `oya-http-{router,middleware,sse,runtime,tenant-mw,deadline-mw,telemetry-mw}-*` | 7 | Not in masterplan at all — masterplan inherits Bominal ADR-0209 (Leptos SSR), not custom hyper stack |
| `oya-foundry-architecture-map-{kernel,app}` | 2 | Visualization-as-code directive substrate; not Wave 1 |
| `oya-check-{active-artifact-contract,cedar-fragment-coverage,openapi-rest-route-parity}` | 3 | Cross-cutting; cedar-fragment-coverage tangentially supports lean-a9 |
| `contracts/ops-{workspace-shell,docs}.openapi.yaml` | 2 | Wave 1 docs sub-plan uses 13 Leptos pages, not OpenAPI 3.2; these are misaligned |
| ADR-0090 hyper backbone | 1 | **Conflicts** with dep-seam-phaseout R5 consensus plan |

## §4 Gap analysis

### MIS-ALIGNED
- **workspace-shell scope**: I treated `oya-ops-workspace-shell` as a Wave 1 BC. Per ops-portal ralplan §6(a), workspace-shell is implicit in the "composition-root binary `oya-ops-app`" — NOT a BC. The actual Wave 1 BCs (per docs sub-plan §6(a)) are `portal / generator / manifest / search / cross-ref / live-diff`.
- **docs-portal routes**: my `/workspace/docs/manifest`, `/workspace/docs/live`, `/workspace/docs/api/v1/extractors/{id}/refresh` paths don't match the 13 MVP Leptos page paths from docs sub-plan §6(c).
- **Transport choice**: hyper + tokio direct REST is not the masterplan's HTTP stack. Per Bominal ADR-0209 inheritance, Leptos SSR is the canonical client tier — same stack as Workflow Studio + Connect. My OpenAPI 3.2 contracts presuppose a non-Leptos transport.
- **ADR-0090 hyper backbone**: just-merged R5 dep-seam-phaseout plan declares hyper/tokio as known debt to phase out post-ontology-v1. ADR-0090 should be marked Superseded by ADR-0091 once the phase-out plan lands.

### MISSING (Wave 1 not yet built)
- 16 extractors (zero authored; canonical for ADR-0066 hot/warm/cold)
- `oya-ops-docs-watch` daemon (zero; required for `/live` SSE)
- 13 Leptos pages (zero; my crates are OpenAPI REST, not Leptos)
- 4 CI lane binaries (zero; the 3 check kernels I shipped don't overlap lean-a5/a6/a7/a8)
- pgroonga + pgvector adapters (zero)
- 4 Cedar policy fragments at M02-P20 (declared in registry only; not authored)
- M02-P19 IP-X1 `[workspace.metadata.oya.microservices]` registration for `ops` parent µservice + `ops.docs` BC (partial in `.omc/registries/microservices.json`; not in workspace metadata)

### BLOCKED
- Parallel-session WIP: `Cargo.toml [workspace.members]` declares 440 entries; only 148 exist on disk. **292 missing crate dirs block any new code compile** (workspace metadata resolution fails).
- 1,136 masterplan-coverage violations at HEAD (per `oya-check-documentation`); parallel-dispatchable but not by me.

## §5 What is the actual problem?

**Three-layer problem**:

1. **Build blocker (parallel session WIP)**: 292 missing workspace-member crate dirs prevent any new code from compiling. This blocks all of my work + theirs until they author or remove those entries.

2. **Misaligned freelance work (my session)**: 20 commits of hyper-based ops transport pattern were authored without grounding in the docs-portal ralplan. The work is structurally sound but mis-scoped: workspace-shell isn't a Wave 1 BC; docs-portal routes don't match the 13 MVP pages; transport stack doesn't match Bominal ADR-0209 (Leptos SSR); ADR-0090 hyper backbone contradicts the R5 dep-seam-phaseout plan.

3. **Hidden coordination debt**: dep-seam-phaseout R5 plan (3-of-3 reviewer consensus achieved) targets hyper removal post-ontology-v1, while the same week I added hyper as a backbone (ADR-0090). The two artifacts are simultaneously Accepted and mutually inconsistent — one of them must be superseded.

## §6 What are we trying to achieve?

**Immediate (Wave 1 / M03-P04..P08)**: ship `ops.oyatie.com` Wave 1 = live-introspection docs portal = 24 docs crates + 16 extractors + watch daemon + 13 Leptos pages + 4 CI lanes, exposing the portal as the second Application B2B product after Workflow Studio. M03-P06 deliverable.

**Medium-term (5-product release: corporate / connect / ops / workflow / ontology)**: ship 5 customer-facing products on the canonical global base + KR pack overlay (pack #1 foundational). M01-M07 ships canonical + KR lockstep. Wave 1 docs portal is the FIRST customer-facing surface of "ops" product.

**Long-term (post-ontology-v1)**: execute the dep-seam-phaseout R5 plan to remove hyper/tokio/bytes external deps via systematic phase-out (ADR-0091 + ADR-0092 + ADR-0093 + ADR-0094). Replace with std-only / canonical building blocks per Linus-style "support everything ourselves with 0-to-minimal-dependency."

## §7 Recommendation: next 1–2 slices

**Slice 1 — Unblock the workspace**:
Surface to parallel session: "`Cargo.toml [workspace.members]` declares 440 entries; only 148 exist. 292 missing crate dirs need to be authored or removed before any new code compiles." This is their work, not mine. Until resolved, no Wave 1 substrate authoring is possible.

**Slice 2 — Reconcile mis-aligned freelance + Wave-1-misalignment in a single planning artifact**:
Author `.omc/plans/ralplan-ops-freelance-realignment-2026-05-14.md` that:
1. Reclassifies the 5 `oya-ops-workspace-shell-*` crates as **Wave 2 candidate substrate** for the workspace-shell BC under the "overview" BC at M03-P06 IP-Y1 — or retire them per `feedback_autonomous_implementation_artifacts.md` "stale removed in reality" if not adopted.
2. Reclassifies the 4 `oya-ops-docs-portal-*` crates as **prototype/learning artifacts** — the actual Wave 1 docs portal will rebuild against the 6 docs sub-plan BCs (`portal / generator / manifest / search / cross-ref / live-diff`).
3. Marks **ADR-0090 Superseded by ADR-0091** (per dep-seam-phaseout R5) — the hyper foundation is now under formal phase-out trajectory.
4. Authors the **smallest Wave-1-prereq**: M02-P19 IP-X1 = catalog registration of `ops` parent µservice + `ops.docs` BC in `[workspace.metadata.oya.microservices]`. This is a Cargo.toml-only edit, ~10 lines, doesn't depend on the 292-missing-crate blocker resolving.

**Defer**: actual Wave 1 substrate (24 docs crates + 16 extractors + watch daemon + 13 Leptos pages) until M02-P19 catalog registration + M02-P20 prereqs (4 Cedar fragments + 5 G1 hot extractors + lean-a8 binary scaffold) land. These are sequential per docs sub-plan §6(g) dispatch sequence.

## §8 Reviewer-verifiable citations

- Masterplan: `consensus-masterplan-2026-05-13.md:15` (horizon) + `:23-29` (3 pillars) + `:113` (1,136 violation backlog)
- Ops portal Wave 1: `ralplan-ops-portal-2026-05-13.md:44` (Wave 1 = docs BC) + `:108-131` (20-BC inventory) + `:139-148` (7 CI lanes)
- Docs sub-plan: `ralplan-docs-portal-2026-05-13.md:104-113` (6 BCs × layer matrix) + `:121-126` (16 extractors G1-G4) + `:160-164` (13 MVP pages) + `:211-217` (4 CI lane binaries) + `:222-261` (dispatch sequence with zero new phase IDs)
- dep-seam-phaseout R5 consensus: `.omc/plans/ralplan-dep-seam-phaseout-round-5.md` (Architect + Critic APPROVE round 5; codex APPROVE after §18.C spot-fix)
- Bominal inheritance: ADR-0209 (Leptos SSR), ADR-0117 (cell architecture), ADR-0132 (Cedar pillars)
