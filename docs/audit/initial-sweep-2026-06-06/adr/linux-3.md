# ADR Audit — LINUX chunk linux-3

- **side:** LINUX (pilot staging series, `~/Developer/linux/docs/decisions/`)
- **chunk:** linux-3
- **range:** ADR-0015 … ADR-0021 (slice rows 15–21 of `ls -1 ADR-*.md | sort`)
- **ADRs reviewed:** 7 (ADR-0015, ADR-0016, ADR-0017, ADR-0018, ADR-0019, ADR-0020, ADR-0021)
- **baseline:** keystone map `_map/canonical-posture-and-supersession-map.md` (read in full)
- **posture:** all 7 carry `supersedes: [] / superseded_by: []` + `renumber_note` — parallel pilot series, none supersede SOURCE ADRs; all collide with SOURCE 0001–0026 on merge (keystone §6.4).

---

### ADR-0015 — Orchestration control plane at scale: K8s-compatible, cellular, owned datastore, typed config (Manifold)

- **decision_atom:** The orchestration component is a Rust, Kubernetes-API-compatible, cell-bounded (~5k nodes/150k pods/cell) control plane (apiserver/scheduler/controller-manager/client/cell-directory) that designs out documented K8s-at-scale bottlenecks via mechanism-level mitigations (streaming-list default, sharded watch, multi-shard scheduler, jittered reconcile, owned range-sharded multi-Raft datastore behind an etcd-v3 translation adapter) and ships a built-in Rust-native typed config/packaging system (Manifold, CUE+Timoni class).
- **domain:** orchestration-scheduling (secondary: ci-cd-build via Manifold packaging — but it is control-plane config, keep single-domain orchestration-scheduling).
- **current_status:** accepted (2026-06-06).
- **disposition:** MERGE — near-duplicate of ADR-0016 (same scope, same Manifold, same complaint→mitigation matrix, same crate layout, same staging path). Two Accepted ADRs one number apart describing the identical decision is itself an SSOT defect. ADR-0015 is the richer/more-detailed of the pair (full alternatives A/B/C, open-risk table, per-stage effort). Recommend ADR-0015 survives as the canonical orchestration+Manifold ADR; ADR-0016 folds in.
- **proposed_resolution:** NA (status=accepted, not Proposed).
- **governing:** MERGE target = self (ADR-0015 is the keeper); ADR-0016 merges INTO 0015.
- **truth_flag:** TRUE (internally honest; risks named, monotonic-revision write-wall flagged as unproven, Manifold language under-scope flagged). The one PARTIAL inside: "owned datastore removes etcd write ceiling" holds only for cloud-data native KV, NOT yet for the etcd-v3 revision-emulation path — and the ADR says so explicitly.
- **in_masterplan:** PARTIAL — orchestration control-plane intent maps to SOURCE canonical-posture "Orchestration/k8s" but SOURCE adopts **Talos+CAPI+ArgoCD** (ADR-0375), not a from-scratch Rust K8s control plane. This is the keystone §5 fault-line #3/#5 (own-the-control-plane vs assemble-the-substrate). Manifold has no SOURCE analog (SOURCE uses Helm/Kustomize/external CNCF tooling).
- **tensions:** (1) Duplicate-of-ADR-0016 (intra-chunk). (2) vs SOURCE ADR-0375 Talos/CAPI adoption — LINUX builds the control plane SOURCE chose to assemble. (3) Manifold "own a CUE/KCL/Pkl-class language" vs SOURCE's reuse-CNCF posture; keystone §3 has no owned-config canon on the SOURCE side. (4) etcd-v3 adapter monotonic-revision risk cross-refs ADR-0006/ADR-0019 datastore exemplar.
- **hyperscaler_challenge:** QUESTIONABLE. Google/AWS/Azure all run bespoke control planes (Borg/Omega, GKE/EKS/AKS management planes) and Google explicitly cellular (Borg cells) — so "cellular + owned datastore + designed-out-bottlenecks" is hyperscaler-aligned in *shape*. But none rewrote a full Kubernetes-API-compatible control plane in a new language as a product day-0; they either own a non-K8s system (Borg) or operate managed K8s on commodity etcd with scale patches. The owned Manifold typed-config language is the weakest leg — hyperscalers reuse/extend existing config tooling rather than own a CUE-class language. Implication: AMEND to down-scope/flag Manifold-language ownership as the highest-risk, possibly-defer item (the ADR already self-flags this in Consequences/Risk — so the AMEND is mostly a masterplan-binding note, not a body rewrite).
- **ai_slop:** No. Dense but substantive, cited (SIG-scalability KEPs, named hyperscaler postmortems), and self-critical. Not slop.
- **refinement:** On merge with 0016, dedupe the complaint→mitigation matrix (they are ~95% identical) and keep ONE staging path (0015's STAGE 0–5 and 0016's STAGE 0–5 differ slightly in where Manifold lands — reconcile).
- **consensus_needed:** Founder question — "Does the masterplan commit to OWNING a Kubernetes-compatible control plane + a bespoke typed-config language (Manifold), or adopt SOURCE's Talos+CAPI+ArgoCD + CNCF-config posture? The pilot's ADR-0015/0016 assume the former; SOURCE ADR-0375 assumes the latter. These cannot both be canonical."

---

### ADR-0016 — Orchestration component: built-in typed config (Manifold) + complaint-driven mitigations

- **decision_atom:** (Same canonical decision as ADR-0015) — the bounded Kubernetes-compatible cell control plane plus the Rust-native Manifold typed-config/OCI-packaging system, with the K8s-at-scale complaint→root-cause→mitigation matrix and a multi-year evidence-gated staging path.
- **domain:** orchestration-scheduling.
- **current_status:** accepted (2026-06-06).
- **disposition:** ARCHIVE (redundant duplicate) — or equivalently the MERGE-source into ADR-0015. Two Accepted ADRs describing the same decision is a redundancy defect; 0016 adds little 0015 lacks except a slightly sharper self-critique of Manifold fork-option (d) and the Alternatives A–D config-engine analysis. Fold those two deltas into 0015 and archive 0016.
- **proposed_resolution:** NA (accepted).
- **governing:** ADR-0015 (the merge keeper).
- **truth_flag:** TRUE (same honesty profile as 0015; explicitly concedes Manifold language front-end is "the ONE thing ADR-0003 §6 does NOT justify" — a genuinely useful admission to preserve on merge).
- **in_masterplan:** PARTIAL (identical mapping to 0015).
- **tensions:** Primary tension = it duplicates ADR-0015 (same `related_specs: orchestration-and-typed-config-canonical.json`, same crate names, same matrix). Same SOURCE-Talos and SOURCE-no-owned-config tensions as 0015.
- **hyperscaler_challenge:** QUESTIONABLE (identical to 0015) — the duplicate-ADR existence is itself the kind of doc-drift a hyperscaler ADR process (single authoritative decision per topic) would reject. Implication: ARCHIVE.
- **ai_slop:** Borderline — the *duplication* of 0015 reads like two generation passes on the same prompt that were both committed. Each individually is substantive (not slop), but their coexistence is a process-hygiene defect to clean.
- **refinement:** Before archiving, lift into 0015: (a) the explicit fork-option-(d) "adopt CUE engine, own only lowering+OCI+WASM seam" rejection-with-caveat, and (b) Manifold STAGE-1 "skeleton vs full language surface" split (0016 sequences Manifold earlier-but-thinner than 0015 — pick one).
- **consensus_needed:** Same founder question as 0015, plus: "Confirm ADR-0015 (not 0016) is the canonical orchestration ADR before the other is archived."

---

### ADR-0017 — Container platform: full L0–L8 stack (extends ADR-0014 to storage/image/distribution/build/manager+CRI)

- **decision_atom:** Extend the ADR-0014 L0–L3 isolation core into a single hexagonal L0–L8 "container-platform" component (rename containerd→container-platform) that owns content store + snapshotters (L4), OCI image model (L5), distribution client + owned zot-class registry server (L6), LLB-class reproducible build + SBOM/SLSA orchestration (L7, signing/scanning permanently reused per ADR-0020), and the manager+metadata+shim+CRI daemon (L8), delivered via an evidence-gated multi-year staging path with L7 flagged for possible boundary split.
- **domain:** isolation-runtime (secondary: security-supplychain — L7 SBOM/SLSA/sign-verify is a genuine cross-cutting supply-chain concern).
- **current_status:** accepted (2026-06-06).
- **disposition:** KEEP (with AMEND-on-merge). Sound, well-bounded, hexagonal, honestly staged; `review_note` shows a critic pass reconciled L7 to ADR-0020 PERMANENT_REUSE and de-hand-waved the oyago bootstrap. The AMEND is only the merge-time naming/number reconciliation, not a content defect.
- **proposed_resolution:** NA (accepted).
- **governing:** n/a (KEEP).
- **truth_flag:** TRUE. Self-aware about the L7 sprawl risk ("a second mountain"), the redb-vs-sled provisional engine choice, and the multi-year solo-team reality. oyago is honestly framed as a "starting scaffold," not a deliverable.
- **in_masterplan:** PARTIAL — SOURCE canonical posture has container runtime via Talos+containerd+Kata/Firecracker/wasmtime (keystone §3 isolation row, ADR-0200/0147/0254); LINUX owns the whole platform instead. The L7 supply-chain piece (Cosign/Sigstore/Trivy reuse) DOES align with SOURCE PERMANENT_REUSE (keystone §3 — Cosign/Trivy reused). So masterplan-fit is PARTIAL: supply-chain reuse aligns, but "own the entire container platform" diverges from SOURCE's assemble-containerd posture.
- **tensions:** (1) vs SOURCE ADR-0375/containerd adoption — own-the-platform vs use-containerd (keystone fault-line #3/#5). (2) vs ADR-0018, which says containerd "dissolves → CRI/OCI compat adapter" and the runtime IS the Capsule — 0017 builds a full owned containerd-class platform while 0018 reframes that same platform as a thin compat adapter to the Capsule contract at the owned endpoint. These need reconciliation (0017's L8 manager vs 0018's "containerd→compat adapter"). (3) L7 build engine is a different product class bolted on — internal sprawl tension the ADR itself flags.
- **hyperscaler_challenge:** ALIGNED-ish but BROAD. Hyperscalers do own end-to-end supply-chain substrates (Google's distroless/BuildKit-derived internal build, internal registries, binary provenance/SLSA originated at Google) — so owning L4–L8 as one content-addressed graph is hyperscaler-shaped. BUT they do NOT generally rewrite the whole stack in a new language solo; and they reuse signing/scanning (which this ADR correctly does). The questionable part is breadth for a pilot: a registry server + LLB build engine + CRI manager is 3 multi-year products. Implication: KEEP the decision, but AMEND the masterplan binding to mark L7 (build) as an OWN_EARLY/boundary-split candidate (the ADR already recommends this at the Stage-7 gate).
- **ai_slop:** No. Concrete crate layout, per-layer Go→Rust mapping, explicit DROP list, conformance gates per stage. Substantive.
- **refinement:** Reconcile the L8 boundary with ADR-0018 (is L8 manager a full owned daemon, or the "CRI/OCI compat adapter" 0018 describes? both can't be the literal endpoint simultaneously without a stated H1-vs-H2 mapping).
- **consensus_needed:** "Is the container platform an OWNED full-stack product (ADR-0017's L0–L8), or a thin Capsule-compat adapter over the framekernel (ADR-0018's framing)? The two ADRs describe the same component with opposite ownership centers of gravity."

---

### ADR-0018 — Host-framekernel + Capsule model: staged host-grades; isolation as a native primitive

- **decision_atom:** Adopt the "we are the host / unify container-VM-pod-OS as a Capsule / containerize via the framekernel's own isolation" vision as a DESTINATION, but commit only to a staged host-grade ladder where H0 (today, framekernel boots as a QEMU guest) and H1 (years 1–3, the committed product, runs the flagship Native Capsule on **Linux** as a userspace shim, framekernel only the Microvm guest kernel) are real, and the literal "we are the host" claim is true only at the optional, uncommitted, budget-gated H2 (bare-metal drivers + in-Frame hypervisor).
- **domain:** kernel-frame (secondary: isolation-runtime — the Capsule = ADR-0014's IsolationBackend trait).
- **current_status:** accepted-with-reservations (2026-06-06).
- **disposition:** KEEP. This is the model-honesty exemplar the keystone §"Verdict" praises: `review_note` records `consensus=FALSE`, the critic forced four corrections, and the ADR time-boxes the literal host claim to uncommitted H2. The reservations ARE the value — do not amend them away.
- **proposed_resolution:** NA (accepted-with-reservations; treat as an Accepted variant, not a Proposed needing RATIFY/DROP — the "reservations" are the four named honesty corrections, already resolved in-text).
- **governing:** n/a (KEEP).
- **truth_flag:** TRUE — and notably the most self-aware ADR in the chunk. Ground-truth re-verified (43/42 sys_* handlers, single-namespace tmpfs VFS, zero-link netlink, 0 unsafe tokens PASS). Explicitly states "'we are the host' is literally FALSE through H1."
- **in_masterplan:** NO (as a literal posture) / PARTIAL (as staged). SOURCE has no framekernel-as-host concept; SOURCE uses Talos node-OS + containerd + external VMM (keystone fault-line #3). The framekernel/Capsule "own the host/kernel" direction is deliberately divergent from SOURCE's assemble-the-substrate posture. The H1-on-Linux commitment actually RE-CONVERGES toward SOURCE (Linux host now) — the divergence is the H2 research bet.
- **tensions:** (1) Keystone fault-line #3 (framekernel vs Talos+containerd+firecracker+wasmtime) — the sharpest LINUX↔SOURCE isolation tension. (2) Internal: vs ADR-0017 (containerd "dissolves → compat adapter" here, but 0017 builds the full owned platform) — must be reconciled. (3) vs ADR-0025 (per keystone, a Rust-"Talos" node-OS) — overlapping own-the-host ambition. (4) Self-noted open question: is the Sandbox/Sentry strength even worth the Stage-4 re-architecture.
- **hyperscaler_challenge:** MISALIGNED for the literal H2 vision; ALIGNED for H1. No hyperscaler ships a from-scratch microkernel/framekernel-as-host as a cloud substrate today (they run Linux/KVM, Firecracker on Linux, gVisor on Linux). The "own the host kernel + in-Frame hypervisor" H2 is a research bet none of Google/AWS/Azure has made at production scale (closest: AWS Nitro/Firecracker, but those run ON Linux/KVM, not a replacement host kernel). Implication: the ADR's own staging ALREADY encodes this — H2 is "justify-or-drop, gated, budgeted." No amend needed beyond ensuring the masterplan never books H2 as committed.
- **ai_slop:** No — the opposite. The `review_note` honesty (consensus=FALSE, writer stalled, authored from final model + corrections) is anti-slop by construction.
- **refinement:** Reconcile the containerd-dissolution claim with ADR-0017's full-platform build (single sentence stating: 0017 builds the platform that 0018's compat-adapter fronts at H1, becomes Capsule-native at the owned endpoint).
- **consensus_needed:** "Is H2 (framekernel-as-bare-metal-host with in-TCB hypervisor) a funded destination or a perpetual research option? The masterplan must book it as one or the other; the ADR deliberately leaves it uncommitted." Also: the unresolved unsafe-token budget ceiling for the in-Frame hypervisor (open question #1).

---

### ADR-0019 — Universal port/adapter ratchet: vendored now, owned when proven; design to the CONTRACT

- **decision_atom:** Every component is a port: ship a vendored/reused adapter now and transition to an owned adapter only when it is *ready* (passes its conformance evidence bundle) AND *proven* (beats-or-parities the vendored adapter across the no-cherry-pick four-axis scorecard, sustained over production burn-in), with the governing discipline that each port is designed to its CONTRACT — either owned-ideal (we define the contract; design to the final owned ideal) or external-standard (the standard pins the shape; own the engine behind it, never replace the contract).
- **domain:** governance-process (secondary: docs-ssot-masterplan — it is the meta-rule governing all ownership ADRs).
- **current_status:** accepted (2026-06-06).
- **disposition:** KEEP. This is the load-bearing meta-ADR for the whole pilot; clean two-category port discipline, defined gate terms, named failure modes, reviewed (critic verdict REVISE → revised text). The keystone §5.5 notes both repos share this "own when proven" ratchet language (LINUX 0019/0020 ↔ SOURCE 0211/0173) — so this ADR is a CONVERGENCE point, not a divergence.
- **proposed_resolution:** NA (accepted).
- **governing:** n/a (KEEP).
- **truth_flag:** TRUE. Honest about the datastore external-standard exemplar being NOT a clean lossy bridge (etcd-v3 monotonic-revision over HLC is an open design risk, cross-ref ADR-0015) — it does not over-claim.
- **in_masterplan:** YES (as a principle) — the "own when proven" ratchet aligns with SOURCE's staged-ownership posture (keystone §5.5: LINUX 0019/0020 agree with SOURCE 0211/0173). The disagreement per keystone is the *trigger threshold*, not the principle. This is the most masterplan-compatible ADR in the chunk.
- **tensions:** Minor: the per-component ratchet table here (host/node-OS/runtime/config/registry/datastore) overlaps ADR-0020's scored inventory — 0019 = the discipline, 0020 = the scored application. They are complementary (0020 explicitly "operationalizes" 0019), not conflicting. Watch only for table drift between the two on merge.
- **hyperscaler_challenge:** ALIGNED. "Adopt now, own when it beats-or-parities, design to the contract" is exactly the hyperscaler build-vs-buy discipline (Google's gradual ownership of storage/network/silicon; AWS's Nitro-over-time). The owned-ideal-vs-external-standard distinction is genuinely good architecture. No amend.
- **ai_slop:** No. Tight, principled, reviewed. Among the strongest ADRs in the chunk.
- **refinement:** Operational §2 self-notes that `engineering-conventions.md` SHOULD be updated to reference this and ADR-0007/0003 should back-link — a real follow-up doc task, not a defect.
- **consensus_needed:** None on the principle. The only founder-level question is the keystone's: the *trigger threshold* for "own when proven" (LINUX day-0 ownership ambition vs SOURCE later-when-proven) — but that surfaces in 0020/0021, not here.

---

### ADR-0020 — Staged-ownership roadmap: rubric, scored inventory, day-0 focus

- **decision_atom:** Operationalize ADR-0019 into a four-axis scoring rubric (ownership_cost_now / switching_cost_later / port_isolatable / data_gravity, plus is_external_contract and api_surface_spread) that buckets every infra/datastore/runtime dependency into OWN_DAY0 / OWN_EARLY / DEFER_VENDORED / PERMANENT_REUSE, with Cedar as the lone day-0 ownership and a deliberately small focus set, and hard gates on unsafe deferrals (notably Milvus's monotonic vector-count gate).
- **domain:** governance-process (secondary: data-storage — the inventory is dominated by datastore/observability dependency calls).
- **current_status:** accepted (2026-06-06).
- **disposition:** KEEP, with AMEND (retired-vocab + SOURCE-binding). The rubric and bucketing are sound and reviewed. AMEND needed because the inventory references SOURCE component decisions and uses one retired term.
- **proposed_resolution:** NA (accepted).
- **governing:** n/a (KEEP, with the amend below).
- **truth_flag:** STALE in two spots, TRUE in structure. (1) **Retired-vocab leak:** L89 references "all **Foundry** invocations" for OTel API spread — "Foundry" is RETIRED (keystone §2 → intelligence/governance, ADR-0335/0347). This is exactly the residual `foundry`-residue the keystone warns about (MFL-0002/0003). (2) The inventory's specific tool choices (ClickHouse/Pulsar/Wasmtime/Milvus/Valkey/Cosign/Kyverno/Trivy/PostgreSQL+Citus/Iceberg) are LINUX's reading of the SOURCE stack — mostly correct and aligned with keystone §3 (Valkey not Redis ✓, Pulsar ✓, Cosign/Trivy reused ✓), but they assume the SOURCE substrate that the rest of the LINUX pilot (own-DB ADR-0001, own-policy ADR-0021) partly contradicts.
- **in_masterplan:** PARTIAL. The rubric/buckets are a governance principle that maps cleanly. The specific inventory rows partially align with SOURCE canonical posture (keystone §3): Valkey/Pulsar/Cosign/Trivy/Kyverno/Iceberg classifications match SOURCE. BUT "PostgreSQL + Citus → PERMANENT_REUSE / reuse indefinitely" DIRECTLY CONTRADICTS LINUX ADR-0001's "eliminate the PostgreSQL/sqlx dependency / own a from-scratch multi-model engine" (keystone fault-line #1, the sharpest unflagged conflict). 0020 says reuse Postgres forever; 0001 says eliminate it. **This intra-LINUX contradiction is the most important finding in the chunk.**
- **tensions:** (1) **HARD INTERNAL CONTRADICTION vs LINUX ADR-0001:** 0020 classifies PostgreSQL+Citus as PERMANENT_REUSE ("reuse indefinitely"); ADR-0001 wants to eliminate Postgres and own the DB engine. These are mutually exclusive at face value. (2) Cedar OWN_DAY0 here is operationalized by ADR-0021 (consistent within chunk). (3) "Foundry" retired-vocab leak (L89). (4) Kyverno PERMANENT_REUSE here vs SOURCE ADR-0379 making **Kubewarden** the default admission (keystone §1.1/§3 superseded ADR-0183) — LINUX 0020 names Kyverno, SOURCE moved off it; minor staleness on merge.
- **hyperscaler_challenge:** ALIGNED. A scored buy-vs-build inventory with data-gravity and switching-cost axes is textbook hyperscaler dependency governance. The "keep the day-0 set tiny (just Cedar)" discipline is correct (hyperscalers do NOT own everything at once). One questionable row: Cedar as the SOLE day-0 own, while a from-scratch DB engine (ADR-0001) and owned policy language (ADR-0021) are also being pursued — the inventory's own restraint contradicts the pilot's broader own-everything-day-0 pattern (keystone fault-line #5). Implication: AMEND to reconcile the Postgres row with ADR-0001 and the Kyverno row with SOURCE Kubewarden; refresh "Foundry" → intelligence.
- **ai_slop:** No. Genuinely useful rubric; `review_note` shows critic fixes (Grafana LGTM reclassified, Milvus unsafe-deferral gate added). Substantive.
- **refinement:** (1) Replace "Foundry" (L89) with "intelligence" per keystone §2. (2) Reconcile PostgreSQL+Citus PERMANENT_REUSE row against ADR-0001 own-DB intent — either 0001 or 0020 must yield. (3) Update Kyverno → Kubewarden to track SOURCE ADR-0379, or note the divergence explicitly.
- **consensus_needed:** **"Do we OWN the database engine (ADR-0001, eliminate Postgres) or REUSE PostgreSQL+Citus permanently (ADR-0020)? The pilot's own roadmap and its own foundation ADR give opposite answers."** This is the load-bearing contradiction the masterplan must resolve before either is booked.

---

### ADR-0021 — Owned Authorization Policy Framework: typed, compile-to-Rust, tier-aware (Cedar-compatible)

- **decision_atom:** Operationalize ADR-0020's Cedar OWN_DAY0 as a typed authorization policy language that is a *superset of Cedar* (retains PARC + entity-DAG + forbid-trumps-permit no-false-allow), adds a first-class autonomy-tier (T1–T4) compiled dimension, and replaces Cedar's runtime interpreter with a build-time compiler emitting a generated typed Rust evaluator — with `cedar-policy` as the vendored adapter/differential oracle now and the owned compiler proof-gated (differential parity + Kani invariant + production burn-in), plus a two-tier runtime-update model (structure compiles, data stays hot-evaluable).
- **domain:** authz-policy (secondary: intelligence-ai / agentic-platform — the T1–T4 autonomy-tier dimension is the "regulated-AI-workflow product edge").
- **current_status:** accepted (2026-06-06).
- **disposition:** KEEP. Sound, reviewed (critic verdict REVISE SOUND_WITH_FIXES; phantom citations removed per keystone §"Verdict"), positions explicitly as the owned successor to Cedar (extends Cedar, not replaces the model). Honest about probabilistic-not-machine-checked soundness.
- **proposed_resolution:** NA (accepted).
- **governing:** n/a (KEEP).
- **truth_flag:** TRUE. The `review_note` documents removal of phantom "(research §N)"/"(landscape §N)" citations and correction of tier-model source attribution — the keystone §"Verdict" specifically cites this as evidence the linux edits are honest, not slop.
- **in_masterplan:** PARTIAL. SOURCE makes **Cedar the universal authorization gate** (keystone §3 policy/authz, ADR-0243/0246) — LINUX ADR-0021 is **Cedar-COMPATIBLE** (vendors `cedar-policy` now, owns a compile-to-Rust superset later). Per keystone fault-line #2 this is "own vs reuse Cedar," NOT a flat contradiction — LINUX positions as the owned successor to the same Cedar model. The autonomy-tier T1–T4 is a NEW live policy dimension (keystone explicitly warns: do NOT conflate with the retired tenant "tier-system" of ADR-0329 — different axis; this is policy-autonomy ceiling, legitimately live).
- **tensions:** (1) Keystone fault-line #2: own-the-policy-language (LINUX) vs reuse-Cedar-as-universal-gate (SOURCE ADR-0243/0246). Reconcilable (LINUX = owned successor, Cedar-compatible ingest). (2) Shared `oya-compiler-infra-*` spine with ADR-0016 Manifold — a real coupling (both are typed-DSL→compile-to-Rust); if 0016 archives into 0015, the shared-compiler-infra reference must re-point to 0015. (3) References ADR-0022 ("adopt research, own the impl") as governing methodology — 0022 is outside this chunk; verify it exists and is Accepted.
- **hyperscaler_challenge:** QUESTIONABLE-leaning-aligned. AWS *built* Cedar (and runs it as an interpreter at scale) — so "own the authz policy semantics" is hyperscaler-validated (AWS owns Cedar end-to-end). The QUESTIONABLE part is "compile policies to native Rust at build time" instead of interpreting: no major hyperscaler compiles authz policy to native code as the production path (AWS/Styra deploy Cedar/OPA as data + interpreter for exactly the hot-update reason this ADR must engineer around with its two-tier model). The autonomy-tier compiled dimension is a genuine differentiator for regulated-AI but unproven at scale. Implication: KEEP, but flag the compile-to-Rust-vs-interpret bet as the item most likely to need revisiting if hot-update friction proves real (the ADR's §6 two-tier model exists precisely to de-risk this).
- **ai_slop:** No — and it is the keystone's named example of de-slopped work (phantom citations removed). Substantive.
- **refinement:** (1) On 0016→0015 merge, re-point the "shared compiler infrastructure with ADR-0016" references to the surviving ADR. (2) Confirm ADR-0022 (governing methodology) is a real, Accepted ADR.
- **consensus_needed:** "Own a compile-to-Rust authz language (ADR-0021) vs reuse Cedar-the-interpreter as the universal gate (SOURCE ADR-0243/0246)? Reconcilable as 'owned Cedar-compatible successor,' but the masterplan must state whether the compile-to-Rust differentiator is a committed bet or an optional S3 ratchet." Also confirm the T1–T4 autonomy-tier is booked as a LIVE policy axis (NOT the retired tenant tier-system).

---

## Chunk notes

**Headline findings (in priority order):**

1. **DUPLICATE ACCEPTED ADRs — ADR-0015 ≡ ADR-0016.** Both are `status: accepted`, dated the same day, one number apart, sharing `related_specs: orchestration-and-typed-config-canonical.json`, and describe the SAME decision (bounded K8s control plane + Manifold + the identical complaint→mitigation matrix + identical crate layout). This is an SSOT defect on its face. **Recommend MERGE: ADR-0015 survives (richer), ADR-0016 archives into it** after lifting 0016's two deltas (fork-option-(d) rejection-with-caveat; Manifold STAGE-1 skeleton-vs-full split). A masterplan generated from the ADR log must not carry two canonical orchestration ADRs.

2. **HARD INTRA-LINUX CONTRADICTION — ADR-0020 vs ADR-0001 on PostgreSQL.** ADR-0020 classifies **PostgreSQL+Citus as PERMANENT_REUSE / "reuse indefinitely"**; LINUX ADR-0001 (out-of-chunk, per keystone fault-line #1) wants to **eliminate the PostgreSQL/sqlx dependency** and own a from-scratch multi-model DB engine. These are mutually exclusive. The keystone flags ADR-0001's "eliminate Postgres" as "the sharpest unflagged conflict with source" — but it is ALSO an unflagged conflict *inside the LINUX pilot itself* (0020 reuses what 0001 eliminates). **This is the single most important resolution the masterplan must force in this chunk.**

3. **Own-vs-assemble is the through-line.** ADR-0015/0016 (own the K8s control plane + own a config language), 0017 (own the container platform), 0018 (own the host/kernel via framekernel) all choose OWN against SOURCE's assemble-the-substrate canon (Talos+CAPI+ArgoCD, containerd, external CNCF config). Keystone fault-lines #3/#5. NONE are reconciliation bugs — they are deliberate, self-aware divergences (every ADR carries a `review_note` and honest reservations). The masterplan decision is a single founder call: **does the pilot's day-0 OWN ambition become canon, or does SOURCE's staged-assemble posture?**

4. **The ratchet ADRs (0019/0020/0021) are the convergence spine.** 0019 (port discipline) and 0020 (scored inventory) ENCODE "own when proven," which the keystone (§5.5) says BOTH repos share (LINUX 0019/0020 ↔ SOURCE 0211/0173). 0021 is Cedar-COMPATIBLE (owned successor, not a fork). So the disagreement is the *trigger threshold / day-0 set*, not the principle — and 0020's own restraint (Cedar as the SOLE day-0 own) actually argues AGAINST the broad own-everything pattern of 0001/0015/0017/0018. The pilot contains its own counter-argument.

5. **ADR-0017 ↔ ADR-0018 internal seam.** 0017 builds a full owned L0–L8 container platform (incl. L8 manager daemon); 0018 says "containerd dissolves → CRI/OCI compat adapter" and "the runtime IS the Capsule." Same component, opposite ownership center of gravity. Needs a one-line H1-vs-owned-endpoint reconciliation on merge.

**Retired-vocab leaks found:** ADR-0020 L89 — "all **Foundry** invocations" (Foundry RETIRED → intelligence/governance, keystone §2; ADR-0335/0347). Only one leak in the chunk; the other 6 ADRs are vocab-clean. (ADR-0020's "Kyverno PERMANENT_REUSE" is not retired-vocab but is stale vs SOURCE ADR-0379 Kubewarden-default.)

**Proposals needing RATIFY/DROP:** none — all 7 are `accepted` (ADR-0018 is `accepted-with-reservations`, treated as Accepted; its "reservations" are four resolved honesty corrections, not an open proposal). No unaccounted proposals in this slice.

**Truth-flag summary:** TRUE ×6 (0015 with an internal PARTIAL on the etcd-write-wall claim, self-flagged), STALE ×1 (0020 — Foundry leak + Kyverno/Postgres staleness; structurally TRUE). No WRONG, no GARBAGE. The keystone §"Verdict" that the LINUX edits are "internally coherent and self-aware, deliberately divergent not erroneous" holds across this entire chunk — ADR-0018 (consensus=FALSE recorded) and ADR-0021 (phantom citations removed) are the named exemplars and both are in this slice.

**Cross-chunk dependencies to verify (out of my range):** ADR-0001 (DB engine — the 0020 contradiction), ADR-0012 (cellular — heavily cited by 0015/0016/0018), ADR-0014 (IsolationBackend — base of 0017/0018), ADR-0022 (adopt-research-own-impl — governing 0021), ADR-0025 (Rust-"Talos" node-OS — overlaps 0018's own-the-host).
