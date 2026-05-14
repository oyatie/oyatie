# Oyatie

Oyatie is a cohesive ecosystem-as-a-service across SaaS, Workspace, Vertical, Foundry, Cloud, Search, and Ads + Analytics.

Canonical knowledge lives in machine-readable form per ADR-0069 + the markdown-retirement-policy. The 3 surviving Markdown files at repo root (`README.md`, `CLAUDE.md`, `AGENTS.md`) are thin pointer hubs; humans read summaries here, agents read [`.omc/specs/root-hub-pointers.json`](.omc/specs/root-hub-pointers.json) for the canonical entry-point registry.

## Machine-readable entry points

Authoritative list: [`.omc/specs/root-hub-pointers.json`](.omc/specs/root-hub-pointers.json). Top entries:

- **Agent contract:** [`docs/AGENTS.md`](docs/AGENTS.md) → migrating to `.omc/specs/agent-operating-contract.json` (PHASE-5; will absorb decision principles from retired Constitution)
- **Masterplan:** [`docs/MASTERPLAN.md`](docs/MASTERPLAN.md) → migrating to `.omc/specs/masterplan.json`
- **Active-artifact contract v3.0.0 (ADR-0069):** [`.omc/specs/active-machine-readable-artifact-contract.json`](.omc/specs/active-machine-readable-artifact-contract.json)
- **Knowledge-graph catalog (Ontology):** [`.omc/registries/knowledge-graph-catalog.json`](.omc/registries/knowledge-graph-catalog.json)
- **Capability registry (control plane):** [`.omc/registries/artifact-capabilities-registry.json`](.omc/registries/artifact-capabilities-registry.json)
- **Reusable blocks (DRY):** [`.omc/registries/reusable-building-blocks-registry.json`](.omc/registries/reusable-building-blocks-registry.json)
- **Markdown retirement ledger:** [`.omc/ledger/markdown-retirement-ledger.json`](.omc/ledger/markdown-retirement-ledger.json)
- **ADR-0015 (flat crates):** [`docs/decisions/ADR-0015-architectural-flattening-target.md`](docs/decisions/ADR-0015-architectural-flattening-target.md)

## Run

```sh
scripts/check.sh
cargo run -p oya-dev-cli -- demo
```

The current runnable slice is W-Foundation: tenancy, identity, data-use boundary, cell routing, audit chain, capability policy, regional packs, Object Graph, and idempotent outbox.
