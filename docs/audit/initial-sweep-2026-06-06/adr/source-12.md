# ADR Audit — SOURCE, Chunk 12

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 12
- **Slice (map lines 78–84 / `ls | sed -n 78,84p`):** ADR-0100 … ADR-0106
- **ADRs actually reviewed (7):** ADR-0100, ADR-0101, ADR-0102, ADR-0103, ADR-0104, ADR-0105, ADR-0106
- **Cross-checks performed:** ADR-0107 (Superseded-by-0105, on disk), ADR-0056 (Accepted, the BNF being amended), ADR-0052 (Superseded-by-0118, the renumber-source of 0103), MASTERPLAN.md grep (0 hits for `usecase`/`foundry-supervisor`/`13-layer`).

> **Cluster identity.** This chunk is the **"foundry-supervisor bring-up + crate-naming doctrine" cluster**, all dated 2026-05-14/15, all authored by `council-architecture`. Two sub-clusters:
> 1. **Foundry-supervisor implementation ADRs** (0100, 0101, 0102) — saturated with the **RETIRED `foundry` brand** (ADR-0335/0347; founder: "cloud-intelligence is the valid name"). The *decisions* are sound micro-architecture; the *naming* is dead vocabulary.
> 2. **Crate-naming / layer-enum doctrine** (0104, 0105, 0106) + **agentic-VCS cutover inventory** (0103). 0105/0106 are the live canonical layer-enum authority (13 values, `usecase` not `application`); 0103 sits on the retired `grit`/agentic-VCS substrate.

---

### ADR-0100 — Foundry Supervisor Public Contract (Lean-a10)

- **decision_atom:** A supervisor that composes existing kernels (RoutePolicy/UsageEnforcement/Billing/AutonomyCeiling) adds **zero new public API to those kernels**; all supervisor-specific types and the `AccountSnapshotProvider` port live in a new dedicated supervisor-kernel, composed purely via ports.
- **current_status:** Accepted (`doc_status: published`).
- **disposition:** AMEND.
- **governing:** Brand governed by ADR-0335 (foundry-µsvc retired → absorbed by intelligence) + ADR-0347 (foundry→governance rename). The *zero-surface-change / port-composition* principle is sound and survives; only the `foundry-supervisor`/`oya-foundry-*` naming must be reconciled to `intelligence` (the supervisor is a consumer-AI orchestration surface) or `governance` if it is a CI/gate surface.
- **truth_flag:** PARTIAL — the architectural decision is TRUE; the `foundry` naming is STALE (retired vocab).
- **in_masterplan:** NO — no `planning_impact` front-matter; 0 MASTERPLAN hits for `foundry-supervisor`. Pure implementation-tier ADR.
- **tensions:** (a) Retired-vocab vs ADR-0335/0347 GLOSSARY "Foundry (RETIRED)". (b) Depends on ADR-0056 "Port-in-Kernel" — fine, 0056 is Accepted. (c) "AutonomyCeiling" kernel here is the LIVE autonomy-tier T1–T4 axis (NOT the retired tenant tier-system of ADR-0329) — keep distinct.
- **hyperscaler_challenge:** **Aligned** as a principle. "New composite surface adds no public API to stable primitives; compose via narrow ports" is exactly how AWS/Google guard published service contracts (additive-only, snapshot-verified). The `cargo public-api` byte-identical gate is a real hyperscaler discipline. Argues for **amend** (naming only), not archive.
- **ai_slop:** Low. "Existing kernels remain byte-identical (verified via `cargo public-api` snapshots)" is concrete, not fabricated. Minor: "Lean-a10 (Zero-Surface-Change)" is an internal codename with no external referent — mild private-jargon hedging.
- **refinement:** Rename `oya-foundry-supervisor-kernel` → `oya-intelligence-supervisor-kernel` (or governance, per surface). Add a `superseded_by`/`amends` link to ADR-0335 so the brand retirement is traceable from the ADR itself.
- **consensus_needed:** no (naming reconciliation is mechanical once the foundry→intelligence/governance split is applied per existing ADR-0335/0347).

---

### ADR-0101 — Foundry Supervisor Mountpoint (Direct Hyper)

- **decision_atom:** The supervisor mounts its webhook/health surface **directly on the Hyper/Tokio HTTP runtime adapter**, bypassing the stubbed `api-rest-adapter`, as an explicitly temporary expedient until a real REST router lands.
- **current_status:** Accepted.
- **disposition:** AMEND (candidate ARCHIVE-when-resolved). It is a self-declared **temporary** decision ("the supervisor may be migrated once M02-P04 lands a real router"). Once that router exists it should be archived as completed-or-reversed.
- **governing:** Brand: ADR-0335/0347 (foundry retired). Milestone ref `M02-P04` rides the RETIRED `M0/M1/M2` milestone vocabulary (GLOSSARY L250/L504 retired → descriptive Wave names) — stale ref.
- **truth_flag:** PARTIAL — true-as-tactical-expedient; both the `foundry` brand and the `M02-P04` milestone token are STALE.
- **in_masterplan:** NO — implementation-tier, no planning front-matter.
- **tensions:** (a) Self-tension with ADR-0105's own posture that bypasses/temporary exceptions must carry an explicit unblock trigger — here the trigger is named but tied to a retired milestone id. (b) "Direct Hyper bypasses the standard REST adapter" is a temporary parallel path — soft brush against FO-01 "no parallel canonical trees" (acceptable because explicitly transitory).
- **hyperscaler_challenge:** **Questionable.** Hyperscalers tolerate a direct-runtime mount for a high-cadence internal tick path, but would not leave "bypass the standard adapter" as an open-ended Accepted decision without a dated cutover. The performance rationale (lower overhead for high-cadence ticks) is plausible but unquantified. Argues for **amend**: convert to a tracked, dated migration item or archive on completion.
- **ai_slop:** Low-moderate. "Direct Hyper mounting is lower overhead" is **fabricated precision** (no benchmark cited) — exactly the DP-09 "bench before claiming performance" failure that ADR-0104 itself polices. Internal inconsistency worth flagging.
- **refinement:** Replace `M02-P04` with a Wave-name + numeric trigger; add a measured latency/overhead figure or drop the performance claim; set a removal date like ADR-0107 does.
- **consensus_needed:** no.

---

### ADR-0102 — Foundry Settings Template Canonical Rendering

- **decision_atom:** Per-provider account settings (hooks/skills/MCP servers) are normalized through one canonical `SettingsTemplate` value type with per-provider renderers, rendered atomically (tempfile+rename), with `sref://` secret references resolved at render/spawn time and drift verified at snapshot time.
- **current_status:** Accepted.
- **disposition:** AMEND (naming only).
- **governing:** Brand: ADR-0335/0347 (foundry retired → intelligence). The decision itself is non-conflicting and well-formed.
- **truth_flag:** PARTIAL — decision TRUE; `oya-foundry-settings-template-*` naming STALE.
- **in_masterplan:** NO — implementation-tier, no planning front-matter.
- **tensions:** (a) Retired-vocab only. (b) `SecretStorePort` + `sref://` should reconcile with the canonical secret substrate (OpenBao/`oya-foundry-account-adapter-inmemory` rename references OpenBaoAdapter per ADR-0104) — verify the secret-store port name matches the canonical identity/secrets posture (ADR-0187–0190 cluster), not a foundry-local invention.
- **hyperscaler_challenge:** **Aligned.** Atomic tempfile+rename rendering, late-bound secret resolution, and snapshot-time drift detection are textbook config-management hygiene (cf. AWS AppConfig / Google's config-as-data). Solid. Argues at most for **amend** (naming).
- **ai_slop:** None material. Crisp and concrete.
- **refinement:** Rename to `oya-intelligence-settings-template-*`. Confirm `SecretStorePort` is the canonical secrets port, not a parallel one. Cite the canonical secret-substrate ADR.
- **consensus_needed:** no.

---

### ADR-0103 — Grit cutover inventory of legacy primitives

- **decision_atom:** Adopt a single canonical inventory mapping each legacy agent repo-access primitive (direct `git`/`gh`, hand-rolled locks, aggregate check commands, markdown plans) to a sanctioned replacement, with banned-primitives CI enforcement and an explicit, ADR-anchored compatibility window.
- **current_status:** Accepted; `supersedes:[] / superseded_by:[]`; `renumbered_from: ADR-0052`.
- **disposition:** ARCHIVE (superseded-in-fact / substrate retired).
- **governing:** **ADR-0116** retires the external coord tooling (grit/rtk/icm/vox) — the *replacements* this inventory sanctions (`grit claim/done`, `icm` path) are themselves retired. **ADR-0363** retires the agentic-VCS / controller-owned merge-queue substrate (M01-P07 IP-007 referenced here) → plain git + Forgejo PRs + Prow-shaped cloud-ci. Its renumber-parent **ADR-0052 is `Superseded by ADR-0118`** on disk. The "banned direct-git" *principle* survives (plain-git + forge-PR discipline is the current truth); the *grit/icm sanctioned-trio* does not.
- **truth_flag:** STALE — the inventory's replacement column ("`grit`", "controller-owned merge queue", "ICM scaffold-claim") names retired tooling; the table is a historical cutover record, not current doctrine.
- **in_masterplan:** NO. (MASTERPLAN does encode "promotion is plain-git branch → PR" — the surviving principle — but this ADR's grit/icm mechanism is not in the masterplan and is retired.)
- **tensions:** (a) **Sharp** — sanctions `grit`/`icm` which ADR-0116 retires. (b) References "Controller-owned merge queue (M01-P07 IP-007)" — retired by ADR-0363 (no bespoke merge-queue; Forgejo PRs). (c) `oya-governance-banned-primitives-kernel` is the live enforcement name (good — already uses governance prefix, post-0347). (d) Front-matter drift: carries `superseded_by:[]` while its whole substrate is retired (mirrors the ADR-0136 stale-front-matter pattern called out in map §1.3/§5.6).
- **hyperscaler_challenge:** **Aligned in principle, misaligned in mechanism.** Google/Meta absolutely ban direct VCS reach from automation and route everything through a controlled CLI + merge-automation — that principle is correct. But they would NOT enshrine a third-party tool brand (`grit`/`icm`) as the sanctioned primitive; they own the CLI. Argues for **archive** of this specific inventory, retaining the banned-primitives *lane* as the durable artifact.
- **ai_slop:** Low. The "Linus good-taste row" section is mild rhetorical garnish (founder-style framing) but each bullet is a real, testable rule, not filler. The table is concrete.
- **refinement:** Archive as historical; fold the surviving "banned-primitives + mandatory-replacement-column + explicit-compat-window" rules into the current governance/forge ADRs (0363/0116/0118 lineage). Re-point the merge-queue row to Forgejo-PR + cloud-ci.
- **consensus_needed:** no (the supersession is well-attested; this is bookkeeping).

---

### ADR-0104 — Ecosystem-expansion principle for check-lane + adapter crate reintroduction

- **decision_atom:** A crate ships **iff** its kernel/domain layer is shipped, ≥1 workspace consumer imports it, and it has a real (non-stub) implementation; otherwise it is deferred with a documented ecosystem trigger — "the toolchain expands with the ecosystem."
- **current_status:** Accepted.
- **disposition:** KEEP (core principle) / AMEND (naming + stale milestone refs).
- **governing:** None supersedes the principle. Brand: the worked examples (`oya-foundry-account-app`, `oya-foundry-*-adapter`) carry the retired `foundry` brand; milestone triggers (`M02-P12`, `M02-P18`) use retired milestone vocab. The enforcement lane it spawns (`oya-governance-adapter-with-no-importer-kernel`) correctly uses the live `governance` prefix.
- **truth_flag:** TRUE (principle) / PARTIAL overall (examples carry stale `foundry`/`M0x` naming).
- **in_masterplan:** PARTIAL — no `planning_impact` flag, but the ADR explicitly states the rule "lives in `specs/masterplan.json`" and amends `specs/crate-naming-audit.json`. So it claims a masterplan binding without front-matter to prove it (8.8%-binding problem per map §4).
- **tensions:** (a) Directly cites DP-09 (bench before claiming performance) — which **ADR-0101 in this very chunk violates** (unquantified "lower overhead"). Internal cluster contradiction. (b) FO-01 "no parallel canonical trees" cited approvingly — consistent with the rest of the corpus. (c) The deferred cloud-adapter families (aws/oci/fake) intersect the canonical cloud-substrate posture (ADR-0192/0196 best-of-breed) — the deferral logic is sound but the cloud-vendor adapter list should reconcile with the actual chosen substrates.
- **hyperscaler_challenge:** **Aligned — strongly.** "No crate without a consumer and a real impl; defer with an explicit trigger" is precisely how large monorepos (Google's `BUILD`-visibility + no-orphan-targets, Meta's Buck) prevent dead code. The deletion-first + reintroduce-with-real-impl discipline is exactly right. Argues for **keep**.
- **ai_slop:** Low, but **heavy euphemism**: "scheduled-for-distinct-tracked-work" appears ~10× as a verbose stand-in for "deferred/TODO" — classic AI hedging/word-inflation. The arithmetic line in Consequences ("41 - (21+36+13) = -29 ← (math fix:...)") is a visible self-correction that should have been cleaned up, not shipped.
- **refinement:** Replace "scheduled-for-distinct-tracked-work" with "deferred (trigger: …)". Add proper `planning_impact: true` front-matter since it claims masterplan + crate-naming-audit bindings. Refresh `foundry`/`M0x` example names. Resolve the visible math-fix annotation.
- **consensus_needed:** no — the principle is good and uncontested; reconciliation is mechanical.

---

### ADR-0105 — 13-value canonical layer enum + check-family + backend-suffix patterns (amends ADR-0056)

- **decision_atom:** Extend the canonical Clean-Architecture layer enum from 12 to 13 by adding a protocol-neutral **`api`** contract-surface layer (depends on `kernel` only), and formally recognize two crate patterns: `oya-check-<feature>` self-layering fitness-check crates and `*-adapter-<backend>` backend-qualified adapters.
- **current_status:** Accepted; `planning_impact: true`; `amends: ADR-0056`; `supersedes: [ADR-0107]`.
- **disposition:** KEEP (this is the live canonical layer-enum authority).
- **governing:** Governs ADR-0107 (Superseded — confirmed on disk). Superseded-in-spelling by ADR-0106 only for the `application`→`usecase` rename (enum size stays 13). So 0105 + 0106 together are the current enum truth.
- **truth_flag:** TRUE (with the caveat that the `application` value it lists is renamed by 0106 in the very next ADR — read 0105∧0106 as a pair).
- **in_masterplan:** PARTIAL — carries `planning_impact: true` (proper front-matter, good) and updates `specs/crate-naming-audit.json` in-commit; but MASTERPLAN.md has 0 hits for "13-value/13-layer" so the human projection is not yet backfilled.
- **tensions:** (a) Internal: lists `application` as a canonical value, immediately renamed by ADR-0106 — a same-day pair that must be read together (the `api` extension is 0105's; the `application`→`usecase` rename is 0106's). (b) The embedded `ALLOWED_DEPENDENCY_ROLES` reconciliation still carries legacy `application`/`runtime`/`test` "transitional" rows — staged-migration debt. (c) Supersedes ADR-0107 (clean). (d) The visible "math fix" parenthetical in Consequences (line 80) is the same un-cleaned self-correction pattern as ADR-0104.
- **hyperscaler_challenge:** **Aligned.** A closed, ADR-gated layer enum with a protocol-neutral contract layer separate from `rest`/`grpc`/`graphql` is exactly the API-design-review discipline at AWS/Google (the "API surface ≠ transport" split is canonical, cf. gRPC service vs transport, Smithy models). A 1-ADR gate to extend the enum is sound governance. Argues for **keep**.
- **ai_slop:** Moderate. The Consequences arithmetic ("245 + 21 + 36 + 13 + 41 - (21+36+13) = -29 ← (math fix: …)") is **internal contradiction left in the document** — a real slop instance. Multiple overlapping "Amendment 2026-05-15" sections make the ADR sprawling (length-cap pressure vs ADR-0056's 500-line cap).
- **refinement:** Remove the broken arithmetic line; collapse the three appended same-day amendment sections into the body or split into successor ADRs; explicitly note at the top that `application` is renamed by ADR-0106 (forward-reference). Backfill MASTERPLAN.md with the 13-value enum since `planning_impact: true`.
- **consensus_needed:** no — this is settled, well-formed canonical doctrine. (Soft cross-chunk note: only if the founder decides masterplan-is-generated-from-ADRs, this ADR's `planning_impact` block is exactly the kind of front-matter the generator would consume — relevant to the OPEN authored-vs-generated question, but not itself contested.)

---

### ADR-0106 — Rename `application` layer to `usecase` (amends ADR-0105)

- **decision_atom:** Rename the port-only orchestration layer from `application` to **`usecase`** (Clean-Architecture canonical name) to disambiguate it from the `app` composition-root layer; enum size stays 13, six `*-application` crates rename to `*-usecase`.
- **current_status:** Accepted; `planning_impact: true`; `amends: [ADR-0056, ADR-0105]`.
- **disposition:** KEEP.
- **governing:** None supersedes it; it is the terminal node of the 0056→0105→0106 layer-enum chain. It is the current canonical spelling.
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — proper `planning_impact: true` front-matter; MASTERPLAN.md has 0 hits for "usecase" → human projection not backfilled.
- **tensions:** (a) Leaves **5 disk-but-not-workspace `*-application` crates** unrenamed (audit finding #6) — a known, tracked drift, not a contradiction. (b) Downstream doctrinal docs (decision-principles.json, forbidden-operations.json, ADR-0061, clean-architecture.md) still say "application" as a layer → cite-sweep debt acknowledged in Follow-ups. (c) Pairs tightly with ADR-0105 (must be read together for the enum).
- **hyperscaler_challenge:** **Aligned.** Borrowing the 30-year-old Clean-Architecture "Use Cases" name to kill `app`/`application` ambiguity is exactly the kind of naming-clarity decision Google/AWS API councils make; the rename-cost analysis (6 vs 10 crates) and the rejected alternatives (`service` overloaded, `bin` too generic) are well-reasoned. Argues for **keep**.
- **ai_slop:** None material. Unusually clean ADR — explicit alternatives, blast-radius math that actually checks out, honest tracking of unrenamed disk crates.
- **refinement:** Execute the Follow-up cite-sweep so `application`-as-layer disappears from decision-principles.json / forbidden-operations.json / clean-architecture.md / ADR-0056 body. Backfill MASTERPLAN.md with `usecase`. Decide the 5 orphan `*-application` crates (add-to-workspace or delete).
- **consensus_needed:** no.

---

## Chunk notes for synthesis

**Patterns / clusters.**

1. **The "foundry" retired-brand stain (0100, 0101, 0102, and 0104's examples).** Four of seven ADRs in this chunk carry the RETIRED `foundry` brand in crate names, µservice names, and templates (`oya-foundry-supervisor-*`, `oya-foundry-settings-template-*`, `oya-foundry-account-*`, historically `templates/foundry-supervisor/` — **deleted**; hooks pointed at missing `tools/foundry-supervisor-*`). Per ADR-0335/0347 + GLOSSARY "Foundry (RETIRED)" + founder ruling ("cloud-intelligence is the valid name"), every one is **AMEND-for-naming**: the *decisions* are sound, the *vocabulary* is dead. This is a corpus-wide pattern (map: "hundreds of `oya-foundry-*` strings persist"), so synthesis should treat 0100/0101/0102 as a batch foundry→intelligence rename, NOT individual archives. Open sub-question: supervisor/account/settings are consumer-AI orchestration → **intelligence**; banned-primitives/governance lanes → **governance**. The split per crate must follow the ADR-0335 (intelligence) vs ADR-0347 (governance) cut.

2. **Retired milestone vocabulary (`M01-P0x`, `M02-P0x`) everywhere (0101, 0103, 0104).** These ride the retired `M0/M1/M2/Milestone` vocab (GLOSSARY L250/L504, MFL-0003 → descriptive Wave names). Every milestone-keyed trigger in this chunk is a stale ref needing a Wave-name + numeric-trigger rewrite.

3. **The layer-enum doctrine pair (0105 + 0106) is the cluster's durable, KEEP-worthy output.** Together they are the **current canonical 13-value layer enum** with `usecase` (not `application`) — read as a pair. They correctly supersede ADR-0107, carry proper `planning_impact: true`, and are the only chunk-12 ADRs with planning front-matter. They are prime MASTERPLAN backfill material (the enum is not yet in MASTERPLAN.md — 0 hits). 0106 is the cleanest ADR in the chunk; 0105 is sound but sprawling (multiple same-day appended amendment sections) and carries a broken-arithmetic slop line.

4. **An internal cluster contradiction on the DP-09 "bench before claiming performance" principle.** ADR-0104 *cites DP-09 as a driver*; ADR-0101 (same author, same day) *violates it* with an unquantified "Direct Hyper mounting is lower overhead" claim. Synthesis should flag 0101's performance assertion as fabricated precision against the cluster's own stated doctrine.

5. **Recurring un-cleaned self-correction slop.** Both ADR-0104 and ADR-0105 ship a visible parenthetical "math fix" inside Consequences ("… = -29 ← (math fix: …)"). This is an AI-slop signature (the author corrected the count inline instead of fixing the sentence) and should be cleaned in any masterplan-bound version.

6. **ADR-0103 is the only ARCHIVE in the chunk.** It sits on the retired `grit`/`icm`/agentic-VCS/merge-queue substrate (ADR-0116 retires grit-coord tooling; ADR-0363 retires agentic-VCS + merge-queue; its renumber-parent ADR-0052 is Superseded-by-0118). Its *surviving principle* — "ban direct git/gh from automation; every ban names its replacement; compat windows are explicit + ADR-anchored" — is the durable, masterplan-relevant atom; the grit/icm *mechanism* is dead. Note the front-matter drift (`superseded_by:[]` despite a fully-retired substrate), mirroring the ADR-0136/ADR-0005 stale-front-matter pattern from map §5.6.

**Cross-chunk tensions to carry up.**

- **Forge fault-line (map §5.4) touches 0103.** ADR-0103's "controller-owned merge queue" is exactly what ADR-0363 retires in favor of plain-git + Forgejo PRs — which itself is the contested transitory canon vs the founder's GitHub directive vs ADR-0510's bespoke-VCS destination. 0103 is downstream of that whole fault-line; archive it and let the forge resolution govern the surviving banned-primitives rule.
- **Masterplan authored-vs-generated (map §4) is live for 0104/0105/0106.** These three claim masterplan/spec bindings (`specs/masterplan.json`, `specs/crate-naming-audit.json`) and 0105/0106 carry `planning_impact: true`. Under **generated-from-ADRs**, they are model citizens (the generator consumes exactly this front-matter). Under **masterplan-as-authority**, the layer enum needs to be *written into* MASTERPLAN.md (currently 0 hits). Flag the 13-value enum + `usecase` rename for masterplan backfill **under both readings** — it is TRUE, load-bearing, and currently absent from the human projection.

**Consensus questions surfaced (load-bearing).**

- *(Brand cut, batch)* For the retired-`foundry` decisions in this chunk (supervisor 0100/0101, settings-template 0102, account-family 0104 examples): confirm the per-crate split — supervisor/account/settings → **cloud-intelligence**, banned-primitives/naming lanes → **governance** — so the rename is mechanical, not re-litigated per ADR? (Phrasing for founder ruling; default answer implied by ADR-0335/0347 is "yes".)
- *(Masterplan binding)* Should the canonical 13-value layer enum + `usecase` rename (ADR-0105/0106) be **written into MASTERPLAN.md** (masterplan-as-authority) or remain authored-only-in-ADR with the masterplan **generated** from their `planning_impact` front-matter? This is the OPEN founder question (map §4) instantiated on a concrete, TRUE, currently-unbacked decision.
