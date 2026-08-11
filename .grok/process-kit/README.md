# `.grok/process-kit` — Rust-first swarm process kit

**Owner tip:** `integ/ci` (`roots.grok`).  
**BAN:** re-birth shell under `tools/swarm/**` or `.grok/swarm/**` (automation-language / #1644 abort).

## Incremental slice (this land)

| Shipped | Not yet (Done-when blockers) |
| --- | --- |
| `detect_env_escapes` / `require_orchestrator` | Wire into every runtime lane-shell (Cursor/Codex/…) |
| `git_shim::refuse_no_verify` | Real PATH git-shim binary installed in lane shells |
| `toolguard` worker cargo/buck2 refuse | Runtime hook install per agent surface |
| `claim_push` receipt shape stub | Full claim-mechanical envelope check |
| `BUCK` (`//.grok/process-kit:oya-process-kit(-check-daemon)`) | Root workspace membership via **integ/build** lock absorb |
| `oya-process-kit-check-daemon` stub | Real `buck2 build //...[check]` hot-set fan-out |

## Verify

```bash
buck2 test //.grok/process-kit:oya-process-kit-unittest
SWARM_ORCHESTRATOR=1 buck2 run //.grok/process-kit:oya-process-kit-check-daemon
```

Do **not** add this package to root `Cargo.toml` from `integ/ci` — `#planes.root_manifests` sole owner is `integ/build`.
