---
doc_class: Template
template_id: TPL-IMPL
status: Accepted
date: 2026-05-13
purpose: |
  Canonical Implementation Plan shape for every IP under
  .omc/plans/milestones/M0X-slug/phases/P0Y-slug/<impl-plan-name>.md.
  An autonomous executor reading this plan must be able to act without escalation:
  concrete file targets, crate names with BNF v4.1 justification, code skeletons,
enforcing_fitness_lane: governance-plan-hierarchy
owner_team: council-architecture
related:
  - docs/templates/phase-spec-template.md
  - docs/templates/milestone-readme-template.md
  - docs/templates/INDEX.md
adrs_cited:
  - ADR-0054  # scaffold-claim pattern
  - ADR-0056  # BNF v4.1 + layer enum
doc_status: published
---

```yaml
# Required frontmatter for every implementation plan file
---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M0X-<slug>
phase: P0Y-<slug>
impl_plan_id: <impl-plan-name>    # e.g. IP-001-hr-domain-scaffold
status: pending | open | in-progress | merged | blocked
execution_unit: ChangeSet      # every IP is a ChangeSet-sized execution unit
changeset_contract: claimable-verifiable-bundleable-promotable
owner: <team-id>
blocked_by:
  - impl_plan: <impl-plan-name>   # or omit if unblocked
    reason: "<one line>"
acceptance_lanes:
  - buck2-build
  - buck2-test
  - pipeline-static-analysis
  - pipeline-supply-chain
  - pipeline-formatting
  - lean-a1
  - lean-a2
  - lean-a3
  - lean-a4
---
```

# <impl-plan-name>: <one-line title, present tense>

## Intent

One to two sentences. What this implementation plan delivers. Present tense;
durable outcome framing. Example: "Scaffolds the `hr-employee-domain` crate
with Employment entity, EmployeeRepository port-trait, and unit tests. Establishes
the domain boundary for the HR µservice's Employee BC."

---


## ChangeSet boundary

State why this IP is one cohesive ChangeSet: exact issue-level scope, affected symbols/artifact pointers, affected crates/packages/deployables, required tests, evidence bundle, and promotion boundary. If the work cannot be claimed, verified, bundled, and promoted independently, split it before execution. Whole-tree locks or full-workspace cold builds require explicit graph-proven rationale.

## Concrete File Targets

Full paths. For existing files: which lines change. For new files: expected
content shape. An executor reads this list and touches ONLY these files.

| Path | Action | Description |
|---|---|---|
| `crates/oyatie-<ms>-<bc>-<layer>/Cargo.toml` | create | `[package]` + `[dependencies]` skeleton |
| `crates/oyatie-<ms>-<bc>-<layer>/src/lib.rs` | create | module declarations + `pub use` surface |
| `crates/oyatie-<ms>-<bc>-<layer>/src/<module>.rs` | create | trait / entity / use-case body |
| `Cargo.toml` | update | add `crates/oyatie-<ms>-<bc>-<layer>` to `[workspace.members]` |
| `docs/standards/bounded-contexts.md` | update | register new BC rows |

---

## Crate Naming

For EACH new crate or binary introduced, include the full justification block
(mandatory per `feedback_naming_justification.md`; unjustified names are
rejected at scaffold time):

```
NAME: oyatie-<microservice>[-<bc-tokens>]-<layer>
JUSTIFICATION:
- microservice = <kebab-token(s)>: <product/capability; registered in
  [workspace.metadata.oyatie.microservices]; ADR-0056 v4.1 flat BNF; no
  shared|vertical bisection>
- bc-tokens = <kebab-token(s)> (OPTIONAL): <omit if µservice has single
  concept at this layer; include if multiple BCs or binaries split at same
  layer; ADR-0056 v4.1 BC-optionality rule>
- layer = <layer>: <12-enum value; ADR-0056 §"Layer semantics"; e.g.
  "port-traits + entity types → domain",
  "use-case orchestrators holding port bounds → application",
  "framework/driver impl → infrastructure",
  "composition-root binary → app",
  "REST handler wiring → rest",
  "gRPC handler wiring → grpc">
- exemptions claimed: <none | cite ADR-0056 line>
```

---

## Code Shape

Skeleton for each new file. Trait signatures, data types, module structure.
No implementation detail required — just enough for an executor to understand
the shape and fill it without escalation.

### `crates/oyatie-<ms>-<bc>-<layer>/src/lib.rs`

```rust
// Module declarations
pub mod <module>;

// Public surface re-exports
pub use <module>::{<Type>, <Trait>};
```

### `crates/oyatie-<ms>-<bc>-<layer>/src/<module>.rs`

```rust
use std::error::Error;

/// <one-line doc comment>
pub struct <EntityName> {
    pub id: <IdType>,
    // <fields>
}

/// Port trait — implemented in infrastructure layer
#[async_trait::async_trait]
pub trait <RepositoryTrait>: Send + Sync {
    async fn find_by_id(&self, id: &<IdType>) -> Result<<EntityName>, Box<dyn Error + Send + Sync>>;
    async fn save(&self, entity: &<EntityName>) -> Result<(), Box<dyn Error + Send + Sync>>;
}
```

### `crates/oyatie-<ms>-<bc>-<layer>/Cargo.toml`

```toml
[package]
name = "oyatie-<ms>-<bc>-<layer>"
version.workspace = true
edition.workspace = true

[dependencies]
# kernel deps only at domain layer; no framework crates
async-trait = { workspace = true }
serde = { workspace = true, features = ["derive"] }

[dev-dependencies]
tokio = { workspace = true, features = ["full", "test-util"] }
```

---

## Acceptance Gates

Per-lane commands + exit-0 expectations. Run in order; stop at first failure.

```bash
# 1. Build
buck2 build <crate-or-app-target>                         # exit 0

# 2. Tests
buck2 test <targeted-test-targets>                        # exit 0; 0 failures

# 3. Static/lint gate packets
buck2 test <relevant-pipeline-static-analysis-targets>    # exit 0

# 4. Supply chain
buck2 test <supply-chain-pipeline-target>                 # exit 0

# 5. Docs/API contract checks
buck2 test <docs-or-api-contract-pipeline-targets>        # exit 0

# 6. LEAN / architecture checks (per ADR-0057)
buck2 test <pipeline-lean-a1-target>                     # layer ordering
buck2 test <pipeline-lean-a2-target>                     # cross-vertical refusal
buck2 test <pipeline-lean-a3-target>                     # BC boundary
buck2 test <pipeline-lean-a4-target>                     # naming conformance
```

---

## Test Plan

### Unit tests

Location: `crates/oyatie-<ms>-<bc>-<layer>/src/<module>.rs #[cfg(test)]`

| Test name | What it verifies |
|---|---|
| `test_<entity>_create` | Entity construction; invariants hold |
| `test_<repository>_round_trip` | Port-trait mock; save → find_by_id returns same entity |

### Integration tests

Location: `crates/oyatie-<ms>-<bc>-<layer>/tests/<test_file>.rs`

| Test name | What it verifies |
|---|---|
| `integration_<bc>_<scenario>` | Full use-case path; DB adapter wired |

### E2E / acceptance tests

| Scenario | Command | Expected output |
|---|---|---|
| `<scenario>` | `buck2 test <acceptance-test-target>` | `PASS; 0 failures` |

---

## Clean Architecture Compliance

Declare layer for every new crate this IP scaffolds. Executor must verify
before marking IP in-progress (per `feedback_clean_architecture_requirements.md`).

### Dependency direction check

```
kernel  ←  domain  ←  application  ←  adapter  ←  {rest, grpc, worker}  ←  app
```

For each new crate, state its layer and list the layers it imports:

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oyatie-<ms>-<bc>-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oyatie-<ms>-<bc>-domain` | `domain` | `kernel` | `application`, `adapter`, `infrastructure`, presentation, `app` |
| `oyatie-<ms>-<bc>-application` | `application` | `domain`, `kernel` | `adapter`, `infrastructure`, presentation, `app` |
| `oyatie-<ms>-<bc>-adapter` | `adapter` | `application`, `domain`, `kernel` | `infrastructure`, presentation, `app` |
| `oyatie-<ms>-<bc>-rest` | `rest` | `application`, `domain`, `kernel` | `adapter`, `infrastructure` directly |

### Port traits (must live in kernel)

Every port trait this IP introduces — zero business logic, zero I/O:

```rust
// oyatie-<ms>-<bc>-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait <PortName>: Send + Sync + sealed::Sealed {
    // Only method signatures here. No logic.
    async fn <method>(&self, id: &<IdType>) -> Result<<ReturnType>, Box<dyn std::error::Error + Send + Sync>>;
}
```

Implementations live exclusively in `oyatie-<ms>-<bc>-adapter`. Domain calls
through the trait; domain never imports the adapter.

### Cross-product integration check

This IP introduces NO direct imports between product µservices. Any
cross-product data flow uses:
- Workflow events (action/orchestration) — list event types: `<EventType>`
- Ontology reads/writes (information) — list Object Types: `<ObjectType>`

If this IP is for a product µservice: confirm the Buck2/pipeline lean-a2 gate will pass by design (no product crate deps in `[dependencies]`).

### CI lanes this IP must green

```bash
buck2 test <pipeline-dependency-direction-target>          # dependency-direction
buck2 test <pipeline-cross-product-refusal-target>         # cross-product-refusal
buck2 test <pipeline-port-location-target>                 # ports in kernel
buck2 test <pipeline-layer-correctness-target>             # layer correctness
```

---

## Load Test

Mandatory per `feedback_quality_performance_scalability_bar.md`. Must pass
before IP merges to main. Targets inherited from the µservice's PRD
§"Performance Targets".

### k6 smoke test (run in CI on every PR)

```javascript
// tests/load/smoke-<impl-plan-name>.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 10,
  duration: '30s',
  thresholds: {
    http_req_duration: ['p(99)<200'],  // p99 ≤200ms
    http_req_failed: ['rate<0.001'],    // error rate <0.1%
  },
};

export default function () {
  const res = http.get(`${__ENV.BASE_URL}/<endpoint>`);
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(0.1);
}
```

Run: `k6 run tests/load/smoke-<impl-plan-name>.js --env BASE_URL=http://localhost:<port>`

Expected: all thresholds green; exit 0.

### Load test (run against staging before merge)

```bash
# vegeta: sustained load
echo "GET http://staging.<svc>/<endpoint>" \
  | vegeta attack -rate=1000/s -duration=60s \
  | vegeta report
# Pass criteria: p99 ≤200ms; p999 ≤500ms; success rate ≥99.9%
```

### Throughput target verification

| Scenario | Tool | Target | Pass criterion |
|---|---|---|---|
| Read `<endpoint>` | k6 | p99 ≤200ms at 1k RPS | `http_req_duration{p(99)}<200` |
| Write `<endpoint>` | k6 | p99 ≤200ms at 500 RPS | `http_req_duration{p(99)}<200` |
| Sustained load | vegeta | 0 errors at 10k RPS (cell baseline) | success_rate=100% |

---


Claim these symbols BEFORE beginning work (per `feedback_grit_claim_work_done.md`):

```bash
  --agent <agent-id> \
  --intent "<impl-plan-name>: <one-line intent>" \
  --ttl 3600 \
  crates/oyatie-<ms>-<bc>-<layer>/src/lib.rs::<TypeOrTrait> \
  crates/oyatie-<ms>-<bc>-<layer>/src/<module>.rs::<fn_name>
```



---


Emit at IP completion (mandatory per `CLAUDE.md §Store — MANDATORY triggers`):

```bash
  -t context-oyatie \
  -c "<impl-plan-name> merged at <git-sha>; crates scaffolded: <list>;
      next IP: <impl-plan-name+1>" \
  -i high \
  -k "M0X,P0Y,<impl-plan-name>,<µservice>"
```

---

## Halt Conditions

Stop work and escalate to architect agent if ANY of the following occur:

1. `buck2 build <touched-build-targets>` or `buck2 test <targeted-test-targets>` fails after 3 attempts with the same error.
2. A LEAN-A2 (cross-vertical refusal) violation cannot be resolved by moving
   code — indicates a design boundary error; requires ADR amendment.
3. A new crate name cannot satisfy BNF v4.1 justification — do not land an
   unjustified name; escalate.
5. Any acceptance gate exits non-zero after fix attempts and the root cause is
   unclear — do not mask with test-specific hacks.

---

## Next IP Pointer

`<next-impl-plan-name>.md` (or `phases/P0Z-<slug>/IP-001-<slug>.md` if phase
boundary). Cite the exact relative file path.

---

## Cross-References

- Phase spec: `../README.md` (or `../INDEX.md`)
- Milestone README: `../../README.md`
- PRD: `docs/prds/<µservice>.md`
- ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim), ADR-0056 (BNF v4.1)
- Memory: `feedback_naming_justification.md`, `feedback_grit_claim_work_done.md`
