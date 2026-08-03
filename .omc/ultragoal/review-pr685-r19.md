# Security Review — PR #685 (G011 main-checkout guard) — r19 BYPASS-HUNT

**Reviewer of record:** r19 fresh-context BYPASS-HUNT (security-reviewer)
**Head SHA:** 11aa8036804fcc5a2c0173f5f1f97c3504c1f63c
**Branch:** agent/g011-checkout-guard
**Worktree:** /Users/jasonlee/oyatie-worktrees/g011-checkout-guard
**File under attack:** tools/oya-checkout-guard-app/src/lib.rs (binary wiring: src/main.rs)
**Canonical under test:** /Users/jasonlee/Developer/oyatie (OYA_CANONICAL_CHECKOUT override)
**Lenses:** Torvalds (hostile inspection, no credit for narration) + hyperscaler (would a prod security team accept this control?)
**Method:** every candidate tested LIVE against the built binary (/tmp/r19-guard) from a foreign cwd (/tmp) per the mandated payload-pipe protocol. Claims that did not reproduce are excluded.

---

## Build + Test Evidence

- `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out /tmp/r19-guard` => BUILD SUCCEEDED
- `buck2 test //tools/oya-checkout-guard-app:` => **Pass 1 target; 31 unit tests passed, 0 failed, 0 ignored**
- Baseline: canonical `git -C <canon> reset --hard HEAD` => exit 2 DENY; worktree-targeted same => exit 0 ALLOW; `ls -la` => exit 0 ALLOW.

---

## CRITICAL FINDING — stale-overwrite transitive-binding bypass (verb AND path side)

**Severity:** CRITICAL
**Category:** A01 Broken Access Control (control bypass) / OWASP-LLM tool-policy bypass
**Location:** `tools/oya-checkout-guard-app/src/lib.rs:1751-1803` (`collect_same_line_bindings` + `resolve_binding_refs` fixpoint, lines 1778-1795)
**Exploitability:** Local, unauthenticated (any agent emitting a Bash command); deterministic, single-line, no runtime-dynamic content.
**Blast radius:** Full canonical-checkout mutation. Every blocked verb is reachable: `reset --hard`, `switch`, `clean -fdx`, `checkout`, `rebase`, `restore`. Both the VERB side and the `-C <canonical>` PATH side are bypassable.

### Root cause
The r18 transitive-binding fix (review #685 r18 F2) resolves `$ref` inside a binding value against the **final last-wins snapshot** of all same-line bindings, not against the value **at the point the referencing assignment executed**. `collect_same_line_bindings` overwrites `A` with its last value (`=` at lib.rs:1771-1776), then `resolve_binding_refs` (lib.rs:1831) resolves `B=$A` using A's *final* value. Bash captures `$A` at B's assignment time. Divergence:

```
real bash:  A=reset; B=$A; A=log   =>  B == "reset"
guard:      collects A=log (last-wins) => resolves B=$A => B == "log"
```

So `git -C <canon> $B --hard` is modeled as `git log --hard` (not a blocked op) => ALLOW, while bash runs `git reset --hard` on the canonical checkout => destructive mutation. This is static text the parser DID model (the r18 feature) but with incorrect ordering semantics — it is NOT the accepted runtime-unknowable residual.

### Live-binary evidence (exit 0 = ALLOW = bypass; exit 2 = DENY = correct)

| Payload (cwd=/tmp, OYA_CANONICAL_CHECKOUT=/Users/jasonlee/Developer/oyatie) | bash resolves to | expected | observed | result |
|---|---|---|---|---|
| `A=reset; B=$A; A=log; git -C <canon> $B --hard` | git reset --hard | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=clean; B=$A; A=log; git -C <canon> $B -fdx` | git clean -fdx | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=switch; B=$A; A=status; git -C <canon> $B foo` | git switch foo | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=checkout; B=$A; A=log; git -C <canon> $B foo` | git checkout foo | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=rebase; B=$A; A=log; git -C <canon> $B origin/dev` | git rebase origin/dev | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=restore; B=$A; A=log; git -C <canon> $B .` | git restore . | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=reset; B=${A}; A=log; git -C <canon> ${B} --hard` | git reset --hard | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=reset; B=$A; C=$B; A=log; git -C <canon> $C --hard` (multi-hop) | git reset --hard | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=reset; B=$A; C=$B; B=log; git -C <canon> $C --hard` (shadow intermediate) | git reset --hard | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=reset; B=$A; A+=x; git -C <canon> $B --hard` (+= on source) | git reset --hard | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=/Users/jasonlee/Developer/oyatie; B=$A; A=/tmp; git -C $B reset --hard` (PATH side) | -C canonical | DENY(2) | **ALLOW(0)** | **BYPASS** |
| `A=reset; B=$A; git -C <canon> $B --hard` (no overwrite — r18 happy path) | git reset --hard | DENY(2) | DENY(2) | OK (control intact) |
| `A=reset; B=$A; A=log; git -C /srv/acme/checkout $B --hard` (foreign repo) | git reset --hard | DENY(2) | **ALLOW(0)** | **BYPASS (machine-agnostic)** |

bash ground-truth verified live: `A=reset; B=$A; A=log; echo $B` => `reset`; `A=reset; B=$A; C=$B; A=log; echo $C` => `reset`; `A=reset; B=$A; A+=x; echo $B` => `reset`; `A=/x; B=$A; A=/tmp; echo $B` => `/x`.

### Remediation (Rust, same language)
Resolve `$ref` at assignment time, not against a final snapshot. Snapshot each binding's RHS against the bindings already collected *before* it, in source order — i.e. resolve references as bindings are appended, using only entries to the left of the current assignment, instead of the post-hoc 8-pass fixpoint over the last-wins map.

```rust
// BAD (current, lib.rs:1778-1795): fixpoint over the FINAL last-wins snapshot.
// `acc` already collapsed A to its last value, so B=$A resolves to A's LAST value.
for _ in 0..8 {
    let snapshot = acc.clone();
    for (_, value) in acc.iter_mut() {
        if value.contains('$') {
            *value = resolve_binding_refs(value, &snapshot); // wrong: uses final A
        }
    }
}

// GOOD: resolve each RHS against the values visible at THAT assignment point,
// in source order, so B=$A captures A's value-so-far (bash semantics).
// Build an ordered list of (name, rhs) as they appear; fold left:
let mut resolved: Vec<(String, String)> = Vec::new();
for (name, rhs) in ordered_assignments {            // source order, incl. each =/+= step
    let value = resolve_binding_refs(&rhs, &resolved); // only left-of-here bindings
    match resolved.iter_mut().find(|(n, _)| *n == name) {
        Some(e) if is_append => e.1.push_str(&value),
        Some(e) => e.1 = value,
        None => resolved.push((name, value)),
    }
}
```
Note: this requires `collect_same_line_bindings` to preserve per-step ordering (currently it collapses `=`/`+=` eagerly), so the reference-resolution sees the pre-overwrite value. A value still carrying `$`/backtick after left-fold stays unresolved => fail closed at the use site (existing behavior, correct).

---

## Attack surfaces that held (no findings)

### AS1 r18 fixes — other branches CLEAN
- **Positional defaults** `${N:-d}`/`${N:=d}`/`${N-d}` vs `set --` (`resolve_positional_default`, lib.rs:1809): 11/11 correct — `set -- reset; ${1:-x}` DENY; `${10:-x}` with <10 positionals ALLOW; with >=10 positionals DENY; `shift` reindex DENY/ALLOW correct; `${0:-x}` ALLOW; unbound `${1:-reset}` default-injects DENY.
- **Nested-substitution-preserve restricted to echo** (`static_command_output`, lib.rs:2361-2365): 6/6 + 6 echo-flag cases correct. `printf "$(echo reset) --hard"` DENY (no r18 F1 regression), `echo $(echo reset) --hard` DENY, nested backticks DENY, `echo -e/-E/-ne/-en/-nE` DENY, `xpg_echo` DENY.

### AS2 prior-corpus regression (r1-r17) — 24/24 still DENY
env/GIT_DIR context, nice/timeout/firejail/flock wrappers, eval, IFS resplit, function inlining (single + multi-hop), brace `{reset,}`, ANSI-C `$'reset'`, command-sub `$(echo reset)`, positionals `set -- … $@`, `--git-dir`/`--work-tree`, `read -ra` arrays, multi-word binding `P="…"`, dynamic `-C "$(printf …)"`, `command` prefix, `-c alias.co=checkout`, `printf "%s --hard"`, line-continuation, `P+=" --hard"`, `printf "reset\t--hard"`. Residual runtime-unknowable (`$(cat …)`, `$(curl …)`) correctly ALLOW.

### AS3 universality/hermeticity — CLEAN
- No hardcoded user paths/repo names/machine assumptions outside the `#[cfg(test)]` module (lib.rs >= line 3140). Non-test scan empty. main.rs has zero hardcoded paths.
- Canonical derivation = `OYA_CANONICAL_CHECKOUT` override (absolute or cwd-relative, main.rs:46-62) + `git rev-parse --show-toplevel`/`--git-common-dir` default. `.git`/`.git/worktrees` literals are git-universal conventions, not machine assumptions.
- Foreign-repo proof: `/srv/acme/checkout` DENY/ALLOW behavior identical to oyatie (F1-F4 correct); the CRITICAL bug reproduces on the foreign repo too (machine-agnostic defect, not an oyatie-specific quirk).
- Relative-`OYA_CANONICAL_CHECKOUT` cwd-resolution observed once as ALLOW only when the harness sandbox cwd differed from the literal target path (test artifact); with consistent cwd/canon it DENYs (`.` canon + bare `git reset` => DENY). NOT a finding.

### False-positive check — 8/8 legitimate worktree commands ALLOW
`git -C <worktree> status|commit -m wip|switch|reset --hard|clean -fdx|{reset,} --hard`, `cd <worktree> && git reset --hard`, `git worktree add ../lane branch` — all ALLOW. Chained-binding worktree commands (`A=reset;B=$A;A=log; git -C <worktree> $B --hard`) ALLOW. No over-denial on legitimate worktree usage.

---

## Findings ranked

| # | Severity | Finding | Reproducing payload (head 11aa80368) |
|---|----------|---------|--------------------------------------|
| 1 | **CRITICAL** | Stale-overwrite transitive-binding resolution: `resolve_binding_refs` uses the last-wins snapshot, not the value at the referencing assignment, so a shadowed source var hides the mutating verb (and the `-C canonical` retarget). Entire blocked-verb surface bypassable, repo/machine-agnostic. | `A=reset; B=$A; A=log; git -C /Users/jasonlee/Developer/oyatie $B --hard` => ALLOW (exit 0); bash runs `git reset --hard` on canonical. |

No HIGH/MEDIUM/LOW findings.

---

## Verdict rationale
The strict VERDICT RULE permits APPROVE only if the sole residual bypass class is runtime-unknowable dynamic content (`$(cat …)`, `$(curl …)`, stdin reads). Finding #1 is the opposite: fully static, single-line, parser-modeled text resolved with incorrect (non-bash) ordering semantics — a regression introduced by the r18 transitive-binding fix itself. It reproduces deterministically against the live binary and grants full canonical-checkout mutation. This mandates REQUEST_CHANGES.

VERDICT: REQUEST_CHANGES
