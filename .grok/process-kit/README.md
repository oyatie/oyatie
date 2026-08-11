# `.grok/process-kit` — Rust-first swarm process kit

**Owner tip:** `integ/ci` (`roots.grok`).  
**BAN:** re-birth shell under `tools/swarm/**` (automation-language / #1644 abort).

Buck targets compile sources now. No `Cargo.toml` on this tip: a `[package]` without
workspace membership trips `crate_dir_not_covered` / freshness orphans, and
membership + `Cargo.lock` is sole-owned by `integ/build` (#1662).

## Verify

```bash
buck2 test //.grok/process-kit:oya-process-kit-unittest
SWARM_ORCHESTRATOR=1 buck2 run //.grok/process-kit:oya-process-kit-check-daemon
```
