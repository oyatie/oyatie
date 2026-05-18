---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-mail + ops-onboarding + council-privacy
deciders: axis-mail, council-architecture, ops-onboarding
related_adrs: [ADR-0117, ADR-0131, ADR-0132, ADR-0133, BominalADR-0208]
related_artifacts:
  - microservices/mail/PRD.md (FR-08 migration import)
  - microservices/mail/capacity-model.md
  - microservices/mail/contracts/asyncapi/mail-events.yaml
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (mail µservice)

## Purpose

Specify how the mail µservice handles two scenarios:
1. **Backfill** — a tenant migrates from Gmail / Exchange / IMAP-source / Proton; the import preserves chain-of-custody (PRD FR-08).
2. **Replay** — a stored audit-chain event needs re-emission (e.g., after a bug fix in audit-chain math); search index needs re-derivation from authoritative store; legal-hold scope re-evaluated against the underlying corpus.

## Backfill (tenant migration)

### Contract

When a tenant initiates migration:

1. Tenant provides source credentials + scope (per-mailbox; date range; folder mapping).
2. Migration adapter reads source via IMAP (Gmail / Exchange) or JMAP (Fastmail) or local mbox files; per-message ingest.
3. Per-message integrity preservation:
   - Source SHA-256 hash retained as `MailMessage.source_hash` metadata.
   - Source folder labels preserved (Inbox / Archive / Custom) as `Folder` entities; mapping table stored.
   - Source Message-Id (RFC 5322) retained as `MailMessage.message_id`.
   - Source DKIM-Signature header retained verbatim (NOT re-signed); verification result recorded (`MailMessage.dkim` from import time).
   - Source SPF + DMARC + ARC results recorded as-of-import-time.
   - Source retention class inferred from labels + tenant policy.
   - Audit-chain emits `MailImportBatchSealed{batch_id, source_provider, message_count, time_range, chain_of_custody_seal}`.
4. Per-pack invariants:
   - pack-us-healthcare: BAA must be in place before migration begins; source provider must also have BAA.
   - pack-kr-fss: source retention floor verified (≥ 5y); imports preserved at floor.
   - pack-eu: GDPR Art. 30 record-of-processing entry for the import.
5. Context tagging at import:
   - Tenant-initiated migration defaults to **Professional** context for all imported mail.
   - User-initiated migration defaults to **Personal** context for all imported mail.
   - Mixed migration requires per-mailbox explicit `context_kind` declaration.
   - LEAN check `oya-check-migration-context-tagging` refuses ambiguous import (FM-DCI-03).

### Constraints

- Migration is rate-limited: per-source-provider connection cap + per-tenant cap.
- Migration window: bounded by source provider's IMAP/JMAP IDLE limits (Gmail: ~500 concurrent; Exchange: ~100; depends on tenant license).
- Source DKIM re-verification: NOT re-validated at import (would fail because DNS keys have rotated since); the DKIM-Signature header is preserved verbatim for chain-of-custody only.
- Encryption-at-rest: every imported message is re-encrypted under tenant DEK at storage time.
- Idempotency: re-running migration with same `import_batch_id` is a no-op.

### Verification

- Integration test: 1000-message Gmail-style mbox imported; source hash + folder + retention class preserved; PRD AC-04 + AC-08.
- Chain-of-custody seal: `MailImportBatchSealed` event sealed by Ed25519; verifier `oya-mail-cli migration verify --batch=<id>` re-derives.
- Counts: imported_count == source_count - import_filter_count (per skip rules); no silent drops.

## Replay (audit-chain + index rebuild)

### Contract

Replay re-computes one or more derived data products from authoritative stores:

| Scenario | Replay target | Trigger |
|---|---|---|
| Audit-chain bug fix | Re-seal events from authoritative event log | engine fix; ops-security request |
| Search index corruption | Rebuild Tantivy index from Postgres + S3 | `runbooks/mailbox-restore-from-backup.md` Path A + indexer worker |
| Legal-hold scope re-evaluation | Re-resolve hold scope against current mailbox state | new evidence; hold-scope amendment |
| Retention ledger gap fill | Replay retention sweep deterministically | post-incident reconciliation |
| Deliverability reputation backfill | Recompute reputation score over 30d window from raw deliverability events | post-incident; new scoring algorithm |

### Procedure

1. Operator invokes: `cargo run -p oya-dev-cli -- vcs replay-mail --kind=<kind> --scope=<scope> --reason "<rfc>"`.
2. CLI requires 2-person rule + council-privacy approval (replay can shift historical "truth" and must be audit-trail-bounded).
3. Engine recomputes the derived product from authoritative stores (Postgres + S3 + audit-chain seal log).
4. New events emitted with `replay_kind=<kind>`, `original_event_id=<id>`, `reason=<rfc>`.
5. Audit-chain seal: the replay itself is sealed, distinguishing from the original event.

### Constraints

- Replay does NOT mutate the original event record; it appends a new sealed record with `replayed=true` label.
- Replay cannot resurrect cryptographically-erased mail (hard-deleted; DEK destroyed; Path B of `e2e-encryption-key-recovery.md`).
- Replay output never triggers retro-active legal-hold scope changes on Personal-pillar (DCI-04 invariant remains).
- Per-pack retention preserved across replay; floors enforced.

### Verification

- Integration test: induce a corrupted Tantivy segment; replay rebuilds correctly from Postgres + S3.
- Audit-chain integrity: replay event sealed; original event remains sealed; chain reconstructable.
- Idempotency: re-running replay with same `replay_id` produces identical output.

## Cost model

| Operation | Frequency | Est. cost / call |
|---|---|---|
| Tenant migration import (5GB mailbox) | per-tenant-onboarding | ~$2-10 (S3 PUT + Postgres write + Tantivy index per `capacity-model.md`) |
| Replay audit-chain after bug fix | per-engine-deploy | ~$50 (full re-seal across all tenants × windows) |
| Replay Tantivy index per mailbox | per-corruption | ~$0.50 (single mailbox) |
| Replay deliverability reputation | per-incident | ~$0.10 (30d window per tenant) |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers".

## Limitations

- Backfill quality is bounded by source provider's IMAP/JMAP support. Gmail IMAP omits some metadata (e.g., label hierarchy); inferred at import.
- Replay assumes deterministic functions; if an audit-chain seal function changes between code versions, replay output is explicitly tagged with engine version (`replayed_by_engine_version`).
- Personal-pillar imports require user-side credential entry; no admin shortcut.
- Cross-pack imports require tenant SCC + per-pack regulator approval where applicable.

## References

- `microservices/mail/PRD.md` FR-08 migration import; FR-13 retention ledger
- `microservices/mail/capacity-model.md`
- `microservices/mail/cost-budget.md`
- `microservices/mail/contracts/asyncapi/mail-events.yaml`
- `microservices/mail/runbooks/mailbox-restore-from-backup.md`
- `microservices/mail/policy/dual-context-isolation.md` (Invariant DCI-03, DCI-04)
- ADR-0117 (residency)
- ADR-0133 (cross-tenant pattern)
- Bominal ADR-0215 (Connect retention/legal-hold dual-context)
- RFC 5322 §3.6.4 (Message-Id)
- RFC 9051 §6.4.4 (IMAP UID FETCH)
- Imapsync (open-source IMAP migration tool) — `imapsync.lamiral.info`
