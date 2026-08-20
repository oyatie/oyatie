# Master parallel drive — full backlog fan-out

**Updated:** 2026-08-05  
**User directive:** unlock more runners, nativelink CAS, buck2 RE, CI optimization, product lanes, reorg/rebrand/rewrite, k8s Go→Rust port, all backlog items.  
**Policy:** one concern per PR; agent dual-critic + `oya-ci-required` for merge; **no false-complete ultragoal**; CAS before RE; no warm CAS without #1541; no agent secrets.

## Sequencing (hard)

```
G039 #1558 MERGED (packet after trunk green)
  → Lane 3A NativeLink rehome (no warm flip)
  → Lane 3B Buck2 cache package move
  → Lane 3C CI cache policy behavior
  → #1541 human creds → G041 cache-only proof
  → G043 RE only if measured reopen criteria

G001 #1561 → packet → W0-B engine (G002) → W0-C…H → W1+ unapproved

R1 runners: capacity contract + storage math before maxRunners>1 apply
R2 path-filter #1562 → merge when green
R3 packets after each promote
```

## Parallel lanes NOW (agent-executable)

| ID | Lane | Worktree/PR | Start | Hard stop |
|----|------|-------------|-------|-----------|
| P0 | Merge babysit #1561/#1562 + trunk G039 | mm-drive | continuous | none |
| P1 | **R1 unlock runners** — capacity contract + maxRunners path + Talos volume math | new from origin/dev | **NOW** | no cluster apply from agent; no gate delete |
| P2 | **CAS 3A** NativeLink → `storage/adapters/nativelink/` | new from origin/dev | **NOW** | no warm_reads; no RE; no credential |
| P3 | **CI opt** materialize-once / further path filters (not coupled to #1562) | new | after or parallel R2 | keep oya-ci-required singleton |
| P4 | **K8s W0-B** mechanical port Slice 1 scaffold | after G001 packet | sequenced | no bulk corpus; no W1 |
| P5 | **Kit** D3 checkpoint-check / D4 briefs | kit worktree | **NOW** | not merge authority |
| P6 | **RE sandbox docs/plan only** #1549 prep | docs only | **NOW** | no remote_enabled |
| P7 | **#1541 status** security awareness | evidence only | **NOW** | no secrets implementation |
| P8 | **Reorg/rebrand/rewrite/delete** mined debt → isolated PRs | per card | **NOW** | ADR re-query; one concern per PR; multi-cap OK as epic only |
| P9 | **Product portfolio** G023–G037 pick ready slices | per card | after mine | not CAS coupling |

## Reorg doctrine (summary)

Full text: [`REORG-DOCTRINE.md`](REORG-DOCTRINE.md)

- **Authority:** Accepted ADRs (amended) > consensus plans > move-plans > mined backlog  
- **Classes:** move · refactor · rewrite · delete · rebrand · mixed(stages) — not “move only”  
- **Span:** may cross **multiple capabilities**; still **one concern per PR** + temporal ownership  
- **Move-plan:** only for path-bijection moves (0614); never sole law for refactor/rewrite/delete  
- **Defects:** plan lag vs ADRs, and implementing stale inventory without re-query, are **separate** failures  

## Not doing in one PR

- scale runners + rewire CI + activate CAS  
- RE + warm CAS  
- W0-B product + W0-A governance  
- hand-edit `*.generated.json`  
- multi-capability mega-PR that mixes move + rewrite + delete without staged concerns
