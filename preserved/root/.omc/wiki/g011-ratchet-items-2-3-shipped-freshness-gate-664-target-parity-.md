---
title: "G011 ratchet items 2-3 shipped — freshness gate (#664) + target-parity gate (#665)"
tags: ["G011", "ADR-0539", "ADR-0540", "FRIC-1781082000", "FRIC-1781063357", "ultragoal"]
created: 2026-06-10T13:27:20.725Z
updated: 2026-06-10T13:27:20.725Z
sources: []
links: []
category: session-log
confidence: medium
schemaVersion: 1
---

# G011 ratchet items 2-3 shipped — freshness gate (#664) + target-parity gate (#665)

# G011 items 2–3 shipped — 2026-06-10 (same session as #661/#662)

**dev @ `2c097d181`. Session total: 5 PRs merged (#661, #662, #664, #665 + #660 pre-verified).**

## PR #664 — freshness gate (ADR-0539, FRIC-1781082000 RESOLVED)
`oya-cloud-ci-freshness-app`: lock-freshness via oya-workspace-members-kernel (3 violation codes) + face-freshness reusing the EXACT CI producer binaries (byte-parity, no reimpl false-green). Remediation commands verbatim in findings. dev-cli `gate run-all --ci-required` lane (local bridge) + standalone buck2-binary CI job (canonical, no needs-edges). generated-output-diff-policy upgraded blunt-block → precise byte-equality (ADR-documented). APPROVE first pass, CI 18/18. **Scored its first real catch within the hour** — caught 2 stale faces on PR #665 (one diagnosed push vs #662's two blind round-trips).

## PR #665 — target-parity gate (ADR-0540, FRIC-1781063357 ratchet active)
Measured: 817 members, all have BUCK, but **634 have test code with NO rust_test target** (their tests never compile in CI — the PR #645 sqlx-Debug false-green class); 74 benign (no tests). Gate: `member_missing_buck` born-blocking frozen-empty + `member_test_code_without_rust_test_target` baseline-block-on-new with the 634 keys mechanically frozen (reviewer re-derived byte-exact — zero padding/omission). Gate dogfoods its own rust_test. Firewall registration data-driven (auto-covers future gates). APPROVE first pass, CI 19/19.

## Verified state changes
- dev now **locally green** on the 4 formerly-red buck2 gate tests (Pass 4/Fail 0 on 2705d1c96) — FRIC-009 local-materialization symptom cleared by fresh faces + the freshness gate keeping them fresh.
- Lane pattern (now proven 3×): spec file → `omc team 1:codex --no-decompose` with FILE-BASED brief (long inline texts break omc team arg parsing) → tmux nudges (text, settle, separate Enter) → fresh-context adversarial review with re-derivation probes → merge train (base==tip ⇒ green==projected state).

## Open follow-ons
1. **634-key target-parity burn-down** — mass rust_test BUCK wiring; will surface latent uncompilable tests (intended). Big parallel-team candidate.
2. NativeLink remote cache — NEEDS FOUNDER hosting decision.
3. Corpus-liveness-graph research → Proposed ADR.
4. Founder-held: #651 identity ratification, #644 sanction-or-close, ADR-0536/0537 (+0538/0539/0540 now also Proposed).
