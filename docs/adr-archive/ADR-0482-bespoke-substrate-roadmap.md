---
id: ADR-0482
status: Superseded
planning_impact: true
deciders: founder
date: 2026-05-28
authority: founder
doctrine_meta: true
owner: founder
supersedes: []
superseded_by: [ADR-0701]
related: [ADR-0388, ADR-0392, ADR-0394, ADR-0409, ADR-0434, ADR-0451, ADR-0474, ADR-0475, ADR-0476, ADR-0477, ADR-0478, ADR-0479, ADR-0480, ADR-0481, ADR-0483, ADR-0484, ADR-0506, ADR-0507, ADR-0508, ADR-0516, ADR-0520, ADR-0521]
amended_by: [kubers-anchor-2026-05-28, ADR-0394, ADR-0520, ADR-0521]
door: one-way
milestone: M-BESPOKE-ROADMAP
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0482 — Bespoke Substrate Roadmap: multi-decade kernel + OS ambition with phased timeline + bridges

## Status

Accepted 2026-05-28 — founder authority, foundational doctrine. Locked.

## Context

oyatie is a multi-decade kernel + OS + cloud-platform ambition. OSS substrates are bridges, not
destinations. Every hyperscaler at FAANG scale converged on bespoke everything:

- Google: Borg, Spanner, Colossus, Piper
- Meta: Buck2, Mercurial monorepo
- AWS: Aurora, IAM, Lambda, Nitro
- Microsoft: Azure internals end-to-end
- Cloudflare: Pingora, Workers runtime

oyatie follows that path. Founder direction (2026-05-28): _"We are building an entire kernel and
operating system. Anything is acceptable cost for bespoke. Just need to timeline it
appropriately. And have appropriate bridges in place."_

ADR-0388 established the doc-axis convention; ADR-0392 established the bespoke-over-oss doctrine.
This ADR is the top-level sequencing and bridge-discipline document for all bespoke work.

## Decision

Adopt a phased bespoke roadmap with explicit timeline tiers and bridge mappings. Unless a later
Accepted ADR explicitly narrows a component's bridge posture, each bespoke component ships with a
parallel OSS bridge, quality-gated cutover criteria, and tenant opt-in granularity. No
hard-deadline cutover — quality gates only. ADR-0394 is the explicit portal amendment: Backstage is
a bounded one-way import source only, never a parallel runtime, plugin host, or live authority.

## Phased Roadmap

### Tier 1 — NOW → 12 months (in flight or imminent)

| Bespoke | Supersedes OSS | Bridge during Phase-1 | Cutover criteria |
|---|---|---|---|
| oya-vcs (ADR-0409) | GitHub (ADR-0363) | Parallel-run; tenant opt-in per repo | Feature parity per-feature gates |
| Rust-native portal (ADR-0434, amended by ADR-0394) | First-party portal; Backstage is reference/import input only | Bounded one-way import of Backstage Catalog YAML; no Backstage runtime or plugin host | Import completeness and provenance validated; importer can then be removed |
| oya-notify (ADR-0451) | Postal/Haraka alternatives | None — bespoke from day 1 | N/A |
| oya-errors (ADR-0474) | GlitchTip (ADR-0466) | Phase-1 GlitchTip ingest endpoint | Sentry SDK protocol parity |
| oya-status (ADR-0475) | Gatus (ADR-0468) | Gatus runs parallel | OpenSLO/Mimir integration parity |
| oya-identity (ADR-0476) | Keycloak (ADR-0421) | Keycloak parallel-run | Feature parity: OIDC+OAuth2.0+WebAuthn+IdP federation+MFA |
| oya-code-search (ADR-0477) | Sourcegraph (ADR-0448) | Sourcegraph parallel-run | SCIP indexing + cross-references parity |
| oya-billing (ADR-0478) | Lago (ADR-0457) | Lago parallel-run; tenant opt-in | Plans + subscriptions + invoicing + payments parity |
| oya-meter (ADR-0479) | OpenMeter (ADR-0429) | OpenMeter parallel-run | Event ingest + meter aggregation parity |
| oya-cost (ADR-0480) | OpenCost (ADR-0443) | OpenCost parallel-run | K8s allocation parity |
| oya-flags (ADR-0481) | flagd (ADR-0428 OpenFeature backing) | flagd parallel-run | OpenFeature protocol compat parity |

### Tier 2 — 12 → 24 months (mid-term bespoke)

| Bespoke | Supersedes | Bridge | Cutover |
|---|---|---|---|
| oya-oncall | Grafana OnCall (ADR-0450) | Grafana OnCall parallel | IRM bundle UX parity |
| oya-ml | MLflow (ADR-0459) | MLflow parallel; Python compat shim | Experiment + model registry parity |
| oya-prompt-eval | Promptfoo (ADR-0460) | Promptfoo parallel | LLM eval framework parity |
| oya-realtime | Centrifugo (ADR-0472) | Centrifugo parallel | WebSocket + presence + multiplexing parity |
| oya-admin | AppSmith (ADR-0473) | AppSmith parallel | Retool-pattern low-code parity |
| oya-waf | Coraza (ADR-0454) | Coraza CRS rules compat layer | OWASP CRS evaluation parity |
| oya-pipelines | Apache Airflow (ADR-0458) | Airflow parallel; DAG compat shim | DAG orchestration parity |
| oya-webauthn | webauthn-rs (ADR-0507) | webauthn-rs parallel; MPL-2.0 bridge | Parity table green (ADR-0507) + oya-identity Phase-2 promotion gate |

### Tier 3 — 24 → 60 months (substrate-level bespoke)

| Bespoke | Supersedes | Bridge | Cutover |
|---|---|---|---|
| oya-events | Apache Pulsar (ADR-0397) | Pulsar parallel-run; protocol-compat ingress | Multi-tenant durable event bus parity at FAANG scale |
| oya-search | OpenSearch (ADR-0419) | OpenSearch parallel; Quickwit-rs hybrid | Full-text + analytics search parity |
| oya-workflow | Temporal (ADR-0399) | Temporal parallel; SDK shim | Durable execution parity |
| oya-graph | JanusGraph (ADR-0462) | JanusGraph parallel; TinkerPop compat | Distributed graph parity |
| oya-wide-column | Cassandra (ADR-0461) | Cassandra parallel | Wide-column NoSQL parity |
| oya-lakehouse | Iceberg+Lakekeeper (ADR-0413) | Iceberg format compat | Table format parity |
| oya-clickhouse | ClickHouse (ADR-0193) | ClickHouse parallel | OLAP query engine parity |
| oya-authn-device (ADR-0508) | OpenSK (Phase-1 reference firmware per ADR-0508) | OpenSK vendored at `tools/opensk-vendored/`; nRF52840 dev dongle for engineers | Tier-3 hardware-readiness + parity-table-green (ADR-0508) + first manufacturing run validated + OpenTitan port verified; cross-ref: oya-webauthn (Tier-2 RP partner, ADR-0507) + oya-crypto (Tier-4 crypto primitive, ADR-0506) |

### Tier 4 — 60+ months (kernel + OS bespoke)

| Bespoke | Supersedes | Bridge | Cutover |
|---|---|---|---|
| oya-os (ADR-0483) | Talos Linux (ADR-0378, ADR-0382) | Talos parallel for years | Linux capability parity (eBPF, containers, K8s) + Rust microkernel migration |
| oya-kernel | Linux kernel | Linux substrate underneath; gradual driver migration | POSIX + Linux ABI compat + Rust-native kernel components |
| oya-hypervisor | KVM/Firecracker/cloud-hypervisor | KVM parallel | TEE + nested virt parity |
| oya-runtime | containerd | containerd parallel; OCI spec compat | Container runtime parity |
| oya-crypto (ADR-0506) | aws-lc-rs (Phase-1 bridge per ADR-0506) | aws-lc-rs indefinitely (FIPS-validatable, drop-in) | kubers Phase-B kernel proofs landed + FIPS 140-3 module validation + hyperscaler-readiness gate #10 |

## Bridge Discipline

Every bespoke tier MUST have the following by default, except where a later Accepted ADR records a
bounded component-specific amendment. ADR-0394 supplies that amendment for the portal and permits
only a one-way Backstage catalog import, not a parallel Backstage runtime:

1. **Parallel-run period** — OSS Phase-1 and bespoke Phase-2 coexist; neither blocks the other.
2. **Feature parity target table** — explicit per-feature exit criteria per the bespoke-over-oss
   doctrine (ADR-0392).
3. **Tenant opt-in granularity** — tenants choose which implementation to use during transition.
4. **Rollback path** — documented in the implementation plan before any cutover begins.
5. **No hard-deadline cutover** — quality-gated only; OSS bridge remains live until parity is
   independently verified.

## Cost Model

Any upfront investment is acceptable. Engineering headcount can be hired or contracted as needed.
Timeline is the only constraint. OSS bridges cap short-term burn while bespoke compounds.

## Sequencing Principle

Tier N+1 never blocks on Tier N completion; tiers can begin in parallel. Tier 4 (kernel + OS) is
appropriately deferred because Tiers 1-3 build the engineering capacity and Rust-substrate
familiarity that kernel work requires. Beginning Tier 4 before Tier 3 substrate maturity would
be premature optimization.

## Existential Framing

oyatie's destination is **full bespoke kernel + OS + cloud platform**. OSS adoptions are bridges
that get us to revenue and tenant validation while we build the canonical substrate. Every ADR
that adopts an OSS component is a bridge ADR — it includes an explicit superseded-by pointer to
the bespoke replacement ADR when that ADR exists, or a `bespoke_replacement_planned: true`
field when it does not yet exist.

## Consequences

- All future component ADRs must identify their tier placement and bridge strategy.
- Bridge ADRs (OSS adoptions) must carry `bespoke_replacement_planned: true` in frontmatter.
- Tier 4 work (oya-os, oya-kernel) is governed by ADR-0483.
- No OSS runtime bridge is retired without the corresponding bespoke component passing its parity
  gate, unless a later Accepted ADR explicitly rejects that runtime and defines a narrower import
  transition, as ADR-0394 does for Backstage.

## Amendment (2026-08-01, ADR-0394 first-party portal)

ADR-0394 amends this roadmap's generic portal bridge. The portal is first-party Rust only.
Backstage may be consulted as a feature reference or consumed through a bounded, provenance-bearing,
one-way catalog import. It is not operated in parallel, and it is not a runtime dependency, plugin
host, catalog authority, deployment substrate, or extension point. The importer is transition
tooling with explicit validation and deletion criteria, not a supported product bridge.

## Amendment (2026-05-28, kubers active development)

**Tier mapping correction**: My original Tier-4 (kernel+OS at 60+ months) was wrong. kubers (`/Users/jasonlee/Developer/kubers`) is being actively built — bespoke Rust translation of Kubernetes + Talos userspace per ADR-0484 + memory `kubers-canonical-substrate`.

**Corrected Tier mapping**:

| Tier | Original framing | Corrected reality |
|---|---|---|
| Tier 1 (NOW → 12 mo) | source-side bespoke µservices only | source-side bespokes + **kubers Phase A (Rust userspace substrate, active)** |
| Tier 2 (12 → 24 mo) | source-side mid-term | source-side mid-term + **kubers conformance + benchmark gates** (10 gates per HYPERSCALER_READINESS) |
| Tier 3 (24 → 60 mo) | source-side substrate-level | source-side substrate-level + **kubers Phase B Rust hardware kernel (proof-gated)** |
| Tier 4 (60+ mo) | kernel+OS speculation | RETRACTED — kubers + Phase B kernel cover this; no separate Tier-4 needed |

**Bridge discipline reaffirmed**:
- kubers Phase A: bespoke Rust userspace replacing Talos userspace; Linux as bootstrap/reference kernel
- kubers Phase B: Linux-compatible Rust hardware kernel — admissible ONLY after Phase A conformance + measured Linux-config/Rust-for-Linux/eBPF/LSM exhaustion (per kubers/docs/TALOS_RUST_APPROACH.md)

**HYPERSCALER_READINESS gate #8** explicitly names `/Users/jasonlee/Developer/source` adoption as required integration target — oyatie IS the explicit kubers customer/integration product.

**Source-side ADRs** that assumed upstream Kubernetes + containerd + Talos-userspace as eternal: amend to note kubers as canonical destination (in-flight). Particularly: ADR-0378 (Talos), ADR-0381 (BuildKit/containerd), ADR-0411 (Crossplane), ADR-0148/0433 (Cilium), all operator ADRs.

See ADR-0484 for full kubers anchor.

## Amendment (2026-05-28, oya-authn-device Tier-3 + ADR-0508)

`oya-authn-device` added to the Tier-3 table above. This is the oyatie-branded hardware
security key destination — bespoke Rust firmware fork of OpenSK, eventually targeting OpenTitan
SoC for full open-silicon ownership. Bridge = OpenSK vendored at `tools/opensk-vendored/`.

Closed-loop identity family (all three tiers required for full stack ownership):
- **oya-webauthn** (Tier-2, ADR-0507) — RP/server side: validates authenticator assertions
- **oya-authn-device** (Tier-3, ADR-0508) — authenticator/hardware side: generates + signs assertions
- **oya-crypto** (Tier-4, ADR-0506) — crypto primitive layer underpinning both RP and authenticator

See ADR-0508 for full parity table, phasing, and silicon ownership roadmap.

## Amendment (2026-06-08, WAVE-1 Agentic Delivery Fabric convergence)

Amended in place (no tombstone; git history preserves the pre-amendment body):

- **ADR-0520** inserts the Agentic Delivery Fabric (ADR-0516) + the owned AST substrate (ADR-0517) as
  the TOP layer above this tiered bespoke-component roadmap, and reaffirms — does not replace — this
  ADR's bridge-discipline (parallel-run, per-feature parity gates, quality-gated cutover, no
  hard-deadline cutover).
- **ADR-0521** sequences this roadmap as the staged W0–W6 fabric roadmap (convergence-first,
  interface-locking, cutover-gated).

The phased tiers, bridge discipline, and existential framing below are unchanged; the fabric sits
above them as the apex destination.
