# Intent → hardened prompt → decompose → dispatch → lifecycle (mechanical)

**Does this make sense?** Yes. Raw user language is a poor control plane.  
**Process fix:** every prompt submit enters a **mechanical** path that (1) hardens the prompt, (2) decomposes work, (3) dispatches under path-disjoint rules, (4) runs the rest of `pipeline.json`, (5) self-improves via LEARN/process_edits.

Not merge authority. `oya-ci-required` + dual-critic independence rules still apply.

## Lifecycle (SSOT: `harness/pipeline.json`)

```text
PROMPT_SUBMIT (user / CLI / goal)
        │
        ▼
┌───────────────────┐
│ CAPTURE_INTENT    │  → intent.v1  (raw text + refs only; no inventing scope)
└─────────┬─────────┘
          ▼
┌───────────────────┐
│ PROMPT_HARDEN     │  → hardened_prompt.v1  (LLM-assisted, schema-fail-closed)
│  instruction-best │     objective, DoD, non-goals, constraints, verify,
│  practices        │     ambiguities_resolved/open, authority block
└─────────┬─────────┘
          ▼
┌───────────────────┐
│ DECOMPOSE         │  → work_graph.v1  (slices, paths, depends_on, parallel_ok)
└─────────┬─────────┘
          ▼
┌───────────────────┐
│ DISPATCH          │  path-overlap fail closed → mm-role / worktrees / shards
└─────────┬─────────┘
          ▼
   PREFLIGHT → CONTRACT → PLAN → dual CRITIC (plan) → ADMIT_PLAN
          │
          ▼
   mechanical implement loop (see IMPLEMENT-LIFECYCLE.md):
   RED_TEST → IMPLEMENT → GREEN_TEST → INTEGRATION_TEST → FALSE_GREEN_SCAN
   → REVIEW_DIFF → SIMPLIFY → HARDEN → VERIFY → ADMIT_SLICE
          │
          ▼
   PR_PACKET → (CI babysit / mm-drive) → SCORE_GRADE → LEARN → POSTMORTEM
          │
          ▼
   process_edits → harness/roles/pipeline (self-improve the loop)
```

## Stage contracts

| Stage | Input | Output schema | Mechanical rule |
|-------|--------|---------------|-----------------|
| CAPTURE_INTENT | raw prompt | `intent.v1` | No expansion of scope; record only |
| PROMPT_HARDEN | intent.v1 | `hardened_prompt.v1` | Must fill success_criteria + verification; open ambiguities block high/critical execute |
| DECOMPOSE | hardened_prompt.v1 | `work_graph.v1` | Every slice has paths + DoD; overlap list empty for parallel |
| DISPATCH | work_graph.v1 | dispatch ledger | One write root per slice; `mm-role` for roles; kit required |
| RED→ADMIT | red/green/false_green/… | implement reports | **TDD mechanical** — `IMPLEMENT-LIFECYCLE.md` |
| LEARN | run journal | tips / process_edits | **Always** on wave end; promote only via human-gated pack edit |

## Prompt harden (instruction-following bar)

The hardener is **not** free prose. It must produce:

1. **One objective** (done-looks-like)  
2. **Success criteria** (testable)  
3. **Non-goals** (scope fence)  
4. **Constraints** including: base `origin/dev`, PR → `dev`, not merge authority, no hand-edit `*.generated.json`, kit presence when multi-model claimed  
5. **Verification commands** (real)  
6. **Assumptions** vs **open questions** (open + high risk ⇒ human gate)  
7. **Blast radius** / risk  
8. Rendered `instruction_pack` for downstream roles  

Role: `PROMPT_HARDENER` in `multi-model-roles.json` (read-only; schema output).

## Decomposition + dispatch (mechanical)

- Slices are the unit of **parallel** work only when `paths` are disjoint (`parallelism.v1.json` / path-overlap = 0).  
- Depends_on forms a DAG; DISPATCH respects topo order.  
- Dispatch invokes `.grok/bin/mm-role <ROLE>` — not same-family subagent cosplay for CRITIC when `require_cross_model_critics`.  
- Worktree: bootstrap kit if missing (`F-WORKTREE-MISSING-KIT`).

## Self-improvement of the loop

| Trigger | Action |
|---------|--------|
| Stage fail / thrash / same-family launder / push thrash | Append `process_edits.md`; fix harness before retry |
| Wave end | SCORE_GRADE + LEARN always |
| KPI class repeats ×2 | Human-gated promote to packs/roles (`learning-loop.v1.json`) |
| Static-only A with runtime D | **Forbidden claim** of process-healthy (`F-STATIC-EVAL-FALSE-GREEN`) |

## Entry points (agents must use these)

```bash
# Preferred: pipeline owns the full path after objective
.grok/bin/mm-pipeline start --objective "…" --risk medium

# Goals SSOT (durable) then activate into pipeline
.grok/bin/mm-goals create --brief-file path/to/brief.md
.grok/bin/mm-drive tick --json

# Explicit harden only (debug / interview output)
.grok/bin/mm-role PROMPT_HARDENER -- --intent-file mm-runs/<id>/intent.json
```

**Forbidden shortcut:** orchestrator free-form implements from raw chat without CAPTURE → HARDEN → DECOMPOSE → DISPATCH. That is the class that produced #1574 process failures.

## Admit rules (summary)

| Gate | Fail closed when |
|------|------------------|
| Hardened prompt | missing success_criteria or verification |
| Hardened prompt | ambiguities_open non-empty AND risk ∈ {high, critical} without human_gate |
| Work graph | path_overlap_violations non-empty for parallel batch |
| Dual critic | independence ≠ cross_model when required |
| Merge | no oya-ci-required SUCCESS on tip |

## Relation to interview skills

Human interview (`interview-me`) may run **before** CAPTURE when intent is underspecified.  
After interview, **mechanical path starts** — interview notes become `raw_prompt` / constraints, not a parallel control plane.
