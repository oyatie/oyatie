# Oyatie

Oyatie is a cohesive ecosystem-as-a-service across SaaS, Workspace, Vertical, Foundry, Cloud, Search, and Ads + Analytics.

Canonical guidance and machine-readable specs live under [`docs/`](docs/) and [`.omc/`](.omc/). Humans typically start here; agents read [`AGENTS.md`](AGENTS.md) and [`CLAUDE.md`](CLAUDE.md) for the canonical entry-point pointers into machine-readable artifacts.

## Run

```sh
buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check
infra/ci/buck2-affected-gate.sh origin/dev HEAD
# Local Cargo commands are advisory only when a dual Cargo+Buck2 setup is intentionally maintained.
```

The current runnable slice is W-Foundation: tenancy, identity, data-use boundary, cell routing, audit chain, capability policy, regional packs, Object Graph, and idempotent outbox.


## Current dev-lane bridge

ADR-0516 records the temporary GitHub/GitHub Actions lane-unlocker for highly parallel product, infra, and cloud work. Buck2 remains build/test/check authority; the native destination is cloud native, Kubernetes-native, and hyperscaler native.

Cloud auth/shared substrate and Oyatie product auth/shared substrate are decoupled now; no shared contract or shared surface until a later rewrite and rewire of Oyatie products to consume the Cloud IdP.
