---
id: ADR-0482
status: Accepted
planning_impact: true
deciders: founder
date: 2026-05-28
authority: founder
doctrine_meta: true
owner: founder
supersedes: []
superseded_by: []
related: [ADR-0388, ADR-0392, ADR-0476, ADR-0478, ADR-0479, ADR-0480, ADR-0481, ADR-0506, ADR-0507, ADR-0508]
amended_by: [kubers-anchor-2026-05-28]
door: one-way
milestone: M-BESPOKE-ROADMAP
---

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

Adopt a phased bespoke roadmap with explicit timeline tiers and bridge mappings. Every bespoke
component ships with a parallel OSS bridge, quality-gated cutover criteria, and tenant opt-in
granularity. No hard-deadline cutover — quality gates only.

## Phased Roadmap

### Tier 1 — NOW → 12 months (in flight or imminent)

| Bespoke | Supersedes OSS | Bridge during Phase-1 | Cutover criteria |
|---|---|---|---|
| oya-vcs (oya-vcs roadmap plan) | Forgejo (ADR-0363) | Parallel-run; tenant opt-in per repo | Feature parity per-feature gates |
| Rust-native portal (Rust-native portal plan) | Backstage (Backstage bridge plan) | Backstage Catalog YAML spec retained | Same |
| oya-notify (oya-notify plan) | Postal/Haraka alternatives | None — bespoke from day 1 | N/A |
| oya-errors (oya-errors plan) | GlitchTip (GlitchTip bridge plan) | Phase-1 GlitchTip ingest endpoint | Sentry SDK protocol parity |
| oya-status (oya-status plan) | Gatus (Gatus bridge plan) | Gatus runs parallel | OpenSLO/Mimir integration parity |
| oya-identity (ADR-0476) | Keycloak (Phase-1 bridge plan) | Keycloak parallel-run | Feature parity: OIDC+OAuth2.0+WebAuthn+IdP federation+MFA |
| oya-code-search (oya-code-search plan) | Sourcegraph (Sourcegraph bridge plan) | Sourcegraph parallel-run | SCIP indexing + cross-references parity |
| oya-billing (ADR-0478) | Lago (Lago Phase-1 plan) | Lago parallel-run; tenant opt-in | Plans + subscriptions + invoicing + payments parity |
| oya-meter (ADR-0479) | OpenMeter (Phase-1 bridge plan) | OpenMeter parallel-run | Event ingest + meter aggregation parity |
| oya-cost (ADR-0480) | OpenCost Phase-1 plan | OpenCost parallel-run | K8s allocation parity |
| oya-flags (ADR-0481) | flagd (OpenFeature backing plan) | flagd parallel-run | OpenFeature protocol compat parity |

### Tier 2 — 12 → 24 months (mid-term bespoke)

| Bespoke | Supersedes | Bridge | Cutover |
|---|---|---|---|
| oya-oncall | Grafana OnCall (Grafana OnCall bridge plan) | Grafana OnCall parallel | IRM bundle UX parity |
| oya-ml | MLflow (MLflow bridge plan) | MLflow parallel; Python compat shim | Experiment + model registry parity |
| oya-prompt-eval | Promptfoo (Promptfoo bridge plan) | Promptfoo parallel | LLM eval framework parity |
| oya-realtime | Centrifugo (Centrifugo bridge plan) | Centrifugo parallel | WebSocket + presence + multiplexing parity |
| oya-admin | AppSmith (AppSmith bridge plan) | AppSmith parallel | Retool-pattern low-code parity |
| oya-waf | Coraza (Coraza WAF bridge plan) | Coraza CRS rules compat layer | OWASP CRS evaluation parity |
| oya-pipelines | Apache Airflow (Airflow bridge plan) | Airflow parallel; DAG compat shim | DAG orchestration parity |
| oya-webauthn | webauthn-rs (ADR-0507) | webauthn-rs parallel; MPL-2.0 bridge | Parity table green (ADR-0507) + oya-identity Phase-2 promotion gate |

### Tier 3 — 24 → 60 months (substrate-level bespoke)

| Bespoke | Supersedes | Bridge | Cutover |
|---|---|---|---|
| oya-events | Apache Pulsar (Pulsar 4.x + Oxia substrate plan) | Pulsar parallel-run; protocol-compat ingress | Multi-tenant durable event bus parity at FAANG scale |
| oya-search | OpenSearch (OpenSearch bridge plan) | OpenSearch parallel; Quickwit-rs hybrid | Full-text + analytics search parity |
| oya-workflow | Temporal (Temporal workflow bridge plan) | Temporal parallel; SDK shim | Durable execution parity |
| oya-graph | JanusGraph (JanusGraph bridge plan) | JanusGraph parallel; TinkerPop compat | Distributed graph parity |
| oya-wide-column | Cassandra (Cassandra bridge plan) | Cassandra parallel | Wide-column NoSQL parity |
| oya-lakehouse | Iceberg+Lakekeeper (Iceberg/Lakekeeper lakehouse plan) | Iceberg format compat | Table format parity |
| oya-clickhouse | ClickHouse (ADR-0193) | ClickHouse parallel | OLAP query engine parity |
| oya-authn-device (ADR-0508) | OpenSK (Phase-1 reference firmware per ADR-0508) | OpenSK vendored at `tools/opensk-vendored/`; nRF52840 dev dongle for engineers | Tier-3 hardware-readiness + parity-table-green (ADR-0508) + first manufacturing run validated + OpenTitan port verified; cross-ref: oya-webauthn (Tier-2 RP partner, ADR-0507) + oya-crypto (Tier-4 crypto primitive, ADR-0506) |

### Tier 4 — 60+ months (kernel + OS bespoke)

| Bespoke | Supersedes | Bridge | Cutover |
|---|---|---|---|
| oya-os (oya-os plan) | Talos Linux (ADR-0378, ADR-0382) | Talos parallel for years | Linux capability parity (eBPF, containers, K8s) + Rust microkernel migration |
| oya-kernel | Linux kernel | Linux substrate underneath; gradual driver migration | POSIX + Linux ABI compat + Rust-native kernel components |
| oya-hypervisor | KVM/Firecracker/cloud-hypervisor | KVM parallel | TEE + nested virt parity |
| oya-runtime | containerd | containerd parallel; OCI spec compat | Container runtime parity |
| oya-crypto (ADR-0506) | aws-lc-rs (Phase-1 bridge per ADR-0506) | aws-lc-rs indefinitely (FIPS-validatable, drop-in) | kubers Phase-B kernel proofs landed + FIPS 140-3 module validation + hyperscaler-readiness gate #10 |

## Bridge Discipline

Every bespoke tier MUST have:

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
- Tier 4 work (oya-os, oya-kernel) is governed by oya-os plan.
- No OSS bridge is retired without the corresponding bespoke component passing its parity gate.

## Amendment (2026-05-28, kubers active development)

**Tier mapping correction**: My original Tier-4 (kernel+OS at 60+ months) was wrong. kubers (`/Users/jasonlee/Developer/kubers`) is being actively built — bespoke Rust translation of Kubernetes + Talos userspace per kubers anchor plan + memory `kubers-canonical-substrate`.

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

**Source-side ADRs** that assumed upstream Kubernetes + containerd + Talos-userspace as eternal: amend to note kubers as canonical destination (in-flight). Particularly: ADR-0378 (Talos), ADR-0381 (BuildKit/containerd), Crossplane XR plan (Crossplane), ADR-0148/0433 (Cilium), all operator ADRs.

See kubers anchor plan for full kubers anchor.

## Amendment (2026-05-28, oya-authn-device Tier-3 + ADR-0508)

`oya-authn-device` added to the Tier-3 table above. This is the oyatie-branded hardware
security key destination — bespoke Rust firmware fork of OpenSK, eventually targeting OpenTitan
SoC for full open-silicon ownership. Bridge = OpenSK vendored at `tools/opensk-vendored/`.

Closed-loop identity family (all three tiers required for full stack ownership):
- **oya-webauthn** (Tier-2, ADR-0507) — RP/server side: validates authenticator assertions
- **oya-authn-device** (Tier-3, ADR-0508) — authenticator/hardware side: generates + signs assertions
- **oya-crypto** (Tier-4, ADR-0506) — crypto primitive layer underpinning both RP and authenticator

See ADR-0508 for full parity table, phasing, and silicon ownership roadmap.
