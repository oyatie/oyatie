# Implement lifecycle (mechanical TDD → review → simplify → harden)

**Doctrine:** Implementation is not free-form coding. Every admitted slice runs a **mechanical** sequence.  
SSOT stage list: `harness/pipeline.json` → `implement_loop.sequence`.  
Parent path: `programs/INTENT-LIFECYCLE.md` (intent → harden → dispatch → … → here).

## Sequence (fail closed)

```text
RED_TEST          write failing tests; proof suite exits non-zero; product_code_changed=false
    ↓
IMPLEMENT         minimal code to satisfy RED only (EXECUTOR)
    ↓
GREEN_TEST        same suite (or documented strict superset) exits 0
    ↓
INTEGRATION_TEST  boundary / integration coverage (or explicit N/A waiver for pure docs)
    ↓
FALSE_GREEN_SCAN  mechanical anti-cheat (skipped tests, deleted RED, weakened asserts, generated faces)
    ↓
REVIEW_DIFF       dual orthogonal CRITIC (cross_model when required)
    ↓
SECURITY_DIFF     when risk warrants
    ↓
SIMPLIFY          reduce complexity; re-run GREEN
    ↓
HARDEN            fail-closed edges, telemetry, error paths; re-run GREEN + integration
    ↓
VERIFY            final command packet (verify_report.v1)
    ↓
ADMIT_SLICE       admit or loop next slice / process_edit
```

## TDD rules (mechanical)

| Rule | Fail closed |
|------|-------------|
| No IMPLEMENT without RED proof (`proof_failed=true`, `product_code_changed=false`) | yes |
| GREEN must re-run the **same** RED suite commands (or strict superset listed) | yes |
| Deleting or `@ignore`/skip of RED tests to go green | reject |
| Weakening asserts / `assert true` / empty tests | reject |
| Hand-edit `*.generated.json` | reject |
| Claiming green without recorded exit codes | reject |

## False-green scan (minimum checks)

`false_green_scan.v1` must report `clean=true` with:

- `no_skipped_tests`
- `no_deleted_red_tests`
- `no_weakened_asserts`
- `no_generated_face_hand_edit`
- `red_proof_present`
- `green_same_suite`
- `verify_commands_nonempty`
- `no_ignore_attribute_added_to_hide_fail`

Any false ⇒ stage fail ⇒ process_edit if class repeats.

## Review

- Dual CRITIC on **diff + tests** (`REVIEW_DIFF`), not only “LGTM”.
- Independence: `cross_model` when `require_cross_model_critics` (see `safety.md`).
- SECURITY_DIFF for medium+ risk.

## Simplify then harden (order matters)

1. **SIMPLIFY** — delete dead code, collapse branches, no behavior change; GREEN re-run.  
2. **HARDEN** — fail-closed, validation, telemetry, constant-work; GREEN + integration re-run; **no feature creep**.

Reversing order invites “harden” that adds features under the guise of safety.

## Roles

| Stage | Role (`mm-role`) |
|-------|------------------|
| RED_TEST / INTEGRATION_TEST | `TEST_ENGINEER` |
| IMPLEMENT | `EXECUTOR` |
| GREEN_TEST / VERIFY | `VERIFIER` |
| REVIEW_DIFF | `CRITIC` ×2 |
| SECURITY_DIFF | `SECURITY` |
| SIMPLIFY | `SIMPLIFIER` |
| HARDEN | `HARDENER` |
| FALSE_GREEN_SCAN / ADMIT_SLICE | harness (deterministic) |

## Waivers

Only **human-journaled** waivers:

- Integration N/A for pure docs/disposition with no runtime code  
- Simplify noop if already minimal (still re-run green)  

Waivers never cover: missing RED, skipped tests, same-family critic laundering as multi-model.

## Self-improvement

Failures of TDD/false-green classes append `process_edits.md` and may strengthen:

- `admit_rules` / `implement_loop` in `pipeline.json`  
- EXECUTOR / TEST_ENGINEER system prompts  
- `false_green_scan` checks  

LEARN at wave end remains mandatory.
