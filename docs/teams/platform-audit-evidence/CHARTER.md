---
doc_status: published
---

# Team: Platform — Audit & Evidence

## Mission
This team owns the tamper-evident audit chain that is the backbone of cross-axis trust across the entire Oyatie product. It exists because the PRD's hard zero — "tenant data egress without consent receipt = 0 events ever" — cannot be enforced without an immutable, replayable, per-tenant-sharded audit chain. It owns ADR-0003 and every crate that emits, validates, and serves audit evidence. It does **not** own the business logic of what events to emit (each axis owns its emission calls); it owns the chain schema, the append-only guarantees, and the evidence portal surface.

## Owned axes / surfaces / contracts
- **Axis(es):** Cross-cutting (emitters live in every axis; chain infrastructure lives here)
- **Surfaces:**
  - `platform-audit-chain-kernel` — `AuditRecord`, `BlockHash`, `ChainShard`, `TenantChainId`
  - `platform-audit-chain-app` — append use-cases, hash-chain integrity, periodic root anchoring
  - `platform-audit-chain-adapter-postgres` — append-only store adapter (Postgres + write-ahead)
  - `platform-audit-chain-api` — read surface for evidence queries, regulatory replay, trust-portal feed
  - Evidence portal (search-axis surface, but audit team owns the evidence-emission spec it reads from)
- **Cross-axis contracts (DESIGN §10):**
  - `Audit-chain event` (owner) — every axis is an emitter; schema changes require all-emitter review
  - `DSR / consent withdrawal cascade` (co-owner with `platform-privacy-dub`) — proof-of-erasure record spec
- **Catalog records:** `crates/platform-audit-chain-*`
- **Runbooks:** `runbooks/audit-chain-integrity-check.md`, `runbooks/dsr-cascade-proof-of-erasure.md`, `runbooks/regulatory-replay.md`
- **ADRs:** ADR-0003 (audit chain — sole owner and maintainer)

## In-scope work
- `AuditRecord` schema design and versioning (schema changes require all-emitter review)
- Hash-chain append-only enforcement and integrity proofs
- Per-tenant chain sharding (one shard per tenant; cross-tenant root index for global proofs)
- Periodic root anchoring publication to trust portal
- Regulatory replay: reconstruct state at any prior timestamp per regulator request
- DSR cascade: proof-of-erasure record emission + cryptographic invalidation pointer
- Evidence portal API (read side) for regulators and tenants
- Fitness function `governance-audit-emission` — CI hard-fail on any surface that touches regulated data without emitting a chain record
- Cross-axis emission contract: publish the `AuditEmitter` trait that every axis implements
- Per-tenant chain query surface for `ops-compliance`

## Out-of-scope (anti-scope)
- Business logic that *decides* what events occur (each axis owns its domain logic)
- Tenancy kernel / identity kernel (→ `platform-tenancy-identity`)
- Data Use Boundary ADR (→ `platform-privacy-dub`)
- Eventing backbone / outbox (→ `platform-eventing-og` — audit chain has its own append path, distinct from the outbox)
- Compliance matrix maintenance (→ `ops-compliance`)
- Security program (→ `ops-security`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-tenancy-identity` | `TenantId` for chain shard keying | Per-schema-change |
| `platform-eventing-og` | Kafka topic for cross-axis audit fanout reads (analytics plane) | Per-release |
| `ops-sre-reliability` | SLO targets for chain append latency and integrity-check runbooks | Quarterly |
| `axis-search` | Evidence portal surface (search axis provides the trust-portal read surface) | Wave gate |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All 7 axes | `AuditEmitter` trait, chain append endpoint | Every regulated capability invocation |
| `ops-compliance` | Regulatory replay API, per-regulator evidence packs | Monthly + on-demand |
| `council-privacy` | DSR cascade proof-of-erasure, consent withdrawal records | Per DSR event |
| `axis-foundry` | Evidence chain emission for every agent step (ADR-0003 + `intelligence-evidence`) | Every Foundry run |
| `gtm-customer-success` | Trust portal evidence export for design-partner auditors | Per audit request |

## Success metrics
- **Audit-chain evidence completeness:** 100% of regulated capability invocations emit a record (PRD §4.1 target)
- **Chain append p99 latency:** < 50 ms (data-plane path; control-plane is < 500 ms)
- **Regulatory replay correctness:** 100% — replay must reconstruct state at any prior `t`
- **DSR cascade proof-of-erasure emission:** 100% within 24 h of cascade trigger
- **Chain integrity check failure rate:** 0 detected per week (integrity-check runbook runs daily)
- **Evidence portal query p99:** < 2 s for 90-day window queries

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) — schema changes
- Privacy: privacy council (`teams/council-privacy/CHARTER.md`) — DSR cascade / erasure proof design
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 30-min sync — integrity anomalies, emission coverage gaps, schema-change queue
- Cross-team review: monthly audit-chain schema review with all-axis emitter leads

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; audit-chain schema PRs require security-reviewer agent
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; ADR-0003 amendments are P0

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Axis emits regulated event without chain record | Catastrophic | `governance-audit-emission` CI gate; coverage metric tracked weekly |
| Hash-chain integrity violation (data corruption or tamper) | Catastrophic | Daily integrity-check runbook; periodic root anchoring published publicly |
| Replay fails to reconstruct prior state | High | Replay tests run in CI against snapshot fixtures; quarterly regulator-drill |
| DSR cascade proof-of-erasure delayed > 24 h | High | Automated cascade monitor; PagerDuty alert at 20 h |

## Sources scanned
PRD.md §4.2 (hard zero metric), DESIGN.md §7 (audit chain), §10 (audit-chain event contract row, DSR cascade row), ADR-0003, DOC-CATALOG.md §2.1 (doc.privacy_program dependent docs).
