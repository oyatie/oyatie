---
doc_class: SDK-PLAN
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: axis-foundry + axis-sdk
related_adrs: [ADR-0136, ADR-0137]
---

# SDK Plan — foundry (consolidated)

## Scope

Cross-BC SDK strategy for foundry. Per-BC SDK plans preserved at
`bc-sources/<bc>/sdk-plan.md`.

## SDK shape

Foundry ships **one tenant-facing SDK family** with per-BC modules:

| Language | M01 | Post-M01 |
|---|---|---|
| Rust | first-party; in-workspace `oya-foundry-<bc>-sdk` crates | — |
| TypeScript | first-party; npm `@oyatie/foundry-{runtime,supervisor,eval,evidence,guardrails,providers}` | — |
| Python | subsequent-to-M01-completion via PyO3 bindings or pure-Python | M03 |
| Go | subsequent-to-M01-completion via openapi-generator | M04 |

## Per-BC SDK surfaces

| BC | Rust crate(s) | TS module(s) | Primary use case |
|---|---|---|---|
| runtime | `oya-foundry-runtime-capability-executor-sdk`, `oya-foundry-runtime-session-state-sdk` | `@oyatie/foundry-runtime` | invoke capability; read session history |
| supervisor | `oya-foundry-supervisor-{agent-fleet-lifecycle,autonomy-policy-enforcement,capability-deployment,kill-switch-circuit-breaker,supervision-event-bus}-sdk` | `@oyatie/foundry-supervisor` | deploy capability; query fleet; engage kill-switch |
| eval | `oya-foundry-eval-eval-runner-sdk` | `@oyatie/foundry-eval` | submit eval run; query results |
| evidence | `oya-foundry-evidence-sdk` (single combined crate per `bc-sources/evidence/PRD.md`) | `@oyatie/foundry-evidence` | query evidence; request regulator export |
| guardrails | (in-process; no public SDK in M01 — guardrails called inline by runtime) | — | — |
| providers | `oya-foundry-providers-router-sdk` | `@oyatie/foundry-providers` | ops-portal admin (provider config, credential pin) |

## Cross-BC concerns

- **Auth**: All SDKs accept `OyaCredential` (mTLS cert + tenant token);
  per-call `tenant_id` scope check.
- **Tracing**: All SDKs propagate OpenTelemetry trace IDs end-to-end.
- **Errors**: Uniform `OyaError` family across BCs with BC-specific variants.
- **Versioning**: Per-BC SemVer; foundry SDK metapackage pins one cross-BC
  vetted set per release.

## Per-BC SDK plan archives

- `bc-sources/runtime/sdk-plan.md`
- `bc-sources/supervisor/sdk-plan.md`
- `bc-sources/eval/sdk-plan.md`
- `bc-sources/evidence/sdk-plan.md`
- `bc-sources/guardrails/sdk-plan.md`
- `bc-sources/providers/sdk-plan.md`

## References

- ADR-0136 / ADR-0137: foundry topology.
- `microservices/foundry/PRD.md` — SDK scope per BC.
