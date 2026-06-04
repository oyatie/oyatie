# Oyatie

Oyatie is a cohesive ecosystem-as-a-service across SaaS, Workspace, Vertical, Foundry, Cloud, Search, and Ads + Analytics.

Canonical guidance and machine-readable specs live under [`docs/`](docs/) and [`.omc/`](.omc/). Humans typically start here; agents read [`AGENTS.md`](AGENTS.md) and [`CLAUDE.md`](CLAUDE.md) for the canonical entry-point pointers into machine-readable artifacts.

## Run

```sh
scripts/check.sh
cargo run -p oya-dev-cli -- demo
```

The current runnable slice is W-Foundation: tenancy, identity, data-use boundary, cell routing, audit chain, capability policy, regional packs, Object Graph, and idempotent outbox.

## License

This repository is proprietary and all rights are reserved. See [`LICENSE`](LICENSE)
for the repository-default IP posture. Third-party materials and files/components
with explicit license notices remain governed by those notices for those materials
only.
