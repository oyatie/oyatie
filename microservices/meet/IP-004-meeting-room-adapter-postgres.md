---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-004-meeting-room-adapter-postgres
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet
acceptance_lanes: [cargo-nextest, oya-governance-postgres-rls-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: meeting-room Postgres adapter (RLS + tenant partitioning)

## Intent

Implement `MeetingRoomRepository` against Postgres 16 with mandatory Row-Level Security (`tenant_id = current_setting('app.tenant_id')`). Partitioned by `(tenant_id, room_id mod N)`. Migrations idempotent + reversible via `sqlx migrate`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-meet-meeting-room-adapter-postgres/src/lib.rs` | create — impl of `MeetingRoomRepository` |
| `src/crates/oya-meet-meeting-room-adapter-postgres/migrations/000001_meeting_room.sql` | create — table + RLS policy + indexes |

## Code Shape

```sql
-- 000001_meeting_room.sql
CREATE TABLE meet_meeting_room (
    room_id           UUID PRIMARY KEY,
    tenant_id         TEXT NOT NULL,
    name              TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 80),
    topic             TEXT,
    lobby_policy      JSONB NOT NULL,
    waiting_room_policy JSONB NOT NULL,
    retention_binding JSONB NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    archived_at       TIMESTAMPTZ,
    content_hash      BYTEA NOT NULL
) PARTITION BY HASH (tenant_id);

-- 16 partitions for sharding
CREATE TABLE meet_meeting_room_p0 PARTITION OF meet_meeting_room FOR VALUES WITH (modulus 16, remainder 0);
-- ...p1 through p15

ALTER TABLE meet_meeting_room ENABLE ROW LEVEL SECURITY;
CREATE POLICY meet_meeting_room_tenant_isolation ON meet_meeting_room
  USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_meet_meeting_room_tenant_created ON meet_meeting_room (tenant_id, created_at DESC);
```

## Acceptance Gates

```bash
cargo nextest run -p oya-meet-meeting-room-adapter-postgres
cargo run -p oya-dev-cli -- gate validate postgres-rls-coverage --microservice meet
```

## Test Plan

- RLS enforcement: integration test with two `tenant_id` GUC settings; refuse cross-read.
- Migration reversibility: `sqlx migrate run` then `sqlx migrate revert` clean.
- Partition routing: tenant_id hashes correctly to partition.

## Halt Conditions

- RLS not enabled on the table — refuse merge.
- Missing `tenant_id` on any column — refuse.

## Next IP

[`IP-005-meeting-instance-and-livekit.md`](IP-005-meeting-instance-and-livekit.md)

## References

- ADR-0105; ADR-0131.
- PostgreSQL 16 RLS docs.
