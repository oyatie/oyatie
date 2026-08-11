# `.grok/process-kit` — Rust-first swarm process kit

**Owner tip:** `integ/ci` (`roots.grok`).  
**BAN:** re-birth shell under `tools/swarm/**` (automation-language / #1644 abort).

## Incremental slice (this land)

| Shipped | Not yet (Done-when blockers) |
| --- | --- |
| `detect_env_escapes` / `require_orchestrator` lib + unit tests | `buck2` packaging (`rust_library` / `rust_binary`) + root workspace membership via **integ/build** lock absorb |
| `oya-process-kit-check-daemon` stub binary | Real `buck2 build //...[check]` hot-set fan-out (`daemon_hotset`) |
| Harness JSON cites this crate | `git-shim` / `toolguard` / `claim-push` Rust successors; wire into every runtime (Cursor/lane-shell) |

## Verify (buck2 once wired)

```bash
buck2 test //...   # after BUCK + integ/build membership land
SWARM_ORCHESTRATOR=1 buck2 run <process-kit-check-daemon>
```

Do **not** add this package to root `Cargo.toml` from `integ/ci` — `#planes.root_manifests` sole owner is `integ/build`.
