---
id: ADR-0598
title: "Commission the comms meet capability-first core slice (comms-meet-api port + comms-meet-usecase)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-22
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: [ADR-0701]
amends: []
depends_on: [ADR-0029, ADR-0510, ADR-0536, ADR-0538, ADR-0555, ADR-0562, ADR-0563]
related: [ADR-0083, ADR-0105, ADR-0131, ADR-0139, ADR-0245, ADR-0280, ADR-0512, ADR-0570]
related_specs:
  - /specs/capability-registry.json
  - /specs/reachability-registry.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Comms meet capability slice

# ADR-0598: Commission the comms meet capability-first core slice

## Status

**Proposed - 2026-06-22 (authored for founder sign-off; door: two-way — an additive build of two new
pure crates behind already-accepted ports/clean-arch seams on top of the existing
`comms/core/meet-domain` kernel; removable by deleting the new crates without unwinding any SSOT or
moving any existing crate; the accounting producer remains the sole face generator).**

## Context

The capability-first reorg (ADR-0562) homes each product capability under
`<capability>/{core,ports,adapters,facade}/`. The `comms` capability tree was established by the
twelfth strangler move (ADR-0562 §10.16, mail + messenger + meet + contact-center) under the existing
`comms/*/*` glob-only workspace membership (ADR-0538), and its ownership/reachability seed is
`comms/OWNERS` (breadth-unlimited, ADR-0555).

The meet domain kernel already lives in its capability home at `comms/core/meet-domain` (the typed,
invariant-checked W-Workspace Meet session/participant records: `MeetSession`, `MeetSessionCreate`,
`ParticipantRef`, `ParticipantRole`, `ParticipantConnectionState`, `RecordingConsentMode`, and the
`MeetError` invariant surface — ADR-0029, `docs/products/workspace/PRD.md`). Its only Cargo
dependency is `libs/oya-data-boundary-kernel`.

Beyond the domain kernel, the meet capability had NO cloud-agnostic application layer: no port
defining the persistence/lifecycle seams, and no usecase composing the room/session lifecycle the
kernel cannot enforce alone. Media/SFU routing, transcription engines, and durable archive storage
were (correctly) absent — those stay out of the kernel — but so was the clean-arch boundary that lets
those adapters be wired LATER without touching the domain.

This slice is purely ADDITIVE: it MOVES nothing (the meet domain is already home) and adds NO
`specs/reorg/*` move plan. It builds exactly two new crates plus their gate-bound catalog records.

## Decision

### D1 — BUILD the cloud-agnostic core slice (clean-arch, owned-stack shape)

Add the port `comms/ports/meet-api` (`comms-meet-api`) and the usecase `comms/core/meet-usecase`
(`comms-meet-usecase`):

- The port defines the seam concrete adapters implement LATER: `MeetSessionStore` (a tenant-scoped
  session-persistence repository trait) plus the typed lifecycle commands (`OpenRoomRequest`,
  `JoinSessionRequest`, `CloseSessionRequest`) and the `MeetLifecycleReceipt` provenance carrier. The
  trait shape models the W5 owned-stack destination; adapters absorb Postgres/Valkey/SFU/transcription
  and durable archive. Per clean-arch ports-in-core (ADR-0570), the `MeetSessionStore` storage-port
  trait is DEFINED in the `ports/` crate, NOT in an adapter.
- The port also defines the fail-closed `AuthorizedMeetContext`: default-deny by construction (no
  anonymous constructor), admitting a call ONLY when a verified principal, a `tenant:`-prefixed scope,
  a non-empty Cedar policy-decision ref, an idempotency key, and an audit-correlation id are ALL
  present. `AuthorizedMeetContext::validate` is the single default-deny gate; a usecase that does not
  call it cannot construct a lifecycle effect. Any future HTTP/gRPC facade MUST present a valid
  context before it touches tenant data (the new-HTTP-surfaces default-deny doctrine), mirroring
  `comms-mail-mailbox-api` and `comms-calendar-api`.
- The port carries the REST/AsyncAPI/proto parity binding for the room-opened lifecycle event
  (`meet_room_opened_protocol_binding`), mirroring messenger's binding discipline so the facade
  surfaces stay in parity across the three transports (REST + gRPC + streaming; ZERO GraphQL,
  ADR-0565).
- The usecase composes the room/session lifecycle over the domain + port: fail-closed authz at every
  entrypoint (OPEN a room creating its first live session with the host, JOIN a participant, CLOSE the
  session), a principal-ownership check, and tenant-isolation defense independent of any backend RLS.
  It is pure application logic — NO persistence, cloud, identity, SFU, or transcription backend; those
  are DEFERRED behind the port traits. An in-memory test fake (`tests/room_session_lifecycle_acceptance.rs`)
  proves the lifecycle without coupling the build to any infra.

### D2 — Adapters DEFERRED behind the ports

The cloud/persistence/identity/media adapters (a Postgres `MeetSessionStore` with FORCE-RLS tenant
isolation, an SFU/media-routing adapter, a transcription adapter, and durable archive storage) are
intentionally OUT of this slice. They are commissioned later behind the unchanged D1 ports, so the
domain and usecase never change at adapter-wiring or owned-stack cutover.

## Consequences

The meet capability gains a tested, cloud-agnostic core (existing domain + new port + new usecase)
with fail-closed authorization wired at the application boundary, while deferring every transient-infra
concern behind clean-arch ports. The slice is byte-deterministic (producer-regenerated faces) and adds
zero new acyclicity/membership/total-accounting debt: the new crates + catalog records are justified by
THIS ADR (the producer derives `justification_ref: ADR-0598` from the paths named below), owned by
`comms/OWNERS` / `registry/catalog/OWNERS`, and reachable via the existing `comms/OWNERS` and
`registry/catalog/` reachability anchors.

## Files

This ADR commissions and justifies the following born paths (the producer's justification resolver
maps each tracked path mentioned here to `ADR-0598`):

`comms/ports/meet-api/BUCK`,
`comms/ports/meet-api/Cargo.toml`,
`comms/ports/meet-api/src/lib.rs`,
`comms/core/meet-usecase/BUCK`,
`comms/core/meet-usecase/Cargo.toml`,
`comms/core/meet-usecase/src/lib.rs`,
`comms/core/meet-usecase/tests/room_session_lifecycle_acceptance.rs`,
`registry/catalog/comms-meet-api.yaml`,
`registry/catalog/comms-meet-usecase.yaml`.

The meet domain crate `comms/core/meet-domain/{BUCK,Cargo.toml,src/lib.rs}` and its catalog record
`registry/catalog/comms-meet-domain.yaml` already live on `dev` and are NOT born by this slice.
