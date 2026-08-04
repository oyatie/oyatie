# G026 console-shell and marketplace-plugin placement proof — 2026-08-02

State: **PLANNING_ONLY — FACE CLASSES PROVEN; EXACT LEAVES PENDING REVIEW; NO MOVE PLAN**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G026-APPLICATION-CONE-OWNER-SPLIT-2026-08-02.md`.  
No path, package, policy, generated face, PR, GitOps declaration, or cluster state was changed.

## Result

| Source | What it is | Placement class | Measured blocker |
|---|---|---|---|
| `oya-application-shell-frontend` | canonical Leptos portal shell: UI, SSR/hydration, design system, render envelope, client session state, module registry, token broker | `console/facade` | exact leaf spelling and relationship to existing ops workspace-shell composition require console-owner review |
| `oya-saas-plugin-app` | framework-free plugin invocation engine wrapping the marketplace manifest registry; future Wasmtime execution remains behind an adapter | `marketplace/core` | package/path importer in `billing/facade/saas-bench` plus exact core leaf spelling require marketplace-owner review |

Neither crate belongs in `app/application`. The shell is the console substrate surface under ADR-0615 Q12; the plugin runtime is a single-capability engine, not a tenant product composition.

## Console shell proof

The shell crate exports the production portal surface itself:

- `App`, `DashboardIsland`, static/SSR rendering and hydration entry points;
- design-system and render-envelope modules;
- shell capability registry and token broker;
- client session state and server bindings.

The existing console facade contains `workspace-shell-app`, `workspace-shell-rest`, and `docs-portal-rest`. Those leaves implement the **ops workspace-shell** route/catalog composition: route constants, surface catalog, ops visibility tiers, Hyper composition, and authn middleware. They do not implement the end-user Leptos portal shell.

Therefore this is not a duplicate semantic implementation and does not justify merging the portal into `workspace-shell-app` mechanically. Exact immutable-ref probes found both `console/facade/application-shell-frontend` and `console/facade/portal-shell` absent. The owner must choose the leaf after deciding whether the product-facing name or existing package identity governs.

Machine rewrite obligations include:

- `ci/facade/module-membership/capability-membership-policy.json` source-path row;
- shell catalog `contracts_exposed` path;
- `client-manifest.json` working-directory path;
- client-stack-discipline test fixture paths;
- the crate's own Buck labels and relative Cargo paths;
- any design-system, contract, SLO, dashboard, ownership, and deployment artifacts semantically owned by the shell.

The root workspace already uses the shape glob `oya/*/crates/oya-*`; the canonical workspace resolver, not hand editing the root member list, must prove the new capability path is included.

## Marketplace plugin proof

`oya-saas-plugin-app` calls itself a runtime, but its measured implementation is an engine/library:

- typed invocation request, context, outcome, audit row, and errors;
- deterministic invocation registry;
- dependency only on `marketplace/core/plugin-kernel`;
- no Wasmtime or transport implementation—the source explicitly reserves execution for a future adapter.

Under ADR-0562's closed grammar, an engine belongs in `core`; the `-app` suffix does not override what the code is. `marketplace/facade/plugin-app` would wrongly promote the engine to a sold surface. The least-invented destination class is `marketplace/core/<owner-approved-plugin-runtime-leaf>`.

Exact probes found `marketplace/core/plugin-runtime` and `marketplace/facade/plugin-app` absent. Existing `marketplace/core/plugin-kernel` is complementary, not overlapping: it owns manifest/listing/trust-tier/auction contracts; the source runtime consumes those contracts to execute invocations.

One live external importer exists and must move atomically:

- `billing/facade/saas-bench/Cargo.toml` path dependency;
- two `billing/facade/saas-bench/BUCK` labels;
- Rust imports in the bench library remain package-name based unless the owner-approved package rename changes them.

The bench is an end-to-end acceptance harness that composes workflow, marketplace kernel, and plugin runtime. Its dependency proves the plugin engine is live and prevents a path-only move without importer rewrites.

Machine rewrite obligations also include the module-membership policy source row, own Cargo/Buck paths, catalog/ownership rows, and producer-owned generated faces. No generated face is hand edited.

## Safe serial sequence

After independent approval and fresh immutable-tip collision/importer probes:

1. move the portal shell in the console capability lane with all path-locked shell consumers and semantic non-code ownership;
2. separately move the plugin engine in the marketplace capability lane with the billing bench importer transaction;
3. run codemod-owned path/package/importer rewrites and producer regeneration;
4. prove both with Buck2-authoritative targets and the relevant authz/membership/client-stack or bench contract gates.

The two moves must not be batched merely because the crates share a legacy source directory.

## Unresolved owner choices

- Console: `portal-shell` vs `application-shell-frontend` leaf spelling, and any reviewed composition relationship to `workspace-shell-*`.
- Marketplace: exact `plugin-runtime` leaf spelling and whether to retain package name for compatibility or rename atomically.
- Both: independent design approval and fresh collision proof immediately before execution.

## Non-actions and non-claims

- No `app/application` or generic app root.
- No exact destination leaf selected.
- No package rename or compatibility alias invented.
- No move-plan JSON.
- No source deletion or non-code reassignment.
- No generated artifact or frozen policy hand edit.
- No independent APPROVE; transport failure remains non-approval.
