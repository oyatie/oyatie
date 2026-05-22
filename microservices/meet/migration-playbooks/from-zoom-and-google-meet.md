---
doc_class: MigrationPlaybook
microservice: meet
vendor: Zoom + Google Meet + Microsoft Teams + Webex (parallel migration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Zoom / Google Meet / Microsoft Teams / Webex → oyatie meet

Audience: an oyatie tenant migrating their video-conferencing substrate from Zoom (Meetings or Phone), Google Meet, Microsoft Teams, or Cisco Webex to oyatie's `meet` µservice.

## Why this migration is non-trivial

- **Zoom** has dedicated apps for desktop / mobile; users have entrenched workflows.
- **Google Meet** is web-first + Workspace-integrated; calendar-meeting auto-join is the main user touch.
- **Microsoft Teams** is deeply Office365-integrated; channels, files, and calls are interleaved.
- **Webex** is enterprise-focused; integration with Webex Calling + Webex devices is the gating concern.
- **All** have recording archives that must be ported or accessed in dual-state.

The 80/20: meeting substrate itself ports (rooms, joins, recordings, transcripts); the 20 % needing care is calendar-integration + recording archive migration + native-app rollout.

## Step 1 — Inventory the source (≤ 1-2 weeks per provider)

For Zoom:

```sh
oya meet migrate inventory \
    --source zoom \
    --zoom-api-key "$ZOOM_API_KEY" \
    --zoom-api-secret "$ZOOM_API_SECRET" \
    --window 2020-01-01..2026-05-20 \
    --out inventory/zoom.yaml
```

Captures: users, scheduled meetings (single + recurring), recordings (cloud), webinars, large-meeting licenses, account-level policies (recording defaults, breakout-room defaults, waiting-room defaults), Zoom Phone numbers (if applicable).

For Google Meet:

```sh
oya meet migrate inventory \
    --source google-meet \
    --google-workspace-id "$WORKSPACE_ID" \
    --service-account-json ./service-account.json \
    --out inventory/google-meet.yaml
```

For Microsoft Teams:

```sh
oya meet migrate inventory \
    --source microsoft-teams \
    --tenant-id "$M365_TENANT_ID" \
    --graph-token "$GRAPH_TOKEN" \
    --out inventory/teams.yaml
```

For Webex:

```sh
oya meet migrate inventory \
    --source webex \
    --webex-access-token "$WEBEX_TOKEN" \
    --out inventory/webex.yaml
```

## Step 2 — Audit recording archive (≤ 1-2 weeks per provider)

Recordings are the long-tail. Decide per-tenant:

- Migrate recordings to oyatie drive µservice (good for searchability + retention compliance).
- Leave on source platform (read-only access via source app; recordings expire per source's retention).
- Hybrid: critical recordings ported; bulk left on source.

```sh
oya meet migrate recording-audit \
    --source zoom \
    --tenant drill-acme \
    --window 2023-01-01..2026-05-20 \
    --out audit/zoom-recording-portability.yaml
```

The audit:

1. Lists every recording.
2. Estimates size + bandwidth cost to download + re-upload.
3. Identifies sensitive recordings (per filename / participant patterns).
4. Recommends per-tenant disposition.

Typical decision: migrate recordings from last 90 days (most-accessed); leave older recordings on source until expiration.

## Step 3 — Provision oyatie meet substrate + per-tenant config (≤ 1 week)

```sh
oya meet tenant-onboard \
    --tenant drill-acme \
    --tenant-class paid \
    --pop-preference us-east-1,eu-west-1,ap-northeast-2 \
    --default-recording-retention 90d \
    --default-transcription-language en-US \
    --default-transcription-vendor whisper-large-v3 \
    --default-translation-targets "es-ES,fr-FR,ja-JP,ko-KR"
```

## Step 4 — Calendar integration cutover (≤ 1-2 weeks)

For Google Workspace:

```sh
oya meet migrate calendar-integration \
    --source google-calendar \
    --target oyatie-meet \
    --replace-existing-meet-links false \
    --new-meetings-only true
```

This adds oyatie meet links to NEW meetings; existing meetings keep their Google Meet links.

For Microsoft 365:

```sh
oya meet migrate calendar-integration \
    --source outlook-calendar \
    --target oyatie-meet \
    --replace-existing-meet-links false \
    --new-meetings-only true
```

## Step 5 — User rollout (≤ 2-8 weeks)

Wave-based rollout:

- Week 1-2: tenant admins + IT champions.
- Week 3-4: 25 % of users.
- Week 5-6: 50 % of users.
- Week 7-8: 100 %.

Each user gets:

- A welcome email with link to oyatie meet web app.
- A native app download (when available).
- A 30-min training session OR a recorded walkthrough.
- Office-hours support for migration questions.

During rollout, users can fall back to source platform (Zoom / Meet / Teams / Webex stay licensed at this stage).

## Step 6 — Recording migration (≤ 4-12 weeks; runs in background)

```sh
oya meet migrate recordings \
    --source zoom \
    --tenant drill-acme \
    --recording-window 2025-08-20..2026-05-20 \
    --target-path drive://drill-acme/meet-recordings-from-zoom/ \
    --concurrency 4 \
    --transcript-port true
```

For each recording:

1. Download from source.
2. Re-encode if necessary (preserve quality).
3. Upload to drive µservice.
4. Port the transcript (if source recording had transcription).
5. Apply retention policy.

Typical throughput: ~ 4 GB / hour per worker; for a 5 TB recording archive, ~ 1 250 hours sequential or ~ 300 hours at concurrency=4.

## Step 7 — Decommission source (≤ 1-2 months)

After 100 % oyatie meet adoption + recording migration complete:

```sh
oya meet migrate decommission \
    --tenant drill-acme \
    --source zoom \
    --evidence-out evidence/migrations/zoom-to-oyatie-drill-acme.json
```

Decommission includes:

- Per-user license downgrade / cancel.
- Source recording deletion (verify ports complete first).
- API integration removal.
- Calendar link cleanup (old meeting links become inert).

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| User workflow disruption (especially Teams users) | High | Wave rollout + training + office hours |
| Recording archive too large to port | High | Decide per-tenant; port-most-recent strategy |
| Calendar integration breaks during cutover | High | New-meetings-only flag; existing meetings retain source link |
| Zoom Phone or Webex Calling tightly integrated | Critical | OUT OF SCOPE for meet; route to comms-voice µservice when available |
| Microsoft Teams channels (chat + files) not in scope | High | Migrate to oyatie messenger + drive µservices separately |
| Webex devices (room kits, headsets) won't work | High | Validate per-device; some support SIP join (in-development for oyatie meet) |
| Transcription accuracy gap (Zoom built-in vs oyatie Whisper) | Medium | Tenant pre-test on representative meetings; tune glossary |
| AV1 codec not universally supported | Low | Fallback to VP9 / H.264 transparently |
| HIPAA-covered meetings require BAA on file | Critical | Validate BAA per-tenant before any PHI-class meet |
| 4K screen-share bandwidth surge | Medium | Tier participants by bandwidth; SVC adapts |
