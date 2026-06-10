# ADR Audit — SOURCE chunk 47

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** 47 (slice 323–329 of `ls -1 decisions/ADR-*.md | sort`)
- **range:** ADR-0387 → ADR-0393
- **ADRs reviewed:** 7 (0387, 0388, 0389, 0390, 0391, 0392, 0393)
- **auditor lens:** masterplan = GENERATED from the ADR log (ADRs = immutable SSOT; ADR-0364/0365 Accepted). A decision not represented as a LIVE ADR is "not needed."

---

### ADR-0387 — CI Webhook Gateway: Forgejo → Jenkins → GitHub Commit-Status Bridge

- **decision_atom:** A pure-Rust hexagonal `oya-ci-webhook-gateway-kernel` receives signed Forgejo webhooks, normalizes them to a canonical `CiTriggerEvent`, triggers the CI build, and posts the required commit-status contexts back, eliminating the founder-OK admin-relax-merge bridge.
- **domain:** ci-cd-build (cross-cutting: forge-vcs)
- **current_status:** Proposed.
- **disposition:** AMEND. The trigger-bridge decision is sound and still needed, but the ADR is laced with retired/transitory vocab: it hard-binds the *destination* CI engine to **Jenkins** (`oyaCiLane`, `JenkinsClient`, "trigger the Jenkins parameterized job") which ADR-0511 demotes to transitory bootstrap (Argo Workflows is the destination), and its sink target is internally contradictory — D5/Decision say post to **GitHub** (`gh api .../statuses/<sha>`) while the title and binding ADR-0363 say the substrate is **Forgejo Commit Status**. Amend to (a) abstract the trigger target behind the `JenkinsClient`-shaped seam renamed to a CI-engine-agnostic port (Argo Workflows per ADR-0511), and (b) reconcile the Forgejo-vs-GitHub status sink against the live forge fault-line (§5).
- **proposed_resolution:** RATIFY (as AMEND). The trigger-bridge is the genuine missing piece that removes the admin-merge antipattern; the decision atom is real and masterplan-worthy. Ratify the *intent*; the Jenkins/GitHub coupling is an implementation detail to be re-pointed, not a reason to drop.
- **governing:** n/a (not archived). Watch ADR-0511 (CI destination) + ADR-0363/0510 (forge sink) as governing context for the amendment.
- **truth_flag:** PARTIAL. The bridge concept is TRUE; the Jenkins-as-destination and GitHub-vs-Forgejo-sink specifics are STALE/contradictory.
- **in_masterplan:** PARTIAL. The "PRs auto-merge once required contexts post green; no admin bypass" rule belongs in the masterplan; the Jenkins-specific wiring does not.
- **tensions:** (1) Jenkins coupling vs ADR-0511 (Argo Workflows destination; Jenkins transitory). (2) GitHub `gh api .../statuses` sink (D5) vs Forgejo Commit Status substrate (title + ADR-0363) — internal contradiction, also collides with the three-way forge fault-line (founder GitHub directive vs Forgejo-transitory vs bespoke-VCS destination, §5). (3) cites ADR-0112 (webhook-driven Foundry invocation) as binding, but ADR-0112 is `Superseded` by ADR-0363 per the keystone map §1.1 — citing a superseded ADR as a live binding authority.
- **hyperscaler_challenge:** ALIGNED in shape, QUESTIONABLE in coupling. Google/AWS/Azure all run signed-webhook → CI-trigger → status-back bridges (fail-closed HMAC/ed25519 before parse is correct hyperscaler practice). They would NOT pin the destination to Jenkins, and would not split the status sink ambiguously across two forges. Implication: amend to engine-agnostic + single canonical sink.
- **ai_slop:** Low. The deliverable spec is concrete and TDD-staged (RED/GREEN), not padding. The slop risk is *citation rot* (binding a superseded ADR-0112) and *retired-vocab leakage* (Jenkins-as-destination), not fabrication.
- **refinement:** Rename `JenkinsClient`/`oyaCiLane` seam to a CI-engine port; pick one status sink (Forgejo Commit Status per the transitory canon, or GitHub per founder directive) and delete the other; replace the ADR-0112 binding citation with ADR-0363.
- **consensus_needed:** Does the commit-status sink post to **Forgejo** (transitory canonical host) or **GitHub** (founder migration directive)? This single ADR contains both and cannot ship until the forge fault-line (§5) is resolved.

---

### ADR-0388 — Doc-axis convention to prevent doc sprawl

- **decision_atom:** All documentation must live on one of seven canonical axes (DECISIONS/PLANS/INDEX/SPECS-MS/SPECS-CRATE/RUNBOOKS/IPS) plus a transient IDEAS axis with a 14-day promote-or-archive timer; everything off-axis is a `no-shadow-docs` gate violation.
- **domain:** docs-ssot-masterplan (cross-cutting: governance-process)
- **current_status:** Accepted.
- **disposition:** KEEP. This is a clean, live, enforceable governance decision with a named gate (`oya gate validate doc-axis`). It is exactly the kind of process invariant the masterplan should encode.
- **proposed_resolution:** n/a (Accepted, not Proposed).
- **governing:** n/a.
- **truth_flag:** TRUE — with one internal nit: the prose says "enforces all **four** rules" while the doc defines more than four (axes + ideas timer + shadow-doc + status-casing + catalog/manifest consistency). Minor copy drift, not a correctness fault.
- **in_masterplan:** YES. The doc taxonomy + the `planning_impact: true ⇒ masterplan` rule is foundational to the generated-masterplan model (it literally names PLANS = `masterplan.generated.json`, auto-gen from ADRs — directly supporting the ADRs-generate-masterplan reading the keystone §4 flags as OPEN).
- **tensions:** Strongly REINFORCES the "masterplan GENERATED from ADRs" side of the open founder question (PLANS axis = `masterplan.generated.json`, "Derived from ADRs with `planning_impact: true`. Never hand-edit."). This is in mild tension with `planning-ssot-drift-prevention.md`'s "masterplan IS authority, ADRs bind in" direction. Also: the Notes block references promoting idea-pagers to ADR-0389/0390/0391 — self-consistent with this chunk (those three are the promoted pagers).
- **hyperscaler_challenge:** ALIGNED. Google (monorepo doc conventions + readability review), AWS, and Azure all enforce closed doc taxonomies + lint gates to kill shadow docs. A 14-day promote-or-archive timer on ideas is a reasonable forcing function. They would make this decision.
- **ai_slop:** Low. Concrete table, named gate, decision tree. The "four rules" miscount is the only slop signal.
- **refinement:** Fix the "four rules" count; the trailing Notes paragraph (lines 107–110) is a half-dangling sentence fragment ("(and any sibling idea-pagers) to formal ADRs…") that should be cleaned.
- **consensus_needed:** None on the decision itself. But this ADR is a *vote* for generated-masterplan — surface it as evidence when the founder resolves the authored-vs-generated question.

---

### ADR-0389 — cloud-intelligence: Bedrock-on-Talos pattern as a cloud primitive

- **decision_atom:** Position `oya-invoke` as a provider-agnostic capability port fronting a self-hosted, AWS-Bedrock-Converse-compatible inference surface on Talos, delivered in phases (v1 passthrough → v1.5 SSE → v2 Bedrock-compat → v3+ failover), so internal workloads call one gateway instead of provider SDKs.
- **domain:** intelligence-ai (cross-cutting: api-contracts)
- **current_status:** Accepted.
- **disposition:** KEEP. Correct canonical-vocab ("cloud-intelligence," not the retired "foundry"/"llm-gateway" brand), correctly Talos-based (ADR-0375/0378), phased and data-gated. The Bedrock-Converse-as-self-hosted-primitive bet is a deliberate, coherent positioning decision.
- **proposed_resolution:** n/a (Accepted).
- **governing:** n/a.
- **truth_flag:** TRUE. One staleness watch: the v1 surface is "Accepted" but most of the substance (Bedrock-compat) is deferred to v2/v3+ and gated on "≥5 tenants" / "≥1 tenant validated" — i.e. this is largely an *aspirational* accepted decision, true as doctrine but unproven in fact.
- **in_masterplan:** PARTIAL. The capability-port doctrine (`oya-invoke` = the abstraction boundary; no direct provider SDK imports) belongs in the masterplan; the v2/v3 Bedrock-compat specifics are forward-looking and should be marked deferred.
- **tensions:** Mild — overlaps heavily with ADR-0384 (OAuth-pool kernel) and ADR-0390 (v1 pipeline); the three form a tight cluster and could MERGE-by-reference, but each has a distinct decision atom (positioning vs kernel vs pipeline), so keep separate. Watch: references ADR-0384 as the kernel — confirmed live on disk (`ADR-0384-llm-gateway-oauth-subscription-pool-redesign.md`), but ADR-0384's filename still carries the retired `llm-gateway` brand (rename sweep is ADR-0390 Lane N, pending).
- **hyperscaler_challenge:** ALIGNED. This IS the hyperscaler move — AWS Bedrock Converse is the literal reference; building a self-hosted Converse-compatible surface is "be your own Bedrock." Google (Vertex unified API) and Azure (Azure AI model router) made the same provider-abstraction bet. Cleanly aligned; no archive implication.
- **ai_slop:** Low-moderate. The ASCII diagrams and four-filter "hyperscaler lens" recital read slightly templated, but the phased table + open-issues are concrete. No fabricated facts.
- **refinement:** Add an explicit "status: Accepted-as-doctrine, v2+ surface deferred/unproven" qualifier so the masterplan generator doesn't promote unbuilt Bedrock surfaces as delivered.
- **consensus_needed:** None contested. (Minor: confirm `oya-invoke` call convention = plain HTTP client — already leaned-accepted in the open issues.)

---

### ADR-0390 — cloud-intelligence v1: request pipeline and proof layer

- **decision_atom:** The cloud-intelligence v1 service is the 8-stage P0–P7 request pipeline (ingress → Cedar authz → OAuth-pool lease → provider call → three-tier receipt → window/state → egress → audit-chain) with a first-class orthogonal proof layer (Loom + proptest + chaos), built via six disjoint-path lanes K/R/Z/A/C/N.
- **domain:** intelligence-ai (cross-cutting: observability)
- **current_status:** Accepted.
- **disposition:** KEEP. The most concrete, implementation-grade ADR in the chunk: explicit concurrency primitives, named metrics, proof properties, and lane fanout. Correctly uses Cedar (ADR-0243/0246), Valkey (not Redis — ADR-0336 compliant), ClickHouse, Sigstore. Vocab is clean.
- **proposed_resolution:** n/a (Accepted).
- **governing:** n/a.
- **truth_flag:** TRUE. Highly specific and internally consistent; the proof-layer-as-deliverable framing is a genuine engineering commitment, not slop.
- **in_masterplan:** YES (as the cloud-intelligence v1 build plan). The pipeline-stage contract + proof-layer requirement are masterplan-worthy; the per-metric names are spec-level detail that should live in the service manifest, not the masterplan body.
- **tensions:** (1) The Lane N rename sweep (`llm-gateway` → `cloud-intelligence`) is declared but the dependency ADR-0384's filename still says `llm-gateway` — rename not yet executed corpus-wide (retired-brand residue). (2) Tight coupling to ADR-0391 (lane-overlap gate) and ADR-0384 (kernel) — a healthy cluster, not a conflict. (3) Valkey + ClickHouse + S3 + Sigstore is a broad substrate surface; consistent with SOURCE best-of-breed posture (keystone §3) but worth noting against LINUX's own-the-substrate posture (§5 fault-line 1).
- **hyperscaler_challenge:** ALIGNED. A staged inference proxy with per-tenant seat pools, idempotency keys (Stripe-style), Merkle audit chain, and Sigstore attestation is exactly how a hyperscaler would build a managed-inference control plane. The Loom/proptest/chaos proof obligation exceeds typical OSS rigor — *more* disciplined than the hyperscaler baseline, not less. Aligned.
- **ai_slop:** Low. This is the gold-standard ADR of the chunk for concreteness. The only slop-adjacent risk is the sheer density of forward-looking validation claims in Open Issues, all correctly flagged as unvalidated.
- **refinement:** Move per-metric/per-stage minutiae to the service manifest (SPECS-MS axis per ADR-0388); keep the ADR as the stage-contract + proof-obligation decision.
- **consensus_needed:** None contested.

---

### ADR-0391 — N-lane parallel safety proof and unified DevOps console

- **decision_atom:** Two decisions: (a) a `oya gate validate lane-overlap` gate proves disjoint file paths across concurrently-enqueued parallel-agent PRs (∀ i≠j, file_set(lane_i) ∩ file_set(lane_j) = ∅), and (b) a read-only operator DevOps console v0 aggregates subscription/lane/proof/audit/gateway state from existing APIs with no new data storage.
- **domain:** ci-cd-build (lane-overlap gate) + observability (DevOps console) — genuinely cross-cutting two domains.
- **current_status:** Accepted.
- **disposition:** AMEND. The lane-overlap gate is a clean KEEP. The DevOps console v0 is sound but carries stale/retired substrate references that need amendment: it aggregates from **Jenkins API** (`/api/proof`, "Jenkins build history") — Jenkins is transitory per ADR-0511 — and its stack is **SolidJS SPA** (Part B "Stack: SolidJS SPA served by an Axum static-file backend"), which ADR-0393 (this same chunk) retires SolidJS as a canonical frontend target in favor of Leptos. The console's frontend choice is now stale on arrival.
- **proposed_resolution:** n/a (Accepted).
- **governing:** Frontend stack governed by ADR-0393 (Leptos canonical, SolidJS retired). CI source governed by ADR-0511 (Argo Workflows; Jenkins transitory).
- **truth_flag:** PARTIAL. Lane-overlap gate = TRUE. DevOps console architecture = STALE on two axes (SolidJS frontend retired by ADR-0393; Jenkins-API source demoted by ADR-0511).
- **in_masterplan:** PARTIAL. The disjoint-path invariant for parallel-agent merge batches is a core masterplan safety rule (it underwrites the whole N-lane swarm model). The console v0 is an operator tool — masterplan-relevant as "there is one operator console," but its SolidJS/Jenkins specifics must not be promoted.
- **tensions:** (1) **Self-contradiction within the chunk:** ADR-0391 specifies a SolidJS DevOps console; ADR-0393 (same date-cluster, accepted later) retires SolidJS canonically. The console must be re-specified as Leptos. (2) Jenkins-API data source vs ADR-0511 Argo-Workflows destination. (3) lane-overlap gate "runs as part of `oya verify --ci-required` in the Jenkins pipeline" — Jenkins coupling again. (4) reads ADR-0111 merge-queue projected state for batch boundary — ADR-0110 (changeset state machine) is Superseded by ADR-0363 per keystone §1.1; confirm ADR-0111 is still live (it is referenced as current).
- **hyperscaler_challenge:** ALIGNED. A disjoint-path collision gate over a parallel-merge batch is exactly the kind of structural invariant Google's monorepo tooling enforces (no two concurrent changes silently touch the same file without merge-queue serialization). A single-pane operator console aggregating from source-of-truth APIs (no new store) is standard hyperscaler ops practice. The decision class is aligned; only the named substrates (SolidJS/Jenkins) are stale.
- **ai_slop:** Low. The gate algorithm is precise; the console panel spec is concrete. Slop signal = substrate references that were already retired by sibling ADRs (frontend/CI), i.e. intra-batch incoherence rather than fabrication.
- **refinement:** Re-point console frontend SolidJS → Leptos (ADR-0393); re-point `/api/proof` and the gate host from Jenkins → Argo Workflows / `oya-ci` (ADR-0511/0513); confirm ADR-0111 liveness.
- **consensus_needed:** None on the gate. For the console: confirm Leptos (per ADR-0393) is now its mandated frontend — should be auto-resolved by ADR-0393's supersession of SolidJS, but the console ADR was authored against the pre-supersession assumption.

---

### ADR-0392 — Buck2 canonical build graph (reverses ADR-0358 §2 Bazel rules_rust)

- **decision_atom:** Buck2 (Rust binary) + `buck2-prelude` first-party Rust rules + Reindeer-buckified checked-in `third-party/rust/BUCK` (Cargo.toml/lock remain the human SSOT) on a self-hosted NativeLink RBE is the canonical build graph, deliberately reversing ADR-0358 §2's Bazel `rules_rust` choice.
- **domain:** ci-cd-build (cross-cutting: forge-vcs / toolchain)
- **current_status:** Proposed (explicitly "DRAFT for founder review; must NOT auto-merge").
- **disposition:** KEEP (as the governing reversal) — but see truth_flag drift. This is the current canonical build-graph decision per keystone §3 ("Buck2 = build/RBE") and §2 retired-vocab ("Bazel/rules_rust → Buck2"). It correctly supersedes ADR-0358 §2 only, honestly confronts the prior objection, and makes NO unproven speedup claims (0% adopted, explicitly).
- **proposed_resolution:** RATIFY. The keystone map already treats Buck2 as the live canonical posture and ADR-0358 carries `superseded_by: [ADR-0392, ADR-0408]` on disk — i.e. the corpus already behaves as if this is accepted. The "Proposed/must-not-auto-merge" status is a founder-review gate, not genuine indecision. Ratify to Accepted on founder sign-off; the decision is doctrinally settled. Why: a build-graph reversal cannot sit Proposed indefinitely while downstream specs and the keystone treat it as TRUE.
- **governing:** This ADR is itself governing for the build graph; it governs (supersedes) ADR-0358 §2. Sibling ADR-0408 governs the CI/CD half.
- **truth_flag:** TRUE (as canonical posture) with a SUPERSESSION-SCOPE drift to flag: the front-matter `supersedes: [ADR-0358]` (whole-ADR) is broader than the body's explicit "§2 only — §1/§3/§4 of ADR-0358 stand." ADR-0358's own front-matter agrees it is only §2-reversed (`amendment_note`), so the bare `supersedes: [ADR-0358]` array is imprecise and should read as a partial/§2 supersession. Auditors must not infer ADR-0358 is fully dead.
- **in_masterplan:** YES (canonical build graph). The masterplan generator must reflect Buck2 (not Bazel) — and the superseded Bazel inputs in `specs/cloud-toolchain-target.json` / `specs/masterplan.json` (P-TOOLCHAIN ~L5860-5960, including the literal `"rejected": "Buck2..."`) are stale and require a follow-up generated-artifact regen (the ADR explicitly defers that spec rewrite out of scope — a known drift the masterplan generator must close).
- **tensions:** (1) Partial-supersession imprecision (front-matter vs body, above). (2) Machine-readable specs still encode Bazel as canonical AND still encode the now-reversed `"rejected": Buck2` rationale — the specs literally contradict the governing ADR until regenerated (the sharpest live spec-vs-ADR drift in the chunk). (3) Non-contiguous numbering: ADR-0392 forward-allocated by founder convention, leaving ADR-0377–0391 gap open (keystone §6.3) — index-poisoning but declared. (4) Buck2 = 0% adopted; doctrine-only, so it is TRUE-as-decision but the *implementation* is entirely future.
- **hyperscaler_challenge:** ALIGNED (and on-brand for the bespoke-Rust ambition). Buck2 is literally Meta's production monorepo build system; NativeLink (Bazel REv2 API, Apache-2, self-hostable) is the right RBE choice under the OSI-strict + self-hostable hyperscaler lens. Google would use Blaze/Bazel; Meta uses Buck2 — choosing Buck2 aligns with a Rust-native, own-the-toolchain hyperscaler. The reversal is defensible; the only honest caveat is that ADR-0358's "less OSS-battle-tested" objection is real and acknowledged. No archive implication; this is the aligned destination.
- **ai_slop:** Very low. This is a model ADR for honesty: it quotes the prior objection verbatim, confronts it, accepts the tradeoff with open eyes, and explicitly blocks all numeric claims pending green evidence. Anti-slop exemplar.
- **refinement:** Tighten `supersedes:` to express §2-only/partial supersession of ADR-0358; schedule the deferred `specs/*.json` regen (Bazel→Buck2 + delete the stale `"rejected": Buck2` line) as a tracked follow-up so the generated masterplan doesn't re-emit Bazel.
- **consensus_needed:** Should ADR-0392 be promoted Proposed → Accepted now (the keystone + ADR-0358 front-matter already treat it as governing), or does the founder want it to remain a held review-gate? It cannot stay half-accepted while specs and the keystone diverge.

---

### ADR-0393 — Leptos canonical app-shell frontend (Rust/WASM SSR+hydration; supersedes ADR-0372 SolidJS)

- **decision_atom:** Leptos (full-stack Rust/WASM, SSR + hydration) is the single canonical app-shell/portal-shell frontend; the existing SolidJS app-shell is frozen/retired, the Leptos prototype is promoted to production, and Rust→WASM compute islands + the `render_envelope` SSR contract survive — fully superseding ADR-0372's SolidJS choice.
- **domain:** product-ux (frontend) (cross-cutting: api-contracts via `render_envelope` SSR contract)
- **current_status:** Accepted (2026-06-01 founder-confirmed; originally drafted Proposed 2026-05-29).
- **disposition:** KEEP. Clean, honest supersession: it states the headline decision (Leptos canonical), records the SolidJS→Leptos reversal, accounts for live-code drift transparently, and chose a fresh superseding ADR over a third confusing amendment. Bidirectional markers landed (verified: ADR-0372 on disk = `status: Superseded, superseded_by: [ADR-0393]`).
- **proposed_resolution:** n/a (now Accepted; was Proposed, correctly promoted by founder confirmation rather than auto-merge).
- **governing:** This ADR governs; it supersedes ADR-0372 (SolidJS). Forward-binds ADR-0394 (IDP central hub built on this shell) and flags ADR-0513 (`oya-ci-deck`) to flip to Leptos.
- **truth_flag:** TRUE. Front-matter, body, and the ADR-0372 counterpart are all consistent. The honest "the code drifted to SolidJS, the decision is Leptos" accounting is exactly right.
- **in_masterplan:** YES. "One canonical frontend = Leptos/Rust-WASM; SolidJS retired" is a load-bearing masterplan fact (it also removes the Node/pnpm toolchain from the frontend path — consistent with ADR-0394's Node/React-forbidden stance and the one-Rust-toolchain doctrine).
- **tensions:** (1) **Intra-chunk conflict (now resolved-in-principle):** ADR-0391 (same date-cluster) specifies a **SolidJS** DevOps console; ADR-0393 retires SolidJS — so ADR-0391's frontend is stale and must be re-pointed to Leptos (flagged under ADR-0391). (2) Keystone §1.1 lists ADR-0372 as "Superseded (see ADR-0372 body)" without naming the superseder; this chunk confirms the superseder is **ADR-0393** — a precision the keystone map can absorb. (3) Implementation drift is explicitly deferred to follow-up code PRs (SolidJS tree not yet removed; Leptos prototype not yet de-mocked) — decision is TRUE, execution pending. (4) Non-contiguous numbering (ADR-0393/0394 forward-allocated to dodge the Buck2 ADR-0392 collision) — declared, index-poisoning per keystone §6.3.
- **hyperscaler_challenge:** QUESTIONABLE (deliberately, and honestly owned). No hyperscaler ships its primary *consumer* surface as Rust/WASM SSR — TS/React/Solid dominate for TTI + talent reasons, which the ADR concedes ("Rust-frontend talent is scarcer… accepted as a deliberate doctrine cost," "WASM cold-start floor returns"). BUT the scope is an *internal operator console / IDP hub*, not a massive-scale consumer surface, where the one-toolchain + dogfooding value can outweigh TTI — that narrowing makes it defensible. Implication: KEEP for the internal-shell scope; do NOT generalize Leptos-canonical to a future high-scale consumer frontend without a fresh decision. This is the one "own-it over best-of-breed" bet in the chunk where the hyperscaler default would differ.
- **ai_slop:** Very low. Exemplary honesty: it inventories the exact drifted files, explains why a supersede beats a third amendment, and lists rejected alternatives crisply. Anti-slop exemplar alongside ADR-0392.
- **refinement:** Execute the tracked follow-ups (quarantine SolidJS tree, de-mock the Leptos prototype, add the superseded-reference lint so SolidJS can't reappear, register the `axum` justifications); cascade the Leptos decision into ADR-0391's console and ADR-0513's `oya-ci-deck`.
- **consensus_needed:** Confirmed (founder-confirmed 2026-06-01). The only open founder question is scope-creep guard: does Leptos-canonical bind ONLY internal operator/IDP shells, or also a future consumer-scale frontend? (The ADR implies internal-only via the "massive-scale" carve-out language.)

---

## Chunk notes

**Coherence of the chunk.** This is a high-quality, mostly-recent (2026-05-28/29) cluster authored under the post-0335/0388 discipline. Four of seven (0388, 0389, 0390, 0393) are clean KEEPs; ADR-0392 is a KEEP-as-governing with a supersession-scope nit; ADR-0387 and ADR-0391 are AMENDs driven by *transitory-substrate coupling* (Jenkins) and *intra-batch staleness* (SolidJS), not by being wrong. No GARBAGE, no fabrication, no retired-brand ("foundry"/"tier"/"Redis"/"Kafka"/"M0-M3") leakage — vocab discipline is strong (cloud-intelligence used correctly; Valkey not Redis; Talos correct).

**The dominant cross-cutting drift = transitory-substrate references.** Three ADRs (0387, 0391, and 0390's console wiring) hard-reference **Jenkins** as if it were the destination CI engine. Per ADR-0511 (keystone §1.3/§3) Jenkins is transitory bootstrap; Argo Workflows + bespoke `oya-ci` is the destination. Every Jenkins coupling in this chunk should be read as transitory and re-pointed when the CI-engine port is abstracted. This is consistent corpus-wide drift, not a per-ADR error.

**The sharpest intra-chunk contradiction:** ADR-0391 mandates a **SolidJS** DevOps console while ADR-0393 (same author-cluster, same week) retires SolidJS as a canonical frontend. ADR-0391 was authored against the pre-supersession assumption; the masterplan generator must apply ADR-0393's supersession transitively so the console is generated as Leptos, not SolidJS.

**The sharpest spec-vs-ADR drift:** ADR-0392 reverses Bazel→Buck2 but the machine-readable specs (`cloud-toolchain-target.json`, `masterplan.json` P-TOOLCHAIN) still encode Bazel as canonical AND still carry a literal `"rejected": "Buck2..."` line — the specs actively contradict the governing ADR until regenerated. Under the ADRs-generate-masterplan model (ADR-0388 PLANS axis), the ADR wins and the specs are stale inputs awaiting regen; this is a concrete, named backfill task for the masterplan generator.

**Supersession-graph corrections/confirmations for the keystone map:**
- ADR-0372 (SolidJS) is superseded specifically by **ADR-0393** (keystone §1.1 left the superseder unnamed) — confirmed bidirectional on disk.
- ADR-0392's `supersedes: [ADR-0358]` is **§2-only/partial** (body + ADR-0358's `amendment_note` agree); the bare front-matter array overstates it. ADR-0358 remains `Proposed` with §1/§3/§4 in force.
- ADR-0387 cites **ADR-0112 as a live binding authority**, but ADR-0112 is Superseded by ADR-0363 (keystone §1.1) — a stale binding citation to fix on amendment.

**Masterplan authored-vs-generated signal (keystone §4, OPEN):** ADR-0388 is a strong data point for the **generated-from-ADRs** reading — it defines PLANS = `docs/machine-readable/masterplan.generated.json`, "Derived from ADRs with `planning_impact: true`. Never hand-edit," auto-genned by `oya gen masterplan`. All six planning-impact ADRs in this chunk carry `planning_impact: true`, consistent with feeding a generator. Surface this when the founder resolves the direction; this chunk leans generated.

**Numbering hygiene:** ADR-0392 and ADR-0393 are both declared non-contiguous forward-allocations (0392 for the Buck2 reversal; 0393/0394 to dodge the 0392 collision), leaving ADR-0377–0391 and 0395–0407 gaps open. Declared, not collisions, but they poison any flat `ADR-NNNN` index and reinforce keystone §6.3's "re-derive `decisions.json next_adr` from disk, never trust face value."

**Net dispositions:** KEEP ×4 (0388, 0389, 0390, 0393), KEEP-as-governing-with-nit ×1 (0392, ratify Proposed→Accepted), AMEND ×2 (0387 Jenkins/forge-sink coupling + stale ADR-0112 binding; 0391 SolidJS→Leptos + Jenkins→Argo re-point). ARCHIVE ×0. GARBAGE ×0.
