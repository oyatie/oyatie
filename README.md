# Oyatie

Oyatie is an owned, cloud-native, hyperscale platform built in Rust: a unified, productized
**delivery fabric** (SCM + CI + CD over one owned substrate) together with the cloud, workspace,
vertical, search, and analytics products that run on it. AI agents are the primary producers, and
quality is enforced and auto-remediated so that sub-standard output cannot enter the canonical tree.

Canonical, machine-readable specs live under [`docs/`](docs/), [`specs/`](specs/), and
[`registry/`](registry/). Agents MUST load root [`AGENTS.md`](AGENTS.md) / [`CLAUDE.md`](CLAUDE.md)
(INV-DOC-9 doctrine survival) for entry-point pointers and binding short-form law; the mandatory
agent entry surface is [`specs/masterplan.json`](specs/masterplan.json) (single-writer authority
for live plan content, work items, status evidence, and the dependency DAG — see
[`specs/root-hub-pointers.json`](specs/root-hub-pointers.json)). Architecture decisions live in
[`docs/decisions/`](docs/decisions/) (ADRs); the apex vision is the **Agentic Delivery Fabric**
(ADR-0516…0535).

How we build and review — the review lenses (Cartesian doubt, Red Team, blast-radius, opportunity
cost…), the hyperscale architecture lenses, and the bars every change clears — is in
[`AGENTS.md`](AGENTS.md#engineering-principles--review-lenses). The bounded fan-out, preservation,
and evidence-source rules are in
[`docs/AGENTS.md`](docs/AGENTS.md#bounded-delivery-and-preservation).

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
