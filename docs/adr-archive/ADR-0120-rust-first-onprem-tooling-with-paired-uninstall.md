---
id: ADR-0120
status: Superseded
doc_status: published
date: 2026-05-16
supersedes: []
superseded_by: [ADR-0375]
related: [ADR-0083, ADR-0121]
---

# ADR-0120: Rust-first on-prem tooling; every install paired with uninstall

> **Status:** Superseded by [ADR-0375](ADR-0375-talos-capi-argocd-fleet-substrate.md)
> **Owner:** `axis-foundry` + `ops-sre`
> **Date:** 2026-05-16
> **Decision driver:** user directive 2026-05-16 ("we only keep limited number of shell scripts and keep everything else rust"; "every install, cleanup, action, script, bootstrap should come with cleanup and uninstall").

---

## Context

The 2026-05-16 on-prem bring-up dropped ~15 bash scripts under `infra/onprem/{hardening,openbao,podman,containerd,kubeadm,istio,cloudflared,security,cleanup,sanoid,reboots,foundry,build,tarball}/install.sh`. Each script is idempotent and self-contained, but:

1. **Shell is a poor long-term substrate** for orchestration logic — no type safety, easy to break, hard to test under CI, and contradicts ADR-0083's tier-1 Rust posture for the rest of the codebase.
2. **Installs have no inverse.** Every bring-up step needs a paired teardown so we can clean-room re-test on the same host, and so contributors aren't trapped in partially-installed states.

## Decision

Two coupled rules:

### Rule 1 — Limited shell surface; Rust elsewhere.

The on-prem tooling collapses to **one binary** plus a small bootstrap layer:

```
crates/oya-onprem-cli            ← Rust binary `oya-onprem`
├── install <component>          ← installs one component (idempotent)
├── uninstall <component>        ← reverses one install (idempotent)
├── status                       ← machine-readable diagnostics
├── scan                         ← security scan (delegates to gitleaks/trivy/...)
├── cleanup                      ← apt autoremove + agent-state reap
└── doctor                       ← runs status + suggests fixes
```

Authorized shell scripts (the **bootstrap layer**, capped at 3 files total):

1. `infra/onprem/bootstrap.sh` — minimal: ensure Rust toolchain present, `cargo build --release -p oya-onprem-cli`, install binary at `/usr/local/bin/oya-onprem`, then `oya-onprem install all`. Single file.
2. `infra/onprem/uninstall-all.sh` — the inverse: `oya-onprem uninstall all`, then remove the binary, then optionally remove Rust toolchain. Single file.
3. `infra/onprem/diagnose.sh` — thin shim that just calls `oya-onprem doctor` for users who don't want to memorize the new CLI yet (will be retired once muscle memory shifts).

Every other current `*/install.sh` script becomes either:
- a Rust **component** under `crates/oya-onprem-cli/src/components/<name>.rs` implementing the `Component` trait (with `install()`, `uninstall()`, `status()` methods), OR
- a config-only directory (TOML/HCL) consumed by the Rust binary.

Removal trigger: when `oya-onprem install all` produces a host bit-for-bit identical to the current `setup.sh` output (modulo timestamps), the per-component shell scripts move to `infra/onprem/legacy-shell/` for one release cycle, then deleted.

### Rule 2 — Every install/action has a paired uninstall.

For every component, the `Component` trait MUST provide:

```rust
pub trait Component {
    fn install(&self, env: &HostEnv) -> Result<InstallReport>;
    fn uninstall(&self, env: &HostEnv) -> Result<UninstallReport>;
    fn status(&self, env: &HostEnv) -> Result<StatusReport>;
}
```

Uninstall semantics:

- **Idempotent.** Running `uninstall` twice produces the same final state.
- **Reversible.** After `uninstall`, the host MUST be in the state it would have been if `install` had never run, modulo any user data the component MUST preserve (audit-chain, vault-stored secrets, ZFS snapshots — those are flagged `preserve_user_data = true` and require an explicit `--purge` flag to also wipe).
- **Cascades respected.** Uninstalling `kubeadm` first removes `istio`; uninstalling `containerd` first removes `kubeadm`; the binary computes the reverse-dependency order automatically from a topological sort over component dependencies.
- **Auditable.** Every uninstall emits an audit-chain event `EVT-ONPREM-UNINSTALL-<component>` with the timestamp and any data preserved/purged.

The interim `*/uninstall.sh` scripts added in this commit are placeholders. They implement Rule 2 immediately in shell; they're replaced by Rust under Rule 1 in a successor-IP ADR.

## Consequences

### Required successor-IP

- **Phase A** (this commit): every `infra/onprem/*/install.sh` gets a paired `infra/onprem/*/uninstall.sh`. Top-level `uninstall-all.sh` runs them in reverse dependency order. Each is idempotent.
- **Phase B** (next ChangeSet, M03-P01-IP-001b): scaffold `crates/oya-onprem-cli` with the `Component` trait + a no-op component to validate the wiring.
- **Phase C**: migrate components one at a time, with each PR replacing one shell pair with one Rust component. Order: simplest first (`cleanup`, `reboots`, `sanoid`), then the apt-driven ones (`podman`, `security`), then the multi-file installs (`containerd`, `kubeadm`, `istio`, `openbao`, `cloudflared`, `foundry`, `hardening`).
- **Phase D**: delete `infra/onprem/legacy-shell/` and update CLAUDE.md / docs to reference only the Rust CLI.

### Authorized exceptions

- `bootstrap.sh` exists because the Rust binary needs *some* way to enter the system when there's no Rust toolchain yet (cold-start case). It does *nothing* beyond toolchain-acquire + `cargo build` + `exec oya-onprem`.
- `uninstall-all.sh` exists because if the binary is broken/missing, you still need a way to clean-room the host. It does *nothing* beyond `oya-onprem uninstall all` with a fallback that calls every `*/uninstall.sh` directly when the binary is unavailable.
- `diagnose.sh` is a transitional alias and will be removed in Phase D.

### Rejected alternatives

- **Stay in shell entirely.** Rejected — see Rule 1 rationale; contradicts ADR-0083.
- **Ansible / chef / puppet.** Rejected — introduces a new runtime + DSL for a single-host scope. Our cluster is small enough that orchestration value doesn't exceed maintenance cost. Adopting one of these would land in ADR-0120 if the host fleet grows beyond ~5 nodes.
- **One Rust crate per component.** Rejected for now — proliferation of micro-crates increases build time and dilutes the workspace; one binary with a `Component` registry is easier to maintain.

## Test plan

- `oya-onprem install all` on a clean Debian 13 host produces the same final state as `setup.sh` (compared by `diagnose.sh` output diff).
- `oya-onprem uninstall all && oya-onprem install all` is a no-op — runs cleanly with no errors and produces the same state.
- `oya-onprem uninstall <c>` for each component leaves the host with no traces of that component (no systemd unit, no binary, no config dir, no data unless `preserve_user_data`).
- `cargo test -p oya-onprem-cli` covers the dependency-graph topological sort + idempotent install/uninstall on a fake host.
