---
doc_class: BackfillReplayPlan
template_id: TPL-BACKFILL
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + ops-sre-reliability + council-privacy
related_artifacts:
  - microservices/anonymous/PRD.md §"Design Invariants" (I1, I3)
  - microservices/anonymous/decisions/ADR-ANON-0001-cryptographic-blinding-protocol.md
  - microservices/anonymous/decisions/ADR-ANON-0004-retention-and-deletion-policy.md
  - microservices/anonymous/failure-modes.md
doc_status: published
---

# Backfill + Replay Plan — anonymous µservice

## Purpose

Define the canonical procedures for **backfill** (initial data load into a new region / pack / shard) and **replay** (re-emit historical events into downstream consumers after a regression). Both operations must preserve the **cryptographic blinding** that the µservice's I1 invariant rests on — this distinguishes anonymous from every sibling µservice's backfill/replay plan.

**Load-bearing constraint**: A backfill or replay MUST NOT introduce any path by which `user_id ↔ post_id` becomes correlatable. If a backfill procedure cannot preserve blinding it MUST be refused and escalated as a P0 design failure.

## When backfill is needed

| Scenario | Frequency | Owner |
|---|---|---|
| New regulatory pack onboarding (e.g., pack-ae spinning up) | per-pack-onboarding | axis-anonymous + ops-platform |
| New region / cell onboarding within an existing pack | per-cell | ops-sre-reliability |
| Postgres shard split (when a cell hits 10k posts/sec aggregate per PRD §"Horizontal Scalability") | rare; per shard split | ops-sre-reliability |
| Meilisearch hashtag index rebuild after corruption | rare; on hashtag-index FM-08 incident | ops-data |
| Valkey feed-cache warm after a flush | rolling; routine | ops-sre-reliability |
| Vote-counter Postgres flush after Valkey loss | per Valkey FM-04 incident | ops-sre-reliability |

## When replay is needed

| Scenario | Frequency | Owner |
|---|---|---|
| Downstream Workflow consumer rebuild (replays `AnonymousPostPublished` etc.) | per consumer rebuild | per-consumer team |
| Audit-chain consumer rebuild after audit-chain µservice regression | rare; on audit-chain incident | ops-security + audit-chain team |
| Foundry-runtime classifier replay after model upgrade (re-classify historical posts) | per model upgrade | axis-foundry-runtime + axis-anonymous |
| Trending-aggregator window recomputation after time-source skew | rare | axis-anonymous |
| Transparency-report aggregator replay after retention-policy change | per ADR-ANON-0004 successor-IP | axis-anonymous |

## Backfill Procedure (privacy-preserving)

### Step 0 — Pre-flight refusal checks

A backfill is REFUSED if any of the following are detected (CI lane + manual check):

1. The source dataset contains a **plaintext** identifier column (`user_id`, `email`, `employer_email`, etc.) in any row of `posts`, `votes`, `attestation-bindings`, or `notifications`.
2. The target schema is missing the `blinded_commitment` column on the `posts` table.
3. The blinding protocol private key set at source does not match the target key set (would prevent verification at target).
4. The source dataset is from a tenant that has not authorised cross-region replication.
5. The legal-process disclosure ledger is in `pending` state for any record in the source — backfill must wait until ledger is sealed.

### Step 1 — Source extraction (privacy-aware)

```bash
cargo run -p oya-dev-cli -- vcs backfill extract \
  --microservice anonymous \
  --source pack-kr \
  --target pack-eu \
  --tenant <tenant-id> \
  --invariant-check I1 \
  --blinding-key-rotation-aware \
  --output /tmp/backfill-extract.jsonl.gz
```

The extractor emits records of the shape:

```json
{
  "post_id": "<opaque>",
  "blinded_commitment": "<bbs-plus-commitment-bytes>",
  "affinity_cluster_id": "<opaque>",
  "post_body_ciphertext_at_rest": "<aes-gcm-bytes>",
  "post_created_at": "<rfc3339>",
  "retention_tier": "30d",
  "audit_chain_seal_hash": "<merkle-root>"
}
```

**Note**: `blinded_commitment` is the cryptographic commitment under ADR-ANON-0001's protocol; it is structurally NOT correlatable to a `user_id`.

### Step 2 — Target validation

```bash
cargo run -p oya-dev-cli -- vcs backfill validate \
  --input /tmp/backfill-extract.jsonl.gz \
  --target-schema microservices/anonymous/contracts/asyncapi/anonymous-events.yaml
```

Validation enforces:

- Every record has a non-null `blinded_commitment`.
- No record contains the strings `user_id`, `email`, `employer_email`, or any column named in the I4 deny-list.
- Audit-chain seal hashes link to a verifiable Merkle root at source.

### Step 3 — Target load with re-blinding (if cross-pack)

Cross-pack backfill REQUIRES re-blinding because the blinding-key set is pack-scoped. The procedure:

1. The source emits a **blinded commitment** and an opaque **proof-of-commitment-equivalence** signed by the source's `legal-process-quorum-key` (ADR-ANON-0003).
2. The target verifies the proof-of-commitment-equivalence and re-blinds under the target pack's blinding-key set.
3. The target writes the new commitment + the proof-of-rotation to its own `posts` table.
4. The audit-chain emits a `BackfillReBlindingExecuted` event.

This procedure preserves I1 across packs — the platform never sees a plaintext user_id at any step.

### Step 4 — Post-backfill verification

```bash
cargo run -p oya-dev-cli -- vcs backfill verify \
  --target pack-eu \
  --expected-record-count <N> \
  --invariant-check I1 \
  --invariant-check I3
```

Verification enforces:

- Record count at target matches source extract count.
- I1: a randomised sample of 1000 records verifies via the blind-signature verifier; no record contains a plaintext identifier.
- I3: retention tier preserved per record.

## Replay Procedure

### Replay envelope

```json
{
  "replay_id": "<uuid>",
  "replay_from_event_seq": <N>,
  "replay_to_event_seq": <M>,
  "downstream_consumer": "<consumer-id>",
  "envelope_type": "AnonymousPostPublished | ModerationVerdictEmitted | ...",
  "blinding_preservation_attestation": "<signed-by-replay-operator>"
}
```

### Replay invariants

1. **Replay is idempotent at the consumer**. Downstream consumers MUST handle replay duplicates without re-correlating.
2. **Replay envelopes carry the same blinded-author-commitment as the original event**. The replay operator MUST NOT substitute a plaintext identifier.
3. **Replay is rate-limited**. Default 1000 events/sec/consumer; configurable per-consumer via OpenBao secret.
4. **Replay emits its own audit-chain event** (`ReplayExecuted`); the audit-chain seals both the original event and the replay event.
5. **Hard-deleted records are NEVER replayed**. The tombstone is sufficient signal that the record must not be re-emitted.

### Replay command

```bash
cargo run -p oya-dev-cli -- vcs replay \
  --microservice anonymous \
  --consumer <consumer-id> \
  --from-seq <N> \
  --to-seq <M> \
  --invariant-check I1 \
  --rate-limit 1000eps \
  --audit-chain-seal-required
```

## Cross-µservice replay interactions

| Consumer µservice | Replay shape | Privacy notes |
|---|---|---|
| `audit-chain` | replay all sealed events to rebuild Merkle root | audit-chain consumer treats blinded-commitment as opaque bytes |
| `foundry-runtime` (abuse-classifier re-classify) | replay `AnonymousPostPublished` to re-run classifier with new model version | classifier reads ciphertext-at-rest decrypted under per-post key; classifier verdict is sealed under new `ClassifierVersion` |
| `workflow-engine` (downstream Workflow consumers) | replay events to downstream Workflow nodes per consumer state | downstream Workflows MUST NOT correlate replay events; LEAN lane `oya-check-downstream-no-author-correlation` |
| `observability` (re-emit metrics) | replay never emits metrics; metrics are recomputed | n/a |

## Failure modes during backfill / replay

| Failure | Mitigation | Severity |
|---|---|---|
| Source-extract worker leaks a plaintext identifier into output JSONL | refuse + halt + escalate; rotate extract worker's signing key; audit-chain seal the incident | Sev-1 (anonymity-leak surface) |
| Target re-blinding key not available | halt; require key-ceremony per ADR-ANON-0001 §"Key Ceremony" | Sev-2 |
| Replay envelope contains a plaintext identifier | reject envelope; alert replay operator; rotate operator's key; audit-chain seal | Sev-1 |
| Replay rate exceeds consumer throughput | back off automatically; persist envelope at replay broker | Sev-3 |
| Tombstone violated (hard-deleted record replayed) | abort replay; audit-chain seal incident; ADR-ANON-0004 §"Post-Mortem" runbook | Sev-1 |

## SLOs for backfill / replay

| SLO | Target | Defined at |
|---|---|---|
| Backfill correctness (record count match) | 100% | `slos/hard-delete-propagation-correctness.openslo.yaml` (related) |
| Backfill blinding-preservation | 100% | this doc + LEAN lane `oya-check-blinding-column-isolation` |
| Replay idempotency at consumer | per-consumer SLO | consumer's own SLO file |
| Replay rate-limit conformance | ≥ 99.9% of envelopes within rate-limit | replay broker SLO |

## References

- ADR-ANON-0001 (cryptographic blinding — informs re-blinding procedure)
- ADR-ANON-0003 (legal-process — informs replay-event audit-chain seal model)
- ADR-ANON-0004 (retention — informs tombstone-not-replayed invariant)
- PRD-anonymous §"Design Invariants" + §"Horizontal Scalability"
- `microservices/anonymous/failure-modes.md` FM-01..FM-15
- Bominal ADR-0028 (audit-chain Merkle / Ed25519)
