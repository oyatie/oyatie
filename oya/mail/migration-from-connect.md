---
doc_class: MigrationGuide
template_id: TPL-MIGRATION-GUIDE
microservice: mail
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-MAIL accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134]
related_specs: [/specs/microservices/mail.json, /specs/microservices/mail/mail.json]
owner_team: axis-mail
date: 2026-05-17
doc_status: published
---

# Migration: `oya-mail-*` → `oya-mail-*`

This document applies the Strangler Pattern from the agent-skills
`deprecation-and-migration` skill to the **mail** µservice specifically. It is
the consumer-facing companion to ADR-0134 (which carries the cross-µservice
migration policy) and ADR-0135 (which carries the target topology).

## Status

**Deprecated as of 2026-05-17 — replacement available and production-proven
in dev cluster.**

| Field | Value |
|---|---|
| Replacement | `oya-mail-*` crate family under `microservices/mail/src/crates/` |
| Removal date | **Advisory** — concrete target is HG-MAIL accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #1) |
| Reason | ADR-0132 no-grouping forward-policy + ADR-0139 per-µservice SLO authority + ADR-0131 per-µservice flat layout + the 11-pack-overlay program (per ADR-0133) is only addressable at µservice granularity, not at suite granularity |
| Migration owner (Churn Rule) | axis-mail |
| Migration window | Phase 2 adapter + Phase 3 canary = ~5 months; Phase 5 removal sweep in month 6 (see ADR-0134) |

## Replacement

The 8 bounded-contexts of the `mail` µservice live under
`microservices/mail/src/crates/` per ADR-0131. Each legacy `oya-mail-*`
crate has a 1:1 replacement under the new prefix:

### Crate import-path map

| Legacy `oya-mail-*` path | New `oya-mail-*` path |
|---|---|
| `oya-mail-domain` | `oya-mail-mailbox-store-domain` (the domain-layer kernel split out; see note below) |
| `oya-mail-mailbox-kernel` | `oya-mail-mailbox-store-kernel` |
| `oya-mail-mailbox-usecase` | `oya-mail-mailbox-store-usecase` |
| `oya-mail-mailbox-api` | `oya-mail-mailbox-store-api` |
| `oya-mail-mailbox-adapter-postgres` | `oya-mail-mailbox-store-adapter-postgres` |
| `oya-mail-mailbox-adapter-s3` | `oya-mail-mailbox-store-adapter-s3` |
| `oya-mail-mailbox-rest` | `oya-mail-mailbox-store-rest` |
| `oya-mail-mailbox-worker` | `oya-mail-mailbox-store-worker` |
| `oya-mail-mailbox-sdk` | `oya-mail-mailbox-store-sdk` |
| `oya-mail-mailbox-app` | `oya-mail-mailbox-store-app` |
| `oya-mail-inbound-smtp-kernel` | `oya-mail-inbound-smtp-kernel` |
| `oya-mail-inbound-smtp-usecase` | `oya-mail-inbound-smtp-usecase` |
| `oya-mail-inbound-smtp-rest` | `oya-mail-inbound-smtp-rest` |
| `oya-mail-inbound-smtp-worker` | `oya-mail-inbound-smtp-worker` |
| `oya-mail-inbound-smtp-app` | `oya-mail-inbound-smtp-app` |
| `oya-mail-outbound-smtp-kernel` | `oya-mail-outbound-smtp-kernel` |
| `oya-mail-outbound-smtp-usecase` | `oya-mail-outbound-smtp-usecase` |
| `oya-mail-outbound-smtp-rest` | `oya-mail-outbound-smtp-rest` |
| `oya-mail-outbound-smtp-worker` | `oya-mail-outbound-smtp-worker` |
| `oya-mail-outbound-smtp-app` | `oya-mail-outbound-smtp-app` |
| `oya-mail-imap-frontend-kernel` | `oya-mail-imap-frontend-kernel` |
| `oya-mail-imap-frontend-usecase` | `oya-mail-imap-frontend-usecase` |
| `oya-mail-imap-frontend-rest` | `oya-mail-imap-frontend-rest` |
| `oya-mail-imap-frontend-app` | `oya-mail-imap-frontend-app` |
| `oya-mail-search-index-kernel` | `oya-mail-search-index-kernel` |
| `oya-mail-search-index-usecase` | `oya-mail-search-index-usecase` |
| `oya-mail-search-index-adapter-tantivy` | `oya-mail-search-index-adapter-tantivy` |
| `oya-mail-search-index-adapter-elasticsearch` | `oya-mail-search-index-adapter-elasticsearch` |
| `oya-mail-search-index-worker` | `oya-mail-search-index-worker` |
| `oya-mail-search-index-app` | `oya-mail-search-index-app` |
| `oya-mail-retention-policy-kernel` | `oya-mail-retention-policy-kernel` |
| `oya-mail-retention-policy-usecase` | `oya-mail-retention-policy-usecase` |
| `oya-mail-retention-policy-worker` | `oya-mail-retention-policy-worker` |
| `oya-mail-retention-policy-app` | `oya-mail-retention-policy-app` |
| `oya-mail-legal-hold-kernel` | `oya-mail-legal-hold-kernel` |
| `oya-mail-legal-hold-usecase` | `oya-mail-legal-hold-usecase` |
| `oya-mail-legal-hold-worker` | `oya-mail-legal-hold-worker` |
| `oya-mail-legal-hold-app` | `oya-mail-legal-hold-app` |
| `oya-mail-dual-context-isolation-kernel` | `oya-mail-dual-context-isolation-kernel` |
| `oya-mail-dual-context-isolation-usecase` | `oya-mail-dual-context-isolation-usecase` |
| `oya-mail-dual-context-isolation-app` | `oya-mail-dual-context-isolation-app` |

> **Note on the `-domain` split.** The legacy `oya-mail-domain` crate
> bundled mailbox + SMTP + IMAP + search + retention + legal-hold + context
> isolation into a single domain-layer crate. Per ADR-0131 + ADR-0105 (13-layer
> enum), the new layout splits the domain layer per bounded context. Migration
> imports from the legacy bundled `oya-mail-domain` must each pick the
> specific replacement BC; a one-line wholesale `use oya_mail::*` import is
> not supported.

### Concrete import migration recipes

```rust
// BEFORE
use oya_connect_mail_mailbox_kernel::{Mailbox, MailMessage};
use oya_connect_mail_mailbox_usecase::DeliverInbound;
use oya_connect_mail_legal_hold_kernel::HoldScope;

// AFTER
use oya_mail_mailbox_store_kernel::{Mailbox, MailMessage};
use oya_mail_mailbox_store_usecase::DeliverInbound;
use oya_mail_legal_hold_kernel::HoldScope;
```

```toml
# BEFORE — Cargo.toml of a downstream consumer
[dependencies]
oya-mail-mailbox-kernel  = { workspace = true }
oya-mail-mailbox-usecase = { workspace = true }
oya-mail-legal-hold-kernel = { workspace = true }

# AFTER
[dependencies]
oya-mail-mailbox-store-kernel  = { workspace = true }
oya-mail-mailbox-store-usecase = { workspace = true }
oya-mail-legal-hold-kernel     = { workspace = true }
```

## Reason

The legacy `oya-mail-*` family was authored before the no-grouping
forward-policy (ADR-0132) and the per-µservice flat layout (ADR-0131)
crystallised. Specifically:

1. **ADR-0132 no-grouping forward-policy.** A `connect-*` crate prefix encodes
   bundle membership at the architecture layer; bundle membership is a
   brand-layer concept and must not appear in crate names.
2. **ADR-0139 per-µservice SLO authority.** Mail's mailbox-fill, inbound
   DKIM/SPF/DMARC verify latency, outbound queue depth, IMAP fetch p99, and
   eDiscovery export turnaround each need independent SLO targets. A
   `connect-*` SLO bucket cannot honour those.
3. **ADR-0131 per-µservice flat layout.** Mail's IaC, runbooks, threat-model,
   DPIA, compliance overlay, capacity-model and cost-budget all need to live
   under one folder (`microservices/mail/`). A `connect-*` crate has no
   matching folder.
4. **The 11-pack-overlay program (ADR-0133).** Mail's `pack-kr` (KR-FSS 5y
   retention + 전자문서법 audit-chain), `pack-eu` (GDPR Art. 17 right-to-erasure
   reconciliation), `pack-us` (HIPAA-mail variant), `pack-jp`, `pack-sg`,
   `pack-uk`, etc. each live as `microservices/mail/policy/pack-<region>/`.
   They cannot share a folder root with messenger/calendar/community.

## Migration Guide (step-by-step)

For each consumer crate that imports `oya-mail-*`:

### Step 1 — Add the new dependency

```bash
# In your consumer crate's Cargo.toml, add the new mapped dependency.
# Keep the legacy dependency for now (Phase 2 adapter soak).
```

### Step 2 — Update imports per the import-path map above

```bash
# Use this command per file as a guided rewrite (review every hit; manual
# disambiguation needed for the `oya-mail-domain` split case):
rg -l "oya_connect_mail_" --type rust path/to/your/crate
```

### Step 3 — Verify behavioural parity

```bash
# Inside your consumer crate:
cargo nextest run --features mail-strangler-canary
```

Run with the feature flag enabled to route through the new µservice; run
without to route through the legacy adapter. Compare:

- error variant ordering (Hyrum's Law: external consumers may pattern-match
  on `Err(MailError::Variant)` ordering; new µservice preserves the order
  from the legacy `oya-mail-*` API contract).
- p99 latency (must be ≤ legacy + 5% per ADR-0134 Phase 3 canary gate).
- log-line format (preserved verbatim during the canary; may be tightened in
  a successor-IP `feedback_no_silent_regression`-conforming ADR).

### Step 4 — Remove the legacy dependency

Only after your consumer crate's tests pass against the new imports AND the
mail µservice's Phase 3 canary reaches 100% traffic (per ADR-0134), remove
the legacy dependency from your `Cargo.toml`:

```toml
# Remove this line:
oya-mail-mailbox-kernel = { workspace = true }
```

### Step 5 — Verify zero residual

```bash
# Per ADR-0134 Phase 4 verification:
cargo tree -e normal -p your-crate | grep oya-mail   # expect empty
rg "use oya_connect_mail_" --type rust path/to/your/crate    # expect zero hits
```

## Configuration delta

| Configuration key | Legacy | New |
|---|---|---|
| Feature flag namespace | `connect.mail.*` | `mail.*` |
| OpenSLO file | bundled in `Connect.openslo.yaml` (umbrella) | `microservices/mail/slos/mail.openslo.yaml` (per-µservice) |
| Helm chart values key | `.Values.connect.mail.*` | `.Values.mail.*` |
| K8s namespace | `connector` | `mail` |
| Cedar policy fragment path | `policy/connect/mail/*.cedar` | `microservices/mail/policy/cedar/*.cedar` |
| pack-kr overlay path | `policy/connect/mail/pack-kr/*` | `microservices/mail/policy/pack-kr/*` |
| Workflow event prefix | `connect.mail.*` | `mail.*` (e.g., `mail.MessageReceived`) |
| Ontology type prefix | `Connect.Mail.*` | `Mail.*` (e.g., `Mail.Mailbox`, `Mail.LegalHold`) |
| Telemetry metric prefix | `oya_connect_mail_*` | `oya_mail_*` |
| Tracing span attribute namespace | `connect.mail.*` | `mail.*` |

## Dual-context isolation invariant (preserved)

The Personal ↔ Professional context isolation invariant from the legacy
`oya-mail-dual-context-isolation-kernel` is preserved verbatim in
`oya-mail-dual-context-isolation-kernel`. Specifically:

- The `ContextBoundaryGuard` port trait keeps the same method signatures.
- Cross-context attempts (Professional → Personal mailbox read) emit the
  same 403 + same audit-chain event variant (`MailCrossContextRefused`).
- The kernel-layer refusal (not adapter-layer) invariant is preserved.

This means downstream consumers that wrap the boundary guard via the legacy
import path will see identical behaviour after migration; no test rewrite
needed for the isolation surface.

## Hyrum's-Law surfaces — explicit callouts

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law Makes Removal
Hard", these are the legacy mail surfaces with observable behaviour that may
be depended on. Each is preserved verbatim during the canary; consumers must
re-test after Phase 5 removal in case they had a long-tail dependency:

1. **Error variant ordering** in `MailError`. New µservice preserves the
   variant declaration order; pattern-matchers that rely on `_` fallthrough
   ordering still work.
2. **Timing characteristics of inbound DKIM verify.** Legacy p99 was
   ~80ms; new µservice targets ≤84ms (legacy + 5%). Consumers with hard
   timeouts < 80ms were already broken; consumers with timeouts ≥ 84ms are
   safe.
3. **IMAP fetch line buffering.** Legacy used 4 KiB line buffer; new µservice
   uses the same 4 KiB buffer. IMAP clients that grew dependent on exact
   line-fragmentation timing observe no change.
4. **`Message-ID` header round-trip preservation.** Legacy preserved
   `Message-ID` byte-for-byte across delivery; new µservice does the same.
5. **Retention sweep tick cadence.** Legacy ran retention sweeps every 60s;
   new µservice runs every 60s by default. Consumers that depended on
   retention being checked within 60s are unaffected.
6. **Search result ordering for tied-score documents.** Legacy used insertion
   order as the tie-breaker; new µservice preserves insertion order as the
   tie-breaker.

## Phases (per ADR-0134)

| Phase | Description | Status (mail) | Exit condition |
|---|---|---|---|
| 1. Parallel ship | New µservice + legacy coexist | **active** | HG-MAIL passes at p99 SLOs in dev cluster sustained 7d |
| 2. Adapter soak | `oya-mail-migration-adapter` shims legacy symbols → new impl | pending | All consumers compile against adapter; 3-month soak elapses |
| 3. Feature-flagged canary | 10% → 50% → 100% traffic shift over 6 weeks | pending | New µservice carries 100% traffic for 7 consecutive days |
| 4. Zero-active-usage verification | Dependency-graph + telemetry + grep all clean | pending | Verification commands all exit 0 |
| 5. Code removal sweep | Delete legacy crates + Cargo.toml entries + spec pointers | pending | `cargo build --workspace` exits 0; no `oya_connect_mail_*` symbol resolves |
| 6. Umbrella retirement | Conditional on all 8 sub-µservices reaching their own Phase 5 | pending | All 8 HG-<MS> gates green at p99 SLO sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

Per the deprecation-and-migration skill, every deprecation closeout must
satisfy these checks. Each is gated by a concrete command:

- [ ] **Replacement is production-proven and covers all critical use cases.**
  ```bash
  cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice mail
  # expect: HG-MAIL accepts at p99 SLOs sustained 30d
  ```
- [ ] **Migration guide exists with concrete steps and examples.**
  ```bash
  test -f microservices/mail/migration-from-connect.md   # this file
  ```
- [ ] **All active consumers have been migrated** (per Phase 4):
  ```bash
  cargo tree -e normal -p oya-mail-domain --invert    | grep -v 'oya-mail-migration-adapter' | wc -l   # expect 0
  rg "use oya_connect_mail_" --type rust    | rg -v "migration-adapter|legacy_in_process|tests/"    | wc -l   # expect 0
  ```
- [ ] **Old code, tests, documentation, configuration removed** (per Phase 5):
  ```bash
  find crates -maxdepth 1 -type d -name "oya-mail-*" | wc -l   # expect 0
  test ! -f /specs/microservices/mail.json                          # expect file absent
  ```
- [ ] **No references to the deprecated system remain in the codebase**
  (excluding historical ADR / RETIRED.md / git-log surfaces):
  ```bash
  rg "oya_connect_mail" --type rust    | rg -v "docs/decisions/|RETIRED.md|tests/reference/"    | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed (they served their purpose)** (per
  Phase 5):
  ```bash
  test ! -f microservices/mail/deprecation-notice.md          # expect file absent
  test ! -f microservices/mail/migration-from-connect.md      # expect file absent (this file removes itself in Phase 5)
  ```

## Breaking changes (flagged per `feedback_no_silent_regression`)

This migration is **NOT a breaking change** during Phases 1–4: the adapter
preserves the legacy symbol surface verbatim, including error variant
ordering and timing characteristics within the +5% canary tolerance.

Phase 5 (code removal) **IS a breaking change** for any consumer that did
not migrate during the 5-month adapter+canary window. Per
`feedback_no_silent_regression`:

- Sunset schedule (advisory): 6 months from this document's `deprecation_date`
  (2026-05-17), so a target advisory removal date of **2026-11-17**
  (subject to the HG-MAIL retirement trigger gating).
- Owning axis (axis-mail) ships migration ChangeSets for every internal
  consumer per the Churn Rule before Phase 5.
- External consumers (reading `/specs/microservices/mail.json`) receive
  a 6-month sunset window from this notice; the spec file's `deprecated:
  true` + `replacement_path: /specs/microservices/mail/mail.json` fields render
  in the agent-coordination dashboard.

## References

- ADR-0135: super-app expansion into 8 flat µservices.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-grouping forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0134: dissolution Strangler migration (operational policy).
- `microservices/mail/PRD.md` — full target-state product definition.
- `microservices/mail/PHASE-01-MAIL-DISSOLUTION-FROM-CONNECT.md` — phase plan.
- `microservices/mail/deprecation-notice.md` — formal deprecation notice.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern + Adapter Pattern + Churn Rule + Verification.
