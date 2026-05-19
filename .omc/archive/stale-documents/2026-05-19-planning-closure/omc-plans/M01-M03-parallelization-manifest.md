---
purpose: Auto-backfilled purpose for M01-M03-parallelization-manifest.md
---

---
title: M01-M03 Parallelization Manifest
status: canonical
authority: feedback_autonomous_decision_principles.md + feedback_milestone_phase_hierarchy.md
authored: 2026-05-13
purpose: |
  Authoritative dependency DAG + parallel-wave assignment + critical-path
  call-out + per-wave grit claim symbol space + concrete dispatch script for
  M01-M03 autonomous execution. Used by orchestrator agents to fire
  non-overlapping executors in dependency-aware waves.
phase_count:
  M01: 11  # 6 foundational (P01-P06, all EXISTS/complete) + 5 operational BNF-cutover (PShard0-PShard4)
  M02: 22  # P01-P22 in .omc/plans/milestones/M02b-substrate/phases/
  M03: 8   # P01-P08 in .omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/
  total: 41 # (35 operational + 6 M01 foundational completed)
grit_session_note: |
  grit session start failed (no registered symbol scope for doc-only work).
  Fallback: ICM scaffold-locks-oyatie topic used per ADR-0054 §"scaffold-claim pattern".
  Doc-only manifest; no code symbols claimed.
---

# M01-M03 Parallelization Manifest

## 1. Phase Master Table

All phases in dependency order. `parallel_wave` encodes which phases may fire simultaneously.

### M01 — v4 BNF Cutover

**Foundational phases** (directory: `.omc/plans/milestones/M01-foundation/phases/`). These shipped as pre-work and are treated as COMPLETED gates for M01 operational phases.

| Phase ID       | Slug                          | depends_on    | unblocks             | parallel_wave | critical_path | status    | est_hours |
|----------------|-------------------------------|---------------|----------------------|---------------|---------------|-----------|-----------|
| M01-F01        | data-use-boundary-tenancy     | —             | M01-F02,F03,F04,F05  | M01.F0        | false         | complete  | 0         |
| M01-F02        | identity-cedar                | M01-F01       | M01-F05,F06          | M01.F0        | false         | complete  | 0         |
| M01-F03        | audit-chain-evidence          | M01-F01       | M01-F05,F06          | M01.F0        | false         | complete  | 0         |
| M01-F04        | eventing-object-graph         | M01-F01       | M01-F05,F06          | M01.F0        | false         | complete  | 0         |
| M01-F05        | cell-plane                    | M01-F01..F04  | M01-P02              | M01.F0        | true          | complete  | 0         |
| M01-F06        | regional-pack-flattening      | M01-F01..F04  | M01-P02              | M01.F0        | false         | complete  | 0         |

**Operational phases** (5 BNF-cutover phases per MASTERPLAN §4):

| Phase ID       | Slug                          | depends_on              | unblocks              | parallel_wave | critical_path | status       | est_hours |
|----------------|-------------------------------|-------------------------|-----------------------|---------------|---------------|--------------|-----------|
| M01-P01        | shard0-bnf-rename-landed      | M01-F01..F06            | M01-P02               | M01.W0        | true          | complete      | 0         |
| M01-P02        | shard1-atomic-rename-114-rows | M01-P01                 | M01-P03,P04           | M01.W1        | true          | queued        | 4         |
| M01-P03        | shard1.5-deferred-26-rows     | M01-P02                 | M01-P04               | M01.W2        | true          | proposed      | 2         |
| M01-P04        | iter4-src-inspection          | M01-P02,P03             | M01-P05               | M01.W3        | true          | proposed      | 3         |
| M01-P05        | post-cutover-lean-hardening   | M01-P04                 | M02.W1 (all)          | M01.W4        | true          | proposed      | 2         |

**Path notes:**
- `M01-P02` requires regenerating TSV with `--bnf-version v4.1` flag in `xtask-metadata-augment` (114-row atomic rename).
- `M01-P05` flips LEAN checks from `--report-only` to BLOCKER; this is the M01 exit gate and the prerequisite for all M02.W1 dispatches.

---

### M02 — Substrate Ready

**Directory:** `.omc/plans/milestones/M02b-substrate/phases/`

Dependency logic: P01-P11 are independent substrate µservices that can all fan out in parallel after M01 exits. P12-P16 require the foundational substrate (P01-P11) to be at least structurally established (kernel crates + DDL merged). P17-P19 require P12-P16 complete (workflow engine, tenancy, policy, data-boundary, records). P20 is a CI lane gate running in parallel with P17-P19. P21 serializes on P17-P20. P22 serializes on P21.

| Phase ID  | Slug                         | depends_on                                          | unblocks                         | parallel_wave | critical_path | est_hours |
|-----------|------------------------------|-----------------------------------------------------|----------------------------------|---------------|---------------|-----------|
| M02-P01   | foundry-engine-consolidation | M01-P05                                             | M02-P19,P20                      | M02.W1        | false         | 6         |
| M02-P02   | ontology                     | M01-P05                                             | M02-P12,P17,P19                  | M02.W1        | true          | 8         |
| M02-P03   | identity                     | M01-P05                                             | M02-P12,P13,P19                  | M02.W1        | false         | 6         |
| M02-P04   | audit-chain                  | M01-P05                                             | M02-P12,P15,P19                  | M02.W1        | false         | 5         |
| M02-P05   | eventing                     | M01-P05                                             | M02-P12,P13,P19                  | M02.W1        | false         | 5         |
| M02-P06   | secrets                      | M01-P05                                             | M02-P08,P13,P19                  | M02.W1        | false         | 4         |
| M02-P07   | observability                | M01-P05                                             | M02-P20,P21                      | M02.W1        | false         | 4         |
| M02-P08   | kms                          | M01-P05, M02-P06                                    | M02-P13,P15,P18,P19              | M02.W1        | false         | 5         |
| M02-P09   | search                       | M01-P05                                             | M02-P19,P21                      | M02.W1        | false         | 5         |
| M02-P10   | vector                       | M01-P05                                             | M02-P19,P21                      | M02.W1        | false         | 4         |
| M02-P11   | finance-library              | M01-P05                                             | M02-P19,P21                      | M02.W1        | false         | 3         |
| M02-P12   | workflow-engine              | M02-P02,P03,P04,P05                                 | M02-P17,P19,P21                  | M02.W2        | true          | 10        |
| M02-P13   | tenancy                      | M02-P03,P05,P06,P08                                 | M02-P18,P19,P21                  | M02.W2        | false         | 6         |
| M02-P14   | policy                       | M01-P05                                             | M02-P17,P18,P19,P21              | M02.W2        | false         | 5         |
| M02-P15   | data-boundary                | M02-P04,P08                                         | M02-P17,P19,P21                  | M02.W2        | false         | 5         |
| M02-P16   | records                      | M01-P05                                             | M02-P17,P19,P21                  | M02.W2        | false         | 6         |
| M02-P17   | capability-registry          | M02-P12,P14,P15,P16                                 | M02-P19,P21                      | M02.W3        | false         | 5         |
| M02-P18   | cloud-tenancy                | M02-P08,P13,P14                                     | M02-P19,P21                      | M02.W3        | false         | 8         |
| M02-P19   | application                  | M02-P01,P02,P03,P12,P13,P14,P15,P16,P17,P18        | M02-P21, M03.W1                  | M02.W3        | true          | 10        |
| M02-P20   | ci-lanes-operational         | M02-P01,P07                                         | M02-P21                          | M02.W4        | false         | 4         |
| M02-P21   | architecture-planes-green    | M02-P17,P18,P19,P20                                 | M02-P22                          | M02.W5        | true          | 3         |
| M02-P22   | m02-exit-gate                | M02-P21                                             | M03.W1 (all)                     | M02.W6        | true          | 1         |

**Wave note:** M02-P08 (kms) depends on M02-P06 (secrets) — both can start in W1 if secrets is submitted first or if P06 is treated as a sub-dependency within W1. For practical dispatch, schedule P06 as the first W1 task; P08 may start immediately after P06 completes (within the same wave window).

---

### M03 — First-Paying-Tenant GA

**Directory:** `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/`

Entry gate: M02-P22 exit gate complete.

| Phase ID  | Slug                                   | depends_on                    | unblocks                | parallel_wave | critical_path | est_hours |
|-----------|----------------------------------------|-------------------------------|-------------------------|---------------|---------------|-----------|
| M03-P01   | cloud-foundations                      | M02-P22                       | M03-P02,P03,P04,P05,P06 | M03.W1        | true          | 8         |
| M03-P02   | cloud-compute                          | M03-P01                       | M03-P06                 | M03.W2        | true          | 8         |
| M03-P03   | cloud-data-billing-observability       | M03-P01                       | M03-P06                 | M03.W2        | false         | 8         |
| M03-P04   | saas-platform-preview                  | M03-P01                       | M03-P06                 | M03.W2        | false         | 6         |
| M03-P05   | search-preview                         | M03-P01                       | M03-P06                 | M03.W2        | false         | 5         |
| M03-P06   | workspace-14-surfaces                  | M03-P02,P03,P04,P05           | M03-P07                 | M03.W3        | true          | 12        |
| M03-P07   | regional-pack-onboarding               | M03-P06                       | M03-P08                 | M03.W4        | true          | 5         |
| M03-P08   | cross-axis-contracts                   | M03-P07                       | —                        | M03.W5        | true          | 4         |

**Note on M03 first-paying-tenant phases from MASTERPLAN §4/§13:** MASTERPLAN §4 scopes M03 as enterprise µservices (HR + Payroll + Accounting) + Connect Pro + Cloud-Tenancy substrate + first paying tenant onboarding (1 KR group). MASTERPLAN §13 lists P09-P12 as TBD impl plans within the existing M03 directory milestone. The 8 phases above are the directory-confirmed phases. The TBD impl plans (P09 enterprise-hr-payroll, P10 connect-professional, P11 audit-chain-tenant-segmentation, P12 first-paying-tenant-onboarding) are authored as impl plans **within** existing phases P04/P06/P07/P08 respectively — they do not constitute separate phases. Dispatch script targets the confirmed phase directories.

---

## 2. Dependency DAG (ASCII Art)

```
M01 FOUNDATIONAL (all complete)
  [M01-F01..F06] ──────────────────────────────► M01-P01 (Shard0, complete)
                                                         │
                                                    M01-P02 (Shard1 atomic rename) ◄─── ENTRY POINT
                                                         │
                                                    M01-P03 (Shard1.5 deferred 26 rows)
                                                         │
                                                    M01-P04 (iter4 src inspection)
                                                         │
                                                    M01-P05 (LEAN checks → BLOCKER) ◄── M01 EXIT GATE
                                                         │
            ┌────────────────────────────────────────────┼───────────────────────┐
            ▼                                            ▼                       ▼
     M02.W1 (11 parallel):                        M02.W1 continued:         M02.W1 continued:
     P01-foundry   P02-ontology*   P03-identity   P04-audit   P05-eventing  P06-secrets
     P07-obs       P09-search      P10-vector     P11-finance P08-kms(after P06)
            │
            ▼
     M02.W2 (5 parallel after W1):
     P12-workflow*  P13-tenancy  P14-policy  P15-data-boundary  P16-records
            │
            ▼
     M02.W3 (3 parallel):         M02.W4 (parallel with W2/W3):
     P17-capability  P18-cloud-tenancy  P19-application*     P20-ci-lanes
            │                                │
            └──────────────────┬─────────────┘
                               ▼
                          M02.W5: P21-architecture-planes-green
                               │
                          M02.W6: P22-m02-exit-gate ◄── M02 EXIT GATE
                               │
            ┌──────────────────┼──────────────────┐
            ▼                  ▼                  ▼
     M03.W1: P01-cloud-foundations* (serialized; then fans out)
            │
     M03.W2 (4 parallel):
     P02-cloud-compute*  P03-cloud-data  P04-saas-platform  P05-search-preview
            │
     M03.W3: P06-workspace-14-surfaces* (serializes on all W2)
            │
     M03.W4: P07-regional-pack-onboarding
            │
     M03.W5: P08-cross-axis-contracts ◄── M03 EXIT GATE

* = critical path node
```

---

## 3. Critical Path

The longest dependency chain through M01-M03. Every node on this path has zero float — any slip directly delays M03 exit.

```
M01-P01 (Shard0, complete)
  → M01-P02 (Shard1 atomic rename, 4h)
    → M01-P03 (Shard1.5 deferred rows, 2h)
      → M01-P04 (iter4 src inspection, 3h)
        → M01-P05 (LEAN checks BLOCKER flip, 2h)
          → M02-P02 (ontology, 8h)
            → M02-P12 (workflow engine, 10h)
              → M02-P19 (application B2B shell, 10h)
                → M02-P21 (9 architecture planes green, 3h)
                  → M02-P22 (M02 exit gate, 1h)
                    → M03-P01 (cloud foundations, 8h)
                      → M03-P02 (cloud compute, 8h)
                        → M03-P06 (workspace 14 surfaces, 12h)
                          → M03-P07 (regional pack onboarding, 5h)
                            → M03-P08 (cross-axis contracts, 4h)
```

**Total critical-path steps:** 14 nodes (15 including Shard0 already complete).
**Minimum wall-clock (max parallelism, critical path only):** ~80h sequential on critical path.
**Parallel savings:** Off-critical-path work (M02.W1 11-way fan-out, M03.W2 4-way fan-out) reduces total elapsed time significantly — W1 happens in parallel with critical path's 8h ontology phase, W2 in parallel with workflow-engine, etc.

---

## 4. Blocker Phases (Highest Unblock Count)

Sorted by number of phases unblocked. These get resource priority and must be dispatched FIRST within their wave.

| Rank | Phase ID  | Slug                   | Unblocks (count) | Unblocks (list)                                                    | Dispatch priority annotation        |
|------|-----------|------------------------|------------------|--------------------------------------------------------------------|--------------------------------------|
| 1    | M01-P05   | post-cutover-lean-hardening | 11          | All 11 M02.W1 phases                                               | **DISPATCH FIRST** — M01 exit gate; entire M02 blocked on this |
| 2    | M02-P02   | ontology               | 4                | P12-workflow, P17-capability, P19-application, P21-arch-planes     | **DISPATCH FIRST in M02.W1** — critical-path node; highest downstream leverage |
| 3    | M02-P12   | workflow-engine        | 3                | P17-capability, P19-application, P21-arch-planes                   | **DISPATCH FIRST in M02.W2** — critical-path; fans into application shell |
| 4    | M02-P19   | application            | 2                | P21-architecture-planes-green, M03.W1                              | **DISPATCH FIRST in M02.W3** — critical-path; gates M03 entry |
| 5    | M02-P22   | m02-exit-gate          | 8                | All M03 phases (via M03.W1 entry gate)                             | **DISPATCH FIRST in M02.W6** — M02 exit gate; unblocks entire M03 |

---

## 5. Parallel Waves — Detail with Grit Claim Symbol Space

### Wave M01.W0 — Pre-work complete

All M01-F01..F06 + M01-P01 are complete. No dispatch needed.

---

### Wave M01.W1 — 1 executor

**Entry gate:** M01-F01..F06 complete (verified).
**Phases:** M01-P02 (shard1-atomic-rename-114-rows)

| Executor | Phase | Grit claim symbol space | Path |
|----------|-------|------------------------|------|
| M01.W1.A | M01-P02 | `oya-shard1-rename-*` / `oya-xtask-metadata-*` / `oya-check-*` | `.omc/plans/milestones/M01-foundation/phases/P-shard1-bnf-rename/` |

**Est. wall-clock:** 4h (single executor, no parallelism available)
**Sequential fallback:** Same — no parallelism in this wave.

---

### Wave M01.W2 — 1 executor (after M01.W1)

**Entry gate:** M01-P02 merged to main; grit done confirmed.
**Phases:** M01-P03 (shard1.5-deferred-26-rows)

| Executor | Phase | Grit claim symbol space |
|----------|-------|------------------------|
| M01.W2.A | M01-P03 | `oya-shard1-5-rename-*` / deferred-26-row crates only |

**Est. wall-clock:** 2h.

---

### Wave M01.W3 — 1 executor (after M01.W2)

**Entry gate:** M01-P03 merged.
**Phases:** M01-P04 (iter4-src-inspection)

| Executor | Phase | Grit claim symbol space |
|----------|-------|------------------------|
| M01.W3.A | M01-P04 | `oya-check-architecture-cli` / `oya-check-*` read-only audit (no renames) |

**Est. wall-clock:** 3h.

---

### Wave M01.W4 — 1 executor (after M01.W3)

**Entry gate:** M01-P04 audit complete; all BNF violations documented and resolved.
**Phases:** M01-P05 (post-cutover-lean-hardening)

| Executor | Phase | Grit claim symbol space |
|----------|-------|------------------------|
| M01.W4.A | M01-P05 | `oya-check-architecture-cli` / CI pipeline config / Cargo workspace flags |

**Est. wall-clock:** 2h.
**M01 EXIT GATE fires here.** After `grit done` on M01-P05, M02.W1 may dispatch.

---

### Wave M02.W1 — 11 executors in parallel (after M01.W4)

**Entry gate:** M01-P05 `grit done` confirmed; LEAN checks green on main.

Symbol spaces are non-overlapping. Each executor owns its µservice namespace exclusively.

| Executor  | Phase   | Slug                      | Grit claim symbol space                                                    | Path |
|-----------|---------|---------------------------|----------------------------------------------------------------------------|------|
| M02.W1.A  | M02-P01 | foundry-engine-consolidation | `oya-foundry-*` / `oya-tooling-agent-read` / grit-integration symbols    | `.omc/plans/milestones/M02b-substrate/phases/P01-foundry-engine-consolidation/` |
| M02.W1.B  | M02-P02 | ontology                  | `oya-ontology-*` (all BCs: entity, link, action, function, agent-gateway, audit-chain, pillar) | `.omc/plans/milestones/M02b-substrate/phases/P02-ontology/` |
| M02.W1.C  | M02-P03 | identity                  | `oya-identity-*` (user, person, organization, employee, session, passkey)  | `.omc/plans/milestones/M02b-substrate/phases/P03-identity/` |
| M02.W1.D  | M02-P04 | audit-chain               | `oya-audit-chain-*` (kernel, adapter, merkle, sealer, worker)              | `.omc/plans/milestones/M02b-substrate/phases/P04-audit-chain/` |
| M02.W1.E  | M02-P05 | eventing                  | `oya-eventing-*` (outbox, kafka-adapter, dispatcher, worker)               | `.omc/plans/milestones/M02b-substrate/phases/P05-eventing/` |
| M02.W1.F  | M02-P06 | secrets                   | `oya-secrets-*` (kernel, openbao-adapter, hsm-adapter)                     | `.omc/plans/milestones/M02b-substrate/phases/P06-secrets/` |
| M02.W1.G  | M02-P07 | observability             | `oya-observability-*` (otel-kernel, victoria-metrics-adapter, log-drain)   | `.omc/plans/milestones/M02b-substrate/phases/P07-observability/` |
| M02.W1.H  | M02-P08 | kms                       | `oya-kms-*` (kernel, envelope-encryption, dek-store, hsm-adapter) — **starts after M02-P06 grit done within W1** | `.omc/plans/milestones/M02b-substrate/phases/P08-kms/` |
| M02.W1.I  | M02-P09 | search                    | `oya-search-*` (pgroonga-adapter, tantivy-adapter, morphology-ko, worker)  | `.omc/plans/milestones/M02b-substrate/phases/P09-search/` |
| M02.W1.J  | M02-P10 | vector                    | `oya-vector-*` (pgvector-adapter, hnsw-adapter, embedding-kernel)          | `.omc/plans/milestones/M02b-substrate/phases/P10-vector/` |
| M02.W1.K  | M02-P11 | finance-library           | `oya-finance-*` (money, currency-code, journal-entry, rounding-policy)     | `.omc/plans/milestones/M02b-substrate/phases/P11-finance-library/` |

**Est. wall-clock (max parallelism):** 8h (bottleneck = M02-P02 ontology).
**Sequential fallback est.:** ~51h (sum of all 11 estimates).

**Symbol-lock pre-flight:** All 11 symbol spaces are disjoint kebab-prefix namespaces. No overlap. P06 → P08 dependency within the wave: W1.F (secrets) must `grit done` before W1.H (kms) starts its claim. Orchestrator monitors P06 completion and fires P08 as a W1 sub-wave.

---

### Wave M02.W2 — 5 executors in parallel (after M02.W1)

**Entry gate:** All 11 M02.W1 phases have `grit done` confirmed on main.

| Executor  | Phase   | Slug            | Grit claim symbol space                                                            | Path |
|-----------|---------|-----------------|------------------------------------------------------------------------------------|------|
| M02.W2.A  | M02-P12 | workflow-engine | `oya-workflow-*` (state-machine, dag, approvals, escalations, sla-timer, worker)   | `.omc/plans/milestones/M02b-substrate/phases/P12-workflow-engine/` |
| M02.W2.B  | M02-P13 | tenancy         | `oya-tenancy-*` (kernel, cell-placer, product-registry, rls-bootstrap, adapter)    | `.omc/plans/milestones/M02b-substrate/phases/P13-tenancy/` |
| M02.W2.C  | M02-P14 | policy          | `oya-policy-*` (cedar-kernel, rule-pack, evaluation-log, enforcement-adapter)      | `.omc/plans/milestones/M02b-substrate/phases/P14-policy/` |
| M02.W2.D  | M02-P15 | data-boundary   | `oya-data-boundary-*` (12-class-kernel, hard-deny-enforcer, cedar-policy, audit)   | `.omc/plans/milestones/M02b-substrate/phases/P15-data-boundary/` |
| M02.W2.E  | M02-P16 | records         | `oya-records-*` (fhir-r5-kernel, encounter, observation, medication, adapter)      | `.omc/plans/milestones/M02b-substrate/phases/P16-records/` |

**Est. wall-clock (max parallelism):** 10h (bottleneck = M02-P12 workflow-engine).
**Sequential fallback est.:** ~32h.

**Symbol-lock pre-flight:** All 5 symbol spaces are disjoint. No overlap with W1 symbols (W1 already done).

---

### Wave M02.W3 — 3 executors in parallel (after M02.W2)

**Entry gate:** All 5 M02.W2 phases have `grit done` confirmed.

| Executor  | Phase   | Slug                    | Grit claim symbol space                                                                 | Path |
|-----------|---------|-------------------------|-----------------------------------------------------------------------------------------|------|
| M02.W3.A  | M02-P17 | capability-registry     | `oya-capability-*` (mcp-discovery, endpoint-registry, binding-store, adapter)           | `.omc/plans/milestones/M02b-substrate/phases/P17-capability-registry/` |
| M02.W3.B  | M02-P18 | cloud-tenancy           | `oya-cloud-tenancy-*` / `oya-cloud-iam-*` / `oya-cloud-kms-*` (cloud substrate layer) | `.omc/plans/milestones/M02b-substrate/phases/P18-cloud-tenancy/` |
| M02.W3.C  | M02-P19 | application             | `oya-application-*` (product-enablement, tenant-onboarding, capability-menu, rest)      | `.omc/plans/milestones/M02b-substrate/phases/P19-application/` |

**Est. wall-clock (max parallelism):** 10h (bottleneck = M02-P18 cloud-tenancy and M02-P19 application, both 10h).
**Sequential fallback est.:** ~23h.

---

### Wave M02.W4 — 1 executor (parallel with W2/W3; fires as soon as M02-P01 and M02-P07 are done)

**Entry gate:** M02-P01 (foundry) + M02-P07 (observability) both `grit done` (available as soon as M02.W1 completes — these are W1 phases).

| Executor  | Phase   | Slug                  | Grit claim symbol space                                                       | Path |
|-----------|---------|-----------------------|-------------------------------------------------------------------------------|------|
| M02.W4.A  | M02-P20 | ci-lanes-operational  | `oya-check-statelessness-cli` / `oya-check-shardability-cli` / `oya-check-perf-budget-cli` / `oya-check-benchmark-cli` / CI pipeline YAML | `.omc/plans/milestones/M02b-substrate/phases/P20-ci-lanes-operational/` |

**Est. wall-clock:** 4h.
**Note:** W4 fires in parallel with W2 and W3, not after them. It only requires W1 to complete.

---

### Wave M02.W5 — 1 executor (after M02.W3 + M02.W4 both complete)

**Entry gate:** M02-P17, P18, P19, P20 all `grit done`.

| Executor  | Phase   | Slug                         | Grit claim symbol space                                              | Path |
|-----------|---------|------------------------------|----------------------------------------------------------------------|------|
| M02.W5.A  | M02-P21 | architecture-planes-green    | Read-only audit crate / CI configuration / fitness lane enforcement | `.omc/plans/milestones/M02b-substrate/phases/P21-architecture-planes-green/` |

**Est. wall-clock:** 3h.

---

### Wave M02.W6 — 1 executor (after M02.W5)

**Entry gate:** M02-P21 complete (9 architecture planes green).

| Executor  | Phase   | Slug             | Grit claim symbol space                                           | Path |
|-----------|---------|------------------|-------------------------------------------------------------------|------|
| M02.W6.A  | M02-P22 | m02-exit-gate    | `oya-shared-migrate-cli` / evidence checklist / ICM milestone row | `.omc/plans/milestones/M02b-substrate/phases/P22-m02-exit-gate/` |

**Est. wall-clock:** 1h.
**M02 EXIT GATE fires here.** After `grit done` on M02-P22 and ICM row emitted, M03.W1 may dispatch.

---

### Wave M03.W1 — 1 executor (serialized; after M02.W6)

**Entry gate:** M02-P22 `grit done`; 9 planes green; ICM M02-complete row confirmed.

| Executor  | Phase   | Slug                  | Grit claim symbol space                                                                 | Path |
|-----------|---------|-----------------------|-----------------------------------------------------------------------------------------|------|
| M03.W1.A  | M03-P01 | cloud-foundations     | `oya-cloud-kms-*` / `oya-cloud-storage-*` / `oya-cloud-network-*` / `oya-cloud-iam-*` / `oya-cloud-region-*` | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P01-cloud-foundations/` |

**Est. wall-clock:** 8h.
**Note:** M03-P01 is serialized (all M03 fan-outs depend on it). It gates 4 parallel W2 dispatches.

---

### Wave M03.W2 — 4 executors in parallel (after M03.W1)

**Entry gate:** M03-P01 `grit done`.

| Executor  | Phase   | Slug                                 | Grit claim symbol space                                                               | Path |
|-----------|---------|--------------------------------------|---------------------------------------------------------------------------------------|------|
| M03.W2.A  | M03-P02 | cloud-compute                        | `oya-cloud-compute-*` / `oya-cloud-cell-*` / Firecracker/OKE adapters                | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P02-cloud-compute/` |
| M03.W2.B  | M03-P03 | cloud-data-billing-observability     | `oya-cloud-billing-*` / `oya-cloud-observability-*` / FinOps pipeline                | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P03-cloud-data-billing-observability/` |
| M03.W2.C  | M03-P04 | saas-platform-preview                | `oya-workflow-studio-*` / plugin-substrate / marketplace-listing crates               | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P04-saas-platform-preview/` |
| M03.W2.D  | M03-P05 | search-preview                       | `oya-search-kr-*` / pgroonga-morphology / pgvector-tenant-private / rag-endpoint      | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P05-search-preview/` |

**Est. wall-clock (max parallelism):** 8h (bottleneck = M03-P02 and M03-P03, both 8h).
**Sequential fallback est.:** ~27h.

**Symbol-lock pre-flight:** All 4 symbol spaces are disjoint. M03.W2.D (search-preview) uses `oya-search-kr-*` distinct from M02.W1.I `oya-search-*` (M02 already done at this point).

---

### Wave M03.W3 — 1 executor (after M03.W2 all complete)

**Entry gate:** M03-P02, P03, P04, P05 all `grit done`.

| Executor  | Phase   | Slug                    | Grit claim symbol space                                                                             | Path |
|-----------|---------|-------------------------|-----------------------------------------------------------------------------------------------------|------|
| M03.W3.A  | M03-P06 | workspace-14-surfaces   | `oya-connect-mail-*` / `oya-connect-calendar-*` / `oya-connect-docs-*` / `oya-connect-meet-*` / `oya-connect-forms-*` / `oya-connect-drive-*` / `oya-connect-sites-*` | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P06-workspace-14-surfaces/` |

**Est. wall-clock:** 12h (largest single phase in M03).

---

### Wave M03.W4 — 1 executor (after M03.W3)

**Entry gate:** M03-P06 `grit done`.

| Executor  | Phase   | Slug                        | Grit claim symbol space                                                | Path |
|-----------|---------|-----------------------------|------------------------------------------------------------------------|------|
| M03.W4.A  | M03-P07 | regional-pack-onboarding    | `oya-regional-kr-pack-*` / second-region-pack crates / kr-regulatory  | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P07-regional-pack-onboarding/` |

**Est. wall-clock:** 5h.

---

### Wave M03.W5 — 1 executor (after M03.W4)

**Entry gate:** M03-P07 `grit done`; KR regional pack green.

| Executor  | Phase   | Slug                   | Grit claim symbol space                                                 | Path |
|-----------|---------|------------------------|-------------------------------------------------------------------------|------|
| M03.W5.A  | M03-P08 | cross-axis-contracts   | cross-µservice contract audit / `oya-check-*` / evidence pack assembly | `.omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P08-cross-axis-contracts/` |

**Est. wall-clock:** 4h.
**M03 EXIT GATE fires here.** After `grit done` on M03-P08 and KR acceptance evidence emitted, M03 is complete.

---

## 6. Grit Symbol-Lock Pre-flight

### Non-overlap verification by wave

| Wave      | Executors | Symbol space prefixes (must be disjoint) | Overlap risk | Verdict |
|-----------|-----------|------------------------------------------|--------------|---------|
| M01.W1    | 1         | `oya-shard1-rename-*`                    | none         | CLEAR   |
| M01.W2    | 1         | `oya-shard1-5-rename-*`                  | none         | CLEAR   |
| M01.W3    | 1         | `oya-check-*` (audit only, no writes)    | none         | CLEAR   |
| M01.W4    | 1         | `oya-check-architecture-cli`, CI config  | none         | CLEAR   |
| M02.W1    | 11        | foundry / ontology / identity / audit-chain / eventing / secrets / observability / kms / search / vector / finance | Each is a distinct kebab prefix; kms starts after secrets done | CLEAR (intra-wave P06→P08 sequencing required) |
| M02.W2    | 5         | workflow / tenancy / policy / data-boundary / records | All distinct | CLEAR   |
| M02.W3    | 3         | capability / cloud-tenancy / application | All distinct | CLEAR   |
| M02.W4    | 1         | ci-lanes (check crates + CI YAML)        | None with W3 (different crate types) | CLEAR   |
| M02.W5    | 1         | architecture-planes audit (read + fitness lane config) | None | CLEAR   |
| M02.W6    | 1         | shared-migrate-cli + evidence            | None         | CLEAR   |
| M03.W1    | 1         | cloud-kms / cloud-storage / cloud-network / cloud-iam / cloud-region | None | CLEAR |
| M03.W2    | 4         | cloud-compute / cloud-billing+obs / workflow-studio / search-kr | All distinct | CLEAR   |
| M03.W3    | 1         | connect-* (mail/calendar/docs/meet/forms/drive/sites) | None | CLEAR   |
| M03.W4    | 1         | regional-kr-pack                         | None         | CLEAR   |
| M03.W5    | 1         | cross-axis-contracts (audit)             | None         | CLEAR   |

### Wave-to-wave dependency lock release timing

| Gate                  | Condition                                                      | Releases        |
|-----------------------|----------------------------------------------------------------|-----------------|
| M01 exit gate         | M01-P05 `grit done`; LEAN checks green on main                | M02.W1 (all 11) |
| M02.W1 complete       | All 11 phases `grit done` (P06 before P08 within wave)        | M02.W2 + M02.W4 |
| M02.W2 complete       | All 5 phases `grit done`                                       | M02.W3          |
| M02.W3 + W4 complete  | P17+P18+P19+P20 all `grit done`                                | M02.W5          |
| M02.W5 complete       | P21 `grit done`; 9 planes green                                | M02.W6          |
| M02 exit gate         | M02-P22 `grit done`; ICM row emitted                           | M03.W1          |
| M03.W1 complete       | M03-P01 `grit done`                                            | M03.W2 (all 4)  |
| M03.W2 complete       | All 4 phases `grit done`                                       | M03.W3          |
| M03.W3 complete       | M03-P06 `grit done`                                            | M03.W4          |
| M03.W4 complete       | M03-P07 `grit done`                                            | M03.W5          |

### ICM scaffold-locks-oyatie ledger entries (doc-only work)

Per ADR-0054 §"scaffold-claim pattern" — since grit v0.3.0 has no active symbol registrations for doc-only work, all manifest authoring is logged to ICM:

```
icm store -t scaffold-locks-oyatie \
  -c "parallelization-manifest: claimed doc-only work on M01-M03-parallelization-manifest.md. No code symbols. Session: 2026-05-13" \
  -i high \
  -k "manifest,parallelization,M01,M02,M03,doc-only"
```

---

## 7. Dispatch Script (Orchestrator-Runnable)

Format: `Agent("<phase-slug>", brief: "<impl-plan-path>")`. When a phase has no impl-plan file yet (TBD), the brief points to the phase directory README (to be authored first by a planning executor before implementation begins).

```
═══════════════════════════════════════════════════════════════════
WAVE M01.W1  (1 executor; fires immediately)
═══════════════════════════════════════════════════════════════════

Agent("M01-P02 shard1-atomic-rename-114-rows",
  brief: ".omc/plans/milestones/M01-foundation/phases/P-shard1-bnf-rename/impl-plan.md",
  grit_symbols: ["oya-shard1-rename-*", "oya-xtask-metadata-*", "oya-check-*"])


═══════════════════════════════════════════════════════════════════
WAVE M01.W2  (1 executor; fires after M01.W1 grit done)
═══════════════════════════════════════════════════════════════════

Agent("M01-P03 shard1.5-deferred-26-rows",
  brief: ".omc/plans/milestones/M01-foundation/phases/P-shard1-bnf-rename/impl-plan.md#shard-1-5",
  grit_symbols: ["oya-shard1-5-rename-*"])


═══════════════════════════════════════════════════════════════════
WAVE M01.W3  (1 executor; fires after M01.W2 grit done)
═══════════════════════════════════════════════════════════════════

Agent("M01-P04 iter4-src-inspection",
  brief: ".omc/plans/milestones/M01-foundation/phases/P-shard1-bnf-rename/impl-plan.md#iter4",
  grit_symbols: ["oya-check-architecture-cli"])


═══════════════════════════════════════════════════════════════════
WAVE M01.W4  (1 executor; fires after M01.W3 grit done)
═══════════════════════════════════════════════════════════════════

Agent("M01-P05 post-cutover-lean-hardening",
  brief: ".omc/plans/milestones/M01-foundation/phases/P-shard1-bnf-rename/impl-plan.md#lean-hardening",
  grit_symbols: ["oya-check-architecture-cli", "ci-pipeline-config"])


═══════════════════════════════════════════════════════════════════
WAVE M02.W1  (11 executors in parallel; fires after M01 exit gate)
═══════════════════════════════════════════════════════════════════

Agent("M02-P01 foundry-engine-consolidation",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P01-foundry-engine-consolidation/phase-spec.md",
  grit_symbols: ["oya-foundry-*", "oya-tooling-agent-read"])

Agent("M02-P02 ontology",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P02-ontology/phase-spec.md",
  grit_symbols: ["oya-ontology-*"],
  priority: CRITICAL_PATH)

Agent("M02-P03 identity",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P03-identity/phase-spec.md",
  grit_symbols: ["oya-identity-*"])

Agent("M02-P04 audit-chain",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P04-audit-chain/phase-spec.md",
  grit_symbols: ["oya-audit-chain-*"])

Agent("M02-P05 eventing",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P05-eventing/phase-spec.md",
  grit_symbols: ["oya-eventing-*"])

Agent("M02-P06 secrets",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P06-secrets/phase-spec.md",
  grit_symbols: ["oya-secrets-*"],
  note: "Must complete before M02-P08 (kms) can start within this wave")

Agent("M02-P07 observability",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P07-observability/phase-spec.md",
  grit_symbols: ["oya-observability-*"])

Agent("M02-P08 kms",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P08-kms/phase-spec.md",
  grit_symbols: ["oya-kms-*"],
  entry_gate: "M02-P06 grit done")

Agent("M02-P09 search",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P09-search/phase-spec.md",
  grit_symbols: ["oya-search-*"])

Agent("M02-P10 vector",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P10-vector/phase-spec.md",
  grit_symbols: ["oya-vector-*"])

Agent("M02-P11 finance-library",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P11-finance-library/phase-spec.md",
  grit_symbols: ["oya-finance-*"])


═══════════════════════════════════════════════════════════════════
WAVE M02.W2  (5 executors in parallel; fires after M02.W1 complete)
═══════════════════════════════════════════════════════════════════

Agent("M02-P12 workflow-engine",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P12-workflow-engine/phase-spec.md",
  grit_symbols: ["oya-workflow-*"],
  priority: CRITICAL_PATH)

Agent("M02-P13 tenancy",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P13-tenancy/phase-spec.md",
  grit_symbols: ["oya-tenancy-*"])

Agent("M02-P14 policy",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P14-policy/phase-spec.md",
  grit_symbols: ["oya-policy-*"])

Agent("M02-P15 data-boundary",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P15-data-boundary/phase-spec.md",
  grit_symbols: ["oya-data-boundary-*"])

Agent("M02-P16 records",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P16-records/phase-spec.md",
  grit_symbols: ["oya-records-*"])


═══════════════════════════════════════════════════════════════════
WAVE M02.W3  (3 executors in parallel; fires after M02.W2 complete)
AND
WAVE M02.W4  (1 executor in parallel; fires after M02.W1 complete)
(W3 and W4 run concurrently; W5 waits for both)
═══════════════════════════════════════════════════════════════════

Agent("M02-P17 capability-registry",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P17-capability-registry/phase-spec.md",
  grit_symbols: ["oya-capability-*"])

Agent("M02-P18 cloud-tenancy",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P18-cloud-tenancy/phase-spec.md",
  grit_symbols: ["oya-cloud-tenancy-*", "oya-cloud-iam-*", "oya-cloud-kms-*"])

Agent("M02-P19 application",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P19-application/phase-spec.md",
  grit_symbols: ["oya-application-*"],
  priority: CRITICAL_PATH)

Agent("M02-P20 ci-lanes-operational",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P20-ci-lanes-operational/phase-spec.md",
  grit_symbols: ["oya-check-statelessness-cli", "oya-check-shardability-cli",
                 "oya-check-perf-budget-cli", "oya-check-benchmark-cli"],
  entry_gate: "M02.W1 complete (not W2)")


═══════════════════════════════════════════════════════════════════
WAVE M02.W5  (1 executor; fires after M02.W3 + M02.W4 both complete)
═══════════════════════════════════════════════════════════════════

Agent("M02-P21 architecture-planes-green",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P21-architecture-planes-green/phase-spec.md",
  grit_symbols: ["oya-check-architecture-planes-cli", "fitness-lane-config"])


═══════════════════════════════════════════════════════════════════
WAVE M02.W6  (1 executor; fires after M02.W5 complete)
═══════════════════════════════════════════════════════════════════

Agent("M02-P22 m02-exit-gate",
  brief: ".omc/plans/milestones/M02b-substrate/phases/P22-m02-exit-gate/phase-spec.md",
  grit_symbols: ["oya-shared-migrate-cli", "m02-evidence-checklist"],
  note: "Emit ICM row: icm store -t context-oyatie -c 'M02 exit gate complete...' -i high")


═══════════════════════════════════════════════════════════════════
WAVE M03.W1  (1 executor; fires after M02 exit gate)
═══════════════════════════════════════════════════════════════════

Agent("M03-P01 cloud-foundations",
  brief: ".omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P01-cloud-foundations/phase-spec.md",
  grit_symbols: ["oya-cloud-kms-*", "oya-cloud-storage-*", "oya-cloud-network-*",
                 "oya-cloud-iam-*", "oya-cloud-region-*"],
  priority: CRITICAL_PATH)


═══════════════════════════════════════════════════════════════════
WAVE M03.W2  (4 executors in parallel; fires after M03.W1 grit done)
═══════════════════════════════════════════════════════════════════

Agent("M03-P02 cloud-compute",
  brief: ".omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P02-cloud-compute/phase-spec.md",
  grit_symbols: ["oya-cloud-compute-*", "oya-cloud-cell-*"],
  priority: CRITICAL_PATH)

Agent("M03-P03 cloud-data-billing-observability",
  brief: ".omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P03-cloud-data-billing-observability/phase-spec.md",
  grit_symbols: ["oya-cloud-billing-*", "oya-cloud-observability-*"])

Agent("M03-P04 saas-platform-preview",
  brief: ".omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P04-saas-platform-preview/phase-spec.md",
  grit_symbols: ["oya-workflow-studio-*", "oya-marketplace-*"])

Agent("M03-P05 search-preview",
  brief: ".omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P05-search-preview/phase-spec.md",
  grit_symbols: ["oya-search-kr-*", "oya-search-rag-*"])


═══════════════════════════════════════════════════════════════════
WAVE M03.W3  (1 executor; fires after M03.W2 all grit done)
═══════════════════════════════════════════════════════════════════

Agent("M03-P06 workspace-14-surfaces",
  brief: ".omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P06-workspace-14-surfaces/phase-spec.md",
  grit_symbols: ["oya-connect-mail-*", "oya-connect-calendar-*", "oya-connect-docs-*",
                 "oya-connect-meet-*", "oya-connect-forms-*", "oya-connect-drive-*",
                 "oya-connect-sites-*", "oya-connect-sheets-*", "oya-connect-slides-*"],
  priority: CRITICAL_PATH)


═══════════════════════════════════════════════════════════════════
WAVE M03.W4  (1 executor; fires after M03.W3 grit done)
═══════════════════════════════════════════════════════════════════

Agent("M03-P07 regional-pack-onboarding",
  brief: ".omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P07-regional-pack-onboarding/phase-spec.md",
  grit_symbols: ["oya-regional-kr-pack-*", "oya-regional-second-pack-*"])


═══════════════════════════════════════════════════════════════════
WAVE M03.W5  (1 executor; fires after M03.W4 grit done)  ← M03 EXIT GATE
═══════════════════════════════════════════════════════════════════

Agent("M03-P08 cross-axis-contracts",
  brief: ".omc/plans/milestones/M03-cloud-saas-search-workspace-preview/phases/P08-cross-axis-contracts/phase-spec.md",
  grit_symbols: ["oya-cross-axis-contracts-audit-*"],
  note: "Emit ICM row: icm store -t context-oyatie -c 'M03 complete. 1 KR paying tenant live.' -i critical")
```

---

## 8. Wave Summary Table

| Wave       | Phases                        | Executor count | Entry gate                       | Est. wall-clock (max parallel) |
|------------|-------------------------------|----------------|----------------------------------|-------------------------------|
| M01.W0     | F01-F06, P01 (all complete)   | 0 (done)       | —                                | 0h                            |
| M01.W1     | P02                           | 1              | M01-F01..F06 complete            | 4h                            |
| M01.W2     | P03                           | 1              | M01.W1 done                      | 2h                            |
| M01.W3     | P04                           | 1              | M01.W2 done                      | 3h                            |
| M01.W4     | P05                           | 1              | M01.W3 done                      | 2h                            |
| M02.W1     | P01-P11                       | 11             | M01 exit gate                    | 8h                            |
| M02.W2     | P12-P16                       | 5              | M02.W1 complete                  | 10h                           |
| M02.W3+W4  | P17-P20 (W3=3, W4=1)          | 4              | W3: M02.W2; W4: M02.W1           | 10h                           |
| M02.W5     | P21                           | 1              | M02.W3+W4 complete               | 3h                            |
| M02.W6     | P22                           | 1              | M02.W5 complete                  | 1h                            |
| M03.W1     | P01                           | 1              | M02 exit gate                    | 8h                            |
| M03.W2     | P02-P05                       | 4              | M03.W1 done                      | 8h                            |
| M03.W3     | P06                           | 1              | M03.W2 complete                  | 12h                           |
| M03.W4     | P07                           | 1              | M03.W3 done                      | 5h                            |
| M03.W5     | P08                           | 1              | M03.W4 done                      | 4h                            |

**Total unique waves:** 15 (including M01.W0 as completed pre-work).
**Peak parallelism:** 11 executors (M02.W1).
**Minimum total elapsed time (critical path only):** ~80h of sequential critical-path work.
**Total phase count covered:** 41 (6 M01 foundational + 5 M01 operational + 22 M02 + 8 M03).

---

## 9. Halt Conditions

The orchestrator must halt and surface to the user if any of the following occur:

1. Any phase fails its `grit done` (non-zero exit or merge conflict) — halt subsequent waves for that milestone; surface error with phase ID and grit output.
2. Any LEAN check (`oya-check-architecture-cli`, `oya-check-statelessness-cli`, etc.) exits non-zero after a phase merge — halt the phase's downstream wave.
3. M01-P05 LEAN hardening fails — entire M02 halted until resolved.
4. M02-P22 exit gate fails — entire M03 halted.
5. M02-P02 (ontology) or M02-P12 (workflow-engine) exceed 2× estimated wall-clock — escalate to user (critical-path risk).
6. Two executors claim overlapping grit symbols — abort the later claim; orchestrator must verify symbol-lock pre-flight before dispatching each wave.

---

## 10. How to Use This Manifest

1. **Orchestrator reads this manifest** before firing any executor.
2. **Check current status:** `ls .omc/plans/milestones/M01-foundation/phases/P-shard1-bnf-rename/` — if missing, a planning executor must author it first.
3. **Fire waves in order** per the Dispatch Script (§7). Do not skip waves. Do not dispatch Wave N+1 until all Wave N `grit done` calls confirm.
4. **Intra-wave sequencing:** Within M02.W1, fire P06 (secrets) first; P08 (kms) starts only after P06 `grit done`. All other 9 phases are fully parallel.
5. **ICM checkpoint:** After each milestone exit gate, emit the ICM row as specified in the dispatch notes. This provides durable progress anchors across sessions.
6. **Impl plans not yet authored (TBD):** The dispatch script points to `phase-spec.md` for phases where impl plans are TBD. The dispatched executor reads the phase-spec, authors the impl plan under the same directory, then implements.
7. **Blocker priority:** When resource-constrained, prioritize M01-P05 → M02-P02 → M02-P12 → M02-P19 → M02-P22 → M03-P01 → M03-P02 → M03-P06 → M03-P07 → M03-P08 (critical-path sequence).

---

## 11. Grit Session + ICM Outcome Log

**Grit session outcome:** `grit session start parallelization-manifest-2026-05-13` failed (no registered symbol scope; doc-only work has no pre-registered symbols in grit v0.3.0). Fallback applied per ADR-0054 §"scaffold-claim pattern":

```bash
# ICM scaffold-locks-oyatie ledger entry (run by manifest author):
icm store -t scaffold-locks-oyatie \
  -c "parallelization-manifest-2026-05-13: authored M01-M03-parallelization-manifest.md. Doc-only. No code symbols. All 41 phases (6 M01-foundational + 5 M01-operational + 22 M02 + 8 M03) accounted for. Critical path: 14 nodes. 15 waves total. Top blockers: M01-P05, M02-P02, M02-P12, M02-P19, M02-P22." \
  -i high \
  -k "manifest,parallelization,M01,M02,M03,critical-path,doc-only,2026-05-13"
```

**grit done:** Not applicable for doc-only work without registered symbols. Manifest is committed to `.omc/plans/M01-M03-parallelization-manifest.md` on main directly via git commit (ADR-0053 bootstrap window allowance for doc-only work during M01 in-flight).
