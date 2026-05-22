---
doc_class: MigrationPlaybook
microservice: recordings
vendor: Zoom Cloud Recordings + Microsoft Stream (parallel migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Zoom Cloud Recordings / Microsoft Stream → oyatie recordings

Audience: an oyatie tenant moving meeting recordings + transcripts + chat logs from Zoom Cloud Recording or Microsoft Stream to oyatie's native `recordings` µservice — including legal-hold preservation + eDiscovery continuity.

## Why this migration is non-trivial

Zoom and Microsoft Stream both store:

- Recording bytes (MP4).
- Auto-generated transcripts (VTT / SRT or proprietary).
- In-meeting chat logs.
- Meeting metadata (participants, duration, sharing-status).
- Engagement signals (views, comments, reactions in some).

These export cleanly per-recording but the metadata mapping is non-trivial:

- Zoom's "Personal Cloud Recording" vs "Account Cloud Recording" distinction needs to be preserved (the latter is sometimes a SEC 17a-4(f) record-of-record).
- Microsoft Stream's "Classic" recordings (pre-2024) are stored in Azure Media Services + need re-ingest; "New Stream" recordings are in SharePoint + transferable via Graph API.
- In-flight legal holds at the source must transfer to oyatie's legal-hold registry; gaps in hold continuity are spoliation risk.

## Step 1 — Export source inventory (≤ 1-3 days per 10 k recordings)

Zoom:

```sh
oya recordings migrate inventory \
    --source zoom \
    --zoom-account-id "$ZOOM_ACCOUNT_ID" \
    --zoom-jwt "$ZOOM_JWT" \
    --window 2020-01-01..2026-05-20 \
    --include-personal-recordings true \
    --out inventory/zoom-recordings.jsonl
```

Zoom's API rate-limits at ~ 80 requests/min for recordings.list; for a 10 k-recording corpus, plan 24-48 h.

Microsoft Stream (New Stream + SharePoint):

```sh
oya recordings migrate inventory \
    --source ms-stream-new \
    --tenant-id "$AZURE_TENANT_ID" \
    --client-credential "$AZURE_CLIENT_CRED" \
    --window 2024-01-01..2026-05-20 \
    --out inventory/ms-stream-new-recordings.jsonl
```

Classic Stream (Azure Media Services):

```sh
oya recordings migrate inventory \
    --source ms-stream-classic \
    --ams-account "$AMS_ACCOUNT_ID" \
    --ams-key "$AMS_KEY" \
    --window 2020-01-01..2023-12-31 \
    --out inventory/ms-stream-classic-recordings.jsonl
```

Each inventory entry contains: recording_id, title, duration_seconds, ingested_at, owner_user, organizer_user, participant_list, transcript_url, recording_url, legal_hold_status, retention_policy.

## Step 2 — Legal-hold continuity audit (≤ 1 week)

For each recording in the inventory, check the source's legal-hold status:

```sh
oya recordings migrate hold-audit \
    --inventory inventory/zoom-recordings.jsonl \
    --source zoom \
    --out hold-audit/zoom-hold-status.yaml
```

Cross-reference against the tenant's litigation-hold registry (the records team's authoritative list). Discrepancies are CRITICAL — a recording the tenant THINKS is held but the source says is NOT is a spoliation risk if anyone touches it during migration.

For each discrepancy:

1. Engage a defensive hold on oyatie BEFORE downloading.
2. Notify the tenant's outside counsel.
3. Re-engage the source-side hold (and capture screenshot evidence of the source state).

## Step 3 — Download recordings + transcripts (≤ 2-6 weeks per 10 k recordings)

Zoom recordings download:

```sh
oya recordings migrate download \
    --inventory inventory/zoom-recordings.jsonl \
    --hold-audit hold-audit/zoom-hold-status.yaml \
    --output-dir ./migration-staging/zoom/ \
    --concurrency 4 \
    --resume-on-failure \
    --verify-sha256-on-download
```

The download tool:

1. Engages oyatie-side defensive hold on each recording BEFORE downloading.
2. Downloads recording MP4 + transcript VTT + chat-log JSON.
3. Computes SHA-256 of the downloaded bytes.
4. Stores in the staging area.
5. Marks the inventory entry with `staging_complete: true` + the staged-file paths + hashes.

The throttle (`--concurrency 4`) keeps the source's per-account rate-limit in budget. For 10 k recordings, plan 14-30 d of wall-clock.

## Step 4 — Ingest into oyatie (≤ 1-3 weeks)

```sh
oya recordings migrate ingest \
    --staging ./migration-staging/zoom/ \
    --tenant drill-acme \
    --target-cell drill-syd-1 \
    --preserve-original-metadata \
    --auto-engage-hold-if-source-held \
    --throttle-rate 100-recordings-per-hour
```

The ingest tool:

1. For each recording in the staging area:
   - Verifies SHA-256 matches the inventory entry.
   - Issues a synthetic `recording.ingest.v1` event with provenance metadata (source_platform, source_recording_id, source_ingested_at).
   - Triggers oyatie's standard pipeline: redaction overlay (auto), legal-hold engage (if source-held), audit-chain ingest.
   - Marks the inventory entry with `oyatie_recording_id` + ingestion timestamp.
2. Maintains the throttle to avoid quota burst on the target.

## Step 5 — Transcript fidelity audit (≤ 1 week)

Source-platform transcripts have known WER differences from oyatie's Whisper-large-v3. For each recording:

```sh
oya recordings migrate transcript-fidelity-audit \
    --inventory inventory/zoom-recordings.jsonl \
    --tenant drill-acme \
    --sample-rate 0.05 \
    --out audit/zoom-transcript-fidelity.yaml
```

The audit samples 5 % of recordings; for each, runs oyatie's transcription pipeline and computes WER against the imported source transcript. Expected: source transcripts have WER ~ 9 % (Zoom) vs oyatie's ~ 5.4 %; oyatie's transcripts are MORE accurate but DIFFERENT.

The tenant decides:

- Keep the source transcript as canonical (default; provides litigation continuity — "the transcript at time of meeting").
- Promote oyatie's re-transcription as canonical (preferred for accuracy; emits `transcript_re_transcribed` audit event).
- Maintain both (the default at our retention layer; the source transcript is stored as a sibling artifact).

## Step 6 — eDiscovery continuity check (≤ 3-5 days per case)

For any in-flight eDiscovery case at the source, regenerate the export from oyatie:

```sh
oya recordings ediscovery export \
    --tenant drill-acme \
    --case-id case-smith-vs-acme \
    --custodian drill-user-z \
    --redaction-spec ./case-smith-redaction.yaml \
    --output ./oyatie-export-validation/case-smith-acme/
```

Compare against the source-platform's existing export:

```sh
oya recordings migrate ediscovery-cross-check \
    --source-export ./zoom-export-case-smith-acme/ \
    --oyatie-export ./oyatie-export-validation/case-smith-acme/ \
    --out cross-check/case-smith-acme.yaml
```

The cross-check verifies: same recording_ids, same Bates-numbering scheme, same redaction-spec coverage. Discrepancies are flagged for outside-counsel review BEFORE the source is decommissioned.

## Step 7 — Cutover + source decommission (≤ 2 weeks)

Cutover sequence:

- Day 0: oyatie ingest complete; transcript audit complete; eDiscovery cross-check complete.
- Day 0-7: producers (meet/messenger) continue to send NEW recordings to BOTH source + oyatie; oyatie is shadow.
- Day 7-14: cut over producers to oyatie only; source stops receiving new recordings.
- Day 14-30: keep source recordings accessible for compare-against. NO deletions at source.
- Day 30+: per source contract terms, downgrade or cancel.

Decommission evidence:

```sh
oya recordings migrate sunset-evidence \
    --source zoom \
    --tenant drill-acme \
    --out evidence/migrations/zoom-to-oyatie-drill-acme.json
```

The evidence file enumerates: inventory size, hold-continuity audit, ingest receipts, transcript-fidelity audit, eDiscovery cross-check, cutover timeline, source contract termination date.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Zoom rate-limit slows export | Medium | Schedule over 30-60 d; budget 3× expected wall-clock; use --concurrency 4 not 8 |
| Hold-continuity gap during migration | Critical | Engage oyatie-side defensive hold BEFORE downloading; do not download a held recording without an oyatie hold in place |
| Transcript fidelity surprises the tenant | Medium | Audit + decide canonical transcript per recording before sunset |
| eDiscovery export divergence between source + oyatie | High | Cross-check every in-flight case; do NOT decommission source while a case is in-flight |
| Source contract auto-renews during migration | Medium | Check contract terms; schedule decommission 30+ d before renewal |
| Recording bytes corrupted in transit | High | Verify SHA-256 on download; re-download on mismatch |
| Microsoft Stream Classic recordings stored in Azure Media Services (deprecated) | High | AMS deprecation timeline overlaps with migration; export AMS recordings FIRST to avoid AMS sunset windowing |
| Personal-Cloud-Recording vs Account-Cloud-Recording confusion in Zoom | Medium | Inventory tags both kinds; tenant decides classification per recording before ingest |
| Outside-counsel demand for source-side audit trail | Medium | Retain source-side audit chain ZIPs alongside oyatie's audit chain; do not delete source audit trail during the 30-d post-cutover window |
