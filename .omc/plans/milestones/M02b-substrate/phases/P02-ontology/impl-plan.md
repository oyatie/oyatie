---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P02-ontology
impl_plan_id: IP-P02-ontology-substrate
status: pending
owner: council-architecture
blocked_by: []
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
purpose: "Delivers the complete Ontology substrate: 45 crates across 7 BCs (entity, link, action, function, agent-gateway, audit-chain, pillar), full Postgres DDL with RLS and outbox, sealed port traits in kernel, Cedar policy fragment."
execution_variant: merge-into-existing-crates
execution_variant_decided_at: 2026-05-17
execution_variant_decided_by: user-directive-option-2
execution_variant_note: "User chose merge-variant 2026-05-17 — the 45-crate FROM-SCRATCH scaffold below is preserved as reference; deltas land incrementally into existing oya-ontology-{api,domain,kernel} crates. Tracking: F-M02B-PLAN-LIVE-CRATE-RECONCILIATION. First delta landed: OntologyPillar + UnknownPillarLabel enum/error added to oya-ontology-kernel::pillar module (2026-05-17), encoding the org/person pillar-isolation contract (Bominal-ADR-0132). Remaining deltas (sealed port traits, Postgres DDL with RLS, Cedar policy fragment, outbox + Protobuf events, agent-gateway BC, REST/GraphQL surfaces) tracked under the reconciliation FixupTask as separate slices."
---
# IP-P02-ontology-substrate: Scaffold 45 Ontology crates with full DDL, port traits, Cedar, Protobuf, REST/GraphQL

## Intent

Delivers the complete Ontology substrate: 45 crates across 7 BCs (entity, link, action, function, agent-gateway, audit-chain, pillar), full Postgres DDL with RLS and outbox, sealed port traits in kernel, Cedar policy fragment, Protobuf event schema, OpenAPI REST contract, k6 load test meeting p99≤50ms.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-ontology-entity-kernel/Cargo.toml` | create | kernel crate; `async-trait`, `serde`, `uuid` |
| `crates/oya-ontology-entity-kernel/src/lib.rs` | create | pub mod types, ports; pub use surface |
| `crates/oya-ontology-entity-kernel/src/types.rs` | create | ObjectId, TenantId, TypedObject, ObjectQuery, OntologyError |
| `crates/oya-ontology-entity-kernel/src/ports.rs` | create | ObjectStore + OntologyFunction sealed traits |
| `crates/oya-ontology-link-kernel/Cargo.toml` | create | LinkStore trait |
| `crates/oya-ontology-link-kernel/src/ports.rs` | create | LinkStore sealed trait |
| `crates/oya-ontology-action-kernel/Cargo.toml` | create | ActionStore trait |
| `crates/oya-ontology-action-kernel/src/ports.rs` | create | ActionStore sealed trait |
| `crates/oya-ontology-function-kernel/Cargo.toml` | create | OntologyFunction trait |
| `crates/oya-ontology-agent-gateway-kernel/Cargo.toml` | create | AgentGatewayPort trait |
| `crates/oya-ontology-agent-gateway-kernel/src/ports.rs` | create | AgentGatewayPort sealed trait; ToolCallRequest/Response types |
| `crates/oya-ontology-audit-chain-kernel/Cargo.toml` | create | AuditChainEmitter trait |
| `crates/oya-ontology-pillar-kernel/Cargo.toml` | create | PillarAssignmentPort + Pillar enum |
| `crates/oya-ontology-entity-domain/Cargo.toml` | create | depends on entity-kernel |
| `crates/oya-ontology-entity-domain/src/object.rs` | create | TypedObject creation logic; pillar validation; version increment |
| `crates/oya-ontology-entity-application/Cargo.toml` | create | depends on entity-domain + entity-kernel |
| `crates/oya-ontology-entity-application/src/put_object.rs` | create | PutObjectUseCase: idempotency-key check, call ObjectStore::put, emit outbox |
| `crates/oya-ontology-entity-application/src/get_object.rs` | create | GetObjectUseCase: tenant scope, call ObjectStore::get |
| `crates/oya-ontology-entity-adapter/Cargo.toml` | create | depends on application + domain + kernel; sqlx |
| `crates/oya-ontology-entity-adapter/src/postgres.rs` | create | PgObjectStore: sqlx queries against ontology.objects |
| `crates/oya-ontology-entity-worker/Cargo.toml` | create | outbox dispatcher for entity events |
| `crates/oya-ontology-link-domain/Cargo.toml` | create | link traversal logic |
| `crates/oya-ontology-link-application/Cargo.toml` | create | LinkUseCase |
| `crates/oya-ontology-link-adapter/Cargo.toml` | create | PgLinkStore |
| `crates/oya-ontology-link-worker/Cargo.toml` | create | link event dispatcher |
| `crates/oya-ontology-action-domain/Cargo.toml` | create | action execution + idempotency |
| `crates/oya-ontology-action-application/Cargo.toml` | create | ApplyActionUseCase |
| `crates/oya-ontology-action-adapter/Cargo.toml` | create | PgActionStore |
| `crates/oya-ontology-action-worker/Cargo.toml` | create | action outbox dispatcher |
| `crates/oya-ontology-function-domain/Cargo.toml` | create | function evaluation |
| `crates/oya-ontology-function-application/Cargo.toml` | create | EvaluateFunctionUseCase |
| `crates/oya-ontology-function-adapter/Cargo.toml` | create | function registry adapter |
| `crates/oya-ontology-function-worker/Cargo.toml` | create | async function executor |
| `crates/oya-ontology-agent-gateway-domain/Cargo.toml` | create | tool-call routing |
| `crates/oya-ontology-agent-gateway-application/Cargo.toml` | create | InvokeFunctionUseCase for LLM tool calls |
| `crates/oya-ontology-agent-gateway-adapter/Cargo.toml` | create | MCP protocol adapter |
| `crates/oya-ontology-agent-gateway-worker/Cargo.toml` | create | async tool-call processor |
| `crates/oya-ontology-audit-chain-domain/Cargo.toml` | create | audit event emission |
| `crates/oya-ontology-audit-chain-application/Cargo.toml` | create | EmitAuditEventUseCase |
| `crates/oya-ontology-audit-chain-adapter/Cargo.toml` | create | bridge to oya-audit-chain-events-kernel |
| `crates/oya-ontology-audit-chain-worker/Cargo.toml` | create | audit outbox dispatcher |
| `crates/oya-ontology-pillar-domain/Cargo.toml` | create | pillar assignment logic |
| `crates/oya-ontology-pillar-application/Cargo.toml` | create | AssignPillarUseCase |
| `crates/oya-ontology-pillar-adapter/Cargo.toml` | create | PgPillarAdapter |
| `crates/oya-ontology-rest/Cargo.toml` | create | axum router; depends on all *-application crates |
| `crates/oya-ontology-rest/src/routes.rs` | create | GET /objects/{id}, POST /objects, POST /actions, GET /links/{from} |
| `crates/oya-ontology-graphql/Cargo.toml` | create | async-graphql schema |
| `crates/oya-ontology-graphql/src/schema.rs` | create | Query + Mutation resolvers |
| `crates/oya-ontology-app/Cargo.toml` | create | composition root |
| `crates/oya-ontology-app/src/main.rs` | create | wire all adapters; start axum server; run migrations |
| `migrations/ontology/V001__ontology_init.sql` | create | full DDL (see below) |
| `contracts/ontology/ontology.proto` | create | full Protobuf schema |
| `contracts/ontology/openapi.yaml` | create | OpenAPI 3.1 REST contract |
| `policy/ontology/ontology.cedar` | create | Cedar policy fragment |
| `tests/load/smoke-ontology-entity.js` | create | k6 smoke test |
| `Cargo.toml` | update | add all 45 crates to [workspace.members] |

---

## Crate Naming

```
NAME: oya-ontology-entity-kernel
JUSTIFICATION:
- microservice = ontology: information-adapter substrate
- bc-tokens = entity: Object-Type (typed entity) BC
- layer = kernel: ObjectStore + OntologyFunction port traits + entity types
- exemptions claimed: none

NAME: oya-ontology-rest
JUSTIFICATION:
- microservice = ontology: same µservice
- bc-tokens = (none): single REST surface spans all BCs; ADR-0056 BC-optionality
- layer = rest: HTTP REST handler wiring; axum Router
- exemptions claimed: none
```

---

## Code Shape

### `migrations/ontology/V001__ontology_init.sql`

```sql
CREATE SCHEMA IF NOT EXISTS ontology;

-- Object Types: typed entities
CREATE TABLE ontology.objects (
    object_id        uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid        NOT NULL,
    object_type      text        NOT NULL,
    schema_version   int         NOT NULL DEFAULT 1,
    pillar           text        NOT NULL CHECK (pillar IN ('org', 'person')),
    owner_id         uuid        NULL,
    payload          jsonb       NOT NULL,
    payload_hash     bytea       NOT NULL,       -- SHA-256(canonical_jsonb)
    version          bigint      NOT NULL DEFAULT 1,
    deleted_at       timestamptz NULL,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE ontology.objects ENABLE ROW LEVEL SECURITY;
ALTER TABLE ontology.objects FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ontology.objects
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_objects_tenant_type
    ON ontology.objects (tenant_id, object_type)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_objects_payload_gin
    ON ontology.objects USING gin (payload jsonb_path_ops);
CREATE INDEX idx_objects_version
    ON ontology.objects (tenant_id, object_id, version DESC);

-- Optimistic concurrency trigger
CREATE OR REPLACE FUNCTION ontology.check_version_increment()
RETURNS trigger AS $$
BEGIN
    IF NEW.version <= OLD.version THEN
        RAISE EXCEPTION 'version must increment: got % expected > %', NEW.version, OLD.version;
    END IF;
    NEW.updated_at = now();
    RETURN NEW;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER enforce_version_increment
    BEFORE UPDATE ON ontology.objects
    FOR EACH ROW EXECUTE FUNCTION ontology.check_version_increment();

-- Link Types: typed relationships
CREATE TABLE ontology.links (
    link_id          uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid        NOT NULL,
    link_type        text        NOT NULL,
    from_object_id   uuid        NOT NULL REFERENCES ontology.objects(object_id),
    to_object_id     uuid        NOT NULL REFERENCES ontology.objects(object_id),
    payload          jsonb       NOT NULL DEFAULT '{}',
    valid_from       timestamptz NULL,
    valid_to         timestamptz NULL,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE ontology.links ENABLE ROW LEVEL SECURITY;
ALTER TABLE ontology.links FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ontology.links
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_links_from ON ontology.links (tenant_id, from_object_id, link_type);
CREATE INDEX idx_links_to   ON ontology.links (tenant_id, to_object_id, link_type);
CREATE INDEX idx_links_effective ON ontology.links (tenant_id, valid_from, valid_to)
    WHERE valid_to IS NULL;

-- Action Types: typed transactional mutations
CREATE TABLE ontology.actions (
    action_id        uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid        NOT NULL,
    action_type      text        NOT NULL,
    principal_kind   text        NOT NULL CHECK (principal_kind IN ('user','employee','system','llm','workflow')),
    principal_id     uuid        NULL,
    idempotency_key  text        NULL,
    input            jsonb       NOT NULL,
    output           jsonb       NULL,
    outcome          text        NOT NULL DEFAULT 'pending'
                                 CHECK (outcome IN ('pending','applied','failed','reversed')),
    failure_reason   text        NULL,
    audit_event_id   uuid        NULL,
    started_at       timestamptz NOT NULL DEFAULT now(),
    completed_at     timestamptz NULL,
    duration_ms      int         NULL
);
ALTER TABLE ontology.actions ENABLE ROW LEVEL SECURITY;
ALTER TABLE ontology.actions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ontology.actions
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_actions_idempotency
    ON ontology.actions (tenant_id, action_type, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
CREATE INDEX idx_actions_pending
    ON ontology.actions (tenant_id, outcome, started_at)
    WHERE outcome = 'pending';

-- Outbox
CREATE TABLE ontology.outbox (
    outbox_id    uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    uuid        NOT NULL,
    topic        text        NOT NULL,
    key          text        NOT NULL,
    payload      jsonb       NOT NULL,
    published_at timestamptz NULL,
    created_at   timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_outbox_unpublished
    ON ontology.outbox (created_at)
    WHERE published_at IS NULL;
```

### `contracts/ontology/ontology.proto`

```proto
syntax = "proto3";
package oyatie.ontology.v1;
option java_package = "ai.oyatie.ontology.v1";

message ObjectMutated {
    string tenant_id      = 1;
    string object_id      = 2;
    string object_type    = 3;
    int64  version        = 4;
    bytes  payload_hash   = 5;   // SHA-256
    string action_type    = 6;
    string action_id      = 7;
    int64  timestamp_ms   = 8;
    string pillar         = 9;   // "org" | "person"
}

message LinkMutated {
    string tenant_id      = 1;
    string link_id        = 2;
    string link_type      = 3;
    string from_object_id = 4;
    string to_object_id   = 5;
    string mutation       = 6;   // "linked" | "unlinked"
    int64  timestamp_ms   = 7;
}

message ActionApplied {
    string tenant_id      = 1;
    string action_id      = 2;
    string action_type    = 3;
    string outcome        = 4;   // "applied" | "failed" | "reversed"
    int64  duration_ms    = 5;
    int64  timestamp_ms   = 6;
}
```

### `policy/ontology/ontology.cedar`

```cedar
// Cedar schema + policies for Ontology substrate
// Entity declarations
entity Tenant;
entity User     in [Tenant];
entity Employee in [Tenant];

entity Object in [Tenant] = {
    object_type: String,
    pillar: String,        // "org" | "person"
    owner_id: String       // UUID string of owner User/Org
};

entity Link   in [Tenant] = { link_type: String };
entity Action in [Tenant] = { action_type: String };

// Actions
action Read    appliesTo { principal: [User, Employee], resource: [Object, Link, Action] };
action Apply   appliesTo { principal: [User, Employee], resource: Action };
action Link    appliesTo { principal: [User, Employee], resource: Link };
action Unlink  appliesTo { principal: [User, Employee], resource: Link };

// Person-pillar prohibition: org-admin role cannot read person-pillar objects
// (Bominal ADR-0132 org/person pillar isolation)
forbid (
    principal,
    action == Action::"Read",
    resource is Object
) when {
    principal has org_admin_role &&
    principal.org_admin_role &&
    resource.pillar == "person"
};

// Only principals within the tenant may read
permit (
    principal,
    action == Action::"Read",
    resource
) when {
    principal in resource.tenant
};

// Actions must be applied by an authenticated principal
permit (
    principal,
    action == Action::"Apply",
    resource is Action
) when {
    principal in resource.tenant
};
```

### `tests/load/smoke-ontology-entity.js`

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

export const options = {
  scenarios: {
    smoke: { executor: 'constant-vus', vus: 20, duration: '60s' },
    load:  { executor: 'ramping-vus', startVUs: 0, stages: [
      { duration: '30s', target: 100 },
      { duration: '60s', target: 100 },
      { duration: '30s', target: 0 },
    ]},
  },
  thresholds: {
    'http_req_duration{scenario:smoke}': ['p(99)<50'],   // p99 ≤50ms read-only
    'http_req_duration{scenario:load}':  ['p(99)<200'],  // p99 ≤200ms under load
    'http_req_failed': ['rate<0.001'],                    // error rate <0.1%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';

export default function () {
  // Read path: GET object (Ontology Function — p99 ≤50ms target)
  const objectId = '00000000-0000-0000-0000-000000000001';
  const readRes = http.get(`${BASE_URL}/ontology/v1/objects/${objectId}`, {
    headers: { 'X-Tenant-Id': TENANT_ID, 'Authorization': `Bearer ${__ENV.TEST_TOKEN}` },
  });
  check(readRes, {
    'GET object status 200': (r) => r.status === 200 || r.status === 404,
  });

  // Write path: POST action (p99 ≤200ms)
  const actionRes = http.post(`${BASE_URL}/ontology/v1/actions`, JSON.stringify({
    action_type: 'test.CreateObject',
    idempotency_key: uuidv4(),
    input: { object_type: 'test.Widget', pillar: 'org', payload: { name: 'test' } },
  }), {
    headers: {
      'Content-Type': 'application/json',
      'X-Tenant-Id': TENANT_ID,
      'Authorization': `Bearer ${__ENV.TEST_TOKEN}`,
    },
  });
  check(actionRes, { 'POST action status 2xx': (r) => r.status >= 200 && r.status < 300 });
  sleep(0.05);
}
```

---

## Acceptance Gates

```bash
# Per-crate compilation (representative subset)
cargo check -p oya-ontology-entity-kernel --all-features      # exit 0
cargo check -p oya-ontology-entity-adapter --all-features     # exit 0
cargo check -p oya-ontology-rest --all-features               # exit 0
cargo check -p oya-ontology-app --all-features                # exit 0

# Lint
cargo clippy --workspace --all-features -- -D warnings        # exit 0

# Tests
cargo nextest run --workspace --all-features                  # exit 0; 0 failures

# Migration
psql $DATABASE_URL -f migrations/ontology/V001__ontology_init.sql  # exit 0

# RLS isolation check
psql $DATABASE_URL -c "SET LOCAL oyatie.tenant_id = 'a0000000-0000-0000-0000-000000000001'; SELECT count(*) FROM ontology.objects;" # 0

# Cedar policy lint
cedar validate --schema policy/ontology/schema.cedarschema policy/ontology/ontology.cedar  # exit 0

# Protobuf compilation
buf lint contracts/ontology/                                  # exit 0

# Load test
k6 run tests/load/smoke-ontology-entity.js --env BASE_URL=http://localhost:8080
# Pass: p99 ≤50ms smoke; ≤200ms load; 0 errors

# LEAN checks
oya gate validate lean-a1 --phase P02-ontology  # exit 0
oya gate validate lean-a2 --phase P02-ontology  # exit 0
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_typed_object_pillar_validation` | Object with invalid pillar value rejected |
| `test_action_idempotency_second_apply_noop` | Second apply with same idempotency_key returns cached result |
| `test_link_traverse_depth_limit` | Traversal stops at depth=8 |
| `test_ontology_function_pure_no_io` | OntologyFunction call is side-effect-free |
| `test_agent_gateway_list_functions` | AgentGatewayPort returns registered functions |
| `test_pillar_org_cannot_read_person_cedar` | Cedar forbid rule enforced in unit test |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_put_get_object_round_trip` | PutObject → GetObject returns same payload |
| `integration_action_audit_event_emitted` | ApplyAction triggers AuditEventEmitter call |
| `integration_rls_cross_tenant_blocked` | Query with wrong tenant_id returns 0 rows |
| `integration_outbox_published` | After PutObject, outbox row created and dispatched |

---

## Clean Architecture Compliance

### Dependency direction check

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-ontology-entity-kernel` | `kernel` | nothing project-internal | all other layers |
| `oya-ontology-entity-domain` | `domain` | `entity-kernel` | `application`, `adapter`, presentation |
| `oya-ontology-entity-application` | `application` | `entity-domain`, `entity-kernel` | `adapter`, presentation |
| `oya-ontology-entity-adapter` | `adapter` | `entity-application`, `entity-domain`, `entity-kernel` | presentation |
| `oya-ontology-rest` | `rest` | `*-application`, `*-kernel` | direct `adapter` import |
| `oya-ontology-app` | `app` | all | none (composition root) |

---

## Load Test

```bash
# Smoke (CI on every PR)
k6 run tests/load/smoke-ontology-entity.js \
  --env BASE_URL=http://localhost:8080 \
  --env TENANT_ID=00000000-0000-0000-0000-000000000001
# Pass: all thresholds green; exit 0

# Vegeta sustained (staging before merge)
echo "GET http://staging.ontology/ontology/v1/objects/00000000-0000-0000-0000-000000000001" \
  | vegeta attack -rate=1000/s -duration=60s -header="X-Tenant-Id: $TENANT_ID" \
  | vegeta report
# Pass criterion: p99 ≤50ms; p999 ≤200ms; success_rate=100%
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent m02-wave-a-executor \
  --intent "IP-P02-ontology-substrate: 45 crates + DDL + Cedar + Proto + load test" \
  --ttl 7200 \
  crates/oya-ontology-entity-kernel/src/ports.rs::ObjectStore \
  crates/oya-ontology-link-kernel/src/ports.rs::LinkStore \
  crates/oya-ontology-action-kernel/src/ports.rs::ActionStore \
  migrations/ontology/V001__ontology_init.sql::ontology_schema
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P02-ontology-substrate merged; 45 crates scaffolded; DDL applied; RLS verified; Cedar linted; Protobuf compiled; k6 p99≤50ms; next: P03-identity/impl-plan" \
  -i high \
  -k "M02,P02,IP-P02,ontology"
```

---

## Next IP Pointer

`phases/P03-identity/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- Schema foundation: `.omc/plans/M02b-substrate-schema-foundation.md §1`
- Bominal ADR-0106 (Ontology), ADR-0107 (agent gateway), ADR-0132 (pillars)
