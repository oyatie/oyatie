# Fitness Lane: banned-primitives

- purpose: Verify fenced `agent-instructions` sections use the Oya VCS / `oya git` sanctioned surface and block direct VCS / forge / manual branch operations unless the policy permits a documented genuine-need rationale.
- enforces: `specs/master-plan-sequencing.json::forbidden_primitives` and `F-FORBIDDEN-PRIMITIVES-CI-GUARD`.
- activation: Active required check for `dev` once this ChangeSet promotes; workflow, branch-protection, quality lane, and `oya gate run-all` wiring are part of CS-FITNESS-001.
- kernel_crate: `oya-foundry-fitness-banned-primitives-kernel` — `scan_agent_instruction_file(...)` plus `check_documented_genuine_need(...)`.
- runner_path: `tools/oya-foundry-fitness-banned-primitives-app`
- gate_invocation: `cargo run -q -p oya-dev-cli -- gate validate banned-primitives`
- direct_runner_invocation: `cargo run -q -p oya-foundry-fitness-banned-primitives-app --`
- inputs: `AGENTS.md`, `CLAUDE.md`, `docs/**/*.md`, `.omc/**/*.md`, plus YAML / JSON / TOML files under those roots that contain `<!-- agent-instructions:start -->` fences.
- failure_modes:
  - `AGENTS.md`, `CLAUDE.md`, or `docs/AGENTS.md` lacks an `agent-instructions` fence
  - fenced agent block invokes hard-banned primitives: hook bypass, force push, home-directory mutation, external fetch, forge merge, process kill, manual branch, manual rebase, manual merge, or manual push
  - fenced agent block invokes direct VCS / forge primitives without a known genuine-need rationale
  - rationale id is not in the supplied `--known-rationale` set
- ci_workflow: `.github/workflows/oya-foundry-fitness-banned-primitives.yml`
- branch_protection_context: `oya-foundry-fitness-banned-primitives`
- quality_lane: `registry/quality/lanes.yaml::oya-foundry-fitness-banned-primitives`
- runtime_budget: 500 ms local detector target; CI budget 20 minutes including Rust toolchain and cargo cache
- severity: BLOCKER
