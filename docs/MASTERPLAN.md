---
doc_class: MasterPlan
shape: anchor
length_cap: 500
authority_tier: 0
status: Accepted
date: 2026-05-12
owners: ["council-architecture"]
canonical_authority: docs/CONSTITUTION.md
companion_docs:
  - docs/PRD.md
  - docs/DESIGN.md
  - docs/SPEC.md
  - docs/ROADMAP.md
  - docs/RACI-OWNERSHIP.md
  - docs/RISK-REGISTER.md
  - docs/CHANGELOG.md
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class files > working drafts
foundation_adrs:
  - ADR-0052
  - ADR-0053
  - ADR-0054
---

# Oyatie — MASTERPLAN

## §Authority-anchor

This is the canonical Master Plan for oyatie. All milestone INDEXes / phase INDEXes / Implementation Plans under `docs/plans/milestones/M*/` derive their authority chain from this document and ultimately from `docs/CONSTITUTION.md`.

---

> **Status:** Accepted (canonical at `docs/MASTERPLAN.md`).
> **Owner:** council-architecture (cross-axis); Founder Jason Lee (north-star arbiter).
> **Date:** 2026-05-12.

## 1. Executive summary

Oyatie is one cohesive **ecosystem-as-a-service** expressed across **seven axes** — SaaS, Workspace, Vertical, Foundry, Cloud, Search, Ads + Analytics — sharing one tenancy model, one identity surface, one capability registry, one audit chain, and one agent runtime (per [`docs/CONSTITUTION.md`](CONSTITUTION.md) §Mission and [`docs/PRD.md`](PRD.md) §1). The integration thesis: non-leakage of identity / audit / tenancy / runtime across every layer is more valuable than best-of-breed substitutes, because it removes the integration tax every multi-vendor stack pays ([`docs/PRD.md`](PRD.md) §1, §5).

This Master Plan is the single anchor for product launch at AWS/Google/Microsoft/Oracle quality. Six milestones (plus one cross-cutting cluster) decompose into ~33 phases and ~84 implementation plans, all built to **final shape on day one** — no MVP-shaped artifacts that need replacement, no placeholders that need migration.

**Current state (2026-05-12).** Cloud foundations kernels are shipping in flight at ~114 catalog/workspace records. The grit/icm agentic-pipeline cutover is mid-consensus iter-2. Foundry Phase 00 contract surface is salvaged from upstream bominal ultragoal awaiting cross-cite.

**Definition of "done" at the masterplan level:** see §13.

## 2. Compound principles (the directive stack)

All ten principles compound; none overrides. Every artifact in the milestone tree inherits them.

Foundation ADRs underpinning this section: **ADR-0052** (4-tier hierarchy), **ADR-0053** (final-shape discipline), **ADR-0054** (provider-agnostic interface contract).

| # | Principle | Rationale | Verification |
|---|---|---|---|
| 1 | **4-tier hierarchy.** Master Plan > Milestone > Phase > Implementation Plan. Every file in `docs/plans/milestones/` declares its tier in frontmatter. | A fresh agent should be able to descend the tree in O(log n) clicks; flat lists do not scale to 84+ IPs. | `oya-foundry-fitness-plan-hierarchy` lane validates frontmatter `doc_class ∈ {MasterPlan, MilestoneIndex, PhaseIndex, ImplementationPlan}` and parent-pointer present. |
| 2 | **Autonomous senior-engineering decisions for long-term outcomes.** Take on upfront cost if the long-term outcome benefits. No corner-cutting for short-term wins. | AWS / Google / MS / Oracle decisions optimize for 5-10 year maintainability; we adopt the same posture. | Per-IP `decision-log:` frontmatter field; ADR required when an IP defers a known-better-but-costlier path. |
| 3 | **Final-shape adoption from day one.** Build to the final form. No MVP that needs replacement; no placeholders; no temporary names; no stub implementations marked "to be rewritten." | Migration cost across 84 IPs and 21 axis/vertical product directories dwarfs upfront cost. | Per-IP `final_shape_compliance:` field; `oya-foundry-fitness-no-placeholder` lane refuses code containing `TODO`/`FIXME`/`unimplemented!()`/`todo!()` outside `flaky/` or explicit ADR-tracked carve-outs. |
| 4 | **Provider-agnostic by default.** Cloud, KMS, storage, network, observability, secrets, identity, AI providers — all use provider-neutral interfaces. Provider-specific code lives in `oya-<context>-adapter-<provider>-*` crates only. | Already proven in Foundry's Claude/OpenAI/Gemini adapter pattern (per [`docs/DESIGN.md`](DESIGN.md) §3.0). Extends to AWS/OCI/GCP/Azure cloud, OpenBao/Vault/AWS-KMS for secrets, Postgres/RDS/Cloud-SQL for db. | `oya-foundry-fitness-provider-coupling` lane refuses provider-specific imports outside adapter crates. |
| 5 | **Distroless + smallest-image containers.** Production binaries are `cargo build --release` + static (or `musl` static-linked where possible) and ship in `gcr.io/distroless/static-debian12` (or `distroless/cc-debian12` for FFI). No shells, no package managers, no debug tooling. | Smaller attack surface, faster pull, smaller registry storage. Hyperscaler convention. | `oya-foundry-fitness-image-discipline` lane: image-size budget per binary, distroless-base verified, no `apt`/`apk` layers. |
| 6 | **AWS / Google / MS / Oracle launch-quality bar.** Working Backwards / PRFAQ for product launches; Design Doc per phase; SRE postmortem-blameless on every Sev-1/2; Microsoft 1ES-templated CI; Oracle Engineering Excellence Council–style merge gate. | Customers buying Oracle/AWS/Google-tier products expect Oracle/AWS/Google-tier engineering rigor. | Per-milestone INDEX declares which named practices apply; `oya-foundry-fitness-hyperscaler-practice` lane checks adoption. |
| 7 | **Linus-style discipline.** Delete bureaucracy; reshape data to eliminate special cases; flat over deep when the deep is ceremony; good taste = simplest representation handling all cases without branching. | The "good-taste audit" PR section is mandatory per SP-01 A10 and inherited here per-IP. | Per-IP PR template "good-taste audit" block must enumerate special cases eliminated; empty block = fail. |
| 8 | **Current LTS dependencies, CI-enforced.** Every direct dependency tracks the current LTS major.minor where the project publishes LTS lines. Placeholder targets (pending verification): Rust stable (≥ 1.82), Node 20 "Iron" LTS, Python 3.12, PostgreSQL 16, Kubernetes 1.31, Debian 12 base, OpenTelemetry SDK current LTS. | LTS-tracking reduces CVE exposure, eliminates surprise EOL migration storms. | `oya-foundry-fitness-lts-dependency` lane (CI gate, blocking); per-IP `dependency-additions:` listed with LTS-conformance flag. |
| 9 | **Hyperscaler-bar internal toolchain + architectural robustness.** Practices adopted: AWS Working-Backwards / PRFAQ, Google Design-Doc + Postmortem-blameless, Microsoft 1ES-templated-pipelines, Oracle Engineering-Excellence-Council. Rust: `cargo-deny`, `cargo-audit`, `cargo-nextest`, `cargo-semver-checks`, `sccache`, `cargo-llvm-cov`. CI/CD: Sigstore/SLSA, OpenTelemetry, Distroless. | These are the field-proven practices at the four reference vendors. | Per-phase INDEX cites which Rust-practice gates inherit; per-IP CI checklist verifies. |
| 10 | **Auto-doc + purpose-driven + agentic-dev-optimized.** Generated docs > hand-written wherever a machine-readable source exists (rustdoc, OpenAPI, ADR-INDEX, fitness-lane reports). Every artifact has a declared `purpose` in frontmatter. Every directory has an INDEX.md or .json. IP files name their grit-claim symbols as real `file::Identifier`. | A fresh agent must navigate the tree without orchestrator hand-holding (read MASTERPLAN → pick milestone → pick phase → pick IP → grit-claim → work → icm-store → grit-done). | `oya-foundry-fitness-doc-freshness`, `-orphan-detection`, `-agentic-navigability` lanes. |
| 11 | **Visualization-as-code, Foundry-owned, auto-updated.** Architecture, product map, service map, tech-stack, roadmap, and dependency graphs are auto-generated from canonical sources (Cargo workspace metadata, `contracts/`, `docs/products/`, `docs/ROADMAP.md`, `docs/ADR-INDEX.md`, milestone/phase/IP frontmatter). The Foundry visualization kernel (`oya-foundry-architecture-map-kernel`) walks these sources and emits Mermaid (inline mdbook) + D2 (`terrastruct/d2` for richer service maps) + Graphviz (DAG fallback). Outputs are SVG + PNG + versioned markdown source. | Hand-drawn architecture diagrams age out of sync the moment they ship; the only sustainable form at AWS/Google/MS/Oracle scale is generated-from-truth. | `oya-foundry-fitness-architecture-map-freshness` lane blocks PRs that drift from generated state. Renders publish via `oya-foundry-mdbook-kernel`. |
| 12 | **Pragmatic git/gh — permitted with documented genuine need.** Default sanctioned primitives remain `{grit, icm, oya-tooling-agent-read}`. Direct `git`/`gh` invocation is permitted (by any operator — agent or human) when no grit/icm primitive exists AND inventing one would be over-engineering. Rationale logged via `icm store -t direct-tool-invocations -c "<one-line rationale>" -i high -k "git,<context>"` BEFORE execution. Workflows that repeatedly invoke `git`/`gh` for a common purpose are migration candidates into `oya-tooling-agent-read` to amortize the audit cost. | A strict ban produces theater; a documented exception produces an audit trail with migration signal. | `oya-foundry-fitness-banned-primitives` lane (revised): catches *undocumented* `git`/`gh` invocations in agent-instruction sections, not all invocations. Repeat invocations (≥ 5 of the same shape in 30 days) auto-emit a migration-candidate row in `MISTAKES-LEDGER`. |

## 3. Milestone index (the 4-tier root)

| ID | Title | Wave alignment (per [`docs/ROADMAP.md`](ROADMAP.md)) | Status | Phases | Owner axis | Index |
|---|---|---|---|---|---|---|
| **M01** | Foundation | W-Foundation | open | 6 | council-architecture + platform-tenancy-identity | [`docs/plans/milestones/M01-foundation/INDEX.md`](plans/milestones/M01-foundation/INDEX.md) |
| **M02** | Foundry-Preview | W-Foundry-Preview | gated on M01 | 6 | axis-foundry | [`docs/plans/milestones/M02-foundry-preview/INDEX.md`](plans/milestones/M02-foundry-preview/INDEX.md) |
| **M03** | Cloud + SaaS + Search + Workspace Preview (parallel) | W-Cloud-Preview ∥ W-SaaS-Preview ∥ W-Search-Preview ∥ W-Workspace-Preview | gated on M02 | 8 | axis-cloud + axis-saas + axis-search + axis-workspace | [`docs/plans/milestones/M03-cloud-saas-search-workspace-preview/INDEX.md`](plans/milestones/M03-cloud-saas-search-workspace-preview/INDEX.md) |
| **M04** | Vertical-Pilot (Korea-first) | W-Vertical-Pilot | gated on M03 | 4 | vertical-corporate (or council-elected) + tactical-first-vertical-pilot | [`docs/plans/milestones/M04-vertical-pilot-korea/INDEX.md`](plans/milestones/M04-vertical-pilot-korea/INDEX.md) |
| **M05** | Cloud-Stable + Search-Stable | W-Cloud-Stable + W-Search-Stable | gated on M04 | 4 | axis-cloud + axis-search + ops-compliance | [`docs/plans/milestones/M05-cloud-search-stable/INDEX.md`](plans/milestones/M05-cloud-search-stable/INDEX.md) |
| **M06** | Ads-Preview + Vertical-Fan-Out | W-Ads-Preview + W-Vertical-Fan-Out | gated on M05 | 4 | axis-ads-analytics + all vertical teams | [`docs/plans/milestones/M06-ads-vertical-fanout/INDEX.md`](plans/milestones/M06-ads-vertical-fanout/INDEX.md) |
| **M-CC** | Cross-cutting workstreams (thread across all milestones) | n/a (parallel) | open | 8 | per-thread owner; council-architecture coordinates | [`docs/plans/milestones/M-CC-cross-cutting/INDEX.md`](plans/milestones/M-CC-cross-cutting/INDEX.md) |

Long-horizon waves (W-DataCenter-Operations, W-Robotics-Vision-Speech, W-AI-Model-Substrate, W-AI-Model-Stable, W-Ads-Stable, W-Region-Fan-Out) become M07..M12 once the first commercial wave (M01..M04) is committed. They are named here for completeness but their milestone folders are NOT pre-instantiated to avoid orphaning before scope crystallizes.

## 4. Dependency graph

```
        M01 (Foundation) ──┐
                           ▼
                    M02 (Foundry-Preview) ──┐
                                            ▼
                              M03 (Cloud + SaaS + Search + Workspace Preview, 4-way parallel) ──┐
                                                                                                ▼
                                                                                M04 (Vertical-Pilot KR) ──┐
                                                                                                          ▼
                                                                                          M05 (Cloud-Stable + Search-Stable) ──┐
                                                                                                                               ▼
                                                                                                            M06 (Ads-Preview + Vertical-Fan-Out)

  M-CC (cross-cutting) threads through every milestone above; its phases land in parallel with whichever
  milestone is in flight.
```

Critical-path: **M01 → M02 → M03 → M04 → M05 → M06.** Within each milestone, phases parallelize per the milestone INDEX `§Parallelism strategy`.

## 5. Parallelism strategy (batches across milestones)

At any time, at least one M-CC phase and one main-spine milestone phase run concurrently. Within M03, four axis previews run as 4-way parallel. Within M04, vertical pilot serializes by design but its phases (capability-pack authoring, regulatory binding, design-partner onboarding, evidence collection) parallelize. M05 splits Cloud-Stable and Search-Stable across two teams in parallel.

Target: ≥ 3-5 agents in parallel per active milestone; ≥ 2 active milestones at any time (one main-spine + one M-CC phase batch). Per-agent worktree, merge-queue serialization on root `Cargo.toml [workspace.members]` per [`docs/DESIGN.md`](DESIGN.md) §3.0.5.2.

## 6. Per-tier artifact contract (the agentic-navigation contract)

A fresh agent navigates: MASTERPLAN → milestone INDEX → phase INDEX → IP → grit-claim → work → icm-store → grit-done. The contract enforces this end-to-end.

**Master Plan** (this file): §1 exec summary, §2 principles, §3 milestone index, §4 dependency graph, §5 parallelism, §6 contract, §7 dual-audience, §8 cross-cutting workstreams, §9 risks, §10 RACI, §11 cadence, §12 communication, §13 done-definition, §14 out-of-scope, §15 status footer.

**Milestone INDEX** (`docs/plans/milestones/<MNN>/INDEX.md`, ≤100 lines): frontmatter `doc_class: MilestoneIndex`, `parent: MASTERPLAN.md`, `status:`, `wave:`, `owner:`, `purpose:`. Body: §Purpose, §Status, §Scope, §Dependencies, §Acceptance gate, §Phases (linked list), §Hyperscaler practices adopted, §Agent-navigability-pointer (the first symbol/file an agent should claim).

**Phase INDEX** (`docs/plans/milestones/<MNN>/phases/<PNN-slug>/INDEX.md`, ≤50 lines): frontmatter `doc_class: PhaseIndex`, `parent: <MNN>/INDEX.md`. Body: §Purpose (1 line), §Acceptance, §Implementation Plans (linked list), §Estimated parallelism, §Symbols-touched (high level), §Agent-handoff (icm-store payload to emit at phase complete).

**Implementation Plan** (`docs/plans/milestones/<MNN>/phases/<PNN-slug>/IP-NNN-<slug>.md`, ≤80 lines stub / full when lifted): frontmatter `doc_class: ImplementationPlan`, `parent: <PNN>/INDEX.md`, `final_shape_compliance:`, `dependency-additions:`. Body: §Purpose, §Symbols-to-grit-claim (real `file::Identifier`), §Agent-prerequisites, §Acceptance-test-commands, §Done-criteria, §Rollback-procedure, §Next-IP-pointer, §Icm-store-payload, §Decision-log (Linus good-taste row).

Every IP that ships a deployed binary additionally includes:
- "Distroless image built; size < {budget}; no provider-specific deps outside adapter crates."
- "Dependency additions are current LTS or have ADR-tracked exception."

## 7. Dual-audience contract (agent + junior developer)

Each IP has two sections. The **agent-actionable** section is fenced `<!-- agent-instructions:start --> ... <!-- agent-instructions:end -->` and contains only sanctioned primitives (`grit`, `icm`, `oya-tooling-agent-read`). The **junior-developer** section sits outside the fence and contains plain-English summary, doc/runbook/ADR pointers, `rtk`-prefixed terminal commands (per [`docs/CONSTITUTION.md`](CONSTITUTION.md) §4 dual-audience clause), expected output samples.

## 8. Cross-cutting workstreams (the M-CC milestone)

Threads across every milestone. Each thread is a phase under [`docs/plans/milestones/M-CC-cross-cutting/`](plans/milestones/M-CC-cross-cutting/INDEX.md):

| Phase | Title | Owner |
|---|---|---|
| M-CC-P01 | Agentic-pipeline cutover (grit/icm SoT) | axis-foundry + council-architecture |
| M-CC-P02 | Documentation auto-generation + freshness | crew-adr-promotion + axis-foundry |
| M-CC-P03 | Purpose-discipline + orphan-detection lane | council-architecture |
| M-CC-P04 | Agentic-development optimization (navigability lanes) | axis-foundry |
| M-CC-P05 | Provider-agnosticism + adapter discipline | council-architecture + per-axis leads |
| M-CC-P06 | Distroless + image-discipline + LTS-dependency hygiene | ops-security + ops-sre-reliability |
| M-CC-P07 | Hyperscaler-practice adoption (Working Backwards / Design Doc / Postmortem / 1ES / Eng-Excellence) | council-architecture |
| M-CC-P08 | Supply-chain security (Cosign + Rekor + SLSA + SBOM) per ADR-0039 | ops-security |
| M-CC-P09 | Visualization-as-code (Foundry-owned architecture/product/service/tech-stack maps) | axis-foundry |

## 9. Risk register (top 10 across all milestones)

| ID | Description | Prob | Impact | Mitigation owner | Linked milestones | Status |
|---|---|---|---|---|---|---|
| RM-01 | Cross-axis contract drift | High | High | council-architecture | M03, M-CC-P02 | open |
| RM-02 | Tenant data leak into Search/Ads via PHI/PII | Med | Catastrophic | council-privacy | M01, M03, M05, M06 | open |
| RM-03 | Agent runtime escapes autonomy ceiling | Med | Catastrophic | axis-foundry | M02 | open |
| RM-04 | Flattening migration breaks `main` | Med | High | council-architecture | M01, M02 | open |
| RM-05 | grit 0.3.0 session bug widens | Low | High | axis-foundry | M-CC-P01 | open |
| RM-06 | Cloud axis built before tenancy/Move-#0 evidence | Med | High | axis-cloud + platform-tenancy-identity | M01, M03 | open |
| RM-07 | Banned-primitives lane bypass (agent uses raw git/gh) | Med | High | axis-foundry | M-CC-P01 | open |
| RM-08 | Provider-adapter secret leak | Low | Catastrophic | ops-security + axis-foundry | M02 | open |
| RM-09 | Korea regulatory shift mid-build | Med | Med | regional-packs + ops-compliance | M04, M05 | open |
| RM-10 | Provider lock-in regression (provider-specific code outside adapter) | Med | High | council-architecture | M-CC-P05 | open |

Full register at [`docs/RISK-REGISTER.md`](RISK-REGISTER.md).

## 10. RACI summary

| Milestone | Responsible | Accountable | Consulted | Informed |
|---|---|---|---|---|
| M01 | platform-tenancy-identity + platform-audit-evidence + platform-eventing-og | council-architecture | council-privacy, ops-security | All teams |
| M02 | axis-foundry | council-architecture | council-privacy, ops-security | All teams |
| M03 | axis-cloud + axis-saas + axis-search + axis-workspace | council-architecture | platform-tenancy-identity, regional-packs | All teams |
| M04 | vertical-corporate (or council-elected) | council-architecture + gtm-customer-success | All preceding M-owners | All teams |
| M05 | axis-cloud + axis-search + ops-compliance | council-architecture | regional-packs (KR) | All teams |
| M06 | axis-ads-analytics + per-vertical leads | council-architecture | council-privacy | All teams |
| M-CC | per-phase owner (see §8) | council-architecture | All teams | All teams |

Full RACI at [`docs/RACI-OWNERSHIP.md`](RACI-OWNERSHIP.md).

## 11. Status reporting cadence

- **Weekly** Monday 09:00 KST: per-milestone one-row update `MNN | active phases | % | blockers | next gate`. Archived to `docs/status-reports/YYYY-Www.md`.
- **Fortnightly** stakeholder review: Founder + Council-Architecture + axis leads walk the milestone tree, dependency graph, risk register.
- **Wave-gate** review: at each W- boundary, Council-Architecture signs per [`docs/ROADMAP.md`](ROADMAP.md) §2 acceptance lists.
- **Escalation**: Sev-1 → incident commander → Council-Architecture → Founder. Blocked milestone ≥1 week → owner lead → Council-Architecture.

## 12. Communication plan

| Channel | Audience | Frequency | Owner |
|---|---|---|---|
| `#oyatie-masterplan-status` | All contributors + agents | continuous + weekly summary | council-architecture |
| Milestone/phase/IP PR threads | Per-IP contributors | per PR | IP owning team |
| Council-Architecture sync | Council members | weekly | council-architecture chair |
| Stakeholder review | Founder + axis leads | fortnightly | council-architecture chair |
| `MISTAKES-LEDGER` row | All | per Sev-1/Sev-2 | Incident commander |
| `CHANGELOG.md` row | All | per merged PR (auto via Foundry `pr.changelog.row`) | axis-foundry |
| Regulator audit packs | KR-MFDS/PIPC/FSC/KISA/KCC/NIS | quarterly + on-request | ops-compliance |

## 13. Definition of "done" at the masterplan level

Oyatie has shipped the first commercial wave (M04 complete) when **all** of:

1. M01 (Foundation) merged: all foundation ADRs Accepted; fitness lanes hard-fail on violations.
2. M02 (Foundry-Preview) merged: capability registry ≥ 50 capabilities; 3 providers × 2 auth modes operational; autonomy ceiling Cedar+runtime; audit-chain on every regulated invocation; license-policy gate hard-fails.
3. M03 (Cloud/SaaS/Search/Workspace Preview) merged: W-Cloud-Preview gate per [`docs/ROADMAP.md`](ROADMAP.md) §2.3; all 14 Workspace surfaces stable; ≥ 2 regional packs onboarded.
4. M04 (Vertical-Pilot KR) merged: ≥ 1 design-partner tenant end-to-end with full audit-chain emission; pilot retention ≥ 80% over 8 weeks.
5. M-CC-P01 through M-CC-P08 all merged: agentic-pipeline, doc-automation, purpose-discipline, agentic-navigability, provider-agnosticism, distroless+LTS, hyperscaler practices, supply-chain — all lanes green on `main`.
6. [`docs/PRD.md`](PRD.md) §4.1 first-commercial-wave metrics met: ≥ 3 KR Group tenants live; ≥ 50K Foundry agent runs/week at ≥ 99.5%; 100% audit-chain on regulated invocations.
7. `oya-foundry-fitness-authority-cohesion` lane green: `docs/CONSTITUTION.md` cite-coverage 100% on Tier-1 docs.

M05 + M06 then unlock the next-wave commercialization (Cloud-Stable, Search-Stable, Ads-Preview, Vertical-Fan-Out).

## 14. Out-of-scope (this masterplan)

- Frontier-model R&D / AGI lab (per [`docs/PRD.md`](PRD.md) §1 non-goals; W-AI-Model-Substrate is a future milestone).
- Custom silicon / chip design.
- Consumer social network.
- Multi-region day-one (W-Region-Fan-Out is future).
- Public ad serving (W-Ads-Stable is future).
- Defense / weaponized robotics.
- GitHub repo slug rename (per ADR-0017).
- `~/.claude/CLAUDE.md` user-machine config edits.

## 15. Status footer

Status: **Accepted** (canonical at `docs/MASTERPLAN.md`).
Iteration: 3 — restructured to 4-tier hierarchy + 12 compound principles per coordinator directives 1-12 (2026-05-12). Adds Directive 11 (visualization-as-code Foundry-owned) and Directive 12 (pragmatic git/gh — documented genuine need permitted).
Lifted: Stage 1 Wave 1 — 2026-05-12 (promoted from `.omc/plans/MASTERPLAN.md` to canonical `docs/MASTERPLAN.md`).

Sources scanned: [`docs/CONSTITUTION.md`](CONSTITUTION.md), [`docs/PRD.md`](PRD.md), [`docs/DESIGN.md`](DESIGN.md), [`docs/SPEC.md`](SPEC.md), [`docs/ROADMAP.md`](ROADMAP.md), [`docs/RACI-OWNERSHIP.md`](RACI-OWNERSHIP.md), `docs/products/` (21 axis subdirs).
