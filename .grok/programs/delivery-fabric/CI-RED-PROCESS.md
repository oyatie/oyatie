# CI red process failure analysis (2026-08-06 fleet #1579–#1583)

**Not merge authority.**

## What went wrong

Agents opened multiple reorg/process PRs. GitHub showed **all red**. That looked like a product failure. Triage (`mm-ci-triage --all-open`) showed:

| Class | Meaning |
|-------|---------|
| **GHA_INFRA_FLAKE** | Job died in **Set up job** / `Failed to resolve action download info` / `Service Unavailable` |
| **REAL_PRODUCT** | Compile/test/drift after actions downloaded |

For fleet **#1579–#1583**, sampled fails were **100% GHA_INFRA_FLAKE** (action CDN / runner acquire). Concurrently, `oya-ci-required` stayed **`queued`** for 45–60+ minutes with **no self-hosted runners** registered and no job start — admission never greened.

### Process defects (not “fix the reorg”)

1. **No pre-PR local gate for reorg leaves** — `preflight-ci-infra` only fires on CI-infra paths; dual-home / tools / move-plan PRs could open without `mm-preflight-pr`.
2. **No post-open triage** — babysit treated red ≡ product, risk tip thrash.
3. **No distinction** between missing `oya-ci-required` (queue/ARC) and failed binding gates.
4. **Fan-out of 5 PRs** during Actions degradation multiplied flake surface without adding product signal.

## Required gates (harness)

| When | Tool | Stops |
|------|------|--------|
| **Before** `gh pr create` / push of reorg or process work | `.grok/bin/mm-preflight-pr` | Local JSON/py/singleton/cargo consumer checks; dual-home without registry/evidence |
| **Before** push of CI-infra paths | `.grok/bin/preflight-ci-infra` | Existing R4 receipt |
| **After** PR red | `.grok/bin/mm-ci-triage --pr N` | Classifies flake vs real; **forbids** product thrash on flake |
| **Merge** | `mm-drive merge-check` + `oya-ci-required` SUCCESS | Unchanged admission |

### Implement (W2) hard rule

```
mm-preflight-pr --write-receipt   # must exit 0
git push
gh pr create ...
# on red:
mm-ci-triage --pr N
# if ALL GHA_INFRA_FLAKE → re-run jobs / wait ARC; DO NOT dual-critic tip thrash
# if REAL_PRODUCT → fix in worktree, re-run preflight, one signed push
```

### Babysit (W3) hard rule

1. `mm-ci-triage --pr N` first  
2. Flake-only → `gh run rerun` when terminal; never invent product fixes  
3. Real → hand back to implement lane with finding list  
4. `oya-ci-required` missing/queued → capacity/ARC, not product  

## What local preflight cannot catch

- GitHub Actions CDN 503 at “Getting action download info”  
- ARC runner pool empty / jobs stuck `queued`  
- Hosted runner acquire delays  

Those require triage + ops, not reorg rewrites.

## Evidence commands

```bash
python3 .grok/bin/mm-ci-triage --all-open --json
python3 .grok/bin/mm-preflight-pr --base origin/dev
```
