# Landing Plan — Wave 15 → origin/dev (2026-05-21)

**Single source of truth for landing this session's work into `origin/dev`.** Maintainer: orchestrator session 8f603fc7. Status: ACTIVE.

## §0. Pre-landing state (snapshot 2026-05-21)

| Metric | Value |
|---|---|
| Local branch | `post-merge-2026-05-18` |
| Local HEAD | `0122c1e6` |
| origin/dev HEAD | `0e9105ac` |
| Commits ahead of origin/dev | 100 |
| Commits behind origin/dev | 29 |
| Tracked modified files | 5,547 |
| Untracked files | 3,184 |
| Total uncommitted artifacts | ~8,731 |

## §1. Authority chain for landing decisions

- ADR-0110 changeset-state-machine
- ADR-0111 merge-queue-projected-state
- ADR-0112 webhook-driven foundry-agent invocation (replace with intelligence per ADR-0335)
- ADR-0113 vcs-orchestrator end-to-end
- `feedback_foundry_pipeline_canonical` (now retired per ADR-0335; agentic landing via `oya git` + `oya vcs` until pipeline post-retirement design lands)
- `feedback_oya_git_canonical_2026_05_18` (canonical git invocation)
- ADR-0221 hooks are GUIDANCE; CI gates enforce
- root `CLAUDE.md` foundry-pipeline-required-workflow (layer 0 isolation → layer 2 PR → admission gate → merge queue → completion gate)

## §2. Race conditions identified

| origin/dev commit | Conflict surface | Strategy |
|---|---|---|
| #175 (21dba101) Align cloud data cache to Valkey | `microservices/cloud-data/` | 3-way merge: take our broader Valkey doctrine + their specific edits; resolve before our Wave 15P Valkey PR lands |
| #174 (8fc74ea4) capacity management provider-neutral | `microservices/cloud-data/` | possible overlap with our cloud-data IMPL-truth-up; verify |
| #172 (4cbaa157) VM provisioning provider-neutral | `microservices/cloud-compute-vm-*` | low overlap; verify |
| #173 (aeb6bbf5) K8s + Functions API entrypoints | `microservices/cloud-compute-functions-*` | low overlap |
| #176 (0e9105ac) regional tax invoice policy | `microservices/cloud-billing-tax/` | check vs our Wave 15B cloud-billing spec sprint + V-BUCKET-7 Valkey rewrite |

## §3. Phased landing sequence

### Phase 1 — Doctrine checkpoint commit (immediate, low conflict)

**Trigger**: WAVE-D D-1 schema verification completes (no race risk; schema files don't overlap with origin/dev recent commits).

**Scope** (~35-50 files):
- `docs/decisions/ADR-0337-iceberg-canonical-olap-write-path.md`
- `docs/decisions/ADR-0338-pod-runtime-tier-0-to-3.md`
- `docs/decisions/ADR-0339-shared-iac-module-library.md`
- `docs/decisions/ADR-0340-capacity-model-per-microservice-manifest.md`
- `docs/decisions/ADR-0341-cellular-promotion-gates-explicit-tier-criteria.md`
- `docs/decisions/ADR-0342-api-versioning-hybrid-date-public-semver-sdk.md`
- `docs/decisions/ADR-0343-dr-rto-rpo-matrix-per-microservice-per-compliance-pack.md`
- `docs/decisions/ADR-0344-sustainability-finops-dimensional-model.md`
- `docs/decisions/ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md`
- 4 NEW spec files:
  - `specs/compliance-pack-floors.json`
  - `specs/oss-stewardship-registry.json`
  - `specs/finops-dimensional-model.json`
  - `specs/audit-event-schema.json`
  - `specs/iac-module-library.json` (if D-1 confirms or scaffolds)
- `specs/master-plan-sequencing.json` (14 sub-wave landings)
- `specs/microservices/manifest-schema.json` (5 new field blocks: capacity_model + dr + pod_runtime_tier + tenant_version_pinning + oss_stewardship_class)
- `tools/hooks/_canonical-primitives.md`
- `docs/standards/dependency-policy.md` (§7 substitutions + §11 OSS stewardship)
- `docs/GLOSSARY.md`
- `docs/machine-readable/glossary.json`
- 2 state files (canonical):
  - `.omc/state/oyatie-architecture-2026-05-21.md`
  - `.omc/state/audit-doctrine-2026-05-21.md`
- `.omc/state/landing-plan-2026-05-21.md` (this file)
- `.omc/state/wave-14-aggregation.md`
- `.omc/state/wave-15-progress-2026-05-21.md`
- `.omc/state/wave-15-ca-verify-2026-05-21.md`
- `.omc/state/wave-15-ca-verify-workspace-2026-05-21.md`
- `.omc/state/wave-d-naming-normalization-2026-05-21.md`
- 8+ memory files at `~/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_*.md` (note: outside repo; tracked separately)

**Commit message** (HEREDOC):
```
Wave 15 doctrine bundle: 9 ADRs + 5 specs + canonical primitives

ADR-0337 Iceberg canonical OLAP write path
ADR-0338 Pod runtime tier 0..3 (Kata for Tier 0/1; runc for Tier 2/3)
ADR-0339 Shared IaC module library
ADR-0340 Capacity model per µservice manifest
ADR-0341 Cellular promotion gates explicit Tier 0..4 criteria
ADR-0342 API versioning HYBRID (date public + semver SDK)
ADR-0343 DR + RTO/RPO matrix per-µservice + per-compliance-pack
ADR-0344 Sustainability + FinOps dimensional model
ADR-0345 OSS stewardship class + CVE-response SLA

Constraint: nine independent doctrine decisions; per-µservice propagation deferred to WAVE-D sub-waves 15P-15Y.
Rejected: monolithic doctrine ADR (too large to review).
Confidence: high.
Scope-risk: narrow (pure additions; no µservice rewrites).
Directive: WAVE-D propagation lands as separate PRs per sub-wave.
Tested: jq empty on all JSON; ADR template lint via documentation-and-adrs skill; cross-references verified.
Not-tested: per-µservice manifest field adoption (sequenced in 15P-15Y).
```

**Pre-commit verification**:
- `jq empty` on all 5 new spec files + manifest-schema + master-plan-sequencing → PASS
- `grep` for stale ADR-0316/ADR-0220 references in the 9 new ADRs → PASS
- Confirm 14 sub-wave landings in master-plan-sequencing.json
- Confirm canonical-primitives.md has Pod Runtime + Cell Promotion + API Versioning + IaC Library sections

**Estimated review effort**: 2-4 hours (focused doctrine review)

### Phase 2 — Pull origin/dev + reconcile cloud-data (immediate after Phase 1)

```bash
oya git fetch origin dev
oya git merge-base HEAD origin/dev  # find ancestor
# Identify conflict files:
oya git diff --name-only origin/dev -- microservices/cloud-data/
```

Strategy:
- Take origin/dev #175 cache-surface alignment as baseline
- Layer our broader corpus Valkey vocabulary on top (where they differ)
- For substantive code edits, take their version (they're closer to production-tested)
- For PRD/ARCH/IP prose, take our version (broader doctrine coverage)

### Phase 3 — WAVE-D D-2/D-3/D-4 dispatch (after Phase 2)

Now writing onto a clean base + with doctrine committed.

### Phase 4 — Topic-grouped corpus commits

| # | PR title | Scope | Estimated files |
|---|---|---|---:|
| PR-2 | Wave 15I foundry retirement | ADR-0335 + foundry/RETIRED.md + intelligence absorb + Hermes drop + 8 structural updates | ~50 |
| PR-3 | Wave 15O shorts → social merge | ADR-0334 + social PRD expanded + shorts retire (~80 absorbed/deleted) | ~100 |
| PR-4 | Wave 15K network → community | network/RETIRED.md + community absorb | ~30 |
| PR-5 | Wave 15L cell retirement | oya-shuffle-sharding crate + 6-µservice absorb | ~40 |
| PR-6 | Wave 15M healthcare decomposition | emr + diagnostics + emergency + pharmacy + patient-monitoring + imaging + healthcare-integration narrowing | ~17,400 lines / ~200 files |
| PR-7 | Wave 15A Big-8 rewrites | crm + marketing-automation + CLM + itsm + performance-management | ~3,500 lines |
| PR-8 | Wave 15P Valkey corpus migration | ~470 files corpus-wide (after Phase 2 reconciliation) | ~470 |
| PR-9 | Wave 15J tier-scrub corpus removal | 4 codex batches (15J-1..15J-4) + residue mop-up + final cleanup | ~1,000 |
| PR-10 | Wave 15-IP-substance | stamped→bespoke conversion (~454 IPs rewritten) | ~1,200 |
| PR-11 | Wave 15-IMPL-truth-up | tenancy/audit-chain/payments/data-warehouse/cloud-billing crate scaffolds | ~80 |
| PR-12 | WAVE-D propagation outputs | per-µservice manifest + PRD + IP fan-out (after D-2/3/4 lands) | TBD |

Each PR ≤500 files (reviewer-tractable). Sequential dependency graph documented per PR.

### Phase 5 — Per-PR review + merge

Per ADR-0111 merge-queue projected-state fix:
- PR opens against `dev`
- Foundry pipeline admission gate (Cedar + Kyverno + cosign signature verification)
- Multispectrum review v2.4.0 facets (F1-F11 + M1+M2 + A1-A7)
- Reviewer-agent APPROVE + CI green
- Auto-merge via queue

## §4. Risk register

| Risk | Probability | Mitigation |
|---|---|---|
| origin/dev advances during landing (more commits to rebase against) | HIGH | Land Phase 1 doctrine within 24h; subsequent PRs within 72h |
| cloud-data merge conflict more complex than expected | MEDIUM | Pre-merge dry-run in worktree before pushing |
| WAVE-D D-2/3/4 writes destabilize Phase 1 base | LOW | D-1 schema-lock + D-2 manifest fan-out are append-mostly; conflicts unlikely on doctrine surface |
| Reviewer fatigue across 10+ PRs | MEDIUM | Doctrine PR-1 lowest-effort (focused review); subsequent PRs grouped by sub-wave (single-reviewer-cognitive-load per PR) |
| Memory files outside repo not auto-tracked | LOW | Memory files persist in `~/.claude/projects/-Users-jasonlee-oyatie/memory/`; not committed to repo (intentional per CLAUDE.md memory section) |

## §5. Verification checklist before Phase 1 commit

- [ ] D-1 manifest-schema verification PASS
- [ ] All 9 ADR files exist + line counts ≥600
- [ ] All 5 new spec files valid JSON
- [ ] master-plan-sequencing.json valid + 14 sub-wave landings present
- [ ] No `15-X-name` style keys remaining (D-0 normalized to 15{P..Y})
- [ ] tools/hooks/_canonical-primitives.md updated for each ADR
- [ ] No accidental commits of OLD-name keys
- [ ] git diff --check passes
- [ ] cargo check --workspace PASS (post foundry retirement transition debt)
- [ ] Pre-commit hook results green (lints + JSON validation)

## §6. Post-landing tasks

- [ ] Update `.omc/state/wave-15-progress-2026-05-21.md` with Phase 1 LANDED status
- [ ] Update memory `feedback_realignment_review_findings_2026_05_21.md` with landing pointer
- [ ] Tag commit with `wave-15-doctrine-bundle-2026-05-21`
- [ ] Open PR-1 against `origin/dev` with title "Wave 15 doctrine bundle (ADR-0337..0345)"
