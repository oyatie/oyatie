# Cross-Tension Register — Theme: policy-authz-autonomy-governance

> CONTRADICTION HUNTER pass (initial sweep 2026-06-06). READ-ONLY synthesis; no audited doc modified.
> SOURCE = `~/Developer/source` (346 ADRs). LINUX = `~/Developer/linux` (pilot, ADR-0001–0026).
> Scope: the authorization / policy-engine / autonomy-ceiling / admission / EU-AI-Act cluster.
> SOURCE ADRs read on disk: 0007, 0022, 0099, 0139, 0144, 0183, 0191, 0243, 0246, 0294, 0379 (+ verified 0150/0140/0021 identity).
> LINUX ADRs read on disk: 0021 (owned policy framework), 0022 (adopt-research-own-Rust methodology).
> Governing rule: latest/locked ADR wins; resolutions are SURGICAL (cross-ref / supersede-pointer / clarify) — never new policy.
> Founder questions are flagged where the call is genuinely the founder's (own-vs-reuse, authored-vs-generated masterplan, forge).

---

## 0. The headline (theme thesis, with the answer the corpus already implies)

The founder's framing question — *"Is the resolution: Cedar = external-standard CONTRACT, owned PARC = the engine behind it?"* — is **already the design both repos converge on, and it is NOT a contradiction.** It is the cleanest reconciliation in the entire audit:

- **LINUX ADR-0021** explicitly designs an *owned, typed, compile-to-Rust, tier-aware* authorization framework that is **"Cedar-compatible"**: it ingests Cedar syntax unchanged (the DSL is a *superset* of Cedar), retains Cedar's PARC model + forbid-trumps-permit + Lean-verified soundness as **non-negotiable**, vendors `cedar-policy` as the day-0 adapter AND the differential-testing **oracle**, and only swaps in the owned compiler when it beats-or-parities the oracle (ADR-0019 ratchet, S0→S3). (ADR-0021 §2.1, §4, §7.)
- **LINUX ADR-0022** (methodology) names this verbatim in its matrix: *"policy framework (ADR-0021): ADOPT Cedar (PARC + Lean-verified soundness), Zanzibar, OPA; FLAVOR = Cedar-compatible + compile-to-Rust (not interpret); autonomy-tier; sound evaluator."* (ADR-0022 line 56.)
- **SOURCE ADR-0183** independently arrives at the same split from the buy side: *"Cedar's formal analyzability is a deep research investment… Building an Oya-native authz **engine** would reimplement Cedar's analyzability with less rigor… oyatie's in-house effort goes into the **policy compiler + ClusterPolicy catalog** running on KEEP-classified standard engines."* (ADR-0183 §In-house roadmap.) SOURCE keeps Cedar-the-engine and owns the *compiler/asset layer*; LINUX owns the *engine* but keeps Cedar-the-contract. **Same contract, opposite build-vs-buy verdict on the engine internals** — a reconcilable own-vs-reuse axis, not a flat contradiction. The trigger-threshold (when does the owned compiler fire) is the only genuine open question, and BOTH sides already share the "own-when-proven" ratchet (LINUX ADR-0019/0020, SOURCE ADR-0211).

Everything below is either (a) this own-vs-reuse axis instantiated on a concrete ADR, (b) a SOURCE-internal data-integrity defect that must be fixed before the masterplan can bind the policy cluster, or (c) a vocabulary/status-drift cleanup.

---

## 1. TENSIONS (each: positions → contradiction-or-overlap → which governs → proposed surgical resolution → founder flag)

### T-1 — Own-the-policy-ENGINE (LINUX) vs reuse-Cedar-the-engine (SOURCE). **RECONCILABLE OVERLAP, not contradiction.**
- **Positions.**
  - LINUX **ADR-0021** (`accepted`, 2026-06-06): owned typed compile-to-Rust evaluator; `cedar-policy` is the *vendored adapter now / owned port later*; autonomy-tier T1–T4 promoted to a compiler-enforced schema dimension; SMT analysis via vendored `cedar-policy-symcc`.
  - SOURCE **ADR-0243** (`Proposed`) + **ADR-0246** (`Proposed`): Cedar v4.2 (the interpreter, `cedar-policy` crate) is the **universal gate**, promoted to its own `microservices/policy-engine/` substrate; in-house value is the *fragment lifecycle + compiler that emits CiliumNetworkPolicy/AuthorizationPolicy + coverage CI*, NOT the engine. SOURCE **ADR-0183** §In-house roadmap explicitly **rejects building an in-house authz engine**.
- **True contradiction?** **No.** Both pin the SAME contract (Cedar PARC syntax/semantics, forbid-trumps-permit, formal analyzability). They differ only on whether the *evaluator binary* is reused (SOURCE) or eventually owned via a compile-to-Rust port behind a Cedar-compatible seam (LINUX). LINUX ADR-0021 positions itself as the **owned successor to exactly ADR-0243's model**, gated by the shared own-when-proven ratchet. This is the keystone map §5 fault-line #2, confirmed.
- **Which governs (today).** For the *running platform*: SOURCE's Cedar-as-reused-engine is the only thing with downstream substrate, CI lanes, and a bootstrap slot — it governs the merged system **now**. LINUX ADR-0021 governs the *owned-ideal port shape* and is correctly staged (S0 vendored adapter = today; owned compiler S1–S2 gated by differential parity vs the `cedar-policy` oracle). Neither is "wrong"; they are different points on one ratchet.
- **Proposed resolution (surgical).**
  1. On merge, add a reciprocal cross-ref: LINUX ADR-0021 ↔ SOURCE ADR-0243/0246 noting "same Cedar contract; ADR-0021 is the owned-engine port, ADR-0243/0246 is the vendored-engine substrate; the ADR-0019/0211 ratchet decides when the owned compiler replaces the vendored evaluator."
  2. SOURCE ADR-0183's "Why no in-house authz engine" paragraph should gain a one-line cross-ref to LINUX ADR-0021's *compile-to-Rust* thesis — the two are not contradictory because ADR-0021 does **not** reimplement Cedar's analyzability (it vendors `cedar-policy-symcc` and uses `cedar-policy` as the Lean-verified oracle). Without this note, a naive synthesis logs a spurious "build vs buy" conflict.
- **DECISION-NEEDED-FROM-FOUNDER.** *Is Cedar the permanent external-standard CONTRACT with the owned compile-to-Rust PARC engine as the long-horizon implementation behind it (LINUX ADR-0021 thesis), or does Cedar-the-interpreter remain the engine indefinitely with ownership limited to the fragment/compiler asset layer (SOURCE ADR-0183/0243 thesis)? And if the former, what is the trigger threshold that fires the owned compiler (differential-parity-only, or a measured latency/sovereignty mandate)?* This is load-bearing for both repos and is the single most important policy ruling in the merge.

### T-2 — DATA-INTEGRITY ALARM: the canonical "Cedar policy engine" ADR (`ADR-0150-cedar-policy-engine.md`) **does not exist on disk**; the whole SOURCE Cedar chain is anchored on a phantom/misnumbered ADR. **TRUE DEFECT.**
- **Evidence (verified on disk).**
  - ADR-0243 `amends: ADR-0150-cedar-policy-engine.md`; ADR-0246 `amends: ADR-0150-cedar-policy-engine.md`; ADR-0294 `related: ADR-0150-cedar-policy-engine.md`. The keystone map §1.3 policy chain also reads "ADR-0150 (Cedar engine)."
  - **On disk, `ADR-0150` = `ADR-0150-cursor-pagination-canonical.md` ("Cursor Pagination Canonical")** — zero Cedar content. There is **no** `ADR-0150-cedar-policy-engine.md` file.
  - There is **no `status: Accepted` Cedar-engine ADR anywhere** in the corpus. The engine pick lives only in: ADR-0007 (`proposed`), ADR-0183 (`Superseded`→0379), ADR-0243/0246/0294 (all `Proposed`). The most-cited "establishing" ADR-0150 is a ghost.
  - The real policy-engine-separation ADR is **ADR-0183** (correctly `Superseded by ADR-0379`); ADR-0148 line 257 and the keystone map both *also* mislabel 0150 as the Cedar/Kyverno-separation ADR (it is 0183). (Confirmed in chunk-18 disposition picture and on disk here.)
- **True contradiction?** This is not an opinion conflict — it is a **broken-reference / number-reuse defect** that poisons any generated-from-ADRs masterplan graph (the supersede/amend edges point at a cursor-pagination doc). Under "masterplan GENERATED from ADR front-matter," ADR-0243's `amends:` edge would silently bind Cedar's universal-gate doctrine to *pagination*.
- **Which governs.** The *intent* (Cedar v4.2 as the app-authz engine) is real and carried by ADR-0243's body + ADR-0183's body; the *pointer* (ADR-0150) is wrong.
- **Proposed resolution (surgical, no new policy).**
  - Re-key every `ADR-0150-cedar-policy-engine.md` reference (ADR-0243 amends, ADR-0246 amends, ADR-0294 related, ADR-0148:257, keystone map §1.3) to point at the **actually-existing** Cedar authority: the live app-authz pick is **ADR-0007** (engine adoption) → **ADR-0183** (separation, now superseded by **ADR-0379**) → **ADR-0243** (universal-gate extension). If a distinct "Cedar engine v4.2" ADR was ever authored under another number, locate it and repoint; if it was never authored, ADR-0243 must **author** the engine-pick clause inline (it currently only *extends* a non-existent parent) before it can promote to Accepted.
  - Leave `ADR-0150-cursor-pagination-canonical.md` untouched and KEEP — it must NOT be archived (it is a valid live ADR mis-identified by the chain).
- **DECISION-NEEDED-FROM-FOUNDER (data-integrity escalation, ties to chunk-8 ADR-0055/0057 number-reuse finding).** *Does the founder want a strict no-dangling-`amends`/`supersedes` invariant (mandatory if the masterplan is GENERATED from ADR edges) — and how is the reused/missing `ADR-0150-cedar-policy-engine` number reconciled on the re-founded log (re-author the engine pick as a fresh ADR-0000-series node, or fold it into ADR-0243)?*

### T-3 — Autonomy-ceiling is authored THREE times across SOURCE + LINUX with diverging tier definitions and conflicting ownership; no single canonical autonomy ADR. **PARTIAL CONTRADICTION (semantic drift) + ownership ambiguity.**
- **Positions / the three authorings.**
  - SOURCE **ADR-0007** (`proposed`): T1=view-only, T2=advisory, **T3=execute-with-approval, T4=auto-execute**; default T2; runtime gate in `oya-foundry-runtime-policy-*`; owner = `tenancy-identity` + `foundry` + `council-privacy`.
  - SOURCE **ADR-0022** (`proposed`): T1=recommend-only, **T2=supervised, T3=scheduled-autonomous, T4=continuous-autonomous**; effective ceiling = `min(tenant, capability_min, vertical_pack, subject_class)`; gate in `oya-foundry-policy-app`; owner = `foundry`; adds break-glass M-of-N (2-of-3 / 3-of-5), tenant-class forcing (healthcare/fintech→T2), `ads.*`→T1.
  - SOURCE **ADR-0099** (`accepted`): a THIRD T1–T4 labelling (T1=read-only-observer, T2=suggest-only, T3=act-with-approval, T4=full-autonomy) for `foundry.supervisor.*` capabilities, in `docs/policies/foundry-supervisor.cedar`.
  - LINUX **ADR-0021**: T1<T2<T3<T4 as a **closed enum entity type, total order**, promoted to a *compiler-enforced* schema dimension (a missing tier guard = compile error).
- **True contradiction?** **Partial.** All four agree T1–T4 is an ordered autonomy ceiling enforced via Cedar on every capability invocation (genuine cohesion). But the **tier-label semantics diverge between ADR-0007 (T2=advisory/draft) and ADR-0022 (T2=supervised-execution)** — T2 means "draft, human approves" in one and "execute under supervision" in the other; T3/T4 likewise shift. ADR-0099 invents a third gloss. This is real semantic drift that will produce inconsistent enforcement if both fragments load. Ownership is also split (`tenancy-identity`+`foundry` vs `foundry` vs unclear), and the gate crate name differs (`oya-foundry-runtime-policy-*` vs `oya-foundry-policy-app`).
- **Which governs.** **ADR-0022** is the more complete and operationally-specified autonomy authority (effective-ceiling resolution, break-glass, tenant-class forcing, CI lanes) and ADR-0099 explicitly cites *both 0007 and 0022* as its mandate — so 0022's semantics should be canonical; ADR-0007's T2=advisory gloss should defer to it. LINUX ADR-0021's total-order enum is the *type-level* canonical encoding and is compatible with 0022's `min_of` semantics.
- **Proposed resolution (surgical).**
  1. Pick **one** T1–T4 semantic table (ADR-0022's) as canonical; add a one-line "tier semantics per ADR-0022" cross-ref to ADR-0007 (re-scope its table as the early draft 0022 supersedes-in-spirit) and ADR-0099 (note its labels are the 0022 tiers).
  2. Name ONE canonical autonomy-ceiling owner and ONE gate crate post-foundry-retirement (see T-5): the gate is `oya-policy-*` (LINUX ADR-0021) / `oya-policy-engine-*` (SOURCE ADR-0246), NOT `oya-foundry-policy-*`.
  3. Bind the *effective-ceiling = min(4 sources)* rule + break-glass M-of-N + tenant-class forcing into the masterplan as the canonical autonomy invariant (currently unbound — none of 0007/0022/0099 carry `planning_impact`/`masterplan_ref`).
- **DECISION-NEEDED-FROM-FOUNDER.** *Which T1–T4 semantics are canonical (ADR-0007's advisory-centric or ADR-0022's execution-centric), and does the autonomy ceiling live in `intelligence` (post-foundry-absorption, ADR-0335) or in `governance`/`policy-engine`? ADR-0007/0022 both name the retired `foundry` owner.*

### T-4 — "Accepted-on-Proposed" inversion: `accepted` Cedar/autonomy ADRs rest on `proposed` parents; an `Accepted` engine separation is superseded by a `Proposed`-adjacent chain. **TRUE DEFECT (status drift / maturity inversion).**
- **Positions / evidence (verified on disk).**
  - **ADR-0099** is `accepted` but both its mandating parents — **ADR-0007 (`proposed`)** and **ADR-0022 (`proposed`)** — are still Proposed. An accepted policy file rests on two unaccepted foundations.
  - **ADR-0243/0246** (`Proposed`) declare they `amend` ADR-0150 (phantom, T-2) and extend ADR-0183 (`Superseded`) — i.e., the universal-gate doctrine extends an already-dead separation ADR without citing its successor ADR-0379.
  - **ADR-0144** (`Accepted`) cites `ADR-0140 — Cedar policy enforcement substrate` as a live anchor, but **ADR-0140 on disk = cross-cutting-carriers, `status: Superseded`** (→ADR-0145), and has **zero Cedar content**. A second phantom-Cedar-anchor, parallel to T-2.
- **True contradiction?** Yes — these are objective front-matter/status defects, not opinions. They block any generated-from-ADRs masterplan (an accepted node generated from proposed parents; amend-edges into superseded/non-existent nodes).
- **Which governs.** The superseding/real ADRs: ADR-0379 over ADR-0183; ADR-0145 over ADR-0140; and ADR-0007/0022 must reach `Accepted` (or be folded into ADR-0243) before ADR-0099 can legitimately be accepted.
- **Proposed resolution (surgical).**
  1. ADR-0144: repoint `ADR-0140 (Cedar policy enforcement)` → the real Cedar authority (ADR-0007/0243), exactly as T-2.
  2. ADR-0243/0246: change the ADR-0183 reference from "separation preserved" to "separation per ADR-0183 **as carried forward by ADR-0379** (Kubewarden admission)."
  3. Promote ADR-0007 + ADR-0022 to `Accepted` (or fold their durable atoms into ADR-0243) so ADR-0099's parents are no longer Proposed — a status-only fix, no policy change.
- **DECISION-NEEDED-FROM-FOUNDER.** Ties to the keystone §4 open question: *under the authored-vs-generated masterplan, is "status" stored in the ADR (then these inversions must be hand-fixed before binding) or derived from gate output (then the front-matter `status:` fields are advisory and the masterplan binds on `verified_by` instead)?*

### T-5 — Retired-`foundry` brand saturates the entire policy/authz cluster. **NOT a contradiction — retired-vocabulary leakage; KEEP-atom / AMEND-vocab.**
- **Evidence.** ADR-0007 (`oya-foundry-runtime-policy-*`, owner `foundry`), ADR-0022 (`oya-foundry-policy-kernel/app/domain`, owner `foundry`), ADR-0099 (`foundry.supervisor.*` capability namespace, `docs/policies/foundry-supervisor.cedar`, `oya-foundry-autonomy-ceiling-app`), ADR-0243/0246/0294 (`oyatie.foundry.meta-trust-root`, `oyatie.foundry.adr-drafter`, `oyatie.foundry.bootstrap-ca` principals). All retired per **ADR-0335** (foundry→intelligence) + **ADR-0347** (`oya-foundry-*`→`oya-governance-*`); founder-confirmed "cloud-intelligence is the valid name."
- **Which governs.** ADR-0335/0347. The *decisions* are sound; the *names* are dead.
- **Proposed resolution (surgical, batch).** Fold the policy-cluster rename into the ADR-0347 bulk rename so crates aren't renamed twice: autonomy-gate/policy crates → `oya-policy-*` / `oya-policy-engine-*` (the policy substrate is `governance`-adjacent per ADR-0335's "Governance stays separate"); agent-session/supervisor principals → `cloud-intelligence`/`intelligence`. Do NOT archive; the atoms survive verbatim under new names.
- **DECISION-NEEDED-FROM-FOUNDER.** *Does the canonical policy/autonomy substrate belong to `governance` (CI/gates/policy-engine) or to `intelligence` (foundry-absorbing AI subsystem)? ADR-0335 keeps Governance separate but ADR-0007/0022 put the gate in `foundry` — the split is currently ambiguous for the autonomy ceiling specifically.*

### T-6 — Admission engine: ADR-0183 Kyverno (Superseded) vs ADR-0379 Kubewarden (Accepted). **RESOLVED IN SOURCE — pure status/cross-ref hygiene.**
- **Positions.** ADR-0183 (`Superseded by ADR-0379`) made Kyverno the admission engine; **ADR-0379** (`Accepted`, founder among deciders) makes **Kubewarden the default**, Kyverno a first-class adapter, and explicitly **preserves the Cedar↔admission SEPARATION principle unchanged**.
- **True contradiction?** **No** — this is a clean supersession; the only defect is downstream refs (ADR-0191 origin-tier table, ADR-0243 §"What this is NOT," ADR-0246 context) that still say "Kyverno per ADR-0183" without the ADR-0379 repoint.
- **Which governs.** **ADR-0379** (latest, Accepted).
- **Proposed resolution (surgical).** Repoint admission references: ADR-0243 §"What this is NOT" + §References, ADR-0191 origin-tier note, and the keystone map §1.1/§3 — "admission = **Kubewarden** (ADR-0379, supersedes ADR-0183's Kyverno); Kyverno retained as adapter; Cedar/admission separation unchanged."
- **No founder decision needed** — locked by ADR-0379. (Surface only: ADR-0379's Rust→WASM admission-policy model aligns with the Rust-everywhere posture and is consistent with both repos.)

### T-7 — "tier" is overloaded 4-ways across the policy cluster; autonomy-tier T1–T4 must not be conflated with the retired tenant tier-system. **NAMING-COLLISION RISK (not a contradiction).**
- **Evidence.** (a) **autonomy-tier T1–T4** (live; ADR-0007/0022/0099/0021); (b) **EU-AI-Act risk tier 0–4** (ADR-0144, `canonical-tier-schema.json`, `T2-auto`); (c) retired **tenant tier-system** → `tenant_class` (ADR-0329); (d) DR/SLO **T1/T2 tiers** (ADR-0241, used throughout ADR-0243/0246/0294 for RTO/RPO). ADR-0007 §Cedar example even writes `principal.autonomy_tier` *and* `context.requested_autonomy_tier` while ADR-0243 §D-13 feature-flag uses `TenantTier::["enterprise","pro"]` (retired per ADR-0329).
- **Which governs.** Keystone map §2: autonomy T1–T4 is a DIFFERENT live axis from the retired tenant tier-system; ADR-0329 governs tenancy naming.
- **Proposed resolution (surgical).** Namespace the four axes in the masterplan glossary: `autonomy_tier` (T1–T4), `eu_ai_act_risk_tier` (0–4), `dr_tier`/`slo_tier` (T1/T2), and replace ADR-0243 §D-13's `TenantTier::["enterprise","pro"]` with `tenant_class`+`billing_components` per ADR-0329. One-token edits; no semantics change.
- **DECISION-NEEDED-FROM-FOUNDER.** Minor: *confirm `autonomy_tier`, `eu_ai_act_risk_tier`, `dr_tier` as the three canonical non-colliding "tier" names so the EU-AI-Act `canonical-tier-schema.json` is never read as the retired tenancy tiers.*

### T-8 — Edge-vs-origin authz boundary (ADR-0191) leaks retired Redis + Kafka and cites the superseded admission ADR, but the BOUNDARY DOCTRINE is sound. **PARTIAL — atom survives, vocab/cross-ref stale.**
- **Evidence.** ADR-0191 (`Accepted`) cleanly separates edge (Envoy: IP/ASN/geo/WAF/rate/DDoS) from origin (Istio waypoint Cedar PDP: principal/action/resource/residency/ACR/data-class) with a `oya-check-authz-tier-discipline` gate that *forbids* a Cedar policy from mentioning IP/geo and an Envoy filter from mentioning OIDC claims — a genuinely hyperscaler-grade, correct doctrine. But it leans on Redis-backed counters (retired→Valkey ADR-0336) and cites ADR-0183 (Superseded→0379) as the origin-tier admission anchor.
- **Which governs.** Doctrine stands (KEEP). Repoint substrate names (Valkey per ADR-0336) and admission ADR (0379).
- **Proposed resolution (surgical).** Two one-token vocab fixes (Redis→Valkey) + one cross-ref repoint (0183→0379). Bind the edge/origin boundary table into the masterplan as a canonical authz invariant (currently no `masterplan_ref`).
- **No founder decision needed.**

### T-9 — SOURCE Cedar PDP is Postgres+Citus+Valkey-backed (fragment registry) — collides with LINUX ADR-0001 "eliminate PostgreSQL." **CROSS-SIDE DATA-TIER FAULT-LINE touching policy (surface, do not resolve here).**
- **Evidence.** ADR-0246 §D-7 pins the Cedar fragment-registry to **Postgres 17 + Citus + Valkey hot-cache**; ADR-0243 §D-6 cold path = "Postgres+Citus query." LINUX ADR-0001 wants an owned Rust multi-model engine that *eliminates* Postgres. The policy substrate is therefore downstream of the keystone §5 fault-line #1.
- **Which governs.** Out of theme scope to resolve — defer to the data-tier cross-tension register. Flag only: the policy-engine substrate is one of the concrete artifacts that breaks if ADR-0001 governs the data tier.
- **DECISION-NEEDED-FROM-FOUNDER.** Cross-referenced, not owned here: *does the Cedar fragment registry sit on reused Postgres+Citus (SOURCE) or on the owned multi-model engine (LINUX ADR-0001)?* Bind wherever the data-tier ruling lands.

---

## 2. Soundness check on the LINUX auto-reconciliation (wm4gkcey5) for THIS theme — NOT "plain wrong"
- LINUX **ADR-0021** is internally coherent and self-aware: its `review_note` (C2) records that **phantom `(research §N)/(landscape §N)` citations were removed** and (C3) that an **incorrect ADR-0007 attribution for the tier model was corrected** to the real sources (`spec:232` + `ADR-0020:103`). Verified on disk: ADR-0021 cites spec:232 / ADR-0020:103 for autonomy-tier provenance, not a fabricated source. ADR-0021 does **not** fabricate a SOURCE posture — it explicitly positions as the *owned successor* to the Cedar model and keeps `cedar-policy` as the vendored oracle. This is **deliberate divergence (own-vs-reuse), not a reconciliation bug.** No correction needed; the one item to watch is the merge cross-ref in T-1 (ADR-0021 ↔ ADR-0243/0246) so synthesis does not log a spurious build-vs-buy contradiction.
- LINUX **ADR-0022** matrix line for policy is accurate and consistent with ADR-0021. No defect.

---

## 3. RESULTING DISPOSITION CHANGES (driven by the tensions above)

| ADR | Side | Prior/typical disposition | Tension-driven change | Reason |
|---|---|---|---|---|
| **ADR-0150** (cursor-pagination) | source | KEEP | **KEEP — flag mis-identification** | T-2: must NOT be archived; the Cedar chain mis-cites it. Re-key the citers, leave the file. |
| **ADR-0007** (Cedar+persona-tier) | source | amend | **AMEND + PROMOTE-status** | T-3/T-4/T-5: defer T-semantics to ADR-0022; foundry→governance vocab; promote `proposed`→`Accepted` (or fold into ADR-0243) to unblock ADR-0099. |
| **ADR-0022** (autonomy ceiling) | source | (autonomy authority) | **AMEND-vocab + BIND + PROMOTE-status** | T-3 canonical tier semantics; foundry→governance; bind effective-ceiling/break-glass/tenant-class into masterplan; `proposed`→`Accepted`. |
| **ADR-0099** (supervisor Cedar) | source | amend | **AMEND** (parents-accepted dependency) | T-4 inversion: legitimate only once 0007/0022 are Accepted; foundry vocab; cross-check vs PR#605 agent-execution-controller (session-mgmt overlap). |
| **ADR-0140** (cross-cutting carriers) | source | archive (already Superseded) | **ARCHIVE — and scrub the phantom-Cedar cite in ADR-0144** | T-4: 0140 is NOT Cedar; ADR-0144 must repoint. |
| **ADR-0144** (EU-AI-Act tiers) | source | amend | **AMEND** | T-4 (repoint 0140→0007/0243) + T-7 (namespace `eu_ai_act_risk_tier`). Atom TRUE; bind to masterplan. |
| **ADR-0183** (Cedar/Kyverno sep) | source | archive (Superseded→0379) | **ARCHIVE — separation principle survives via ADR-0379** | T-6; ensure citers repoint to 0379. |
| **ADR-0243** (Cedar universal gate) | source | (Proposed keystone) | **AMEND** | T-2 (author engine-pick inline / repoint phantom 0150), T-6 (0183→0379), T-7 (TenantTier→tenant_class), foundry-principal vocab; bind to masterplan. |
| **ADR-0246** (policy-engine substrate) | source | (Proposed keystone) | **AMEND** | T-2 (phantom-0150 amends), T-6, T-9 (Postgres flag), foundry vocab. |
| **ADR-0294** (Cedar soak) | source | (Proposed keystone) | **AMEND** | T-2 (phantom-0150 related), foundry-principal vocab; Kafka→Pulsar (ADR-0377) on the soak topics. |
| **ADR-0379** (Kubewarden) | source | keep | **KEEP — promote to canonical admission row in keystone map** | T-6: latest/Accepted; map currently understates it. |
| **ADR-0191** (edge/origin authz) | source | (Accepted) | **AMEND-vocab** | T-8: Redis→Valkey, 0183→0379; bind boundary table. |
| **ADR-0021** (owned policy) | linux | keep (accepted) | **KEEP + add merge cross-ref to ADR-0243/0246** | T-1: prevent spurious build-vs-buy conflict; renumber on merge (collides with source ADR-0021 foundry-capability-registry, map §6.4). |
| **ADR-0022** (methodology) | linux | keep (accepted) | **KEEP** | T-1: policy matrix line accurate; renumber on merge (collides with source ADR-0022 autonomy-ceiling). |

---

## 4. Founder questions, consolidated (crisp)

1. **(T-1, load-bearing) Cedar contract + owned PARC engine?** Is Cedar the permanent external-standard CONTRACT with LINUX ADR-0021's owned compile-to-Rust PARC evaluator as the long-horizon engine behind it — or does the vendored Cedar interpreter stay the engine (SOURCE ADR-0183/0243) with ownership limited to the fragment/compiler asset layer? If owned: what fires the ratchet (differential-parity-only, or a measured latency/sovereignty mandate)?
2. **(T-2/T-4, data-integrity) Dangling-ref invariant.** The canonical `ADR-0150-cedar-policy-engine` and `ADR-0140-cedar-policy-enforcement` Cedar anchors do not exist / are mis-numbered, yet ADR-0243/0246/0294/0144 cite them as live. Mandate a strict no-dangling-`amends`/`supersedes` invariant (required if the masterplan is GENERATED from ADR edges)? And re-author the missing Cedar-engine pick as a fresh node or fold it into ADR-0243?
3. **(T-3/T-5) Canonical autonomy semantics + home.** Which T1–T4 table is canonical (ADR-0007 advisory-centric vs ADR-0022 execution-centric), and does the autonomy ceiling + policy gate live in `governance`/`policy-engine` or in `intelligence` (post-foundry-absorption, ADR-0335)?
4. **(masterplan, recurring) Authored vs generated for the policy cluster.** None of ADR-0007/0022/0099/0144/0191/0243(+) carry `masterplan_ref`/`planning_impact` for their durable atoms (effective-ceiling = min(4); forbid-trumps-permit no-false-allow; edge/origin boundary; EU-AI-Act 5-tier obligations; Cedar universal-gate coverage). Do these get WRITTEN INTO the masterplan (authored-as-authority) or do the ADRs gain front-matter and GENERATE it? (Status inversions in T-4 must be fixed first under the generated reading.)
5. **(T-9, cross-ref) Policy substrate data tier.** Does the Cedar fragment registry run on reused Postgres+Citus+Valkey (SOURCE ADR-0246) or on the owned engine that eliminates Postgres (LINUX ADR-0001)? (Owned by the data-tier register; bind the policy substrate to whatever lands.)

*End of policy-authz-autonomy-governance cross-tension register.*
