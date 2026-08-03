# Oyatie

Oyatie is an owned, cloud-native, hyperscale platform built in Rust: a unified, productized
**delivery fabric** (SCM + CI + CD over one owned substrate) together with the cloud, workspace,
vertical, search, and analytics products that run on it. AI agents are the primary producers, and
quality is enforced and auto-remediated so that sub-standard output cannot enter the canonical tree.

Canonical, machine-readable specs live under [`docs/`](docs/), [`specs/`](specs/), and
[`registry/`](registry/). Agents read [`AGENTS.md`](AGENTS.md) and [`CLAUDE.md`](CLAUDE.md) for the
canonical entry-point pointers; the mandatory agent entry surface is
[`specs/masterplan.json`](specs/masterplan.json) (single-writer authority for live plan content,
work items, status evidence, and the dependency DAG — see
[`specs/root-hub-pointers.json`](specs/root-hub-pointers.json)). Architecture decisions live in
[`docs/decisions/`](docs/decisions/) (ADRs); the apex vision is the **Agentic Delivery Fabric**
(ADR-0516…0535).

How we reason, build, and review — the 16 general and hyperscale lenses (Cartesian doubt, Red Team,
blast radius, opportunity cost…), and the bars every change clears — is in
[`AGENTS.md`](AGENTS.md#engineering-principles--review-lenses). The reproducible
map→pilot→bounded-fan-out loop, review separation, verification progression, learning loop, and
evidence-led drafting contract are in
[`docs/AGENTS.md`](docs/AGENTS.md#reasoning-and-delivery-method).

## Build & verify

The canonical build is a fully hermetic, lifecycle-wide [buck2](https://buck2.build) graph — a
clean checkout builds and tests with no setup script and no prebuilt blobs:

```sh
buck2 build //...
buck2 test //...
```

Quality is enforced on every change by the cloud-ci gate fleet behind the **single required status
context `oya-ci-required`** (ADR-0515): conformance, accounting, cross-artifact agreement,
freshness, hygiene, security, and planning gates, each shipped as a neutral engine plus
policy-as-data so any repo can adopt it (pipeline-as-product). Live plan state, work items, and
status evidence live in [`specs/masterplan.json`](specs/masterplan.json); decision history is in
[`docs/decisions/`](docs/decisions/).

## License

This repository is proprietary and all rights are reserved. See [`LICENSE`](LICENSE)
for the repository-default IP posture. Third-party materials and files/components
with explicit license notices remain governed by those notices for those materials
only.
