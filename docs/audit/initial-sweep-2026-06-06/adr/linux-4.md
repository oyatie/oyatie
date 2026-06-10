# ADR Audit — LINUX pilot chunk 4

- **side:** LINUX (pilot/staging; `~/Developer/linux`, 26 ADRs, parallel staging series renumbered into SOURCE on merge)
- **chunk:** linux-4
- **range:** slice 22–28 of `ls ADR-*.md | sort` ⇒ ADR-0022 … ADR-0026 (5 files; slice has only 5 entries, 26 total)
- **ADRs reviewed:** ADR-0022, ADR-0023, ADR-0024, ADR-0025, ADR-0026

> Auditor binding: ADRs are the immutable SSOT; the masterplan is GENERATED from the live ADR log. All five carry `status: accepted`, `supersedes: []`, `superseded_by: []`, and the standard `renumber_note` (collide with SOURCE ADR-0022/0023/0024/0025/0026 on merge — renumber to ~ADR-0515+). All five are dated 2026-06-06 and are part of the recent linux auto-reconciliation cluster the keystone map judged "internally coherent, deliberately divergent — not slop."

---

### ADR-0022 — Adopt hyperscaler research, own the Rust implementation, design for extreme scale (the governing methodology)

- **decision_atom:** Every component must adopt proven hyperscaler research (named, never reinvented), implement it in memory-safe Rust with a named measured differentiator, own it via the ratchet, and design for extreme scale from day 0 — gated by a per-ADR checklist of (research-adopted, Rust-flavor, extreme-scale design).
- **domain:** governance-process (cross-cutting: docs-ssot-masterplan — it is the meta-methodology every design ADR must satisfy).
- **current_status:** accepted.
- **disposition:** KEEP — this is the governing methodology ADR for the whole LINUX pilot; correct and self-consistent. It is the LINUX analogue of SOURCE's "own when proven" ratchet doctrine (ADR-0211/0173) and engineering-conventions §6.
- **proposed_resolution:** NA (not Proposed).
- **governing:** —
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — as a methodology it should be a masterplan invariant/principle, not a feature row; today the masterplan does not encode a "per-ADR hyperscaler-adoption checklist" gate explicitly.
- **tensions:** The adoption matrix asserts `cloud-data` = "owned Rust engine … pg-wire + etcd-v3 compat" — re-anchors LINUX ADR-0001's "eliminate PostgreSQL" stance against SOURCE best-of-breed Postgres+Milvus (keystone fault-line #1). The framekernel row cites "~14% audited-unsafe TCB" as a live number; if the Frame is not yet built this is aspirational, not measured (matrix presents it as current proof). Matrix also lists `node-OS: vendored Talos → owned` — consistent with ADR-0025 but in direct competition with SOURCE's adoption of *actual* Talos (ADR-0375).
- **hyperscaler_challenge:** Aligned — "adopt the method, own the implementation, design for scale day-0" is precisely how Google/AWS/Azure build (Borg→K8s, Aurora's log-is-the-DB, Nitro). The questionable part is *breadth*: a hyperscaler would not own DB-engine + policy-lang + kernel + node-OS + VMM all at day-0; they sequence ownership behind proven demand. Implication: KEEP the methodology; the breadth/trigger-threshold concern lands on the per-component ADRs (0001/0021/0025), not here.
- **ai_slop:** No. Dense but load-bearing; each matrix cell names a real system. The one slop-adjacent risk is presenting "~14% TCB" and matrix differentiators as established fact rather than targets.
- **refinement:** Mark the matrix's TCB ratio and "our flavor" cells as TARGET vs MEASURED so the methodology's own "the flavor must be real / measured edge" rule (its Negative #2) is not violated by the matrix itself.
- **consensus_needed:** "Is the day-0 hyperscaler-adoption checklist a binding masterplan gate for every future ADR, or guidance? And does 'own it' here ratify owning the DB engine + policy lang + kernel + node-OS simultaneously, or must each clear the ADR-0019/0020 proven-trigger first?"

---

### ADR-0023 — Isolation security posture: assume-breach; strength by blast-radius, not authorship

- **decision_atom:** Isolation strength is set by blast-radius × data-sensitivity × attack-surface (assume-breach), defaulting every workload — first-party included — to a strong microVM boundary, with hardened-native a classified, admission-gated downgrade only for the genuinely-low-stakes tail and confidential computing for crown jewels; and the framekernel's tiny-TCB ratio is a hard ceiling (get the hardware boundary from a separate VMM, never by bloating the Frame).
- **domain:** security-supplychain (cross-cutting: isolation-runtime).
- **current_status:** accepted.
- **disposition:** KEEP — resolves the open "secure-by-default" question left dangling by ADR-0018; correct and hyperscaler-aligned. Note: the Status text says it "supersedes a provisional earlier suggestion" (default first-party→native) — that is superseding a *prose stance*, not a numbered ADR, so `superseded_by`/`supersedes` correctly stay empty.
- **proposed_resolution:** NA.
- **governing:** — (no ADR archived; the superseded item is an unnumbered provisional suggestion internal to ADR-0018's open question).
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — "microVM-per-pod secure-by-default + risk-classified RuntimeClass + assume-breach" should be a masterplan isolation invariant; the admission-gated workload-classification step is a concrete deliverable not yet reflected as a masterplan line.
- **tensions:** Strongly *agrees* with SOURCE BeyondProd/Talos posture (this is convergence, not conflict). Internal tension it deliberately surfaces and resolves: ADR-0018's H2 "in-Frame hypervisor + drivers" GROWS the unsafe Frame — ADR-0023 makes the TCB ratio a hard invariant and gates H2 on not regressing it, i.e. it constrains ADR-0018 without superseding it (a healthy amend-by-relation that ADR-0018 should back-reference). Also re-raises ADR-0018 open-question #2 (value of Sandbox/Sentry strength) — left open, flagged not resolved.
- **hyperscaler_challenge:** Aligned — assume-breach / strength-by-blast-radius / confidential-for-crown-jewels is exactly Google BeyondProd + AWS Nitro + NIST 800-207. A hyperscaler would make THIS decision. No amend/archive implication.
- **ai_slop:** No. This is one of the strongest ADRs in the chunk — it explicitly rejects the seductive "trust our own code" shortcut and self-corrects a prior stance.
- **refinement:** ADR-0018 should be amended to carry a back-reference noting its H2 host claim is now gated by ADR-0023's TCB-ratio invariant (cross-ref discipline, so the constraint is discoverable from 0018).
- **consensus_needed:** None contested. (Optional: confirm the cost posture — "most workloads pay microVM overhead" — is economically acceptable for the low-margin demo_trial tenant-class.)

---

### ADR-0024 — Capability layer placement: build on Linux to measure, push down only when earned, keep the Frame minimal

- **decision_atom:** Capabilities default to userspace and drop to the safe-kernel or (last-resort, irreducibly-unsafe-only) Frame ONLY on measured justification (perf hot-path / security-is-the-boundary / kernel-atomicity correctness), with product policy never dropping to the kernel; every std component is written to the Linux ABI and run on Linux first so the running product is itself the placement evidence-generator, perf baseline, and differential-compat oracle.
- **domain:** kernel-frame (cross-cutting: isolation-runtime).
- **current_status:** accepted.
- **disposition:** KEEP — coherent and directly operationalizes the "build on Linux first to measure" founder directive plus ADR-0011 (general primitives) and ADR-0023 (TCB ceiling). The placement table is sound.
- **proposed_resolution:** NA.
- **governing:** —
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — "Linux-first measurement loop drives push-down; the running product = perf baseline + diff-oracle" is a methodology that belongs in the masterplan as a build-sequencing invariant; not yet an explicit masterplan line.
- **tensions:** Tightly consistent with ADR-0009/0010/0011/0019/0023/0026 (it is the hub that ties them). No contradiction; the only watch-item is that it asserts ~14% TCB and "the differentiator" as established — same aspirational-vs-measured caveat as ADR-0022. Mild redundancy with ADR-0026 (both describe the kernel-level port/measure-before-push-down discipline) — see MERGE note below; kept separate because 0024 is *placement* policy and 0026 is *adapter mechanism*.
- **hyperscaler_challenge:** Aligned — "default userspace, push down only on measured benefit, keep the trusted base minimal" is textbook microkernel/hyperscaler discipline (gVisor, Google's measured-before-optimize culture). A hyperscaler would make THIS decision. No amend/archive implication.
- **ai_slop:** No. Tables and reasoning are concrete and non-generic.
- **refinement:** Add an explicit cross-reference to ADR-0026 at the decision level (they are two halves of one doctrine: 0024 = where a capability lives, 0026 = how it is adapted Linux→Frame) so a reader isn't left to infer the seam.
- **consensus_needed:** None contested.

---

### ADR-0025 — node-OS: a Rust "Talos" (K8s-compatible immutable node OS); beat-or-parity vs Talos + Rust-vs-Go security analysis

- **decision_atom:** The node-OS is an immutable, API-managed (no-SSH), Kubernetes-compatible "Rust Talos" — Talos-config-compatible during a vendored→owned ratchet — that replaces vendored Talos only after a sustained beat-or-parity perf/economics scorecard, with the OS-replacement preceding the kernel-replacement and the Rust-over-Go security advantage treated as structural-but-unproven-until-measured.
- **domain:** node-os (cross-cutting: security-supplychain — the Rust-vs-Go analysis).
- **current_status:** accepted.
- **disposition:** KEEP — fills a genuine gap (component #2 had no dedicated ADR) and is admirably honest (the Rust>Go win is "shared on memory-safety, real on no-GC/data-race/bounded-unsafe/type-state; unproven until measured"). The staged ladder (Talos day-0 → owned node-OS → kernel ratchet) is the most concrete sequencing statement in the chunk.
- **proposed_resolution:** NA.
- **governing:** —
- **truth_flag:** TRUE (the security claim is explicitly flagged as unproven-pending-measurement, which is the honest truth-state — not WRONG, not STALE).
- **in_masterplan:** PARTIAL — "Rust Talos node-OS, Talos-compatible transition, beat-or-parity gate" should be the masterplan's node-os component line; the beat-or-parity-vs-Talos scorecard is a standing gate that should appear in the masterplan's gate set.
- **tensions:** SHARPEST cross-repo tension in the chunk (keystone fault-line #3): LINUX wants a *Rust Talos* that competes with and replaces Talos, while SOURCE *adopts actual Talos* as the node-OS (ADR-0375, sup. ADR-0121/0120). This is "own-the-node-OS vs assemble-Talos" — a real architecture fork, not an error. LINUX mitigates by making the early stage "Talos day-0 (vendored)" and Talos-config-compatible, so the divergence is a deferred trigger, not an immediate contradiction; still must be surfaced to the founder because the masterplan cannot simultaneously canonize "we run Talos" (SOURCE) and "we replace Talos with our Rust OS" (LINUX) without a resolved sequencing/ownership-trigger.
- **hyperscaler_challenge:** Questionable — a hyperscaler with infinite resources DOES write its own node-OS (Google COS, AWS Bottlerocket — which is *Rust-adjacent* and exactly this thesis), so the *direction* is aligned; but a hyperscaler would not greenfield a node-OS before its product exists and revenue justifies it — they would run a proven immutable OS (Bottlerocket/Talos/COS) first. Implication: AMEND-adjacent only in trigger discipline (keep the decision, but the "replace Talos" trigger must be an explicit proven-economics gate, which the ADR already states — so KEEP with the gate emphasized). The unresolved bit is the SOURCE-vs-LINUX masterplan canon, a founder consensus item, not a defect in this ADR.
- **ai_slop:** No. The Rust-vs-Go section is unusually honest (concedes the shared memory-safety win) — anti-slop.
- **refinement:** Tie the "beat-or-parity-vs-Talos scorecard" to a concrete conformance-gate id so it is machine-checkable; note that AWS Bottlerocket (Rust-in-the-node-OS) is the strongest real-world precedent and should be cited as adopted research (ADR-0022 compliance — currently the matrix only cites Talos for node-OS).
- **consensus_needed:** "Does the masterplan canonize running adopted Talos (SOURCE ADR-0375) or building/owning a Rust Talos that replaces it (LINUX ADR-0025)? If both, what is the exact proven-economics trigger and timeline that flips vendored-Talos → owned-node-OS, and is that trigger compatible with SOURCE's Talos+CAPI+ArgoCD fleet investment?"

---

### ADR-0026 — Kernel-level capabilities are ports too: Linux-kernel-extension adapters now → framekernel-Frame later; the choice is per-capability and deferred

- **decision_atom:** Every kernel-level capability is a port designed to the owned framekernel-Frame interface with a Linux-kernel-extension adapter today (eBPF / io_uring / vfio / AF_XDP / Rust-for-Linux) and a Frame adapter later, graduated per-capability only when it passes beat-or-parity-over-a-span within the TCB-ratio ceiling — so kernel-level performance is available on Linux now, both adapters are retained for per-capability rollback, and the Linux-vs-framekernel cutover is gradual, gated, measured, and never a big-bang.
- **domain:** kernel-frame (cross-cutting: isolation-runtime).
- **current_status:** accepted.
- **disposition:** KEEP — the keystone "fractal ratchet" ADR that removes the forced Linux-vs-own-kernel binary; technically credible (the adapter tier table maps real Linux kernel-extension mechanisms correctly). Strongest de-risking move in the LINUX kernel story.
- **proposed_resolution:** NA.
- **governing:** —
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL — "per-capability kernel transition; H2-readiness = fraction graduated to Frame, measured not declared" should be the masterplan's kernel-cutover gauge; currently the masterplan/ADR-0018 expresses H2 as a stage, not as this measured fraction.
- **tensions:** Mild redundancy with ADR-0024 (shared push-down-on-measured-benefit discipline) — complementary, not conflicting (0024 = placement, 0026 = adapter substrate + graduation gate). It refines ADR-0018's H2 ("we are the host") from a declared stage to a *measured fraction-graduated* gauge — a constructive amendment-by-relation that ADR-0018 should back-reference (same cross-ref gap noted for ADR-0023). No cross-repo SOURCE conflict (SOURCE has no framekernel-Frame concept; this is LINUX-owned territory).
- **hyperscaler_challenge:** Aligned — incremental, measured, reversible migration with dual adapters and no big-bang cutover is exactly how hyperscalers migrate kernel/dataplane paths (eBPF/XDP adoption at Meta/Google/Cloudflare; gVisor's incremental syscall coverage). A hyperscaler would make THIS decision. No amend/archive implication.
- **ai_slop:** No. The adapter tier table is accurate and specific; the "fractal ratchet" framing is substantive, not filler.
- **refinement:** Add the back-reference from ADR-0018 so the H2 definition ("fraction graduated to Frame, measured") is discoverable from the host-staging ADR it refines.
- **consensus_needed:** None contested. (Methodologically depends on ADR-0022/0024 staying KEEP, which they do.)

---

## Chunk notes

- **All five = KEEP, all TRUE, all accepted.** This is the cleanest possible chunk: a tightly cross-referenced *methodology + posture cluster* (0022 governing method → 0023 security posture → 0024 layer placement → 0025 node-OS instance → 0026 kernel-port mechanism). None is Proposed, so there are no RATIFY/DROP proposals to account for; none archives or supersedes another. The keystone map's verdict ("internally coherent, deliberately divergent, not slop") holds across the whole slice — these are genuine architecture decisions, not reconciliation bugs or AI slop.
- **No intra-chunk contradictions.** The cluster is mutually reinforcing. The only intra-LINUX item is a **cross-reference-discipline gap**: ADR-0023 and ADR-0026 both *refine/constrain* ADR-0018 (TCB-ratio invariant; H2 = measured-fraction-graduated) but ADR-0018 carries no back-reference to them. This is the SOURCE-internal "stale front-matter / inconsistent cross-ref discipline" disease (keystone fault-line #6) appearing on the LINUX side — flag as AMEND-0018 (out of this chunk's range; noted for the 0018 auditor).
- **Two real cross-repo fault-lines surface here, neither resolvable by an auditor:**
  1. **node-OS (ADR-0025) — SHARPEST:** LINUX "Rust Talos that replaces Talos" vs SOURCE "adopt actual Talos" (ADR-0375). The masterplan cannot canonize both; needs the founder's proven-economics trigger + sequencing decision. AWS Bottlerocket (Rust node-OS) is the strongest real precedent and should be cited.
  2. **cloud-data (referenced in ADR-0022 matrix):** the "owned Rust engine, eliminate-Postgres" stance (LINUX ADR-0001) re-stated as current methodology proof, against SOURCE best-of-breed Postgres+Milvus. Out of this chunk's range (0001) but ADR-0022's matrix re-asserts it, so it is in-scope to flag.
- **Recurring refinement across the chunk (single fix):** the "~14% audited-unsafe TCB" ratio and the "our flavor" differentiator cells are presented as established/measured fact in ADR-0022's matrix and echoed in 0024, but the Frame is not yet built — they are TARGETS. ADR-0022's own Negative #2 ("the flavor must be real / a *measured* edge or it is reuse") demands these be labeled TARGET-vs-MEASURED. Single small amendment to ADR-0022's matrix would fix the corpus-wide echo. (Not enough to drop truth_flag below TRUE — the docs are directionally true and self-aware — but it is the one honesty-of-presentation nit in an otherwise exemplary chunk.)
- **Masterplan-generation readiness:** all five are masterplan-ready *as principles/invariants/gates* (governance methodology, isolation posture, placement rule, node-os component, kernel-cutover gauge) and each has a clean one-sentence decision_atom above. Under BOTH open masterplan readings (ADRs-generate-masterplan vs masterplan-is-authority-ADRs-bind-in, keystone §4) these decisions bind cleanly — they are method/posture, not feature rows, so they survive either direction. The only thing blocking full canonization is the **node-OS cross-repo consensus** (ADR-0025 vs SOURCE ADR-0375).
- **Merge hazard reminder:** all five collide numerically with SOURCE ADR-0022–0026 (build-vs-buy / foundry-capability-registry-adjacent territory in SOURCE). Renumber to ADR-0515+ on merge; never merge at face value (keystone §6.4).
