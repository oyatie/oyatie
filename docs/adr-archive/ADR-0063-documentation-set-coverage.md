---
id: ADR-0063
status: Superseded
superseded_by: [ADR-709]
doc_status: published
---

# ADR-0063: Documentation set coverage — every planned feature ships with a complete doc set, CI-enforced

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0001, ADR-0011, ADR-0056, ADR-0058, ADR-0059, ADR-0060, ADR-0062, ADR-0064

---

## Context

Per user instruction 2026-05-13: "Ensure that every planned feature makes it into the master plan with all the necessary and full documentation set following. Make sure this is enforced."

Oyatie's autonomous-implementation charter (per `feedback_autonomous_decision_principles.md`) requires that a phase ships complete or doesn't ship. "Complete" means **code + docs + tests + evidence land in the same commit** — no stubs, no placeholders, no deferrals.

Templates already exist at `docs/templates/`: ADR, PRD, microservice record, bounded-context registration, phase-spec, impl-plan, milestone-readme. What was missing: an enforced contract that every µservice ships with the full set, not just an ad-hoc subset.

This ADR closes that gap.

---

## Decision

### 1. The canonical artifact set

For every µservice registered in `[workspace.metadata.oya.microservices]`, the following artifacts MUST exist before the µservice's introducing-phase can pass its exit gate:

| Artifact | Path convention | Template |
|---|---|---|
| Microservice record | `docs/microservices/<microservice>.md` | `docs/templates/microservice-template.md` |
| Product Requirements (canonical, pack-neutral) | `docs/prds/<microservice>.md` | `docs/templates/prd-template.md` |
| Naming-scope ADR | `docs/decisions/ADR-####-microservice-<microservice>.md` | `docs/templates/adr-template.md` |
| Bounded-context registrations (one per BC) | `docs/bounded-contexts/<microservice>-<bc>.md` | `docs/templates/bounded-context-registration-template.md` |
| Phase-Specs (≥1 referencing the µservice) | `.omc/plans/milestones/M*/phases/*/phase-spec.md` | `docs/templates/phase-spec-template.md` |
| Impl-Plans (one per IP) | `.omc/plans/milestones/M*/phases/*/impl-plan.md` | `docs/templates/impl-plan-template.md` |

### 2. The per-localization-pack overlay suite

For every (pack × µservice) pair declared in `docs/localization-packs/<pack>/pack.yaml > microservices_in_scope` (per ADR-0064), required artifacts depend on the `material_scope` flag set in `pack.yaml`:

| Artifact | Path convention | When required |
|---|---|---|
| Pack overlay PRD | `docs/prds/<microservice>-<pack>.md` | `material_scope: true` |
| Pack regulatory ADR | `docs/decisions/ADR-####-<pack>-<microservice>-regulatory.md` | always (every pack-scoped µservice has at least one regulatory binding) |
| Pack acceptance evidence | `docs/localization-packs/<pack>/evidence/<microservice>.md` | always |

`pack.yaml` is the authoritative source for the (pack × µservice × material_scope) tuple; the CI lane reads it directly. Pack.yaml is the single source of truth — no parallel hand-maintained tables.

### 3. Per-milestone artifacts

| Artifact | Path convention |
|---|---|
| Milestone README | `.omc/plans/milestones/M<NN>-<slug>/README.md` |
| Acceptance evidence bundle | `.omc/plans/milestones/M<NN>-<slug>/acceptance-evidence/` |

### 4. Section-completeness checks

Every PRD MUST contain:

- `## Competitive Benchmark` (per ADR-0062 quality bar)
- `## Performance Targets` (per ADR-0062 perf bar)
- `## Horizontal Scalability` (per ADR-0062 scale bar)
- `## Bounded Contexts` enumerating each BC the µservice owns

Every Phase-Spec frontmatter MUST declare:

- `acceptance_lanes:` (list of fitness lanes the phase must green)
- `depends_on:` (predecessor phase list)
- `entry_gate:` and `exit_gate:` prose

Every Impl-Plan MUST contain:

- `## Concrete File Targets` (table)
- `## Code Shape` (port traits, DDL, key types)
- `## Acceptance Gates` (cargo + fitness lane invocations)
- `## Load test` (k6 / locust / vegeta script meeting the µservice perf targets)
- `## Grit Claim Symbols` (list of symbols + TTL)
- `## ICM Rows to Emit` (phase-start, phase-complete)

### 5. Enforcement — `oya-check-documentation-cli` (LEAN-A5)

Crate: `crates/oya-check-documentation/` (BNF-exempt; `oya-check-*` namespace).

Lane registered in `registry/quality/lanes.yaml` as `lean-a5-documentation`. Runs on every PR.

Algorithm:

1. Read `[workspace.metadata.oya.microservices]` for canonical µservice list (registered set).
2. Read `docs/MASTERPLAN.md` §2.1 catalog for planned µservice list (planned set). Reconcile: every planned µservice MUST appear in workspace metadata once it has a Phase-Spec referencing it (i.e., once it's claimed by an introducing phase); planned-only µservices with no Phase-Spec are exempt from §1 enforcement but logged.
3. For each µservice in the registered set, verify every row in §1 exists; collect missing as violations.
4. Read `docs/localization-packs/INDEX.md` for catalog of packs; for each pack, read `<pack>/pack.yaml` for `microservices_in_scope`.
5. For each (pack × µservice) pair in `microservices_in_scope`:
   - Always verify the regulatory ADR + acceptance evidence (per §2 table).
   - If `material_scope: true`, also verify the overlay PRD.
6. Per-milestone directory in `.omc/plans/milestones/`, verify §3 exists.
7. Section-completeness: parse markdown of every PRD / Phase-Spec / Impl-Plan; verify §4 sections present and non-empty.
8. **Orphan-scan**: walk `docs/microservices/`, `docs/prds/`, `docs/bounded-contexts/`, `docs/localization-packs/<pack>/evidence/`. For every file, parse the µservice / pack reference and verify it exists in workspace metadata + pack `microservices_in_scope`. Any file referencing a retired µservice / unscoped pack pair is an orphan violation; the µservice or pack-scope was removed without removing the doc.
9. Mode `--report-only` (default until M02-P22): print violations, exit 0.
10. Mode `--blocker` (subsequent-to-M02-completion-P22): exit nonzero if any violation.

Coverage snapshot (auto-emitted): `docs/DOC-COVERAGE.md`.

### 6. Suite-completeness is a phase exit gate

A phase that introduces a new µservice or new BC is **not Complete** until the doc-coverage lane is green for that µservice. Per scope-completion rule (`feedback_autonomous_decision_principles.md`): no stubs, no placeholders, no deferrals.

The doc set ships in the same commit that introduces the µservice. A PR may not split code and docs across commits if the code introduces a new µservice or BC.

### 7. Retired-µservice path

A µservice removed from `[workspace.metadata.oya.microservices]` MUST have its docs physically removed in the same commit (no marked-retired flags). Per `feedback_autonomous_implementation_artifacts.md`: stale information is removed rather than marked as retired. The lane verifies no orphaned doc files reference retired µservices.

---

## Consequences

**Positive:**

- Autonomous-execution agents have a deterministic doc contract to satisfy. They cannot mark a phase complete with partial docs.
- New µservices cannot land without naming justification, PRD with benchmarks, BC registrations, and ADR — eliminating the "code merges then docs lag" antipattern.
- Localization packs (per ADR-0064) get a first-class doc surface; pack-specific regulatory work has a home and an enforcement gate.
- The CI lane is the source of truth for coverage; `docs/DOC-COVERAGE.md` becomes a snapshot, not a hand-maintained table that drifts.

**Negative:**

- Substantial up-front authoring cost for the 59 canonical µservices currently in the catalog with incomplete docs. Mitigation: parallel-dispatch executors per cluster (Healthcare / Enterprise / FinTech / Industrial / Substrate / Cloud / Personal) to author the missing PRDs/ADRs/microservice records under the canonical templates.
- The lane runtime grows with µservice count; budget at 600s for the full workspace (with caching on unchanged directories).

**Neutral:**

- The lane is `--report-only` until M02-P22 (so M02 substrate work can proceed without doc-coverage blocking), then BLOCKER thereafter. This mirrors the LEAN-A1..A4 ramp pattern (per M01-P05 hardening).

---

## Compliance

CI lane: `lean-a5-documentation`.

Owner team: `council-architecture` (matrix authority) + `axis-foundry` (lane implementation).

First green window: M02-P22 exit gate (lane flips to BLOCKER).

---

## Deliberate-mode pre-mortem (3 scenarios)

Per RALPLAN-DR deliberate mode, three concrete failure scenarios with triggers, blast radius, prevention, detection, rollback:

### Scenario 1: Lane never lands as BLOCKER

- **Trigger**: M02-P22 exit gate slips; lane stays `--report-only` indefinitely.
- **Blast radius**: New µservices land without docs; coverage erosion compounds; by M04-onward the doc-debt is unrecoverable.
- **Prevention**: M02-P22 phase-spec + impl-plan explicitly list `cargo run -p oya-check-documentation -- --workspace --blocker` in BLOCKER command list (per `.omc/plans/milestones/M02b-substrate/phases/P22-m02-exit-gate/phase-spec.md` + `impl-plan.md`). The CLI exits nonzero only when `--blocker` is set (`crates/oya-check-documentation/src/main.rs:38`); removing `--report-only` alone leaves the lane permissive, so the explicit `--blocker` flag is the load-bearing invariant. M02-P22 cannot pass its exit gate without doc-coverage going green under `--blocker` or being explicitly waived per ADR.
- **Detection**: Weekly report comparing previous-week violation count vs current; rising count = warning, doubled count = page.
- **Rollback**: Flip lane back to `--report-only` and open a remediation phase before re-flipping.

### Scenario 2: Lane gives false positives — agents start ignoring it

- **Trigger**: Orphan-scan over-flags (e.g., template doc files like `prd-template.md` get classified as orphans).
- **Blast radius**: Agents silence the lane in CI; real violations leak through; trust collapses.
- **Prevention**: Whitelist `*-template.md`, `INDEX.md`, `README.md`, `MASTERPLAN.md`, `DOC-COVERAGE.md`, `RETIRED.md`, and the entire `docs/templates/` and `docs/standards/` subtrees in the orphan-scan step. Unit-test the whitelist against the live repo's actual templates.
- **Detection**: Per-PR coverage delta — if a PR doesn't touch docs but the violation count changes, flag for investigation.
- **Rollback**: Add whitelist entries; re-run lane.

### Scenario 3: pack.yaml drifts from kr.md / DOC-COVERAGE.md / INDEX.md

- **Trigger**: A new µservice is added to `kr.md` but not to `kr/pack.yaml`, or `material_scope` is set in one and not the other.
- **Blast radius**: Lane reports pass against pack.yaml, but human-readable docs claim broader coverage than CI verifies. Tenants see promises that aren't enforced.
- **Prevention**: Lane treats `pack.yaml` as authoritative (already in algorithm §5 step 4). Add a parity check: lane re-renders the expected `## µservice scope` table from `pack.yaml` and compares to `kr.md` §2 + `INDEX.md` scope summary; mismatch is a violation. CI fails if any pack overview doc diverges from its manifest.
- **Detection**: Parity check runs on every PR.
- **Rollback**: Regenerate `kr.md` scope section from `pack.yaml`; commit.

## Expanded test plan

| Tier | Coverage | Fixture / harness |
|---|---|---|
| **Unit** (parser correctness) | `read_workspace_microservices` returns the keys of `[workspace.metadata.oya.microservices]`; `read_masterplan_catalog` extracts kebab-case tokens from §2.1; `read_pack_catalog` discovers every `pack.yaml`; `has_naming_adr` matches the `ADR-####-microservice-<ms>.md` pattern. | `crates/oya-check-documentation/tests/smoke.rs` (already has 2 tests); planned per-module unit tests in M02-P20. |
| **Integration** (end-to-end against synthetic repo) | Synthesize a tmp dir with: (a) Cargo.toml registering 3 µservices, (b) 1 PRD authored, (c) 1 pack.yaml with `material_scope: true`. Verify the report contains exactly the expected violation kinds. | `tests/integration/` fixtures (M02-P20 scope). |
| **E2E** (lane in CI) | The `.github/workflows/ci-fitness-lanes.yml` job runs the binary against `HEAD` of the actual workspace; archives the markdown report. Pre-M02-P22: `--report-only` (exit 0). Post-M02-P22: `--blocker` (exit nonzero if violations). | `.github/workflows/ci-fitness-lanes.yml` (M02-P20 scope). |
| **Observability** | The markdown report is the canonical observability surface; total violation count is emitted as a Prometheus gauge `oyatie_doc_coverage_violations{kind="..."}` for trend tracking. | M02-P20 scope; gauge wired into VictoriaMetrics per Bominal ADR-0020. |

## Deterministic re-verification recipe

To re-verify this consensus in 1 week (or N weeks) without conversation context, run:

```bash
git rev-parse HEAD                                                # confirm working commit
cargo run -p oya-check-documentation -- --workspace --report-only  # exit 0; lane operational
cargo test -p oya-check-documentation                              # 2/2 pass
rg -nP "oya-check-documentation" registry/quality/lanes.yaml      # lean-a5 lane registered
rg -nP "## Architect verdict" docs/MASTERPLAN.md || echo no-consensus-block-in-masterplan
diff <(rg -nP '^\s*-\s+`(\w[\w-]*)`' docs/MASTERPLAN.md | grep -oE '`\w+[\w-]*`' | tr -d '`' | sort -u) \
     <(cargo run -p oya-check-documentation -- --workspace --report-only 2>/dev/null | grep -oE 'pack regulatory ADR for \([a-z]+, [a-z-]+\)' | grep -oE '[a-z-]+\)' | tr -d ')' | sort -u) \
     || echo masterplan-catalog-vs-pack-scope-diverged
icm recall -t context-oyatie -q "ralplan masterplan consensus" --limit 3
```

Expected outputs documented in `docs/standards/ci-lanes.md`. Re-runs against the same commit produce identical reports (modulo path normalization).

## References

- ADR-0056 (BNF v4.1; `[workspace.metadata.oya.microservices]` source of truth)
- ADR-0058 (flat µservice catalog)
- ADR-0062 (quality/performance/scalability bar; mandates PRD sections)
- ADR-0064 (canonical base + localization packs; per-pack overlay requirement)
- `docs/MASTERPLAN.md` §13.5 (Documentation set coverage)
- `docs/templates/` (canonical artifact templates)
- `feedback_autonomous_decision_principles.md` (scope-completion rule)
- `feedback_autonomous_implementation_artifacts.md` (stale removed in reality)
