---
doc_class: Tutorial
microservice: meet
persona: tenant-meeting-host + collab-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Host a 100-person webinar with recording, transcription, real-time translation, and breakout rooms

You will: create a webinar room, configure co-host permits, enable recording + transcription + translation, host 100 participants, run breakout rooms, and verify post-meeting artifacts. Total time ≤ 2 hours (1 hour host + 1 hour meeting).

## Pre-requisites

- A paid tier meet cell.
- Tenant `drill-acme` provisioned.
- A host principal + 2 co-host principals.
- 100 participant accounts (or anonymous-join enabled).
- A target webinar topic + slide deck.

## Step 1 — Create the webinar room (≤ 5 min)

```sh
oya meet room create \
    --tenant drill-acme \
    --name 2026-q3-product-update-webinar \
    --capacity 100 \
    --type webinar \
    --recording-enabled true \
    --transcription-enabled true \
    --transcription-language en-US \
    --translation-target-languages "es-ES,fr-FR,ja-JP,ko-KR,zh-CN" \
    --breakout-rooms-allowed true \
    --max-breakouts 10 \
    --pack-overlay public \
    --scheduled-start "2026-05-22T15:00:00Z" \
    --scheduled-duration 60m \
    --recording-retention 90d
```

Output:

```
[room] rm-2026q3-pu
[join-url] https://meet.drill-syd-1.oyatie.local/rm-2026q3-pu
[admin-url] https://meet.drill-syd-1.oyatie.local/rm-2026q3-pu/admin
[recording-storage] drive://drill-acme/meet-recordings/2026-q3-product-update-webinar.mp4 (will be created at recording-start)
[transcription-output] drive://drill-acme/meet-transcripts/2026-q3-product-update-webinar.{txt,srt,vtt,json}
```

## Step 2 — Grant co-host permits (≤ 5 min)

As host:

```sh
oya meet co-host grant \
    --room rm-2026q3-pu \
    --co-host drill-host-b \
    --permissions mute-others,start-stop-recording,manage-breakouts,remove-participants \
    --justification "Q3-product-update-rehearsed-with-host-b"

oya meet co-host grant \
    --room rm-2026q3-pu \
    --co-host drill-host-c \
    --permissions mute-others,manage-breakouts \
    --justification "Q3-product-update-breakout-facilitator"
```

The Cedar gate `meet::co-host::grant` evaluates; permissions are scoped per-room.

## Step 3 — Pre-meeting: pre-join check (≤ 10 min)

Before the meeting starts, run a pre-join check to verify capacity:

```sh
oya meet room pre-join-check \
    --room rm-2026q3-pu \
    --concurrent-participants 100 \
    --simulate-bandwidth \
    --simulate-codecs "VP9,H.264,AV1"
```

Expected:

```
[sfu] sfu-syd-1-az-a active; can serve 100 × 1080p
[bandwidth] aggregate downlink budget: 220 Mbit/s; headroom OK
[codec-negotiation] VP9 99 %, H.264 1 %, AV1 0 % (per expected client distribution)
[recording-substrate] available; capacity OK
[transcription-substrate] available; can handle 5 streams concurrently
[translation-substrate] 5 target languages; capacity OK at ≤ 100 participants
```

## Step 4 — Host opens the room + admit participants (≤ 5 min)

```sh
oya meet room open --room rm-2026q3-pu --as host
```

As participants join, they appear in the lobby. Host admits:

```sh
oya meet room admit --room rm-2026q3-pu --batch-pending
```

This admits all currently-pending participants (a webinar pattern). Alternatively, admit individually for screening.

## Step 5 — Start recording + transcription + translation (≤ 2 min)

```sh
oya meet recording start --room rm-2026q3-pu --include-screen-share
oya meet transcription start --room rm-2026q3-pu
oya meet translation start --room rm-2026q3-pu --target-languages "es-ES,fr-FR,ja-JP,ko-KR,zh-CN"
```

Within ~ 3 s, the room shows:

- Red recording indicator visible to all participants.
- Live captions (en-US) appearing.
- Per-participant language preference: each participant selects which translation to view (or watches the source captions).

## Step 6 — Run breakout rooms (≤ 30 min into the webinar)

```sh
oya meet breakout create \
    --room rm-2026q3-pu \
    --count 8 \
    --name-prefix product-feedback \
    --assignment automatic \
    --duration-minutes 15 \
    --recording-enabled true
```

The platform:

- Creates 8 sub-rooms.
- Auto-assigns ~ 12-13 participants per room.
- Notifies each participant to "move to breakout in 30 seconds".
- Co-host (drill-host-c) can join any breakout for facilitation.
- Each breakout has its own recording + transcription.

After 15 minutes, breakouts auto-close + participants return to main room.

```sh
oya meet breakout close-all --room rm-2026q3-pu
```

## Step 7 — End the webinar (≤ 5 min)

```sh
oya meet recording stop --room rm-2026q3-pu
oya meet transcription stop --room rm-2026q3-pu
oya meet translation stop --room rm-2026q3-pu
oya meet room close --room rm-2026q3-pu
```

The recording-finalisation pipeline:

1. Encode the final MP4 (≤ 2 minutes for 60-min webinar).
2. Generate transcript files (.txt, .srt, .vtt, .json with timestamps).
3. Generate per-language translation files.
4. Upload to drive µservice.
5. Apply retention policy (90 d per the room config).

## Step 8 — Post-meeting: review artifacts (≤ 30 min)

```sh
oya meet room artifacts --room rm-2026q3-pu
```

Expected:

```
[recording-main] drive://drill-acme/meet-recordings/2026-q3-product-update-webinar.mp4 (size: 1.4 GB)
[recording-breakouts] 8 files in drive://drill-acme/meet-recordings/2026-q3-product-update-webinar-breakouts/
[transcript-en-US] drive://drill-acme/meet-transcripts/...en-US.{txt,srt,vtt,json}
[transcript-es-ES] ...es-ES.txt (translated)
[transcript-fr-FR] ...
[participant-list] CSV: 96 / 100 attended; avg time-in-room 52 min
[participant-engagement] median chat messages 8; median raise-hand 0.4; median reaction 12
```

Spot-check transcription accuracy:

```sh
oya meet transcription accuracy \
    --room rm-2026q3-pu \
    --language en-US \
    --sample-size 100
```

Expected: ≥ 92 % WER (Word Error Rate). Common errors: technical product names, proper nouns; consider adding a tenant glossary for next session.

## Step 9 — Audit-chain verification

```sh
oya audit query --tenant drill-acme --service meet --since 4h
```

Expected events:

- `room_created` × 1
- `co_host_granted` × 2
- `pre_join_check_completed` × 1
- `room_opened` × 1
- `participant_admitted` × 100
- `recording_started` × 1
- `transcription_started` × 1
- `translation_started` × 1
- `breakout_created` × 1 (parent event for the batch)
- `breakout_participant_assigned` × 100
- `breakout_closed` × 8
- `recording_stopped` × 1
- `recording_finalized` × 1
- `room_closed` × 1
- `consent_recorded` × 100 (participant consent at join)

## What you've learned

- Webinar room creation + capacity planning.
- Co-host Cedar permit grants.
- Pre-join check + capacity verification.
- Recording + transcription + translation pipelines.
- Breakout room creation + auto-assignment.
- Post-meeting artifact retrieval.
- Audit-chain shape for meet operations.

Next tutorial: `tutorials/run-pack-bound-telemedicine-session.md` — HIPAA-compliant telemedicine session with PHI controls.
