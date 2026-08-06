---
id: ADR-0374
status: Superseded
planning_impact: true
deciders: council-architecture, ops-platform
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
related: [ADR-0363, ADR-0366, ADR-0367, ADR-0349, ADR-0361, ADR-0124, ADR-0039, ADR-0043, ADR-0131]
related_specs: [/infra/branch-protection/dev.json, /infra/ci/jenkins/reported-status-contexts.json]
milestone: M-AGENTIC-PIPELINE
depends_on: [ADR-0363]
door: two-way
affected_surfaces:
  crates: [oya-ci-webhook-gateway-app]
  microservices: [ci-webhook-gateway]
  specs: []
deliverables:
  - id: ADR-0374-D1
    description: "GitHub webhook receiver as a flat single-concern Rust microservice (ci-webhook-gateway) with src/ root, blessed deps only (Tokio/Axum/Tower/Hyper/serde/tracing + RustCrypto sha2, with RFC 2104 HMAC and constant-time compare implemented in-module), exposing POST /webhook/github + GET /healthz."
    exit_criteria: "cargo build + cargo fmt --check + cargo clippy --all-targets -D warnings + cargo test are green for the crate; the service tree satisfies the design/spec maturity surfaces (proto3 deferred as N/A)."
    verified_by: "oya gate validate design-spec-maturity-claims"
  - id: ADR-0374-D2
    description: "HMAC-SHA256 webhook-signature verification that fails closed on the RAW body BEFORE any parse/route, constant-time, with the secret redacted in Debug and read only from sref://openbao/oya/ci/github-webhook-secret."
    exit_criteria: "a delivery with a missing/invalid signature is rejected (HTTP 401) and never dispatches; a known-answer HMAC vector verifies; the secret never appears in logs."
    verified_by: "cargo test -p oya-ci-webhook-gateway-app"
  - id: ADR-0374-D3
    description: "PR-event parsing (opened/reopened/synchronized against the gated branch dev) + a closed router table where unknown (event, action) is a typed UnroutableEvent, and the event->pipeline dispatch trait (admission -> oya gate run-all via the Jenkins oyaCiLane trusted runner)."
    exit_criteria: "a valid pull_request:opened against dev kicks the Jenkins lane (HTTP 202); wrong-base/draft is ignored (HTTP 200); unknown event is 422; missing Jenkins URL is a typed transport error (HTTP 502), never a silent success."
    verified_by: "cargo test -p oya-ci-webhook-gateway-app"
  - id: ADR-0374-D4
    description: "Honest typed boundaries for the not-yet-built downstream stages (Intelligence-service reviewer gate per ADR-0367 D2; merge-queue per ADR-0111) via the GatewayError::Unimplemented variant (HTTP 501), each recorded in registry/placeholder-debt/adr-follow-ups.yaml — no lying stub."
    exit_criteria: "the placeholder-debt + honest-claims gates pass; each Unimplemented boundary names its stage and its placeholder-debt token."
    verified_by: "oya gate validate honest-claims"
purpose: >
  Define the CI webhook gateway — the missing trigger that turns a GitHub
  pull_request event into a real, gated CI run on the ADR-0363 substrate
  (git + Jenkins + GitHub (interim)) — so PRs against dev are gated by REAL
  automated checks (Jenkins posts the required GitHub commit statuses) and the
  manual enforce_admins-toggle admin-relax-merge seam is retired. Scaffolds a
  flat single-concern Rust microservice with fail-closed HMAC verification, PR-
  event parsing, and the event->pipeline dispatch trait, with honest typed
  boundaries for downstream stages not yet built.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0374: CI webhook gateway (GitHub → Jenkins gated pipeline trigger)

## Status

Accepted — 2026-05-26.

## Context

The change-coordination substrate is **git + Jenkins + GitHub (interim)**
(ADR-0363), with the agentic pipeline (ADR-0366) and the trustless pre-merge
verification gateway (ADR-0367) layered on top. The pieces that exist today:

- `infra/branch-protection/dev.json` requires **15 status contexts** on `dev`
  (plus `required_signatures: true`, `required_linear_history`, no force-push).
- `infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy` already **POSTs 14**
  of those contexts to the **GitHub Commit Status API**, and the reviewer
  agent posts the 15th (`oya-pr-review`).
- `infra/gitops/vcs-substrate.yaml` stands up GitHub (interim) (GPLv3+, OSI-clean) with native
  branch protection, required status checks, webhooks, and auto-merge.

**The missing piece is the trigger.** Nothing converts a GitHub `pull_request`
event into a Jenkins run. So, per the root `CLAUDE.md` Wave-B bootstrap note and the VCS substrate
(`infra/gitops/vcs-substrate.yaml`), every merge to `dev` historically disabled
`enforce_admins` and used an admin-merge — because the 15 required checks were
never actually produced for a given PR head. That manual relax-merge seam is the
thing this ADR retires by building the trigger.

This is the **GitHub-substrate successor** to the retired ADR-0112
GitHub/foundry webhook-receiver (superseded by ADR-0363). The "foundry" name is
eradicated repo-wide (ADR-0362); this service is named for its single concern.

## Decision

Build a **flat single-concern Rust microservice**, `ci-webhook-gateway`
(`microservices/ci-webhook-gateway/`, `src/` root per ADR-0131; package
`oya-ci-webhook-gateway-app`), that is the FIRST hop of the gated pipeline:

1. **Receive** GitHub webhook deliveries at `POST /webhook/github` (axum/Tokio/
   Tower/Hyper — blessed runtime deps).
2. **Verify** the `X-Hub-Signature-256` (or legacy `X-Gitea-Signature`)
   HMAC-SHA256 on the RAW body, **fail-closed, constant-time** (RustCrypto
   `sha2`; RFC 2104 HMAC and `ct_eq_bytes` constant-time compare live in-module), BEFORE any parse/route/dispatch. The secret is read
   only from `OYA_GITHUB_WEBHOOK_SECRET`, injected from
   `sref://openbao/oya/ci/github-webhook-secret` (ADR-0043), and is redacted in
   `Debug`.
3. **Parse + route** `pull_request` events (opened/reopened/synchronized) whose
   base is the gated branch (`dev`). The router is a **closed** mapping; an
   unknown `(event, action)` is a typed `UnroutableEvent` (logged + 422, never
   silently dropped). Wrong-base / draft / non-gated actions are `Ignored`
   (HTTP 200, no GitHub redelivery storm).
4. **Dispatch** the gated pipeline by kicking the Jenkins `oyaCiLane` lane
   (admission → `oya gate run-all` — the **trusted-runner** re-execution per
   ADR-0367 that posts the GitHub commit statuses). The gateway never trusts
   author-reported evidence; it only KICKS the trusted runner.

### Honest boundaries (no lying stub)

The downstream stages **not yet stood up in the substrate** are expressed as the
typed `GatewayError::Unimplemented` variant (HTTP 501), each naming its stage and
a `registry/placeholder-debt/adr-follow-ups.yaml` token:

- the adversarial **reviewer gate** (Intelligence-service CI stage, ADR-0367 D2)
  — `adr-0374-reviewer-gate-dispatch`.
- the speculative **merge-queue** (ADR-0111, parked per ADR-0363 §3) —
  `adr-0374-merge-queue-admit`.
- an explicit **delivery-dedup log** (ADR-0112 idempotency carried forward) —
  `adr-0374-delivery-dedup-log`. v1 relies on GitHub at-least-once redelivery +
  the idempotent `(pr, head_sha)` kick.

### Commit signing (separate, human-provisioned)

`dev`'s `required_signatures: true` is satisfied by REAL signed commits, NOT by
this gateway. Enabling commit signing (Ed25519 per the ADR-0039 signed-commits
discipline) is a human-provisioning step documented in
`microservices/ci-webhook-gateway/SETUP-RUNBOOK.md`; it removes the second half
of the relax-merge hack (the gateway removes the required-checks half).

## Rejected alternatives

- **Revive the ADR-0112 GitHub/foundry webhook-receiver** — rejected: superseded
  by ADR-0363; substrate is GitHub (interim), and "foundry" is eradicated.
- **Trigger Jenkins via polling / cron** — rejected: ADR-0124's webhook-driven,
  no-cron principle; polling wastes cycles and adds latency.
- **Let the gateway run the gates + post statuses itself** — rejected: violates
  ADR-0367's trusted-runner separation (the producer must not certify its own
  work). Jenkins re-executes hermetically and signs; the gateway only kicks.
- **Stub the reviewer gate / merge-queue as always-pass** — rejected: a lying
  stub. They are typed `Unimplemented` boundaries tracked in placeholder-debt.
- **Build the merge-queue now** — rejected per ADR-0363 §3 (auto-merge +
  required checks suffice at current scale; adopt, don't build, later).

## Consequences

### Positive

- PRs against `dev` are gated by REAL Jenkins-produced checks; the manual
  `enforce_admins`-toggle admin-relax-merge seam can be retired.
- Webhook-spoofing is structurally blocked (fail-closed constant-time HMAC).
- The gateway holds zero durable state — crash-restart-replay safe (GitHub
  at-least-once + idempotent kick).
- Honest boundaries keep the honest-claims gate green while the reviewer gate +
  merge-queue are still being built.

### Negative / risk

- A new hosted endpoint to operate (one small stateless pod). If it is down,
  PRs stop being auto-gated — the runbook's hard rule is "fix the gateway, do
  NOT revert to admin-merge."
- No merge-queue → at high concurrency, semantic conflicts between concurrently
  merged PRs are not caught; accepted at current scale (ADR-0363 §3), revisit
  via `adr-0374-merge-queue-admit`.
- The Jenkins kick uses a minimal HTTP/1.1 POST (plain HTTP on the mesh; TLS at
  ingress) — deliberately not a full HTTP client, to stay on blessed deps.

### Operational

- Runbook: `microservices/ci-webhook-gateway/runbooks/on-call.md`.
- Setup: `microservices/ci-webhook-gateway/SETUP-RUNBOOK.md` (webhook
  registration, HMAC secret, commit-signing, deploy).
- SLOs: `microservices/ci-webhook-gateway/slos/ci-webhook-gateway.openslo.yaml`.

## Orchestrator authority — RESOLVED 2026-05-26 (founder decision: Jenkins-as-orchestrator)

**Orchestrator authority: Jenkins-as-orchestrator vs the Intelligence-service
as a dogfood orchestrator.** This ADR builds the *trigger* and the *dispatch
port* but deliberately does NOT settle WHO owns multi-stage orchestration once
past the first kick:

- **Jenkins-as-orchestrator**: Jenkins pipeline stages sequence admission →
  gates → reviewer → merge (a Jenkinsfile DAG). Simplest; rides ADR-0361's
  Jenkins-native posture; one system owns CI ordering. But it puts agentic
  reviewer logic inside a Groovy pipeline, and couples orchestration to the CI
  engine.
- **Intelligence-service-as-dogfood-orchestrator**: the Intelligence
  microservice (which already absorbs the AI-agent platform per ADR-0363, and
  is the reviewer per ADR-0367 D2) owns orchestration as its first self-tenant
  dogfood job; Jenkins is "just" the trusted runner it invokes. Aligns with the
  self-governing-platform north star (ADR-0368) and dogfood tenancy. But it adds
  a control-plane dependency on Intelligence for every merge, and risks the
  layering concern (an orchestrator that depends on the service it orchestrates).

**Decision (2026-05-26, founder): Jenkins-as-orchestrator.** Jenkins sequences
admission → gates → reviewer → merge, riding ADR-0361's Jenkins-native posture —
the fastest path to retiring the admin-relax-merge seam. The gateway's
`PipelineDispatcher` trait stays intentionally agnostic, so migrating
orchestration into the Intelligence dogfood (the ADR-0368 north star) remains a
cheap, reversible follow-up that does not touch the receiver — recorded as a
future evolution, not a commitment here.

## Verification

- `cargo build` / `cargo fmt --check` / `cargo clippy --all-targets -- -D
  warnings` / `cargo test` green for `oya-ci-webhook-gateway-app`.
- `oya gate validate design-spec-maturity-claims` green (proto3 deferred as N/A
  in `registry/design-spec-maturity/wave-3-i-deferred-surfaces.tsv`).
- `oya gate validate honest-claims` + `placeholder-debt` green (the three
  deferred downstreams are tracked, not faked).

## References

- ADR-0363 (substrate: git + Jenkins + GitHub (interim); supersedes the
  ADR-0112 webhook-receiver), ADR-0366 (self-enforcing pipeline), ADR-0367
  (trustless pre-merge gateway — trusted runner + adversarial reviewer),
  ADR-0349/0361 (Jenkins-native CI farm), ADR-0124 (webhook-driven, no-cron),
  ADR-0039 (signed commits), ADR-0043 (OpenBao secrets), ADR-0131 (flat
  microservice layout).
- `infra/branch-protection/dev.json`, `infra/gitops/vcs-substrate.yaml`
  (VCS substrate; infra/forge consolidated into infra/gitops per ADR-0515 D3).
