# Oyatie

Oyatie is a cohesive ecosystem-as-a-service across SaaS, Workspace, Vertical, Foundry, Cloud, Search, and Ads + Analytics.

Canonical guidance lives under [`docs/`](docs/). Before changing implementation, read [`docs/AGENTS.md`](docs/AGENTS.md), [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md), [`docs/DESIGN.md`](docs/DESIGN.md), and [`docs/decisions/ADR-0015-architectural-flattening-target.md`](docs/decisions/ADR-0015-architectural-flattening-target.md).

## Run

```sh
scripts/check.sh
cargo run -p oya-tooling-cli-dev-runtime -- demo
```

The current runnable slice is W-Foundation: tenancy, identity, data-use boundary, cell routing, audit chain, capability policy, regional packs, Object Graph, and idempotent outbox.
