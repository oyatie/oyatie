# Optional multi-provider channels (omx / omc / gjc / grok)

> Founder-authorized idea one-pager (2026-08-10). Records **advisory** affordances only —
> never default implement/scout drivers, never merge authority, never ranked by external
> score-cost corpora. Cite Amendment C / [`specs/agentic-operating-patterns.json`](../../specs/agentic-operating-patterns.json)
> for routing provenance. Freeze note: `docs/ideas/` is under `#reorg_debt_freeze` — this page
> is debrand-only; forever placement is a reorg-move-out bead (do not deepen the freeze tree).

## How Might We

**HMW** keep `omx` / `omc` / `gjc` / `grok` available as **optional second-opinion / multi-provider**
channels without making them default drivers, without mapping external score-cost bench
rankings onto them, and with **accurate flag semantics**.

**Audience:** orchestrator / founder when a second opinion, alternate provider path, or
multi-provider spend view is useful. **Not** worker-lane default execution.

**Success:** truthful capability card + corrected routing copy. Using these tools remains
optional (“if useful”), never required. Task `model:` slugs remain the default dispatch path
(`ci/facade/harness/model-routing.v1.json` — Amendment C /
`specs/agentic-operating-patterns.json` for catalog cites).

## Flag semantics (founder lock)

| Need | Command | Semantics |
|------|---------|-----------|
| Codex tmux, normal perms | `omx` / `omx --tmux` | tmux interactivity |
| Claude tmux, normal perms | `omc launch` | tmux interactivity (`omc interop` for OMC+OMX split) |
| Multi-model tmux | `gjc --tmux` | tmux only (no madmax-equivalent in help) |
| Grok interactive | run `grok` **inside** tmux | no built-in `--tmux`; host/founder tmux is valid |
| Codex bypass + tmux | `omx --madmax` | **permission bypass + tmux** (compound) |
| Claude bypass + tmux | `omc --madmax` | **permission bypass + tmux** (compound) |
| Multi-provider spend | `gjc` + `/usage` and/or `gjc stats` | dashboard `http://localhost:3847` until Ctrl+C |
| Side panel | editor side panel | advisory; human-driven |

**`--madmax` (omx/omc)** = **permission bypass + tmux** (compound). Dangerous. Optional founder
tool — not worker-lane normal. **`gjc --tmux`** is tmux-only. Grok `--always-approve` is
auto-approve tools, **not** the same compound as madmax.

External score-cost benches **do not transfer** to these channels. Pick omx/omc/gjc/grok by
provider need and independence — not by bench rank. Routing defaults stay Amendment C /
Task `model:` slugs.

## Capability matrix (smoke 2026-08-10)

| Check | Result | Evidence |
|-------|--------|----------|
| `omx doctor` | **works** (partial) | exit 0; 1 fail: process identity unavailable; ownership warnings on stale `.omx/tmp` |
| `omc info` | **works** | exit 0; prints system/agent info |
| `gjc stats` | **works** (interactive) | syncs sessions then serves dashboard until Ctrl+C — not a one-shot print |
| Live `omx` tmux | **works** | e.g. `omx-oyatie-agent-fanout-07-…` attached |
| Live `omc` tmux | **works** | e.g. `omc-oyatie-agent-fanout-07-…` attached |
| Live `gjc` tmux | **works** | e.g. `gajae_code_agent-…` attached |
| Live `grok` tmux | **works** | session `88` running `grok-1.0.0-macos-aarch64` cwd=oyatie (founder-opened) |
| `omx --madmax` / `omc --madmax` from agent sandbox | **doesn't** (env) | sandbox `EPERM` on `~/.omx-runs`; do not leave long-lived madmax writing main checkout from agent probes |
| Side panel drive from agent | **unverified** | human UI; no full “open side panel” API |
| In-session `gjc /usage` | **unverified** | CLI confirms `gjc stats`; `/usage` needs live interactive confirm |
| External score-cost bench → these channels | **doesn't transfer** | law (Amendment C) |

Encoded on `#1644` harness **1.4.0** @ `69ed6b0d0` (+ routing rule text). Programme SSOT:
live plan under the Swarm Delivery Law programme (session plan path — not a tracked corpus).

## Rules

- Channel output is **advisory** — never merge authority, never a substitute for green
  `presubmit` / merge-admission.
- Prefer Task `model:` slugs for implement/scout/plan/critic inside agent dispatches.
- Prefer side panel when usable; use tmux when attach/interact is needed.
- Worker lanes must not treat bare `gjc` / `omc` / `omx` / `hermes` as merge-authority stand-ins.
- Orchestrator may open advisory side-panel or tmux sessions; madmax only when founder
  intentionally wants bypass+tmux.

## Not doing

- Not making omx/omc/gjc/grok default for implement/scout/babysit
- Not ranking panel/CLI with external score-cost benches
- Not teaching workers madmax as routine
- Not leaving madmax sessions writing the main checkout during probe
- Not replacing Swarm Delivery Law with third-party orchestration brands
- Not auto-wiring channels into `deliver.js` / CI / lane-shell
- Not using `*-fast` model slugs
