# Oyatie Pipeline-as-Product — Program Operating Plan

date: 2026-06-24
status: synthesis for founder review (capability/PR counts being confirmed by recon; design is context-grounded)
authors: orchestrator synthesis per founder directives 2026-06-23/24
supersedes-context: folds [[pipeline-universal-product]], [[pipeline-four-property-bar]], [[corpus-governance-substrate]], [[optimal-monorepo-shape-cellular-hub-aware]], [[friction-is-process-failure-productize]], [[automation-maximalism-staleness]]

---

## 0. North Star (the reframe that orders all work)

The deliverable is **not** "oyatie's repo is clean." It is a **paved-road platform-as-product**: an average, non-technical person states intent, and the pipeline produces an application that is — *by construction* — highly scalable, highly secure, highly optimized, cloud-native, hermetic, oyatie-cloud-optimized yet portable.

**Corollary that reorders everything:** every gate, accounting rule, codemod, and materializer we build to keep our own monorepo correct **is a product feature** of that platform — it is the mechanism by which a non-expert ships production-grade without knowing why. Therefore:

- The bar for each is the **7-property product bar** (UNIVERSAL neutral engine + policy-as-data packs · PRODUCTIZED engine+packs+control-plane+public-contract · HERMETIC pure-Rust/buck2/no-shell-net-clock-rand · AUTOMATED ships-its-own-auto-fix-not-flag-only · CLOUD-NATIVE CRD/operator/typed-API · MODERN right-tool · LATEST-info) — never "good enough for us."
- **DX is a first-class acceptance criterion.** A developer (or the platform's end user) must never hit a gate failure they can't auto-resolve. Green-by-construction is the goal; a red gate without a one-command fix is itself a product defect.
- Portability is a constraint: oyatie-cloud-optimized, but the hermetic/declarative substrate must run on any cloud / any repo.

Everything below is in service of this.

---

## 1. The single invariant under all six founder asks: **Total Accounting + Liveness**

The founder's distinct asks — (a) docs/ADRs auto-maintaining + drift impossible, (b) every file/code/json/yaml/toml accounted-for with a reason to exist, (c) merge conflicts = bad design, (d) parallelization-friendly, (e) pipeline-as-product/enterprise-bar/DX, (f) the north star — are **one invariant** wearing six hats:

> **Every artifact is a typed, owned, live node in a content-addressed graph; every human-facing view (docs, ADRs, catalogs, indices) is a DERIVED build artifact of that graph; staleness/drift is structurally caught fail-closed; nothing exists without a registered reason.**

This is the **corpus-governance-substrate** (founder-approved 2026-06-23, executing Phase -1). It is the keystone because it collapses asks (a)(b)(d) simultaneously:

- **(a) Docs/ADRs auto-maintaining, drift impossible:** ADRs/PRDs/catalogs/indices stop being hand-maintained markdown that drifts. They become **typed nodes** (decision-node, capability-node) whose markdown rendering is a *derived artifact* (`docs/` = generated view). A code change that invalidates an ADR's premise fails the **liveness gate** (Cedar invariant, fail-closed) — drift is impossible because the doc is recomputed from the live graph, not edited. `/documentation-and-adrs` protocol becomes: edit the node, the view regenerates; you cannot commit a stale view.
- **(b) Everything accounted-for:** total-accounting (GATE-2, already shipped) + born-accounting already make unaccounted artifacts unmergeable for *crates*. Extend the node taxonomy to **every** file class (json/yaml/toml/proto/sql/cedar/md) so each has an owner + reason-to-exist node or it fails admission. The root-workspace-hygiene gate (shipping now) is the first extension to the file-system root; the corpus generalizes it to all paths.
- **(d) Parallelization-friendly:** see §2 — merge conflict = a committed *whole-tree derivation*; the graph derives views on-demand, so there is no committed surface to conflict on.

**Granularity = AST, not file (founder bar, 2026-06-24): "account for every code element through AST → a complete grasp on the entire code graph."**
File-level total-accounting (every json/yaml/toml/md/proto has an owner + reason-to-exist) is the **boundary**; **AST-level accounting is the interior**: every `fn`/`type`/`trait`/`impl`/HTTP-route/Cedar-policy/SQL-table is a typed node, and every call/dependency/capability relationship is an edge. The corpus extractor makes the interior LIVE — syn v1 for Rust today, the contract-IDL extractor family (#129) for proto/OpenAPI/Cedar/SQL next (syn-only = ~45% true-miss on `include_proto!`-style codegen, so IDL extractors are not optional). With the full AST graph:
- nothing in the codebase is invisible — the graph *is* the grasp;
- ADRs/catalogs/SLOs/indices are **derived views** over it (auto-maintaining);
- liveness invariants fail-closed the instant code moves out from under a claim (drift impossible);
- and — closing back to the north star — the platform can only auto-generate production-grade apps for a non-expert if it holds a **complete, live model of the code**. The code graph is simultaneously our governance substrate AND the product's reasoning substrate. Same artifact.

**Phase 0 of the corpus (task #128)** — typed governance nodes + shard the 12.6 MB accounting registry + the contract-IDL extractor family (#129) — is therefore the **highest-leverage foundational investment**, because it is the common substrate for (a)(b), the AST code-graph the founder is asking for, and the long-term answer to staleness.

---

## 2. Merge-conflict elimination = parallelization by design

**Principle: no committed artifact may be a whole-tree derivation. If two independent PRs conflict on a file, the file is mis-designed — shard it, derive it on-demand, or scope it per-cell.** Merge conflict is never "resolve it"; it is a design bug in the artifact.

Root causes seen this session and their structural fixes:

| Conflict source | Why it serializes PRs | Structural fix | State |
|---|---|---|---|
| 7 whole-tree `*.generated.json` faces | every PR rewrites all → all-pairs conflict | **de-commit** pure-view faces (derive-on-demand) + **universal `oya-ci-materializer`** (E1-E6) so CI derives, nothing committed | #828 ✅, #831 guard ✅, E1 #833 in-flight, **E2-E6 pending** |
| `gate-baseline` / `scm-facts` (frozen-ref + SCM facts) | committed, every merge mutates | merge-base ratchet (ADR-0551) + scm-facts split (ADR-0552); frozen-ref must stay committed (it's the ratchet anchor) — but it's tiny & merge-base-keyed so low-conflict | landed; residual mini-treadmill on scm-facts |
| ADR/PRD ids = monotonic counter vs stale base | two PRs grab the same id (saw 0597 ×3 this session) | **reserve ids across in-flight open PRs** (allocate-on-open / content-hash / merge-queue re-stamp) | friction filed **#138** |
| stale PRs rot into conflict | no auto-rebase | **auto-rebase / merge-queue rebase** on stale-but-approved PRs | **#123 pending** |
| one global registry / DAG / capability-registry | every capability edits the same file | **de-globalize**: shard `<cap>/.facts/`, derive global view on main only; per-cell nested `[workspace]` | per [[optimal-monorepo-shape-cellular-hub-aware]], **pending founder go** |

**The parallelization engine = the cellular disjoint-scope speculative merge queue** (ADR-0515 Tide model): independent capabilities (disjoint path scopes) merge concurrently and never serialize; only shared-hub changes serialize. Combined with stable-port+semver between cells, this is how N capabilities → real N-way throughput. **Key correction (already internalized): N cells ≠ N-way parallel** — low-rung substrate is a high-fan-in HUB; parallelism comes from *stable ports*, not from pretending hubs are independent.

---

## 3. Parallelization map, dependencies, and prioritization

### Dependency structure (what blocks what)
```
TIER A — throughput multipliers (fix once, everyone speeds up). HIGHEST PRIORITY.
  ├─ universal materializer E2-E6        (kills faces-merge friction globally)
  ├─ ADR/PRD id reservation #138         (kills allocation collisions)
  ├─ auto-rebase stale PRs #123          (kills rot-into-conflict)
  ├─ green-by-construction pre-push #104 (kills the buck2-green≠CI-green grind; saw it cost #833 31 red gates)
  └─ register_crate AUTO-on-birth        (kills the onboarding grind — scaffold exists #105, make it mandatory)
        │ unblocks ↓ (less serialization, faster everything below)
TIER B — foundational substrate (enables auto-governance + staleness-death).
  ├─ corpus Phase 0 #128  (typed nodes + shard 12.6MB registry)  ──┐
  └─ contract-IDL extractors #129                                   ├─ enables auto-docs/ADRs (a), total file-accounting (b)
TIER C — security keystone (must-not-ship-broken). Parallel per boundary.
  └─ AUTH-005 fleet remediation (11+ forgeable-authz trust boundaries; #815/#99 in flight; codex review found ~30 CRIT/98 HIGH)
TIER D — capability moves (mechanical, gated, parallel BEHIND hub-stability).
  ├─ move-22 intelligence (FINAL, 142 crates, ~7 batches) — in progress #86
  ├─ app-product homing: connect ✅#811 · meet ✅#810 · calendar ✅#812 · then design-collaboration/docs/sheets/office/crm/hr #126
  └─ tools/ + libs/ DISSOLVE into capabilities; microservices/ removal-candidate
TIER E — durable product verticals (parallel, each independent).
  └─ G005 SCIM ✅ · G006 tenancy ✅ · G009 billing-accounting durable store · G002 SVID durable CA #108
```

### Prioritization rule
**Sequence by unblock-power × risk, not by FIFO.** Tier A first (each multiplies everyone's velocity and directly attacks the merge-conflict/DX friction the founder is calling out). Tier B in parallel (foundational, different skill-surface, no contention with A). Tier C always-on in its own lane (security cannot wait behind throughput). Tier D/E fan out *behind* hub stability — a capability move only parallelizes once the hubs it depends on are port-stable.

### What runs concurrently *right now* (no contention)
- Materializer E2 ‖ AUTH-005 boundary-N ‖ corpus Phase-0 ‖ capability-move batch ‖ a durable-vertical slice — five independent lanes, different files, no shared mutable surface once Tier-A landed.

---

## 4. The friction-closing pipeline-as-product loop (the universal/hermetic/automated workflow)

This is the engine that closes its own loops — the thing the founder means by "pipeline that is a product."

```
detect ──► LEDGER ──► CLASSIFY ──► PRODUCTIZE ──► RATCHET ──► VERIFY-CLOSED
(any        (append-   (it is a    (ship a        (merge-     (terminal only when the
 stage)      only,      PROCESS     hermetic gate   base        defining condition is
             id-merge    defect —   that makes the  baseline,   FALSIFIED on the PRODUCTION
             driver✅)   fix the     class           no          path — not a test-only
                         CLASS)      IMPOSSIBLE +    regression)  capability; gate #46)
                                     auto-fix,
                                     not flag-only)
```

Invariants that make it a *product*, not an oyatie patch:
- **Universal:** neutral engine + policy-as-data pack. The gate runs on *any* repo; oyatie's rules are just one pack. (This is also literally a sellable feature: "the platform enforces your org's invariants.")
- **Hermetic:** pure-Rust predicate, buck2, no shell/net/clock/rand → deterministic, cacheable, reproducible.
- **Automated / auto-fix:** every gate ships a remediation. Flag-only is an *incomplete* gate (enforcement-layering doctrine). DX = the dev/end-user gets the fix, not a lecture.
- **Cloud-native:** CRD + operator + typed API surface, not a CLI. (no-shell / no-new-CLI doctrine.)
- **Closed-loop:** a recurrence is caught by the gate, not a human. The friction-ledger trends to zero open *classes*, not zero *instances*.

**Auto-governance corollary (answers "docs maintenance-free, drift impossible"):** docs/ADRs are derived nodes (§1); the liveness gate is the VERIFY-CLOSED step for documentation drift. You cannot merge a code change that strands an ADR premise — the gate denies it and points at the node to update. Maintenance trends to zero because the human edits *intent nodes*, never *views*.

---

## 5. The meta-workflow to run it (maximize productivity, minimize friction)

Per work-item, a **pipelined fan-out** (no barriers; item A can be verifying while item B implements):

```
IMPLEMENT ─► ADVERSARIAL-VERIFY ─► AUTO-ONBOARD ─► GREEN-BY-CONSTRUCTION ─► ARM ─► MERGE-QUEUE
(executor,    (cross-model codex    (register_crate   SELF-CHECK            (auto-   (cellular
 worktree)     on security/gate;     auto on birth;   (freshness+faces+     merge)    disjoint-
               caught a CRITICAL      no 31-red)       affected-set BEFORE             scope,
               in EVERY sec PR)                        push — task #104)               serialize
                                                                                       only hubs)
        ▲                                                                                │
        └──────────────── FRICTION lane (§4) runs in PARALLEL, feeds gates back ◄────────┘
```

This is exactly the loop I've been running **manually** this session (implement → cross-model review → fix → onboard → arm). **The productization is to make each arrow automatic:**
- AUTO-ONBOARD: register_crate fires on crate birth (today it's invoked manually and #833 skipped it → 31 red gates). *Make it a precondition of the crate-creation codemod.*
- GREEN-BY-CONSTRUCTION: the pre-push self-verify (#104, in progress) runs freshness+faces+affected-set locally and refuses to push red. This single automation would have prevented the #833 incident entirely.
- Graph-invisible-test ratchet (#77) + affected-set as data inputs (#71) close the "gate that can't see its own inputs" blind spots.

---

## 6. CI/CD as enterprise product + DX (the acceptance bar)

"Passes the enterprise production bar as a product" decomposes into:

- **Engine** (neutral gate runtime) + **policy packs** (per-org rules as data) + **control-plane** (CRD/operator, projected merge state, Tide admission) + **public contract** (the one required check `oya-ci-required`, versioned) + **SLOs** (every promoted service has `*.openslo.yaml`; slo-coverage gate already enforces) + **observability** (structured telemetry, not shell-and-look).
- **DX surface:**
  - one-command scaffold: `register_crate`-style birth → a new capability arrives fully accounted, catalogued, tier-tagged, SLO-stubbed, gate-green.
  - green-by-construction: the contributor never debugs CI; the auto-fix runs locally pre-push.
  - the **non-technical-person surface** (the north star): declarative intent → the platform generates the production-grade, secure-by-default (fail-closed authz, RLS, mTLS/SVID), hermetic, scalable app. The gate fleet is the guardrail that makes "non-expert" and "production-grade" compatible.
- **Portability:** the substrate is declarative + hermetic, so the same packs run on any cloud; oyatie-cloud is the optimized default, not a lock-in.

---

## 7. Recommended execution (prioritized, with parallelism)

**Now / in-flight (this session):** #833 E1 materializer (CI-fix running) · #810/#812 app-homing (armed) · root-hygiene gate (building) · #138 allocator friction filed.

**Immediate next (Tier A — throughput multipliers, run concurrently):**
1. **Materializer E2** (executor + shell byte-parity canary) — continue E1→E6; biggest single merge-friction kill.
2. **Green-by-construction pre-push self-verify #104** — prevents the #833-class incident for everyone; pure DX + CI-cost win.
3. **register_crate auto-on-birth** — fold onboarding into the crate-creation codemod so it can't be skipped.
4. **ADR/PRD id reservation #138** + **auto-rebase stale PRs #123** — kill the two remaining allocation/rot conflict classes.

**In parallel (Tier B foundational):**
5. **Corpus Phase 0 #128 + IDL extractors #129** — the auto-docs/total-accounting substrate; the durable answer to drift/staleness and the `/documentation-and-adrs` maintenance-free goal.

**Always-on lane (Tier C):** AUTH-005 fleet remediation, cross-model-verified per boundary.

**Behind hub-stability (Tier D/E):** finish move-22 intelligence; continue app-product homing; durable verticals.

**Founder decision gates (I will NOT proceed without explicit go):**
- the **whole-repo cellular-hub-aware migration** (Phase-0 preconditions: PUBLIC→scoped visibility, cross-cell core edges→base/ DAG-LUB) — pending founder approval + convergence with the corpus/AST determination.
- de-globalizing / deleting the committed registry (the 11–12 MB file) — safe strangler, but founder-grade blast radius.

---

## 8. Open items folded into the backlog (so nothing is lost)
#104, #123, #128, #129, #138, #77, #71, #46, #130/#131 (shell/CLI retirement), #126 (app-product homing), plus the materializer E2-E6 program and the corpus Phase-0 program. Each is a node with a reason to exist; this plan is the derived view over them.

---

## 9. Naming & code-quality doctrine (founder, 2026-06-24): "architect like hyperscalers, criticize like Torvalds"

Naming is not cosmetic — it is part of the maintainability product bar. Extend the de-brand grammar ([[naming-grammar-debrand-path-namespace]]) from crate-ids to **every identifier**:
- **Crates / capabilities / paths:** drop the vendor prefix (`oya-`/`oya_`), path == namespace, buck label == canonical id (`//iam/pdp:app`), cargo name == path tail. The repo-wide de-brand is a STOP-THE-WORLD deterministic-Rust rename (~821 crates, bijective, 0-collision); until it runs, *new* crates still match the current `oya-cloud-ci-*` convention for consistency, then ride the mass rename.
- **Gates:** kebab-case canonical id == firewall baseline id == `oya-ci.toml` gate id == disposition key (the 3-surface registration must agree, or the producer silently skips the gate — this is exactly why a gate can ship "green" while unmaterialized).
- **Functions / variables / types:** idiomatic Rust (snake_case fn/var, UpperCamel type/trait), names that state intent, no abbreviations that need a glossary, no lying names (the materializer's old `materialize_closure` *claimed* a closure it didn't compute — a Torvalds-grade naming defect that the re-review caught).
- **Files / docs:** path states purpose; no scratch in the root (the gate shipping now); generated views named `*.generated.*` and never hand-edited.

**Productize it (don't rely on review):** a hermetic **naming-convention gate** (universal engine + per-org convention pack) that enforces these mechanically — de-branded ids, gate 3-surface agreement, fn/var casing, no-vendor-prefix, no-lying-`generated` suffix. Pairs with the existing de-brand validators (#72). Torvalds-grade criticism is the *review posture* ([[quality-torvalds-review-discipline]]: verify intent AND execution separately, hostile inspection of the riskiest surface by hand); the gate is the mechanical floor so review can focus on what machines can't judge.

---

## 10. Status snapshot (recon-confirmed, origin/dev, 2026-06-24)
- **Crates ~897 total.** Homed under capability-first: **~292 across 21 capability dirs** (iam 67, workflow 48, data 23, comms 22, tenancy 22, intelligence 17, k8s 17, billing 16, secrets 10, gateway 10, …). **Un-homed: oya/ 246 · libs/ 189 · tools/ 27 · microservices/ 0.**
- **Reorg:** capability-registry **CLOSED** (ADR-0562); strangler **execution nascent** — ~1 of ~66 marked cross-capability "junk-drawer" crates moved (calendar in flight). 60+ homing PR cycles ahead, gate-enforced (no-new-top-level + membership-lint regression).
- **Approval arbiter:** the **planning-closure gate** is the gate (no separate founder marker). Gate-green unlocks the FD-001 (11 production-depth surfaces) + G001-G013 long-running execution claim. The whole-repo migration + registry de-globalization remain founder-blast-radius decisions.
- **Security:** AUTH-005 campaign **authorized**, standing merge authority — **~230 findings (54 CRIT / 119 HIGH)**. **Wave-0 = the DTO-authz class-fix gate must land first** (shrink-only on the frozen baseline), then instance waves fan out. This is the single highest-leverage security item.
- **Friction ledger:** 215 entries, ~85 open, G011-heavy (gate/CI hygiene). **Gate fleet: 35 apps** under `oya-ci-required`.
