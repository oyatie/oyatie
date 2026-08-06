# Cross-Tension Register — Theme: isolation / kernel / OS / mesh

> **Auditor:** CONTRADICTION HUNTER, theme `isolation-kernel-os-mesh`.
> **Scope:** LINUX (substrate pilot, `~/Developer/linux`, 26 ADRs) ↔ SOURCE (company monorepo, `~/Developer/source`, 346 ADRs).
> **Method:** Read the real files on both sides (LINUX 0014/0017/0018/0023/0024/0025/0026; SOURCE 0023/0044/0121/0146/0147/0148/0182/0200/0253/0254/0338/0370/0375/0378/0379/0382). Cross-checked against the keystone map and the prior wm4gkcey5 linux register.
> **READ-ONLY.** No audited doc was modified. This file is the only artifact written.
> **Founder goal binding:** masterplan = single source of truth; capture every TRUE+relevant decision; authored-vs-generated masterplan is OPEN — flagged both ways where load-bearing.

---

## 0. Executive shape of the theme

The two sides are **deliberately divergent, not erroneous**, and they have a clean architectural seam: the **Linux syscall ABI**. SOURCE **assembles the isolation substrate** (Talos node-OS + upstream Kubernetes + containerd + Kata/Cloud-Hypervisor + wasmtime + Cilium/Istio-Ambient + Kubewarden). LINUX **owns the isolation substrate from the kernel up** (framekernel + Capsule + `IsolationBackend` + owned VMM + Rust "Talos" node-OS), but writes every component **to the Linux ABI port first** so it runs ON the source stack now and *replaces it under a proven-over-a-span ratchet later*.

The single most important finding: **most of these are NOT flat contradictions.** LINUX positions itself as the *owned successor* to each source substrate, gated behind the shared "own-when-proven" ratchet (LINUX ADR-0019/0020 ≈ SOURCE ADR-0211/0173). The disagreement is almost always the **trigger threshold and the day-0 ownership breadth**, plus a layer of **stale source-internal vocabulary/supersession drift** that pollutes any merge. The genuine founder calls are: (1) does the owned framekernel/Capsule stack supersede the assembled Talos/containerd stack, or coexist as a research track; (2) where each owned component graduates; (3) one cross-side **ADR-0023 number collision** that is a hard data-integrity issue.

The prior linux auto-reconciliation (wm4gkcey5) is **confirmed not "plain wrong"**: the linux ADRs honestly time-box the "we are the host" claim (ADR-0018 `consensus=FALSE`, H1-on-Linux), ground secure-by-default in assume-breach (ADR-0023), and never fabricate a source posture. One residual nit (T-9) is internal to linux ADR-0014's repointed citation.

---

## 1. TENSION INDEX (severity-ordered)

| # | Tension | Type | Governs (current locked decision) | Founder call? |
|---|---|---|---|---|
| T-1 | Isolation substrate: LINUX framekernel/Capsule/own-VMM **vs** SOURCE Talos+containerd+Kata+Cloud-Hypervisor | Reconcilable overlap (own-successor ratchet) — NOT flat contradiction | SOURCE assembled stack governs **now** (ADR-0375/0147/0338); LINUX is the proof-gated owned successor (ADR-0014/0018/0019) | **YES** — coexist-as-research vs supersede-target |
| T-2 | **ADR-0023 NUMBER COLLISION** across sides: LINUX 0023 = isolation-security assume-breach; SOURCE 0023 = foundry sandbox wasmtime+firecracker | Hard data-integrity collision | Both are real, different domains; renumber on merge (keystone §6.4) | **YES** — renumber policy |
| T-3 | Node-OS: LINUX Rust "Talos" (own, beat-or-parity) **vs** SOURCE adopts actual Talos | Reconcilable (vendored-Talos→owned-OS ratchet) — LINUX ADR-0025 self-declares Talos day-0 | SOURCE Talos governs now (ADR-0375/0370/0378/0382); LINUX owned OS is the later ratchet (ADR-0025) | **YES** — is a Rust-Talos a funded goal or aspiration |
| T-4 | "Secure-by-default" axis: LINUX assume-breach microVM-per-pod (authorship is NOT a trust axis) **vs** SOURCE tiered runc-for-first-party (Tier 0..3, ADR-0338) | **True tension on the default**, reconcilable on mechanism | Per-side both internally locked; cross-side UNRESOLVED — opposite defaults | **YES** — fleet default: strong-by-default vs runc-for-first-party |
| T-5 | WASM isolation: SOURCE wasmtime canonical (ADR-0200) **vs** LINUX has NO wasm-runtime ADR; Capsule ladder omits a WASM class | Gap, not contradiction | SOURCE wasmtime governs (ADR-0200, Accepted) | **YES** — does the pilot adopt wasmtime as the 5th Capsule class |
| T-6 | Admission/policy engine: SOURCE Kubewarden-default + Kyverno-adapter (ADR-0379 sup. 0183) **vs** LINUX `IsolationBackend.capabilities()` "no-silent-downgrade" admission gate + conformance-gates | Reconcilable (different layers: k8s-admission vs runtime-class capability gate) | SOURCE ADR-0379 governs k8s admission; LINUX gate is runtime-internal | Minor — cross-ref only |
| T-7 | Service mesh: SOURCE Cilium L3/L4 + Istio-Ambient L7 (ADR-0148) **vs** LINUX (no mesh ADR; framekernel "own-the-host" implies owned dataplane eventually via ADR-0026 eBPF/AF_XDP) | Latent gap + future overlap | SOURCE ADR-0148 governs; LINUX has no competing decision yet | Minor — surface only |
| T-8 | containerd disposition: LINUX "containerd dissolves → CRI/OCI compat adapter; container-platform owns L0–L8" (ADR-0017/0018) **vs** SOURCE keeps upstream containerd+runc (ADR-0121-retired→Talos default, ADR-0338) | Reconcilable overlap at the CRI seam | SOURCE upstream-containerd governs now; LINUX owned container-platform is the successor behind CRI | **YES** — own-the-container-platform breadth |
| T-9 | LINUX-internal: ADR-0014 "no-silent-downgrade" citations repointed to `conformance-gates §1a + capabilities()` by wm4gkcey5; verify the §0-item-5 anchor actually exists | Possible dangling-ref residue from the prior reconcile | conformance-gates.md SSOT governs | No — mechanical verify |
| T-10 | Confidential computing: LINUX owned VMM confidential profile + measured-boot in framekernel (ADR-0026/0014-Backend4) **vs** SOURCE `kata-clh-sev-snp`/`kata-clh-tdx` RuntimeClasses (ADR-0147/0338) | Reconcilable overlap (both SEV-SNP/TDX; own vs adopt VMM) | SOURCE kata-clh-CC governs now; LINUX owned-CC is Stage-5 hardware-gated | Minor — ratchet endpoint |

---

## 2. TENSIONS IN DETAIL

### T-1 — Isolation substrate: own-the-host (framekernel/Capsule) vs assemble-the-substrate (Talos+containerd+Kata) — **KEYSTONE FAULT-LINE #3**

**Positions.**
- **LINUX ADR-0014** (`docs/adr-archive/ADR-0014-build-vs-buy-policy.md`, Accepted): ONE OCI/CRI frontend + pluggable `IsolationBackend` port spanning `Native|Sandbox|Microvm|Confidential`; **owned Rust VMM** behind a `Vmm` port with firecracker-minimal / cloud-hypervisor-rich profiles; guest kernel = the framekernel. Reuse rust-vmm / Firecracker / Cloud-Hypervisor **now** (Stage 2), own the VMM **later** (Stage 3) only on a benchmark gain.
- **LINUX ADR-0018** (`ADR-0018-host-framekernel-and-capsule-model.md`, **accepted-with-reservations**, `consensus=FALSE`): the Capsule unifies container/VM/pod under one trait; "we are the host; framekernel's own isolation, no separate containerd" — but **literal only at the optional, uncommitted H2 (year 5+)**; H1 (years 1–3, the committed product) runs the flagship Native Capsule **on Linux**, framekernel's only H1 role is the microVM guest kernel.
- **LINUX ADR-0017** (`ADR-0017-container-platform-full-stack.md`): extends the runtime to a full owned L0–L8 `oya-cloud-container-platform-*`; "containerd → container-platform" rename; containerd dissolves to a CRI/OCI compat adapter (see T-8).
- **SOURCE ADR-0375** (`ADR-0375-talos-capi-argocd-fleet-substrate.md`, Accepted, `planning_impact:true`, supersedes 0120/0121): Talos immutable node-OS + Cluster API + ArgoCD; **Kata + Cloud Hypervisor worker pools** for untrusted workloads; Cilium+Istio mesh. This is Oyatie's own OKE/GKE/EKS product.
- **SOURCE ADR-0147** (`ADR-0147-container-sandboxing-runtime-ladder.md`, Amended): workload-class runtime ladder — bare runc for app-tier, **Kata+Cloud-Hypervisor (`kata-clh`)** primary for untrusted content, `kata-fc` per-request, `wasmtime` for WASM. gVisor demoted to opt-in.
- **SOURCE ADR-0338** (`ADR-0338-pod-runtime-tier-0-to-3.md`, Proposed): Tier 0..3 — Kata+CLH for tenant-untrusted (T0) and tenant-data-plane substrate (T1); **runc for first-party app (T2) + edge (T3)**.

**Type.** **Reconcilable overlap, NOT a flat contradiction.** Both sides agree on the *primitives* (KVM microVMs, Cloud-Hypervisor/Firecracker device models, SEV-SNP/TDX, OCI/CRI, RuntimeClass selection). LINUX even names Firecracker/Cloud-Hypervisor as its Stage-2 reused backends and admits "we rewrite it in Rust buys nothing since it IS already Rust" (ADR-0014 §MUST-FIX HONESTY). The divergence is **ownership depth + the host claim**: SOURCE *consumes* Kata+CLH on Talos; LINUX *owns* the VMM, the guest kernel, and (only at H2) the host. Both are governed by the same proven-over-a-span ratchet (LINUX ADR-0019; SOURCE ADR-0211 own-when-proven).

**Which governs.** For the running system **today and through H1**, the **SOURCE assembled stack governs** — it is Accepted, `planning_impact:true`, dogfood-deployed (ADR-0375), and LINUX ADR-0018 itself concedes H1 runs on Linux with an *external* hypervisor "owned by the cloud node below us." The LINUX owned stack is the **declared destination**, gated; it does not supersede source today. This matches keystone §5 (own-when-proven; disagreement is the trigger threshold).

**Proposed resolution (surgical).**
1. **Cross-ref, both directions.** Add to LINUX ADR-0014/0018 `related:` a pointer to SOURCE ADR-0375 (Talos fleet substrate) + ADR-0147/0338 (runtime ladder/tiers) noting "SOURCE assembles this substrate today; LINUX owns it behind the Linux-ABI port under the ADR-0019 ratchet." Add to SOURCE ADR-0375/0147 a note that the LINUX pilot is the owned-substrate successor track. (No new policy; pure linkage — but this is a WRITE on audited docs, so it is a **recommendation for the reconcile pass, not done here.**)
2. **Masterplan capture:** record ONE "isolation-substrate" masterplan node with two phases: *assembled (Talos+Kata+CLH, current authority)* → *owned (framekernel/Capsule/own-VMM, ratchet endpoint)*. Bind LINUX ADR-0014/0018/0019 and SOURCE ADR-0375/0147/0338 to it.

**DECISION-NEEDED-FROM-FOUNDER.** *Is the owned framekernel + Capsule + owned-VMM stack the committed long-horizon REPLACEMENT for the assembled Talos+containerd+Kata+Cloud-Hypervisor substrate (i.e. a funded destination on the ADR-0019 ratchet), or a parallel research track that must beat the assembled stack on the conformance-gates scorecard before any production role? The H2 "we are the host" grade is explicitly uncommitted in LINUX ADR-0018 — does the masterplan record framekernel-as-host as a TARGET or as RESEARCH?*

**Disposition impact.** None archive. LINUX ADR-0014/0017/0018 stay KEEP/AMEND-for-cross-ref. SOURCE ADR-0375/0147/0338 stay KEEP (per the 54-auditor table 0147=amend, 0375=keystone-canonical).

---

### T-2 — ADR-0023 NUMBER COLLISION across sides — **DATA-INTEGRITY**

**Positions.**
- **LINUX ADR-0023** = `ADR-0023-isolation-security-posture-assume-breach.md` (Accepted, council-security): isolation strength by blast-radius × data-sensitivity × attack-surface; assume-breach; microVM-per-pod default; native is a classified downgrade.
- **SOURCE ADR-0023** = `ADR-0023-foundry-sandbox-wasmtime-firecracker.md` (Proposed, owner `foundry`): two-tier Foundry tool sandbox — Wasmtime+WASI-P2 for short tools, Firecracker microVMs for full-kernel tools. **Retired-vocab:** owner `foundry` (RETIRED→intelligence per ADR-0335), `oya-foundry-sandbox-kernel` crate.

**Type.** **Hard collision** — two authoritative ADRs share id 0023 in different repos, different domains. This is the keystone §6.4 guaranteed-collision-on-merge, made concrete in the most security-sensitive theme. A generated-from-ADRs masterplan graph keyed on `ADR-NNNN` would **silently merge two unrelated isolation decisions**.

**Which governs.** Both are real and survive on their own side. The SOURCE 0023 is Proposed + retired-vocab-laden; the LINUX 0023 is Accepted. Neither supersedes the other — they are *different decisions*.

**Proposed resolution (surgical).** On merge, **renumber ALL 26 LINUX pilot ADRs to ADR-0515+** (keystone §6.4 already mandates this; every LINUX ADR carries `renumber_note`). Specifically flag the 0023 pair as a *semantic* collision (assume-breach posture vs foundry tool sandbox) so the renumber map does not conflate them. Independently, SOURCE 0023 needs the foundry→intelligence brand rename (its own disposition is `amend`).

**DECISION-NEEDED-FROM-FOUNDER.** *Confirm the LINUX pilot series renumbers to a fresh ADR-0515+ block on merge (never merged at face value), and confirm whether a strict no-reuse / no-dangling-ref invariant on ADR ids is adopted — mandatory if the masterplan is GENERATED from ADR supersede-edges (the open §4 authored-vs-generated question).*

**Disposition impact.** No archive; renumber-on-merge for all LINUX ADRs. SOURCE ADR-0023 → AMEND (foundry brand).

---

### T-3 — Node-OS: Rust "Talos" (own) vs adopt actual Talos

**Positions.**
- **LINUX ADR-0025** (`ADR-0025-node-os-rust-talos-and-rust-vs-go-security.md`, Accepted): an immutable, API-managed, Kubernetes-compatible node OS **in Rust** = a "Rust Talos"; **Talos-config-compatible** during transition; beat-or-parity-vs-Talos scorecard; a Rust-vs-Go security thesis (no GC, compile-time data-race freedom, bounded unsafe). **Critically, it self-declares the ratchet:** *"Talos day-0 (vendored OS on the Linux kernel) → our node-OS replaces Talos when proven."*
- **SOURCE ADR-0375 / 0370 / 0378 / 0382**: Talos is THE node OS (immutable, API-managed, no-SSH), via USB zero-touch / vfkit-local / Sidero bare-metal / CAPI cloud images. Accepted, dogfood-deployed.

**Type.** **Reconcilable** — LINUX ADR-0025 *adopts Talos day-0* and only ratchets to an owned OS later. This is the cleanest example of the shared own-when-proven ratchet. Keystone §5.5 / chunk-14 already flagged it ("LINUX ADR-0025 re-opens it wanting a Rust 'Talos'").

**Which governs.** SOURCE Talos governs now (ADR-0375 Accepted `planning_impact:true`). LINUX owned node-OS is the proof-gated successor; ADR-0025 explicitly defers ("home: this is jason931225/oyatie/source's roadmap; the linux pilot is staging").

**Proposed resolution (surgical).** Cross-ref LINUX ADR-0025 ↔ SOURCE ADR-0375 (note Talos is the day-0 vendored adapter; owned Rust-OS is the ratchet endpoint). Masterplan: one "node-OS" node, phase *vendored-Talos (authority now)* → *owned-Rust-OS (ratchet, beat-or-parity gated)*.

**DECISION-NEEDED-FROM-FOUNDER.** *Is a Rust "Talos" a committed funded destination (the OS-replacement leg of the ratchet, ahead of the kernel-replacement leg per ADR-0025 §1), or is Talos the permanent node-OS and the Rust-OS thesis aspirational? The Rust-vs-Go security advantage is explicitly "structural, not yet proven" (ADR-0025 §3 caveat).*

**Disposition impact.** None archive. LINUX ADR-0025 KEEP. SOURCE ADR-0121 already `Superseded by ADR-0375` (correct on disk).

---

### T-4 — Secure-by-default axis: assume-breach-strong vs runc-for-first-party — **TRUE TENSION on the default**

**Positions.**
- **LINUX ADR-0023** (Accepted): **default to the strong boundary (microVM-per-pod) even for first-party services**; "it's ours" is NOT a license for weaker isolation; **authorship is explicitly NOT a strength axis**; native is a deliberate, risk-classified, hardened-only downgrade for the genuinely-low-blast-radius tail. Grounded in BeyondProd / Nitro / NIST 800-207.
- **SOURCE ADR-0338** (Proposed): the WHOLE POINT is to **stop using Kata everywhere** — Tier 2 = first-party application µservices run **runc** (namespace-isolated + mTLS + Cedar-policed deemed sufficient); Kata+CLH reserved for tenant-untrusted (T0) + tenant-data-plane substrate (T1). Rationale: Kata costs 30–40% pod density + 200–500ms cold-start "for zero marginal security benefit" on first-party code.

**Type.** **Genuine cross-side tension on the DEFAULT**, though reconcilable on mechanism (both use RuntimeClass selection; both keep a microVM class for the highest-risk tier; both keep a cheap class for the low-risk tail). The *axis of selection collides*: SOURCE selects by **trust/provenance** (first-party→runc); LINUX ADR-0023 **explicitly rejects authorship as the axis** and selects by blast-radius/data-sensitivity/attack-surface, defaulting first-party to microVM. SOURCE ADR-0147 even uses the inverse phrasing ("first-party Rust services… zero marginal security gain"). LINUX ADR-0023's own §"How memory-safety fits" notes the memory-safe kernel lets *more* low-blast-radius workloads safely take the native downgrade — a partial bridge, but the **default** still differs.

**Which governs.** Per-side, each is internally locked and self-consistent. **Cross-side this is UNRESOLVED** and is the sharpest *policy* disagreement in the theme (distinct from the *substrate* disagreement T-1). It is a real founder call, not a drift artifact.

**Proposed resolution (surgical — clarify, do not invent policy).** The two are **not formally contradictory if scoped**: LINUX ADR-0023 is a *pilot-substrate security posture*; SOURCE ADR-0338 is a *fleet cost/density posture for the current assembled stack*. The reconcilable framing already latent in both: *default-strong where the memory-safe-kernel microVM is cheap (LINUX's thesis is that its lean guest kernel makes microVMs cheaper than Linux-guest Kata, dissolving the 30–40% tax that motivates ADR-0338's runc-for-first-party)*. Add a cross-ref noting that LINUX ADR-0023's default-strong posture is **predicated on the lean-framekernel-guest cost assumption**; if that assumption is proven (cheap microVMs), ADR-0338's density rationale weakens and the defaults converge.

**DECISION-NEEDED-FROM-FOUNDER.** *What is the canonical fleet ISOLATION DEFAULT for first-party services: (a) SOURCE ADR-0338 "runc for first-party, Kata only for tenant-untrusted + data-plane substrate" (density-optimized on the assembled stack), or (b) LINUX ADR-0023 "assume-breach, microVM-per-pod even for first-party, authorship is not a trust axis" (security-optimized, betting the lean framekernel guest makes microVMs cheap enough to erase the density tax)? These pick OPPOSITE defaults for the ~60 first-party µservices.*

**Disposition impact.** No archive. LINUX ADR-0023 KEEP (+cross-ref). SOURCE ADR-0338 stays Proposed/KEEP but the default-axis note should be recorded against the merged decision.

---

### T-5 — WASM isolation: SOURCE wasmtime canonical vs LINUX Capsule ladder has no WASM class

**Positions.**
- **SOURCE ADR-0200** (`ADR-0200-wasm-runtime-canonical-wasmtime.md`, Accepted): Wasmtime + WASI Preview 2 Component Model is the canonical in-process WASM sandbox (Envoy filters, workflow-studio nodes, foundry/intelligence tool sandbox). Apache-2.0. No in-house engine planned — own the *integration layer*, not the runtime. SOURCE ADR-0147 also lists a `wasmtime` RuntimeClass (runwasi) for WASM-only workers.
- **SOURCE ADR-0023** (Proposed, foundry): Wasmtime+WASI-P2 for short tools (the in-process tier feeding the Firecracker microVM tier).
- **LINUX**: the Capsule / `IsolationBackend` ladder is `Native | Sandbox(framekernel-Sentry) | Microvm | Confidential` — **there is no WASM class**, and **no LINUX ADR adopts wasmtime**. The "Sandbox" class is a gVisor-style framekernel Sentry, not a WASM bytecode sandbox.

**Type.** **Gap, not contradiction.** WASM (in-process bytecode sandbox) and the framekernel Sentry (process-level Linux-ABI sandbox) are *different isolation primitives at different layers* — they coexist in SOURCE (ADR-0200 §"Native sub-process per tenant… the two coexist"). LINUX simply has no decision in this lane.

**Which governs.** SOURCE ADR-0200 governs WASM isolation (Accepted). LINUX is silent.

**Proposed resolution (surgical).** No conflict to resolve — note in the masterplan that wasmtime (ADR-0200) is the canonical in-process WASM tier and that the LINUX Capsule ladder is a *complementary* process/VM-level tier (the same coexistence SOURCE already records). If the pilot ever needs WASM workers, it adopts wasmtime (a vendored-now port), not a new framekernel mode.

**DECISION-NEEDED-FROM-FOUNDER.** *Does the pilot's Capsule ladder gain an explicit WASM class that adopts wasmtime (ADR-0200) as a vendored adapter — i.e., wasmtime is the 5th isolation strength alongside Native/Sandbox/Microvm/Confidential — or does WASM stay out of the substrate-pilot scope as a pure application-tier concern?*

**Disposition impact.** None. SOURCE ADR-0200 KEEP. No LINUX ADR affected (gap to note).

---

### T-6 — Admission/policy engine: Kubewarden-default (ADR-0379) vs LINUX capability-gate no-silent-downgrade

**Positions.**
- **SOURCE ADR-0379** (Accepted, supersedes ADR-0183): **Kubewarden** is the default k8s admission substrate (WASM policy modules, Rust→WASM), Kyverno demoted to first-class adapter; Cedar remains app-layer authz. ADR-0338's `enforce-pod-runtime-tier` is authored as a Kyverno policy in-text but the canonical engine is now Kubewarden.
- **LINUX ADR-0014/0024**: admission decisions are a **runtime-class capability gate** — `IsolationBackend::capabilities()` makes "no-silent-downgrade" mechanical (a pod requesting confidential on a node without SEV-SNP fails admission loudly), anchored in `conformance-gates §1a`. This is a *userspace admission policy in the k8s control plane* (ADR-0014 explicitly: "set in the k8s control plane, never baked into the kernel").

**Type.** **Reconcilable — different layers.** SOURCE Kubewarden = k8s-resource admission (image signing, PSS, RuntimeClass allow-list, tier enforcement). LINUX capability-gate = runtime-internal capability/RuntimeClass-strength check. They compose: Kubewarden/Kyverno would be the *admission webhook* that enforces the LINUX RuntimeClass→backend mapping and the no-silent-downgrade rule at the cluster boundary.

**Which governs.** SOURCE ADR-0379 governs k8s admission (Accepted). LINUX gate is the runtime's own capability check below it.

**Proposed resolution (surgical).** Cross-ref: note that the LINUX no-silent-downgrade rule (capabilities()) is enforced at the cluster edge by the canonical admission engine (Kubewarden per ADR-0379). No policy conflict. **Watch:** SOURCE ADR-0338 still spells `enforce-pod-runtime-tier` as a *Kyverno* policy — that text should repoint to Kubewarden per ADR-0379 (a SOURCE-internal amend, already in 0338's disposition implicitly).

**DECISION-NEEDED-FROM-FOUNDER.** None — mechanical cross-ref + the source-internal Kyverno→Kubewarden repoint already governed by ADR-0379.

**Disposition impact.** SOURCE ADR-0183 ARCHIVE (already `Superseded`); ADR-0379 KEEP. LINUX unaffected.

---

### T-7 — Service mesh: SOURCE Cilium+Istio-Ambient vs LINUX latent owned-dataplane

**Positions.**
- **SOURCE ADR-0148** (Accepted): Cilium 1.19.x L3/L4 (CNI, eBPF, Hubble, WireGuard) + Istio Ambient L7 (ztunnel Rust mTLS + waypoint Envoy + Cedar `ext_authz`), zero overlap. ADR-0182 separates north-south (Gateway API) from east-west. ADR-0253 = network topology. ADR-0044 (Istio Ambient, **Proposed, foundry-owned**) is the *earlier* mesh ADR the keystone flags as un-superseded; ADR-0148 rewrites its framing (Cilium primary).
- **LINUX**: **no mesh ADR.** But ADR-0026 (kernel-level capabilities as ports) names **eBPF / AF_XDP / XDP dataplane** as owned kernel-extension adapters, and ADR-0024 places the "dataplane fast path (XDP/eBPF-class)" at the safe-kernel/Frame layer. The "we are the host" H2 vision (ADR-0018) implies an eventually-owned dataplane.

**Type.** **Latent gap + future overlap.** No live contradiction — LINUX has no mesh decision. The future tension: if the framekernel owns the host (H2), the eBPF/Cilium dataplane and the ztunnel become candidates for the owned-substrate ratchet, overlapping ADR-0148. Today they don't touch.

**Which governs.** SOURCE ADR-0148 governs the mesh (Accepted, canonical). The keystone notes ADR-0044 has "no canonical-posture-map §3 row" historically; ADR-0148 is the live authority. ADR-0044 (Proposed, foundry) should be marked superseded-in-framing by ADR-0148.

**Proposed resolution (surgical).** Surface only (per instructions). Note in the masterplan that mesh = ADR-0148 (Cilium+Istio-Ambient); the LINUX owned-dataplane (eBPF/AF_XDP per ADR-0026) is a *future ratchet candidate* under the same own-when-proven gate, not a current competitor. SOURCE-internal: ADR-0044 framing superseded by ADR-0148 (a source amend).

**DECISION-NEEDED-FROM-FOUNDER.** *(Long-horizon, low-urgency)* *If the framekernel reaches H2 host-grade, does the owned eBPF/AF_XDP dataplane (ADR-0026) eventually ratchet-replace the Cilium L3/L4 layer, or is Cilium+Istio-Ambient (ADR-0148) a permanent KEEP (ADR-0148 classifies both as "IS-the-standard, no in-house replacement planned")?*

**Disposition impact.** SOURCE ADR-0044 → AMEND (foundry brand + framing superseded by 0148). ADR-0148 KEEP. No LINUX ADR affected.

---

### T-8 — containerd disposition: dissolves to compat adapter (LINUX) vs upstream containerd+runc (SOURCE)

**Positions.**
- **LINUX ADR-0017/0018**: containerd is RENAMED to `container-platform` and **dissolves into a CRI/OCI compat adapter** translating to the Capsule contract; LINUX owns L0–L8 (`oya-cloud-container-platform-*`). containerd's Go plugin reflection, conmon C binary, bbolt, Docker daemon are all dropped/replaced in Rust (ADR-0017 §worth-preserving filter).
- **SOURCE**: upstream **containerd + runc** is the runtime under Talos (ADR-0375 "Kata worker pools"; ADR-0338 D-3 "runc-pool MUST run containerd with runc"; ADR-0146 distroless base images run "in gVisor sandboxes per the foundry/mail/recordings tiering"). SOURCE never decides to rewrite containerd in Rust.

**Type.** **Reconcilable overlap at the CRI seam.** Both expose CRI to kubelet. LINUX owns the implementation behind CRI; SOURCE consumes upstream containerd. This is the same own-vs-assemble axis as T-1, at the container-platform granularity. Keystone chunk: LINUX ADR-0017 self-flags L7 sprawl risk and grounds reuse (Sigstore/Trivy PERMANENT_REUSE) in ADR-0020.

**Which governs.** SOURCE upstream containerd+runc governs now (it is what Talos ships). LINUX owned container-platform is the successor behind the stable CRI/OCI contract.

**Proposed resolution (surgical).** Cross-ref ADR-0017 ↔ SOURCE ADR-0375/0338 (note containerd is the vendored CRI provider now; the owned container-platform impedance-matches the same CRI/OCI external-standard ports). Masterplan: fold under the T-1 isolation-substrate node (container-platform is the L4–L8 extension of the same own-vs-assemble decision).

**DECISION-NEEDED-FROM-FOUNDER.** *Is owning the full L0–L8 container platform (`oya-cloud-container-platform-*`, ADR-0017 — including an LLB build engine, a zot-class registry, a Rust conmon, snapshotters) in-scope day-0/ratchet, or does the pilot reuse upstream containerd+runc behind CRI and own only the isolation backends (ADR-0014 L0–L3)? ADR-0017 itself flags L7 build as "a second mountain / different product class."*

**Disposition impact.** None archive. LINUX ADR-0017 KEEP/AMEND-for-cross-ref.

---

### T-9 — LINUX-internal: ADR-0014 no-silent-downgrade citation anchor (verify wm4gkcey5 edit)

**Position / finding.** The prior linux reconcile (wm4gkcey5) repointed ADR-0014's five "no-silent-downgrade" citations from bare `conformance-gates §0` to **`conformance-gates §1a + the capabilities() mechanism`** (visible at ADR-0014 lines 66, 103, 137, 407, 577), and the reconcile log claims it ALSO added a `conformance-gates §0 item 5` anchor (`#no-silent-downgrade`). ADR-0019's text (per the prior register) was rewritten to say "the no-silent-downgrade rule now has a real anchor at conformance-gates §0 item 5."

**Type.** **Possible internal inconsistency residue**, NOT a cross-side tension. The ADR-0014 body now cites `§1a + capabilities()` while ADR-0019 points at `§0 item 5` — two different anchors for the same rule. Both resolve to *something* (capabilities() is real, §1a is real), so it is not a hard dangling ref, but the **anchor naming is split** across ADR-0014 vs ADR-0019.

**Verdict on the wm4gkcey5 edit.** **NOT "plain wrong"** — the repoint is internally coherent and the capabilities() mechanism genuinely enforces the rule. The only nit: ADR-0014 says `§1a`, ADR-0019 says `§0 item 5`. A future docs pass should unify on one anchor name.

**Proposed resolution (surgical).** Mechanical: confirm `conformance-gates.md §0` actually contains the `item 5 / #no-silent-downgrade` anchor the reconcile claims to have added; if present, repoint ADR-0014's five citations to it for consistency with ADR-0019; if absent, the `§1a + capabilities()` citation in ADR-0014 is the safe one and ADR-0019 should match it. **Verification, not a founder call.**

**DECISION-NEEDED-FROM-FOUNDER.** None.

**Disposition impact.** None. LINUX ADR-0014/0019 KEEP (minor anchor-unification nit for the reconcile pass).

---

### T-10 — Confidential computing: owned VMM CC profile vs kata-clh-sev-snp/tdx

**Positions.**
- **LINUX ADR-0014 Backend-4 + ADR-0026**: Confidential = owned-VMM confidential profile (AMD SEV-SNP / Intel TDX / Arm CCA) + measured boot in the framekernel boot Frame + remote attestation; Stage-5, **hardware-gated** (Apple-Silicon dev host has no x86 KVM). ADR-0024 places crypto/measured-boot/attestation at safe-kernel(logic)+Frame(TPM/SEV).
- **SOURCE ADR-0147/0338**: `kata-clh-sev-snp` (AMD SEV-SNP) + `kata-clh-tdx` (Intel TDX) RuntimeClasses for crypto / sovereign / GPU-CC workloads; CC via Kata+Cloud-Hypervisor, not an owned VMM.

**Type.** **Reconcilable overlap** — identical hardware primitives (SEV-SNP/TDX), own-vs-adopt the VMM that drives them. Same axis as T-1.

**Which governs.** SOURCE kata-clh-CC governs now (Accepted ADR-0147). LINUX owned-CC is the Stage-5 ratchet endpoint, gated on CC hardware procurement.

**Proposed resolution (surgical).** Cross-ref; fold under the T-1 isolation-substrate masterplan node as the confidential phase. The differentiator LINUX names (confidential-launch co-design with the framekernel measured boot, ADR-0014 §MUST-FIX) is the only thing that would license owning the CC VMM over reusing kata-clh-sev-snp — that benchmark gate is the trigger.

**DECISION-NEEDED-FROM-FOUNDER.** Subsumed by T-1 (owned-VMM-vs-kata trigger threshold).

**Disposition impact.** None archive. LINUX ADR-0026/0014 KEEP. SOURCE ADR-0147 AMEND (per existing disposition).

---

## 3. RETIRED-VOCABULARY & SUPERSESSION DRIFT touching this theme (from the keystone, confirmed on disk)

These are SOURCE-internal hygiene items that pollute the isolation/mesh theme on merge — not cross-side tensions, but they must be clean before masterplan backfill:

- **`foundry` brand (RETIRED → cloud-intelligence/governance, ADR-0335/0347)** leaks into theme ADRs: SOURCE ADR-0023 (owner `foundry`, `oya-foundry-sandbox-kernel`), ADR-0044 (owner `foundry`), ADR-0121 (`axis-foundry`), ADR-0200 (`foundry-tool` sandbox class + `Foundry substrate` context), ADR-0379 related ADR-0023. **All AMEND-for-vocab, decisions sound.**
- **ADR-0121** (onprem kubeadm+containerd+istio) correctly `Superseded by ADR-0375` on disk — ARCHIVE candidate (keystone §1.1). Its containerd+istio framing is dead; Talos+Cilium+Istio-Ambient governs.
- **ADR-0183** (Cedar/Kyverno separation) correctly `Superseded by ADR-0379` — ARCHIVE (Cedar-app-authz-vs-admission-engine SEPARATION principle survives; Kyverno→Kubewarden).
- **ADR-0044** (Istio Ambient, Proposed) framing superseded by ADR-0148 (Cilium primary) but **not marked** — stale-front-matter drift (keystone §6). AMEND.
- **gVisor demotion:** ADR-0146 still says µservices "run in gVisor sandboxes per the foundry/mail/recordings tiering" — stale vs ADR-0147 amendment (Cloud-Hypervisor primary, gVisor opt-in only) + ADR-0338 (runc for first-party). Minor stale cross-ref.
- **`wasmtime` RuntimeClass** appears in both ADR-0147 (runwasi handler) and ADR-0200 (in-process) — NOT a conflict (container-level vs in-process WASM), but disambiguate in the masterplan so synthesis doesn't log a spurious duplicate.

---

## 4. RESULTING DISPOSITION CHANGES (this theme's contribution)

| ADR | Side | Prior/independent disposition | This-theme delta | Reason |
|---|---|---|---|---|
| LINUX 0014 | linux | KEEP (Accepted) | KEEP + cross-ref to SOURCE 0375/0147/0338; minor anchor-unify (T-9) | own-successor linkage; T-1/T-9 |
| LINUX 0017 | linux | KEEP (Accepted) | KEEP + cross-ref to SOURCE 0375/0338 | container-platform own-vs-assemble; T-8 |
| LINUX 0018 | linux | KEEP-with-reservations | KEEP; H2 host-claim flagged as founder TARGET-vs-RESEARCH | T-1 keystone fault-line |
| LINUX 0023 | linux | KEEP (Accepted) | KEEP + **renumber-on-merge** (0023 collision); default-axis founder flag | T-2, T-4 |
| LINUX 0024 | linux | KEEP | KEEP | T-4/T-6 support |
| LINUX 0025 | linux | KEEP | KEEP + cross-ref SOURCE 0375 | T-3 own-node-OS ratchet |
| LINUX 0026 | linux | KEEP | KEEP + cross-ref SOURCE 0148 (future dataplane) | T-7/T-10 |
| SOURCE 0023 | source | amend (foundry brand) | **AMEND** confirmed + cross-side renumber flag | T-2 collision |
| SOURCE 0044 | source | amend (foundry brand) | **AMEND** + framing superseded-by-0148 | T-7 |
| SOURCE 0121 | source | archive (Superseded) | **ARCHIVE** confirmed | retired by 0375 |
| SOURCE 0146 | source | keep | KEEP + minor stale gVisor cross-ref note | §3 hygiene |
| SOURCE 0147 | source | amend | **AMEND** (CLH-primary already; foundry-related cross-refs) | T-1/T-4/T-10 |
| SOURCE 0148 | source | keep (canonical mesh) | KEEP (promote to canonical mesh authority; supersedes 0044 framing) | T-7 |
| SOURCE 0183 | source | archive (Superseded) | **ARCHIVE** confirmed | retired by 0379 |
| SOURCE 0200 | source | keep | KEEP (canonical WASM) | T-5 |
| SOURCE 0338 | source | keep (Proposed) | KEEP; default-axis founder flag (T-4); Kyverno→Kubewarden repoint (T-6) | T-4/T-6 |
| SOURCE 0375 | source | keystone-canonical | KEEP (canonical fleet substrate) | T-1/T-3 |
| SOURCE 0379 | source | keep (Accepted) | KEEP (canonical admission) | T-6 |

---

## 5. FOUNDER QUESTIONS — consolidated (crisp)

1. **(T-1, keystone) Owned-substrate vs assembled-substrate end-state.** Is the framekernel + Capsule + owned-VMM stack the committed long-horizon REPLACEMENT for Talos+containerd+Kata+Cloud-Hypervisor (a funded destination on the ADR-0019 proven-over-a-span ratchet), or a research track that must beat the assembled stack before any production role? Is framekernel-as-host (H2) a masterplan TARGET or RESEARCH?
2. **(T-4) Fleet isolation default for first-party services.** SOURCE ADR-0338 "runc for first-party, Kata only for tenant-untrusted + data-plane" (density-optimized) **or** LINUX ADR-0023 "assume-breach, microVM-per-pod even for first-party, authorship is not a trust axis" (security-optimized, betting the lean framekernel guest erases the density tax)? Opposite defaults for ~60 µservices.
3. **(T-3) Rust "Talos" node-OS.** Funded destination (OS-replacement leg of the ratchet, ahead of kernel-replacement per ADR-0025) or permanent Talos adoption with the Rust-OS thesis aspirational?
4. **(T-8) Container-platform ownership breadth.** Own full L0–L8 (`oya-cloud-container-platform-*` incl. LLB build engine, zot-class registry, Rust conmon) or reuse upstream containerd+runc behind CRI and own only the L0–L3 isolation backends? (ADR-0017 self-flags L7 build as "a second mountain.")
5. **(T-5) WASM in the Capsule ladder.** Add a 5th WASM Capsule strength adopting wasmtime (ADR-0200) as a vendored adapter, or keep WASM out of the substrate-pilot scope?
6. **(T-2, cross-cutting) ADR id discipline.** Confirm all 26 LINUX pilot ADRs renumber to ADR-0515+ on merge (the 0023 pair is a semantic collision: assume-breach vs foundry-sandbox), and decide whether a strict no-reuse/no-dangling-ref ADR-id invariant is adopted — mandatory if the masterplan is GENERATED from ADR supersede-edges (open §4 authored-vs-generated question).

---
*End of theme register. All findings re-verified against on-disk files. No audited doc modified; this is the sole artifact written.*
