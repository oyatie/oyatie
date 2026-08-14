# libs/ disposition plan — the 130-crate six-way split (G025)

**Class:** plan-first mixed (move / delete / merge) · **Capability span:** multi (libs → capability homes + base/ + governance/ + build/)
**Authority re-queried:** ADR-0701 (live apex), ADR-0562 §3 placement rule + §6 membership lint, ADR-0615 (registry v1.1.0), `specs/capability-registry.json#membership_lint_coverage` (the ruled globs + the frozen unmapped baseline)
**Status:** disposition plan; execution follows per-capability under the move-plan singleton, one executable `*-move-plan.json` at a time (ADR-0614).
**Measured at:** `origin/dev@4a4f71a14`; consumer graph via `cargo metadata` reverse-deps.
**Tracking:** bead/evidence = `evidence/reorg/` (reorg-finish program goal); the wave plans materialize under `specs/reorg/` at execution time.

## Ruled by the registry (no per-crate judgment needed) — 70 crates

| Group | Count | Destination | Class | Note |
|---|---|---|---|---|
| `libs/oya-check-*` + `libs/oya-governance-*` | 55 | `governance/check/*` | move | #1498 precedent: the dep-lint authority's leaf kernels already homed 56 there; these are the same fleet. Governance-engine code, not capability crates (registry comment). |
| `libs/oya-advisory-mirror-kernel`, `oya-buck-syntax-kernel`, `oya-cargo-lock-transform-kernel`, `oya-ci-gate-contract`, `oya-ci-config`, `oya-ci-materializer-kernel`, `oya-crate-registrar-app`, `oya-crate-registrar-kernel`, `oya-workspace-members-kernel` | 9 | `build/` | move | Registry `build/` glob: buck2/workspace/manifest tooling. |
| `libs/oya-workflow-safe-metadata-kernel` | 1 | `workflow/` | move | Registry capability glob (FANOUT-03). |
| `libs/oya-data-boundary-kernel`, `oya-data-outbox-adapter-postgres`, `oya-data-outbox-kernel`, `oya-data-sql-adapter-sqlx`, `oya-data-sql-kernel` | 5 | `data/` | move | Registry `data/` glob (`libs/oya-data-*`): data-plane SQL/outbox/boundary substrate. |

Execution: one move-plan per group; same-PR registry glob flips + baseline burns + catalog row re-keys.

## Frozen unmapped baseline — 60 crates, per-crate disposition (this plan)

Confidence: HIGH (single consumer domain / charter-named) · MEDIUM (judgment, review first) · LOW (needs archaeology).

| Crate | Consumers (caps) | Disposition | Class | Confidence |
|---|---|---|---|---|
| oya-http-middleware-kernel (13) / oya-http-router-kernel (17) / oya-http-runtime-hyper-adapter (11) / oya-http-latency-budget-middleware-infrastructure (0) / oya-http-telemetry-middleware-infrastructure (0) / oya-http-tenant-middleware-infrastructure (0) / oya-http-wide-event-middleware-infrastructure (0) | billing, console, iac, iam (+ libs/oya tails) | `base/http/` (admission: ≥3 capability consumers = billing/console/iac/iam ✓; strictly-below-all: http runtime is rung-level transport — but the substrate DAG (ADR-0635) covers only 11 of 24 capabilities, so an absent node proves nothing; comparable DAG entries for the crate AND every consumer required, else the move is BLOCKED pending topology coverage) | move | HIGH (home), MEDIUM (admission evidence) |
| oya-http-sse-kernel (2) | oya/ tails | rides with the http family → `base/http/sse` or gateway/ (SSE surface = gateway SSOT) | move | MEDIUM |
| oya-json-kernel (0) | — | `base/json` (registry pre-authorized "irreducible primitives") or delete if fan-out stays zero — base/ admission still needs the DAG-coverage evidence pack | move/delete | MEDIUM |
| oya-shared-connector-kernel (10) | gateway only | `gateway/ports/connector` | move | HIGH |
| oya-shared-pdp-kernel (6) | iam, tenancy | `policy/` capability root (24th; Cedar PDP decision-plane kernel). Registry amendment: policy.absorbs gains `libs/oya-shared-pdp-kernel` pre-move; iam physical PDP crates stay iam-mapped per ADR-0615 double-map rule. This is the first real crate for the forward-declared policy root — creates `policy/` with OWNERS + face | move | MEDIUM (needs council review of the policy-root birth) |
| oya-shared-olap-clickhouse-adapter (4) / oya-shared-olap-client-kernel (7) | data, intelligence | `data/` (OLAP storage engines = data charter) | move | HIGH |
| oya-shared-scim-server-kernel (2) | iam | `iam/` (SCIM provisioning) | move | HIGH |
| oya-shared-ulid-id-kernel (3) | iam, tenancy | UNRESOLVED: cross-fleet identifier primitive (event/message/job/request IDs) — tenancy/ would force unrelated capabilities to depend on tenancy/ for identifiers (review 3781184398). base/ candidate once the ≥3-consumer + DAG admission contract is satisfied | move | MEDIUM |
| oya-shared-resource-provider-contract-kernel (6) | tenancy | `tenancy/` | move | HIGH |
| oya-shared-postgres-command-kernel (27) / oya-shared-postgres-command-adapter-sqlx (3) | comms, iam, tenancy (+ libs/oya) | `base/postgres-command/` (≥3 ✓; adapter rides with its kernel; strictly-below-all requires DAG entries for crate AND every consumer — else BLOCKED pending topology coverage) | move | HIGH (home), MEDIUM (admission evidence) |
| oya-shared-platform-contracts-kernel (15) | iam, secrets, tenancy | `base/platform-contracts/` (≥3 ✓; the cross-capability contract vocabulary — strictly-below-all requires comparable DAG entries for the crate AND every consumer; the substrate DAG (ADR-0635) covers only 11 of 24 capabilities, so an absent node proves nothing — else the move is BLOCKED pending topology coverage) | move | HIGH (home), MEDIUM (admission evidence) |
| oya-shared-protocol-parity-kernel (13) / oya-shared-protocol-transport-kernel (9) / oya-shared-protocol-transport-retry-app (0) / oya-shared-realtime-transport-kernel (1) | comms, community, intelligence (+ libs/oya) | base/ candidates (generic REST/AsyncAPI/proto parity + broker/gRPC transport — NOT comms-domain code; review 3781184400). parity-kernel meets the ≥3-capability census (comms/community/intelligence); the DAG admission contract applies | move | MEDIUM |
| oya-shared-transactional-outbox-{kernel,adapter-sqlx,dispatch-app,poller-app,worker-app,runtime-tokio-app} (16 total) | comms, libs, oya | `messaging/` (outbox = messaging charter: idempotency/outbox/backpressure) | move | HIGH |
| oya-shared-outbox-pattern-kernel (0) / oya-shared-outbox-broker-http-adapter (0) | — | `messaging/` with the outbox family; verify the 0-consumer pair isn't dead first | move/delete | MEDIUM |
| oya-shared-audit-chain-client-kernel (0) / oya-shared-audit-event-kernel (3) / oya-shared-audit-digest-adapter-awslc (2) | iam (+ libs) | `audit/` | move | HIGH |
| oya-shared-hyperscaler-metrics-kernel (7) / oya-shared-hyperscaler-metrics-adapter-otlp (0) / -prometheus (0) | comms, libs, oya | `observability/` | move | HIGH |
| oya-shared-compliance-evidence-kernel (1) | libs | `compliance/` | move | HIGH |
| oya-shared-email-comms-kernel (0) | — | `comms/` (email = comms charter) | move | HIGH |
| oya-shared-presence-kernel (0) | — | `comms/` (presence = meet/messenger) | move | MEDIUM |
| oya-shared-i18n-kernel (1) | libs | UNRESOLVED: cross-stack i18n substrate (ADR-0206: one Fluent source, adapters for Leptos/SwiftUI/Compose/GTK...) — console/ would force non-console surfaces to depend on the Leptos shell (review 3781184408). base/ candidate pending admission | move | MEDIUM |
| oya-shared-oidc-client-kernel (1) | oya | `iam/` (OIDC identity) | move | HIGH |
| oya-shared-webhook-delivery-kernel (0) | — | `messaging/` (SHARED outbound webhook delivery per ADR-0169 in `docs/decisions/ADR-0709-general-live-apex.md` — outbound delivery is the messaging charter; NOT the ci inbound GitHub receiver) | move | MEDIUM |
| oya-shared-idempotency-key-kernel (0) | — | `messaging/` or gateway/ — decide with the outbox family | move | MEDIUM |
| oya-shared-tenant-quota-kernel (0) | — | `tenancy/` | move | HIGH |
| oya-shared-timeseries-kernel (0) | — | `data/` | move | HIGH |
| oya-shared-tracing-client-kernel (0) | — | `observability/` | move | HIGH |
| oya-shared-webauthn-server-kernel (0) | — | `iam/` (passkey ladder, ADR-0188) — verify consumer status first | move | MEDIUM |
| oya-shared-vector-store-kernel (0) | — | `intelligence/` (AI substrate) or delete — archaeology first | move/delete | LOW |
| oya-shared-wasm-runtime-kernel (0) | — | `marketplace/` (plugin substrate, ADR-0036) or delete | move/delete | LOW |
| oya-shared-backup-kernel (1) | libs | storage/ (backup = durable storage) or delete — archaeology | move/delete | LOW |
| oya-shared-backbone-{grpc-generated-adapter,grpc-transport-adapter,proto-contracts-kernel,rest-runtime-adapter} (2 total consumers) | libs | UNRESOLVED: the family combines comms messenger/mail with community/social contracts + runtime adapters — a multi-capability COMPOSITION, not gateway code (review 3781184410). delete/app-composition decision at execution (archaeology gate) | move/delete | LOW |
| oya-shared-cursor-pagination-kernel (0) | — | gateway/ (API design primitive) or delete | move/delete | LOW |
| oya-shared-{architecture,semver,supply-chain}-check-cli / oya-shared-bounded-contexts-check-cli (0 each) | — | DELETE (not move): the four binaries are SCAFFOLD stubs whose commands print SCAFFOLD and return success — moving them preserves fake-green gates; the CLI retirement inventory classifies them retired (review 3781184405). Execution deletes with fan-out-zero proof | delete | HIGH |

## Base/ admission evidence (required per ADR-0562 §6 for the base/ rows)

For each proposed base/ crate the move-plan PR must attach: the ≥3-capability consumer census (cargo reverse-deps above), and the strictly-below-all DAG position proof. The canonical `specs/substrate-dependency-dag.json` (ADR-0635) covers only 11 of 24 capabilities — 13 omitted per the registry — so a crate's absence from that bounded graph proves nothing about ordering. The proof requires comparable DAG entries for the crate AND every consumer; without them the base/ move is BLOCKED pending topology coverage, and the crate homes in its dominant consumer capability instead.

## Sequencing (each step one PR, one executable move-plan)

> **NON-EXECUTABLE INVENTORY:** this document authorizes no moves by itself. Execution authority is the move-plan singleton (ADR-0614) plus the per-lane PRs that carry each `*-move-plan.json`. masterplan_v2 work-item registration (G025) is a filed follow-up, not a precondition of this document's correctness.

1. governance/check fleet (55 libs kernels) — largest single burn; mirror #1498.
2. build/ group (9 libs) + workflow leaf (1).
3. messaging outbox family (16+2) + webhook-delivery kernel (1).
4. comms family (protocol transport, email, presence, realtime).
5. data/ (olap + timeseries + the 5 registry-ruled `oya-data-*` crates) + observability (metrics + tracing).
6. audit/ + compliance + iam family (scim, oidc, webauthn) + tenancy (resource-provider, quota, ulid).
7. gateway/ (connector, backbone?, pagination?) + ci/ (check-cli).
8. base/ wave (http family, json, postgres-command, platform-contracts) — requires the admission evidence pack; creates `base/`.
9. policy/ birth (pdp-kernel) — requires the council-reviewed registry amendment.
10. Delete wave: fan-out-zero + archaeology-resolved crates.

## Non-goals

- No force-mapping of genuinely ambiguous crates (the baseline stays honest; no silent exemption).
- No product-surface changes; every move is behavior-preserving via the codemod.
- `base/` creation only via the §6 admission gate, never by convenience.
