---
id: ADR-0023
status: Superseded
superseded_by: [ADR-709]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Foundry→intelligence sandbox naming; Wasmtime/Firecracker posture remains

# ADR-0023: intelligence sandbox — Wasmtime + WASI Preview 2 for short-lived tools, Firecracker microVMs for full-kernel tools

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0020 (provider adapter), ADR-0021 (capability registry — capability declares which sandbox class it needs), ADR-0022 (autonomy ceiling — sandbox is process-level defense-in-depth), ADR-0025 (engineering platform — sandbox-spawn metering rolls into scorecards)

---

## Context

Capabilities invoke tools — and a tool can be anything from a small deterministic transform (parse JSON, render a template, run a regex) to something that needs a full Linux kernel surface (run a compiler, exec a vendor CLI, mount a working filesystem, open arbitrary network sockets). Running every tool in a single shared process is unacceptable: a tool fault crashes the daemon, a tool exfiltration leaks the daemon's secrets, a tool resource hog starves every other invocation, and the autonomy ceiling provides no protection against a tool that has the daemon's process-level capabilities.

We need two-tier process isolation: a **fast** path for short-lived deterministic tools where we measure spawn overhead in tens of milliseconds, and a **strong** path for tools that need full kernel surface area where we measure spawn overhead in low single-digit seconds. Both paths must enforce per-tool resource caps, per-tool egress allowlists, per-tool filesystem mount, and per-spawn audit emission. Both paths must integrate with the autonomy ceiling (ADR-0022) such that policy-permitted tools still cannot escape the sandbox.

---

## Decision

We adopt a two-tier sandbox: **Wasmtime + WASI Preview 2** for short-lived deterministic tools and **Firecracker microVMs** for tools that require a full kernel surface. The capability declares its sandbox class in the registry; the runtime selects the substrate; both substrates share a uniform per-spawn audit emission and a uniform resource-cap contract.

### Sandbox kernel (`oya-intelligence-sandbox-kernel`)

```rust
// crates/oya-intelligence-sandbox-kernel/src/lib.rs
pub enum SandboxClass {
    Wasm,        // short-lived, deterministic, WASI Preview 2
    Firecracker, // full kernel, microVM
}

pub struct SandboxSpec {
    pub class: SandboxClass,
    pub resource_caps: ResourceCaps {
        pub cpu_millis: u32,
        pub memory_mb: u32,
        pub wallclock_ms: u32,
        pub syscall_allowlist: SyscallSet,           // Firecracker only
    },
    pub network: NetworkPolicy {
        pub egress_allowlist: Vec<EgressTarget>,     // (host, port, proto); deny-by-default
        pub dns_resolver: DnsResolverChoice,
    },
    pub filesystem: FsPolicy {
        pub agent_worktree: PathBuf,                 // per-agent scratch
        pub mounts: Vec<Mount {                      // read-only by default
            host_path: PathBuf,
            guest_path: PathBuf,
            mode: MountMode,                          // RO | RW | ExecOnly
        }>,
    },
    pub provenance: ToolProvenance {
        pub tool_id: ToolId,
        pub digest: ContentDigest,                   // for Wasm: module hash; for Firecracker: rootfs hash
        pub signature: Option<CosignAttestation>,
    },
}

pub trait Sandbox: Send + Sync {
    fn spawn(&self, spec: &SandboxSpec) -> SpawnResult;
    fn exec(&self, handle: &SpawnHandle, input: ToolInput) -> impl Stream<Item = ToolEvent>;
    fn teardown(&self, handle: SpawnHandle) -> TeardownResult;
}
```

### Wasmtime path (`oya-intelligence-sandbox-wasm-app`)

- Runtime: Wasmtime with WASI Preview 2 (component model).
- Cold-start budget: **p99 < 2 s**; **p99 < 100 ms warm** (module cache + instantiation reuse).
- Use cases: text transforms, schema validators, deterministic codegen, regex matchers, JSON/YAML parsing, capability-internal helpers.
- Networking: deny-by-default; tools that need egress must declare it in the capability YAML and pass the allowlist gate.
- Filesystem: WASI preopens limited to the per-agent worktree (read-only by default; RW preopens require explicit declaration).
- Determinism: tools opting into determinism mode get a fixed clock and a deterministic PRNG seed for reproducibility (ADR-0024 replay needs this).

### Firecracker path (`oya-intelligence-sandbox-firecracker-app`)

- Runtime: Firecracker microVM with a hardened minimal Linux rootfs (Alpine base, jailer enforced).
- Cold-start budget: **p99 < 2 s** (kernel + rootfs preboot pool); warm pool of pre-booted VMs accepts injected workloads.
- Use cases: vendor CLIs (provider Codex/Claude/Gemini wrapper invocations when the API path is unavailable), language toolchains (cargo/rustc, node/npm, python/pip, go), heavyweight workflow runners.
- Networking: deny-by-default with a per-tool egress allowlist; egress goes through a per-microVM SOCKS proxy that re-checks the allowlist at the proxy.
- Filesystem: per-agent worktree mounted RW; provider/tool binaries mounted RO from a content-addressed store; no host paths leak through.
- Syscall surface: seccomp-bpf allowlist per tool class; a sandboxed `cargo` does not get the same syscalls as a sandboxed `ffmpeg`.

### Shared per-spawn audit emission

Every spawn (success or failure) emits `EVT-FOUNDRY-SANDBOX-SPAWN` to the audit chain with `{capability, tool_id, sandbox_class, resource_caps, mounts, egress_allowlist, digest, signature, spawn_latency_ms, exit_code, exit_reason}`. The emission is on the spawn path; if the audit chain cannot accept the emission, the spawn is refused (we never run a tool we cannot record).

### Sandbox-escape detection

A separate `oya-intelligence-sandbox-escape-detector` consumes:

- Wasm: any host-call attempt outside the WASI Preview 2 import surface (Wasmtime traps; we record the trap context).
- Firecracker: any seccomp-bpf rejection (logged from the jailer) and any egress attempt that bypasses the per-tool allowlist (the SOCKS proxy refuses + logs).

Detection emits `EVT-FOUNDRY-SANDBOX-ESCAPE-ATTEMPT` and triggers an automatic suspension of the offending tool digest until a human review clears it.

### CI lanes

- `foundry-sandbox-spawn-latency` — asserts p99 cold and warm budgets per substrate.
- `foundry-sandbox-egress-deny` — synthetic test: tool tries an egress not in its allowlist; spawn must terminate with `EgressViolation`.
- `foundry-sandbox-fs-readonly` — synthetic test: tool tries to write to a RO mount; must fail with `FsViolation`.
- `foundry-sandbox-escape-attempt` — adversarial test: tool attempts known WASI/Firecracker escape patterns; detector must fire.
- `foundry-sandbox-audit-emission` — asserts every spawn emits the audit event, including refused spawns.

---

## Consequences

### Positive
- Two-tier model gives us the right cost-isolation tradeoff per tool — fast path for the common case, strong path for the heavy case.
- Per-tool resource caps eliminate noisy-neighbor failures across capability invocations.
- Per-tool egress allowlist forces capability authors to declare the network surface, which becomes catalog-reviewable.
- Per-spawn audit emission lets us replay any tool execution forensically.
- Escape-attempt detection plus automatic suspension shrinks the blast radius of a sandbox-breaking exploit.

### Negative
- Operating two substrates is more work than one; we must maintain expertise in both Wasmtime and Firecracker.
- Firecracker warm pool consumes baseline memory even when idle.
- Cold-start budgets constrain the kinds of tools the runtime can support inline; tools that exceed the budget must be queued out-of-band.
- The egress allowlist is operationally heavy — every new tool needs an allowlist review.

### Operational
- Runbook: `runbooks/foundry-sandbox-escape.md` — triage on `EVT-FOUNDRY-SANDBOX-ESCAPE-ATTEMPT`; how to suspend a digest; how to roll forward.
- Runbook: `runbooks/foundry-sandbox-warm-pool.md` — capacity planning for the Firecracker warm pool; sizing, drain, replace.
- On-call: spawn-latency alerts; escape-attempt alerts (always escalate); audit-emission failure alerts (refused spawns are not a failure but a high spawn-refusal rate is).
- Per-release: re-run the egress-deny and escape-attempt lanes against every changed sandbox crate.

---

## Alternatives considered

1. **gVisor instead of Firecracker.** Pros: smaller spawn cost; user-space kernel. Cons: weaker isolation against kernel-bug exploits; less production-track-record at our intended scale. Rejected — Firecracker's microVM boundary matches our threat model.
2. **Containers (runc / containerd) for both tiers.** Pros: tooling ubiquity. Cons: shared kernel; weaker tenant isolation; not suitable for our autonomy-ceiling defense-in-depth posture. Rejected.
3. **Wasmtime-only (no Firecracker).** Pros: one substrate. Cons: a large class of tools cannot run in WASI (full toolchains, vendor CLIs); we would either reject them or run them outside the sandbox — both unacceptable. Rejected.
4. **Firecracker-only (no Wasmtime).** Pros: one substrate. Cons: the cold-start budget and per-spawn cost wreck the SLO for short-lived deterministic tools that constitute the majority of invocations. Rejected.
5. **No sandbox; rely on autonomy ceiling.** Pros: zero spawn cost. Cons: autonomy ceiling is a policy gate, not an isolation boundary; a policy-permitted tool can still exfiltrate, crash, or starve the host. Rejected.

---

## Open questions

1. How do we share Wasm modules across spawns without losing the per-tenant isolation guarantee? Module cache by digest is safe; module cache by name is not. *Owner: `foundry`; target the next pack.*
2. What is the policy when a tool's resource caps are too tight for the actual workload — auto-promote to a higher cap (and re-emit), or always fail? *Owner: `foundry` + `ops-sre-reliability`.*
3. Should the per-tool egress allowlist support time-windowed grants (e.g. a one-shot egress for a one-shot data pull)? *Owner: `foundry` + `ops-security`.*
4. How does the warm pool reconcile with per-region residency — do strict-residency tenants get region-pinned warm VMs, or do we burn warm-pool capacity per region? *Owner: `foundry` + `cloud`.*

---

## References

- Internal: ADR-0021 (capability registry — capability declares its `sandbox_class`), ADR-0022 (autonomy ceiling — sandbox is the process-level complement), ADR-0025 (audit chain for emission).
- External: Wasmtime (Bytecode Alliance), WASI Preview 2 component model, Firecracker microVM (Amazon), seccomp-bpf, Cosign (Sigstore) for digest signatures.
