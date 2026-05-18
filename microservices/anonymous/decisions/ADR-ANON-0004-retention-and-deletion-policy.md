---
id: ADR-ANON-0004
status: Accepted
date: 2026-05-17
microservice: anonymous
deciders: axis-anonymous, council-privacy, ops-data, general-counsel
owner: axis-anonymous + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-ANON-0001
related_artifacts:
  - microservices/anonymous/PRD.md (I3, FR-13, FR-15)
  - microservices/anonymous/slos/hard-delete-propagation-correctness.openslo.yaml
  - microservices/anonymous/runbooks/hard-delete-tombstone-corruption.md
  - microservices/anonymous/IP-012-retention-policy-worker.md
purpose: |
  Define retention tiers + hard-delete posture + tombstone-seal requirements
  that anchor PRD invariant I3.
---

# ADR-ANON-0004: Retention + deletion policy — 30-day default; 30/60/90-day tenant-selectable tiers; hard-delete with audit-chain tombstone within p99 ≤ 5s

## Status

Accepted — 2026-05-17.

## Context

PRD I3 commits the platform to short-retention default + hard-delete with tombstone within p99 ≤ 5s. The decision is:

1. **Default retention tier** — 30 days vs 60 days vs 90 days?
2. **Tenant-selectable bounds** — what is the maximum a tenant can opt into?
3. **Hard-delete vs soft-delete** — recovery-window posture?
4. **Tombstone seal posture** — what does the tombstone record? (must NOT record deleted content)
5. **Cross-µservice propagation** — how does deletion cascade to Redis + Meilisearch + audit-chain?
6. **Per-pack overrides** — GDPR storage-limitation vs other packs?

Regulatory anchors:

- **GDPR Art. 17 right to erasure**: "without undue delay"; we interpret as p99 ≤ 5s.
- **GDPR Art. 5(1)(e) storage-limitation**: data kept "no longer than necessary"; supports short defaults.
- **KR PIPA Art. 21**: deletion on user request.
- **CCPA §1798.105**: right to delete.
- **COPPA §312.5**: parental-deletion right for under-13.
- **HIPAA 45 CFR §164.530(j)**: PHI retention policy (when pack-us-healthcare applies; we keep tight).

## Decision

1. **Default tier: 30 days.** Posts older than 30 days are hard-deleted by the `retention-policy-worker`.
2. **Tenant-selectable tiers: 30, 60, 90 days.** Tenant operator may opt up; tenant cannot opt below 30 (avoids gaming the retention floor).
3. **Per-pack max-retention overrides**:
   - pack-eu: max 60 days (GDPR Art. 5(1)(e) storage-limitation principle)
   - pack-us-healthcare: max 60 days (HIPAA tightness)
   - pack-jp: max 60 days (APPI storage-limitation)
   - all other packs: max 90 days
4. **Hard-delete only.** No soft-delete; no recovery window. A deletion is **content-purge** + **tombstone seal**; the tombstone records `(tombstone_id, target_id, target_kind, deleted_at, deletion_reason)` — NEVER the deleted content.
5. **Cross-µservice propagation chain**: post-thread Postgres → feed-timeline Redis → search-index Meilisearch → audit-chain tombstone seal. Each step idempotent; saga pattern (no XA); p99 ≤ 5s.
6. **Tombstone audit-chain seal**: Merkle-sealed per ADR-0028. A tombstone is verifiable from the audit-chain Merkle root; an orphan tombstone (one with no matching record) is treated as harmless; a missing tombstone (record deleted but no tombstone) is a Sev-1 privacy regression per `runbooks/hard-delete-tombstone-corruption.md`.
7. **Right-to-erasure flow**: `DELETE /v1/posts/{post_id}` triggers the propagation chain; SLO `anonymous-hard-delete-propagation-correctness` measures correctness.

## Alternatives Considered

### A. Soft-delete with 7-day recovery window

- **Pros**: Accidental-deletion recovery; UX safety net.
- **Cons**: Violates GDPR Art. 17 "without undue delay" if interpreted strictly; "deleted" record is still retrievable, which is a privacy regression; tenant trust regression (the µservice's privacy claim weakens).
- **Rejected because**: Hard-delete is the privacy-by-design posture; recovery-window is a "soft" privacy commitment that competitors ship; we go strict.

### B. 7-day default retention (instead of 30)

- **Pros**: Maximally short; maximally privacy-preserving.
- **Cons**: Tenant-trust regression (community accumulates valuable Q&A that disappears too fast); product utility regression; user-research data suggests 30 days is the privacy-vs-utility sweet spot.
- **Rejected because**: 30 days is the industry privacy-class default (Sidechat 7d; YikYak unbounded; Whisper unbounded; Blind unbounded — 30d is differentiating).

### C. 90-day default with 30/60/90 tenant tiers

- **Pros**: Closer to industry norms.
- **Cons**: Weaker privacy posture; defeats the "short retention as privacy" tenant value proposition.
- **Rejected because**: We position short-retention as a privacy differentiator.

### D. Tombstone records the deleted content (encrypted) for legal-hold

- **Pros**: Legal-hold support without re-creating record.
- **Cons**: Violates GDPR Art. 17 (the "deleted" data isn't deleted); tenant trust regression; opens new attack surface.
- **Rejected because**: Legal-hold is handled via the legal-process workflow (ADR-ANON-0003) which has its own chain-of-custody.

### E. Hard-delete without audit-chain tombstone

- **Pros**: Simpler.
- **Cons**: No way to prove a deletion happened (regulatory exposure under GDPR Art. 17 verification expectations); auditor friction.
- **Rejected because**: Audit-grade requires the tombstone.

## Consequences

### Positive

- **I3 invariant structurally satisfied.** Short retention is the default; hard-delete propagates within 5s; tombstone seal is verifiable.
- **GDPR Art. 17 alignment.** Right-to-erasure honoured within p99 ≤ 5s — far below "without undue delay" thresholds in practice.
- **Tenant trust differentiator.** 30-day default is industry-leading for privacy-class platforms.
- **Operational simplicity.** Hard-delete-only avoids the soft-delete state-machine complexity.

### Negative

- **No accidental-deletion safety net.** Mitigated: client-side draft-saving + author-confirmation modal at delete time.
- **Tenant cannot pin retention above 90 days.** Mitigated: legal-hold flows via legal-process workflow (with court order) can preserve specific records.
- **Per-pack overrides require pack-specific worker configuration.** Mitigated: helm overlay handles tier max per pack.

### Operational

- IP-012 implements `retention-policy-worker`.
- IP-013 implements the cross-µservice propagation chain.
- `runbooks/hard-delete-tombstone-corruption.md` handles regressions.
- SLO `anonymous-hard-delete-propagation-correctness` at 100% target.
- LEAN lane `oya-check-retention-default-short` ensures default is 30d.

### Regulatory

- **GDPR Art. 17 right-to-erasure**: p99 ≤ 5s response.
- **GDPR Art. 5(1)(e) storage-limitation**: 30-day default; 60-day max for pack-eu.
- **KR PIPA Art. 21 deletion**: honoured.
- **CCPA §1798.105**: right-to-delete honoured.
- **COPPA §312.5**: parental-deletion + under-13 ban.
- **HIPAA 45 CFR §164.530(j)**: pack-us-healthcare 60-day max.

### Invariant Preservation

I3 is structurally satisfied.

## References

- GDPR Reg. 2016/679 Arts. 5, 17
- KR PIPA Arts. 21, 24-2
- CCPA §1798.105
- COPPA §312.5
- HIPAA 45 CFR §164.530(j)
- ADR-0028 (audit-chain)
- ADR-ANON-0003 (legal-process workflow — handles legal-hold path)
- Sweeney L. (2002) — informs k-anonymity-aware retention
