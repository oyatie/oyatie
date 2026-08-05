# Hyperscaler delivery lanes (separated)

**Rule:** one concern per lane. No coupling CAS activation to runner scale, or pre-merge matrix edits to #1561 review.

**Orchestration:** single `mm-delivery` pipeline + `mm-drive`. These are **workstreams**, not separate products/workflows.

| ID | Layer | Concern | Agent-drive? | Couples to | Hard stops |
|----|-------|---------|--------------|------------|------------|
| **R1-runner-capacity** | L2 ops | Unlock CI runners / ARC scale / `OYA_CI_MAX_PARALLEL` | Human+ops (agent may draft runbook only) | None | No gate deletion; no CAS identity |
| **R2-premerge-shape** | L2 workflow | Path-filter postgres; single materialize producer; CONE-first | Yes (isolated worktree PR) | Not R3 writers; not CAS warm | Keep `oya-ci-required` singleton |
| **R3-postmerge-trunk** | L4 | Promoted-SHA re-verify; cache-writer; completion packets | Yes (docs/automation only until trusted push) | After merges exist | No baseline write from PR head |
| **R4-local-assist** | L0–L1 | Pre-push recipe: OWNERS+reachability+package YAML | Yes (kit tips/`mm-drive` briefs) | Reduces R2 FULL rate | Non-admission; `--no-verify` still possible |
| **R5-cas-g039-1558** | L0+L2+L5 prep | Draft #1558 G003; cache-only proof later | Yes (cas worktree) | #1541 blocks **warm** CAS | No RE; no merge until green+review; no credential in git |
| **R6-re-sandbox** | L6 | RE after CAS earned (#1549) | No until R5 terminal | R5 + #1549 | Sequenced_blocked |
| **R7-k8s-w0a-1561** | L2+L3 | G001 review/merge | Human review; agent prep only | Not R5 | No self-approve; no W0-B until G001 receipt |
| **R8-mm-drive-kit** | L7 | Stop hook / quant / kit | Yes (kit worktree) | None | No merge authority; no omc/omx/gjc |
| **R9-talos-1541** | Security | Credential rotation / rebuild | Human only | Blocks warm R5 | No secrets in git/logs |

## Short-term priority (parallel)

1. **R1** — if queue_wait dominates (current evidence): scale runners / align `OYA_CI_MAX_PARALLEL`
2. **R5** — babysit #1558 CI; fix only if red
3. **R7** — wait independent APPROVE on #1561
4. **R4** — land local assist tip so next PR stays CONE
5. **R8** — already MVP’d (`mm-drive`); optional polish

## Medium-term (separate PRs)

- **R2** pre-merge shape (workflow-only PR)
- **R3** post-merge packet automation

## Long-term (program)

- **R5** → cache-only CAS/AC proof after #1541  
- **R6** RE only after R5 terminal  

## Anti-patterns

- One PR that “scales runners + rewires CI + activates CAS”
- Treating PR green as G039 promoted proof (that is **R3/R5 post-merge**)
- Agent implementing **R9** secrets
