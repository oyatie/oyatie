# ADR Audit — LINUX chunk 2

- **side:** LINUX (pilot / staging series, `~/Developer/linux/docs/decisions/`)
- **chunk:** linux-2 (slice 8–14 of the sorted ADR list)
- **range:** ADR-0008 → ADR-0014
- **ADRs reviewed:** 7 (ADR-0008, ADR-0009, ADR-0010, ADR-0011, ADR-0012, ADR-0013, ADR-0014)
- **auditor baseline:** keystone map `canonical-posture-and-supersession-map.md` (all linux ADRs are a parallel staging series carrying `supersedes:[] / superseded_by:[]` + `renumber_note`; none supersede source ADRs; guaranteed number-collision on merge — keystone §6.4)

---

### ADR-0008 — Verification: own the deterministic simulator, reuse Elle for consistency checking

- **decision_atom:** For the distributed-database engine, OWN a single-threaded seed-replayable deterministic simulator + fault-injection (the literal first deliverable, "verification leads"), and REUSE Elle as the serializability checker behind a thin Rust↔Elle bridge, gated by a ≥100k-seed soak with zero violations and permanent-regression seed capture.
- **domain:** data-engine-db (cross-cut: ci-cd-build — the soak gate is a CI conformance lane)
- **current_status:** accepted (2026-06-05)
- **disposition:** KEEP
- **proposed_resolution:** NA (status=accepted, not Proposed)
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** PARTIAL — verification-leads + own-sim/reuse-checker is a crisp masterplan-ready atom for the DB-engine vertical; the Elle/Clojure-bridge dependency is a concrete spec detail not yet surfaced as a top-level masterplan invariant.
- **tensions:** Reuses Elle, which pulls a Clojure/JVM toolchain into a Rust-first repo (the ADR itself rejects full Jepsen partly on that ground, then accepts Elle which has the same JVM cost — a mild internal tension, scoped to a bridge rather than end-to-end Jepsen). Coheres with keystone fault-line #5 ("own when proven" ratchet): checker stays reused, simulator owned. No conflict with SOURCE — SOURCE has no competing DB-verification ADR in the keystone canonical-posture table; this is pilot-original.
- **hyperscaler_challenge:** aligned. Google/AWS/Azure all do exactly this — FoundationDB/TigerBeetle own the deterministic simulator; nobody reimplements the consistency checker. Strongly hyperscaler-validated; no amend/archive implication.
- **ai_slop:** none — citations (FoundationDB sim, Elle VLDB 2020, madsim/mysten-sim, TigerBeetle VOPR) are real and load-bearing.
- **refinement:** State the JVM/Clojure runtime dependency of the Elle bridge as an explicit accepted cost (and whether it is allowed in the OSI/license posture lane), so it is not read as a pure-Rust claim.
- **consensus_needed:** None contested. (Optional founder question: is a permanent JVM build-dep in the verification critical path acceptable, or should the bridge target a non-JVM Elle reimplementation later? The ADR already rejects reimplementation, so this is a confirm-not-relitigate.)

---

### ADR-0009 — Kernel target: the complete Linux syscall ABI (all system calls)

- **decision_atom:** The framekernel commits to the COMPLETE Linux userspace syscall ABI (≈350 x86_64 / ≈295 aarch64 now; riscv64 later) as the one hard external invariant, reached by differential-tested incremental coverage (`-ENOSYS` = demand signal; LTP/strace conformance; per-arch coverage matrix; zero-unsafe safe-kernel TCB held), never a big-bang.
- **domain:** kernel-frame (cross-cut: api-contracts — the Linux ABI is the published inter-component contract)
- **current_status:** accepted (2026-06-05)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — "Linux syscall ABI = THE invariant, differential-tested, incremental to full" is a keystone masterplan-grade decision and matches the founder directive quoted in-body ("must support all System Call Support. Full Linux ABI.").
- **tensions:** None internal-to-linux. Cross-side: this is the seam that the whole LINUX-vs-SOURCE differential-testing coupling rests on (MEMORY: "coupled by the Linux syscall ABI + differential testing"). No SOURCE ADR contradicts; SOURCE consumes the ABI rather than defining the kernel. Honest-scope language ("multi-year program") is consistent with keystone's read of the linux series as deliberately divergent but self-aware.
- **hyperscaler_challenge:** aligned (with a caveat). gVisor (Google) is the exact precedent and is cited; differential/test-driven incremental coverage is the hyperscaler-proven method. The caveat a hyperscaler would press: "complete ABI on your OWN kernel" is a multi-year cost most would avoid unless the kernel ownership itself is justified (that justification lives in ADR-0001/0025, not here). aligned on method; the breadth bet is inherited from the own-the-kernel thesis. No amend implication for THIS ADR.
- **ai_slop:** none — syscall counts, gVisor (~260+), Asterinas, LTP all real.
- **refinement:** "Complete Linux ABI" should carry a coverage-matrix pointer as its truth-anchor in the masterplan so the claim is never read ahead of the differential evidence (the ADR already disciplines this; masterplan should inherit the discipline).
- **consensus_needed:** None. The scope is large but the decision is internally honest and directive-backed.

---

### ADR-0010 — Build order: vertical compatibility slices, run-on-Linux-first

- **decision_atom:** Build the five pilot components (kernel, node-OS, kubernetes, container-platform, db-engine) NOT in parallel-to-completion but in thin vertical compatibility slices, developing every user-space component to run on real Linux FIRST then re-host unchanged on the Rust kernel as ABI coverage reaches each slice's syscall set (8-phase ladder, `-ENOSYS` demand signal, differential oracle as the gate).
- **domain:** governance-process (cross-cut: kernel-frame — sequencing is anchored on the ABI-coverage climb)
- **current_status:** accepted (2026-06-06)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — the phase ladder (Phase 0 contracts/harness → … → Phase 7 global/cellular) and the "run-on-Linux-first, re-host unchanged" build-order rule are exactly masterplan sequencing material; this ADR is one of the most masterplan-shaped in the chunk.
- **tensions:** References `ADR-0017` (containerd rename) and `ADR-0023` indirectly via the component set — those are out-of-chunk and must be confirmed live (keystone does not flag them as retired; treat as live unless a sibling chunk archives them). The phase ladder's "Phase 4 minimal Kubernetes cell" + "Phase 7 cellular" pre-commits to the cellular model that ADR-0012 formalizes — coherent forward-reference, not a conflict. No SOURCE contradiction: SOURCE's own "develop on stock Linux first" precedent (gVisor/runc/CockroachDB) is the same instinct.
- **hyperscaler_challenge:** aligned. Build-on-stock-Linux-then-rehost via a stable ABI seam, vertical slices over big-bang, is textbook hyperscaler delivery discipline (Google/AWS both do staged vertical slices with conformance oracles). No amend/archive implication.
- **ai_slop:** none — gVisor/runc/containerd/CockroachDB precedent is accurate; phase ladder is concrete and self-consistent.
- **refinement:** The component count ("five components") and the containerd extraction (ADR-0017) are stated as settled; masterplan should carry the component-boundaries.md contract as the SSOT pointer (the ADR already cites it). Minor: "north-star milestone" language is wave-like; ensure it does not reintroduce retired M0–M3/MVP vocabulary (keystone §2) — it currently uses "Phase 0–7", which is clean.
- **consensus_needed:** None contested.

---

### ADR-0011 — Kernel exposes general high-performance primitives; database-capable, never database-aware

- **decision_atom:** The kernel provides only GENERAL, product-agnostic high-performance mechanisms (huge pages, NUMA, io_uring-family rings, O_DIRECT, fsync/WAL durability ordering, cgroup-v2, eBPF/XDP, PMU) and is database-/container-CAPABLE but never database-/product-AWARE — all product policy (DB WAL layout, k8s admission/scheduling) lives in user space over the standard Linux ABI; this constrains ADR-0009's ABI surface to general syscalls only.
- **domain:** kernel-frame (cross-cut: governance-process — it is a binding boundary/review rule on ADR-0009's surface)
- **current_status:** accepted (2026-06-06)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — "mechanism, not product policy; database-capable not database-aware" is a keystone architectural invariant (the user's "most important architectural correction" per the body) and belongs verbatim as a masterplan boundary rule.
- **tensions:** None. This ADR is the guardrail that keeps ADR-0009 (full ABI) and ADR-0001 (own DB) from collapsing into a co-designed DB-aware kernel; it actively resolves a latent tension rather than creating one. Coheres with keystone fault-line #3 (own-the-host/kernel) by keeping the owned kernel a clean general substrate. No SOURCE conflict.
- **hyperscaler_challenge:** aligned. The mechanism-not-policy separation (io_uring, huge pages, NUMA, cgroup-v2, eBPF/XDP as general primitives consumed from user space) is exactly the Linux/hyperscaler doctrine; Google/AWS/Azure would make this decision without hesitation. No amend/archive implication.
- **ai_slop:** none — every primitive named is a real, general Linux mechanism; no fabricated "Spanner-aware" surface is proposed (it is explicitly rejected).
- **refinement:** None substantive. Could cross-link to ADR-0014's "container-capable never container-aware" restatement so the masterplan carries one general principle, not two near-duplicate ones.
- **consensus_needed:** None.

---

### ADR-0012 — "Scale to extreme" = cellular scale-out with bounded blast radius, not one infinite cluster

- **decision_atom:** "Scale to extreme" is defined as CELLULAR scale-out with bounded blast radius — the cell is the unit of failure (a bounded k8s-compatible control plane within documented limits ~5k nodes/~150k pods + a bounded DB placement group), with regional/global control planes doing only narrow placement + must-be-global coordination (identity, billing, critical metadata, promised cross-cell txns), most fleet ops async/idempotent/retry-safe, and DB write-scale via range-sharding + multi-Raft with no single-leader ceiling (PD = topology authority not state leader; HLC = clock authority, no central TSO).
- **domain:** orchestration-scheduling (cross-cut: data-engine-db — defines DB scale-out shape too)
- **current_status:** accepted (2026-06-06)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — the cellular-scale-out / bounded-blast-radius definition (and the "what must be global must be justified" rule) is a high-level masterplan posture for both the orchestration and DB domains.
- **tensions:** Conceptual overlap with SOURCE's `cell` vocabulary: keystone §2 retired SOURCE's `cell` *as a microservice* (ADR-0333) but kept `cell` *as a deployment pattern* — LINUX ADR-0012 uses `cell` strictly as the deployment/failure-domain pattern, so it is ALIGNED with the surviving meaning, NOT the retired one. Flag for merge: ensure the linux "cell" reads against ADR-0333's pattern-definition, not the dead µsvc. Internally pre-committed by ADR-0010's Phase-4/Phase-7. No hard conflict.
- **hyperscaler_challenge:** aligned. Cell-based architecture / bounded blast radius is the canonical AWS-cells / Google-isolation-domain pattern; the k8s ~5k-node documented limit is real; "no infinite cluster" is exactly how hyperscalers operate. Strongly aligned; no amend/archive implication.
- **ai_slop:** none — k8s large-cluster limits (~5k nodes / ~150k pods / ~300k containers / ~110 pods/node) are accurate; AWS-cells / Spanner-locality precedent is real.
- **refinement:** Cross-reference SOURCE ADR-0333 ("cell = pattern, not service") at merge so the two "cell" usages are explicitly reconciled and the retired-µsvc meaning is not accidentally reintroduced.
- **consensus_needed:** None contested. (Watch-item only: the masterplan must name the exact set of "sanctioned global surfaces" so this stays a closed list.)

---

### ADR-0013 — Modern-only hardware posture (cloud/server-first, purpose-built device model)

- **decision_atom:** The kernel adopts a binding modern-only, cloud/server-first hardware posture — first-class: x86_64+aarch64 (riscv64 later), UEFI/PVH boot, x2APIC/GICv3 + modern per-CPU timers, virtio/NVMe/PCIe, SR-IOV/IOMMU, TPM 2.0 + SEV-SNP/TDX/Arm-CCA readiness, CXL-as-NUMA later; EXCLUDED (unless a paying deployment funds it): legacy BIOS, VGA, PS/2, floppy, IDE, ancient NICs, desktop power-mgmt — with binding timekeeping (never trust raw RDTSC; invariant-TSC detect + pvclock + sync + suspend/resume + clocksource fallback) and boot-time NUMA-topology parse constraints.
- **domain:** hardware-firmware (cross-cut: security-supplychain — the small-attack-surface/Firecracker-Nitro rationale is a TCB-minimization posture)
- **current_status:** accepted (2026-06-06)
- **disposition:** KEEP
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE
- **in_masterplan:** YES — "modern-only, purpose-built device model (Firecracker/Nitro), legacy is a non-goal" is a clean masterplan hardware-posture invariant with an explicit enforced exclusion list; the timekeeping + NUMA binding constraints are spec-level but masterplan-anchorable.
- **tensions:** None internal. The posture is consistent with ADR-0014 (which cites it for the microvm device model) and ADR-0009 (the ABI sits above this hardware floor). No SOURCE conflict — SOURCE's Talos node-OS (ADR-0375) is also modern-server-first; this is harmonious with the SOURCE orchestration posture even though linux owns its own kernel (keystone fault-line #3 is about own-vs-assemble, not about hardware breadth).
- **hyperscaler_challenge:** aligned. Firecracker/Nitro minimal purpose-built device model is literally the AWS posture; modern-only + confidential-compute readiness is exactly Google/AWS/Azure cloud-host doctrine. The one thing a hyperscaler might push: even AWS keeps a tiny legacy-boot leg for some images — but the ADR's "reversible per-device only when a paying deployment funds it" already absorbs that. aligned; no amend/archive implication.
- **ai_slop:** none — x2APIC/GICv3/virtio/NVMe/SEV-SNP/TDX/CCA/CXL and the RDTSC-instability warnings are all technically accurate and load-bearing.
- **refinement:** None substantive. The roadmap §-references (§C8, §P4·1, §P3.3, §6.6) should be confirmed to still exist on disk at masterplan-bind time (drift risk if ROADMAP.md is restructured) — flagged as a citation-freshness item, not a correctness issue.
- **consensus_needed:** None.

---

### ADR-0014 — Container runtime: one OCI/CRI frontend + pluggable IsolationBackend port (native/sandbox/microvm/confidential)

- **decision_atom:** The container runtime is ONE OCI/CRI front-end driving a single owned `no_std` `IsolationBackend` port whose four adapters span the isolation spectrum — native (owned runc-class, ships first), microvm (owned VMM-or-reused-then-owned behind a `Vmm` port + framekernel-guest), sandbox (framekernel-as-userspace-Sentry, requires a multi-quarter re-architecture), confidential (microvm + SEV-SNP/TDX/CCA + attestation) — selected deterministically by Kubernetes RuntimeClass with mechanical no-silent-downgrade via `capabilities()`, four-thin-binaries mandated for TCB isolation, delivered in staged order gated on conformance + benchmarks.
- **domain:** isolation-runtime (cross-cut: kernel-frame — the framekernel plays the dual Sentry/guest-kernel role)
- **current_status:** accepted (2026-06-06)
- **disposition:** KEEP (with AMEND watch on the Sentry-framing + owned-VMM-claim items the ADR itself flags)
- **proposed_resolution:** NA
- **governing:** —
- **truth_flag:** TRUE (PARTIAL on the aspirational legs: the owned-VMM Stage-3 is benchmark-gated and "may never fire"; the Sentry Stage-4 is a not-yet-scoped re-architecture — both honestly self-flagged in the ADR, so TRUE-but-staged rather than overclaimed)
- **in_masterplan:** PARTIAL — the one-frontend + pluggable-IsolationBackend + four isolation classes + no-silent-downgrade + RuntimeClass mapping is a masterplan-grade runtime architecture; the staged effort estimates and the owned-VMM benchmark gate are implementation detail that should live in the spec, with only the architecture + the four-class/no-downgrade invariants promoted to the masterplan.
- **tensions:** This is the keystone's most-reconcilable LINUX-vs-SOURCE point (fault-line #3): ADR-0014 explicitly positions as ONE OCI/CRI frontend + pluggable backend evolving the pilot's containerd — which matches SOURCE's runtime-ladder native→sandbox→microvm→confidential (ADR-0147), wasmtime aside, and Kata/Firecracker/Cloud-Hypervisor microVMs (ADR-0254). LINUX *owns* more (owned VMM, owned Sentry) but stages it behind a reuse-then-own ratchet, so the disagreement is trigger-threshold not direction (keystone fault-line #5). Internal honesty tensions the ADR self-flags: (a) "Sentry IS the framekernel, no new ABI engine" is corrected to "REUSES semantics behind a NEW platform-parametric core + host-memory AddressSpace + interception HAL seam — multi-quarter, not yet scoped"; (b) "we rewrite Firecracker in Rust" buys nothing since Firecracker is already Rust — owned-VMM licensed ONLY on a benchmark proving confidential-launch co-design / snapshot-cold-start / process-boundary-elimination. Notable: ADR-0014 references `wasmtime`-less; SOURCE canon names wasmtime as canonical WASM (ADR-0200) — LINUX's four classes omit a WASM class, a scope gap to surface at merge (not a contradiction, an absence).
- **hyperscaler_challenge:** aligned on architecture, QUESTIONABLE on owned-VMM/owned-Sentry breadth. Google/AWS/Azure absolutely build one CRI frontend + pluggable isolation backends (gVisor + Kata + Firecracker + runc is exactly the AWS/Google spread). But a hyperscaler would NOT rewrite Firecracker (already minimal Rust, ~125ms cold-start) absent a proven differentiator — the ADR concedes this and gates Stage-3 on a benchmark, which is the correct hyperscaler-rational posture. Implication: KEEP the architecture; the owned-VMM and owned-Sentry legs stay benchmark/scope-gated (the ADR already enforces this), so no archive — but masterplan must record them as ASPIRATIONAL/gated, not committed.
- **ai_slop:** none — and notably anti-slop: the ADR is full of in-tree ground-truth citations (`crates/arch-x86_64/src/user.rs:856-1009`, 43 `sys_*` handlers / 3716 lines, no trap-interception seam in `crates/hal/src/`) and explicit MUST-FIX HONESTY corrections of its own overclaims. This is the "self-aware critic-loop" pattern the keystone (§Verdict) credits the linux series with — the opposite of slop.
- **refinement:** (1) Promote only the architecture + four-class + no-silent-downgrade + four-thin-binaries invariants to the masterplan; push effort estimates + the owned-VMM benchmark gate to the spec. (2) Surface the missing WASM/wasmtime isolation class vs SOURCE ADR-0200 at merge. (3) Confirm out-of-chunk refs ADR-0007/0017/0023 are live (keystone does not flag them retired). (4) Track the libseccomp-C-binding as the one genuine TCB-in-C own-later ratchet candidate it self-identifies.
- **consensus_needed:** Founder question (genuinely contested by the ADR's own honesty section): **"Do we COMMIT to owning the VMM and the Sentry, or do we accept reuse-forever (Firecracker/Cloud-Hypervisor + an external Sentry) if the benchmark/scope gates never fire?"** The ADR leaves both legs gated; the founder's "own the heavy base layers" directive (quoted in Alternative B) pushes own, but the engineering-conventions §5 measure-first ratchet permits reuse-forever. This is the sharpest decide-or-defer in the chunk.

---

## Chunk notes

- **Disposition tally:** 7 KEEP, 0 AMEND/ARCHIVE/SUPERSEDE/MERGE/UNCLEAR. All seven carry `status: accepted`; none is `Proposed`, so there are no unaccounted proposals to RATIFY/DROP in this slice. All seven are pilot-original (no `supersedes`/`superseded_by` edges; `renumber_note` on every one).
- **Truth flags:** all TRUE. ADR-0014 is TRUE-but-staged (its owned-VMM and owned-Sentry legs are honestly self-flagged as benchmark-gated / not-yet-scoped — partial-realization, not overclaim). The chunk is unusually clean: no STALE, WRONG, or GARBAGE.
- **No retired-vocabulary leakage** in this chunk: no `foundry`/`tier-system`/`Redis`/`Kafka`/`Jenkins-as-destination`/`Backstage`/`M0-M3-MVP` terms. (Caveat: ADR-0008 says the simulator is "stood up at **M0–M1**" and ADR-0009 references "M1+M2" boot milestones — these are kernel/db internal *rung* labels, but they textually collide with the keystone §2 RETIRED `M0/M1/M2/M3` milestone vocabulary. Flag as a possible naming-hygiene AMEND at merge: rename to descriptive Wave/Phase names to avoid reintroducing dead milestone terms. This is the only retired-vocab risk in the chunk and it is cosmetic, not a decision error.)
- **`cell` reconciliation (load-bearing for merge):** ADR-0012's `cell` is the *deployment/failure-domain pattern*, which ALIGNS with the surviving meaning SOURCE kept after ADR-0333 retired `cell`-as-microservice. Merge must bind linux ADR-0012 against ADR-0333's pattern definition so the dead µsvc meaning is not reintroduced.
- **Internal coherence of the chunk:** these seven form a tight, mutually-referencing cluster — ADR-0009 (full ABI) is the spine; ADR-0011 constrains its surface to general primitives; ADR-0010 sequences how it is delivered (run-on-Linux-first vertical slices); ADR-0013 sets the hardware floor beneath it; ADR-0014 consumes it (Sentry + guest-kernel); ADR-0012 defines the scale-out shape; ADR-0008 is the DB-engine's verification spine. No intra-chunk contradiction found.
- **Cross-side posture:** consistent with the keystone's §Verdict that the linux series is *deliberately divergent, not erroneous, and self-aware* (every ADR carries critic-loop honesty; ADR-0014 in particular cites in-tree line numbers and corrects its own overclaims). The reconcilable point with SOURCE is ADR-0014 (one-frontend + isolation-ladder == SOURCE ADR-0147/0254). The sharpest unresolved item is the own-VMM/own-Sentry COMMIT-vs-reuse-forever founder question (ADR-0014).
- **Out-of-chunk dependencies to confirm live at merge:** ADR-0001/0002/0003/0004/0005/0006/0007 (DB-engine + ports + ratchet cluster, referenced throughout 0008/0011/0012/0014), ADR-0017 (containerd rename, referenced by 0010/0014), ADR-0023 (assume-breach, referenced by 0014), ADR-0025 (Rust-Talos, keystone fault-line #3). None flagged retired by the keystone; treat as live unless a sibling chunk archives them.
- **Masterplan-readiness:** 0009/0010/0011/0012/0013 are directly masterplan-grade (YES). 0008 and 0014 are PARTIAL — promote their architecture/invariants, push their implementation detail (Elle/JVM bridge; staged effort estimates + owned-VMM benchmark gate) to specs.
