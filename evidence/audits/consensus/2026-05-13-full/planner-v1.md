# Full-project Ralplan Planner v1 (2026-05-13)

**Scope:** consensus on EVERYTHING — current state, accumulated directives, standards, queued slices, foundation prereqs, honest gaps. Architect: hyperscaler-grade lens. Critic: Torvalds lens.

## §1 Current state inventory (HEAD post-commit `pending`)

**Today's 7 commits (newest first):**

| Commit | Slice | Net |
|---|---|---|
| `pending` | Direction consensus 2026-05-13 + stale-pointer fix | 5 files; +286/-4 |
| `0806f91` | Palantir 3-layer ontology split + README revert | 4 files; +352/-18 |
| `6938c89` | Phase 1 markdown retirement + Constitution scheduled-for-retirement | 7 files; +1043/-8 |
| `1f96255` | ADR-0069 closeout stale-text purge | 1 file |
| `b0798b0` | Close architect-r17 #3/#7/#8 + r18 NEW defects | 12 files; +2278/-727 |
| `3d6de67` | ADR-0069 + active-artifact-contract v3.0.0 + minimal Rust validator | 24 files; +3451/-67 |
| `5880ce0` | ralplan-ops-portal v7 + Waves 2-5 Accepted | 5 files; +1655/-67 |

**Net:** ~14 machine-readable artifacts + 1 ADR + 1 Rust validator crate + 25 codex consensus outputs archived. ~9000 lines added.

**HEAD-tracked load-bearing files:**
- 7 canonical specs at `/specs/` (active-artifact-contract v3.0.0 + 5 inherited + plan-schema + markdown-retirement-policy + artifact-profile-defaults + root-hub-pointers)
- 5 registries at `/registry/` (artifact-capabilities + reusable-building-blocks + knowledge-graph-{semantic/kinetic/dynamic})
- 2 ledgers/attestations at `.omc/ledger/` + `/evidence/`
- 1 ADR-0069 at `docs/decisions/`
- 1 Rust validator crate (12 tests pass) at `crates/check-active-artifact-contract/`
- 1 CI lane registered (status=planned) at `registry/quality/lanes.yaml`
- 6 wave plans Accepted at `.omc/plans/ralplan-ops-*` (parent v7 + waves 2-5 + Wave 6 JSON v1.0.0; Wave 7 .md deleted)

## §2 Accumulated user directives (chronological, today)

1. "ops starts now" — start implementing per masterplan
2. "Anything that can be automated should be automated"
3. Autonomous-prompt cascade: grit-only, ZERO hand-authored Markdown, machine-readable canonical, final-report-schema, sequencing-JSON ledger
4. "Resume planning and come to consensus with this in mind"
5. "make our json and our machine readable documentation have purpose and features. Enforcement, Verification, Validation, Auto-generations, Self-healing, Self-updating, Self-maintaining" (9 capabilities)
6. "json/toml/yaml/whatever. what can be automated must be automated"
7. "verify and check if those meta-schemas are appropriate"
8. "fine tune and polish hyperscaler grade. Highly scalable, maintainable. critic/verify/review with torvald lens"
9. "Use whatever that is most appropriate for the task"
10. "Make it so that we are able to keep track of reusable building blocks so that we avoid duplicating. implement all the optimization measures that is considered best practice"
11. "ideally every relationship, every capability, every feature, every schema mapped graphed and tracked for automation is ideal"
12. "DRY enforcement is key"
13. "Heavy diet. Make sure everything has a purpose. trace what files need to change/lock/post-action. useful for DRY/Automation/Workflow integration"
14. "our own repo, operation, knowledge graph is in essence can be expressed through workflow and ontology as well"
15. "Review all waves. Make sure our plan meets our requirements and quality. Plan everything out first"
16. "why even use markdown? we don't need markdown"
17. "the only thing that we should maintain is a README.md / CLAUDE.md / AGENTS.md"
18. "that should utilize machine readable pointers to relevant files"
19. "make sure to get rid of markdowns we don't need and consolidate where appropriate"
20. "separate ontology into semantic, kinetic, dynamic layers (borrowing from palantir)"
21. "i dont think constitution is necessary"
22. "README.md should remain hand written and human readable"
23. "/ralplan with codex come to a consensus on our current direction. and changes."
24. "Standardize everything. nothing adhoc. everything planned"
25. "Reach a consensus on everything. /ralplan architect like hyperscalers. critic like torvald."

## §3 Standards now in force (kinetic actions + workflows)

**14 kinetic action types** (per `/registry/knowledge-graph-kinetic.json`):
CreateArtifact, PromoteCapabilityStatus, DeprecateBlock, EmitEvidenceBundle, LandConsensusVerdict, WireLane, ArchiveArtifact, BumpPlanVersion, AddRegistryRow, UpdateCapabilityOverride, RetireMarkdown, ClaimGrit, DoneGrit, RecordICMStore. Each declares audit topic + idempotency + lock pattern + invariants to recheck.

**4 kinetic workflows:**
consensus_loop, markdown_retirement_per_file, wave_acceptance, capability_promotion.

**9 evidence classes** (per evidence-taxonomy.json):
implementation, verification, documentation, operational, security, supply_chain, critic, closeout.

**10 hyperscaler gates** (per hyperscaler-gates.json):
HG-ARCH, HG-CONTRACT, HG-SECURITY, HG-RELIABILITY, HG-OBS, HG-TEST, HG-SUPPLY, HG-OPS, HG-DOCS, HG-GRIT.

**9 stop conditions** (per stop-conditions.json): SC-01..SC-09.

**Per-artifact 9-capability contract** (per ADR-0069):
enforcement / verification / validation / autogen / selfheal / selfupdate / selfmaintain / telemetry / provenance.

**Markdown-retirement 8 phases** + retention rules (only README + CLAUDE + AGENTS survive; /evidence/audits/consensus archives kept).

**5-tier verification taxonomy** (per ADR-0069 §11 lifecycle):
verified-by-existing-CI / verified-by-existing-file / verified-by-Wave-N-prerequisite / committed-future-CI / unverified-blocked-by-foundation.

**Grit-protocol with ICM fallback** (per ADR-0054 + pragmatic-relaxation): claim → work → done. SQLite-FK error → scaffold-locks-oyatie ICM topic.

## §4 Queued slices (post-direction-consensus narrowed)

Direction consensus (consensus-v1.md pending approval) freezes net-new meta-layer; allows enforcement-loop + stale-pointer repair + consumer rewiring + migration slices that reduce drift + add failing fixture/active lane.

| # | Slice | Allowed under narrowing? | Reason |
|---|---|---|---|
| **VL** | **Vertical enforcement loop** (dev-cli + active lane + failing fixture + evidence + graph edge) | ✅ YES | First load-bearing slice; gates all others |
| 1 | Constitution content redistribution (mission→masterplan, principles→agent-contract, prohibitions→forbidden-operations, decision rights→RACI) | ✅ YES if paired with active lane that fails on redistributed-content drift | Migration slice with drift-reduction |
| 2 | Wave 1-5 plan conversion to JSON | ⚠️ ONLY when consumer exists (validator demands JSON; or generator emits Markdown projection) | Sans consumer = more paper |
| 3 | Wave 7 v1.0.0 .json (Wave 6 pattern) | 🛑 BLOCKED | No consumer demand; Wave 6 .json sufficient pattern |
| 4 | ADR migration (88 ADRs → consolidated registries) | ⚠️ Paired with check-crate refactor proving JSON-consumption | Massive scope; can break ~12 check crates |
| 5 | CLAUDE.md thinning to pointer hub | ✅ YES | Consumer rewiring; small |
| 6 | Workflow-task-traceability schema | 🛑 BLOCKED until VL operational | Net-new meta-layer class |
| 7 | Workflow+Ontology dogfooding ADR | 🛑 BLOCKED until VL operational | Semantic anchor; net-new |
| 8 | Foundation handoff (cosign/trivy/audit-chain/OpenBao/KMS) | Parallel-session; not under our control | External |
| 9 | Ops Wave 1 implementation (M02-P19 + M03-P04..P06 docs/workspace BCs) | ✅ YES after VL operational | Real implementation work |

## §5 Vertical enforcement loop blueprint (next slice)

Per direction consensus §"Acceptance criteria for the next slice":

| Hop | Artifact | Current state | Target |
|---|---|---|---|
| 1 | Tracked registry row | ✅ 10 rows in artifact-capabilities-registry.json | Add 1 row for plan-schema.json + verify with validator |
| 2 | Schema validation | ✅ JSON Schema 2020-12 in /specs/active-machine-readable-artifact-contract.json | Validator parses + checks instance |
| 3 | Validator runtime | ✅ crates/check-active-artifact-contract::validate (12 tests pass) | No change |
| 4 | `oya` command | ❌ NOT WIRED | NEW: `dev-cli gate validate active-artifact-contract` subcommand |
| 5 | Pre-claim/pre-done validation | ❌ NOT WIRED | NEW: scripts/hooks/pre-commit/validate-artifact-contract.sh OR grit pre-claim hook |
| 6 | CI lane active | ❌ status=planned | Flip to status=active in registry/quality/lanes.yaml |
| 7 | Evidence bundle | ❌ NOT EMITTED | Emit attestation recording green CI run URL |
| 8 | Graph edge update | ❌ NOT WIRED | Kinetic action CreateArtifact triggers; semantic graph reflects row |

Failing fixture: artifact under applicable_paths_glob without registry row → validator exit≠0 → CI red.

## §6 Honest gaps (pending consensus close)

1. **0/10 HG gates operational** (all planned or blocked-by-foundation).
2. **0 capabilities operational** across 10 registered artifacts (all planned).
3. **Foundation prereqs** (cosign/trivy/audit-chain/OpenBao/KMS/docker/podman/kubectl) block ~40% of operational claims.
4. **rtk-git used 7 times today** instead of grit due to SQLite FK error; logged in HG-GRIT row + ICM `direct-tool-invocations` topic.
5. **Markdown retirement at 0.8% completion** (2 of 250+ files migrated; 7-phase backlog).
6. **No drift-detector running** — DRY enforcement is policy not mechanism.
7. **No autogen capability operational** across any artifact — all status=planned.
8. **No telemetry exporters** — OpenTelemetry metrics declared in registry but no OTel runtime emits them.
9. **Wave plans (5 of 7) still Markdown** — Wave 6 only converted to JSON.
10. **Direction consensus itself is pending approval** — user has not yet executed (a)/(b)/(c) of the direction-consensus options.

## §7 Foundation prereq map

| Foundation work | Status (parallel session) | Blocks | Approx scope |
|---|---|---|---|
| docker/podman | not yet | Container build pipelines | Wave 1+ deploy lanes |
| kubectl | not yet | K8s cell ops | Wave 5 deployments BC |
| cosign | not yet | Evidence-bundle signing | HG-SUPPLY operational claim |
| trivy | not yet | Image scanning | HG-SUPPLY operational |
| audit-chain runtime | not yet (M01/M02) | EvidenceBundle emission; audit_chain_ref event_id | HG-OBS + every Wave's audit-emission |
| OpenBao | not yet | Secret rotation (Wave 7) | secret-rotation BC |
| KMS/HSM | not yet | Key management (Wave 7) | kms-management BC |

## §8 Standardization audit (per "Standardize everything" directive)

| Workflow | Standard declared? | Enforcement? |
|---|---|---|
| Consensus loop (architect+critic via codex) | ✅ kinetic workflow `consensus_loop` | ❌ enforcement is manual today |
| Markdown retirement (per file) | ✅ kinetic workflow + retirement policy + ledger | ❌ no validator; manual |
| Wave acceptance | ✅ kinetic workflow `wave_acceptance` | ❌ no automatic ledger update |
| Capability promotion | ✅ kinetic workflow `capability_promotion` | ❌ no validator runs lifecycle rule |
| Grit claim/done | ✅ master-plan-sequencing forbidden_primitives | ⚠️ ICM scaffold-lock fallback honest but ungated |
| ICM store | ✅ AGENTS.md mandatory triggers | ❌ no enforcement; agent-discretion today |
| Commit format | ❌ no declared standard | ❌ no commit-msg hook |
| ADR authoring | ⚠️ template exists at docs/templates/ but markdown-bound | ❌ no schema-validator for ADR shape |

Per directive: every workflow above must have BOTH standard declared AND mechanical enforcement OR be honestly flagged "policy, not mechanism".

## §9 Conclusion (proposed)

Adopt the full consensus position:

1. **Direction**: α with critic-r1 narrowing (continue meta-layer, freeze net-new classes until VL operational).
2. **Standards**: 14 kinetic actions + 4 kinetic workflows are the law; new state transitions extend these OR are honestly flagged "ad-hoc, standardization pending".
3. **Next slice (mandatory)**: Vertical enforcement loop (VL) end-to-end.
4. **After VL**: queued slices class-by-class with consumer-backed migration + failing fixture + active lane.
5. **Foundation handoff**: track parallel-session foundation completion; promote dependent capabilities as they land.
6. **Honest baseline**: 0/10 HG operational; 0 capabilities operational; ~9000 lines of meta-layer paper; 12 unit tests + 0 integration tests + 0 lane runs.
7. **Standardization gaps**: 4 workflows declared + ≥4 ungated; 2 workflows undeclared (commit format + ADR schema-shape).

---

**Awaiting architect r1 (hyperscaler lens) + critic r1 (Torvalds lens).**
