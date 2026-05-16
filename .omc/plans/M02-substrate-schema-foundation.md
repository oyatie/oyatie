---
purpose: Auto-backfilled purpose for M02-substrate-schema-foundation.md
---

---
title: M02 Substrate Schema Foundation
status: draft (will evolve through Wave 2 Phase SPEC authoring)
authority: feedback_clean_architecture_requirements.md + feedback_workflow_objectgraph_adapter_layer.md
authored: 2026-05-13
purpose: |
  Canonical schemas for the M02 shared substrate µservices. Implementation-ready
  Postgres DDL + Rust port traits + Cedar policy fragments + outbox event
  schemas. Wave 2 Phase SPEC executors expand each section into full Impl Plans.
inheritance: Bominal ADRs 0011/0018/0019/0028/0103/0106/0107/0111/0117/0125/0126/0132
---

## Conventions

- All tables use `gen_random_uuid()` for primary keys (Postgres 16 pgcrypto)
- All multi-tenant tables have `tenant_id uuid NOT NULL` + RLS policy
  `tenant_id = current_setting('oyatie.tenant_id')::uuid`
- All tables have `created_at timestamptz NOT NULL DEFAULT now()` +
  `updated_at timestamptz NOT NULL DEFAULT now()`
- Outbox pattern: every µservice has its own `<bc>_outbox` table + Kafka KRaft
  topic; LISTEN/NOTIFY via Postgres for fan-out per Bominal ADR-0117 stage 1;
  Kafka publisher worker for stage 2+
- Audit: every state-changing action emits an event written to
  `audit-chain` µservice
- Sharding: prepare for Citus by declaring `distribution_column tenant_id` on
  every tenant-bound table (Bominal ADR-0117 stage 2)
- Port traits: declared in `<microservice>-kernel` crates per
  [[feedback-clean-architecture-requirements]] §3; impls in
  `<microservice>-adapter`
- Cedar policy: types declared in `oya-policy-kernel`; per-µservice rule
  packs in `oya-<microservice>-policy/*.cedar`

## 1. Ontology (oya-ontology-*)

Palantir-Ontology equivalent. Foundation for all typed-entity storage across
oyatie. Bominal ADR-0106/0107 + ADR-0132 (pillars) inherited.

### Postgres DDL

```sql
-- ontology objects (Object Types: typed entities)
CREATE TABLE ontology.objects (
    object_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    object_type text NOT NULL,                  -- e.g., 'medical.Encounter', 'hr.Employee'
    schema_version int NOT NULL DEFAULT 1,
    pillar text NOT NULL CHECK (pillar IN ('org', 'person')),  -- Bominal ADR-0132
    owner_id uuid NULL,                          -- Person/Org owner (FK enforced via Cedar)
    payload jsonb NOT NULL,                      -- typed per object_type schema
    payload_hash bytea NOT NULL,                 -- SHA-256(canonical_jsonb)
    version bigint NOT NULL DEFAULT 1,            -- optimistic concurrency
    deleted_at timestamptz NULL,                 -- soft delete; physical purge per retention policy
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE ontology.objects FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ontology.objects USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_objects_tenant_type ON ontology.objects (tenant_id, object_type) WHERE deleted_at IS NULL;
CREATE INDEX idx_objects_payload_gin ON ontology.objects USING gin (payload jsonb_path_ops);

-- ontology links (Link Types: typed relationships)
CREATE TABLE ontology.links (
    link_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    link_type text NOT NULL,                    -- e.g., 'hr.employed_at', 'medical.prescribed_by'
    from_object_id uuid NOT NULL REFERENCES ontology.objects(object_id),
    to_object_id uuid NOT NULL REFERENCES ontology.objects(object_id),
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    valid_from timestamptz NULL,                -- effective-dated relationships
    valid_to timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE ontology.links FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ontology.links USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_links_from ON ontology.links (tenant_id, from_object_id, link_type);
CREATE INDEX idx_links_to   ON ontology.links (tenant_id, to_object_id, link_type);

-- ontology actions (Action Types: typed transactional mutations)
CREATE TABLE ontology.actions (
    action_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    action_type text NOT NULL,                  -- e.g., 'medical.CreateEncounter'
    principal_kind text NOT NULL CHECK (principal_kind IN ('user', 'employee', 'system', 'llm', 'workflow')),
    principal_id uuid NULL,                     -- FK depends on principal_kind
    idempotency_key text UNIQUE,                -- per Bominal ADR-0107 idempotency contract
    input jsonb NOT NULL,
    output jsonb NULL,
    outcome text NOT NULL DEFAULT 'pending' CHECK (outcome IN ('pending', 'applied', 'failed', 'reversed')),
    failure_reason text NULL,
    audit_event_id uuid NULL,                   -- FK to audit-chain.audit_events
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL,
    duration_ms int NULL
);
ALTER TABLE ontology.actions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ontology.actions USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_actions_idempotency ON ontology.actions (tenant_id, action_type, idempotency_key);

-- ontology outbox (per outbox pattern; published to Kafka topic `ontology.<action_type>`)
CREATE TABLE ontology.outbox (
    outbox_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    topic text NOT NULL,
    key text NOT NULL,
    payload jsonb NOT NULL,
    published_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_outbox_unpublished ON ontology.outbox (created_at) WHERE published_at IS NULL;
```

### Rust port traits (oya-ontology-kernel)

```rust
#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn get(&self, tenant_id: TenantId, object_id: ObjectId) -> Result<Option<TypedObject>, OntologyError>;
    async fn put(&self, tenant_id: TenantId, action: TypedAction) -> Result<TypedObject, OntologyError>;
    async fn query(&self, tenant_id: TenantId, q: ObjectQuery) -> Result<Vec<TypedObject>, OntologyError>;
}

#[async_trait]
pub trait LinkStore: Send + Sync {
    async fn link(&self, tenant_id: TenantId, link: TypedLink) -> Result<LinkId, OntologyError>;
    async fn traverse(&self, tenant_id: TenantId, from: ObjectId, via: LinkType, depth: u8) -> Result<Vec<TypedLink>, OntologyError>;
}

#[async_trait]
pub trait ActionStore: Send + Sync {
    async fn apply(&self, tenant_id: TenantId, action: TypedAction) -> Result<ActionResult, OntologyError>;
    async fn reverse(&self, tenant_id: TenantId, action_id: ActionId) -> Result<(), OntologyError>;
}

pub trait OntologyFunction: Send + Sync {
    // Pure side-effect-free reads exposed to product surfaces + LLM tool gateway
    fn call(&self, tenant_id: TenantId, input: FunctionInput) -> Result<FunctionOutput, OntologyError>;
}

#[doc(hidden)]
pub trait Sealed {}  // per Bominal ADR-0101 sealed-port-trait pattern
```

### Cedar policy fragment

```cedar
entity Tenant;
entity Object in [Tenant] = { object_type: String, pillar: String };
entity Link   in [Tenant] = { link_type: String };
entity Action in [Tenant] = { action_type: String };
entity Principal in [Tenant];

action Read appliesTo {
    principal: Principal,
    resource: [Object, Link, Action]
};
action Apply appliesTo {
    principal: Principal,
    resource: Action
};

// Person-pillar prohibition: org-admin cannot read person-pillar objects
forbid (
    principal,
    action == Read,
    resource is Object
) when {
    principal.organization_admin_role &&
    resource.pillar == "person"
};
```

### Outbox event schema (Protobuf)

```proto
syntax = "proto3";
package ontology;

message ObjectMutated {
    string tenant_id = 1;
    string object_id = 2;
    string object_type = 3;
    int64  version = 4;
    bytes  payload_hash = 5;
    string action_type = 6;
    string action_id = 7;
    int64  timestamp_ms = 8;
}
```

## 2. Workflow (oya-workflow-*)

Workflow engine + Workflow Studio editor. Bominal ADR-0035/0103/0121/0148.

### Postgres DDL

```sql
-- workflow definitions (DAG + state machine spec)
CREATE TABLE workflow.definitions (
    workflow_def_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    definition jsonb NOT NULL,             -- nodes + edges + triggers + state-machine spec
    version int NOT NULL,                  -- versioned workflows; tenants can run any version
    status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'archived')),
    created_by uuid NOT NULL,
    published_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.definitions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.definitions USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_workflow_def_name_version ON workflow.definitions (tenant_id, name, version);

-- workflow runs (live executions)
CREATE TABLE workflow.runs (
    run_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    workflow_def_id uuid NOT NULL REFERENCES workflow.definitions(workflow_def_id),
    state text NOT NULL DEFAULT 'draft' CHECK (state IN (
        'draft', 'simulated', 'active', 'blocked', 'escalated',
        'reversed', 'failed', 'closed', 'archived'  -- Bominal 9-state model
    )),
    current_step text NULL,
    step_state jsonb NOT NULL DEFAULT '{}'::jsonb,  -- per-step state for determinism
    triggered_by_kind text NOT NULL CHECK (triggered_by_kind IN ('cron', 'webhook', 'event', 'ontology', 'manual', 'api')),
    triggered_by_id text NOT NULL,
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL,
    sla_due_at timestamptz NULL,
    duration_ms int NULL
);
ALTER TABLE workflow.runs FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.runs USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_runs_active ON workflow.runs (tenant_id, state, sla_due_at) WHERE state IN ('active', 'blocked', 'escalated');

-- workflow step history (append-only; deterministic replay per Temporal-like)
CREATE TABLE workflow.step_history (
    step_event_id bigserial PRIMARY KEY,
    tenant_id uuid NOT NULL,
    run_id uuid NOT NULL REFERENCES workflow.runs(run_id),
    step_name text NOT NULL,
    event_type text NOT NULL,   -- 'entered', 'completed', 'failed', 'retry', 'sla_breach'
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    occurred_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.step_history FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.step_history USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_step_history_run ON workflow.step_history (tenant_id, run_id, step_event_id);

-- workflow triggers (cron / webhook / event subscriptions)
CREATE TABLE workflow.triggers (
    trigger_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    workflow_def_id uuid NOT NULL REFERENCES workflow.definitions(workflow_def_id),
    trigger_type text NOT NULL CHECK (trigger_type IN ('cron', 'webhook', 'event', 'ontology', 'manual', 'api')),
    config jsonb NOT NULL,
    enabled bool NOT NULL DEFAULT true,
    last_fired_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE workflow.triggers FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON workflow.triggers USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- workflow outbox
CREATE TABLE workflow.outbox (LIKE ontology.outbox INCLUDING ALL);
```

### Rust port traits (oya-workflow-engine-kernel)

```rust
#[async_trait]
pub trait WorkflowStateStore: Send + Sync {
    async fn create_run(&self, tenant_id: TenantId, def_id: WorkflowDefId, trigger: TriggerSource) -> Result<RunId, WorkflowError>;
    async fn record_step(&self, tenant_id: TenantId, run_id: RunId, event: StepEvent) -> Result<(), WorkflowError>;
    async fn replay_run(&self, tenant_id: TenantId, run_id: RunId) -> Result<RunState, WorkflowError>;
}

#[async_trait]
pub trait TransitionEngine: Send + Sync {
    async fn evaluate(&self, run: &RunState, def: &WorkflowDefinition, event: WorkflowEvent) -> Result<Vec<Action>, WorkflowError>;
}

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, topic: Topic, key: String, payload: Bytes) -> Result<(), WorkflowError>;
    async fn subscribe(&self, topic: Topic, handler: BoxedHandler) -> Result<(), WorkflowError>;
}

#[async_trait]
pub trait AutomationRunner: Send + Sync {
    async fn run(&self, run_id: RunId, step: AutomationStep) -> Result<StepOutput, WorkflowError>;
}
```

## 3. Identity (oya-identity-*)

Per Bominal ADR-0125 naming canon. Distinct entities: Tenant ≠ Organization ≠
User ≠ Person ≠ Employee.

### Postgres DDL

```sql
-- users (auth principal)
CREATE TABLE identity.users (
    user_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email citext UNIQUE,
    phone citext UNIQUE,
    password_hash text NULL,           -- argon2id; NULL for passwordless-only
    mfa_enrolled bool NOT NULL DEFAULT false,
    mfa_methods jsonb NOT NULL DEFAULT '[]',
    passkey_credentials jsonb NOT NULL DEFAULT '[]',  -- WebAuthn credentials
    locked_at timestamptz NULL,
    last_login_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- persons (human record; may exist without User account)
CREATE TABLE identity.persons (
    person_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    legal_name jsonb NOT NULL,         -- per-jurisdiction structured name
    dob date NULL,
    nationality text NULL,
    pillar text NOT NULL DEFAULT 'person' CHECK (pillar = 'person'),
    user_id uuid NULL UNIQUE REFERENCES identity.users(user_id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- organizations (legal entity inside tenant)
CREATE TABLE identity.organizations (
    organization_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    parent_organization_id uuid NULL REFERENCES identity.organizations(organization_id),
    display_name text NOT NULL,
    kr_entity_kind text NULL CHECK (kr_entity_kind IN ('개인사업자', '법인', '비영리법인', '공공기관')),
    tier text NULL CHECK (tier IN ('5인미만', 'SME', '중견', '대기업', '공공')),
    pillar text NOT NULL DEFAULT 'org' CHECK (pillar = 'org'),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE identity.organizations FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON identity.organizations USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- employees (Person × Organization × Active Employment)
CREATE TABLE identity.employees (
    employee_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    person_id uuid NOT NULL REFERENCES identity.persons(person_id),
    organization_id uuid NOT NULL REFERENCES identity.organizations(organization_id),
    active bool NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE identity.employees FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON identity.employees USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_employees_person_org ON identity.employees (tenant_id, person_id, organization_id);

-- employments (effective-dated role snapshots)
CREATE TABLE identity.employments (
    employment_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    employee_id uuid NOT NULL REFERENCES identity.employees(employee_id),
    classification text NOT NULL CHECK (classification IN (
        '정규직', '계약직', '단시간근로자', '파견', '도급', '프리랜서', '인턴', '임원'  -- Bominal ADR-0126
    )),
    title text NOT NULL,
    department text NULL,
    manager_employee_id uuid NULL REFERENCES identity.employees(employee_id),
    fte numeric(4,3) NOT NULL DEFAULT 1.000 CHECK (fte > 0 AND fte <= 1.000),
    effective_from date NOT NULL,
    effective_to date NULL,
    statute_citations jsonb NOT NULL DEFAULT '[]',  -- ADR-0190 corpus citations
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE identity.employments FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON identity.employments USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_employments_active ON identity.employments (tenant_id, employee_id, effective_from DESC) WHERE effective_to IS NULL;

-- sessions
CREATE TABLE identity.sessions (
    session_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES identity.users(user_id),
    tenant_id uuid NULL,                    -- NULL during cross-tenant SSO
    impersonator_user_id uuid NULL,         -- 4-eyes audit for admin impersonation
    issued_at timestamptz NOT NULL DEFAULT now(),
    idle_timeout_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    mfa_satisfied_at timestamptz NULL,
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    user_agent text NULL,
    ip_address inet NULL
);
CREATE INDEX idx_sessions_user_active ON identity.sessions (user_id, expires_at) WHERE expires_at > now();
```

### Rust port traits (oya-identity-kernel)

```rust
#[async_trait]
pub trait UserStore: Send + Sync {
    async fn get(&self, user_id: UserId) -> Result<Option<User>, IdentityError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, IdentityError>;
    async fn register(&self, draft: UserDraft) -> Result<User, IdentityError>;
}

#[async_trait]
pub trait PersonStore: Send + Sync { /* CRUD on Person */ }

#[async_trait]
pub trait OrganizationStore: Send + Sync { /* CRUD; tenant-scoped */ }

#[async_trait]
pub trait EmployeeStore: Send + Sync {
    async fn hire(&self, tenant_id: TenantId, person_id: PersonId, org_id: OrganizationId, employment: EmploymentDraft) -> Result<Employee, IdentityError>;
    async fn terminate(&self, tenant_id: TenantId, employee_id: EmployeeId, effective_date: NaiveDate) -> Result<(), IdentityError>;
    async fn active_for(&self, tenant_id: TenantId, person_id: PersonId) -> Result<Vec<Employee>, IdentityError>;
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, user_id: UserId, mfa_satisfied: bool) -> Result<Session, IdentityError>;
    async fn validate(&self, session_id: SessionId) -> Result<Session, IdentityError>;
    async fn revoke(&self, session_id: SessionId) -> Result<(), IdentityError>;
}

#[async_trait]
pub trait AuthChallenger: Send + Sync {
    async fn challenge(&self, user: &User, method: ChallengeMethod) -> Result<ChallengeToken, IdentityError>;
    async fn verify(&self, token: ChallengeToken, response: ChallengeResponse) -> Result<(), IdentityError>;
}
```

## 4. Tenancy (oya-tenancy-*)

The product-enablement substrate. Tenants enable µservices à-la-carte.
Bominal ADR-0018 RLS posture + ADR-0009 cell architecture inherited.

### Postgres DDL

```sql
CREATE TABLE tenancy.tenants (
    tenant_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name text NOT NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'terminated')),
    tier text NOT NULL DEFAULT 'starter' CHECK (tier IN ('starter', 'pro', 'enterprise')),
    region text NOT NULL,                    -- KR / US / EU / JP
    primary_jurisdiction text NOT NULL,      -- 'KR' default
    created_at timestamptz NOT NULL DEFAULT now(),
    activated_at timestamptz NULL,
    suspended_at timestamptz NULL
);

CREATE TABLE tenancy.tenant_products (
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants(tenant_id),
    microservice text NOT NULL,               -- 'hr', 'payroll', 'medical', 'connect', ...
    enabled bool NOT NULL DEFAULT false,
    enabled_at timestamptz NULL,
    disabled_at timestamptz NULL,
    tier_limits jsonb NOT NULL DEFAULT '{}'::jsonb,   -- per-µservice quotas
    config jsonb NOT NULL DEFAULT '{}'::jsonb,        -- per-µservice tenant config
    PRIMARY KEY (tenant_id, microservice)
);

CREATE TABLE tenancy.tenant_cells (
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants(tenant_id),
    cell_id text NOT NULL,                    -- e.g., 'kr-1-cell-a'
    region text NOT NULL,
    is_primary bool NOT NULL DEFAULT false,
    PRIMARY KEY (tenant_id, cell_id)
);
CREATE UNIQUE INDEX idx_tenant_primary_cell ON tenancy.tenant_cells (tenant_id) WHERE is_primary = true;

CREATE TABLE tenancy.tenant_admins (
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants(tenant_id),
    user_id uuid NOT NULL REFERENCES identity.users(user_id),
    role text NOT NULL CHECK (role IN ('owner', 'admin', 'billing_admin', 'security_admin')),
    invited_by uuid NOT NULL REFERENCES identity.users(user_id),
    invited_at timestamptz NOT NULL DEFAULT now(),
    accepted_at timestamptz NULL,
    PRIMARY KEY (tenant_id, user_id)
);

-- Session-setting function used by all RLS-protected µservices
CREATE FUNCTION oyatie.set_current_tenant(p_tenant_id uuid) RETURNS void AS $$
BEGIN
    PERFORM set_config('oyatie.tenant_id', p_tenant_id::text, true);  -- LOCAL to txn
END $$ LANGUAGE plpgsql;
```

### Rust port traits (oya-tenancy-kernel)

```rust
#[async_trait]
pub trait TenantStore: Send + Sync {
    async fn create(&self, draft: TenantDraft, owner_user_id: UserId) -> Result<Tenant, TenancyError>;
    async fn suspend(&self, tenant_id: TenantId, reason: SuspensionReason) -> Result<(), TenancyError>;
}

#[async_trait]
pub trait TenantProductRegistry: Send + Sync {
    async fn enable(&self, tenant_id: TenantId, microservice: &str, by: UserId) -> Result<(), TenancyError>;
    async fn disable(&self, tenant_id: TenantId, microservice: &str, by: UserId) -> Result<(), TenancyError>;
    async fn enabled(&self, tenant_id: TenantId) -> Result<Vec<MicroserviceId>, TenancyError>;
    async fn is_enabled(&self, tenant_id: TenantId, microservice: &str) -> Result<bool, TenancyError>;
}

#[async_trait]
pub trait TenantCellPlacer: Send + Sync {
    async fn assign(&self, tenant_id: TenantId, region: Region) -> Result<CellId, TenancyError>;
    async fn primary_cell_for(&self, tenant_id: TenantId) -> Result<CellId, TenancyError>;
}
```

## 5. Audit-chain (oya-audit-chain-*)

Cryptographically auditable record per Bominal ADR-0028.

### Postgres DDL

```sql
CREATE TABLE audit_chain.audit_events (
    event_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    period_date date NOT NULL,
    event_type text NOT NULL,                 -- e.g., 'hr.employee_hired', 'payroll.run_closed'
    payload jsonb NOT NULL,
    payload_hash bytea NOT NULL,              -- SHA-256(canonical_json(payload))
    prev_event_id uuid NULL,
    occurred_at timestamptz NOT NULL DEFAULT now(),
    sealed_segment_id uuid NULL               -- FK to audit_segments after seal
) PARTITION BY RANGE (occurred_at);
-- partitioning is per (tenant_id, period_date) via Citus distribution; Postgres native range partition by month

-- Trigger to enforce append-only invariant
CREATE OR REPLACE FUNCTION audit_chain.deny_modification() RETURNS trigger AS $$
BEGIN RAISE EXCEPTION 'audit_events is append-only'; END $$ LANGUAGE plpgsql;
CREATE TRIGGER no_update BEFORE UPDATE ON audit_chain.audit_events FOR EACH ROW EXECUTE FUNCTION audit_chain.deny_modification();
CREATE TRIGGER no_delete BEFORE DELETE ON audit_chain.audit_events FOR EACH ROW EXECUTE FUNCTION audit_chain.deny_modification();

CREATE TABLE audit_chain.audit_segments (
    segment_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    period_date date NOT NULL,
    event_count int NOT NULL,
    merkle_root bytea NOT NULL,
    signature bytea NOT NULL,                 -- Ed25519 over: tenant_id || merkle_root || period_date || prev_segment_root
    prev_segment_root bytea NULL,
    signing_key_id uuid NOT NULL,
    sealed_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX idx_segments_per_period ON audit_chain.audit_segments (tenant_id, period_date);

CREATE TABLE audit_chain.signing_keys (
    key_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    public_key bytea NOT NULL,
    kms_arn text NULL,                        -- KMS-backed in production per Bominal ADR-0028
    created_at timestamptz NOT NULL DEFAULT now(),
    rotated_at timestamptz NULL,
    revoked_at timestamptz NULL
);
```

### Rust port traits (oya-audit-chain-kernel)

```rust
#[async_trait]
pub trait AuditEventStore: Send + Sync {
    async fn append(&self, tenant_id: TenantId, event: AuditEvent) -> Result<EventId, AuditError>;
    async fn fetch_unsealed(&self, tenant_id: TenantId, period: NaiveDate) -> Result<Vec<AuditEvent>, AuditError>;
}

#[async_trait]
pub trait AuditSegmentSealer: Send + Sync {
    async fn seal_period(&self, tenant_id: TenantId, period: NaiveDate) -> Result<AuditSegment, AuditError>;
    async fn verify_segment(&self, segment: &AuditSegment) -> Result<(), AuditError>;
}

pub trait MerkleTreeBuilder: Send + Sync {
    fn build_root(&self, leaves: &[Sha256Hash]) -> Sha256Hash;
}

#[async_trait]
pub trait ChainSigner: Send + Sync {
    async fn sign(&self, key_id: KeyId, preimage: &[u8]) -> Result<Ed25519Signature, AuditError>;
    async fn verify(&self, key_id: KeyId, preimage: &[u8], sig: &Ed25519Signature) -> Result<bool, AuditError>;
}
```

## 6-N. Remaining substrate µservices (outlined; full schemas in Wave 2)

The pattern from §1-§5 applies. Outline only:

- **Eventing** (`oya-eventing-*`): outbox dispatcher; Kafka KRaft topics per Bominal ADR-0116; CloudEvents framing; per-tenant per-cell partitioning. Tables: `eventing.topics`, `eventing.subscriptions`, `eventing.delivery_log`.
- **Secrets** (`oya-secrets-*`): SecretReference port with OpenBao adapter (day-1 default per user preference) + HSM-per-cell production. Tables: `secrets.refs` (just references, never values).
- **Observability** (`oya-observability-*`): OpenTelemetry SDK + VictoriaMetrics + structured JSON logs. Per Bominal ADR-0042. Tables: `obs.traces` (partitioned by tenant + day) and metric pipeline; mostly external storage.
- **KMS** (`oya-kms-*`): envelope encryption; per-tenant DEK; per-cell HSM. Tables: `kms.keys`, `kms.key_versions`.
- **Policy** (`oya-policy-*`): Cedar engine; per-tenant rule packs. Tables: `policy.tenant_rule_packs`, `policy.evaluation_log` (for audit).
- **Search** (`oya-search-*`): pgroonga + Tantivy; KR morphology via mecab-ko/khaiii FFI. Tables: `search.index_state` (Tantivy index versioning).
- **Vector** (`oya-vector-*`): pgvector day-1; in-house HNSW long-horizon. Tables: `vector.embeddings` (per-tenant; per-object-type).
- **Data-Use-Boundary** (`oya-data-boundary-*`): 12 data classes per oyatie ADR-0008; runtime enforcement of HARD_DENY (PHI/PCI/PIPA/children). Cedar policy + audit.
- **Finance-library** (`oya-finance-*`): per Bominal ADR-0120 platform-finance-library translated. Money + CurrencyCode + JournalEntry (debits = credits invariant at construction). No persistence in this µservice; consumed as a library.
- **Capability-registry** (`oya-capability-*`): MCP-compatible discovery; per-tenant endpoint per Bominal ADR-0021. Tables: `capability.endpoints`, `capability.bindings`.
- **Records** (`oya-records-*`): FHIR R5 canonical (Bominal ADR-0016). Tables: `records.encounters`, `records.observations`, `records.medications`, etc. — Healthcare-specific (defers to M04+ but kernel + ports defined in M02).
- **Application** (`oya-application-*`): the B2B unified shell. Tables: `application.tenant_dashboards`, `application.product_navigation_overrides`, `application.branding` (per-tenant theme + logo).
- **Cloud-tenancy/IAM/KMS/Storage/Compute/Network/Billing** (`oya-cloud-*`): the multi-tenant runtime substrate. Tables per-BC; Bominal ADR-0117 cloud-native infrastructure inherited.

## Cross-cutting

- **Migration tool**: `oya-shared-migrate-cli` runs all `migrations/<microservice>/V###__*.sql` files in dependency order. Per Bominal pattern.
- **RLS bootstrap**: every request handler at the boundary calls `oyatie.set_current_tenant(extracted_tenant_id)` before any query.
- **Idempotency**: every state-changing API accepts an `Idempotency-Key` header per RFC draft + Stripe pattern.
- **Tenant-id propagation**: gRPC metadata + HTTP `X-Tenant-Id` header; signed via JWT claim; verified at boundary.

## Wave 2 expansion plan

Each numbered substrate (§1-§5 in detail; §6-N outlined) becomes a Phase
SPEC + Impl Plan in M02. Wave 2 executors:
- Take the schema fragment here as starting point
- Author full DDL with indexes + RLS + triggers + outbox
- Author full port-trait Rust code with sealed-trait marker
- Author full Cedar policy fragment
- Author full migration file
- Author full Protobuf event schema
- Author full OpenAPI/gRPC contract
- Author load-test script per Performance Targets

This document is the foundation that lets Wave 2 executors fan out in
parallel without re-deriving fundamentals each time.
