# ADR Audit — LINUX chunk 1

- side: LINUX (pilot / staging series, `~/Developer/linux`)
- chunk: linux-1
- range: ADR-0001 … ADR-0007
- ADRs reviewed: 7 (ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0007)
- auditor baseline: keystone canonical-posture-and-supersession-map.md (2026-06-06)
- cross-cutting note: ALL seven carry `supersedes:[] / superseded_by:[]` + `renumber_note` — they are a parallel pilot series that renumbers into SOURCE (0515+) on merge; all collide with existing SOURCE ADR-0001…0007 (keystone §6.4). None supersede a SOURCE ADR today.

---

### ADR-0001 — Distributed-database engine: from-scratch Rust multi-model substrate

- decision_atom: Source builds `distributed-database` (canonical `cloud-data`) — a from-scratch, Rust, hexagonal, multi-Raft distributed multi-model ENGINE (relational+KV+vector+FTS stage-1; native graph later) owning all DB-differentiating layers (LSM/MVCC/WAL/Raft/txn/range/PD/HLC/query) while reusing tokio/hyper/prost behind ports — as the owned differentiator layer that replaces etcd, NOT a replacement for the retained Postgres+Citus OLTP substrate.
- domain: data-engine-db
- current_status: Accepted (2026-06-05)
- disposition: AMEND
- proposed_resolution: NA (not Proposed)
- governing: —
- truth_flag: PARTIAL — core decision is TRUE and internally honest, but body carries STALE retired vocab: "M0–M6" milestone naming (retired per keystone §2 / GLOSSARY L250 → Wave names) and a "Tier — Tier-1 (strategic)" label that risks conflation with the retired tier-system (ADR-0329) though here it means decision-priority not tenant-class.
- in_masterplan: PARTIAL — the owned-distributed-DB differentiator and "Postgres+Citus retained" clarification align with SOURCE canonical posture (keystone §3 data tier names Postgres+pgcat as reused OLTP); but SOURCE's data canon is *best-of-breed assembled* (Milvus/SeaweedFS/ClickHouse/TimescaleDB), which the masterplan does not currently represent as "build a from-scratch Spanner-class engine."
- tensions: keystone §5 fault-line #1 (LINUX owned-DB vs SOURCE best-of-breed) and §1's "sharpest unflagged conflict": the original title/thesis framing "eliminate PostgreSQL" — though the ADR body has been amended with an explicit clarification note that Postgres+Citus is RETAINED behind pg-wire and this ADR owns only the differentiator. The residual tension is scope/ownership (build a Spanner-class engine) vs SOURCE assembling proven OSS substrates; also LINUX ADR-0020 flags Milvus as an unsafe deferral. Naming drift: ADR uses pilot prefix `oya-distributed-database-*` but acknowledges canonical `oya-cloud-data-distributed-*`.
- hyperscaler_challenge: aligned-but-questionable. Google/AWS/Azure DO own core data substrates (Spanner, Aurora, Cosmos) — so owning the differentiator is hyperscaler-aligned in principle. BUT no hyperscaler builds a from-scratch multi-model engine to *avoid* an external OLTP dependency; they keep best-of-breed for commodity OLTP and own only where differentiation pays. Implication: AMEND to keep the (already-present) "Postgres retained" framing as the headline, retire the "eliminate external DB dependency" motivation language, and reconcile day-0 ownership ambition against SOURCE's staged-ownership ratchet (ADR-0019/0211).
- ai_slop: NO — self-aware, post-challenge, carries explicit clarification notes correcting earlier framing.
- refinement: retitle/reframe away from "eliminate PostgreSQL"; replace M0–M6 with Wave naming; rename "Tier-1" priority label to avoid tenant-class collision; adopt canonical `cloud-data` name in the headline not just footnotes.
- consensus_needed: Founder question — Is a from-scratch Spanner-class owned engine IN the masterplan as a committed differentiator, or is SOURCE's best-of-breed assembly (Milvus/ClickHouse/Postgres+pgcat) the canonical data tier with this engine deferred to "own-when-proven"? (decides KEEP-as-strategic vs AMEND-to-research-track.)

---

### ADR-0002 — Hexagonal ports-and-adapters with no_std kernels + source-compatible structure

- decision_atom: The distributed-database engine is structured as hexagonal ports-and-adapters with `#![no_std]+alloc` pure-domain kernels over five canonical ports (Clock/Network/Storage/WAL/RNG), std-only adapters (production tokio + deterministic-sim + test), and crate naming that merges into SOURCE `cloud/cloud-data/` unchanged as a file-move + rename.
- domain: data-engine-db
- current_status: Accepted (2026-06-05)
- disposition: KEEP (with light AMEND)
- proposed_resolution: NA
- governing: —
- truth_flag: TRUE — architecturally sound, hyperscaler-grounded, consistent with ADR-0001/0003/0004; minor STALE element: M1/M2/M3 milestone labels (same retired-milestone-vocab caveat as ADR-0001, though here M1–M3 read as verification *stages* not the retired M0–M3 product milestones).
- in_masterplan: PARTIAL — pure architecture/structure decision with no direct masterplan posture entry; it is a sound implementation pattern that survives regardless of the §4 authored-vs-generated SSOT question. Source-compat merge target (`cloud/cloud-data/`) aligns with component-boundaries.
- tensions: none material internally. Inherits ADR-0001's owned-DB-vs-best-of-breed scope tension only by association. no_std-kernel discipline aligns with SOURCE's no_std core-library posture (keystone §5 verdict notes LINUX edits are coherent).
- hyperscaler_challenge: aligned. Ports-and-adapters + deterministic-sim-first (FoundationDB method) + own-kernel/reuse-commodity-I-O is exactly the CockroachDB/TiKV/FoundationDB pattern the ADR cites. No hyperscaler would object to this structure; it is the lowest-controversy ADR in the chunk.
- ai_slop: NO — substantive, code-illustrated, precedent-cited.
- refinement: swap milestone letters (M1–M3) for the canonical Wave/stage naming to avoid retired-vocab leakage; otherwise leave intact.
- consensus_needed: none (contingent only on ADR-0001's strategic go/no-go).

---

### ADR-0003 — Dependency posture: own DB-differentiating layers + gRPC framing; reuse tokio/hyper/prost behind ports

- decision_atom: The engine adopts a tiered dependency-ownership model — Tier-1 OWN (LSM/MVCC/WAL/Raft/txn/range/PD/HLC/query + gRPC message-framing semantics + sim + consistency-checker), Tier-2 REUSE permanently behind ports (tokio/hyper-h2/prost, annually audited), Tier-3 provisional→owned ratchet for the tonic gRPC adapter (benchmark-gated reopen if >5% hot-path CPU).
- domain: data-engine-db
- current_status: accepted (2026-06-05)
- disposition: KEEP
- proposed_resolution: NA
- governing: —
- truth_flag: TRUE — coherent, hyperscaler-mapped (Camp A/B/C taxonomy), and consistent with the universal-port-ratchet philosophy that keystone §5 notes LINUX (ADR-0019/0020) and SOURCE (ADR-0211/0173) BOTH share. Minor staleness: front-matter `status: accepted` lowercased (cosmetic drift vs sibling `Accepted`); says crates live in `libs/` while ADR-0002 corrects destination to `cloud/cloud-data/` (internal cross-ref drift).
- in_masterplan: PARTIAL — dependency-ownership ratchet principle is shared canon (own-when-proven), so it aligns; the specific Tier-1/2/3 list is engine-local detail not separately represented in masterplan.
- tensions: minor internal drift with ADR-0002 on landing dir (`libs/` vs `cloud/cloud-data/`) — ADR-0002 is the more-corrected statement. Substantive agreement with SOURCE ratchet posture (keystone §5: disagreement is trigger-threshold, not principle).
- hyperscaler_challenge: aligned. "Own what differentiates, reuse commodity I/O, ratchet ownership on benchmark" is precisely how hyperscaler data teams reason (the ADR's own Camp A/B/C evidence). No hyperscaler would reject this posture.
- ai_slop: NO.
- refinement: normalize `status:` capitalization to `Accepted`; fix the `libs/` reference to `cloud/cloud-data/` to match ADR-0002 §7; otherwise KEEP verbatim.
- consensus_needed: none.

---

### ADR-0004 — Distributed by design + bottom-up simulator-first verification

- decision_atom: Distributed architecture is embedded in day-1 design (single-node bedrock is the same kernel as multi-node; replication reaches into the WAL; ranges are a rebalancing boundary), and verification is simulator-first + bottom-up across M0–M6 milestones — owning the fault-injection harness, reusing the ELLE serializability checker, gating every layer on ≥100K-seed deterministic soak with zero violations.
- domain: data-engine-db
- current_status: Accepted (2026-06-05)
- disposition: AMEND
- proposed_resolution: NA
- governing: —
- truth_flag: PARTIAL — verification methodology (simulator-first, own-harness/reuse-ELLE, bottom-up gating) is TRUE and excellent; but the ADR is the heaviest user of retired M0–M6 milestone vocabulary (keystone §2: M0–M3/Milestone/MVP RETIRED 2026-05-09 → Wave names). The full M0…M6 bedrock plan must be re-expressed in Wave/stage naming.
- in_masterplan: PARTIAL — methodology (deterministic-sim moat, ELLE reuse) is a sound process decision; it does not occupy a top-level masterplan domain row but underpins ADR-0001's deliverability. Milestone scaffolding conflicts with current Wave-naming canon.
- tensions: no architectural tension with SOURCE (verification rigor is universally welcome). Overlaps heavily with ADR-0001/0002/0005/0006 (same M-milestones, same HLC/WAL/Raft seams) — candidate for MERGE-by-reference rather than restating bedrock milestones in four ADRs (see Chunk notes). The §5 owned-DB scope question is inherited, not introduced here.
- hyperscaler_challenge: aligned. Deterministic simulation as the first deliverable (FoundationDB/TigerBeetle doctrine) is exactly what hyperscaler-grade DB teams do; AWS/Google/Azure would endorse simulator-first bottom-up verification without reservation.
- ai_slop: NO — dense, precedent-anchored, honest about cost (negative consequences enumerated).
- refinement: replace every M0–M6 label with canonical Wave/stage naming; deduplicate the bedrock-milestone table that is restated across ADR-0001/0002/0004/0005 (point to one canonical milestone spec, `distributed-database-engine-canonical.json`).
- consensus_needed: none on method; only the inherited ADR-0001 strategic question.

---

### ADR-0005 — Storage: local shared-nothing LSM for stage-1, disaggregation as explicit later stage

- decision_atom: Stage-1 bedrock uses an owned shared-nothing local LSM with the replication/durability seam drawn at the WAL+Raft boundary (NOT the LSM), so that disaggregation (compute/storage separation, Aurora-style quorum-log) is a planned later protocol-level evolution — explicitly NOT a storage-engine swap — with horizontal write-scale coming from range-sharding+multi-Raft, not the single-node LSM.
- domain: data-engine-db, data-storage
- current_status: Accepted (2026-06-05)
- disposition: AMEND
- proposed_resolution: NA
- governing: —
- truth_flag: PARTIAL — the architectural decision (WAL/Raft seam, disaggregation-is-protocol-not-swap, LSM-becomes-cache) is TRUE and notably self-correcting (it explicitly refutes two false claims). Staleness: "M04–M05 horizon" and stage-2/stage-3 milestone vocab; one citation reliability concern — it attributes "Socrates (Meta)" and "TaurusDB (Alibaba) … VLDB 2024 SQL-Native Time Series" which are MISATTRIBUTED (Socrates is Microsoft SQL Server's cloud engine, SIGMOD 2019; TaurusDB is Huawei, a MySQL-compatible cloud DB — not Alibaba, not a time-series DB). These bad references should be corrected.
- in_masterplan: PARTIAL — disaggregation-readiness is an engine-internal design constraint, not a separately-tracked masterplan row; depends on ADR-0001 being in-scope.
- tensions: heavy restatement overlap with ADR-0001/0002/0004 on the WAL/Raft seam and disaggregation framing (MERGE candidate — this is the canonical storage-seam ADR, the others should cite it). Note this LINUX ADR-0005 is unrelated to SOURCE ADR-0005 (Kafka/outbox eventing, retired by ADR-0377) — pure number collision, no semantic link.
- hyperscaler_challenge: aligned. "Local LSM first, draw the durability seam at the log, disaggregate later (Aurora/Socrates/Spanner pattern)" is the actual hyperscaler evolution path; AWS/Google lived this exact ladder. The decision is well-aligned; only the supporting citations are wrong.
- ai_slop: PARTIAL — content is strong and self-critical, but the misattributed Socrates/TaurusDB references are a factual-accuracy slop signal that must be fixed (a reviewer would catch these).
- refinement: correct Socrates (Microsoft, SIGMOD 2019) and TaurusDB (Huawei, MySQL-compatible, VLDB 2020) attributions; replace stage/M04–M05 milestone vocab with Wave naming; designate this as the canonical storage-seam ADR and have ADR-0001/0002/0004 reference it instead of restating.
- consensus_needed: none beyond inherited ADR-0001 question; flag the citation-accuracy fix to the founder as a quality gate.

---

### ADR-0006 — HLC consistency model & clock: plain serializable (not strict-serializable), no central TSO

- decision_atom: Stage-1 bedrock locks Hybrid Logical Clocks + uncertainty-restart (CockroachDB model) with NO central TSO, targeting PLAIN serializable isolation — explicitly NOT strict-serializable/linearizable/external-consistency, which is a deferred later-stage (TrueTime-class) opt-in.
- domain: data-engine-db
- current_status: Accepted (2026-06-05)
- disposition: KEEP (with light AMEND)
- proposed_resolution: NA
- governing: — (this ADR itself supersedes an *earlier internal framing* "TSO-first, HLC-stage-4" per its own body, but that framing is not a separate live ADR, so no `superseded_by` edge is needed)
- truth_flag: TRUE — consistency decision is sound, hyperscaler-mapped, and the honest non-goal (HLC ≠ linearizable) is consistent across ADR-0001/0002/0004/0006. Minor staleness: "stage 4 / stage 1" milestone vocab and "Tier — A (stage 1)" label.
- in_masterplan: PARTIAL — engine-internal consistency choice; not a top-level masterplan domain row, underpins ADR-0001 deliverability.
- tensions: none with SOURCE (no SOURCE ADR mandates a competing consistency model for an owned engine). Internal consistency with ADR-0001/0002/0004 is clean (all four state the same HLC non-goal). LINUX ADR-0006 vs SOURCE ADR-0006 is a pure number collision (unrelated topics).
- hyperscaler_challenge: aligned. HLC+uncertainty (CockroachDB/TiKV) vs TSO (Spanner/FDB) is a legitimate, well-understood hyperscaler design fork; choosing HLC-no-TSO for stage-1 with TrueTime deferred is a defensible, mainstream call AWS/Google engineers would accept.
- ai_slop: NO.
- refinement: replace "stage 1 / stage 4" and "Tier — A" labels with Wave/stage naming; otherwise KEEP.
- consensus_needed: none.

---

### ADR-0007 — gRPC transport: provisional tonic, own framing later

- decision_atom: The engine ships a provisional tonic `RpcTransport` adapter NOW for wire-compatible gRPC over HTTP/2, while OWNING the gRPC message-framing kernel immediately, and schedules ownership of HTTP/2 transport + async dispatch as a benchmark-gated ratchet (proven per ADR-0019's burn-in bar, not a one-shot benchmark).
- domain: data-engine-db, api-contracts
- current_status: Accepted (2026-06-05)
- disposition: AMEND
- proposed_resolution: NA
- governing: —
- truth_flag: PARTIAL — decision is TRUE and is essentially the gRPC-specific instance of ADR-0003's Tier-3 ladder. Naming staleness: this ADR alone uses the retired `oya-spanner-rpc-*` crate prefix ("spanner" codename) — ADR-0001 explicitly RETIRES "spanner" in favor of `distributed-database`/`cloud-data`. This is the clearest retired-vocab leak in the chunk.
- in_masterplan: PARTIAL — transport-ownership ratchet detail; aligns with the shared own-when-proven ratchet; not a standalone masterplan row.
- tensions: REDUNDANCY/MERGE tension with ADR-0003 (ADR-0003 §Tier-3 already decides "provisional tonic, own framing later, 5% gate"; ADR-0007 re-decides the same thing at finer grain). Candidate to MERGE into or be explicitly subordinated to ADR-0003. Naming tension with ADR-0001 (`spanner` retired). Correctly defers the "PROVEN" definition to ADR-0019 (good cross-ref discipline).
- hyperscaler_challenge: aligned. "Don't own HTTP/2+gRPC together; reuse a proven binding, own the semantic framing, swap the transport only if a benchmark proves tail-latency gain" matches TiKV/CockroachDB/FoundationDB practice exactly. No hyperscaler builds its own HTTP/2 stack without a measured reason — the ADR's gate enforces that discipline.
- ai_slop: NO — but the stale `oya-spanner-*` naming is a quality blemish.
- refinement: rename all `oya-spanner-rpc-*` → `oya-distributed-database-rpc-*` (pilot) / `oya-cloud-data-distributed-rpc-*` (destination) to retire the "spanner" codename per ADR-0001; consider MERGE/subordinate under ADR-0003 to remove the duplicate Tier-3 decision.
- consensus_needed: none of substance; resolve the MERGE-vs-keep-separate editorial question (ADR-0003 Tier-3 vs standalone ADR-0007).

---

## Chunk notes

- **Coherent, honest, deliberately divergent — not slop.** All 7 ADRs are the LINUX distributed-database (Spanner-class) pilot bedrock. They are internally consistent, hyperscaler-grounded, and self-critical (ADR-0005 refutes its own false claims; ADR-0006 supersedes an earlier internal framing; ADR-0001 carries an explicit Postgres-retained clarification). This matches keystone §5's verdict that the LINUX auto-reconciliation is "NOT plain wrong" — these are genuine architecture tensions to surface, not reconciliation bugs.

- **One strategic question dominates the whole chunk (keystone fault-line #1).** ADR-0001 commits to a from-scratch owned multi-model engine; ADRs 0002–0007 are all implementation detail UNDER that commitment. The single founder decision — *is the owned Spanner-class engine a committed masterplan differentiator, or is SOURCE's best-of-breed assembly (Milvus/SeaweedFS/ClickHouse/TimescaleDB/Postgres+pgcat) the canonical data tier with this engine deferred to own-when-proven?* — determines whether this entire cluster is KEEP-strategic or AMEND-to-research-track. ADR-0001's body has already softened the sharpest conflict (it now says Postgres+Citus is RETAINED, engine owns only the differentiator), so the remaining gap is scope-ambition, not a flat contradiction.

- **Pervasive retired-vocabulary leakage (the dominant AMEND driver).** Retired M0–M6 / Milestone / stage-N vocab (keystone §2, RETIRED 2026-05-09 → Wave names) appears in ADR-0001/0002/0004/0005/0006; the retired "spanner" codename survives in ADR-0007's `oya-spanner-rpc-*` crate names despite ADR-0001 retiring it; pilot prefix `oya-distributed-database-*` everywhere vs canonical `oya-cloud-data-distributed-*` (acknowledged in 0001/0002 footnotes only). None of this is wrong-decision; all of it is naming/milestone drift → AMEND, not ARCHIVE.

- **Heavy restatement → MERGE/canonicalize opportunity.** The WAL/Raft durability seam, the HLC non-goal, and the M0–M6 bedrock milestone table are each restated across 3–4 of these ADRs. Recommend designating canonical owners (ADR-0005 = storage seam, ADR-0006 = HLC, ADR-0004 = verification milestones, ADR-0003 = dependency ladder) and having siblings cite rather than restate. ADR-0007 is a finer-grained duplicate of ADR-0003's Tier-3 gRPC ladder — strongest single MERGE candidate.

- **One factual-accuracy fix (ADR-0005).** Misattributed references: "Socrates (Meta)" is actually Microsoft (Azure SQL DB Hyperscale, SIGMOD 2019); "TaurusDB (Alibaba) … SQL-Native Time Series, VLDB 2024" is actually Huawei, a MySQL-compatible cloud DB (VLDB 2020), not a time-series DB. Only quality issue rising to a reviewer-blocking signal in the chunk.

- **No ARCHIVE/SUPERSEDE/DROP in this chunk; no Proposed ADRs.** All 7 are Accepted and live; dispositions are KEEP (0002, 0003, 0006) or AMEND (0001, 0004, 0005, 0007) — every AMEND is naming/milestone/citation/merge hygiene, never a wrong decision. No unaccounted proposals exist.

- **Merge collisions (keystone §6.4).** LINUX 0001–0007 each collide with SOURCE 0001–0007 (e.g., LINUX-0001 distributed-DB vs SOURCE-0001 foundation; LINUX-0005 storage-LSM vs SOURCE-0005 Kafka-outbox; LINUX-0007 gRPC vs SOURCE-0007). Renumber the whole pilot series to 0515+ on merge; never merge at face value.
