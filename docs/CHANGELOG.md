---
purpose: Oyatie — Canonical Docs Changelog
doc_status: published
---

# Changelog

## 2026-08-03 — W0-D non-authorizing reset-eligibility gate

- Added the W0-D eligibility-only schema, historical discovery observation, and
  fail-closed Rust policy gate under acceptance authority issue #1535.
- Reset authorization remains unconditionally disabled: the candidate records
  hard stops, unknowns, recovery gaps, approval failures, and stale evidence but
  exposes no destructive API, controller, nonce, or actuation path.
- Removed syntactic HTTP(S) URIs from the secret-scanner exemption. Public URLs
  can still pass ordinary entropy analysis, while secret-like userinfo, query,
  fragment, and path values are rejected by red/green regression coverage.
- The committed 2026-08-01 discovery artifact expired on 2026-08-02 and is
  historical, unverified, and non-authorizing; any future decision requires a
  fresh reviewed capture. Protected admission and post-merge evidence remain
  pending in PR #1524.

## 2026-08-03 — ADR-0635 bounded graph-v2 authority repair

- Restored graph-v2 regression coverage for self-loop, two-node, three-node, and buried six-node
  SCC cycles; present forbidden edges; invalid bootstrap ordering; deterministic Kahn output;
  valid nonalphabetical topology; wrong gate IDs; and parent/absolute policy-path escapes.
- Retired the standalone v1 cycle fixtures after adapting their failure classes into the
  live-document graph-v2 mutation corpus. Their bytes remain explicitly marked inert only for the
  baseline producer's merge-base path replay; no gate loads them.
- Dispositioned ADR-0280 §D-13.G as a 24-capability target model rather than current coverage,
  enumerated the 13 omitted capabilities and missing B0 hosting-chain faces, and bound completion
  to `W0-C-TOPOLOGY-COVERAGE` ([#1537](https://github.com/jason931225/oyatie/issues/1537))
  without a new frozen baseline.
- Marked `specs/platform-architecture.json`'s v1 topology block stale with
  `current_parity_claim: false`; `/specs/substrate-dependency-dag.json` remains the sole current
  machine topology authority for its bounded 19-unit/11-capability slice.
- ADRs cited: ADR-0280, ADR-0562, ADR-0615, ADR-0635.
- Protected admission and post-merge completion evidence remain pending.

## 2026-08-03 — Reorg manifest selector authority restored

- **doc.mistakes_ledger** (Tier 2, updated): recorded `MFL-0019` for duplicated
  caller-side first-sorted move-plan selection that bypassed the reorg codemod's
  active-versus-landed selector.
  - The crate-registration and registry-drift callers now omit `--plan` and leave
    PARKED exclusion, landed-plan filtering, multi-active failure, and zero-active
    canonical-empty behavior to the authoritative codemod.
  - Related lanes: `ci-crate-registration`, `ci-inventory-registry-drift`,
    `oya-reorg-codemod-app`.
  - Protected admission and post-merge product-completion evidence remain pending.

## 2026-07-24 — ADR-0624 Accepted immutable ADR census epoch transition

- **doc.adr_index** (Tier 1, added): accepted the four-step protected merge train
  for exact historical P2 replay, dormant P3 protection, later pointer-only P3
  activation, and post-proof P2 cleanup.
  - The mutable control plane selects only an already-protected epoch; the
    protected producer and gate own and derive parser, selector, execution, and
    predecessor identity.
  - The receipt core is content-addressed and squash-stable, excludes commit and
    full-tree self-reference, keeps generated faces producer-only, and preserves
    `BLOCKED/HOLD`, `planning_impact: false`, and `HOLD(Planning)`.
  - Old implementation and receipt bytes remain only in Git history; no readable
    archive directory is allowed.
  - Authors: `@jason931225`
  - ADRs cited: ADR-0515, ADR-0525, ADR-0552, ADR-0595, ADR-0597, ADR-0613,
    ADR-0619, ADR-0623, ADR-0624
  - Related lanes: `oya-check-adr-index`, `oya-governance-adr-shape`,
    `ci-cross-artifact-agreement`, `ci-scm-facts-snapshot`
  - Protected admission and post-merge product-completion evidence remain
    pending and will be recorded in the PR evidence packet.

## 2026-07-24 — ADR-0623 Proposed mechanism-neutral Stage-1 evidence epoch

- **doc.adr_index** (Tier 1, added): recorded ADR-0623 as a Proposed,
  planning-neutral description of the C01–C15, A–G, sixteen-lens, fresh-dissent,
  immutable-successor, and context-free-exit evidence population while preserving
  `HOLD(Planning)` and keeping the historical executable prototype only in Git history.
  - Authors: `@jason931225`
  - ADRs cited: ADR-0623
  - Related lanes: `oya-check-adr-index`, `oya-governance-adr-shape`,
    `ci-cross-artifact-agreement`
  - PR: #1364; exact-head admission, protected squash/merge,
    `oya-ci-required`, and post-merge evidence remain pending and will be
    recorded in the PR evidence packet.

## 2026-07-24 — ADR-0622 Proposed FixupTask v2 successor foundation

- **doc.adr_index** (Tier 1, added): recorded ADR-0622 as a Proposed,
  planning-neutral successor-foundation decision while keeping its executable
  prototype out of the current authority tree and preserving `HOLD(Planning)`.
  - Authors: `@jason931225`
  - ADRs cited: ADR-0622
  - Related lanes: `oya-check-adr-index`, `oya-governance-adr-shape`,
    `ci-cross-artifact-agreement`
  - PR: #1363; exact-head admission, protected squash/merge,
    `oya-ci-required`, and post-merge evidence remain pending and will be
    recorded in the PR evidence packet.

## 2026-07-13 — Pre-planning authority consolidation

- Reconciled the root authority chain around `specs/root-hub-pointers.json`,
  `docs/AGENTS.md`, `specs/masterplan.json#masterplan_v2`, Accepted ADRs, and live
  Git/GitHub evidence; reduced `HANDOFF.md` to a redirect rather than a parallel plan.
- Recorded a machine-readable Past/Present/Future snapshot, unresolved founder-choice
  matrix, and fail-closed pre-planning dispatch hold without changing the founder-ratified
  dependency graph, work-item order, execution waves, or sequencing digest.
- Added direct, independent determinism coverage for both de-committed controller
  projections and made digestless sequencing ratification fail closed.
- Recorded MFL-0017 for the direct-masterplan determinism gap and its independent
  masterplan-and-product-graph regeneration checks.
- Recorded MFL-0018 for missing or stale sequencing-digest proof and its three-way
  digest plus open-hold approval/dispatch regressions.
- Corrected Markdown-retirement, ADR-lifecycle, and Phase-0 evidence surfaces while
  preserving explicit nonclaims: the candidate is not current authority until protected-PR
  admission, and Phase 0 remains non-green.

## 2026-07-01 — Data Use Boundary policy-gate ownership

- Recorded the first Data Use Boundary policy-gate implementation surface and
  council-privacy ownership seed for `libs/oya-data-boundary-kernel`.

## 2026-07-01 — Cloud-native infrastructure automation standard

- Added `docs/standards/cloud-native-infrastructure-automation.md` as concise
  review guidance for API-shaped Rust/config/controller/gate infrastructure
  automation.
- The standard explicitly rejects new ad-hoc infrastructure CLIs and new Python
  or shell scripts for core infra behavior, and requires configuration-driven,
  idempotent, observable, deployment-compatible automation.
- Added born-accounting registration for the new standard through standards ownership,
  exact reachability entries, and ADR justification pointers.

## 2026-07-01 — Root-of-trust ceremony and cloud trust drift port

- Ported the root-of-trust ceremony runbook and machine-readable redacted evidence contract from the dirty preservation root, registering the cloud runbook in `docs/RUNBOOKS-INDEX.md`.
- Ported the still-current cloud trust/IaC safety slices: sealed `trustd` CA persistence over the real ECDSA signer, and Cloud IaC service/tenant fixture evidence with current `cloud/cloud-iac` paths.
- Discarded old cloud KMS and managed-K8s quota runtime crate resurrection from this lane because those crates are absent from current `origin/dev`; their security intent is recorded on the Kanban handoff for follow-up on current destination surfaces.
## 2026-07-01 — WORKSPACE-DRIFT-FOLLOWUP-C docs/procedure reconciliation

- Ported the still-valid docs/procedure subset from the preserved dirty workspace: explicit PHASE-5 promotion evidence wording, ADR-0515 `oya-ci-required` merge authority, and release-governance/release-note impact language.
- Discarded stale dirty changes that would reintroduce retired multispectrum evidence files, local `oya` or legacy CI merge authority, unreviewed PR-template rewrites, or absent generated-face helper paths.

## 2026-06-30 — DEVFLOW-003 post-merge product-completion gate

- Added post-merge product-completion packet requirements to root agent guidance,
  the operating contract, done-definition/pre-merge/review checklists, PR templates,
  and release standards.
- Product-complete now requires promoted-sha `oya-ci-required`, rollout verification,
  rollback note, observability check, browser/user-story evidence, and
  release-governance/release-note impact after squash merge; Release Please is
  only mandatory when a live repo config/workflow exists.

## 2026-06-30 — ADR-0536 substrate-to-port matrix

- Added a shape-neutral D-1..D-16 substrate-to-port contract matrix to ADR-0536 so downstream product fanout locks owned port/contract seams, treats `oya-*` / `cloud-*` names as migration aliases, and preserves explicit non-claims for runtime readiness.

## 2026-06-30 — Review/fix evidence packet for oya-ci-required

- Updated the PR templates, PR review checklist, done-definition checklist, pre-merge checklist, and code-review standard so merge-ready PR evidence records `oya-ci-required` status, exact failing/fixed checks, review-thread resolution, and reviewer approval state on the current PR head.
- Clarified that local CLI or hook output is shift-left evidence only, never merge authority, and that generated faces must be producer-materialized rather than hand-edited.
- Added REVIEW-001 review-lens closure language: worker-completed implementation cards need a protected PR URL plus independent reviewer evidence, reviewer approvals must match the current head SHA, and SEC-001 threat-model coverage must be linked or marked `N/A` with rationale.

## 2026-06-30 — AUTHZ-004 dead Cedar ConfigMap deletion

- Deleted the unused `oya/analytics` Helm Cedar ConfigMap template, which still carried the legacy action-agnostic blanket Cedar permit but was not mounted by the chart Deployment.
- Removed that path from the `cloud-ci-cedar-deploy-parity` shrink-only baseline and added a regression proving AUTHZ-004-deleted dead ConfigMaps are neither baseline-grandfathered nor collected as deployed ConfigMaps.

## 2026-06-29 — Live Postgres sublanes for GH #901

- Split the required live-Postgres bridge into independent adapter and facade jobs, each with its own Postgres service/bootstrap, so safe groups can run in parallel without sharing database state.
- Added fan-in self-test coverage proving both split sublanes are required by `oya-ci-required` and the retired monolithic live-postgres dependency cannot silently remain.

## 2026-06-29 — Cache-hit report fail-closed guard for GH #900

- Made the required buck2 lane's cache-hit report upload binding: a missing report is now RED instead of an upload warning, while current bypass/cold posture remains allowed.
- Added conformance coverage proving the required workflow captures the invocation record, generates the stable cache-hit report artifact, runs the warm/bypass guard, and cannot hide missing report diagnostics behind `continue-on-error`.

## 2026-06-29 — Merge-hold preflight packet for GH #902

- Added the adapter-neutral merge-hold packet contract to the existing PR merge-gate kernel so team task state, native review state, and required check state must agree on the same PR head before merge readiness.
- Documented the failure/success packet contents and explicit premature-merge conflict-avoidance rule without adding a new workflow or reorg-sensitive service surface.

## 2026-06-29 — Cloud Cedar blanket disarm for GH #987

- Replaced the fourteen Cloud control-plane Helm Cedar ConfigMaps named in GH #987 with their authored action/resource-specific PBAC policies and removed those paths from the `cloud-ci-cedar-deploy-parity` shrink-only blanket baseline.
- Added a gate regression proving the Cloud templates are no longer baseline-grandfathered and that every deployed Cloud permit constrains action plus resource/scope before subset parity is evaluated.
- Tightened the mirrored Cloud IAC and tenancy policy fragments by removing executable default-deny forbids, broadening ApplyJob negative guardrails across all mutating actions, and splitting tenancy/auditor permits by principal, action, and resource type.

## 2026-06-29 — PR metadata admission packet wired into oya-ci

- Added a Rust PR metadata packet to `oya-ci-required` so blocked/pending-review PR title or body markers and missing `## Code Review` evidence fail before merge without claiming the F-PR5-06 live review-producer gap is closed.
- Marked quality-lane `check_command` rows as local/transitional bridge feedback only; protected-branch authority remains the single `oya-ci-required` fan-in plus cloud-ci/Rust gate packets.

## 2026-06-29 — OpenBao ESO scope and transport gate hardened

- Extended the existing operator-secret-bootstrap gate instead of adding a new cloud-ci surface: static and values-backed ExternalSecret use of OpenBao stores is now policy-scoped by store, bound role, namespace, and remote key prefix, and plaintext OpenBao listeners require restrictive NetworkPolicy coverage.
- Split the cloud-k8s CSI and cloud-iam SVID-operator ExternalSecrets onto dedicated OpenBao role/store contracts, documented the matching OpenBao policy/role bootstrap, and fenced the OpenBao listener with a committed NetworkPolicy.

## 2026-06-29 — Supply-chain admission proof wired into active gate

- Replaced the in-cluster registry static-key Cosign admission policy with keyless Sigstore/Rekor plus SLSA provenance and CycloneDX SBOM attestation checks; secondary Kyverno/Kubewarden policy fixtures, signed-image dev CLI defaults, and the dev CLI supply-chain verifier now use owned `jason931225/oyatie` subject/repo scope plus live `cloud/cloud-iac` paths.
- Retired `cargo-vet` from live readiness authority until maintained inputs exist, updated the governance-lane index to point SBOM/Cosign/SLSA at the active supply-chain gate, and added a `cloud-ci-supply-chain-audit` self-test proving the active `oya-ci-required` path covers signature/provenance/SBOM/dependency posture.

## 2026-06-10 — ADR-0544 friction-ledger closed-loop accounting gate authored

- Added ADR-0544 and the `cloud-ci-friction-accounting` meta-gate: every friction-ledger row must
  terminate in a gate, an automation, or an explicit accepted-risk entry, enforced so unconverted,
  undisposed, or unevidenced frictions block merges like code debt (Google SRE postmortem
  action-item model, Rust-native).
- The gate is born pack-shaped: ledger path, free-text status taxonomy, and evidence rules are DATA
  in `friction-accounting-policy.json` (the row field-name schema is the engine's contract); the Rust
  kernel is neutral and runs on any repo. It is a standalone born-blocking buck2 self-test with its
  own reviewed (review-visible, shrink-only) baseline + ceilings, documented in the oya-ci gate
  catalog. A merge-base shrink-only meta-check (FRIC-1781112000) and per-row owner/aging enforcement
  are named follow-ups.

## 2026-06-10 — G011 main-checkout guard Rust hook pattern added

- Added the Rust-owned main-checkout guard hook pattern for FRIC-022 / FRIC-1781062867.
- Updated ADR-0523 with the constrained agent-hook exec shim row: shell may only locate and exec
  the Rust hook binary, with fail-open behavior when the local binary is absent.

## 2026-06-10 — FRIC-012 enforcement-liveness gate added

- Added `cloud-ci-enforcement-liveness` to make tracked hook scripts mechanically live across
  Claude and Codex project hook wiring, while preserving marked compatibility stubs.
- Documented the `enforcement_liveness` producer face and frozen-empty hook liveness codes in the
  oya-ci gate catalog.

## 2026-06-10 — ADR-0540 target-parity gate authored

- Added ADR-0540 and documented the `cloud-ci-target-parity` gate for Cargo workspace member
  BUCK-file and `rust_test` target parity.
- Updated the oya-ci gate catalog with the `target_parity` producer face, frozen-empty
  `member_missing_buck` code, and baseline-block-on-new test-target debt code.

## 2026-05-20 — ADR-0320 transient program identity doctrine authored

- Added ADR-0320 for apprentice, intern, resident, fellow, co-op, and extern identities as multi-tenant transient program memberships with time-bound Cedar permits, labor overlays, portfolio survival, and shared-crate implementation footprint.
- Captured multispectrum evidence at `evidence/multispectrum/adr-0320-transient-identity-1779293714.json`.

## 2026-05-16 — PRs 12-18 multispectrum review backfill + drift-sweep fix-PR

Multispectrum-review v2.3.0 wave covering merged PRs #12, #13, #15, #17, #18.
20 facets per PR (F1-F9 + F10 + F13 + M1 + M2 + A1-A7) as separate
subagents; 105 evidence files at `evidence/debate/pr-N-FX-r1.json` +
`pr-N-synthesis.json`. Independent codex gpt-5 high-reasoning cross-check
captured at `.omc/artifacts/ask/codex-high-cross-check-claude-s-multi...md`.

Consolidated fix-PR addresses systemic findings:

- **BREAKING (ADR-renumber):** ADR-0119-onprem-k8s renumbered to ADR-0121
  to resolve the merge-race collision with ADR-0119-specs-flat-root
  (PR #18 merged 3m31s earlier). 7 inbound onprem refs swept.
- **Drift sweep:** ADR-0100 through ADR-0121 (22 ADRs) added to
  `doc.adr_*` catalog rows in `DOC-CATALOG.md`. `ADR-INDEX.md` +
  `decisions.json` regenerated via `oya doc adr-index --write`
  (99 records, next=ADR-0122).
- **Hygiene:** Untracked 8 `.omc/**/*.json.tmp.<uuid>` stowaways
  (PII surface — one leaked a local absolute path); added
  `**/*.json.tmp.*` and `.omc/**/*.tmp.*` to `.gitignore`.
- **ADR template normalization:** 8 ADR H1s (0110/0111/0112/0113/0114/0115/0117/0119)
  normalized from `# ADR-#### — title` to `# ADR-####: title` for
  generator compliance; ADR-0054 supersession blockquote moved AFTER
  the H1.

Follow-up FixupTasks filed in `registries/cross-cutting/fixuptasks.jsonl`
for findings outside this fix-PR's control-plane-only scope (audit-chain
cryptographic chaining, PR15 v1/v2 template migration, M02/M03 milestone
collision, Bominal-ADR-0119 disambiguation, ADR-0019 self-amendment).

## 2026-05-16 — archive-orphan lane retired after M01-P18 cutover

- Retired the one-time `archive-orphan` fitness lane after ADR-0116 established the Foundry pipeline (M01-P18) as the canonical VCS substrate.
- Removed the pre-grit archive payload, `oya-governance-archive-orphan-kernel`, `oya-governance-archive-orphan-app`, workspace members, and catalog entries.
- Naming justification: `archive-orphan` remains only as a historical lane id because IP-008 used that exact cutover-boundary name.

## 2026-05-15 — Fitness lane `oya-governance-sunset-lifecycle` scaffolded (ADR-0108 sunset → deprecation → removal automation)

- Added `crates/oya-governance-sunset-lifecycle-kernel` (I/O-free pure check + kernel-local std-only `Date` type — zero non-std deps, honoring ADR-0083 Tier 1) and `tools/oya-governance-sunset-lifecycle-app` (composition-root dev-CLI walking 3 discovery surfaces: ADR frontmatter, spec JSON `_sunset` objects, `[package.metadata.oya.sunset]` Cargo manifest sections). Operationalizes the user directive (2026-05-15) `sunset > deprecation > removal. dispatch.` and the `feedback_no_exceptions_canonical.md` doctrine — time-bounded sunset clauses are canonical *because of* the sunset clause, not despite it.
- Kernel exposes `Date`, `SunsetClause`, `LifecycleState` (5 variants: PRE_SUNSET / SUNSET_REACHED / DEPRECATED / REMOVAL_REACHED / MISSING_FIELDS), `Violation`, `evaluate(clauses, now, reached_milestones)`, `effective_deprecation_at`, `effective_removal_at`. Canonical sub-rule defaults: `deprecation_at = sunset_at + 30 days`, `removal_at = effective_deprecation_at + 90 days`. 11 kernel unit tests + 7 dev-CLI tests pass.
- Workspace members updated (`crates/oya-governance-sunset-lifecycle-kernel`, `tools/oya-governance-sunset-lifecycle-app`); `cargo check --workspace` green; lane surfaces 6 baseline violations on first run (3 ADRs: 0037/0067/0083; 3 specs: markdown-retirement-policy, multispectrum-review, oyatie-doctrine — all MISSING_FIELDS). Ratchet plan WARN → BLOCK in `.omc/plans/milestones/M01-foundation/phases/P02-doc-automation-freshness/fitness-sunset-lifecycle-lane.md`.
- ADR-0108 anchors the machine-readable schema (`sunset_at` OR `sunset_milestone`, plus optional `deprecation_at`, `removal_at`, `sunset_topic`); complements ADR-0037 (runtime-side per-tenant `DeprecationUsed` events) and ADR-0109 (generic lifecycle-automation framework). Scaffold-lock logged in `scaffold-locks-oyatie` per ADR-0054.

## 2026-05-15 — Fitness lane `oya-governance-adapter-with-no-importer` scaffolded (ADR-0104 audit-#7 mechanical-prevention)

- Added `crates/oya-governance-adapter-with-no-importer-kernel` (I/O-free check) and `tools/oya-governance-adapter-with-no-importer` (dev-CLI runner) per ADR-0104 Follow-up #4. The lane scans the workspace and flags any `*-adapter` crate that has no `*-importer-*` consumer — the audit-#7 anti-pattern that produced 18 placeholder-shell crates in commit `34c62f2`.
- Kernel exposes `WorkspaceCrate`, `Violation`, `AdapterImporterReport`, and `check`; port-in-kernel per ADR-0056 (filesystem walking lives in the dev-CLI). 8 kernel unit tests + 3 dev-CLI parser tests pass.
- Workspace members updated (`crates/oya-governance-adapter-with-no-importer-kernel`, `tools/oya-governance-adapter-with-no-importer`); `cargo check --workspace` green; lane surfaces 29 baseline violations on first run (ratchet plan WARN→BLOCK in plan file under `.omc/plans/milestones/M01-foundation/phases/P03-purpose-orphan-detection/fitness-adapter-with-no-importer-lane.md`).
- Implements ADR-0104 Consequences §4 mechanical-prevention candidate; scaffold lock logged in `scaffold-locks-oyatie` per ADR-0054.

## 2026-05-15 — M02-P06 Foundry Supervisor implementation complete

- Implemented the Foundry Supervisor daemon with hyperscaler-grade safety: atomic configuration writes (tempfile + fchmod 0600 + rename), symlink defense (O_NOFOLLOW), and automatic timestamped backups.
- Delivered the 4-crate core decomposition (supervisor-kernel, supervisor-app, jsonl-supervisor-adapter, supervisor-conformance) and the 2-crate settings-template expansion (settings-template-kernel, settings-template-adapter).
- Hardened the supervisor orchestration loop with ADR-0003 audit paths, data_class annotations, zero-unwrap error handling, and a 17-step tick_once cycle including saturation checks and silent-switch guards.
- Integrated real CLI drivers for Claude, Codex, and Gemini with template-driven settings drift detection and a minimum-eligible-account "blackhole" defense.
- Established the full documentation set (README, Architecture, Operations, Security, Sample Payloads) and a lifecycle management runbook (RB-SUPERVISOR-001).
- Updated ADR-INDEX and DOC-CATALOG with 7 new ADRs (0096-0102) and 5 new doc surfaces; 2400/2400 workspace tests pass.

## 2026-05-14 — P01 foundation full-check closeout

- Promoted the P01 closeout evidence from standalone-gate green to full `./scripts/check.sh` green under Rust 1.95.0 / edition 2024 / rustfmt 2024.
- Resolved the final shared gate blockers exposed during closeout: glossary cross-doc/vocabulary drift, quality-lane markdown mirror drift, repoctl pre-push manifest wiring, active-artifact/ADR-index drift, and architecture-boundary dependency direction.
- Recorded IP-009, IP-010, and IP-012 as complete in the P01 index while preserving the P00 acceptance/waiver gate before broad master-plan fan-out.

## 2026-05-14 — scripts/check helper-script blocker resolved

- Restored the four helper scripts invoked at the start of `scripts/check.sh`: Stage 0 Application-shell prereq self-test, M02 exit-checklist renderer, master-plan ledger renderer, and master-plan completion honesty audit.
- Verified all four helpers with Python compile checks and their check/self-test modes; `scripts/check.sh` now advances past the missing-script blocker and cargo fmt, then reaches the next real shared blocker: `cargo check --workspace --all-targets --all-features` stale connect-domain imports.

## 2026-05-14 — M01-P04-IP-002 object graph property tiers probe

- Added the ontology-domain `ObjectEntity::upsert_property` seam with explicit insert/update outcomes and no-mutation validation failure behavior.
- Added a true `ObjectGraph::upsert_entity` seam keyed by tenant id + entity id so the stable entity-upsert contract is backed by create/update semantics rather than only property replacement.
- Exposed the five Object Graph property tiers (`vector`, `timeseries`, `geo`, `ciphertext`, `struct`) as a stable domain set while retaining scalar compatibility for existing property paths.
- Promoted the machine-readable `object-graph.entity.upsert` mirror to stable and recorded scoped ontology tests, clippy, nextest, metadata, cargo-deny, and content assertions; `scripts/check.sh` remains the shared acceptance blocker.

## 2026-05-14 — M01-P04-IP-001 eventing review fixup

- Made topic registration invariant-safe by keeping `Topic` fields private and revalidating axis/name/description rules inside `TopicRegistry::register`.
- Added an append-only `v1-published` file-ledger event so published-state transitions for already-persisted outbox records can be durably replayed without rewriting the record prefix.
- Added regressions for invalid topic revalidation and persist-mark-published-reload behavior; scoped eventing tests, clippy, nextest, cargo-deny, metadata, and content assertions are green while `scripts/check.sh` remains blocked at cargo-check stale connect-domain imports.

## 2026-05-14 — M01-P04-IP-001 eventing outbox/topic registry probe

- Strengthened the eventing outbox kernel so replaying the same tenant/topic/idempotency key with a different payload reference fails instead of silently returning the earlier record.
- Added `data_class` annotations to eventing kernel struct fields and hardened the file outbox decoder against malformed UTF-8-boundary length prefixes.
- Corrected the eventing AsyncAPI Proto `$ref` and promoted the machine-readable `eventing.outbox.publish` contract mirror to stable.
- Scoped eventing tests, clippy, nextest, cargo-deny, metadata, and content assertions are green; repository-wide `scripts/check.sh` now reaches `cargo check --workspace --all-targets --all-features` and is blocked by stale connect-domain imports, so the IP is probe-green / acceptance-blocked rather than complete.

## 2026-05-14 — M01-P03 audit-chain evidence complete

- Promoted the cross-axis audit-chain integrity failure runbook from stub to active Sev-1 procedure with exact one-cycle tamper drill commands.
- Verified the domain tamper drill and file-ledger divergent/tampered-history drill against the live audit-chain verification surfaces.
- Marked M01-P03 complete after Merkle + Ed25519 kernel, stable AsyncAPI/Proto contract, and Sev-1 tamper-evidence drill all carried fresh evidence.

## 2026-05-14 — M01-P03-IP-002 audit event AsyncAPI + Proto contract

- Published the stable `audit.event.emit.v1` AsyncAPI/Protobuf source contract with an existing Proto `$ref` target.
- Promoted the Proto payload to `platform.audit.v1.AuditEvent` and included tenant shard, sequence, SHA-256 hash-chain fields, Merkle root, and Ed25519 signature proof material.
- Added no-dependency Node contract lint commands for the IP acceptance gate and aligned SPEC plus machine-readable contract stability to stable.

## 2026-05-14 — M01-P03-IP-001 audit-chain Merkle + Ed25519 kernel

- Added SHA-256 Merkle prefix roots and Ed25519 signature types/sign/verify support to `oya-audit-chain-domain` under the Rust 1.95.0 / edition 2024 / rustfmt 2024 stance.
- Enforced per-tenant-shard append semantics and added regressions for hash-chain tamper, Merkle-root tamper, signature tamper, and missing-signature verification.
- Updated the file ledger to persist v2 audit records carrying tenant shard, SHA-256 Merkle root, and optional Ed25519 signature fields, with malformed UTF-8 length prefixes rejected as parse errors.
- Recorded `ed25519-dalek` 2.x stable and `sha2` 0.10.x as the new direct dependencies for the real Ed25519 + SHA-256 kernel; ed25519-dalek 3.x remains prerelease.

## 2026-05-14 — M01-P02 foundation complete

- Completed the identity/Cedar phase: identity user upsert, STS issue/rotation, and Cedar policy publish all have current crate/runtime evidence.
- Added `oya-platform-policy-cedar-api` to the Rust 1.95.0 / edition 2024 workspace and aligned it to the current `oya-policy-cedar-domain` crate.
- Strengthened Cedar policy versions with strict semver, tenant/global scope, same-scope older-version supersession, chain lookup, idempotent publish, and active-only authorization.
- Recorded M01-P02 evidence for IP-001, IP-002, and IP-003; `scripts/check.sh` remains blocked by the pre-existing missing stage0 prereq script.

## 2026-05-14 — M01-P02-IP-002 STS rotation

- Added `rotate_identity_token_from_app` and `PurposeScope` so STS rotation preserves tenant, subject, credential kind, purpose, and scope while requiring the previous STS record to still be active.
- Kept `identity.token.issue` idempotent and ≤1h, and rejected `long_lived_api_key` at the application parser before typed credential issuance.
- Added rotation regressions for active re-issue, expired previous tokens, scope escalation, and subject drift under Rust 1.95.0 / edition 2024 / rustfmt 2024.

## 2026-05-14 — M01-P02-IP-001 identity kernel

- Promoted the identity user kernel to current flat-crate surfaces: `oya-identity-domain` now owns `User`, `UserId`, and required per-region `IdpBinding`.
- Added domain regressions for tenant/user/idp binding validation and kept STS service-principal issuance compatible while preserving ≤1h token gates.
- Brought `oya-platform-identity-api` into the Rust 1.95.0 / edition 2024 workspace and verified `identity.user.upsert` through its API regression set.
- Aligned SPEC and machine-readable contract mirrors to the current identity crates, with repo-root rustfmt `style_edition = "2024"` retained.

## 2026-05-14 — M01-P01 foundation complete

- Closed IP-003 by locking the DSR cascade preview SLA to 30 days with a regression that accepts exactly 30d and rejects `30d + 1s`.
- Added API-boundary coverage proving `dsr.cascade.execute` emits proof-of-erasure ids for each affected store and rejects completed store acknowledgements missing proof fields.
- Aligned SPEC and machine-readable contract rows to the current clean-architecture DSR crates: `oya-dsr-domain` and `oya-dsr-application`.
- Marked M01-P01 complete with evidence, masterplan parity, repo-root `rustfmt.toml` style-edition 2024, and final code-review `APPROVE` / `CLEAR`.

## 2026-05-14 — M01-P01-IP-002 tenancy kernel contracts

- Added `oya-tenancy-kernel` as the final-shape tenancy kernel with `TenantId`, immutable `RegionBinding`, `ResidencyClass`, `TenantContext`, and `TenantScopedRecord` row-level isolation guard.
- Registered the kernel in the workspace under Rust 1.95.0 / edition 2024, with rustfmt style edition 2024 inherited from repo config.
- Aligned the data-boundary public privacy label for KR financial data to canonical `FINANCIAL_KR` while retaining the legacy `FINANCIAL_KR_신용정보` parser alias.

## 2026-05-14 — M01-P01-IP-001 Data Use Boundary ADR accepted

- Promoted ADR-0008 Data Use Boundary from Proposed to Accepted and regenerated the ADR index/machine-readable mirror from all 67 `docs/decisions/ADR-*.md` files (31 Accepted / 36 Proposed, next ADR number 0091).
- Published the §2.2.2 consent-tier UI mapping in `docs/PRIVACY-PROGRAM.md`, preserving purpose-permission rows as the authoritative grant model.
- Recorded the M01-P01-IP-001 scaffold-claim fallback after grit returned the known new/doc-symbol FK failure.
- Added repo-root `rustfmt.toml` to pin both Rust parsing edition and rustfmt style edition to 2024 under the Rust 1.95.0 stance.

## 2026-05-14 — M01-P08 foundation cleared

- Closed the P01 foundation sequence: IP-009 delete-active-path cleanup, IP-010 parallel-claim demo, and IP-012 authoritative-tracked lane all received code-review APPROVE.
- Marked the P01 phase index `foundation-cleared` with explicit evidence and remaining pre-existing workspace blockers.
- Standalone P01 gates are green: banned-primitives, archive-orphan, authoritative-tracked, and parallel-claim demo regression.

## 2026-05-14 — M01-P08-IP-012 authoritative-tracked lane

- Added `oya-governance-authoritative-tracked-kernel` and `tools/oya-governance-authoritative-tracked` to validate the `docs/AGENTS.md` canonical authority links against tracked repository state.
- The runner parses the canonical doc map, accepts tracked directories through tracked children, and fails on missing, ignored, or untracked authoritative artifacts.
- Corrected `docs/AGENTS.md` masterplan authority pointer to current tracked `docs/MASTERPLAN.md` after the lane exposed an untracked future-target pointer.
- Updated the IP-012 good-taste row with the single typed-list behavior.

## 2026-05-14 — M01-P08-IP-010 parallel-claim demo runbook

- Added `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` and executable script to prove two session-less `grit` agents can claim non-overlapping symbols in one file.
- Recorded the 2026-05-14 transcript under `/evidence/agentic-pipeline/ip-010-parallel-claim-demo-transcript/`, including the duplicate-claim negative case and final lock cleanup.
- Updated the runbooks index with the agentic-pipeline demo entry.

## 2026-05-14 — M01-P08-IP-009 removed DELETE-class Bominal ultragoal ephemera

- Removed the two ADR-0052 DELETE-class active-path files from `bominal/agents/ultragoal/` after P7 gates passed: banned-primitives, archive-orphan, and non-null ARCHIVE timestamps.
- Updated IP-009 to target the actual DELETE-class rows and avoid direct VCS wording in the agent-facing plan.
- Stamped ADR-0052 DELETE-row notes with P7 cleanup time `2026-05-14T13:26:13Z`.

## 2026-05-14 — M01-P08-IP-008 archive-orphan lane and Bominal ultragoal archive

- Archived 15 Bominal ultragoal orchestration-glue files under `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` and stamped ADR-0052 `Archived at` rows for the ARCHIVE class.
- Added `oya-governance-archive-orphan-kernel` and `tools/oya-governance-archive-orphan` to verify archived copies exist, active originals are absent, and living references are zero outside authority/provenance docs.
- Refined inventory checklist samples so they no longer cite a real archived Bominal runtime path as an active example.

## 2026-05-12 — Lifted 5 reference docs (deep-dive ×2, hyperscaler, LTS-versions, cutover-amendments) to canonical docs/{specs,research,plans}/ tree

## 2026-05-12 — Lifted 11 branch-pipeline docs including ADR-0055 to docs/advanced-cicd/branch-pipeline/

## 2026-05-12 — Lifted 10 release-versioning docs to docs/advanced-cicd/release-versioning/

## 2026-05-12 — Lifted 9 progressive-delivery specs (shard A) to docs/advanced-cicd/progressive-delivery/

- 9 progressive-delivery specs lifted from `.omc/advanced-cicd/progressive-delivery/` to `docs/advanced-cicd/progressive-delivery/`. Status set to `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added; `adrs_cited: [ADR-0053, ADR-0052, ADR-0054]` added to all frontmatter. Body content preserved verbatim.
- Files landed: `INDEX.md`, `blue-green-spec.md`, `canary-rail-spec.md`, `dark-launch-spec.md`, `enforcement-lanes.md`, `feature-flag-architecture.md`, `playbook-ads.md`, `playbook-cloud.md`, `playbook-cross-axis-contract.md`.

## 2026-05-12 — Stage 1 Wave 2: 17 standards landed at docs/standards/

- 17 cross-cutting authoring standards lifted from `.omc/standards/` to `docs/standards/` (INDEX + 16 standard files). Status set to `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added to all files. ADR-0053 (sanctioned primitives), ADR-0052 (pre-grit artifact inventory), and ADR-0054 (scaffold-claim pattern) cited in every file's frontmatter `related_adrs:` field. Body content preserved verbatim.
- Files landed: `INDEX.md`, `doc-style.md`, `code-style-rust.md`, `error-handling.md`, `testing.md`, `security-review.md`, `on-call.md`, `claude-code-harness.md`, `multi-agent-tool-map.md`, `observability.md`, `release-management.md`, `git-workflow.md`, `dependency-policy.md`, `image-discipline.md`, `data-class.md`, `autonomy-ceiling.md`, `agent-instructions-discipline.md`.
- Resolves all `<!-- forward-reference: wave-2 -->` sentinels in `docs/AGENTS.md`, `docs/README.md`, and `docs/CONSTITUTION.md` pointing at `standards/*` rows.

## 2026-05-12 — Stage 1 Wave 2: 64 fitness-lane specs lifted to docs/governance-lanes/

- 64 fitness-lane catalogue specs lifted from `.omc/governance-lanes/` to `docs/governance-lanes/` (64 lane files + INDEX). Status set to `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added. ADR-0053 cited in lanes enforcing sanctioned-primitive rules (adapter-kernel, banned-primitives, bypass, cloud-mutation, cutover-bootstrap-window, direct-tool-invocation-audit, provider-agnostic); ADR-0052 cited in portfolio-citation (inventory); ADR-0054 cited in agent-completion-checklist, claim-ceiling, scaffold-claim-pattern. Kernel implementations deferred to Stage 3.

## 2026-05-12 — Stage 1 Wave 2: templates + checklists lifted to docs/templates/ + docs/checklists/ (25 files)

- **doc.templates-index** (Tier 2): 13 template files lifted from `/templates/` to `docs/templates/` (INDEX + 12 templates); 12 checklist files lifted from `/templates/checklists/` to `docs/checklists/`. Status set to `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added; ADR-0052 + ADR-0053 + ADR-0054 cited in every file's frontmatter and body prose where sanctioned primitives, inventory ledger, and scaffold-claim are referenced.
- 4 templates renamed to `-v2` due to conflicts with existing `docs/templates/` files: `pull-request-template-v2.md`, `adr-template-v2.md`, `runbook-template-v2.md`, `capability-record-template-v2.yaml`. Each carries `header_note: "Supersedes prior docs/templates/<name>.md once reviewed."` and `supersedes:` frontmatter field.
- 0 checklist conflicts (all 12 checklists are new additions; existing `docs/checklists/cross-axis-contract-change.md` preserved; new `cross-axis-contract-change-checklist.md` carries `extends:` pointer to the prior file).
- Existing `docs/templates/` files preserved as-is: `migration-runbook-template.md`, `dpia-template.md`, `team-charter-template.md`, `threat-model-template.md`, `incident-postmortem-template.md`, and others out of scope of this delivery.
  - Authors: jason931225
  - ADRs cited: ADR-0052, ADR-0053, ADR-0054
  - Related lanes: oya-governance-plan-hierarchy, oya-governance-pr-shape, oya-governance-capability-publish, oya-governance-inventory-tracker, guard-pr-merge-review.mjs
  - Commit: Stage-1-Wave-2-templates-checklists

## 2026-05-12 — Stage 1 Wave 2: automation pipeline + visualization + discipline specs landed (19 files)

- **doc.automation-index** (Tier 2): 19 automation specs lifted from `.omc/automation/` to `docs/automation/`; covers 8 auto-doc-generation pipelines (rustdoc, openapi, adr-index, runbook-freshness, fitness-lane-reports, schema-doc, changelog, glossary), 7 architecture-visualization specs (architecture-map-kernel, product-map, service-map, tech-stack-map, roadmap-visualization, dependency-graph, audit-chain-map), and 3 discipline specs (doc-freshness, orphan-detection, cross-reference-index). Status set to Accepted; `lift_target:` removed; `date: 2026-05-12` added; ADR-0052 + ADR-0053 + ADR-0054 cited in every file. Kernel crates (architecture-map, doc-freshness, orphan-detection) land in Stage 3.
  - Authors: jason931225
  - ADRs cited: ADR-0052, ADR-0053, ADR-0054
  - Related lanes: oya-governance-doc-freshness, oya-governance-orphan-detection, oya-governance-cross-reference-index
  - Commit: Stage-1-Wave-2

## 2026-05-12 — Stage 1 Wave 3: ai-slop-defense lifted to docs/quality/ai-slop-defense/ (7 files)

- Lifted all 7 files from `.omc/advanced-cicd/ai-slop-defense/` to `docs/quality/ai-slop-defense/`: INDEX, ai-slop-failure-mode-catalogue, production-quality-bar, gap-analysis-ai-vs-production, defense-in-depth-architecture, additional-tooling-recommendations, impossible-to-fail-environment-spec.
- Per-file transforms: `status: pending approval` → `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added; ADR-0053 + ADR-0055 cited in each file's frontmatter (`adr_citations:`) and body prose.

## 2026-05-12 — Stage 1 Wave 2: agent-kickoff layer lifted to docs/agents/ (11 files)

- Lifted all 11 files from `.omc/agent-kickoff/` to `docs/agents/`: INDEX, AGENT-ENTRY-POINT, AGENT-DECISION-TREE, AGENT-TOOL-PROTOCOL, AGENT-COMPLETION-PROTOCOL, AGENT-FAILURE-RECOVERY, AGENT-ICM-TOPIC-CONVENTIONS, CROSS-REFERENCE-INDEX, AGENT-CHEAT-SHEET, HUMAN-OPERATOR-GUIDE, ESCALATION-MATRIX.
- Per-file transforms: `status: pending approval` → `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added; internal references updated from `.omc/standards/` → `docs/standards/`, `/templates/` → `docs/templates/`, `.omc/governance-lanes/` → `docs/governance-lanes/`.
- Foundation ADRs ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim) cited in each file's frontmatter and body.

## 2026-05-12 — ADR-0052 Inventory ledger for grit/icm cutover landed

- ADR-0052 inventory ledger for grit/icm cutover landed; classifies 211 artifacts across oyatie/ and bominal/ by closed-set action; satisfies spec A2; ADR-INDEX updated.

## 2026-05-12 — MASTERPLAN lifted to canonical docs/MASTERPLAN.md (Stage 1 Wave 1)

- Promoted `.omc/plans/MASTERPLAN.md` to `docs/MASTERPLAN.md` as the Accepted canonical Master Plan anchor (authority tier 0).
- Status changed from `pending approval` to `Accepted`; `lift_target` field removed; `date: 2026-05-12` and `owners: ["council-architecture"]` added.
- §Authority-anchor section added: all milestone/phase/IP files under `docs/plans/milestones/M*/` derive authority from this document and ultimately from `docs/CONSTITUTION.md`.
- Foundation ADRs ADR-0052, ADR-0053, ADR-0054 cited in §Principles as the underpinning ADR triad.
- All internal milestone/phase/IP links updated to `docs/plans/milestones/...` canonical paths.
- `docs/README.md` updated: MASTERPLAN.md added to Tier-1 documents section and root document index table (`doc.masterplan`, tier 0).

## 2026-05-12 — ADR-0053: grit + icm + oya-tooling-agent-read as sole sanctioned primitives

- Authored `docs/decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md` (Accepted).
- Fixes the agent-callable coordination/state-transition primitive set at `{grit, icm, oya-tooling-agent-read}`; direct `git`/`gh` permitted only with documented rationale per Directive 12.
- Historical planned enforcement: `oya-governance-banned-primitives` lane was defined for P4/P5 merge-boundary work.
- Consensus reached iter-2 via Planner+Architect+Critic; operational driver: `.omc/plans/ralplan-oyatie-sst-consolidation.md`.
- Sibling ADRs landing in parallel: ADR-0052 (pre-grit artifact inventory), ADR-0054 (grit scaffold-claim pattern).

## 2026-05-12 — ADR-0054: grit scaffold-claim pattern (icm-coordination-lock fallback)

- Authored `docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` (Accepted).
- Formalises the icm-coordination-lock fallback (`scaffold-locks-oyatie` topic) as the canonical scaffold-claim path for new-crate creation at grit v0.3.0, following Lane 3 deep-dive trace confirmation that `Cargo.toml::workspace_members` is not indexed by grit (zero matches, 2026-05-12).
- Documents the verbatim 7-step sequence, two rejected alternatives (workspace_members grit claim; per-file-path lock), worked example with icm store rows, and two upstream follow-up issues.
- Updated `ADR-INDEX.md`: total ADRs 52, next number 0055, ADR-0052/0053 placeholders noted, ADR-0054 row appended.

## 2026-05-12 — Foundry RAG retrieve API contract

- Added the stable `foundry.rag.retrieve` REST boundary via `oya-intelligence-rag-api`, enforcing tenant/index namespace binding, Foundry authorization evidence, idempotent retrieval semantics, privacy-program data-class allowlists, and purpose-bound consent receipts before citation return.
- Registered `contracts/openapi/foundry/rag-v1.yaml` in the OpenAPI registries, catalog, SPEC, Foundry PRD, and machine-readable contract mirror.

## 2026-05-12 — Foundry capability publish API contract

- Added the stable `foundry.capability.publish` REST boundary via `oya-intelligence-registry-api`, enforcing path/body capability binding, Cedar authorization evidence, idempotent publish semantics, typed capability schema projection, provider/cost validation, and signed passing eval gates.
- Registered `contracts/openapi/foundry/registry-v1.yaml` in the OpenAPI registries, catalog, SPEC, Foundry PRD, and machine-readable contract mirror.

## 2026-05-12 — Foundry autonomy ceiling policy publish API contract

- Added the stable `foundry.policy.autonomy-ceiling.publish` REST boundary over `oya-intelligence-policy-kernel`, including idempotent publish semantics, Cedar policy refs, autonomy decision evidence, and OpenAPI runtime/schema parity.
- Registered `contracts/openapi/foundry/policy-v1.yaml` in the OpenAPI registries, catalog, SPEC, Foundry PRD, and machine-readable contract mirror.

## 2026-05-12 — Platform DSR cascade execute API contract

- Added `dsr.cascade.execute` OpenAPI/runtime/schema/catalog parity via `oya-platform-dsr-app`.
- Bound idempotent DSR cascade execution to tenant privacy-officer authorization, path/body DSR identity, cross-axis store scope, terminal acknowledgements, proof-of-erasure coverage, SLA status projection, and stable error envelopes.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Workspace Forms submission ingest API contract

- Added `workspace.forms.submission.ingest` OpenAPI/runtime/schema/catalog parity via `oya-workspace-forms-api`.
- Bound idempotent Forms submission ingest to tenant form schemas, submitter-principal validation, required-answer enforcement, Object Graph route projection, privacy-program data-class labels, and stable error envelopes.
- Mirrored the contract in SPEC, Workspace PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Workspace Chat message send API contract

- Added `workspace.chat.message.send` OpenAPI/runtime/schema/catalog parity via `oya-workspace-chat-api`.
- Bound idempotent Chat message sends to tenant channel membership, sender-principal validation, parent-thread existence, privacy-program data-class labels, and stable error envelopes.
- Mirrored the contract in SPEC, Workspace PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Workspace Meet session start API contract

- Added `workspace.meet.session.start` OpenAPI/runtime/schema/catalog parity via `oya-workspace-meet-api`.
- Bound idempotent Meet session starts to tenant cell/SFU placement, host participant validation, privacy-program data-class labels, and stable error envelopes.
- Mirrored the contract in SPEC, Workspace PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Workspace Drive object API contract

- Added `workspace.drive.put` and `workspace.drive.get` OpenAPI/runtime/schema/catalog parity via `oya-workspace-drive-api`.
- Bound idempotent Drive object metadata writes and ACL-checked reads to the Workspace Drive kernel, preserving KMS-shred object bindings and tenant-scoped data-class labels.
- Mirrored the contract in SPEC, Workspace PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Foundry eval run API contract

- Added `foundry.eval.run` OpenAPI/runtime/schema/catalog parity via `oya-intelligence-eval-app`.
- Bound authenticated, idempotent eval-run recording to signed eval sets, mandatory adversarial + linguistic cohorts, pass-threshold enforcement, and stable error envelopes.
- Mirrored the contract in SPEC, Foundry PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform regulatory pack bind API contract

- Added `regulatory-pack.bind` OpenAPI/runtime/schema/catalog parity via `oya-platform-regulatory-pack-api`.
- Bound authenticated, idempotent tenant pack binding to regional-pack validation, immutable tenant residency binding, multi-pack record projection, and authorization evidence.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform Object Graph entity upsert API contract

- Added `object-graph.entity.upsert` OpenAPI/runtime/schema/catalog parity via `oya-platform-object-graph-api`.
- Bound authenticated, idempotent entity upsert to tenant row-isolation, property tier labels, privacy-program data-class labels, and mutation-event evidence.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform Cedar policy publish API contract

- Added `cedar.policy.publish` OpenAPI/runtime/schema/catalog parity via `oya-platform-policy-cedar-api`.
- Bound authenticated, idempotent policy publication to path/body policy version, principal authorization evidence, semver supersession, and tenant/global Cedar scope.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform identity user upsert API contract

- Added `identity.user.upsert` OpenAPI/runtime/schema/catalog parity via `oya-platform-identity-api`.
- Bound authenticated, idempotent user upsert to path/body tenant and user identity, principal authorization evidence, per-tenant primary-identifier uniqueness, and regional IdP binding.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform tenant create API contract

- Added `tenant.create` OpenAPI/runtime/schema/catalog parity via `oya-platform-tenant-api`.
- Bound authenticated, idempotent tenant creation to path/body tenant identity, operator authorization evidence, global tenant-id uniqueness, and the tenant/residency kernels.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform identity token issue API contract

- Added `identity.token.issue` OpenAPI/runtime/schema/catalog parity via `oya-platform-identity-app`.
- Bound authenticated, purpose-parsed, idempotent STS token issue to the platform identity kernel while forbidding long-lived API keys.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform audit event emit contract

- Added `audit.event.emit` AsyncAPI/Protobuf/runtime/catalog parity via `oya-platform-audit-chain-app`.
- Bound CloudEvents envelope validation, producer authorization, privacy-program data-class parsing, hash-chain append, and eventing outbox publication in typed tests.

## 2026-05-12 — Platform metering event ingest contract

- Added `metering.event.ingest` AsyncAPI/Protobuf/runtime/catalog parity via `oya-platform-metering-app`.
- Bound CloudEvents envelope validation, producer authorization, plane/axis/unit/data-class parsing, metering kernel recording, and eventing outbox publication in typed tests.
- Mirrored the event contract in SPEC, SaaS Platform PRD, machine-readable contracts, and API semver metadata.

## 2026-05-12 — Platform eventing outbox publish contract

- Added `eventing.outbox.publish` AsyncAPI/Protobuf/runtime/catalog parity via `oya-platform-eventing-app`.
- Bound CloudEvents envelope validation, producer authorization, privacy-program data classes, regulatory packs, and idempotent outbox publication in typed tests.
- Mirrored the event contract in SPEC, SaaS Platform PRD, machine-readable contracts, and API semver metadata.

## 2026-05-12 — Cloud billing event ingest contract

- Added `cloud.billing.event.ingest` AsyncAPI/Protobuf/runtime/catalog parity via `oya-cloud-billing-app`.
- Bound CloudEvents envelope, producer authorization, idempotency fingerprinting, billing kernel ingest, platform metering, and eventing outbox publication in typed tests.
- Mirrored the event contract in SPEC, Cloud PRD, machine-readable contracts, and API semver metadata.

## 2026-05-12 — Cloud cell binding API contract

- Added `cloud.cell.bind` OpenAPI/runtime/schema/catalog parity via `oya-cloud-cell-app`.
- Bound authenticated, idempotent tenant cell assignment to tenant/principal/authorization evidence before `CloudRegionCatalog::bind_route_for_tenant`.
- Mirrored the contract in SPEC, Cloud PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Cloud FinOps report API contract

- Added `cloud.finops.report` OpenAPI/runtime/schema/catalog parity via `oya-cloud-finops-api`.
- Bound authenticated, idempotent report generation to tenant/principal/authorization evidence before `CloudFinopsLedger::generate_report`.
- Mirrored the contract in SPEC, Cloud PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Cloud observability audit read API contract

- Added `cloud.observability.audit.read` OpenAPI/runtime/schema/catalog parity via `oya-cloud-observability-api`.
- Bound the authenticated audit-read request to tenant/principal/authorization evidence before kernel projection and exposed cursor/chain metadata in the success envelope.
- Mirrored the contract in SPEC, Cloud PRD, machine-readable contracts, and OpenAPI registries.

# Oyatie — Canonical Docs Changelog

> Per-commit log for `docs/`. Auto-emitted (per [DOC-CATALOG.md](DOC-CATALOG.md) `doc.changelog`).

---

## 2026-05-12 — Cloud billing invoice API contract

### Updated
- **SPEC.md**, **products/cloud/PRD.md**, and **machine-readable/contracts.json** — bound `cloud.billing.invoice.generate` to `contracts/openapi/cloud/cloud-billing-invoice-v1.yaml` and `oya-cloud-billing-tax-app`.

## 2026-05-12 — Cloud network load balancer API contract

### Updated
- **SPEC.md**, **products/cloud/PRD.md**, and **machine-readable/contracts.json** — bound `cloud.network.lb.create` to `contracts/openapi/cloud/cloud-network-lb-v1.yaml` and `oya-cloud-network-lb-api`.

## 2026-05-12 — Cloud network DNS API contract

### Updated
- **SPEC.md**, **products/cloud/PRD.md**, and **machine-readable/contracts.json** — bound `cloud.network.dns.zone.create` to `contracts/openapi/cloud/cloud-network-dns-v1.yaml` and `oya-cloud-network-dns-api`.

## 2026-05-12 — Cloud network VPC API contract

### Updated
- **SPEC.md**, **products/cloud/PRD.md**, and **machine-readable/contracts.json** — bound `cloud.network.vpc.create` to `contracts/openapi/cloud/cloud-network-vpc-v1.yaml` and `oya-cloud-network-vpc-api`.

## 2026-05-12 — Foundry capability record schema gate

### Updated
- **templates/capability-record-template.yaml** — split capability descriptions into agent/human MCP fields and aligned the template with the fail-closed capability-record schema gate.

## 2026-05-11 — OpenAPI 3.2 operation parity hardening

### Updated
- **standards/api-design.md** — documented OpenAPI 3.2 `QUERY` and `additionalOperations` governance, including fixed-method collision rules and runtime parity requirements.
- **MISTAKES-LEDGER.md** — added `MFL-0013` for the OpenAPI 3.2 operation traversal/runtime-parity prevention.

## 2026-05-11 — Flat-crates documentation consistency

### Updated
- **ADR-0015**, **ADR-INDEX.md**, and **machine-readable/decisions.json** — promoted architectural flattening to Accepted and aligned CI lane names with the live flat-crates guard.
- **DESIGN.md**, **ROADMAP.md**, **STANDARDS-AND-TEMPLATES.md**, **TOOLCHAIN.md**, **AGENTS.md**, and **teams/axis-foundry/CHARTER.md** — separated live 64-crate flat baseline from historical 89/91 split planning and retired legacy-root wording.
- **products/foundry/PRD.md**, **ADR-0020**, and **ADR-0022** — replaced current `services/agent/daemon` / `tools/repoctl` references with flat `crates/oya-*` and `crates/oya-tooling-cli-dev-runtime` bindings.
- **PRIVACY-PROGRAM.md**, **ADR-0008**, **ADR-0019**, **ADR-0025**, **GLOSSARY.md**, **checklists/pre-push.md**, and **templates/capability-record-template.yaml** — aligned lane names and catalog paths with the live flat-crates governance model.
- **CONSTITUTION.md**, **README.md**, **DOC-CATALOG.md**, **DOCUMENTATION.md**, ADR references, product PRDs, templates, and machine-readable batches — normalized canonical doc-tree references from retired consolidated-tree paths to the live `docs/` tree.

## 2026-05-11 — Foundry capability invoke ingress hardening

### Updated
- **SPEC.md** — `foundry.capability.invoke` status vocabulary now includes explicit `422` idempotency-conflict errors alongside `202`/`400`/`403`.

## 2026-05-11 — Flat-crates governance hardening

### Updated
- **MISTAKES-LEDGER.md** — added `MFL-0012` for legacy implementation-tree regression prevention.
- **standards/ci-lanes.md** — clarified flat-crates and catalog-record lane behavior.
- **runbooks/flat-crates-move-pr.md**, **runbooks/per-context-flatten-phase.md**, **runbooks/workspace-members-merge-queue.md** — replaced stubs with active ADR-0015 procedures.

## 2026-05-09 — initial consolidation

This is the founding consolidation, authored in one session as the project repositions from "Oyatie" → "Oyatie" and from a 5-axis vertical-cloud thesis to a 7-axis ecosystem-as-a-service behemoth.

### Created
- **README.md** — directory orientation
- **PRD.md** — product north star, 7 axes, optimal-path waves, anti-scope, success metrics, decision log
- **DESIGN.md** — cohesion thesis, planes, Foundry-as-accelerator (incl. multi-provider auth + in-house AI substrate + DC-ops sub-axis + Robotics/Vision/Speech sub-substrates + cloud trajectory + automation-first pipeline), per-axis bounded contexts, tenancy model, audit chain, Data Use Boundary, flattening, horizontal-scale primitives, cross-axis contract surface §10, contradiction audit §11, regional-pack architecture §12
- **PRIVACY-PROGRAM.md** — Data Use Boundary ADR draft (12 data classes + orthogonal subject_class + purpose-permission matrix + four-pillar matrix + KR-specific obligations + agent-runtime privacy + DSR cascade)
- **DOC-CATALOG.md** — protocol + catalog + 19 update-trigger events + per-doc owner+cadence+dependent-docs+validation-check
- **GLOSSARY.md** — industry-aligned vocabulary + Oyatie-specific terms with industry analogs + KR↔EN parity + 13-section structure
- **ADR-INDEX.md** — 127-ADR index + status counts + per-axis distribution + supersession chains + drift findings
- **TOOLCHAIN.md** — best-for-task language stack matrix + agent-specific toolchain + parallelization-first tools + MCP gateway (Section 4.A) + license manifest + investment sequence
- **DOCUMENTATION.md** — Diátaxis-aligned doc system + storage map + generation pipelines + DaaP norms
- **STANDARDS-AND-TEMPLATES.md** — catalog of templates / checklists / hooks / skills / tools / standards / requirements
- **COMPLIANCE-MATRIX.md** — regulator × control × evidence × cadence × owner across KR / JP / US / EU / IN / BR / KSA / UAE / AU / SG + cross-regional standards
- **RISK-REGISTER.md** — 27 scored risks (severity × likelihood × velocity) + 6 anti-risks + per-axis slice
- **CONTRADICTION-LEDGER.md** — LEDG-001..029 from Codex verdict + recon files + team-charter review
- **security-program/security-program.json** — threat model + 12 controls + per-axis controls + continuous control monitoring
- **SLO-CATALOG.md** — per-surface SLOs + error-budget policy + burn-rate gates
- **RELEASE-MANAGEMENT.md** — trunk-based + release branch + CI lane catalog + progressive delivery + hotfix path + per-axis release exceptions
- **QA-TEST-STRATEGY.md** — test pyramid + required tests per change class + fixture discipline + flaky-test policy + coverage targets
- **INCIDENT-MANAGEMENT.md** — Sev taxonomy + roles + lifecycle + comms templates + drills + prevention loop
- **RACI-OWNERSHIP.md** — cross-axis ownership matrix + per-surface CODEOWNERS map + decision rights
- **ROADMAP.md** — wave list (Foundation → Foundry-Preview → Cloud-Preview / SaaS-Preview / Workspace-Preview / Search-Preview parallel → Vertical-Pilot → Vertical-Fan-Out → Cloud-Stable → Search-Stable → Ads-Preview → Ads-Stable → AI-Model-Substrate → DC-Operations → Region-Fan-Out)
- **ADR-CONSOLIDATION-PLAN.md** — strategy for consolidating 127 legacy ADRs into ~30-40 new ADRs
- **products/_TEMPLATE.md** + **products/README.md** — per-product PRD template
- **products/{saas-platform,foundry,cloud,search,ads-analytics,workspace}/PRD.md** — 6 axis PRDs (Foundry deepest at 852 lines)
- **products/vertical-{corporate,healthcare,industrial,logistics,fintech,legal}/PRD.md** — 6 deep vertical PRDs
- **products/vertical-{retail,education,public-sector,hospitality,construction,real-estate,agriculture,food}/PRD.md** — 8 skeleton vertical PRDs
- **teams/README.md + 37 team charters**
- **standards/fintech-compliance.md** — PCI-DSS v4.0 scope + per-jurisdiction overlays for Toss/KakaoBank/PayPal/Robinhood-class
- **templates/{adr-template.md, capability-record-template.yaml, runbook-template.md}**
- **checklists/{pre-push.md, wave-gate.md, foundry-capability-publishing.md}**
- **machine-readable/catalog.json** — initial machine-readable doc catalog

### Drafted ADRs
- `decisions/ADR-0013-product-license-policy.md` — defines product license policy; AGPL/GPL forbidden in product code (Apache-2/MIT/BSD/MPL-2 allowed)

### Direction changes integrated
1. Brand standardized as Oyatie (`oya-*` Cargo prefix)
2. 7 axes (SaaS / Workspace NEW / Vertical / Foundry / Cloud / Search / Ads)
3. Foundry consolidates Foundry engineering platform (originally separate axis)
4. Multi-provider Foundry: Anthropic Claude / OpenAI / Google Gemini × subscription + API
5. Canonical + regional-pack architecture (parallel global launch)
6. Multi-year structural cost-of-deferral horizon
7. In-house build preference + license-conscious posture
8. Architectural flattening (`crates/oya-<context>-<role>`)
9. M0/M1/M2/M3/MVP vocab retired → wave-named phases
10. Repoctl persona-split (`oya dev/admin/build/agent/ops/pack/catalog/gate`)
11. MCP gateway for agent-discoverable CLI
12. Workspace / Productivity Platform as Axis 2 (NEW)
13. In-house AI model training + inference (W-AI-Model-Substrate, long-horizon)
14. DCIM software for own DC ops (W-DataCenter-Operations)
15. Robotics / Vision / Speech intelligence sub-substrates
16. Compute trajectory: OCI + AWS now → Oyatie colo at scale → own greenfield mega-DC (DC-from-scratch back in scope)
17. Automation-first principle (Google + Amazon doctrine; highest yield in git/CI/CD/PR pipeline)
18. ADR consolidation directive (existing 127-ADR corpus → ~30-40 new ADRs)

### Consensus pass
- Codex critic verdict at `docs/raw/codex-verdict.md` — REQUEST CHANGES with 8 BLOCKERs + ~20 HIGH items; 6 of 8 BLOCKERs addressed in this consolidation; 2 partial (re-sequencing in PRD §3.1 needs further pass; Build-vs-Buy ADR drafted in TOOLCHAIN, formal ADR pending)

### v2 backlog
- `docs/raw/plan-v2-draft.md` — 1,847 leaves across P0-P20 in 110 batch tags; full schema per-leaf; covers all 7 axes + cross-cutting + contradiction-resolution + brand-rename + long-tail


---

> **§Note (2026-05-21 transition):** References to `oya-governance-*` in this historical document are intentional — they describe past state. New work uses `oya-governance-*` per the 2026-05-21 transition directive.
