---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-007-valkey-adapter
status: pending
execution_unit: ChangeSet
owner: axis-forms
acceptance_lanes: [cargo-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: Valkey adapter (RESP3 wire-compatible) (rate-limit + session)

## Intent

Per-IP / per-form / per-tenant rate-limit token-bucket; submitter session state (≤ 30min TTL); captcha-token cache.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/adapter/valkey/client.rs` | create |
| `microservices/forms/src/adapter/valkey/rate_limit.rs` | create |
| `microservices/forms/src/adapter/valkey/session.rs` | create |
| `microservices/forms/src/adapter/valkey/captcha_cache.rs` | create |
| `microservices/forms/tests/valkey_rate_limit.rs` | create |

## Acceptance Gates

- Per-IP rate-limit verified under burst.
- Session TTL respected.
- Sentinel HA failover ≤ 30s.

## References

- Valkey Sentinel docs.
- PRD FR-08 and performance budgets for submission latency.
- `microservices/forms/manifest.json` Layer-A Valkey substrate entry.
- `microservices/forms/policy/public-read.cedar` and `tenant-scope.cedar`.
- `microservices/forms/slos/submission-latency.openslo.yaml`.
- `microservices/forms/runbooks/spam-flood-throttle.md`.
- `microservices/forms/runbooks/captcha-degraded.md`.

## Foundation A-G Substance

- A. Product scope: Valkey protects public forms from spam bursts and preserves short-lived submitter state without becoming source of truth.
- B. Domain model: rate-limit decisions are represented as `RateLimitDecision`, `RateLimitBucket`, and `SessionHandle` values.
- C. Contracts: REST returns stable 429/503 response shapes with retry hints and no provider secrets.
- D. Policy: anonymous public-read paths still carry tenant, form, pack, and bot-score context into Cedar before cache acceptance.
- E. Operations: Sentinel failover, cache cold start, and captcha-provider outage are covered by runbooks and fail closed for high-risk flows.
- F. Observability: publish bucket saturation, captcha-token-cache misses, session TTL expirations, and failover duration.
- G. Promotion: burst test, TTL test, failover drill, public-read Cedar check, and submission SLO check must pass.

## Counterpart Benchmark

- Counterpart: Slack workflow form intake rate limiting, HubSpot Forms anti-spam throttles, and Twilio Verify-style token expiry behavior.
- Defensible parity claim: Oyatie must preserve response integrity under burst traffic rather than only hiding spam after capture.
- Differentiator: pack-aware captcha cache and Cedar context prevent one tenant's public form from weakening another tenant's posture.
- Grep counterpart names: Slack workflow form intake; HubSpot Forms; Twilio Verify.

## Remediation Notes

- Expanded the Valkey adapter from a small cache stub into an operational foundation plan.
- Added A-G substance tied to manifest, policies, SLO, and runbooks.
- Added counterpart names to support grep-based parity checks.

## Verification Evidence Required

- Burst corpus proves per-IP, per-form, and per-tenant buckets reject excess submit attempts.
- TTL probe proves submitter session and captcha tokens expire at configured limits.
- Sentinel failover drill records ≤ 30s recovery without accepting unverifiable anonymous submissions.
- Public-read Cedar probe proves cache acceptance still carries tenant and pack context.
- Submission SLO evidence shows cache checks do not break the 150ms p95 budget.
- Runbook drill links spam-flood throttling and captcha degradation to the same cache controls.
- Dashboard evidence records rate-limit saturation separately from captcha-provider failures.
- Replay evidence proves cache loss is regenerable and never treated as canonical response state.

## Next IP

[`IP-008-meilisearch-adapter.md`](IP-008-meilisearch-adapter.md)
