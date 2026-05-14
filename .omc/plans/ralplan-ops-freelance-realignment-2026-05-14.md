---
doc_class: RalplanRealignment
shape: anchor
status: pending architect+critic
date: 2026-05-14
created_by: ralplan --realignment --short (post-investigation-synthesis, session 7e0309c2)
canonical_authority: docs/CONSTITUTION.md
authority_chain: docs/MASTERPLAN.md → .omc/plans/consensus-masterplan-2026-05-13.md → .omc/plans/ralplan-ops-portal-2026-05-13.md (v7 Accepted) + .omc/plans/ralplan-docs-portal-2026-05-13.md (v7 Accepted) + .omc/plans/ralplan-dep-seam-phaseout-round-5.md (3-of-3 APPROVE post-§18.C spot-fix) → this plan
companion_docs:
  - .omc/state/sessions/7e0309c2-f5ac-4d3f-846f-85c2292dd8b6/investigation-synthesis.md (gap analysis; architect APPROVED)
  - .omc/state/cross-session-coordination-2026-05-14.md (297-missing-crate-dir blocker hand-off)
mode: SHORT
codex_model: (deferred to architect+critic consensus loop; not invoked at draft time)
---

# RALPLAN — Ops freelance realignment + smallest Wave-1 prereq (2026-05-14)

## §0 Reframe

Twenty session commits (origin/main `4d6bf91` … back through `7947f44`/`9a95023`/`5a79fcb`/`a8634fc`) authored a hyper-based ops transport pattern that does NOT match ops portal masterplan Wave 1 scope. The eliminated failure mode: *"freelance code that doesn't compose with the Accepted ops-portal v7 + docs-portal v7 + dep-seam-phaseout R5 plans accumulates as silent debt blocking the actual Wave 1 dispatch."*

This plan converts the 20-commit freelance into either (a) Wave 2 candidate substrate (reclassified), (b) prototype/learning artifacts (rebuilt against Wave 1 BCs at dispatch time), or (c) phase-out trajectory (already governed by R5). It lands the smallest Wave-1-prereq edit that doesn't depend on the cross-session 297-missing-crate workspace-resolution blocker.

## §1 Principles (5; deliberate-mode not asserted; this is a SHORT-mode close-out)

1. **No silent regression.** Per `feedback_no_silent_regression.md` + lean-a10. The 20 commits stay in git history (not reverted); their architectural status is explicitly reclassified via this plan + ADR-0090 supersession.
2. **No dead code.** Per `feedback_autonomous_implementation_artifacts.md` "stale removed in reality" — reclassified crates that no Wave will adopt are deletion targets (not `// removed` comments + not retained skeletons).
3. **Inherited Bominal ADR-0209 (Leptos SSR) wins.** The hyper + OpenAPI 3.2 transport pattern in the freelance commits contradicts Wave 1's docs-portal v7 §6(a) decision (`-rest` houses Leptos `pages/` module). Rebuilt Wave 1 ships Leptos SSR.
4. **Smallest-actionable.** Per the user's smallest-actionable rule: this plan ships the catalog-registration delta only; the full 24-docs-crate Wave 1 substrate stays deferred until workspace-resolution clears AND M02-P20 prereqs land per docs sub-plan §6(g).
5. **Cross-plan composition.** ADR-0090 ↔ R5 are simultaneously Accepted today and mutually inconsistent; supersession resolves the contradiction in R5's favor (R5 is the more recent + multi-reviewer-consensus artifact).

## §2 Decision drivers (top 3)

1. **Cross-plan coherence over freelance preservation.** Workspace-shell + docs-portal + hyper-stack freelance does not match masterplan Wave 1 scope; preserving it as-is would leave three Accepted plans (ops-portal v7, docs-portal v7, dep-seam-phaseout R5) and the 20 session commits all simultaneously "right" while contradicting each other.
2. **Smallest-actionable now > big-bang later.** A 10-line Cargo.toml edit ships the M02-P19 IP-X1 catalog-registration prerequisite TODAY; the full Wave 1 substrate dispatch (24 docs crates + 16 extractors + watch daemon + 13 Leptos pages + 4 CI lanes + 4 Cedar fragments) stays gated behind workspace-resolution + M02-P20 + M02-P21 + M02-P22 per the docs sub-plan §6(g) dispatch sequence.
3. **Honest classification > speculative retention.** Path (a) reclassify-as-Wave-2-candidate is chosen for `oya-ops-workspace-shell-*` only when there is a credible Wave 2 BC the crate maps onto (per ops-portal v7 §6(a) Wave 2 inventory: overview / dashboards / tech-stack / architecture); otherwise path (b) prototype-only-retire-on-Wave-1-substrate-land.

## §3 Viable options (≥2)

**Option Ω — Revert all 20 session commits and rebuild from scratch.**
- Pros: clean slate; Wave 1 substrate authored against docs-portal v7 §6(a) verbatim.
- Cons: discards `oya-foundry-architecture-map-{kernel,app}` + `oya-check-{active-artifact-contract,cedar-fragment-coverage,openapi-rest-route-parity}` (3 of 20 commits) which ARE genuinely useful cross-cutting work; discards the hyper-stack which R5 already governs as known-debt. Revert is more cost than reclassification.
- **Rejected.**

**Option α — Reclassify + supersede + land smallest prereq edit (RECOMMENDED).**
- Reclassify the 5 `oya-ops-workspace-shell-*` crates as **Wave 2 candidate substrate** under the overview BC at M03-P06 IP-Y1 IF a credible mapping exists at scaffold time; otherwise retire per §6(b) below.
- Reclassify the 4 `oya-ops-docs-portal-*` crates + 2 `contracts/ops-*.openapi.yaml` files as **prototype/learning artifacts**; the actual Wave 1 docs portal rebuilds against the 6 docs sub-plan BCs (`portal` / `generator` / `manifest` / `search` / `cross-ref` / `live-diff`) per docs sub-plan §6(a) at dispatch time.
- Retain the 7 `oya-http-*` foundation crates (router-kernel / middleware-kernel / sse-domain → sse-kernel after R5 Step 1 rename / runtime-hyper-adapter / 3 middleware-domain → middleware-runtime after R5 Step 1 rename) as the substrate R5 §3 explicitly governs.
- Retain `oya-foundry-architecture-map-{kernel,app}` + the 3 `oya-check-*` crates as cross-cutting work; their phase home is M02-P21 architecture-planes-green (visualization-as-code) or M02-P20 IP-005 expansion (lane scaffolds), to be confirmed at dispatch time.
- **Mark ADR-0090 Superseded by ADR-0091.** Body stays factual (it accurately records the hyper foundation that exists today); status flips so the contradiction with R5 (which targets hyper phase-out post-ontology-v1) is resolved.
- **Land the M02-P19 IP-X1 catalog-registration edit:** refine `Cargo.toml [workspace.metadata.oya.microservices.ops] bounded_contexts` from `["docs", "workspace"]` to `["docs"]` only — `workspace` is not a Wave 1 BC per ops-portal v7 §6(a); if it survives reclassification it lands as a Wave 2 IP-Y1 BC at dispatch time.
- Pros: zero revert; preserves all genuinely useful freelance work; resolves ADR-0090↔R5 contradiction; ships the smallest masterplan-defined prereq today; defers Wave 1 substrate authoring until prereqs clear (per docs §6(g) dispatch sequence).
- Cons: 9 of 20 commits' crates may eventually retire if no Wave adopts them; that retirement is deferred to dispatch-time judgment per §6(b).
- **Chosen.**

**Option β — Reclassify but DON'T supersede ADR-0090.**
- Pros: zero ADR-amendment work.
- Cons: leaves ADR-0090 ↔ R5 simultaneously Accepted + mutually inconsistent. Future agents will read both and not know which governs. Violates `feedback_no_silent_regression.md` "Public contracts protected from silent change" — ADR-0090 is a public contract; R5 ships a phase-out trajectory; failing to mark the supersession leaves a silent contract drift.
- **Rejected.**

## §4 Pre-mortem (3 scenarios)

### Scenario 1: Reclassified workspace-shell crates linger as dead skeletons through Wave 2 dispatch
- **Trigger:** Wave 2 (M03-P06 IP-Y1..Y4) ralplan author doesn't adopt the workspace-shell crates as overview-BC substrate; the 5 crates sit in `crates/` with no consumer.
- **Blast radius:** lean-a8-dead-code-zero-tolerance (BLOCKER day 1 from M02-P21 per docs sub-plan §6(f)) flags the orphans; Wave 2 dispatch is blocked behind cleanup.
- **Prevention:** §6(b) below codifies the retirement trigger — at Wave 2 ralplan author-time, if no credible mapping exists onto overview / dashboards / tech-stack / architecture BCs, the workspace-shell crates retire via `git rm -r` + Cargo.toml workspace-member removal. The reclassification is provisional; retirement is the default if no adoption signal lands.
- **Detection:** `cargo run -p oya-check-dead-code -- --workspace --blocker` (lean-a8 binary; scaffold at M02-P21 IP-005 expansion per docs §6(f)) flags the orphans the moment lean-a8 goes live.
- **Rollback:** retire the 5 crates at Wave 2 dispatch-time.

### Scenario 2: ADR-0090 supersession doesn't actually unblock R5 follow-ups
- **Trigger:** R5 W0 Step 0..8 still requires Architect + Critic + codex consensus per `.omc/plans/ralplan-dep-seam-phaseout-round-5.md` §10 — supersession of ADR-0090 is a §8 deliverable of R5 ADR-0091 itself, not a free-standing edit.
- **Blast radius:** marking ADR-0090 Superseded prematurely (before ADR-0091 is Accepted) creates a different drift — Status: Superseded points at an ADR that doesn't exist yet.
- **Prevention:** §6(c) below ships the supersession edit IFF ADR-0091 itself exists at Accepted status. At the time of this plan's authoring, R5 is "3-of-3 APPROVE post-§18.C spot-fix" — ADR-0091 status per R5 §8 is "Proposed → Accepted on Architect + Critic + codex consensus (round-5 review pending)." If ADR-0091 is not yet Accepted in `docs/decisions/`, the supersession edit on ADR-0090 is deferred to the same atomic PR that lands ADR-0091 Accepted (per R5 §10 Step 4 ADR-authoring sequence).
- **Detection:** `test -f docs/decisions/ADR-0091-*.md && grep -q "^Status: Accepted" docs/decisions/ADR-0091-*.md` before the ADR-0090 frontmatter edit.
- **Rollback:** revert the ADR-0090 frontmatter edit if ADR-0091 lands as Rejected/Withdrawn (low probability per R5 3-of-3 verification status).

### Scenario 3: Catalog-registration BC list edit silently breaks an unknown consumer
- **Trigger:** `Cargo.toml [workspace.metadata.oya.microservices.ops] bounded_contexts` is consumed by some validator (e.g., `oya-check-documentation`, `oya-check-active-artifact-contract`) that hard-codes the current `["docs", "workspace"]` list; dropping `workspace` fails the validator.
- **Blast radius:** CI flips red on the catalog-registration commit; ralplan-realignment session blocked.
- **Prevention:** before the edit, grep for `workspace` literal usage in validator code paths (`crates/oya-check-*/src/` + `crates/oya-shared-documentation-check-cli/src/`) to confirm no hard-code; the validator should consume the BC list dynamically.
- **Detection:** `cargo check --workspace` after the edit (gated behind cross-session blocker — see §6(d) note); also `cargo run -p oya-check-documentation -- --workspace --report-only` if/when workspace builds.
- **Rollback:** restore `["docs", "workspace"]` and document the consumer in a follow-up.

## §5 Expanded test plan (SHORT-mode; minimal)

| Tier | Coverage |
|---|---|
| **Static check (markdown-only edits)** | `.omc/plans/ralplan-ops-freelance-realignment-2026-05-14.md` lints via `oya-check-documentation --report-only` (frontmatter schema). `.omc/state/cross-session-coordination-2026-05-14.md` same. |
| **ADR-0090 supersession** | Per §6(c) — gated behind ADR-0091 existence; verified by `test -f docs/decisions/ADR-0091-*.md && grep -q "^Status: Accepted" …`. |
| **Cargo.toml edit** | Per §6(d) — verified by `cargo metadata --no-deps 2>&1 | grep -q "bounded_contexts"` (informational; the field is metadata, not load-bearing). Workspace-build verification gated behind cross-session 297-missing-crate blocker resolving. |
| **No regression** | None of the four edits change runtime code. lean-a10 self-test (when M02-P21 ships it) will protect the contracts; today no regression is possible because there's no runtime code change. |

## §6 Specific decisions (a-d)

### (a) Reclassify 20 session commits

| Crate / artifact (origin/main commits `4d6bf91` … `a8634fc`) | Count | Reclassification |
|---|---|---|
| `oya-ops-workspace-shell-{kernel,adapter,application,rest,runtime}` | 5 | **Wave 2 candidate** under `overview` BC at M03-P06 IP-Y1 if mapping exists at Wave 2 ralplan author-time; **retire via `git rm` + Cargo.toml removal** if no adoption (default). |
| `oya-ops-docs-portal-{kernel,adapter,application,rest}` (no `runtime` shipped) | 4 | **Prototype/learning artifacts.** The Wave 1 docs portal rebuilds against the 6 docs sub-plan BCs (`portal` / `generator` / `manifest` / `search` / `cross-ref` / `live-diff`) per docs sub-plan §6(a) at M03-P04 IP-X1 dispatch. The freelance crates retire at that point (their `pages/` route set doesn't match the 13 MVP page paths per docs sub-plan §6(c)). |
| `oya-http-{router,middleware,sse,runtime,tenant-mw,deadline-mw,telemetry-mw}-*` | 7 | **Already governed by R5.** R5 Step 1 renames `*-sse-domain` → `*-sse-kernel` and `*-{deadline,telemetry,tenant}-middleware-domain` → `*-middleware-runtime`. No reclassification needed by this plan; defers to R5 §10 Step 0..1 dispatch. |
| `oya-foundry-architecture-map-{kernel,app}` | 2 | **Retain as cross-cutting work.** Phase home is M02-P21 architecture-planes-green (visualization-as-code directive substrate; relates to ops-portal v7 Wave 2 `architecture` BC at M03-P06 IP-Y4 but is not itself a Wave 2 deliverable — it's the substrate that Wave 2 architecture BC composes against). Confirm phase placement at next M02-P21 phase-spec review. |
| `oya-check-{active-artifact-contract,cedar-fragment-coverage,openapi-rest-route-parity}` | 3 | **Retain as cross-cutting lane scaffolds.** Phase home is M02-P20 IP-005 expansion (registered as planned lanes; rampup-trajectory per docs sub-plan §6(f) → report-only → BLOCKER at M02-P22). `cedar-fragment-coverage` partially overlaps with `lean-a9-ops-policy-coverage` per ops-portal v7 §6(b); reconcile at M02-P20 IP-X dispatch. `openapi-rest-route-parity` overlaps with `lean-a7-endpoint-coverage` per docs sub-plan §6(f); same reconciliation. `active-artifact-contract` is a fresh check; phase home M02-P20 IP-X. |
| `contracts/ops-{workspace-shell,docs}.openapi.yaml` | 2 | **Retire at Wave 1 substrate dispatch.** The Wave 1 docs portal uses 13 Leptos page paths (per docs sub-plan §6(c) MVP set), NOT OpenAPI 3.2 REST. The two contracts are misaligned with Bominal ADR-0209 inheritance. Retain in HEAD until Wave 1 substrate lands at M03-P04 IP-X1; retire then via `git rm`. |
| `docs/decisions/ADR-0090-hyper-canonical-http-backbone.md` | 1 | **Mark Superseded by ADR-0091** per §6(c). Body remains factual. |

**Total: 5 retire-candidate + 4 prototype + 7 R5-governed + 2 retain-cross-cutting + 3 retain-lane-scaffold + 2 retire-on-Wave-1-land + 1 ADR-supersede = 24 artifacts across the 20 commits (some commits touch multiple artifacts).**

### (b) Workspace-shell adoption-or-retire trigger

The 5 `oya-ops-workspace-shell-*` crates are reclassified as **provisional Wave 2 candidate substrate** with the following retire trigger:

- **At Wave 2 ralplan author-time** (M03-P06 IP-Y1..Y4 dispatch), the Wave 2 ralplan §6(a) BC inventory MUST decide: does any of `overview` / `dashboards` / `tech-stack` / `architecture` BC consume the workspace-shell substrate?
  - **If yes (one BC adopts):** the workspace-shell crates rename to `oya-ops-<bc>-shell-{kernel,adapter,application,rest,runtime}` and ship as that BC's substrate. The Wave 2 ralplan §6(a) BC count for that BC drops accordingly.
  - **If no (no BC adopts):** the 5 crates retire via `git rm -r crates/oya-ops-workspace-shell-* && <remove 5 workspace-member lines from Cargo.toml>`. ADR-amendment block in the Wave 2 ralplan §8 records the retirement justification.

Until Wave 2 ralplan landing, the 5 crates remain in HEAD as "provisional" — lean-a8 (when M02-P21 ships it) will treat them as "in-flight Wave 2 substrate" via a `wave_provisional_until` field in `[package.metadata.oya]` (NEW; tracked at M02-P21 IP-005 expansion as a `lean-a8` allowlist exemption with hard sunset = Wave 2 ralplan acceptance date).

### (c) ADR-0090 supersession edit (gated on ADR-0091 existence)

**The edit** (atomic single-commit; deferred until ADR-0091 lands Accepted):

`docs/decisions/ADR-0090-hyper-canonical-http-backbone.md` line 5:

```diff
- Accepted (2026-05-14).
+ Superseded by ADR-0091 (2026-05-14; per ralplan-dep-seam-phaseout-round-5.md §8 ADR-0091 phase-out trajectory).
```

Plus append-only `## Supersession note` section at the end:

```markdown
## Supersession note (2026-05-14)

ADR-0091 (`workspace-dependency-seam-debt-ledger-phaseout`) supersedes the
"Accepted in perpetuity" status of this ADR. The hyper / hyper-util /
tokio / http-body-util / bytes backbone REMAINS the canonical HTTP stack
through M03 + the 5-product release; it is governed by ADR-0091's
phase-out trajectory beginning post-ontology-v1-stable. The technical
content of this ADR (the 5-crate layering at §"Building-block layering")
remains factually accurate as the description of the current backbone.

Authority transfers: ADR-0091 + ADR-0092 + ADR-0093 + ADR-0094 govern
seam discipline, ledger semantics, CI-only-read-side carve-out, and
SSE kernel/runtime split respectively per round-5 §8.
```

**Gating:** this edit DOES NOT land until `docs/decisions/ADR-0091-*.md` exists at Status: Accepted. Per R5 §10 Step 4, ADR-0091 is drafted at Step 4 (Proposed) and Accepted somewhere between Step 4 and Step 7 depending on consensus loop closure. The ADR-0090 supersession is the same-PR companion to whatever PR lands ADR-0091 Accepted.

**If R5 W0 has not yet started at this plan's acceptance time:** §6(c) does NOT block — the ADR-0090 supersession is a deferred deliverable carried in this plan's follow-ups list, NOT a blocker on this plan's acceptance. The catalog-registration edit (§6(d)) ships independently.

### (d) M02-P19 IP-X1 catalog-registration delta

**Current state** (`Cargo.toml` line 637-643 at HEAD `4d6bf91`):

```toml
[workspace.metadata.oya.microservices.ops]
owner = "council-foundry"
rationale = "Ops µservice — hyperscaler operations console (`ops.oyatie.com`); …"
adr_cite = "ADR-0067"
bounded_contexts = ["docs", "workspace"]   # Wave 1 only; Waves 2-7 add 18 more BCs per ralplan-ops-portal-2026-05-13.md §6(a)
naming_scope_adr = "ADR-NNNN-microservice-ops"   # authored at M02-P19 IP-X1 (docs §8 follow-up #1)
status = "planned"
```

**Target state** (delta):

```toml
[workspace.metadata.oya.microservices.ops]
owner = "council-foundry"
rationale = "Ops µservice — hyperscaler operations console (`ops.oyatie.com`); …"
adr_cite = "ADR-0067"
bounded_contexts = ["docs"]   # Wave 1 only (per ralplan-ops-portal v7 §6(a)); Waves 2-7 add 19 BCs at their respective ralplan landings
naming_scope_adr = "ADR-NNNN-microservice-ops"
status = "planned"
```

**Diff: one line.**
- `bounded_contexts = ["docs", "workspace"]` → `bounded_contexts = ["docs"]`
- Comment trailer updated: `…Waves 2-7 add 18 more BCs…` → `…Waves 2-7 add 19 BCs…` (because dropping `workspace` from Wave 1 means the remaining 19 BCs distribute across Waves 2-7; per ops-portal v7 §6(a) the total is 20 BCs and Wave 1 = `docs` only).

**Rationale:** `workspace` is not a Wave 1 BC per ops-portal v7 §6(a). The 14-surface workspace-shell is, at most, a Wave 2 candidate substrate (§6(b) above) and at minimum a retire-candidate. Until Wave 2 ralplan adopts it, declaring it as Wave 1 BC is silent drift.

**This edit ships TODAY** independent of:
- Cross-session 297-missing-crate blocker (it's a TOML metadata edit, not a workspace-member change).
- ADR-0090 supersession (`§6(c) is gated on ADR-0091; §6(d) is gated on nothing).
- R5 W0 dispatch.

## §7 Risk register

| ID | Risk | Mitigation |
|---|---|---|
| R1 | Workspace-shell crates linger as dead skeletons until Wave 2 ralplan accepts/rejects | §6(b) adoption-or-retire trigger; lean-a8 catches at M02-P21 ship |
| R2 | ADR-0090 supersession lands before ADR-0091 Accepted | §6(c) gating; deferred-to-same-PR-as-ADR-0091 strategy |
| R3 | Catalog-registration edit breaks an unknown validator consumer | §4 Scenario 3 prevention (grep validator paths before edit); §6(d) is metadata-only so blast radius is small |
| R4 | Reclassification gets adopted but the underlying freelance pattern (hyper-direct REST, OpenAPI 3.2 contracts) seeps into Wave 1 substrate authoring at M03-P04 IP-X1 | docs sub-plan §6(a) authority enforced at dispatch time (Leptos `pages/` module inside `-rest`); architect review at IP-X1 acceptance gates the substrate against the freelance pattern |
| R5 | Cross-session 297-missing-crate blocker stays open past Wave 1 substrate dispatch | `.omc/state/cross-session-coordination-2026-05-14.md` surfaces to parallel session; ops-Wave-1 session work is structured to NOT depend on resolution for these 4 deliverables |

## §8 ADR record

- **Decision**: Reclassify 20 ops-Wave-1-misaligned session commits into 7 buckets (§6(a)); gate ADR-0090 supersession on ADR-0091 acceptance (§6(c)); land the M02-P19 IP-X1 catalog-registration delta TODAY as a 1-line `Cargo.toml [workspace.metadata.oya.microservices.ops].bounded_contexts` edit (§6(d)); workspace-shell crates carry adoption-or-retire trigger to Wave 2 ralplan author-time (§6(b)).
- **Drivers**: cross-plan coherence (3 Accepted plans + freelance commits must compose); smallest-actionable (10-line Cargo.toml edit ships today); honest classification over speculative retention.
- **Alternatives considered**: Option Ω (revert all 20 commits) — rejected; cost > benefit. Option β (reclassify but don't supersede ADR-0090) — rejected; leaves silent contract drift between ADR-0090 and R5.
- **Why chosen**: Option α composes with all three load-bearing Accepted plans (ops-portal v7, docs-portal v7, dep-seam-phaseout R5); preserves genuinely useful cross-cutting work (architecture-map crates, 3 check kernels, 7 hyper-foundation crates already governed by R5); resolves the ADR-0090↔R5 contradiction explicitly; defers actual Wave 1 substrate authoring until prereqs land per docs sub-plan §6(g).
- **Consequences**:
  - Positive: zero revert; zero new compile-blocker; cross-plan coherence restored; one of four follow-ups (catalog-registration) lands today.
  - Negative: 5 workspace-shell + 2 OpenAPI contracts + (potentially) 4 docs-portal-prototype crates carry retirement debt to dispatch time; lean-a8 (M02-P21) will surface this via allowlist exemption with hard sunset.
  - Neutral: 7 hyper-foundation crates retained verbatim under R5 governance.
- **Follow-ups** (per RALPLAN-DR step 6 contract):
  1. **TODAY:** land §6(d) Cargo.toml edit + this ralplan + `.omc/state/cross-session-coordination-2026-05-14.md`.
  2. **At ADR-0091 Accepted (R5 W0 Step 4-7):** land §6(c) ADR-0090 supersession edit (same-PR-as-ADR-0091).
  3. **At Wave 2 ralplan author-time (post-Wave-1-substrate-land):** decide §6(b) workspace-shell adoption-or-retire.
  4. **At M03-P04 IP-X1 dispatch:** retire `contracts/ops-{workspace-shell,docs}.openapi.yaml` + 4 `oya-ops-docs-portal-*` prototype crates as part of the Wave 1 substrate rebuild against docs sub-plan §6(a) BCs.
  5. **At M02-P20 IP-005 expansion dispatch:** reconcile the 3 freelance `oya-check-*` crates against the lean-a7/a8/a9 inventory per docs sub-plan §6(f) + ops-portal v7 §6(b); rename or merge where overlap exists.
  6. **At M02-P21 phase-spec review:** confirm `oya-foundry-architecture-map-{kernel,app}` phase home.

## §9 Verification status

| Round | Architect | Critic | Codex | Iteration delta |
|---|---|---|---|---|
| 1 (draft) | _pending_ | _pending_ | _pending_ | initial draft (2026-05-14; session 7e0309c2) |

Acceptance criteria per RALPLAN-DR SHORT-mode (7 deliberate-mode dimensions condensed to 4 because this is a close-out, not a fresh substrate plan):

1. **Cross-plan composition** — composes with ops-portal v7 + docs-portal v7 + R5 (all three referenced in §0 + §6).
2. **Concrete edits** — §6(c) shows the exact `git diff` for ADR-0090; §6(d) shows the exact `Cargo.toml` line delta.
3. **Risk mitigation clarity** — §7 R1..R5 each names trigger + mitigation + detection lane.
4. **Smallest-actionable** — §6(d) is the only "today" deliverable; §6(b)+(c)+rest are deferred to natural future trigger points.

On Architect+Critic APPROVE: status flips `pending architect+critic` → `Accepted`; §6(d) Cargo.toml edit lands in the same PR; §6(c) ADR-0090 edit is deferred to the R5 W0 ADR-authoring PR.

## §10 Glossary cross-walk (per `feedback_glossary_shared_not_platform.md` + `feedback_glossary_ontology_not_object_graph.md`)

| Term used in this plan | Canonical per MASTERPLAN §2.4 |
|---|---|
| Application B2B shell | Application (capital A) ✓ |
| µservice | µservice ✓ |
| BC / Bounded Context | BC ✓ |
| Workflow Studio | Workflow Studio ✓ |
| Ontology (Palantir-equivalent) | Ontology ✓ (not Object Graph) |
| Ops µservice | ops ✓ (flat catalog; not "ops product group") |

No retired-glossary terms used in this plan.
