---
read_contract:
  audience:
    - agents
    - humans
  read_timing_class: on-demand
  freshness_rule: "On-demand read; live at this path with single-writer mutations through protected PRs against dev; never a mandatory entry surface (the mandatory agent entry surface is exactly /specs/root-hub-pointers.json#agent_entry_surface_allowlist)."
---

# Oyatie

Oyatie is an owned, cloud-native, hyperscale platform built in Rust: a unified, productized
**delivery fabric** (SCM + CI + CD over one owned substrate) together with the cloud, workspace,
vertical, search, and analytics products that run on it. AI agents are the primary producers, and
quality is enforced and auto-remediated so that sub-standard output cannot enter the canonical tree.

Canonical, machine-readable specs live under [`docs/`](docs/), [`specs/`](specs/), and
[`registry/`](registry/). Agents read [`AGENTS.md`](AGENTS.md) and [`CLAUDE.md`](CLAUDE.md) for the
canonical entry-point pointers; [`HANDOFF.md`](HANDOFF.md) carries current cross-cutting state and
the active backlog. Architecture decisions live in [`docs/decisions/`](docs/decisions/) (ADRs); the
apex vision is the **Agentic Delivery Fabric** (ADR-0516…0535).

How we build and review — the review lenses (Cartesian doubt, Red Team, blast-radius, opportunity
cost…), the hyperscale architecture lenses, and the bars every change clears — is in
[`AGENTS.md`](AGENTS.md#engineering-principles--review-lenses).

## Build & verify

The build is moving to a fully hermetic, lifecycle-wide [buck2](https://buck2.build) graph — a clean
checkout builds and tests with no setup script and no prebuilt blobs:

```sh
buck2 build //cloud/cloud-ci/...
buck2 test //cloud/cloud-ci/...
```

Quality is enforced on every change by the canonical **conformance ratchet** — the single required
`oya-ci-required` gate suite (accounting, cross-artifact agreement, staleness, manifest hygiene,
layer-suffix, brand-residue, registry-drift). It is config-driven (`oya-ci.toml`) and reusable as a
product by any project. See [`HANDOFF.md`](HANDOFF.md) and [`docs/decisions/`](docs/decisions/) for
current build/CI state and the staged roadmap.

## License

This repository is proprietary and all rights are reserved. See [`LICENSE`](LICENSE)
for the repository-default IP posture. Third-party materials and files/components
with explicit license notices remain governed by those notices for those materials
only.
