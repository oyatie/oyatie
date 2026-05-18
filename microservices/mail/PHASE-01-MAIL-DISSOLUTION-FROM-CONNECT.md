---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
status: Active
entry_gate: |
  ADR-0135 (Connect full social network super-app dissolution) + ADR-0132 (no-suite forward policy) accepted;
  /specs/microservices/mail.json published; existing crates oya-connect-mail-* recognised as a working
  baseline; cargo workspace ready to accept the ~62 new crates under microservices/mail/src/crates/.
exit_gate: |
  All 15 IPs merged; oya-vcs-promotion-readiness CI lane present and green; oya-mail-* crate family compiles
  + cargo nextest run --workspace exits 0; oya gate validate per-microservice-layout --microservice mail exits 0;
  oya gate validate authority-cohesion exits 0; HG-MAIL gate in /specs/hyperscaler-gates.json registers green;
  end-to-end drills pass (SMTP receive + outbound delivery + IMAP fetch + search + legal-hold engage +
  eDiscovery export + cross-context refusal); pack-kr overlay validates.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: SLO gate must exist before mail's release pointers can advance past dev
  - milestone: M02-multi-pack
    phase: per master-plan-sequencing
    reason: pack-kr + pack-eu + pack-us activation pre-requisite
owner_team: axis-mail + council-privacy
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0208, ADR-0215]
related_specs: [/specs/microservices/mail.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-mail-dissolution-from-connect: Stand up `mail` as its own µservice

## Purpose

Per ADR-0132 + parallel ADR-0135, Connect dissolves into mail / messenger / calendar / community / social / shorts / network / anonymous. This phase ships `mail` as a standalone µservice under `microservices/mail/` per ADR-0131. It carries the dual-context-isolation invariant (Personal vs Professional, kernel-enforced), the four-eyes legal-hold contract, the chain-of-custody-preserving eDiscovery export, and the SMTP/IMAP/JMAP edge surface.

This phase advances master-plan principles:
- Industry-leader competitive parity (Microsoft Exchange Online + Google Workspace + Naver Works Mail).
- Hyperscaler-grade in every practice (per-tenant DEK, encrypted-token search, audit-chained handoffs).
- Nothing scheduled-for-distinct-tracked-work — the legal-hold engine ships GA-quality at M03; no "approximate" hold logic.
- No silent regression — DKIM/SPF/DMARC + retention floors are kernel-enforced.

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `mail` | 8 BCs (see PRD §"Bounded Contexts") | All under `microservices/mail/` per ADR-0131 | `oya-mail-{mailbox-store, inbound-smtp, outbound-smtp, imap-frontend, search-index, legal-hold, retention-policy, dual-context-isolation}-{kernel,domain,usecase,api,adapter,adapter-X,rest,worker,sdk,app}` |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — extend `oya-vcs-promotion-readiness` to cover `release/mail/*` pattern.
- `Cargo.toml` (workspace) — register the ~62 new crates.
- `/specs/hyperscaler-gates.json` — register HG-MAIL gate per ADR-0123.
- `/specs/microservices/mail.json` — promote to `mail.json` reference; retain Connect-side pointer for migration window.
- `registry/artifact-capabilities-registry.json` — add mail-send, mailbox-search, ediscovery-export capabilities.

### Out-of-scope

- Migration of existing `oya-connect-mail-*` callers — owned by IP-M03-CONNECT-MIGR-* phase running in parallel. This phase introduces `oya-mail-*` crates without removing the Connect-side stubs (deprecation cycle).
- Personal-pillar E2E key recovery design — Open Question 4; scheduled-for-distinct-tracked-work to registry/placeholder-debt/adr-follow-ups.yaml#personal-mail-key-recovery.
- JMAP SDK first-class clients (only the wire-level JMAP server ships in this phase).
- Cross-channel hold coordinator — owned by `audit-chain` µservice's own phase; this phase consumes the event.
- Calendar/Messenger/Community integration loops — owned by their respective µservices' phases; this phase exposes mail events for those consumers only.

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| `IP-001-smtp-postfix-layer-a-iac.md` | Helm/Kustomize charts for Layer-A: Postfix-SMTP + Postgres + S3-blob-store + Tantivy/Elasticsearch search index + KMS | pending | axis-mail | — |
| `IP-002-mailbox-store-kernel.md` | `oya-mail-mailbox-store-kernel` crate: port traits + entities (Mailbox, MailMessage, Thread, MimeBlob, RetentionClass) | pending | axis-mail | IP-001 |
| `IP-003-mailbox-store-postgres-adapter.md` | `oya-mail-mailbox-store-adapter-postgres` crate: per-tenant RLS Postgres schema; sharding by tenant_id | pending | axis-mail | IP-002 |
| `IP-004-mailbox-store-s3-adapter.md` | `oya-mail-mailbox-store-adapter-s3` crate: MIME blob CAS with SSE-KMS envelope encryption | pending | axis-mail | IP-002 |
| `IP-005-dual-context-isolation-kernel.md` | `oya-mail-dual-context-isolation-{kernel,domain,usecase,api,adapter,app}` family: ContextBoundaryGuard port + kernel-layer cross-context refusal | pending | axis-mail + council-privacy | IP-002 |
| `IP-006-inbound-smtp-frontend.md` | `oya-mail-inbound-smtp-*` family: SMTP receiver :25/:465 (implicit TLS); DKIM/SPF/DMARC verify; spam/phishing detection (Rspamd integration); cross-tenant routing | pending | axis-mail | IP-002, IP-005 |
| `IP-007-outbound-smtp-frontend.md` | `oya-mail-outbound-smtp-*` family: SMTP submission :587; DKIM sign; deliverability queue; bounce processor; per-tenant reputation score | pending | axis-mail | IP-002, IP-005 |
| `IP-008-imap-frontend.md` | `oya-mail-imap-frontend-*` family: IMAP + JMAP + REST mailbox read surfaces; Apple Mail / Thunderbird / mobile compatibility | pending | axis-mail | IP-002, IP-005 |
| `IP-009-search-index.md` | `oya-mail-search-index-*` family: encrypted-token search index (Tantivy first; Elasticsearch adapter optional later) | pending | axis-mail | IP-002 |
| `IP-010-retention-policy.md` | `oya-mail-retention-policy-*` family: per-tenant + per-mailbox policy; statutory floor enforcement; expiry scheduler with hold-check | pending | axis-mail + council-privacy | IP-002 |
| `IP-011-legal-hold-engine.md` | `oya-mail-legal-hold-*` family: scoped hold; hold-before-purge invariant; four-eyes plaintext-disclosure; chain-of-custody seal; eDiscovery export job | pending | axis-mail + council-privacy + ops-legal | IP-002, IP-010 |
| `IP-012-ediscovery-export.md` | eDiscovery export sealed-bundle format; chain-of-custody verifier; tenant-portal download flow | pending | axis-mail + ops-legal | IP-011 |
| `IP-013-mail-workflow-handoff.md` | mail-to-Workflow handoff: explicit consent/policy-basis check; audit-chain emission; Workflow event integration | pending | axis-mail + axis-workflow | IP-002 |
| `IP-014-hg-mail-authority-cohesion.md` | HG-MAIL registration in `/specs/hyperscaler-gates.json`; authority-cohesion lane integration | pending | axis-mail | IP-001..IP-013 |
| `IP-015-pack-kr-overlay.md` | pack-kr overlay activation: KR-FSS 5y retention floor; PIPA Art. 23/28/29 conformance; 전자문서법 audit-chain seal verification; KR-resident KMS | pending | axis-mail + council-privacy + pack-kr-council | IP-010, IP-011 |

Coverage check vs. PRD §"Bounded Contexts": all 8 BCs covered. Coverage check vs. ADR-0135 Connect-dissolution: mail's slice of the parallel-session dissolution is fully scoped.

## Acceptance Gates

All gates must pass before `exit_gate` is declared.

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --microservice mail
oya gate validate lean-a2 --microservice mail
oya gate validate port-location --microservice mail
oya gate validate layer-correctness --microservice mail
oya gate validate per-microservice-layout --microservice mail
oya gate validate statelessness --microservice mail
oya gate validate shardability --microservice mail
oya gate validate dual-context-cross-boundary --microservice mail
oya gate validate retention-floor-conformance --microservice mail
oya gate validate dkim-key-rotation-conformance --microservice mail
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
```

### Substrate gates introduced by this phase

```bash
oya gate validate ediscovery-chain-of-custody --microservice mail
oya gate validate mail-context-immutability --microservice mail
oya gate validate mail-encryption-tenant-dek --microservice mail
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Inbound SMTP receive | scripted e2e in `tests/e2e/inbound-smtp.sh` | message persisted; DKIM verified; receipt event emitted; ≤1s p99 |
| Outbound SMTP delivery | scripted e2e in `tests/e2e/outbound-smtp.sh` | DKIM signed; recipient MX 2xx; sent event emitted |
| IMAP fetch | scripted e2e in `tests/e2e/imap-fetch.sh` | latest 50 headers ≤300ms p99 |
| Search correctness | `cargo nextest -p oya-mail-search-index-domain --test test_search_correctness_without_plaintext` | results correct; index never plaintext |
| Legal hold engage | `cargo nextest -p oya-mail-legal-hold-app --test test_hold_engage_e2e` | ≤2s; retention sweep skips held messages; audit emitted |
| eDiscovery export | scripted e2e in `tests/e2e/ediscovery-export.sh` | sealed bundle; digest re-derives; ≤24h |
| Cross-context refusal | scripted e2e in `tests/e2e/cross-context-refusal.sh` | 403 + audit-emitted on Professional→Personal attempts |
| Migration import | `cargo nextest -p oya-mail-mailbox-store-app --test migration::test_import_preserves_chain_of_custody` | source hash + folder + retention preserved |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice mail
oya gate validate ontology-type-registry --microservice mail
```

## Clean Architecture Compliance

Per `feedback_clean_architecture_requirements.md`: 12-layer enum + inward-only flow + ports-in-kernel + cross-product refusal + 14 CI lanes.

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-mail-mailbox-store-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-mail-mailbox-store-domain` | `domain` | `kernel` | `usecase`, `adapter*`, `rest`, `worker`, `app` |
| `oya-mail-mailbox-store-usecase` | `usecase` | `domain`, `kernel` | `adapter*`, `rest`, `worker`, `app` |
| `oya-mail-mailbox-store-api` | `api` | `kernel` | `domain`, `usecase` (api is typed-contract only) |
| `oya-mail-mailbox-store-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` |
| `oya-mail-mailbox-store-adapter-postgres` | `adapter` (backend-qualified) | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` |
| `oya-mail-mailbox-store-adapter-s3` | `adapter` (backend-qualified) | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` |
| `oya-mail-mailbox-store-rest` | `rest` | `usecase`, `api`, `kernel` | `adapter*` directly (uses ports) |
| `oya-mail-mailbox-store-worker` | `worker` | `usecase`, `domain`, `kernel` | `adapter*` directly |
| `oya-mail-mailbox-store-sdk` | `sdk` | `api`, `kernel` | everything else |
| `oya-mail-mailbox-store-app` | `app` | (composition-root wiring only) | none — but only wiring |

(Same shape applies to all 8 BCs.)

Port traits live exclusively in `*-kernel` crates; implementations in `*-adapter*`. Domain calls through ports.

Cross-product integration check: this phase introduces NO direct imports between `mail` and other product µservice crates. All cross-product data flow uses Workflow events (`MessageReceived`, `LegalHoldEngaged`, `MailWorkflowHandoffCreated`, etc.) and Ontology reads/writes.

CI lanes that must green: see §"Fitness lane gates" above.

## ChangeSet Contract per IP

Every IP emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). Minimum ChangeSet payload at `microservices/mail/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "mail",
  "milestone": "M03-connect-dissolution",
  "phase": "P01-mail-dissolution-from-connect",
  "claim_paths": ["microservices/mail/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/mail/PRD.md§<section>", "/specs/microservices/mail.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "lean-a3", "lean-a4", "per-microservice-layout", "dual-context-cross-boundary", "retention-floor-conformance"],
  "test_count": {"unit": "<int>", "integration": "<int>", "e2e": "<int>"},
  "coverage_pct": "<float>",
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2", "F10 (privacy)", "F11 (regulatory)"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

Schema validated by `oya-governance-multispectrum-evidence` lane against `/specs/multispectrum-review.json` v2.4.0; PRs without conforming evidence file are refused.

## Per-IP Test Coverage Threshold

Same as observability PHASE-01 §"Per-IP Test Coverage Threshold". Headline:
- kernel: 90% line / 80% branch
- domain: 95% line / 90% branch (math-heavy; high standard)
- usecase: 90% line / 80% branch
- adapter: 85% line / 75% branch (with real backend container)
- rest/worker: 85% line / 75% branch
- sdk: 90% line / 80% branch
- app: 60% line (mostly wiring)
- IaC: helm-install + helm-test smoke per chart + kind cluster e2e

## branch-protection.yaml diff preview

IP-014 (HG-MAIL authority-cohesion) updates `.github/branch-protection.yaml` to extend the `release/mail/*` pattern:

```yaml
branches:
  dev:
    required_status_checks:
      # ADDED:
      - oya-governance-dual-context-cross-boundary
      - oya-governance-retention-floor-conformance
      - oya-governance-dkim-key-rotation-conformance
      - oya-governance-ediscovery-chain-of-custody
      - oya-governance-mail-encryption-tenant-dek

  ? release/mail/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    required_status_checks:
      - oya-vcs-promotion-readiness

  ? release/mail/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    required_status_checks:
      - oya-vcs-promotion-readiness
```

## Oya VCS Symbol Locks

Per ADR-0116, this phase uses `oya vcs` primitives exclusively.

```bash
cargo run -p oya-dev-cli -- vcs claim --agent <id> --intent "<IP-NNN-slug>: <one-line>" --paths "microservices/mail/src/crates/<crate>/**"
cargo run -p oya-dev-cli -- vcs verify --agent <id> --changeset <id>
cargo run -p oya-dev-cli -- vcs done --agent <id> --changeset <id>
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/mail/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- ADR-0135: Connect full social network super-app (parallel-session dissolution authority).
- ADR-0131: Per-microservice flat layout (location authority).
- ADR-0132: No-suite forward policy.
- ADR-0133: Cross-tenant mail-server pattern.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0116: Retire external agent-coordination tooling.
- ADR-0123: Hyperscaler maturity claim gate (HG-MAIL).
- Bominal ADR-0208 + 0210 + 0215 (inherited).
- `/specs/microservices/mail.json`.
- `/specs/per-microservice-flat-layout.json`.
- `microservices/mail/PRD.md`.
- Memory: `feedback_milestone_phase_hierarchy.md`, `feedback_naming_justification.md`, `feedback_oya_vcs_canonical_2026_05_16.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`.
