# Deep Interview Spec: Oyatie Development-Pipeline Optimization Program

## Metadata
- Interview ID: di-pipeline-audit-20260726
- Rounds: 7 (+ Round 0 topology gate)
- Final Ambiguity Score: 19%
- Type: brownfield
- Generated: 2026-07-26T06:30:00Z
- Threshold: 0.2
- Threshold Source: default
- Initial Context Summarized: yes (pause-handoff + Bun-in-Rust post distilled)
- Status: PASSED
- Execution directive (mid-interview, authoritative): start implementation immediately; automate everything; exclude human-dependent steps from the critical path; deployment out of scope; sustain implementation throughput at hyperscaler quality bar across code, tests, CI, and review.

## Clarity Breakdown
| Dimension | Score | Weight | Weighted |
|-----------|-------|--------|----------|
| Goal Clarity | 0.85 | 0.35 | 0.298 |
| Constraint Clarity | 0.72 | 0.25 | 0.180 |
| Success Criteria | 0.80 | 0.25 | 0.200 |
| Context Clarity | 0.88 | 0.15 | 0.132 |
| **Total Clarity** | | | **0.81** |
| **Ambiguity** | | | **0.19** |

## Topology
| Component | Status | Description | Coverage / Deferral Note |
|-----------|--------|-------------|--------------------------|
| Planning & spec intake | active | Bun-style mechanical planning: agent-consumable mapping artifacts, crate/capability-partitioned lane assignment, machine-readable dependency-ordered work queue derived from verifiable signals | R7 |
| Implementation loop | active | Local dev-loop + agent-fleet health: FD/daemon hygiene, buck2 iteration speed, 8-lane certified concurrency | R6 |
| Review & verification | active | Adversarial review + local gate parity, mechanical not manual | Covered via doctrine + acceptance criteria |
| CI admission & merge | active | Affected-set latency (exact-SHA trusted reuse, synthetic-deps broadening), warm-cache CAS (code ready, cutover dark until infra exists), merge-train throughput | R3 |
| Deployment & post-merge | active (receipts-only) | No live deploy target. Optimize what exists: fully mechanical postmerge packet (promotion verification, receipts, observability checks) | R5; live deployment explicitly OUT of scope per final directive |

## Goal
Audit and optimize the entire Oyatie delivery lifecycle (planning → implementation → review → CI admission/merge → post-merge receipts) as one pipeline, then keep implementation flowing: define quantitative per-stage SLO targets, implement owned-Rust fixes until every target is green with receipts, certified at an 8-concurrent-agent-lane design point — while continuously landing implementation slices through the governance pipeline (isolated worktree → protected PR to `dev` → `oya-ci-required` → reviewer APPROVE → squash merge).

## Constraints
- Owned-Rust only: no shell/python/adhoc deps in deliverables; buck2-primary build with BUCK+reindeer wiring part of done; cargo supplementary.
- All changes ride the governance pipeline (worktree, signed commits, protected PR to `dev`, exact `oya-ci-required`, reviewer APPROVE, squash merge, postmerge receipts). Never hand-edit `*.generated.json` — use the sanctioned materializer.
- New code lands in the debranded ADR-0562 capability-first tree; every `oya/**`, `oya-*`, `cloud/**`, `cloud-*` path is a deprecated migration source.
- Soundness before speed: affected-set changes must preserve fail-closed semantics; merge-base baseline stays the ratchet anchor (frozen-baseline is correct for the RATCHET only; candidate predicates evaluate the candidate tree).
- No human-dependent step may block the implementation stream. Anything requiring founder action (paused-PR drain #1379–#1382 per handoff resume order, SLO ratification, CAS cluster provisioning) is queued as a parallel non-blocking track with everything automatable pre-staged.
- 8-lane fleet design point sizes all concurrency/caching/queue targets.
- CAS warm-cache: implement and test all code paths dark-wired (ADR-0556/0560 cold-canary trust anchor preserved); live cutover deferred until a cache host exists — not on the critical path.

## Non-Goals
- Any live deployment target or environment provisioning (explicitly excluded by final directive).
- Merging the four paused PRs (#1379–#1382) — handoff resume order is founder-gated; pre-stage rebases/verifications only if free, never block on them.
- 16/64-lane Bun-scale certification (revisit after 8-lane SLOs are green).
- GraphQL/CLI surfaces of any kind (doctrine: APIs + declarative state + reconcilers).
- Replacing GitHub `merge_group` with an owned merge queue (ADR-0124 superseded by ADR-0515).

## Acceptance Criteria
Stage SLOs (audit slice ratifies exact numbers as policy-as-data; defaults below are the working targets):
- [ ] **Planning**: machine-readable lane-ready work queue exists (dependency-ordered, crate/capability-partitioned, acceptance criteria per item); 8 lanes can pull non-conflicting work with zero hand-dispatch; queue freshness gated mechanically.
- [ ] **Implementation loop**: zero `EMFILE` at 8 concurrent lanes (coordinator FD hygiene fixed at root cause, not ulimit-only); zero stale buck2 daemon accumulation (owned-Rust reaper + liveness policy); local incremental gate-lane iteration ≤ 2 min warm.
- [ ] **Review & verification**: every program PR carries adversarial review evidence (independent reviewer lane, not self-approval) and local gate parity receipts before admission.
- [ ] **CI admission**: affected-set FULL-tier pre-candidate overhead reduced from ~41 min to ≤ 10 min via (a) trusted exact-SHA merge-base baseline-artifact reuse ("download-not-rebuild", owned-Rust selector, fail-closed to rebuild) and (b) synthetic-dependency/ownership broadening for config-file classes so BUCK/yaml/toml/json edits stop escalating to FULL when provably inert — with soundness receipts.
- [ ] **CI admission (cache)**: warm-read CAS path fully implemented + integration-tested dark; cold integrity-canary licensing logic intact; cutover is a config flip, not a code change.
- [ ] **Post-merge**: postmerge packet (promoted-commit `oya-ci-required` green, rollout note, observability check, release-note impact, harvest outcome) generated mechanically per merge — zero manual assembly.
- [ ] **Pipeline-wide**: 8-lane end-to-end certification run recorded (8 concurrent worktree lanes from queue-pull through merged PR) with no manual shepherding events; every SLO target + measurement lives as policy-as-data with a gate that REDs on regression.
- [ ] **Doc corpus**: live md/json doc count measurably reduced (countable before/after); every surviving doc is mechanically justified (authority-chain-reachable, gate-consumed, or code-cited); an orphan-doc gate REDs on regrowth.
- [ ] Every landed slice: buck2 build+test green, BUCK/reindeer wired, tests at the full ladder appropriate to tier, no placeholder/skip/stub patterns.

## Assumptions Exposed & Resolved
| Assumption | Challenge | Resolution |
|------------|-----------|------------|
| "Pipeline" = CI | R0 topology gate | Entire lifecycle planning→deployment, everything needed for implementation work |
| Audit report might suffice | R1 deliverable fork | Full optimization program: implement until targets green |
| "Optimized" is qualitative | R2 | Quantitative per-stage SLOs with receipts |
| Warm-cache infra is out of scope | R3 | In-program; then final directive removed live-infra dependency from critical path (code dark-wired, cutover deferred) |
| Optimize-first means full freeze | R4 contrarian | Drain-first chosen, then superseded by final directive: implementation starts now; founder-gated drain runs as parallel non-blocking track |
| Deployment stage needs a live target | R5 | Receipts-only; live deployment excluded |
| Bun-scale (64) is the design point | R6 simplifier | 8 lanes — simplest useful, certified |
| Planning stays human-gated | R7 (Bun evidence) | Bun-style mechanical intake: mapping artifacts + partition-by-crate + signal-derived work queue |

## Technical Context (brownfield findings, origin/dev)
- `oya-ci-required.yml` (1243 lines): producer-regen → 42-leg gate matrix over `ci/facade/` (49 dirs) → 9 fan-in jobs → zero-build fan-in. Faces artifact shared to save ~45-55s/leg.
- Latency hot spot: `gate-affected-target-set` FULL tier does up to two full workspace passes (merge-base baseline in clean worktree + candidate). Measured on #1379: 9-min setup + 32-min cold baseline ≈ 41 min pre-candidate.
- Levers identified in-tree, unlanded: exact-SHA trusted-artifact reuse (#1323 follow-up; python-glue predecessor removed), NativeLink CAS warm reads (ADR-0560, `infra/ci/buckconfig/warm-cache-{ro,rw}.buckconfig`, dark behind `warm_reads_licensed`), synthetic-deps broadening (`ci/facade/affected-target-set/affected-set-policy.json`: any BUCK edit or unowned yaml/toml/json escalates FULL — the biggest amplifier; policy comment calls broadening "an optimization follow-up, never a soundness requirement").
- ADR-0360 (Proposed) is the pre-existing latency program: affected-target precision, warm cache, sharding, speculative merge queue, content-addressed gate caching — largely unimplemented; this program subsumes it.
- Local findings (issue #1377 / #899 comments): coordinator soft FD limit 256 with 248 open descriptors (completed-subagent rollout files left open) → intermittent `EMFILE`; 37 idle buck2 daemon/forkserver groups reaped by hand.
- Caching today: `actions/cache` on `buck-out`, dev-push-saves/PR-restores split; cold canary workflow (ADR-0556 D2) is the warm-trust anchor.
- Scale: 928 Cargo.toml / 941 BUCK files; glob-only root workspace members (ADR-0538); 15 nested workspaces under `cloud/cloud-kernel/`.
- Bun-in-Rust techniques adopted: pre-generation mapping docs reviewed adversarially; partition-by-crate parallelism; errors-as-work-queue; staged validation (check→smoke→local→full); resource-isolated stress tests; adversarial multi-reviewer gate.

## Ontology (Key Entities)
| Entity | Type | Fields | Relationships |
|--------|------|--------|---------------|
| Pipeline Lifecycle | core domain | 5 stages | has many Lifecycle Stages |
| Lifecycle Stage | core domain | id, SLO targets, status | has many Audit Findings, SLO Targets |
| Optimization Program | core domain | scope, done-bar | implements Audit Findings until SLO Targets green |
| SLO Target | core domain | metric, threshold, receipt | gates Lifecycle Stage; stored as policy-as-data |
| Work Queue | core domain | lane-ready items, dependency order, partitions | feeds Agent Lanes |
| Agent Lane | core domain | worktree, branch, PR | pulls from Work Queue; 8 concurrent (design point) |
| Affected-set Lane | supporting | tiers, escape classes, policy.json | contains Merge-base Baseline |
| Merge-base Baseline | supporting | worktree, ratchet | replaced by Trusted Artifact when SHA matches |
| Trusted Artifact (exact-SHA) | supporting | run id, artifact id, digest | substitutes baseline rebuild |
| CAS Substrate | supporting | NativeLink gRPC, mTLS, warm license | dark-wired; licensed by Cold Canary |
| Cold Canary | supporting | scheduled from-empty build | trust anchor for warm reads |
| Gate Matrix | supporting | 42 legs, ci/facade crates | required context `oya-ci-required` |
| Merge Train | supporting | serialized exact-base admission | throughput ceiling for lanes |
| Postmerge Packet | supporting | receipts, rollout note, observability check | generated mechanically per merge |
| Drain Phase | supporting | PRs #1379–#1382 | founder-gated, parallel non-blocking track |
| Mapping Doc | supporting | pattern equivalences, partitions | reviewed before lane dispatch (Bun PORTING.md analog) |

## Ontology Convergence
| Round | Entity Count | New | Changed | Stable | Stability Ratio |
|-------|-------------|-----|---------|--------|----------------|
| 1 | 11 | 11 | - | - | N/A |
| 2 | 12 | 1 | 0 | 11 | 92% |
| 3 | 13 | 1 | 0 | 12 | 92% |
| 4 | 14 | 1 | 0 | 13 | 93% |
| 5 | 15 | 1 | 0 | 14 | 93% |
| 6 | 16 | 1 | 0 | 15 | 94% |
| 7 | 16 | 0 | 1 | 15 | 94% |

## Execution Work Queue (dependency-ordered, crate-partitioned; first pull set)
1. **audit-slo-policy**: SLO targets as policy-as-data (`specs/` + gate crate) — encode the acceptance-criteria numbers, wire a REDs-on-regression gate. Partition: new `ci/facade/` crate.
2. **fd-daemon-hygiene**: root-cause the coordinator descriptor leak class + owned-Rust buck2 daemon/forkserver reaper with liveness policy. Partition: tools/ci-hygiene capability.
3. **exact-sha-trusted-reuse**: owned-Rust selector (`trusted_dev_push_run_id`/`trusted_build_health_baseline_artifact_id`) in `ci/facade/affected-target-set`, fail-closed to rebuild, digest-verified. Partition: affected-target-set crate.
4. **synthetic-deps-broadening**: ownership/inertness classification for config-file classes in affected-set-policy, with soundness proofs per class. Partition: affected-target-set policy + tests (after 3 lands).
5. **postmerge-packet-automation**: mechanical packet generator + gate. Partition: new facade crate.
6. **cas-warm-dark-complete**: finish + integration-test warm-read paths dark; cutover = config flip. Partition: infra/ci buckconfig + gate tests.
7. **lane-queue-product**: machine-readable work-queue format + freshness gate + partition-assignment (this queue itself becomes item #1's first consumer). Partition: new capability.
8. **doc-corpus-trim** (founder directive 2026-07-26): reduce the markdown/doc corpus to the bare minimum actually consumed. Mechanical needed-ness criterion: reachable from the authority chain (`specs/root-hub-pointers.json` → `docs/AGENTS.md` → ADR chain), consumed by a gate/generator/materializer, or cited by live code — everything else is deleted; duplicated per-service doc trees (ARCH/PRD/README/catalog/runbook/threat-model boilerplate) collapse to generated or single-source forms. Ship an owned-Rust orphan-doc detector gate so the corpus can't regrow (leverages docs-graph-drift + adr-orphan patterns). Countable before/after per masterplan-v2. Partition: docs + new facade gate crate.
9. **8-lane-certification**: end-to-end concurrency certification harness + receipts.

## Interview Transcript
<details>
<summary>Full Q&A (7 rounds + Round 0)</summary>

### Round 0 (Topology)
**Q:** 5 components — CI latency, local dev-loop, merge-train, agent-fleet readiness, paused-PR drain? **A:** "Not just CI — from planning to deployment, the entire lifecycle as pipeline." (+ mid-turn: "everything that is needed for implementation work")

### Round 1
**Q:** Deliverable: audit report vs fix blockers vs full program? **A:** Full optimization program. **Ambiguity:** 44% (G:0.70 C:0.45 Cr:0.30 Ctx:0.85)

### Round 2
**Q:** Done bar? **A:** Quantitative SLO targets, implement until green with receipts. **Ambiguity:** 35%

### Round 3
**Q:** CAS cache infra in scope, where? **A:** Yes — stand it up in-program. **Ambiguity:** 31% *(later narrowed by final directive: code dark-wired, live cutover off critical path)*

### Round 4 (Contrarian)
**Q:** Strict freeze vs relax sequencing? **A:** Drain first, then program. **Ambiguity:** 27% *(later superseded by final directive: implementation starts immediately; drain = founder-gated parallel track)*

### Round 5
**Q:** What is "deployment" with no live target? **A:** Receipts-only — automate the postmerge packet. **Ambiguity:** 24%

### Round 6 (Simplifier)
**Q:** Fleet design point: 8/16/64/derived? **A:** 8 lanes — simplest useful. **Ambiguity:** 23%

### Round 7
**Q:** Planning stage "optimized" means? **A:** Learn from the Bun post — mapping docs, crate partitioning, mechanical signal-derived work queue. **Ambiguity:** 19% ✅

### Final directive (mid-turn, post-threshold)
"Start writing code. Automate everything. Don't worry about anything a human needs to do. Deployment out. Keep implementation coming at hyperscaler bar across code quality, test, CI, review, pipeline."
</details>
