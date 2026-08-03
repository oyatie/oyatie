# Generated Artifact Control Plane — Root-Cause Plan

Date: 2026-06-09
Mode: `$analyze` + `$plan` + CI/CD automation
Scope: Oyatie first, reusable product pattern for any repo with generated artifacts and parallel PR lanes.
Stop condition for this plan: root cause is evidence-backed, weak fixes are rejected, and an implementation-ready roadmap exists without weakening audit/tamper guarantees.

## Requirements Summary

- Solve the recurring merge conflicts caused by generated JSON at the root, not by per-PR hand conflict resolution.
- Preserve intentional deletions and current governance constraints; do not use generated conflicts as a reason to resurrect deleted agent surfaces.
- Preserve audit integrity: generated faces must remain tamper-evident and deterministic.
- Apply hyperscaler lens: design must support many repos, many PRs, many agents, merge queues, and centralized policy.
- Avoid gitignore-only as the answer. Build/cache outputs are ignored; canonical generated projections need explicit authority and CI policy.

## RALPLAN-DR Summary

### Principles

1. Generated outputs are never ordinary multi-writer collaboration surfaces.
2. Source-of-truth and derived projection are separate roles with separate merge policies.
3. CI must prevent conflict topologies before expensive validation, not just detect drift after branches diverge.
4. Audit/tamper guarantees must survive any performance or concurrency optimization.
5. Productized policy beats repo-specific tribal instructions.

### Decision Drivers

1. Parallel agent throughput without generated-file merge storms.
2. Maintained `committed == regenerated` or equivalent signed content-addressed audit guarantee.
3. Portable adoption path across repositories and CI providers.

### Viable Options

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| Keep monolithic tracked generated files and instruct agents to rebase/regenerate | Minimal code change; current audit invariant remains | Structural conflicts recur; human/agent busywork scales with PR count | Reject as current failure mode |
| Gitignore all generated files | Removes git conflicts | Loses committed tamper-evidence unless replaced by stronger artifact manifest/CAS | Reject as standalone fix |
| Broad `.gitattributes` merge drivers (`union`, `ours`) for `*.generated.json` | Looks simple locally | Masks stale rows/baseline growth/provenance conflicts; not portable to GitHub governance | Reject as governance fix |
| Queue/controller-owned materialization with generated-only conflict resolver | Preserves final-tree parity; centralizes regeneration; high throughput | Requires merge-queue/controller tooling | Adopt first |
| Sharded projections plus main-only aggregate publication | Best long-term conflict reduction; aligns ownership to lanes | Requires generator/schema migration | Adopt incrementally |
| External CAS artifacts with committed root manifest | Avoids huge git blobs while preserving audit if signed/immutable | More infra; must fail closed | Optional later for huge outputs |

## Ranked Analysis

| Rank | Explanation | Confidence | Basis |
|---|---|---:|---|
| 1 | Generated JSON conflicts are structural because whole-repo generated projections are tracked on every PR branch. | High | Producer writes multiple shared `.generated.json` faces from repo-wide inputs; CI requires byte parity. |
| 2 | Existing CI is a drift detector, not a concurrency-prevention system. | High | `registry-drift` compares committed bytes after regeneration; merge queue docs separately define pre-admit conflict checks. |
| 3 | Gitignore-only or merge-driver-only fixes would weaken governance or mask semantic conflicts. | High | Repo already scopes `merge=union` only to append-only audit-chain and warns about replacement-style stale rows. |
| 4 | The repo already has the primitives for the correct product direction: artifact registry/profile model, concurrent-safe path registry, projected merge state, sharded shared-surface doctrine. | Medium-High | Existing specs/ADRs describe these patterns, but the generated-face control plane is not yet implemented. |

## Evidence

- `.gitignore:1-5,44-45` — build/runtime outputs such as `target`, `.omx`, and `buck-out` are ignored; tracked generated JSON is not broadly ignored.
- `.gitattributes:1-23` — only `evidence/audit-chain.jsonl` uses `merge=union`, with an explicit warning that union is unsafe for replacement-style files.
- `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs:191-262` — one producer builds the registry/crosswalk/enforcement/baseline and writes five shared generated faces.
- `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:242-321` — registry generation is deterministic, path-sorted, and marked `DO NOT HAND-EDIT` so byte parity can hold.
- `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:264-271` — generated files already needed special fixed-point handling for `last_touch_commit`, proving self-referential generated outputs are a known risk.
- `cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs:40-54,152-174,183-223` — `registry-drift` byte-compares committed generated faces and `git-facts.generated.json` against regenerated output.
- `.github/workflows/oya-ci-required.yml:38-61,102-120,202-219,248-271` — CI regenerates faces, validates committed parity, validates `git-facts`, and fans into the single required `oya-ci-required` check.
- `docs/oya-ci/quick-start.md:22-40,54-61` — current adoption guide tells users to commit generated faces and even uses a two-commit settle procedure for new files.
- `registry/vcs/concurrent-safe-paths.yaml:1-24` — concurrent-safe path whitelist exists but is default-empty.
- `docs/decisions/ADR-0366-agentic-high-throughput-self-enforcing-pipeline.md:62-80` — accepted pipeline doctrine calls for disjoint ownership, concurrent-safe-path admission, speculative merge queue, and deterministic self-repair.
- `docs/decisions/ADR-0111-merge-queue-projected-state-fix-at-any-stage.md:48-75,121-137` — merge queue should run projected-state simulation and conflict checks before expensive tests.
- `specs/repo-hygiene-automation.json:276-304` — shared docs/workflows/registries should be pointer-thin and generated from lane-owned shards to reduce shared-file conflicts.
- `registry/artifact-capabilities-registry.json:4-11` and `specs/artifact-profile-defaults.json:21-74,139-241` — artifact registry/profile system is already the natural control plane for per-artifact governance and autogen metadata.

## Inference

The repo currently treats generated faces as both canonical audit artifacts and ordinary PR-owned files. That is why unrelated PRs can be logically independent but mechanically conflict in `accounting-registry.generated.json`, `git-facts.generated.json`, and `gate-baseline.generated.json`. The durable fix is not “ignore generated outputs”; it is to assign every generated artifact a class, owner, materialization mode, merge policy, and verification contract, then let CI/merge-queue own deterministic regeneration at the final candidate tree.

## Decision / ADR Section

### Decision

Adopt a **Generated Artifact Control Plane (GACP)**:

1. A machine-readable manifest/registry classifies every generated artifact family.
2. PRs edit source-of-truth shards and policies, not monolithic generated aggregates by default.
3. Generated-only merge conflicts are resolved by a trusted controller by discarding conflicted generated outputs and regenerating from the merged source tree.
4. Final merge candidates must contain generated outputs byte-equal to regeneration, or a signed immutable CAS artifact referenced by a committed manifest.
5. Monolithic generated aggregates become main-branch materializations or sharded projections, not multi-writer PR surfaces.

### Drivers

- Reduce rebase/regenerate churn under high parallelism.
- Preserve `registry-drift` tamper-evidence and baseline-ratchet safety.
- Productize a pattern usable by any repo.

### Alternatives Considered

- Gitignore everything generated — rejected unless replaced by signed CAS manifest and equivalent retention.
- `merge=union` / `merge=ours` for `*.generated.json` — rejected because semantic conflicts and stale rows can be masked.
- Manual PR-by-PR conflict resolution — rejected as non-scalable process failure.
- Keep current committed monoliths indefinitely — rejected for throughput, accepted only as short-term bridge.

### Why Chosen

Queue-owned materialization is the smallest change that addresses the root cause while preserving audit guarantees. Sharding then reduces the size/frequency of generated deltas and makes independent lanes genuinely independent. External CAS is optional only after the committed manifest contract is strong enough.

### Consequences

- CI becomes a topology gate plus drift gate, not just a byte-diff after-the-fact gate.
- Generated artifact policy becomes explicit and reusable.
- Some generator code and docs must migrate from “commit the faces in every PR” to “PR validates generated output; controller materializes final outputs.”
- Baseline growth and generator/gate changes need special linearized protocols.

## Artifact Classes

| Class | Tracked? | Merge policy | CI authority |
|---|---|---|---|
| `ephemeral-build-output` | No | Gitignored | Build/test only |
| `review-artifact` | No, uploaded | Immutable CI artifact, not source | Informational unless manifest says otherwise |
| `authoritative-source` | Yes | Normal source merge with owner review | Tests/gates validate |
| `append-only-ledger` | Yes | Union only with invariant check | No deleted rows / schema check |
| `sharded-projection` | Yes, per shard | Disjoint shard ownership; regenerate shard | Shard parity + root digest |
| `main-materialized-aggregate` | Yes, bot/controller-owned | No human PR edits; regenerate final tree | Final-tree parity |
| `externalized-canonical-artifact` | Manifest tracked, blob in CAS | Manifest merge; CAS immutable | Signature, digest, retention, fail-closed fetch |

## Implementation Steps

### Phase 0 — Inventory and classification

1. Add or extend generated-artifact metadata in `registry/artifact-capabilities-registry.json` / a dedicated generated-artifacts registry.
2. Enumerate tracked generated files, including the eight currently tracked `*generated.json` faces.
3. Classify each artifact by class, generator command, input globs, owner, materialization mode, and merge policy.
4. Add a linter that fails on unclassified generated artifacts.

### Phase 1 — Safe generated-only conflict resolver

1. Implement `oya generated plan` to identify generated artifact families and source inputs from the manifest.
2. Implement `oya generated merge-resolve`:
   - If source files conflict: fail and send PR to fix-loop.
   - If only generated files conflict: remove conflict markers, regenerate from merged source tree, stage regenerated outputs.
   - If `gate-baseline.generated.json` grows without `gate-baseline.signoff.json`: fail closed.
3. Wire the merge queue/controller to invoke the resolver before expensive CI.
4. Keep current `registry-drift` blocking after resolution.

### Phase 2 — PR-mode CI without branch-owned aggregate churn

1. Change generated checks to run generator outputs into a temp directory for PR validation and upload diff artifacts.
2. Keep failing on stale/hand-edited protected faces while the bridge mode exists.
3. Add a mode flag: `pr-validate`, `merge-candidate`, `main-materialize`.
4. Document that normal contributors do not author generated aggregate conflicts; the controller regenerates final candidates.

### Phase 3 — Main-only materialization lane

1. Add a trusted bot/controller lane for `main-materialized-aggregate` artifacts.
2. On merge queue candidate, regenerate and sign the final materialization commit or synthesize it inside the merge candidate.
3. On `dev` push, run a postsubmit materialization drift check and open/fix a bot PR if drift is detected.
4. Preserve the single `oya-ci-required` fan-in check; do not create a second CI authority.

### Phase 4 — Shard hot generated surfaces

1. Split `accounting-registry.generated.json` into stable path-hash or lane-owned shards plus a root manifest/digest.
2. Split `git-facts.generated.json` or make it a boundary artifact consumed from CI/main materialization rather than a normal PR-owned blob.
3. Keep `gate-baseline.signoff.json` human-authored; make `gate-baseline.generated.json` shrink-only and controller-materialized.
4. Prove aggregate equivalence against the old monolithic output before changing gate consumers.

### Phase 5 — Productize across repos

1. Package a reusable schema and CLI:
   - `oya generated init`
   - `oya generated check --mode pr|merge-candidate|main`
   - `oya generated materialize`
   - `oya generated merge-resolve`
2. Package a GitHub composite action that preserves consumer-owned check names and the single fan-in pattern.
3. Add docs for adoption in any repo: manifest, generator command, artifact classes, failure modes, merge queue integration, and audit evidence.
4. Add reference tests/simulations for 20+ concurrent PRs.

## Acceptance Criteria

1. Two PRs touching disjoint source paths do not require humans/agents to resolve generated JSON conflicts.
2. Final merge candidates still pass `registry-drift` or stronger signed-manifest parity over the final tree.
3. Hand-edited generated faces fail closed.
4. Stale generated faces fail closed.
5. Baseline growth without explicit signoff fails closed.
6. Shallow/partial history generation fails closed for SCM-facts-dependent artifacts.
7. Candidate changes to generators/gates/workflows cannot validate their own weakening without a trusted migration protocol.
8. Every generated file is declared in the artifact control plane; undeclared `*.generated.*` fails CI.
9. The single required fan-in remains `oya-ci-required`.
10. A new repo can adopt the pattern through config/templates rather than bespoke policy.

## Verification Steps

### Unit

- Manifest parser rejects unknown artifact class / missing generator / missing owner / missing materialization mode.
- Merge resolver refuses source conflicts and resolves generated-only conflicts by regeneration.
- Baseline growth detection refuses unsigned growth.
- Deterministic output tests verify sorted keys, stable ordering, no wall clock, trailing newline.

### Integration

- Run current accounting producer in temp-dir mode and compare against committed faces.
- Simulate generated-only conflicts and prove `merge-resolve` regenerates byte-identical outputs from the merged source tree.
- Run existing `registry-drift` after controller materialization.
- Run `buck2` build/test targets for generated tooling.

### E2E / Queue Simulation

- Create 20+ synthetic branches touching disjoint tracked files.
- Queue them through projected merge state.
- Verify generated conflict rate drops to zero for generated-only conflicts.
- Verify true source conflicts are still parked/refused.
- Measure queue wait, regen time, cache hit rate, and total PR throughput.

### Observability

Emit metrics/events:

- `generated_artifact_conflicts_total{class,family}`
- `generated_artifact_auto_resolved_total{family}`
- `generated_artifact_regen_seconds{family,mode}`
- `generated_artifact_drift_failures_total{reason}`
- `merge_queue_generated_settle_seconds`
- audit-chain event for controller materialization with input/output digests.

## Pre-mortem

1. **Audit regression**: Generated faces are untracked too early and drift becomes invisible. Mitigation: no externalization without committed root digest, signature, retention, fail-closed fetch, and parity gate.
2. **Semantic masking**: A broad merge driver silently keeps stale baseline or duplicate rows. Mitigation: never use broad `*.generated.json` union/ours as authority; only controller regen from source tree is allowed.
3. **Generator self-weakening**: A PR changes the generator and validates itself green. Mitigation: generator-change protocol with old-vs-new comparison, fixtures, trusted base/controller validation, and linearized migration.

## Follow-up Staffing Guidance

Available roles from current catalog: `architect`, `planner`, `critic`, `executor`, `test-engineer`, `verifier`, `code-reviewer`, `code-simplifier`, `git-master`, `researcher`, `dependency-expert`, `writer`, `debugger`.

Recommended Team + Ultragoal split:

- Lane A (`architect`, high): finalize artifact-class schema and migration invariants.
- Lane B (`executor`, medium): implement generated manifest parser and `oya generated plan/check`.
- Lane C (`executor`, medium): implement generated-only merge resolver.
- Lane D (`test-engineer`, medium): build synthetic parallel-PR simulation and adversarial tests.
- Lane E (`executor`, medium): wire CI modes and composite-action template.
- Lane F (`writer`, high): update quick-start docs and ADR/productization guidance.
- Review lane (`code-reviewer` + `critic`, high): verify no audit invariant was weakened.
- Shipping lane (`verifier` + `git-master`, high): evidence, PR sequencing, signed commits, merge-readiness checks.

Launch hints:

```text
$ultragoal .omx/plans/generated-artifact-control-plane-20260609.md
$team 6 --plan .omx/plans/generated-artifact-control-plane-20260609.md
```

Team verification path:

1. Team proves unit/integration/simulation gates locally or explains exact gaps.
2. Ultragoal checkpoints artifact schema, CLI, CI wiring, docs, and evidence as separate completion claims.
3. Code review and code simplification run before shipping.
4. Shipping verifies CI, reviewer approval, signed commits, and governance gate readiness before merge.

## Changelog

- Incorporated architect sidecar requirement for shard ownership, main-only materialization, and topology checks.
- Incorporated planner sidecar requirement to keep tracked/audited canonical outputs unless replaced by equivalent signed manifest/CAS.
- Incorporated critic sidecar rejection of weak fixes: gitignore-only, broad merge drivers, manual regenerate loops, baseline auto-growth, and candidate self-validation.
