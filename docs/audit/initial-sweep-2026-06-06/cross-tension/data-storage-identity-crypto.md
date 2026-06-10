# Cross-Tension Register — Theme: data / storage / identity / crypto / time

> **Contradiction-hunter pass, initial sweep 2026-06-06.** READ-ONLY audit; the only write is this file.
> Scope: LINUX owned distributed-DB engine + HLC (ADR-0001..0008) vs SOURCE best-of-breed data substrates
> (postgres/pgcat 0179, milvus 0192, clickhouse 0193, timescaledb 0194, seaweedfs 0196, valkey 0336,
> iceberg 0337, db-tier 0045, time 0252) + SOURCE identity (zitadel 0187 / bespoke-rust 0394 / oya-identity 0476)
> + SOURCE crypto (aws-lc-rs 0506 / webauthn-rs 0507 / opensk 0508).
> Files cited with absolute or repo-relative paths; SOURCE = `~/Developer/source/docs/decisions/`, LINUX = `~/Developer/linux/docs/decisions/`.
> Governing rule (keystone map): **trust the superseding / later / founder-locked ADR over stale front-matter**; resolutions are surgical (cross-ref, supersede, clarify) — **never new policy**; LINUX pilot ADRs renumber on merge (map §6.4).

---

## 0. Topology of this theme (what is cross-side vs what is SOURCE-internal)

Verified on disk: **LINUX has NO identity, crypto, or IdP ADR** (grep across all 26 pilot ADRs: zero identity/crypto/zitadel/webauthn/oidc files; "crypto" appears only as `cargo deny` license-tier language). The LINUX pilot owns exactly one slice of this theme: the **distributed-database engine + storage + HLC + verification** (ADR-0001/0003/0005/0006/0008, with 0002/0004/0007/0020 supporting). Therefore:

- **Cross-side (LINUX ↔ SOURCE) tensions** exist only on **(A) the data tier** and **(B) time/clock/consistency**. These are the sharp ones.
- **Identity (T-3) and crypto (T-7) tensions are entirely SOURCE-internal** — three competing identity ADRs and a coherent-but-unbound crypto cluster. They matter to this theme because they are the most-regulated substrates and the masterplan must capture exactly one canonical answer.
- **Storage best-of-breed (milvus/clickhouse/timescaledb/seaweedfs/valkey/iceberg) is internally near-coherent on the SOURCE side**; the cross-side conflict is the *meta* question "does any of it survive if LINUX ADR-0001 governs," not engine-vs-engine.

---

## T-1 — DATA TIER: LINUX owned-DB ("eliminate-then-retain" Postgres) vs SOURCE best-of-breed Postgres+Citus  **[SHARPEST CROSS-SIDE — keystone fault-line #1]**

**Positions**
- **LINUX ADR-0001** (`LINUX/ADR-0001-distributed-database-engine-and-scope.md`, Accepted 2026-06-05): build a from-scratch Rust multi-model distributed engine (`oya-distributed-database-*`, canonical name `cloud-data`). The body now carries an auto-reconciliation note (lines 38–40, 115, 136 — inserted by the prior wm4gkcey5 pass) saying the engine "**does NOT eliminate Postgres**; PostgreSQL+Citus remains the reused OLTP substrate," with cloud-data positioned as a Spanner-class differentiator that "replaces etcd as the orchestration datastore."
- **SOURCE ADR-0045** (`SOURCE/ADR-0045-database-tier-strategy.md`, status `proposed`): PostgreSQL + Citus is the **canonical OLTP engine** fleet-wide; explicitly **rejects CockroachDB (BSL), TiDB, and Spanner/proprietary** (Alternatives B/C). SOURCE ADR-0179 (`postgres-connection-pooling-pgcat.md`, Accepted) makes **pgcat** the canonical pooler — an artifact that only exists *because Postgres survives*.

**True contradiction or reconcilable?** **Reconcilable in principle, but the reconciliation is currently asserted only on the LINUX side and is internally self-contradictory.** ADR-0001's own prose still contains the original "eliminate" framing next to the patched "retain" framing: Decision line 36 "eliminate external DB dependencies," Consequences line 148 "must run PostgreSQL in parallel until distributed-database reaches parity," rejected Alternative A is titled "Continue external PostgreSQL dependency" — **all left intact** and now glossed by the inserted lines 38/115/136 that say the opposite. So the *cross-side* tension (own vs assemble) is reconciled by the "cloud-data = differentiator, Postgres = retained OLTP" split, but the *within-ADR-0001* tension (eliminate vs retain) is a live two-pass-authoring contradiction.

**Which governs.** For the merged system, **SOURCE ADR-0045/0179 (Postgres+Citus+pgcat retained as OLTP) governs the OLTP substrate**, and **LINUX ADR-0001's owned engine governs only the "differentiator / orchestration-datastore" layer** — this is exactly what the LINUX-side reconciliation note now claims (citing `staged-ownership-roadmap-canonical.json:245` / `component-boundaries.md:34`). Note: the cited spec line `staged-ownership-roadmap-canonical.json:245` does **not exist in the LINUX tree** (the prior pass flagged this — it is a SOURCE-side reference), so the binding is currently unverifiable from LINUX.

**Proposed resolution (surgical).**
1. **No new policy** — adopt the boundary the LINUX note already states: Postgres+Citus = OLTP substrate of record; `cloud-data` = Spanner-class differentiator + etcd-replacement, NOT an OLTP replacement.
2. **AMEND LINUX ADR-0001** to finish the half-applied reconciliation: the residual "eliminate external DB dependencies" (line 36) and the Alternative-A title should be scrubbed/annotated so the file no longer asserts both directions. (Body-reconciliation pass, like the ADR-0147 fix on the SOURCE side — disposition AMEND, not archive.)
3. **CROSS-REF**: add `related: ADR-0045, ADR-0179` to LINUX ADR-0001 and a one-line "Postgres OLTP substrate is retained per source ADR-0045/0179" pointer, so the merge does not read 0001 as killing pgcat.
4. SOURCE ADR-0045 is `proposed` and pre-dates the Accepted storage cluster (0179/0192/0193/0194); it should be **AMENDED** to mark Citus-columnar's true license (0184 records Citus columnar = AGPL3, 0045 claims Apache-2 — a fabricated-clean license claim) and to drop the retired `foundry` owner/axis vocabulary.

**Disposition impact:** LINUX ADR-0001 → **AMEND** (finish reconciliation, add cross-refs). SOURCE ADR-0045 → **AMEND** (license correction + retired-vocab + status). SOURCE ADR-0179 → **KEEP** (add `related: ADR-0001` note for the merge boundary).

**DECISION-NEEDED-FROM-FOUNDER ❓** *Is the long-horizon end-state "own the entire data tier (cloud-data eventually absorbs OLTP, Postgres fully retired)" or the weaker "own only the differentiator engine; Postgres+Citus is the permanent OLTP substrate of record"? LINUX ADR-0001 was authored as the former and patched toward the latter; the two repos cannot both be canonical at Tier-1. The own-when-proven ratchet (LINUX ADR-0019/0020, SOURCE ADR-0211/0173) is shared — the open question is the **trigger threshold for Postgres retirement**, not the principle.*

---

## T-2 — TIME / CLOCK / CONSISTENCY: LINUX HLC-plain-serializable (no TSO) vs SOURCE HLC-default + TrueTime-Tier-4  **[CROSS-SIDE — mostly reconcilable, one real divergence]**

**Positions**
- **LINUX ADR-0006** (`LINUX/ADR-0006-hlc-consistency-and-serializability-non-goal.md`, Accepted): HLC + uncertainty restarts (CockroachDB model) for **stage-1 bedrock**; consistency ceiling = **plain serializable, explicitly NOT strict-serializable / linearizable**; **no central TSO in stage-1**; TrueTime/external-consistency is a north-star "later stage." Engine-level decision for the owned DB.
- **SOURCE ADR-0252** (`SOURCE/ADR-0252-time-coordination-distributed-consistency.md`, Proposed, planning_impact:true): **platform-wide** clock doctrine — HLC default for ≥95% of operations (canonical crate `oya-shared-time-kernel`), **TrueTime (GPS+atomic clock) reserved for Tier-4 financial-grade + IL5+ cells**, sagas + caller-supplied idempotency keys as the coordination primitive, distributed locks forbidden, Google leap-smear, per-cell cron. Default consistency = **causal**; strict-total-order opt-in via saga raft; external-consistency only at Tier-4.

**True contradiction or reconcilable?** **Largely reconcilable — same HLC family, same CockroachDB lineage, same "TrueTime is a later/Tier-4 thing" posture.** They operate at different layers: ADR-0006 is the *owned-DB engine's* internal consistency model; ADR-0252 is the *platform-wide application/coordination* clock doctrine. Two genuine seams to reconcile:
1. **Consistency-default wording.** LINUX ADR-0006 says the engine's ceiling is "plain serializable." SOURCE ADR-0252 D-3 says platform default is "causal consistency," with serializable/strict-total-order as opt-in, and §"What this is NOT" pins "Postgres REPEATABLE READ default; SERIALIZABLE opt-in." These are different axes (DB isolation vs cross-cell event ordering) but use overlapping words — **naming-collision risk**, not a logic conflict.
2. **TSO.** LINUX ADR-0006 D-4 forbids a central TSO in stage-1 and instructs "do not reserve a code path for it." SOURCE ADR-0252 D-2 introduces a TrueTime provider (`oya-shared-time-kernel::truetime`) for Tier-4. If the owned engine ever runs in a Tier-4 cell, it must accept an external-consistency clock — which ADR-0006 currently tells implementers **not** to leave a seam for. This is the one substantive divergence.

**Which governs.** SOURCE **ADR-0252 governs the platform clock primitive** (it is the cross-cutting, planning_impact keystone #11/14 with a canonical `oya-shared-time-kernel` crate). LINUX ADR-0006 governs the **owned engine's stage-1 internal model** and is consistent with 0252's HLC default. On the TSO/TrueTime seam, ADR-0252's "TrueTime opt-in for Tier-4" is the broader, later, platform-level decision.

**Proposed resolution (surgical).**
1. **CROSS-REF** LINUX ADR-0006 → SOURCE ADR-0252: add `related: ADR-0252` and one clause: *"Platform clock primitive is HLC per source ADR-0252; this ADR fixes the owned-engine's stage-1 ceiling (plain serializable). TrueTime/external-consistency (ADR-0252 D-2, Tier-4) is the engine's later-stage external-consistency mode."*
2. **AMEND** ADR-0006 D-4's "do not reserve a code path for TSO" to "do not *implement* a TSO in stage-1; the Clock **port** must remain swappable so a TrueTime adapter (ADR-0252 Tier-4) can be added later without a kernel rewrite" — this is consistent with ADR-0006's own hexagonal Clock-port design and with the LINUX ADR-0003 port-ratchet, so it is a clarification, not new policy.
3. **Namespacing note** (advisory): when these merge, "serializable" (engine isolation, ADR-0006) and "causal / strict-total-order" (cross-cell ordering, ADR-0252 D-3) need to be kept as distinct named axes in the masterplan to avoid the same "tier"-overload disease flagged corpus-wide.

**Disposition impact:** LINUX ADR-0006 → **AMEND** (cross-ref 0252 + Clock-port-keeps-TrueTime-seam clarification). SOURCE ADR-0252 → **KEEP/AMEND** (status `proposed`→bind; carries retired-vocab "Kafka per ADR-0005" at §"What this is NOT" line ~316 and `axis-identity`/Postgres references — light amend; planning_impact already true).

**DECISION-NEEDED-FROM-FOUNDER ❓** *Does the owned `cloud-data` engine ever need to run inside a Tier-4 external-consistency cell (financial settlement / IL5)? If yes, the owned engine must carry a TrueTime-capable Clock adapter seam from day-0 (amend ADR-0006); if Tier-4 financial workloads always stay on a separate substrate, ADR-0006's "no TSO seam" stands. This is the one place the two time-ADRs genuinely diverge.*

---

## T-3 — IDENTITY: Zitadel (0187) vs bespoke-Rust IDP hub (0394) vs oya-identity bespoke (0476)  **[SOURCE-INTERNAL TRIPLE CONTRADICTION — the sharpest unresolved fault in this theme]**

This is **three live, mutually-incompatible ADRs about the human/OIDC identity substrate**, none cleanly superseding the others, with a **dangling supersedes-pointer** and a **phantom parent set**.

**Positions (all verified on disk)**
- **ADR-0187** (`SOURCE/ADR-0187-canonical-oidc-idp-zitadel-primary.md`, **Accepted 2026-05-18**, `superseded_by: []`): **Zitadel v2.55+** (Go, Apache-2.0) is the canonical OIDC IdP fleet-wide; the *single issuer* of OIDC/SAML/SCIM/WebAuthn. **Explicitly rejects "Self-built IdP"** ("identity is undifferentiated heavy-lifting… zero competitive advantage"). Its in-house roadmap names a *future* `oya-identity-server` only behind concrete triggers (≥50K tenants/pack, p99>200ms, etc.).
- **ADR-0476** (`SOURCE/ADR-0476-oya-identity-bespoke-human-identity.md`, **Accepted 2026-05-28, founder-locked**): build **oya-identity**, a bespoke Rust-native OIDC provider NOW; `supersedes: [ADR-0421]` (Keycloak); **explicitly rejects Zitadel** in its Alternatives table ("Go-based; newer; smaller federation adoption; same Go-stack objection"). Identity is reframed as "a **product primitive, not a runtime dependency**" — the exact opposite of ADR-0187's thesis. Crypto cluster (0507/0508) binds to **0476, not 0187**.
- **ADR-0394** (`SOURCE/ADR-0394-bespoke-rust-idp-central-hub.md`, **Proposed 2026-05-29, founder-deciding, must-not-auto-merge**): the bespoke-Rust IDP *portal hub*; line 142 lists, as an **unsettled reconciliation pre-req**: *"OIDC issuer for IDP login (Zitadel ADR-0187 vs bespoke `oya-identity-oidc-issuer-kernel`)"* — and **does not cite ADR-0476 at all**, despite 0476 being Accepted/founder-locked **the day before** 0394.

**True contradiction?** **Yes — a hard one.** ADR-0187 (Accepted) and ADR-0476 (Accepted, founder-locked, later) make **opposite** build-vs-buy rulings on the same substrate and each **rejects the other's choice by name**. ADR-0187 is **not marked superseded** (`status: Accepted`, `superseded_by: []`) even though 0476 is later and founder-locked. ADR-0476 does **not** supersede 0187 — it supersedes ADR-0421 (Keycloak), which **does not exist on disk** (dangling supersedes-pointer), and its other parents **ADR-0406/0411/0416/0434 also do not exist on disk** (phantom citation set). ADR-0394 treats the question as still open. So the corpus simultaneously asserts: (a) Zitadel is canonical [0187 Accepted], (b) bespoke oya-identity is canonical and Zitadel is rejected [0476 Accepted founder-locked], (c) it's an open question [0394 Proposed].

**Which governs.** Under the keystone rule (latest + founder-locked wins over stale front-matter): **ADR-0476 (oya-identity bespoke, founder-locked 2026-05-28) is the governing intent for the identity *destination***, with **Zitadel (or Keycloak) as a Phase-1 bridge**. ADR-0187's Zitadel decision is, in substance, **demoted to the transitional bridge** that 0187's own in-house-roadmap section already anticipated — but the front-matter has not caught up. ADR-0394's "Zitadel vs bespoke" pre-req is **stale** the moment 0476 landed.

**Proposed resolution (surgical — NO new policy).**
1. **Mark ADR-0187 superseded-in-part:** set `superseded_by: [ADR-0476]` and a status note: *"Zitadel demoted from canonical destination to Phase-1 OIDC bridge per ADR-0476 (founder-locked oya-identity bespoke); 0187's Phase-2 `oya-identity-server` is realized as ADR-0476's oya-identity."* (This is a surgical front-matter edit reflecting the already-decided founder ruling — flagged here, not applied this pass.)
2. **Fix ADR-0476's dangling/phantom refs:** `supersedes: [ADR-0421]` points at a non-existent file, and parents 0406/0411/0416/0434 are absent. Either the Keycloak ADR is mis-numbered or never authored — must be reconciled before any generated-from-ADRs masterplan can build the supersession graph (this is the same number-reuse/dangling-ref disease the keystone flags for ADR-0055/0421).
3. **Retarget ADR-0394's pre-req:** line 142 "Zitadel ADR-0187 vs bespoke" → "oya-identity (ADR-0476, founder-locked) is the OIDC issuer; Zitadel is the Phase-1 bridge" — and add `ADR-0476` to 0394's `related`.
4. **Reconcile the two bespoke-identity surfaces:** ADR-0394 (IDP *portal/console hub*, Leptos) and ADR-0476 (oya-identity *OIDC server*) are **different layers** (console vs issuer) and are complementary, but both say "bespoke Rust identity" — a one-line disambiguation prevents synthesis logging them as a duplicate.

**Disposition impact:** ADR-0187 → **SUPERSEDE/AMEND** (mark Zitadel-as-bridge, `superseded_by:[0476]`). ADR-0476 → **KEEP/AMEND** (fix phantom supersedes/parents). ADR-0394 → **KEEP/AMEND** (retarget identity pre-req to 0476; still founder-decision-pending on its own portal merits).

**DECISION-NEEDED-FROM-FOUNDER ❓** *Confirm the identity end-state: **oya-identity (ADR-0476) bespoke Rust is canonical; Zitadel (ADR-0187) and Keycloak (ADR-0421) are Phase-1 bridges only** — yes/no? And: ADR-0476's `supersedes: [ADR-0421]` points at a file that does not exist — was Keycloak ever an ADR (mis-number), or should 0476 instead carry `supersedes: [ADR-0187]`? This ruling unblocks the entire identity substrate and the crypto cluster (0507/0508) that hangs off oya-identity.*

---

## T-4 — VECTOR / OLAP / OBJECT / KV substrate cluster: SOURCE best-of-breed vs LINUX "owned engine absorbs all"  **[CROSS-SIDE meta — reconcilable; SOURCE-internal coherent]**

**Positions**
- **SOURCE** assembles best-of-breed, each Accepted (or Proposed) and internally coherent: **Milvus** (ADR-0192, vector >10M; pgvector ≤10M; supersedes ADR-0046), **ClickHouse 26.3 LTS** (ADR-0193, OLAP compute), **TimescaleDB 2.26 CE** (ADR-0194, tenant time-series as a Postgres extension), **SeaweedFS→Ceph** (ADR-0196, object), **Valkey** (ADR-0336, KV/cache, Redis retired for license), **Iceberg** (ADR-0337, OLAP *table format*). Every one carries a "Phase-2 in-house replacement" roadmap behind value-anchored triggers (e.g. ADR-0192 §"Phase 2 — oya-vector-store-server").
- **LINUX ADR-0001** scopes the owned engine to include **vector search + full-text search as SQL access-methods** (stage-1) and **graph/GQL** (later) — i.e. it intends to *absorb* the vector tier (and FTS) into one multi-model engine, the exact workloads SOURCE assigns to Milvus / Meilisearch.

**True contradiction?** **No — overlapping-roadmap, not a flat contradiction.** Both sides explicitly hold an "own-when-proven" ladder: SOURCE ADR-0192 §"Phase 2" plans an `oya-vector-store-server` once Milvus is exercised at billion-scale; LINUX ADR-0001 plans vector/FTS inside cloud-data. They **agree on the destination (owned), disagree on the path/granularity** (one multi-model engine vs per-workload owned substrates). The cross-side tension is the same as T-1's: trigger threshold + whether ownership is one-engine or many-engines.

**SOURCE-internal note — Iceberg(0337) vs ClickHouse(0193) is NOT a contradiction.** Verified: ADR-0337 explicitly **layers** them (Iceberg = canonical OLAP *table-format write path*; ClickHouse = canonical *compute engine* layered on Iceberg via the iceberg engine) and `amends` dependency-policy §7 to split the row. The only defect is ADR-0193 carries no `amended_by: ADR-0337` back-edge (stale-front-matter drift, same class as the keystone §1.3 findings). Similarly ADR-0192/0193/0194 cite Pulsar/SeaweedFS/Postgres consistently.

**Which governs.** SOURCE's best-of-breed substrates govern **now** (they are mostly Accepted, 2026-05-18, and carry the canonical-posture-map §3 rows). LINUX ADR-0001's absorb-the-vector-tier ambition is a **later-stage owned target**, gated by the same ratchet — it does not displace Milvus/ClickHouse today.

**Proposed resolution (surgical).**
1. **CROSS-REF** LINUX ADR-0001 (and ADR-0020 staged-ownership) → name the SOURCE substrates its later stages would replace (Milvus 0192, ClickHouse 0193, Meilisearch 0184) so the "own when proven" trigger is explicit, not a silent overlap. (The prior LINUX pass already demoted these to "future ADR TBD" placeholders in ADR-0020 because the source numbers don't resolve in the LINUX corpus — that demotion is correct.)
2. **AMEND SOURCE ADR-0193** to add `amended_by: ADR-0337` (and the dependency-policy §7 split) — mechanical back-edge, no policy change.
3. **AMEND SOURCE ADR-0192** retired-vocab: owner `axis-foundry` + "owned by the `foundry` µservice" + Helm path `microservices/foundry/...` → `cloud-intelligence`/`intelligence` per ADR-0335/0347 (the vector store is an intelligence/RAG primitive). Same retired-`foundry` scrub for 0336/0337 owner lists.

**Disposition impact:** SOURCE ADR-0192 → **KEEP/AMEND** (foundry→intelligence vocab). ADR-0193 → **KEEP/AMEND** (add 0337 back-edge). ADR-0194/0196 → **KEEP**. ADR-0336 → **KEEP** (status `proposed`→bind; it's the canonical Redis→Valkey decision). ADR-0337 → **KEEP** (status `proposed`→bind). LINUX ADR-0001/0020 → **AMEND** (cross-ref the substrates the owned engine would later absorb).

**DECISION-NEEDED-FROM-FOUNDER ❓** *Is the owned `cloud-data` engine intended to absorb the vector tier and FTS (LINUX ADR-0001's "vector/FTS as SQL access-methods"), or do Milvus (ADR-0192) and the search backend stay as permanent separate best-of-breed substrates with their own owned-replacement roadmaps (ADR-0192 Phase-2 `oya-vector-store-server`)? One-multi-model-engine vs many-owned-substrates is a real architectural fork.*

---

## T-5 — CRYPTO cluster (aws-lc-rs 0506 / webauthn-rs 0507 / opensk 0508)  **[SOURCE-INTERNAL — coherent, masterplan-binding gap only]**

**Positions.** All three **Accepted 2026-05-28, founder + council-architecture, `door: two-way`, `planning_impact: false`**, and **internally consistent** under one "bespoke-over-OSS / hyperscaler-lens" doctrine:
- **ADR-0506** — `aws-lc-rs` is the canonical Phase-1 crypto provider (replaces `ring`); **oya-crypto** is the Tier-4 bespoke destination (gated on kubers Phase-B kernel proofs + FIPS 140-3).
- **ADR-0507** — `webauthn-rs` (MPL-2.0) is the canonical Phase-1 WebAuthn relying-party; **oya-webauthn** is the Tier-2 bespoke destination. Consumed via **oya-identity** (ADR-0476) `oya-identity-webauthn-*` crates.
- **ADR-0508** — **OpenSK** (Apache-2.0) is the canonical Phase-1 authenticator-side reference; **oya-authn-device** is the Tier-3 bespoke hardware destination (own-the-silicon, OpenTitan at Tier-4). Closed-loop with 0507.

**True contradiction?** **None internal.** The cluster is the cleanest in this theme: each carries a feature-parity table, a hyperscaler-lens pre-check, and explicit bridge/trigger gates. The only tensions are **dependencies on contested neighbors and binding gaps**:
1. **Hangs off ADR-0476, which is itself contested (T-3).** 0507 is "consumed via oya-identity (ADR-0476)"; if the founder rules Zitadel-canonical (reversing T-3), the webauthn-rs RP home shifts. The crypto cluster is **correct only if T-3 resolves toward oya-identity** — which it should, since 0506/0507/0508 and 0476 are the same founder-locked 2026-05-28 batch.
2. **Crypto vs identity RP overlap:** ADR-0187 (Zitadel) declares itself the "**WebAuthn relying party** (Level 3)" — but ADR-0507 makes **webauthn-rs** the canonical RP. If 0187 were still canonical, 0507 and 0187 would conflict on who owns WebAuthn RP. Resolving T-3 toward 0476/0507 removes this; it is another reason 0187 must be marked bridge-only.
3. **Masterplan binding:** all three are `planning_impact: false` despite being load-bearing security substrates — under either masterplan reading (authored-authority or generated-from-ADRs) the canonical crypto provider belongs in the masterplan. (Same 8.8%-binding finding.)

**Which governs.** The crypto cluster (0506/0507/0508) governs the crypto/WebAuthn substrate and is **consistent with the T-3 resolution toward oya-identity (0476)**. ADR-0187's "WebAuthn RP" claim is superseded by ADR-0507.

**Proposed resolution (surgical).**
1. **CROSS-REF** ADR-0507 ↔ ADR-0187: add to 0187 a note "WebAuthn relying-party is owned by webauthn-rs per ADR-0507 (not Zitadel)" — folds into the T-3 0187-demotion edit.
2. **AMEND** binding: flip `planning_impact` to `true` (or add `masterplan_ref`) on 0506/0507/0508 so the canonical crypto/RP/authenticator substrates are masterplan-visible. No policy change.
3. Otherwise **KEEP** all three verbatim — they are the model the corpus should imitate.

**Disposition impact:** ADR-0506/0507/0508 → **KEEP** (+ optional binding amend). Their correctness is **conditional on T-3 resolving toward oya-identity** — flag that dependency.

**DECISION-NEEDED-FROM-FOUNDER ❓** *(folds into T-3)* *Confirm the crypto/identity stack is the founder-locked 2026-05-28 bespoke-over-OSS line: aws-lc-rs→oya-crypto, webauthn-rs→oya-webauthn, OpenSK→oya-authn-device, all consumed by oya-identity (ADR-0476) — making ADR-0187's "Zitadel is the WebAuthn RP / canonical IdP" the demoted bridge. Yes/no?*

---

## T-6 — Identity OWN-vs-FRONT and the regulated-substrate framing  **[cross-cut of T-3, surfaced for the masterplan]**

ADR-0187 frames identity as "**undifferentiated heavy-lifting**, zero competitive advantage" (buy/front). ADR-0476 frames it as "**a product primitive, not a runtime dependency**" (own/build). This is the same own-vs-buy axis as T-1 (DB) and T-4 (vector) but on the **most-regulated substrate** (OIDC/SCIM/WebAuthn, sovereign-pack air-gap, KCMVP HSM per ADR-0187). The founder ruling on T-3 *is* the ruling on this framing. Surfaced separately because the masterplan's FD-001 (Tenant RBAC at production depth) sits directly on whichever identity substrate wins — the masterplan cannot record FD-001 as "done" while the IdP build-vs-buy is contradictory across 0187/0476/0394.

**No separate disposition** — resolves with T-3.

---

## T-7 — Storage license / retired-vocabulary hygiene (cross-cutting, mechanical)  **[AMEND batch, not contradictions]**

Verified residue across this theme's SOURCE ADRs (mechanical, surgical AMENDs; not architectural disputes):
- **Retired `foundry` brand** leaks into live storage ADRs: ADR-0192 (`axis-foundry` decider, "owned by the foundry µservice", `microservices/foundry/iac/helm/milvus/`), ADR-0045 (owner `foundry`, the "Foundry" axis row + per-axis table), ADR-0336/0337 owner lists. Live name = **cloud-intelligence + governance** per ADR-0335/0347 (founder: "cloud-intelligence is the valid name").
- **Retired `Redis`** → Valkey is the *correct* direction (ADR-0336), but ADR-0045's OLTP extension table and ADR-0252's lock-doctrine examples still name Redis/SETNX as live (0252 correctly says Valkey-SETNX in D-5 but also "Redis cluster" in the failure table — acceptable as counterpart-fact).
- **Fabricated-clean license claims** (data-integrity, founder "plain wrong" class): ADR-0045 asserts **Citus = Apache-2** while ADR-0184 records **Citus columnar = AGPL3**; ADR-0045 marks TimescaleDB "Apache-2 community ed" without the TSL fence that ADR-0194 correctly draws. These are license-bookkeeping defects that must be corrected before any OSI-strict license gate trusts 0045.
- **Stale status:** ADR-0045/0336/0337 read `status: proposed` while their decisions are canonical-posture-map §3 rows and are cited as live by Accepted ADRs — status-drift (the keystone §6 class). Under generated-from-ADRs this would generate a wrong masterplan; under authored-authority it leaves the substrate keystones unbound.

**Disposition impact:** ADR-0045 → **AMEND** (license correction + foundry-vocab + status). ADR-0192/0336/0337 → **KEEP/AMEND** (foundry→intelligence vocab; bind status). No contradictions here — pure hygiene that gates masterplan trust.

---

## 8. Summary of disposition changes driven by this theme's tensions

| ADR | Side | Prior disposition (sweep) | This-theme adjustment | Reason |
|---|---|---|---|---|
| LINUX ADR-0001 | LINUX | (pilot, Accepted) | **AMEND** | T-1: finish half-applied "eliminate→retain Postgres" reconciliation; cross-ref ADR-0045/0179 |
| LINUX ADR-0006 | LINUX | (pilot, Accepted) | **AMEND** | T-2: cross-ref ADR-0252; keep Clock-port seam for Tier-4 TrueTime |
| LINUX ADR-0020 | LINUX | (pilot) | **AMEND** | T-4: name the source substrates (Milvus/ClickHouse) the owned engine later absorbs |
| ADR-0045 | SOURCE | amend / truth=partial | **AMEND (confirmed)** | T-1/T-7: Citus-AGPL3 + TimescaleDB-TSL license corrections, foundry-vocab, status, pgcat boundary |
| ADR-0179 | SOURCE | keep | **KEEP** (+merge-boundary note) | T-1: pgcat survives only if Postgres retained; add `related: ADR-0001` |
| ADR-0187 | SOURCE | keep (canon §3) | **SUPERSEDE/AMEND** | T-3: Zitadel demoted to Phase-1 bridge; set `superseded_by:[ADR-0476]`; WebAuthn-RP→0507 |
| ADR-0192 | SOURCE | (canon §3) | **KEEP/AMEND** | T-4/T-7: add 0046-supersession is fine; foundry→intelligence vocab |
| ADR-0193 | SOURCE | (canon §3) | **KEEP/AMEND** | T-4: add `amended_by: ADR-0337` back-edge |
| ADR-0252 | SOURCE | (keystone #11) | **KEEP/AMEND** | T-2: bind status; light retired-vocab (Kafka/0005) |
| ADR-0336 | SOURCE | (Valkey canon) | **KEEP/AMEND** | T-4/T-7: bind status; foundry-vocab in owner list |
| ADR-0337 | SOURCE | (Iceberg canon) | **KEEP/AMEND** | T-4: bind status; NOT a contradiction with 0193 (layered) |
| ADR-0394 | SOURCE | decision-pending | **KEEP/AMEND** | T-3: retarget identity pre-req from "Zitadel vs bespoke" → oya-identity (0476) |
| ADR-0476 | SOURCE | keep (founder-locked) | **KEEP/AMEND** | T-3: fix dangling `supersedes:[ADR-0421]` + phantom parents 0406/0411/0416/0434 |
| ADR-0506/0507/0508 | SOURCE | keep | **KEEP** (+bind) | T-5: coherent; flip planning_impact; conditional on T-3 toward 0476 |

## 9. Founder questions (consolidated, crispest-first)

1. **Identity (T-3, highest):** Is **oya-identity (ADR-0476, bespoke Rust, founder-locked) canonical**, with **Zitadel (0187) + Keycloak (0421) as Phase-1 bridges only**? And does ADR-0476 supersede **ADR-0187** (currently it supersedes a non-existent ADR-0421)? — unblocks identity + crypto (0507/0508) + FD-001.
2. **Data tier (T-1):** End-state = "own the entire data tier, Postgres eventually retired" **or** "own only the cloud-data differentiator; Postgres+Citus is the permanent OLTP substrate of record"? (LINUX ADR-0001 was authored as the former, patched toward the latter.)
3. **Vector/multi-model (T-4):** Does cloud-data absorb the vector tier + FTS (LINUX ADR-0001), or do Milvus (0192) / search stay permanent separate substrates with their own owned-replacement roadmaps?
4. **Time/Tier-4 (T-2):** Must the owned engine run in Tier-4 external-consistency cells (financial/IL5)? If yes, ADR-0006 must keep a TrueTime Clock-adapter seam from day-0.
5. **Crypto (T-5, folds into T-3):** Confirm the founder-locked 2026-05-28 bespoke-over-OSS crypto line (aws-lc-rs→oya-crypto, webauthn-rs→oya-webauthn, OpenSK→oya-authn-device), which makes ADR-0187's "Zitadel is the WebAuthn RP" obsolete.

---
*End of theme register. Cross-side conflicts (T-1 data, T-2 time) are reconcilable via the shared own-when-proven ratchet — the open question is the trigger threshold. The one hard, currently-live contradiction in this theme is SOURCE-internal: the Zitadel/oya-identity/IDP-hub identity triple (T-3), where an Accepted ADR (0187) and a later founder-locked Accepted ADR (0476) make opposite build-vs-buy rulings and each rejects the other by name, with 0187 never marked superseded and 0476 resting on a dangling supersedes-pointer + phantom parents.*
