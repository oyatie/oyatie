# Plan: email-comms-kernel-inbound-dmarc-alignment-disposition

## Objective

Extend `oya-shared-email-comms-kernel` with a pure inbound DMARC alignment + disposition
evaluator. RFC 7489 §3.1 mandates that a receiving MTA evaluate SPF and DKIM alignment
against the RFC 5322 `From:` header domain, then apply the sender's published DMARC policy
to produce a concrete disposition.

## RFC 7489 Key Rules

- **Alignment** is checked against the *RFC 5322 From header domain* (not envelope sender).
- **Pass condition**: SPF aligned OR DKIM aligned (OR-semantics; either is sufficient).
- **Strict alignment**: exact domain match.
- **Relaxed alignment** (default): organizational domain suffix match — `mail.example.com`
  aligns with `example.com` under the same eTLD+1.
- **Disposition mapping**:
  - DMARC pass → `Accept` (regardless of policy).
  - DMARC fail + `p=none` → `Accept` (monitor only).
  - DMARC fail + `p=quarantine` → `Quarantine`.
  - DMARC fail + `p=reject` → `Reject`.

## Edge Cases

1. Empty SPF result domain or empty DKIM d= domain → not aligned (never panic).
2. Case-insensitive domain comparison (RFC 1035 domain names are case-insensitive).
3. Single-label domains (no dot) under relaxed mode: fall back to exact match only.
4. Organizational domain extraction: strip the leftmost label only (simple two-label
   eTLD+1 approximation — sufficient for unit tests; production callers supply real domains).
5. Both SPF and DKIM pass alignment simultaneously → still `Accept` (idempotent).
6. `p=none` with alignment pass → `Accept` (policy irrelevant when pass).

## Subtasks (ordered)

1. [x] Write plan (this file).
2. [x] Write spec (`docs/specs/task-email-comms-kernel-inbound-dmarc-alignment-disposition.md`).
3. [x] Write red tests in `src/lib.rs` — confirm they fail with `cargo nextest --no-run` or
       `cargo check --all-targets` (compilation failure proves red).
4. [x] Implement `DmarcAlignmentInput`, `DmarcAlignmentMode`, `DmarcEvalVerdict`,
       `DmarcDisposition`, and `evaluate_inbound_dmarc` in `src/lib.rs`.
5. [x] Verify green: `cargo nextest run -p oya-shared-email-comms-kernel`.
6. [x] Self-review: correctness / security / naming.
7. [x] Simplify: guard clauses, dead code, naming consistency.
8. [x] Commit + push + open PR.
