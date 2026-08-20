---
doc_class: Onboarding
microservice: tenancy
persona: tenancy-engineer + iam-engineer + platform-engineer
related_adrs: [ADR-TEN-001, ADR-0313, ADR-0009]
date: 2026-05-20
doc_status: published
---

# Tenancy Engineer onboarding — first 5 working days on `tenancy`

Audience: a new tenancy engineer, IAM engineer, or platform engineer joining the `tenancy` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, created their first tenant + traversed the lifecycle state machine, configured a conglomerate parent-child relationship with scoped permits, exercised DSR cascade on offboarding, walked the sovereign-child-veto runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 40 min). Note the five-vendor displacement + per-tenant doctrine.
2. Read `ARCHITECTURE.md` § lifecycle-state-machine + § parent-child-relationship + § scoped-permits + § DSR-cascade + § cell-residency (∼ 60 min).
3. Read `decisions/ADR-TEN-001-tenant-lifecycle-parent-child-cedar-permit.md` end-to-end (∼ 50 min). The binding architecture.
4. Read `docs/decisions/ADR-0700-ci-admission-live-apex.md` (∼ 30 min). Conglomerate doctrine.
5. Read `docs/decisions/ADR-0700-ci-admission-live-apex.md` + `feedback_oyatie_is_a_tenant_doctrine` (∼ 30 min).
6. Open the Grafana folder `tenancy`. Primary boards: `tenancy-lifecycle-transition-latency`, `tenancy-relationship-permit-count`, `tenancy-lifecycle-lock-active-total`, `tenancy-offboarding-cascade-lag-seconds`, `tenancy-cross-pack-conflict-total`.
7. Walk `runbooks/README.md`. The on-call runbooks: `lifecycle-lock-stuck.md`, `offboarding-cascade-lag.md`, `sovereign-child-veto.md`, `permit-grant-revocation-cancellation.md`, `relationship-cycle-detected.md`, `kyb-pending-stuck.md`, `cryptoshred-blocked-by-legal-hold.md`, `tenant-migration-rollback.md`.
8. Sit in on the Wednesday tenancy-substrate handoff.

Acceptance: you can sketch the lifecycle path: REST POST `/v1/tenancy/tenants` → state=`requested` → KYB/KYC provider check → state=`kyb_pending` → cell assignment + RLS policy generation → state=`provisioning` → product-µservice provision events emitted → state=`active`. And offboarding: tenant admin POSTs transition → state=`offboarding` → DSR cascade fanout → product µservices acknowledge → retention check → state=`retained` → cryptoshred plan (per `compliance`) → state=`cryptoshredded` → state=`retired` (audit metadata only).

## Day 2 — demo_trial cell bootstrap + first tenant lifecycle traversal

```sh
cargo run -p oya-dev-cli -- tenancy bootstrap \
    --tenant-class demo_trial \
    --cell drill-syd-1 \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/tenancy \
    --valkey-endpoint valkey://drill-valkey-syd-1:6379 \
    --kafka-endpoint kafka://drill-kafka-syd-1:9092 \
    --audit-chain-endpoint http://drill-audit-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 10 min. Verify:

```sh
oya tenancy health --cell drill-syd-1
# Expected:
#   postgres.tenant_lifecycle: up (lag_ms=12)
#   valkey.relationship-permit-cache: up
#   kafka.tenancy-events: connected
#   audit-chain.emit: up
#   lifecycle_state_machine_version: tenant-lifecycle-v1
```

Create a tenant + traverse states:

```sh
# 1. REQUESTED → KYB_PENDING
oya tenancy tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --kind workforce \
    --display-name "ACME Corp" \
    --requesting-principal u-admin@drill.test \
    --requested-pack-set "default,gdpr"
# Output:
#   tenant_id: drill-acme
#   state: requested
#   state_version: 1
#   command_id: c_drill_001
#   audit_event_id: ae_ten_lifecycle_001

# 2. Simulate KYB completion
oya tenancy tenant transition \
    --tenant drill-acme \
    --from-state requested \
    --to-state kyb_pending \
    --evidence "kyb-provider-confirmation:persona:ref-12345"

# 3. KYB_PENDING → PROVISIONING
oya tenancy tenant transition \
    --tenant drill-acme \
    --from-state kyb_pending \
    --to-state provisioning \
    --requesting-principal u-admin@drill.test \
    --reason "KYB cleared via Persona"
# Output:
#   state: provisioning
#   home_cell_assigned: drill-syd-1
#   audit_event_id: ae_ten_lifecycle_002

# 4. Cell finishes provisioning (downstream µservices emit ready events)
oya tenancy tenant transition \
    --tenant drill-acme \
    --from-state provisioning \
    --to-state active \
    --requesting-principal u-admin@drill.test
# Cedar evaluates:
#   - active_transition_requires_kyb_complete_and_home_cell ✓
#   - downstream µservices reported ready ✓
# Output:
#   state: active
#   audit_event_id: ae_ten_lifecycle_003
```

Verify the state machine:

```sh
oya tenancy tenant lifecycle --tenant drill-acme
# Output:
#   tenant_id: drill-acme
#   state: active
#   state_version: 4
#   home_cell: drill-syd-1
#   residency_class: default
#   pack_set: ["default", "gdpr"]
#   active_locks: 0
#   transition_history: [requested, kyb_pending, provisioning, active]
```

Acceptance: cell bootstrap + tenant lifecycle traversal verified.

## Day 3 — Conglomerate parent-child hierarchy + scoped permits

Create a child tenant + a relationship (paid tenant_class baseline feature; shadowed at demo_trial):

```sh
# Create child tenant
oya tenancy tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme-pharma \
    --kind healthcare_provider \
    --display-name "ACME Pharma Inc" \
    --requesting-principal u-admin@drill.test \
    --requested-pack-set "default,hipaa,gdpr"

# Take drill-acme-pharma through the lifecycle to active (shortcut for drill)
oya tenancy tenant fast-track-active \
    --tenant drill-acme-pharma \
    --confirm-kyb true

# Create parent-child relationship: drill-acme owns drill-acme-pharma
oya tenancy relationship create \
    --parent-tenant drill-acme \
    --child-tenant drill-acme-pharma \
    --type owns \
    --starts-at 2026-05-20T00:00:00Z \
    --ends-at 2030-12-31T23:59:59Z \
    --pack-scope "default,gdpr"   # NOT including hipaa; child has stricter
# Cedar evaluates:
#   - tenancy::relationship::create ✓
#   - both tenants in `active` state ✓
#   - no cycle detected ✓
# Output:
#   relationship_id: r_drill_001
#   audit_event_id: ae_ten_relationship_created_001
```

Now demonstrate the scoped-permit model: `owns` does NOT imply `data_read`.

```sh
# Parent admin tries to read child data — DENIED
oya drive file list \
    --tenant drill-acme-pharma \
    --requesting-user u-admin@drill.test \
    --requesting-tenant drill-acme
# Cedar denies:
#   - cross-tenant access requires explicit permit grant
#   - tenancy::relationship::data_read NOT GRANTED for relationship r_drill_001
# Output: 403 Forbidden

# Create scoped permit: parent gets billing_read but NOT data_read
oya tenancy permit create \
    --relationship r_drill_001 \
    --action-namespace "drive::file::list,drive::file::stats" \
    --resource-scope "tenant=drill-acme-pharma" \
    --purpose "billing_audit" \
    --expires-at 2026-08-20T00:00:00Z
# Cedar evaluates:
#   - tenancy::relationship::permit_create ✓
# Output:
#   grant_id: pg_drill_001
#   audit_event_id: ae_ten_permit_granted_001

# Parent can now list files (count) but not read content
oya drive file stats \
    --tenant drill-acme-pharma \
    --requesting-user u-admin@drill.test \
    --requesting-tenant drill-acme
# Output: file_count=128, total_size=4.2 GiB (no per-file access)
```

Try to weaken pack-scope: parent attempts to grant itself `data_read` on HIPAA-scoped data — DENIED:

```sh
oya tenancy permit create \
    --relationship r_drill_001 \
    --action-namespace "drive::file::decrypt" \
    --resource-scope "tenant=drill-acme-pharma,data_class=PHI" \
    --purpose "audit"
# Cedar denies (per ADR-TEN-001 Constraint TEN-C11 + higher-restriction-wins):
#   - child pack (hipaa) denies parent data_read for PHI
#   - tenancy::relationship::parent_override_sovereign_child FORBIDDEN
# Output: 403 Forbidden; sovereign_child_veto_reason="child_pack_denies_parent_phi_access"
```

Acceptance: parent-child relationship works; scoped permits enforced; pack conflict resolved.

## Day 4 — DSR cascade on offboarding

Initiate offboarding:

```sh
# Initiate tenant offboarding
oya tenancy tenant transition \
    --tenant drill-acme-pharma \
    --from-state active \
    --to-state offboarding \
    --requesting-principal u-tenant-admin@drill.test \
    --reason "Tenant requested termination"
# Cedar evaluates:
#   - tenancy::lifecycle::transition ✓
#   - no active lifecycle locks ✓
#   - no active legal holds ✓
# Output:
#   state: offboarding
#   dsr_cascade_started: true
#   audit_event_id: ae_ten_lifecycle_004
```

Server emits `tenancy.offboarding.cascade.requested.v1` to Kafka. Downstream µservices (drive, messenger, mail, calendar, identity, etc.) consume + emit acknowledgments:

```sh
oya tenancy offboarding cascade status --tenant drill-acme-pharma
# Output (after ~ 60 s; cascade is per-µservice async):
#   downstream_acks:
#     - drive: completed (4.2 GiB cryptoshred scheduled)
#     - messenger: completed (MLS groups marked for cryptoshred)
#     - mail: completed (mailboxes scheduled for retention)
#     - calendar: completed (events marked for deletion)
#     - identity: completed (sessions revoked; recovery envelopes scheduled for cryptoshred)
#     - compliance: pending (legal-hold + retention clearance check)
#     - audit-chain: NEVER (audit-chain is retain-forever for compliance)
#   total_lag_seconds: 58
```

Move through retention + cryptoshred:

```sh
# Compliance clears retention + legal-hold checks
oya tenancy tenant transition \
    --tenant drill-acme-pharma \
    --from-state offboarding \
    --to-state retained \
    --evidence "legal-hold-cleared-by:u-legal-counsel@drill.test"

# After retention window (or for drill, skip)
oya tenancy tenant transition \
    --tenant drill-acme-pharma \
    --from-state retained \
    --to-state cryptoshredded \
    --evidence "retention-cleared,cryptoshred-plan-executed"
# Server triggers per-µservice cryptoshred (drive CMK destruction, etc.)
# Output:
#   state: cryptoshredded
#   audit_event_id: ae_ten_lifecycle_005

# Finally, retire (audit metadata only)
oya tenancy tenant transition \
    --tenant drill-acme-pharma \
    --from-state cryptoshredded \
    --to-state retired
# Output:
#   state: retired
#   retained_metadata: tenant_id, audit_event_ids, lifecycle_timestamps (no content)
```

Acceptance: full DSR cascade verified through `retired`.

## Day 5 — Sovereign-child-veto runbook + lifecycle lock

Walk the sovereign-child-veto runbook. Read `runbooks/sovereign-child-veto.md`. Scenario: parent attempts a transition or permit that the child pack denies.

Simulate:

```sh
# Try to grant parent admin access to a child healthcare tenant's PHI
oya tenancy permit create \
    --relationship r_drill_001 \
    --action-namespace "drive::file::decrypt" \
    --resource-scope "tenant=drill-acme-pharma,data_class=PHI"
# Cedar denies; runbook step 1
```

Diagnostic:

```sh
oya tenancy permit debug \
    --proposed-action "drive::file::decrypt" \
    --proposed-resource "tenant=drill-acme-pharma,data_class=PHI" \
    --relationship r_drill_001
# Output:
#   cedar_decision: deny
#   reason: "child_pack_denies_parent_phi_access"
#   child_pack_set: ["default", "hipaa", "gdpr"]
#   conflicting_pack: hipaa
#   pack_rule_id: hipaa-rule-phi-access-tenant-bound
#   action_required: child_tenant_admin_must_grant_explicit_permit
```

Resolution: child tenant admin (drill-acme-pharma) must explicitly grant the permit. Even then, the HIPAA pack may still restrict the action.

Lifecycle lock (incident freeze):

```sh
# Apply a lifecycle lock during a security investigation
oya tenancy lifecycle-lock create \
    --tenant drill-acme \
    --lock-type incident_freeze \
    --reason "Security investigation in progress" \
    --created-by u-incident-commander@drill.test \
    --expires-at 2026-05-27T00:00:00Z
# Output:
#   lock_id: ll_drill_001
#   audit_event_id: ae_ten_lifecycle_lock_001

# Now any destructive transition is blocked
oya tenancy tenant transition \
    --tenant drill-acme \
    --to-state suspended
# Cedar denies (per ADR-TEN-001 Constraint TEN-C15):
#   - lifecycle_lock_blocks_transition
#   - lock_id: ll_drill_001
#   - lock_type: incident_freeze
# Output: 403 Forbidden

# Release after investigation
oya tenancy lifecycle-lock release \
    --lock ll_drill_001 \
    --released-by u-incident-commander@drill.test
```

Acceptance: sovereign-child-veto runbook walked; lifecycle lock applied + released.

## What you've learned

- demo_trial bootstrap + tenant lifecycle traversal.
- Conglomerate parent-child relationship + scoped permit grants.
- Sovereign child pack veto (HIPAA denies parent PHI access).
- DSR cascade on offboarding.
- Lifecycle lock as incident freeze.

Next week: paid tenant_class baseline promotion (cross-region cascade + 50k-child conglomerate prefetch), paid tenant_class expanded deployment tour (sovereign children + spinoff/divestiture + workforce-personal boundary), paid tenant_class regulated-pack overlay tour (per-pack residency + regulator-observable transitions + cross-jurisdictional transfer evidence), and your first production shadow on a tenant migration ceremony.
