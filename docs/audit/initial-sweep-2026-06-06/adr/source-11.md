# ADR Audit — SOURCE, Chunk 11

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 11
- **Range requested:** rows 71–77 of the sorted `docs/decisions/ADR-*.md` listing
- **ADRs actually reviewed:** ADR-0093, ADR-0094, ADR-0095, ADR-0096, ADR-0097, ADR-0098, ADR-0099 (7 of 7 — slice non-empty, fully covered)
- **Cluster shape:** Two sub-clusters. (A) **HTTP-seam micro-decisions** 0093/0094/0095 — children of ADR-0092 (workspace dependency-seam policy), all `accepted`, all sound. (B) **M02-P06 "foundry supervisor" cluster** 0096/0097/0098/0099 — all `accepted`, dated 2026-05-15, ALL saturated with the RETIRED `foundry` brand (retired 2026-05-21 by ADR-0335/0347, six days after these were accepted). The technical content survives; the brand/crate/namespace/policy-path strings are retired-vocab leakage.

---

### ADR-0093 — DeadlineMiddleware → LatencyBudgetReporter (honest naming)
- **decision_atom:** Rename the sync HTTP middleware type/crate/identifiers from `Deadline*` to `LatencyBudget*` because the chain cannot actually cancel in-flight work (the 504 is post-hoc), so the name must not imply real deadline enforcement; reserve the `DeadlineMiddleware` name for a future async-chain variant that can truly cancel.
- **current_status:** `accepted` / `doc_status: published` (2026-05-14).
- **disposition:** KEEP.
- **governing:** n/a (not superseded; child of ADR-0092 D5).
- **truth_flag:** TRUE.
- **in_masterplan:** NO — bare `id/status/doc_status` front-matter only; no `planning_impact`, `masterplan_ref`, `supersedes`/`related` machine fields. Falls in the 8.8%-unbound majority (planning-ssot-drift-prevention.md). Reflected in MASTERPLAN.md: NO (too granular).
- **tensions:** None substantive. Pure naming-honesty fix; "name must match behavior" is the same F5/multispectrum quality bar invoked by 0094/0095 — consistent intra-cluster.
- **hyperscaler_challenge:** ALIGNED. Google/AWS/Azure all enforce "names must not lie about semantics" in API-review; a post-hoc latency reporter that calls itself a deadline enforcer would be flagged in any of their design reviews. Argues for KEEP.
- **ai_slop:** Minor. "Extra ADR in the citation graph" as a listed Negative is filler self-reference; the adversarial-test name baked into the Decision is good, not slop. No fabricated precision.
- **refinement:** Add planning front-matter (`related: [ADR-0092]`, `supersedes: []`) so it binds for any generated masterplan. Consider folding 0093/0094/0095 into ADR-0092 as recorded sub-decisions if the corpus ever does the ADR-0000 re-founding (planning-ssot-consolidation.md) — they are arguably D-clauses of 0092, not standalone ADRs.
- **consensus_needed:** no.

---

### ADR-0094 — `Handler` trait with associated `Error` type
- **decision_atom:** Add an additive typed `Handler` trait (`type Error: Into<HttpResponse>`) plus a `handler_to_sync` bridge in the hyper adapter, so handlers can return structured typed errors that render uniformly at the framework boundary, without breaking existing closure-shaped handlers.
- **current_status:** `accepted` / published (2026-05-14).
- **disposition:** KEEP.
- **governing:** n/a (child of ADR-0092 D11; related ADR-0090).
- **truth_flag:** TRUE.
- **in_masterplan:** NO — minimal front-matter, no planning fields, not in MASTERPLAN.md. Same unbound-ADR pattern.
- **tensions:** Self-acknowledged "two ways to define a handler" needs a style-guide pointer — an open follow-up, not a conflict. No cross-ADR contradiction.
- **hyperscaler_challenge:** ALIGNED. The `Into<HttpResponse>` typed-error pattern is idiomatic (mirrors axum/tower `IntoResponse`); AWS/Google Rust service frameworks converge on exactly this. The "no blanket impl due to coherence" reasoning is technically correct. Argues for KEEP.
- **ai_slop:** None of note. Code blocks are concrete and correct; the coherence/`Into`-vs-`Box<dyn Error>` rationale is real engineering, not hedging.
- **refinement:** Resolve the "two handler styles" follow-up (point the style guide at the typed trait for new handlers). Add the `F-HANDLER-ASYNC` future variant as a tracked task ref in front-matter.
- **consensus_needed:** no.

---

### ADR-0095 — `TenantSlug` centralized in `oya-tenancy-kernel`
- **decision_atom:** Move the customer-facing tenant-id grammar out of HTTP middleware into a new `TenantSlug` newtype in `oya-tenancy-kernel` (distinct from the internal `TenantId(ten_*)`), giving a single source of truth for slug validation and compile-time prevention of wire-id ↔ internal-id confused-deputy errors.
- **current_status:** `accepted` / published (2026-05-14).
- **disposition:** KEEP.
- **governing:** n/a (child of ADR-0092 D10; depends on ADR-0056 layer rules).
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — touches the tenancy keystone domain (canonical: tenant = universal scoping primitive, ADR-0244/0242/0329), and is compatible with it, but carries no `planning_impact`/`masterplan_ref` front-matter and is not surfaced in MASTERPLAN.md. The *concept* aligns with the masterplan's tenant model; the *binding* is absent.
- **tensions:** Compatible-not-conflicting with the tenancy keystone. The `TenantSlug`/`TenantId` split is the kind of detail the tenant-model spec (`/specs/tenant-model.json`) should own — mild overlap risk if that spec also defines slug grammar (verify single-source-of-truth holds across ADR + spec). No tier-vocabulary contamination (this is identifier grammar, not the retired tenant `tier-system`).
- **hyperscaler_challenge:** ALIGNED. Distinct external-slug vs internal-id types with a gateway lookup is exactly the GCP project-id-vs-project-number / AWS account-alias-vs-account-id pattern. Strong precedent. Argues for KEEP.
- **ai_slop:** None. The 13-test adversarial fixture list (homoglyph defense, path-traversal-shape rejection) is concrete and security-relevant, not padding.
- **refinement:** Cross-link to `/specs/tenant-model.json` and ADR-0244/0242 in front-matter so the slug grammar has one authoritative home. Complete `F-TENANTID-FORMAL` (slug→id lookup/caching/audit) when the auth slice lands.
- **consensus_needed:** no.

---

### ADR-0096 — Supervisor language: Rust, not Node (build-vs-adopt Siigari/claude-heartbeat)
- **decision_atom:** Build the session supervisor (hook + JSONL inbox/outbox, multi-account Claude/Codex/Gemini management) as native Rust crates rather than adopting the upstream Node `Siigari/claude-heartbeat`, because in-process Rust gives crash-atomic Cedar enforcement + audit emission and zero-copy kernel-type sharing, which a Node IPC sidecar cannot.
- **current_status:** `accepted` / published (2026-05-15). Carries proper planning front-matter (`owner_phase: M02-P06`, deciders, related).
- **disposition:** AMEND. The *build-in-Rust* decision is correct and durable; but the doc is saturated with the RETIRED `foundry` brand ("the foundry supervisor", crate names `oya-foundry-route-policy-kernel`, `oya-foundry-account-domain`, `oya-foundry-autonomy-ceiling-app`, `oya-foundry-jsonl-supervisor-adapter`). Per ADR-0335 (2026-05-21) foundry the µservice/brand is retired and absorbed by **intelligence**; per ADR-0347 the CI/crate prefix `oya-foundry-*` → `oya-governance-*` (or intelligence). Amend: re-home the supervisor under intelligence (`oya-intelligence-*` / `cloud-intelligence`) naming, keep the technical decision verbatim.
- **governing:** Brand retirement governed by **ADR-0335** (foundry→intelligence) + **ADR-0347** (oya-foundry-* → oya-governance-*). The supervisor decomposition itself is not superseded — only its namespace.
- **truth_flag:** PARTIAL — decision TRUE; brand/crate-name layer STALE (retired six days after acceptance).
- **in_masterplan:** PARTIAL — has `owner_phase`/deciders/related planning metadata (better than 0093–0095), but `M02-P06` phase milestones and "foundry" wave naming are themselves anachronistic post-retirement; not reconciled into MASTERPLAN.md's current intelligence posture.
- **tensions:**
  - **vs ADR-0335/0347** — direct retired-vocab conflict on `foundry` brand and `oya-foundry-*` prefix (the load-bearing tension for this whole sub-cluster).
  - **vs cited ADR-0042 (related)** — ADR-0042 (OTel gen_ai semconv) is itself `superseded` by ADR-0383 (Loki/Tempo/Mimir/Grafana); the `related: [ADR-0042]` ref points at a superseded ADR and should be re-pointed to ADR-0383/0263.
  - **vs LINUX side** — LINUX has no supervisor analog; this is a source-internal AI-orchestration concern. (Note: the Node tool being rejected here, claude-heartbeat, is conceptually adjacent to the agent-execution-controller in PR#605 — both are agent-session-management surfaces; worth a founder cross-check that they are not two designs for one thing.)
- **hyperscaler_challenge:** ALIGNED on the principle, QUESTIONABLE on scope. Google/AWS/Azure DO enforce single-language-runtime purity for crash-atomic audit paths and would also build in-house rather than adopt a hobby Node daemon for a security-sensitive supervisor. But none of them would build a bespoke multi-vendor-CLI session supervisor at all — they'd run agents as managed jobs. The decision is internally rational; the *existence* of this surface is the open question. Argues for AMEND (rebrand), not archive.
- **ai_slop:** Low. The grit-unit counts ("49 + 14 grit units") are fabricated-precision-adjacent (estimate dressed as exact), and "grit" is itself retired external-coord-tooling vocab (ADR-0116/0054 retired grit/rtk/icm/vox) — double leakage. Otherwise the cost/atomicity reasoning is substantive.
- **refinement:** (1) Rebrand foundry→intelligence across the doc per ADR-0335. (2) Re-point `related: ADR-0042` → ADR-0383. (3) Drop "grit unit" estimates or restate as the current intelligence/oya-ci unit vocabulary. (4) Reconcile `M02-P06` milestone tokens against current wave naming.
- **consensus_needed:** yes — "Post-foundry-retirement, does the bespoke Rust multi-CLI session supervisor survive as an `intelligence` capability, or is it superseded by the agent-execution-controller (PR#605) — i.e., is claude-heartbeat-shaped supervision still a thing we own?"

---

### ADR-0097 — Rename oya-foundry-account-adapter-* (layer token must be last)
- **decision_atom:** Rename the three provider CLI-driver crates so the BNF-v4.1 layer token `adapter` is the final segment (`oya-foundry-account-adapter-claude-code` → `oya-foundry-claude-account-adapter`, etc.), making them pass `oya-check-architecture --layer-correctness`/`--lib-name-parity` without special-casing.
- **current_status:** `accepted` / published (2026-05-15).
- **disposition:** AMEND (strongest AMEND/near-ARCHIVE in chunk). This is a rename ADR whose *target* names are themselves now retired: it renames `oya-foundry-account-adapter-X` → `oya-foundry-X-account-adapter`, but the entire `oya-foundry-*` prefix was retired six days later by ADR-0347 (→ `oya-governance-*`) and the foundry brand by ADR-0335 (→ intelligence). So this ADR fixes the *layer-position* defect but leaves the doomed *microservice token*. The BNF-conformance principle (layer token last) is permanent and correct; the concrete target names are obsolete-on-arrival.
- **governing:** Naming/brand governed by **ADR-0347** (oya-foundry-* → oya-governance-*) + **ADR-0335** (foundry→intelligence). BNF rule itself governed by ADR-0056 (live).
- **truth_flag:** PARTIAL — BNF layer-last rule TRUE/permanent; the specific renamed crate names STALE (correct microservice token would now be `intelligence`/`governance`, not `foundry`).
- **in_masterplan:** NO as a masterplan-level decision (it is a crate-naming chore); it does carry phase/decider front-matter. The naming *rule* it enforces (ADR-0056 BNF) is the masterplan-relevant invariant, not this instance.
- **tensions:**
  - **vs ADR-0347/0335** — the rename target collides head-on with the brand retirement; whoever executes ADR-0097's "prerequisite rename" must skip straight to the intelligence/governance prefix or do the work twice.
  - **vs ADR-0096 follow-up #3** — 0097 mandates updating 0096's prose to the new names; both now need a third rename to intelligence. Compounding churn.
  - **"grit claim --agent worker-3a"** references retired grit tooling (ADR-0116).
- **hyperscaler_challenge:** ALIGNED on enforcing a machine-checkable naming grammar (all three hyperscalers gate on lint-enforced crate/package naming). MISALIGNED in process: shipping a rename ADR for a brand that gets retired within the same sprint is exactly the thrash a hyperscaler change-management process exists to prevent. Argues strongly for AMEND (collapse the foundry→intelligence rename and the layer-last rename into ONE rename).
- **ai_slop:** Low-moderate. The BNF parse blocks are precise/useful. But the doc is a high-ceremony ADR (alternatives A/B/C, drivers, follow-ups) for what is a mechanical crate rename — ceremony-inflation. The "feedback_naming_justification.md — every new name must carry one-line BNF justification" ref borders on process-for-process.
- **refinement:** Supersede-merge the layer-last rename into the foundry→intelligence bulk rename (ADR-0347 successor) so crates are renamed ONCE to e.g. `oya-intelligence-claude-account-adapter`. Until then, mark the target names as provisional/pending-rebrand.
- **consensus_needed:** yes (shared with cluster) — "Should ADR-0097's pending crate rename be executed at all, or folded entirely into the foundry→intelligence/governance bulk rename so we don't rename twice?"

---

### ADR-0098 — Supervisor dep-policy Branch Y: zero net-new external deps + best-effort durability
- **decision_atom:** For the M02-P06 supervisor's JSONL inbox/outbox/dead-letter, adopt "Branch Y": no net-new external Cargo deps (no `rustix`, no `async_trait`), sync std::fs I/O on the tokio blocking pool, and explicitly accept best-effort durability (file `sync_all`, but NO `fsync(parent_dir)`) — re-openable only when a benchmark proves the blocking pool is the p99 bottleneck.
- **current_status:** `accepted` / published (2026-05-15).
- **disposition:** AMEND. The durability/dep-policy decision is well-reasoned and TRUE; same `foundry` brand leakage as the rest of the cluster ("the foundry supervisor", `oya-foundry-jsonl-supervisor-adapter`, `oya-foundry-supervisor-kernel`). Amend to intelligence naming; keep the policy verbatim.
- **governing:** Brand: ADR-0335/0347. Dep-governance parent: ADR-0092 (live). Decision not superseded.
- **truth_flag:** PARTIAL — policy TRUE; brand/crate-path STALE.
- **in_masterplan:** PARTIAL — phase/decider front-matter present; the explicit, audited non-durability statement is exactly the kind of binding invariant a masterplan/spec should record (it's a real production-risk acceptance), yet it is not surfaced in MASTERPLAN.md or a `/specs/*.json`.
- **tensions:**
  - **vs ADR-0335/0347** — foundry brand (cluster-wide).
  - **Internal honesty flag:** the doc admits a power-loss-invisible-file failure mode and says "not for financial ledger." This is a genuine, well-flagged risk acceptance — surface it to the founder rather than bury it, because the supervisor governs spend/Cedar enforcement (per ADR-0096), which is closer to "ledger" than the doc's "operational resilience" framing admits. Mild internal tension between 0096 ("spend recorded but audit lost" is the failure to avoid) and 0098 (accepts directory-entry loss).
- **hyperscaler_challenge:** QUESTIONABLE. AWS/Google/Azure storage/WAL teams would NOT ship a write path that can lose a freshly-created file's directory entry on power loss for anything touching audit/spend — they fsync the dirfd or use an established WAL. The "best-effort, reopen later" posture is reasonable for a pilot/staging daemon but would not pass a hyperscaler durability review for an audit-adjacent surface. Argues for AMEND with a tighter durability re-open trigger (and possibly tightening, given 0096's crash-atomicity claims).
- **ai_slop:** Low. Branches X/Y/Z analysis is substantive and quantified (64 sessions, 30 s tick, p99>50 ms reopener). The 97%-idle blocking-pool figure is back-of-envelope dressed as fact (fabricated-precision-lite) but flagged as derived, not measured.
- **refinement:** (1) Rebrand to intelligence. (2) Reconcile the durability stance with ADR-0096's "audit must be crash-atomic" claim — if Cedar/audit rows go through these same JSONL files, best-effort-no-dirfd-fsync may be too weak; state explicitly whether audit rows share this path. (3) Promote the accepted-non-durability statement into a tracked spec invariant.
- **consensus_needed:** yes — "Is best-effort durability (no `fsync(parent_dir)`, power-loss can lose the newest record) acceptable for a supervisor that also emits audit-chain + Cedar-enforcement rows, or does audit/spend force Branch X (full power-loss durability) for those files?"

---

### ADR-0099 — Cedar policy extension: foundry supervisor capabilities at T1–T4
- **decision_atom:** Gate the five supervisor capabilities (read/inject_message/idle_tick/restart_session/dead_letter) behind autonomy-tier Cedar policy (T1 read, T3 mutate, T4 destructive) in a SEPARATE policy file `docs/policies/foundry-supervisor.cedar` with a `foundry::supervisor` namespace, rather than appending to the global autonomy-ceiling seed or hardcoding the check in Rust.
- **current_status:** `accepted` / published (2026-05-15).
- **disposition:** AMEND. The tier-gating design is sound and matches the live Cedar-as-universal-gate posture (ADR-0243/0246); but (a) `foundry` brand/namespace/path is retired (ADR-0335/0347 → intelligence/governance), and (b) it is an `accepted` ADR resting on TWO `proposed` foundations (ADR-0007 and ADR-0022 both read `status: proposed` on disk) — an Accepted-on-Proposed inversion. Amend: rebrand the policy file/namespace to intelligence (e.g. `docs/policies/intelligence-supervisor.cedar`, `intelligence::supervisor`) and reconcile the parent-status inversion.
- **governing:** Brand: ADR-0335/0347. Cedar canonical posture: ADR-0243 (Cedar universal gate) + ADR-0246; parents ADR-0007/0022 (both Proposed — status-binding open).
- **truth_flag:** PARTIAL — tier-gating design TRUE and aligned with live Cedar posture; foundry namespace/path STALE; the Accepted-atop-Proposed dependency is a status-integrity defect (STALE/inconsistent front-matter).
- **in_masterplan:** PARTIAL — autonomy-tier T1–T4 IS a live masterplan-relevant concept (keystone §2: distinct from retired tenant tier-system; the live policy-autonomy-ceiling axis). The *mapping* here is plausible masterplan/spec material (belongs in `/specs/cedar-policy-schema.json`), but is not bound there and the file it specifies (`foundry-supervisor.cedar`) is created only by a future Wave 4b/Task #12 — i.e., the ADR is `accepted` for a file that does not yet exist.
- **tensions:**
  - **vs ADR-0007/0022 (both Proposed)** — accepted decision built on proposed mandates; either 0007/0022 should be promoted to Accepted or 0099 down-shifted. Status-graph inversion.
  - **vs ADR-0335/0347** — foundry namespace retirement (cluster-wide).
  - **vs Cedar canonical posture (ADR-0243/0246/0379)** — design is compatible (good), but the ADR cites only ADR-0007/0022, not the later Cedar-universal-gate ADRs that are the current authority; reference graph is stale-upstream.
  - **Autonomy-tier vs tenant-tier** — correctly uses the LIVE T1–T4 autonomy axis (not the retired tenant `tier-system` of ADR-0329); no contamination, but a reviewer must not conflate them.
- **hyperscaler_challenge:** ALIGNED. Policy-as-data in a versionable file, default-deny, capability×tier matrix, separate namespace to avoid action-name collisions — this is exactly GCP IAM / AWS IAM / Azure RBAC doctrine (Alt B "hardcode in Rust" correctly rejected; Alt A "one mega-file" correctly rejected). The design would pass a hyperscaler authz review. Argues for AMEND (rebrand + fix status inversion), not archive.
- **ai_slop:** Low. The Cedar code block is concrete; alternatives are real. Minor hedge: "Cedar's default-deny semantics apply; this comment is documentation only" is a comment-about-a-comment. The claim that the global `actuate-t4` forbid "does not suppress" the T4 supervisor permits "because namespaces are disjoint" is a load-bearing Cedar-semantics assertion that should be proven by the integration test, not asserted in prose.
- **refinement:** (1) Rebrand file/namespace foundry→intelligence. (2) Promote ADR-0007/0022 to Accepted or reconcile 0099's status. (3) Re-point references to current Cedar authority (ADR-0243/0246/0379). (4) Land the integration test that proves the global forbid does not suppress the namespaced T4 permits BEFORE accepting the disjoint-namespace claim. (5) Bind the capability×tier matrix into `/specs/cedar-policy-schema.json`.
- **consensus_needed:** yes — "Should an ADR be `accepted` while both its mandating parents (ADR-0007, ADR-0022) are still `proposed`, and is the per-capability autonomy-tier matrix the right place to encode supervisor authz given Cedar-as-universal-gate (ADR-0243) is the current authority?"

---

## Chunk notes for synthesis

**Two clean clusters, very different health:**

1. **HTTP-seam micro-ADRs (0093/0094/0095)** — uniformly KEEP/TRUE. Disciplined, test-backed, child decisions of ADR-0092. Their only systemic weakness is the corpus-wide one: **no planning front-matter / no masterplan binding** (8.8%-binding problem from planning-ssot-drift-prevention.md). They are strong *backfill material* for the masterplan IF the founder goes authored-as-SSOT, OR strong *fold-into-ADR-0092* candidates if the corpus goes generated-from-ADRs (planning-ssot-consolidation.md ADR-0000 re-founding). Under EITHER masterplan reading these three are arguably sub-clauses of 0092, not standalone ADRs — flag for the consolidation pass.

2. **M02-P06 "foundry supervisor" cluster (0096/0097/0098/0099)** — the dominant pattern of this chunk: **four `accepted` ADRs (2026-05-15) made obsolete-in-brand six days later by the 2026-05-21 foundry retirement (ADR-0335 + ADR-0347).** Every one is AMEND, not archive — the *technical decisions* (Rust-not-Node, BNF-layer-last, Branch-Y deps, Cedar tier-gating) are sound and survive; what is STALE is uniformly the `foundry` brand: crate prefix `oya-foundry-*` (→ `oya-governance-*`/intelligence per ADR-0347), capability namespace `foundry.supervisor.*`, policy path `docs/policies/foundry-supervisor.cedar`, and the "foundry supervisor" phrase. This is a textbook **retired-vocabulary-leakage cluster** (keystone §2 / MFL-0002/0003 brand-residue).

**Cross-cutting tensions / clusters for the synthesizer:**
- **Brand-retirement debt is the headline.** 0096–0099 should be reconciled as a *batch* into intelligence naming, ideally in the SAME rename that ADR-0347 governs — doing them piecemeal renames the same crates twice (0097's whole point is a rename; rebrand makes it a third). Recommend a single "foundry-supervisor cluster → intelligence-supervisor" reconciliation ticket.
- **`grit` tooling leakage** rides along (0096 "49+14 grit units", 0097 "grit claim --agent worker-3a") — grit/rtk/icm/vox were retired by ADR-0116/0054. Double-retired-vocab in the same docs.
- **Stale upstream references:** 0096 `related: ADR-0042` points at a SUPERSEDED ADR (→ ADR-0383). 0099 cites only Proposed parents (0007/0022) and misses the current Cedar authority (0243/0246/0379). Reference graphs in this cluster predate the canon they should now point at.
- **Status-graph inversion (0099):** an `accepted` ADR built on two `proposed` mandates (0007, 0022). Needs founder ruling on promote-parents vs down-shift-child. This is a corpus-integrity smell, not just a local issue — worth a grep for other Accepted-on-Proposed cases in synthesis.
- **Durability-vs-audit tension (0096 ↔ 0098):** 0096 demands crash-atomic audit+Cedar in-process; 0098 accepts power-loss directory-entry loss on the very JSONL files that may carry those audit rows. The hyperscaler verdict on 0098 is the harshest in the chunk (QUESTIONABLE) — no hyperscaler ships audit-adjacent best-effort-no-dirfd-fsync writes. Surface for founder.
- **Possible duplicate-surface with PR#605:** the rejected Node `claude-heartbeat` supervisor (0096) and the canonical agent-execution-controller (source/docs/ideas/agent-execution-controller.md, PR#605) are both agent-session-management surfaces. Flag a founder cross-check that the bespoke Rust supervisor isn't a second design for the same job now that foundry is absorbed into intelligence.
- **Autonomy-tier T1–T4 (0099) is LIVE and must not be confused** with the retired tenant `tier-system` (ADR-0329). 0099 uses the correct axis — note this for any reviewer auto-flagging "tier" as retired vocab (false positive here).
- **No LINUX-side collision in this chunk:** these are source-internal HTTP/AI-supervisor concerns; the LINUX pilot (0001–0026) has no analog. The only adjacency is Cedar (LINUX ADR-0021 owned-policy "Cedar-compatible") — 0099's Cedar usage is consistent with, not contradictory to, the LINUX owned-policy direction.
