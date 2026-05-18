---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-foundation
phase: P01-meet-foundation
status: Active
entry_gate: |
  ADR-0135 (parallel) + ADR-0131 + ADR-0132 accepted; ADR-MEET-0001..0006 accepted;
  /specs/microservices/meet.json published; observability µservice IP-001..IP-015 merged
  so meet can author OpenSLO manifests and pass promotion-readiness gate.
exit_gate: |
  All 15 IPs merged; all ~80 crates compile + nextest green;
  oya gate validate per-microservice-layout --microservice meet exits 0;
  HG-MEET gate registers green; end-to-end create-room + join + record + transcribe +
  webinar-mode drill passes within performance budget; pack-kr overlay deployed.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: meet requires observability gate + tenancy + ontology + audit-chain + cedar + calendar
owner_team: axis-meet
related_adrs: [ADR-0008, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-MEET-0001, ADR-MEET-0002, ADR-MEET-0003, ADR-MEET-0004, ADR-MEET-0005, ADR-MEET-0006]
related_specs: []
date: 2026-05-17
doc_status: published
---

# P01-meet-foundation: Land the meet µservice end-to-end

## Purpose

This phase ships the full meet µservice per ADR-0135 (net-new) + ADR-0132 (single-concern + flat layout): video-meeting platform with named rooms + lobby + recording + live transcription + webinar mode + RTMP egress + opt-in E2E encryption.

It advances master-plan principles:
- Hyperscaler-grade in every practice (Google Meet / Zoom / Microsoft Teams Meetings-class feature parity + native Workflow + Ontology integration).
- Nothing scheduled-for-distinct-tracked-work (no FUTURE stubs; every NFR covered).
- No silent regression (production-tier change gated by observability ADR-0139).
- Per-microservice flat layout (ADR-0131 native authoring).
- Shared substrate (LiveKit + coturn) pattern with messenger huddles per ADR-MSGR-0001.

## Scope

### In-scope

| µservice | Bounded Contexts | Crate count |
|---|---|---|
| `meet` | `meeting-room`, `meeting-instance`, `participant`, `audio`, `video`, `screen-share`, `recording`, `transcription`, `webinar`, `live-stream-egress`, `e2e-encryption` | ~80 crates |

Plus cross-cutting:
- `.github/branch-protection.yaml` — add `release/meet/*` pattern protection.
- `/specs/hyperscaler-gates.json` — register HG-MEET per ADR-0133.
- `Cargo.toml` (workspace) — register 80 crates.

### Out-of-scope

- PSTN dial-in (Open Question 1; successor-IP ADR; Twilio Voice / Vonage adapter).
- SIP / Matrix federation (Open Question 2; successor-IP ADR).
- Whiteboard own-BC vs slides-µservice question (Open Question 5; pending ADR).
- Workflow Studio shell integration UX (owned by `workflow-studio` µservice's PRD).

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-iac-bootstrap.md`](IP-001-iac-bootstrap.md) | Helm chart + Kustomize overlays + Terraform for LiveKit + coturn + Whisper GPU pool + ffmpeg gVisor + SRS RTMP + Postgres + Redis + S3 + Meilisearch | pending | axis-meet + ops-sre-reliability | observability IP-001 |
| [`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md) | Cargo workspace; ~80 crate stubs per BC | pending | axis-meet | — |
| [`IP-003-meeting-room-kernel-domain.md`](IP-003-meeting-room-kernel-domain.md) | `oya-meet-meeting-room-{kernel,domain,usecase}` port traits + entities | pending | axis-meet | IP-002 |
| [`IP-004-meeting-room-adapter-postgres.md`](IP-004-meeting-room-adapter-postgres.md) | Postgres RLS + room CRUD adapter | pending | axis-meet | IP-003 |
| [`IP-005-meeting-instance-and-livekit.md`](IP-005-meeting-instance-and-livekit.md) | Meeting-instance + LiveKit SFU adapter; signaling + room-allocation | pending | axis-meet | IP-004 |
| [`IP-006-participant-and-lobby.md`](IP-006-participant-and-lobby.md) | Participant BC + lobby/waiting-room policy + Cedar gate | pending | axis-meet + ops-security | IP-005 |
| [`IP-007-screen-share-and-tracks.md`](IP-007-screen-share-and-tracks.md) | Audio + video + screen-share track lifecycle via LiveKit | pending | axis-meet | IP-005 |
| [`IP-008-recording-pipeline.md`](IP-008-recording-pipeline.md) | LiveKit egress → ffmpeg (gVisor) → S3 with tenant-DEK envelope | pending | axis-meet + ops-security | IP-005 |
| [`IP-009-transcription-pipeline.md`](IP-009-transcription-pipeline.md) | Whisper streaming live-caption + batch transcript + Meilisearch index | pending | axis-meet + axis-foundry-runtime | IP-008 |
| [`IP-010-webinar-and-breakouts.md`](IP-010-webinar-and-breakouts.md) | Webinar mode (registration + practice + Q&A); breakout rooms | pending | axis-meet | IP-006 |
| [`IP-011-live-stream-egress.md`](IP-011-live-stream-egress.md) | SRS RTMP outbound + WHIP fallback; tenant allow-list | pending | axis-meet + ops-security | IP-005 |
| [`IP-012-e2e-encryption-mls.md`](IP-012-e2e-encryption-mls.md) | Opt-in MLS RFC 9420 + W3C Insertable Streams; Cedar deny recording/transcription | pending | axis-meet + council-privacy | IP-007 |
| [`IP-013-contracts-openapi-asyncapi-proto.md`](IP-013-contracts-openapi-asyncapi-proto.md) | Public OpenAPI 3.2 + AsyncAPI 3.1 + Protobuf v3 contracts | pending | axis-meet | IP-005..IP-012 |
| [`IP-014-cedar-policies-and-data-residency.md`](IP-014-cedar-policies-and-data-residency.md) | Cedar v4.2 policy fragments + pack overlays (kr, eu, us, us-hc, us-financial) | pending | axis-meet + ops-security + council-privacy | IP-006 |
| [`IP-015-hg-meet-registration-and-branch-protection.md`](IP-015-hg-meet-registration-and-branch-protection.md) | HG-MEET registration + branch protection wiring | pending | axis-meet + ops-governance | IP-013 |

## Per-IP Test Coverage Threshold

| Class | Coverage line / branch | Test types required |
|---|---|---|
| kernel | 90 % / 80 % | per-port-trait + per-entity unit; sealed-trait smoke; data-class annotation check |
| domain | 90 % / 80 % | pure-math / pure-logic unit |
| usecase | 85 % / 75 % | orchestrator unit with port mocks; happy + error path |
| adapter (Postgres / Redis / S3 / Meilisearch) | 80 % / 70 % | integration vs real backend |
| adapter (LiveKit / Whisper / ffmpeg / SRS / MLS) | 80 % / 70 % | integration vs real substrate where feasible; else contract-mock |
| rest | 85 % / 75 % | per-endpoint happy + 401 + 403 + 422 |
| worker | 85 % / 75 % | event-loop unit + integration |
| app | 75 % / 65 % | smoke startup |

E2E: ≥ 1 per AC-NN row in PRD.
