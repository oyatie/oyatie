# PRD — CI Webhook Gateway

- Status: drafted (wave-3 scaffold, 2026-05-26)
- Owner: council-architecture + ops-platform
- Primary design ADR: ADR-0374
- Substrate ADRs: ADR-0363 (git + Jenkins + GitHub (interim)), ADR-0366
  (self-enforcing pipeline), ADR-0367 (trustless pre-merge verification)

## Problem

`dev` branch protection (`infra/branch-protection/dev.json`) requires 15 status
contexts before a PR may merge. The Jenkins `oyaCiLane` shared library already
POSTs 14 of those contexts to the GitHub Commit Status API
(`infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy`), and the reviewer
agent posts the 15th (`oya-pr-review`). **But nothing TRIGGERS Jenkins from a
GitHub PR event.** So historically every merge to `dev` briefly disabled
`enforce_admins` and used an admin-merge — the manual relax-merge seam called
out in the root `CLAUDE.md` Wave-B bootstrap note.

This service is the missing trigger: the FIRST hop that converts a GitHub
`pull_request` webhook into a real, gated CI run, so PRs are gated by REAL
automated checks and the admin-relax-merge hack can be retired.

## Goals

1. Receive GitHub webhook deliveries over HTTP at `/webhook/github`.
2. Verify the `X-Hub-Signature-256` HMAC on the raw body, **fail-closed**,
   before any parsing/routing.
3. Parse `pull_request` events (opened / reopened / synchronized) whose base
   branch is the gated target (`dev` by default).
4. Dispatch the gated pipeline by kicking the Jenkins `oyaCiLane` lane
   (admission → `oya gate run-all`, the trusted-runner re-execution per
   ADR-0367 that posts the GitHub commit statuses).
5. Be honest about boundaries: stages not yet built in the substrate (the
   adversarial reviewer gate, the speculative merge-queue) are expressed as a
   typed `Unimplemented` (HTTP 501) and tracked in `registry/placeholder-debt/`.

## Non-goals

- The gateway does NOT itself run the gates or post commit statuses — Jenkins
  (the trusted runner, ADR-0367) does. The gateway only kicks the lane.
- The gateway does NOT implement the adversarial reviewer (Intelligence
  service, ADR-0367 D2) or the merge-queue (ADR-0111). Those are downstream and
  tracked as deferred.
- No `oya git` / `oya vcs` wrapping (retired by ADR-0363). Plain git + GitHub.
- No gRPC/proto3 surface — this is an HTTP webhook receiver, not an
  inter-service API.

## Users

- The **GitHub forge** (the webhook sender).
- **Agent + human contributors** whose PRs against `dev` need real gating.
- **ops-platform** operators who provision the secret + register the webhook.

## Acceptance criteria

- AC-1: A GitHub `pull_request` delivery with a valid HMAC and `base.ref=dev`
  results in a Jenkins pipeline kick and an HTTP 202 response naming the
  furthest kicked stage and the honest boundary.
- AC-2: A delivery with a missing or invalid signature is rejected with HTTP
  401 and NEVER dispatches (verified by `receiver::tests`).
- AC-3: A delivery whose base branch is not the gated target, or which is a
  draft PR, returns HTTP 200 `ignored` (no dispatch, no GitHub redelivery
  storm).
- AC-4: An event class not in the closed router table returns HTTP 422
  `unroutable` (logged, not silently dropped).
- AC-5: When the Jenkins dispatch URL is unset, dispatch returns a typed
  transport error (HTTP 502) — never a silent success.
- AC-6: The webhook secret is read only from `OYA_GITHUB_WEBHOOK_SECRET`
  (injected from `sref://openbao/oya/ci/github-webhook-secret`); it never
  appears in logs (`WebhookSecret` is redacted in `Debug`).
- AC-7: `cargo build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, and `cargo test` are all green for the crate.

## Success metric

Zero admin-relax-merges on `dev` after the gateway + GitHub webhook are
provisioned: every merge rides real, Jenkins-produced, signed status checks.
