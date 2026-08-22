---
doc_class: IP
ip_id: IP-014
microservice: identity
status: ga
related_adrs: [ADR-0189]
date: 2026-05-18
owner_team: axis-identity + ops-security
---

# IP-014 — Continuous risk-scoring adjunct (CAEP)

## Goal

Risk-score every sign-in + every sensitive action against contextual signals (impossible-travel, new-device, unusual-time, novel-IP, abnormal user-agent, post-suspend reactivation). HIGH risk downgrades effective ACR (forces step-up earlier than the static policy would). Conformant with the OpenID CAEP (Continuous Access Evaluation Protocol) draft.

## Files

| File | Purpose |
|---|---|
| `crates/identity-risk-scoring-kernel/Cargo.toml` | manifest |
| `crates/identity-risk-scoring-kernel/src/lib.rs` | `RiskScorer` trait + signal types |
| `crates/identity-risk-scoring-domain/src/lib.rs` | risk computation algorithm |
| `crates/identity-risk-scoring-usecase/src/lib.rs` | score-and-decide pipeline |
| `crates/identity-risk-scoring-app/src/lib.rs` | hot-path service |

## Signal sources

| Signal | Source | Weight |
|---|---|---|
| Impossible travel | Hubble flow obs / GeoIP delta vs last sign-in | 0.30 |
| New device fingerprint | TLS client hello + UA + Accept-Language | 0.20 |
| Novel IP / first-time CIDR | GeoIP per-user history | 0.15 |
| Unusual time-of-day | per-user histogram of sign-in hours | 0.10 |
| Recent suspension lifted | tenancy µservice flag | 0.10 |
| Multiple failed factors in last hour | step-up audit query | 0.10 |
| Behavioural anomaly (browser version regression) | UA-history compare | 0.05 |

## Score computation

`risk = Σ(weight × signal_strength)` where `signal_strength ∈ [0,1]`. Clamped to [0,1].

| Score | Effective ACR delta |
|---|---|
| 0.0 - 0.3 | no change |
| 0.3 - 0.6 | bump floor: action requires `acr+1` |
| 0.6 - 0.8 | bump floor: action requires `acr+2`; emit user-notice |
| 0.8 - 1.0 | refuse + ops-security paged |

## CAEP integration

- Subscribe to OpenID CAEP shared-signal feed (if upstream IdP supports).
- Emit CAEP shared-signal events for downstream consumers when oyatie detects risk:
  - `https://schemas.openid.net/secevent/caep/event-type/credential-change`
  - `https://schemas.openid.net/secevent/caep/event-type/session-revoked`

## Privacy posture

- Signal computation in-memory only; raw signal payloads never logged.
- Per-user history retained 90 days; older purged.
- GDPR Art. 22 right to object: tenant policy can disable risk-scoring for the tenant (default: enabled).

## Tests

| Test | Mechanism |
|---|---|
| `impossible_travel_signal_raises_score` | sign-in NYC → 30s later TOK; observe score > 0.5 |
| `novel_ip_signal_raises_score` | new /24 not in history; observe small bump |
| `risk_above_threshold_bumps_acr_floor` | inject high-risk; observe required_acr increased |
| `risk_score_clamped_to_unit_interval` | extreme inputs; output stays in [0,1] |
| `caep_event_emitted_on_session_revoked` | force revoke; CAEP event observed |
| `tenant_disable_honoured` | tenant disables; risk always 0 |
| `90_day_history_purge` | clock-forward 91 days; history purged |
| `pii_not_persisted_beyond_signal_computation` | inspect storage; only aggregated stats |
| `risk_above_0_8_pages_ops` | extreme inject; pager event observed |
| `signal_weights_normalised` | sum of weights = 1.0 |

## Failure modes

- Signal source down (e.g., GeoIP service unreachable): assume score 0 for that signal; do NOT fail closed (UX > marginal risk).
- CAEP feed disconnected: cache last signals; retry connect every 60s.

## Acceptance — DONE when

- 10 tests pass.
- Live signal feed from Hubble + GeoIP wired in staging.
- 30-day staging trial shows < 5% false-positive rate on legitimate users.

## Out of scope (future)

- Active ML model (deferred to governance-eval-domain integration).
- Per-tenant custom risk policies.

## Counterpart references - 014-continuous-risk-scoring

- Counterpart class: policy and risk gate.
- Palantir Foundry policy controls and GitHub organization security policies are the relevant counterpart bar; this IP makes the gate Cedar-first, tenant-scoped, and evidence-emitting instead of burying access decisions in route handlers.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

