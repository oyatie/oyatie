---
id: ADR-0387
status: Superseded
planning_impact: true
deciders: founder, council-architecture
date: 2026-05-28
owner: council-architecture + ops-platform
supersedes: []
superseded_by: []
related: [ADR-0112, ADR-0359, ADR-0361, ADR-0363, ADR-0374]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
door: two-way
milestone: M-CI-WEBHOOK-GATEWAY
deliverables:
  - id: D1
    description: "HTTP receiver endpoint POST /webhook/github accepting GitHub's webhook payload. Verifies the X-Hub-Signature-256 HMAC header on the RAW body, fail-closed, BEFORE any parsing. Returns 401 on missing/mismatched signature, 200 on accepted events, 422 on unroutable authentic events (no GitHub redelivery storm)."
    exit_criteria: "axum handler at POST /webhook/github compiles; cargo nextest -p oya-ci-webhook-gateway-kernel passes d1_payload_parsing tests (push, PR-open, PR-update, PR-close, ping shapes all parsed without panic)."
    verified_by: "cargo nextest -p oya-ci-webhook-gateway-kernel::d1_payload_parsing"
  - id: D2
    description: "ed25519 signature verification using the shared secret fetched from OpenBao (sref://openbao/oya/ci/github-ed25519-secret). The WebhookSignature value object wraps the raw bytes; SignatureVerifier trait seam allows a MockSignatureVerifier in tests. Fail-closed: missing header → MissingSignature, tampered payload → SignatureMismatch, expired timestamp window → ExpiredTimestamp."
    exit_criteria: "SignatureVerifier trait compiles; d2_ed25519_verification tests cover valid signature, tampered payload, expired timestamp, and missing header — all 4 tests FAIL at Stage-4 RED (no real ed25519 implementation yet) and PASS at Stage-5 GREEN."
    verified_by: "cargo nextest -p oya-ci-webhook-gateway-kernel::d2_ed25519_verification (RED: 4 fail)"
  - id: D3
    description: "Payload normalization to a canonical CiTriggerEvent (repo, branch, head_sha, base_sha, pr_number). Every GitHub webhook shape (push, pull_request open/update/close, ping) maps to either a CiTriggerEvent or an explicit Ignored outcome. The closed router table: unknown (event, action) pairs produce UnroutableEvent, not a silent drop."
    exit_criteria: "CiTriggerEvent struct compiles with all required fields; d3_event_normalization tests assert every supported GitHub payload type normalizes to a CiTriggerEvent with correct field values."
    verified_by: "cargo nextest -p oya-ci-webhook-gateway-kernel::d3_event_normalization"
  - id: D4
    description: "Jenkins client (REST API) that triggers a parameterized build of the oyaCiLane job with the CiTriggerEvent as parameters. JenkinsClient trait seam; Stage-5 implements the reqwest-backed adapter. Parameters: repo_full_name, branch, head_sha, base_sha, pr_number, delivery_id."
    exit_criteria: "JenkinsClient trait compiles; JenkinsJob value object carries job name + parameters + build number + JobStatus enum."
    verified_by: "cargo check -p oya-ci-webhook-gateway-kernel --tests"
  - id: D5
    description: "Jenkins-result consumer: on oyaCiLane job completion, post the 5 required commit-status contexts (cargo-fmt, cargo-check, cargo-clippy, cargo-nextest, oya-pr-review) to GitHub via gh api repos/<owner>/<repo>/statuses/<sha>. CommitStatusPoster trait seam; GitHubStatusRequest struct carries all fields required by the GitHub statuses API. Stage-5 implements the reqwest-backed adapter."
    exit_criteria: "CommitStatusPoster trait + CommitStatusContext enum + GitHubStatusRequest struct all compile; d5_commit_status_post tests assert all 5 contexts produce the correct formatted request body."
    verified_by: "cargo nextest -p oya-ci-webhook-gateway-kernel::d5_commit_status_post"
  - id: D6
    description: "Per-tenant Cedar policy (admin/operator realm). Only authorized webhook sources can trigger jobs. Policy file at microservices/ci-webhook-gateway/policy/ci-webhook-gateway.cedar. Kernel exposes AuthzRequest + WebhookAuthzGate trait seam; Cedar adapter in a follow-up PR implements the policy evaluation. Dogfood doctrine: oyatie-dogfood tenant goes through the same authorization path as all tenants — no internal bypass."
    exit_criteria: "WebhookAuthzGate trait compiles; policy file skeleton present at microservices/ci-webhook-gateway/policy/ci-webhook-gateway.cedar."
    verified_by: "cargo check -p oya-ci-webhook-gateway-kernel --tests + ./bin/oya gate validate honest-claims"
---

# ADR-0387 — CI Webhook Gateway: GitHub → Jenkins → GitHub Commit-Status Bridge

## Status

Proposed

## Context

`dev` branch protection requires 15 status contexts. Jenkins already posts 14
of them to the GitHub Commit Status API via `oyaCiLane.groovy`, but nothing
TRIGGERS Jenkins from a GitHub PR event. The result: every merge historically
required a founder-OK admin-relax-merge that briefly disabled `enforce_admins`.

This is the missing trigger. It eliminates the admin-merge bridge by:

1. Receiving GitHub webhook deliveries at `POST /webhook/github`.
2. Verifying the ed25519 signature (shared secret in OpenBao) — fail-closed
   before any JSON parsing, per ADR-0112 §"Signature handling".
3. Normalizing the payload to a canonical `CiTriggerEvent`.
4. Triggering the Jenkins `oyaCiLane` parameterized job.
5. On job completion, posting the 5 required commit-status contexts to GitHub
   via `gh api repos/<owner>/<repo>/statuses/<sha>`.

**Binding ADRs**:

- **ADR-0112**: webhook-driven Foundry/Intelligence agent invocation — the
  original design that establishes the GitHub-webhook-to-pipeline pattern,
  ed25519 signature mandate, and fail-closed security invariants.
- **ADR-0359**: Jenkins-native CI execution model — `oyaCiLane.groovy` is the
  trusted runner that executes `./bin/oya verify --ci-required` and
  `cargo nextest` on Talos.
- **ADR-0361**: Jenkins substrate configuration — the parameterized job schema,
  build parameter names, and the callback mechanism this gateway relies on.
- **ADR-0363**: GitHub as the self-hosted git substrate — webhook delivery
  format, signature headers (`X-Hub-Signature-256` / `X-GitHub-Delivery`),
  and the mirroring arrangement from GitHub.

## Decision

Build `microservices/ci-webhook-gateway/crates/oya-ci-webhook-gateway-kernel`
as the pure-Rust kernel crate (no I/O, no async, no HTTP) containing:

- `CiTriggerEvent` — the canonical normalized event.
- `WebhookSignature` — value object wrapping raw signature bytes.
- `JenkinsJob` — value object (job name, parameters, build number, status).
- `CommitStatusContext` — the 5 required GitHub status contexts.
- `SignatureVerifier` trait — ed25519 verification seam (OpenBao-backed in Stage-5).
- `JenkinsClient` trait — job-trigger seam (reqwest-backed in Stage-5).
- `CommitStatusPoster` trait — GitHub statuses API seam (reqwest-backed in Stage-5).
- `WebhookEventSink` trait — audit-chain emission seam (ADR-0193/ADR-0263).
- `WebhookAuthzGate` trait — Cedar policy evaluation seam (D6).

Stage-4 RED ships the kernel + failing tests. Stage-5 GREEN implements the
adapter crates.

## Deliverables

See YAML frontmatter for D1–D6 with exit criteria and verification commands.

## Consequences

### Positive

- Eliminates the founder-OK admin-merge bridge: PRs merge automatically once
  all 5 required contexts post green.
- Fail-closed security: the ed25519 signature is verified on the RAW body
  before any parsing or routing.
- Dogfood doctrine honoured: `oyatie-dogfood` tenant traverses the same Cedar
  authorization path as every external tenant.
- Clean hexagonal layering: kernel has zero I/O deps; adapters are swappable.

### Negative / Risks

- Stage-5 requires a live OpenBao `sref://openbao/oya/ci/github-ed25519-secret`
  pre-provisioned before the gateway can accept real webhook traffic.
- Jenkins `oyaCiLane` callback URL must be configured as a build parameter or
  a webhook-notification plugin trigger (follow-up ADR-0361 amendment).

## Compliance

- ADR-0083 Tier-3: no `unwrap`/`expect`/`panic` on the request path.
- ADR-0131 flat layout: kernel lives at `microservices/ci-webhook-gateway/crates/`.
- ADR-0132 no-suite: single-concern crate; no bundle/suite grouping.
- Dogfood tenancy: `oyatie-dogfood` is a regular tenant; no internal bypass.
- Data residency: only repo-coordination metadata (PR numbers, commit SHAs,
  branch names); no tenant PII in this service.
