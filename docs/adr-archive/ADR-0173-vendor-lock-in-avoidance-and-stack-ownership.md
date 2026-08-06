---
id: ADR-0173
status: Superseded
superseded_by: [ADR-0709]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0173 — Vendor lock-in avoidance and stack ownership

| Field | Value |
| --- | --- |
| Status | Accepted |
| Date | 2026-05-18 |
| Deciders | axis-governance, axis-foundry, axis-cloud-k8s, axis-cloud-secrets, axis-observability |
| Supersedes | — |
| Superseded by | — |
| Related | ADR-0013 (product license policy), ADR-0014 (build-vs-buy policy), ADR-0020 (foundry multi-provider adapter model), ADR-0026 (in-house AI model substrate roadmap), ADR-0042 (observability stack OTel + in-house UI), ADR-0043 (secrets management OpenBao + HSM), ADR-0064 (canonical-base + localization pattern), ADR-0083 (kernel-tier invariants / port-in-kernel), ADR-0092 (workspace dependency seam policy), ADR-0104 (ecosystem expansion toolchain and adapters), ADR-0105 (13-layer enum, Amendment 3 backend-qualified adapter naming), ADR-0142 (CRDT portability trait), ADR-0146 (container base image distroless nonroot), ADR-0147 (container sandboxing runtime ladder), ADR-0148 (service mesh Cilium) |

## Context

User directive 2026-05-18 (verbatim):

> Vendor lockin should be surfaced and avoided if there isn't significant
> benefit. be mindful of owning our stack later and phasing out external
> dependencies eventually. defer OK for vendor migration BUT with seams
> in place to avoid migration debt.

Oyatie is being constructed as a hyperscaler-grade
private-cloud-and-SaaS platform that intends to eventually own its
stack end-to-end. Today, several external vendors are unavoidable —
Anthropic / OpenAI / Google Gemini for foundry LLM intake (per
ADR-0020), GitHub Actions for CI (per ADR-0050 automation-first
pipeline), the AWS S3 protocol for object storage (used as an open
contract, not as an AWS-specific lock-in), and a long tail of OSS
dependencies whose stewards are commercial entities (Cedar from AWS,
gVisor from Google, Cloud Hypervisor from Linux Foundation, etc.).

The user explicitly forbids two failure modes:

1. **Silent vendor accumulation** — adopting a vendor without explicit
   ADR + phase-out plan + replacement target, allowing migration debt
   to compound until the platform is permanently captured.
2. **Vendor-first defaults** — preferring a managed vendor service
   (LaunchDarkly, Datadog, Snyk, AWS Lambda, Azure Cognitive Services,
   GCP Vertex AI, etc.) when an OSS equivalent could be self-hosted.

Both failure modes are already partially mitigated by adjacent ADRs:

- ADR-0014 (build-vs-buy policy) records the decision framework for
  individual adoptions.
- ADR-0020 (foundry multi-provider adapter model) requires every LLM
  vendor be wrapped behind a provider-neutral adapter trait.
- ADR-0026 (in-house AI model substrate roadmap) commits the platform
  to building its own foundation models eventually.
- ADR-0064 (canonical-base + localization) prohibits vendor-specific
  flavor leaking into the canonical kernel.
- ADR-0083 (kernel-tier invariants) requires port-in-kernel + adapter
  separation, which is the structural shape of a vendor seam.
- ADR-0092 (workspace dependency seam policy) governs Rust-level crate
  dependencies against vendor lock.
- ADR-0105 Amendment 3 mandates backend-qualified adapter naming
  (`adapter-anthropic-api`, `adapter-anthropic-subscription`, etc.) so
  every adapter declares which vendor / which mode it serves.
- ADR-0142 (CRDT portability trait) is the worked example for how a
  bus-factor-concerning OSS dependency (Loro) is wrapped in a
  portability trait so it can be swapped.

What is missing is a single point of doctrine that **(a)** declares
the default posture (OWN-the-stack), **(b)** classifies every existing
vendor against a tiered taxonomy with explicit replacement paths and
readiness gates, **(c)** enforces the seam-and-multi-impl pattern via
a CI lane, and **(d)** tracks phase-out progress per vendor as a
canonical artifact. This ADR is that doctrine.

## Decision

### Default posture

**OWN-the-stack via OSS substrate with permissive license is the
default.** Vendor adoption requires an ADR-tracked exception that
satisfies all four of:

1. Concrete business or quality benefit that an OSS substrate cannot
   currently match (cite the gap — capability, performance, cost).
2. Explicit phase-out plan (target replacement, readiness gate, owner,
   review cadence).
3. Port-in-kernel adapter trait + at least two registered adapter
   implementations (vendor-A + vendor-B OR vendor-A + working
   in-memory mock for tests).
4. Registry entry in `registry/vendor-lockin-phaseout/index.json` with
   tier classification, adoption date, and phase-out trigger.

A vendor cannot be adopted by accident: any PR that introduces a
Tier II dependency without satisfying (1)..(4) is blocked by the
`cloud-ci/Rust gate packet vendor-lockin-discipline` lane.

### Three-tier vendor classification

#### Tier I — OWNED (long-term)

These are the substrate components we commit to owning indefinitely.
They are either Rust-first crates we author, or OSS projects under a
permissive license (Apache-2.0, MIT, BSD, MPL-2.0) with bus-factor
high enough that downstream maintenance is realistic.

Tier I representative members:

- **Rust toolchain** (rustc, Cargo, rustfmt, clippy) — Apache-2.0 +
  MIT, Rust Foundation steward.
- **Postgres + Citus** — PostgreSQL License + AGPL3 (Citus core under
  PostgreSQL License since 2022); ADR-0045 database tier strategy.
- **Redis or KeyDB or Valkey** — Redis under RSALv2 since 2024 (Tier I
  with asterisk); Valkey is the BSD-3-Clause Linux Foundation fork
  that we treat as the long-term target.
- **Meilisearch** — MIT; ADR-0047 search backend strategy.
- **Tantivy** — MIT; in-process Rust search library for embedded
  search use cases.
- **OpenBao** — MPL-2.0 community fork of HashiCorp Vault; ADR-0043
  secrets management.
- **Kubernetes** — Apache-2.0; ADR-0028 cloud microservice
  architecture; ADR-0121 onprem k8s stack.
- **ArgoCD** — Apache-2.0; ADR-0041 GitOps trunk-based.
- **Cilium** — Apache-2.0; ADR-0148 service mesh Cilium.
- **Cedar (v4.2)** — Apache-2.0, AWS-stewarded, spec stable;
  ADR-0007 cedar authorization. Bus-factor mitigated by spec stability
  and active community engine forks.
- **Loro CRDT** — MIT; bus-factor concern mitigated by the portability
  trait in ADR-0142.
- **Pandoc** — GPL-2.0+ (process boundary only; not linked into
  binaries — invoked as a sidecar / OS process so the GPL does not
  propagate to our products); used by the documentation pipeline.
- **LibreOffice (headless)** — MPL-2.0 (LGPL3 components are
  process-boundary only — same shell-out pattern as Pandoc).
- **ffmpeg** — LGPL-2.1+ at link time; used as a sidecar; used by
  recordings + meet.
- **ImageMagick / libvips** — Apache-2.0 / LGPL-2.1; image processing.
- **gVisor** — Apache-2.0, Google-stewarded; ADR-0147 sandboxing.
- **Kata Containers** — Apache-2.0, CNCF Sandbox project; ADR-0147.
- **Cloud Hypervisor** — Apache-2.0, Linux Foundation, primary VMM per
  ADR-0147 amendment.
- **Grafana stack** (Mimir, Loki, Tempo, Pyroscope, Alloy) — AGPL-3.0
  (process-boundary only — used as separate services accessed via
  HTTP/OTel, never linked into Rust binaries — AGPL does not propagate
  to clients of a network service).
- **Prometheus** — Apache-2.0; metrics; ADR-0042.
- **OpenTelemetry** — Apache-2.0; ADR-0042.
- **MinIO / Garage / SeaweedFS** — AGPL / Apache-2.0 / Apache-2.0;
  S3-protocol-compatible OSS object stores.

#### Tier II — VENDOR-SEAMED (temporary; phase-out planned)

These are external vendors we depend on today but are not committed
to owning indefinitely. Every Tier II adoption MUST satisfy the
seam-and-multi-impl pattern (a port-in-kernel trait + at least two
adapter implementations). The phase-out plan is canonical and
tracked in `registry/vendor-lockin-phaseout/index.json`.

Current Tier II members:

- **Anthropic API** — used in foundry providers; replacement target is
  the in-house foundry-runtime model substrate (ADR-0026); readiness
  gate is `human-eval ≥ 95% on internal benchmark set + tool-use
  parity demonstrated`.
- **Anthropic Subscription (Claude Code / oauth)** — same.
- **OpenAI API** — same.
- **OpenAI Subscription (Codex)** — same.
- **Google Gemini API** — same.
- **Google Gemini Subscription** — same.
- **GitHub Actions** — used as CI runtime; replacement target is
  GitHub (interim) Actions or Woodpecker CI; readiness gate is
  `private-cloud Kubernetes cluster has stable GitHub + runners +
  cosign chain wired and dogfooded on a non-critical lane for ≥
  90 days`.
- **GitHub (code hosting + PR / issue platform)** — used today as the
  canonical VCS host; replacement target is GitHub (interim);
  readiness gate is `Foundry VCS substrate (ADR-0113) reaches
  release-pointer parity with the GitHub workflow`. Seam: the
  agent-coordination surface is the Foundry pipeline (already), not
  raw GitHub APIs.
- **CloudFlare / Fastly** — not currently adopted; pre-classified
  Tier II so adoption requires this ADR's discipline.

#### Tier III — FORBIDDEN

Vendor adoptions in this tier are categorically refused because they
either (a) lock the platform to a single hyperscaler with no portable
substrate, or (b) replace a substrate component that already has a
viable OSS equivalent at Tier I.

Tier III members include:

- **AWS-specific services** — Lambda (use Kubernetes + Knative),
  DynamoDB (use Postgres + Citus), Cognito (use Keycloak), SQS / SNS
  (use Kafka or NATS), API Gateway (use Envoy + Cilium), Secrets
  Manager (use OpenBao), CloudWatch (use Prometheus + Grafana stack),
  EKS Fargate (use Kubernetes on Cloud Hypervisor / Kata / gVisor),
  IAM as identity substrate (use Cedar + OIDC), KMS (use SoftHSM + HSM
  per cell per ADR-0043). S3 *protocol* is allowed because the
  protocol is open and we run OSS implementations; S3 *as a managed
  AWS service* is not relied on.
- **Azure-specific services** — same general principle (Azure
  Functions, Cosmos DB, AAD-as-identity-substrate, etc.).
- **GCP-specific services** — Cloud Run, Spanner, Firestore, Vertex
  AI, etc.
- **Commercial-only proprietary tools where an OSS equivalent exists**
  — LaunchDarkly (use OpenFeature + Flipt; ADR-0159), Snyk (use
  Trivy + Grype; ADR-0039), Datadog (use Grafana stack; ADR-0042),
  PagerDuty as primary (Grafana OnCall is the long-term target,
  PagerDuty acceptable as temporary Tier II under the phase-out
  contract).

### Seam requirement (port-in-kernel pattern)

Every Tier II dependency MUST follow the port-in-kernel pattern from
ADR-0083 + ADR-0105 Amendment 3:

1. A kernel crate (`oya-<bc>-<concern>-<vendor>-<mode>-kernel`)
   defines the port trait — the abstract contract that the rest of
   the system depends on.
2. One or more adapter crates
   (`oya-<bc>-<concern>-<vendor>-<mode>-adapter`) implement the port
   trait against the actual vendor.
3. At least one additional adapter implementation MUST exist so the
   trait is not vendor-shaped by accident. Acceptable forms of the
   second impl:
   - Second real vendor (e.g., `adapter-openai-api` alongside
     `adapter-anthropic-api`).
   - In-memory / in-process mock impl for tests
     (`adapter-*-inmemory`) — must be a real working impl with tests,
     not a `todo!()` / `unimplemented!()` stub.

Worked example today: foundry providers ship six adapter crates
(`adapter-anthropic-api`, `adapter-anthropic-subscription`,
`adapter-openai-api`, `adapter-openai-subscription`,
`adapter-gemini-api`, `adapter-gemini-subscription`) all implementing
the same `ProviderAuthPort` kernel trait — the in-house model adapter
slot is reserved and will land before the Anthropic / OpenAI / Gemini
phase-out date.

### Phase-out tracking

`registry/vendor-lockin-phaseout/index.json` is the canonical
source-of-truth for vendor-independence posture. Schema (see the file
header for the authoritative version):

```json
{
  "name": "anthropic-api",
  "tier": "II",
  "adoption_rationale": "...",
  "replacement_path": "in-house foundry-runtime model substrate (ADR-0026)",
  "replacement_readiness_gate": "human-eval >= 95% on internal benchmark + tool-use parity demonstrated",
  "seam_adapter_trait": "crates/oya-intelligence-adapter-anthropic-api-kernel",
  "seam_adapter_impls": [
    "crates/oya-intelligence-adapter-anthropic-api-adapter",
    "crates/oya-intelligence-adapter-anthropic-subscription-adapter"
  ],
  "phase_out_target_date_or_signal": "signal: foundry-runtime parity demonstrated"
}
```

### CI lane enforcement

The new lane `cloud-ci/Rust gate packet vendor-lockin-discipline` (crate
`oya-check-vendor-lockin-discipline`) enforces:

- Every entry in `registry/vendor-lockin-phaseout/index.json` parses
  cleanly against the schema.
- Every Tier II entry declares a non-empty `replacement_path`,
  `replacement_readiness_gate`, `seam_adapter_trait`, and at least one
  `seam_adapter_impls` member.
- Every Tier II entry whose `seam_adapter_trait` points into the
  workspace has at least one corresponding adapter crate present in
  the workspace member list (the second-impl rule).
- Every Tier III entry has an explicit refusal rationale.
- Every Tier I entry has a steward / license declared.

The lane is wired into `oya-ci-required` and pre-push so vendor
discipline cannot regress silently.

## Alternatives considered

### (a) Universal OSS-only

**Rejected.** foundry providers need access to current
state-of-the-art LLMs for the platform to be useful to its earliest
users. The in-house model substrate (ADR-0026) is on the roadmap but
is not at parity with Anthropic / OpenAI / Gemini today. A pure
OSS-only stance would block product delivery and create a worse
outcome — agents would silently shell out to vendor CLIs anyway
without an enforced seam.

### (b) Vendor-first with no phase-out plan

**Rejected.** The user directive 2026-05-18 explicitly forbids this
mode. Migration debt accumulates exponentially when phase-out is not
planned at adoption time. Industry case studies (Stripe pulling off
Mongo, Linear standing up its own scheduler) repeatedly demonstrate
that the cost-of-exit grows super-linearly with adoption depth, and
the right time to install the exit seam is at adoption time.

### (c) Tiered classification with seams (this ADR)

**Accepted.** Provides:

- A default-deny posture that surfaces every vendor decision.
- An explicit phase-out plan per vendor so debt is bounded.
- A CI lane that mechanically enforces the seam pattern so the
  doctrine cannot rot in prose-only form.
- A registry that is grep-able evidence under audit.

### (d) Per-vendor ADR with no central doctrine

**Considered but subsumed.** Per-vendor ADRs (e.g., ADR-0020 for
multi-LLM, ADR-0142 for Loro) remain — they document the specific
adapter shape. This ADR is the meta-doctrine that requires every
future vendor adoption to follow the same structural pattern.

## Consequences

### Positive

1. Every Tier II vendor adoption is surfaced under audit via the
   phase-out registry — no silent capture.
2. The seam-and-multi-impl rule is mechanically enforced by the new
   `cloud-ci/Rust gate packet vendor-lockin-discipline` lane.
3. Phase-out readiness is a first-class artifact, so the platform
   knows which seam to retire first when in-house substrate matures.
4. Compliance with the platform's stack-ownership goal is auditable
   under industry-standard rubrics (CNCF graduated-project preference
   signal, AWS Well-Architected vendor-lock-in best practices).
5. Future agents and reviewers can answer the question "what does it
   cost to drop vendor X?" by reading one JSON file.

### Negative

1. Every Tier II adoption now requires an adapter trait and a second
   impl, which is non-trivial engineering cost up front. Mitigation:
   the in-memory mock impl pattern satisfies the second-impl rule
   cheaply when no second real vendor exists yet.
2. The vendor inventory must be maintained as the platform grows.
   Mitigation: the CI lane fails closed when the registry drifts.
3. Some OSS dependencies sit in a gray zone (Redis post-RSALv2,
   Loro bus-factor, Cedar AWS-stewarded). The tier classification
   has explicit Tier I-with-asterisk markers to surface the
   risk explicitly rather than hiding it under "OSS = safe".

### Neutral

1. The phase-out registry replaces no existing artifact; it is
   additive. ADR-0014 (build-vs-buy), ADR-0020 (multi-provider
   adapter), and ADR-0092 (workspace dep seam) all remain — this ADR
   is the umbrella that requires them to compose.
2. The `oya-check-vendor-lockin-discipline` crate joins the existing
   check-family per ADR-0105 13-layer enum; no new layer is added.

## Compliance and audit trail

- **AWS Well-Architected Framework — Operational Excellence pillar,
  OPS 4** ("design workloads to be portable across cloud providers"):
  Tier III explicitly forbids cloud-specific-service lock-in.
- **CNCF Graduated-project criterion** ("must avoid single-vendor
  lock-in"): the seam-and-multi-impl rule is the structural shape
  that satisfies this.
- **NIST SP 800-53 SA-12 (Supply Chain Risk Management)**: the
  phase-out registry is the supply-chain-risk artifact for vendor
  dependencies.
- **ISO/IEC 27001 A.15 (Supplier Relationships)**: covered by the
  per-Tier-II `replacement_readiness_gate` field.

## Rollout

| Wave | Scope | Owner | Date |
| --- | --- | --- | --- |
| W0 | Author this ADR + populate `registry/vendor-lockin-phaseout/index.json` with current vendor inventory (30+ entries) | axis-governance | 2026-05-18 |
| W0 | Stand up `oya-check-vendor-lockin-discipline` crate + tests + dev-cli gate dispatch | axis-foundry | 2026-05-18 |
| W0 | Wire gate into `oya-ci-required` aggregator | axis-foundry | 2026-05-18 |
| W1 | Add `External Dependencies` section to each µservice PRD that depends on a Tier II vendor (foundry, observability, cloud-secrets, etc.) | per-µservice owner | 2026-05-25 |
| W2 | First quarterly phase-out review — confirm readiness gates have not stalled | axis-governance | 2026-08-18 |
| W3 | Anthropic API phase-out begins when foundry-runtime achieves the readiness gate | axis-foundry | signal-driven readiness gate |

## References

- ADR-0013 Product license policy
- ADR-0014 Build-vs-buy policy
- ADR-0020 Foundry multi-provider adapter model
- ADR-0026 In-house AI model substrate roadmap
- ADR-0042 Observability stack OTel + in-house UI
- ADR-0043 Secrets management OpenBao + HSM per cell
- ADR-0064 Canonical-base + localization pattern
- ADR-0083 Kernel-tier invariants (port-in-kernel)
- ADR-0092 Workspace dependency seam policy
- ADR-0104 Ecosystem expansion toolchain and adapters
- ADR-0105 13-layer enum + Amendment 3 backend-qualified adapter naming
- ADR-0142 CRDT portability trait (worked example for OSS bus-factor)
- ADR-0146 Container base image distroless nonroot
- ADR-0147 Container sandboxing runtime ladder
- ADR-0148 Service mesh Cilium
- ADR-0159 Feature flag substrate
- AWS Well-Architected Framework — Operational Excellence OPS 4
- CNCF Graduated-project criteria
- NIST SP 800-53 SA-12 (Supply Chain Risk Management)
- ISO/IEC 27001 A.15 (Supplier Relationships)
