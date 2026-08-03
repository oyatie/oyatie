# Security Review — PR #685 (G011 main-checkout guard) — r20 fresh-context VERIFICATION

**Head pinned:** f3f7b79535d554b1a235a3809f3a3748753f6fa6 (branch agent/g011-checkout-guard)
**Worktree:** /Users/jasonlee/oyatie-worktrees/g011-checkout-guard
**Subject file:** tools/oya-checkout-guard-app/src/lib.rs
**Fix under review:** commit c6a8cb7c2 — left-fold binding resolution replacing the r18 8-pass fixpoint in `collect_same_line_bindings` (~L1751); `resolve_binding_refs` (~L1823)
**Method:** live binary (buck2-built) attacked from foreign cwd /tmp; exit 2 = DENY, exit 0 = ALLOW
**Risk Level: LOW** (CRITICAL closed; sole residual is runtime-unknowable dynamic content, fail-closed)

## Summary
- Critical Issues: 0
- High Issues: 0
- Medium Issues: 0
- Low Issues: 0
- r19 CRITICAL: CLOSED (verified live)

## Build & Test Evidence
- `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out /tmp/r20-guard` → BUILD SUCCEEDED
- `buck2 test //tools/oya-checkout-guard-app:` → **32 passed; 0 failed; 0 ignored** (buck2 target Pass 1, Fail 0)

## Live-binary evidence table (env OYA_CANONICAL_CHECKOUT=/Users/jasonlee/Developer/oyatie, cwd=/tmp)
| # | Class | Payload | Want | Got |
|---|-------|---------|------|-----|
| 1 | r19 CRITICAL | `A=reset; B=$A; A=log; git -C <canon> $B --hard` | DENY | DENY exit2 |
| 2 | r19 path-side | `A=<canon>; B=$A; A=/tmp/x; git -C $B reset --hard` | DENY | DENY exit2 |
| 3 | r19 += chain | `A=reset; A+=" --hard"; B=$A; A=log; git -C <canon> $B` | DENY | DENY exit2 |
| 4 | forward-ref | `B=$A; A=reset; git -C <canon> $B --hard` | DENY | DENY exit2 |
| 5 | self-ref | `A=$A; A=reset; git -C <canon> $A --hard` | DENY | DENY exit2 |
| 6 | append-before-def | `A+=$B; B=reset; git -C <canon> $A --hard` | DENY | DENY exit2 |
| 7 | multi-level chain | `A=re; B=${A}set; C=$B; git -C <canon> $C --hard` | DENY | DENY exit2 |
| 8 | braced ref | `A=reset; B=${A}; git -C <canon> $B --hard` | DENY | DENY exit2 |
| 9 | ${A:-default} | `A=reset; B=${A:-log}; git -C <canon> $B --hard` | DENY | DENY exit2 |
| 10 | 4-level chain | `A=re; B=$A; C=${B}set; D=$C; git -C <canon> $D --hard` | DENY | DENY exit2 |
| 11 | shadow over && | `A=reset && B=$A && A=log && git -C <canon> $B --hard` | DENY | DENY exit2 |
| 12 | shadow over newline | (newline-separated A=reset / B=$A / A=log / git) | DENY | DENY exit2 |
| 13 | FP capture→read | `A=log; B=$A; A=reset; git -C <canon> $B` | ALLOW | ALLOW exit0 |
| 14 | fwd-empty benign | `B=$UNDEFINED; git -C <canon> status` | ALLOW | ALLOW exit0 |
| 15 | fwd-read verb | `V=$UNSET; git -C <canon> log --oneline` | ALLOW | ALLOW exit0 |
| 16 | fwd no-verb | `B=$A; A=reset; git -C <canon> $B` | ALLOW | ALLOW exit0 |
| 17 | back-resolved | `A=reset; B=$A; git -C <canon> $B --hard` | DENY | DENY exit2 |
| 18 | two-git wipe | `A=reset; B=$A; git -C <canon> status && git -C <canon> $B --hard` | DENY | DENY exit2 |
| 19 | brace+def bound | `A=reset; B=${A:-log}; C=${B}; git -C <canon> $C --hard` | DENY | DENY exit2 |
| 20 | dyn $(cat) | `B=$(cat /tmp/verb); git -C <canon> $B --hard` | DENY | DENY exit2 |
| 21 | dyn read/stdin | `read B; git -C <canon> $B --hard` | DENY | DENY exit2 |

### r1-r18 regression DENY corpus (all DENY, live)
direct reset --hard; checkout -- .; clean -fdx; set -- positional $@; function inline g(){git "$@";}; IFS-split; $(echo reset); read -ra array ${A[@]}; P=reset;P+=" --hard".

### Legitimate worktree ALLOW corpus (all ALLOW, no false positives, live)
worktree status/log/reset --hard/commit/push/`cd <wt> && git reset --hard`/`P=log;P+=" --oneline"`/no-canonical-configured.

## Two apparent mismatches investigated — both were ERRORS IN THE TEST HARNESS, not guard defects
1. `git -C <canon> push --force` → ALLOW. CORRECT: `push` is intentionally NOT in the deny set (`is_blocked_operation`, L2916–2935). FRIC-022 scope is canonical *checkout* (working-tree / local-ref) protection; `push` is a remote op that does not destroy the local checkout. Excluding it is by-design, not an oversight.
2. `git reset --hard` from cwd=/tmp → ALLOW. CORRECT: effective target is /tmp (no `-C`, foreign cwd), not the canonical checkout. The guard denies only when the effective target IS the canonical checkout.

## Left-fold semantic assessment
The fix replaces the last-wins fixpoint with a left-fold: each RHS resolves via `resolve_binding_refs(value, &acc)` against only bindings accumulated to its LEFT (L1774–1786), matching bash execution-point semantics. Residual `$`/backtick after resolution → value dropped by the resolved-only filter (L1790–1794) → `$name` fails closed at its use site. `collect_same_line_bindings` flattens across separators (no `;`/`&&` scoping); for a guard this is fail-closed-safe on the DENY side, and the one false-positive risk it could create (capture-then-reassign-to-READ, case 13) was verified to ALLOW correctly because the left-fold binds the READ value at capture point, immune to the later wipe reassignment.

## Universality
Zero hardcoded `/Users` / `jasonlee` / `oyatie-worktrees` / `Developer/oyatie` outside `#[cfg(test)]` (tests module starts L3129). Canonical sourced from `OYA_CANONICAL_CHECKOUT` env or `default_canonical_checkout` derived from git common dir (main.rs L47/L61). No machine/repo assumptions in production code.

## Residual (acceptable, fail-closed)
Genuinely runtime-unknowable dynamic content — `$(cat …)`, `$(curl …)`, `read` from stdin/pipe — cannot be statically resolved and is denied fail-closed at the use site (cases 20–21). This is the SOLE residual and is the intended, safe behavior.

## Security Checklist
- [x] r19 CRITICAL reproductions (3) all DENY live
- [x] No new reproducible static-text bypass (forward/self/append/braced/default/multi-level/shadow-over-separators all DENY)
- [x] Zero false positives on legitimate worktree commands (capture→read ALLOW; 8 legit ALLOW)
- [x] Universality clean (no hardcoded paths outside cfg(test))
- [x] Dependency posture: pure-Rust single-binary guard, no new third-party deps introduced by the fix; nothing to audit beyond workspace lockfile
- [x] Sole residual is runtime-unknowable dynamic content, fail-closed

VERDICT: APPROVE
