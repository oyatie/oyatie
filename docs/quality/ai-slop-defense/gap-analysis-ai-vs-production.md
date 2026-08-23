---
doc_class: GapAnalysis
parent: INDEX.md
status: Accepted
purpose: |
  Cross-map AI-slop failure modes (catalogue) onto production-quality
  dimensions (bar). Identify gaps already closed by oyatie tooling vs
  gaps that remain open. Surface the top-3 most-dangerous open gaps.
owner: council-architecture
date: 2026-05-12
adr_citations:
  - ADR-0053
  - ADR-0055
doc_status: published
---

# Gap Analysis — AI-Slop vs Production Quality

> Accepted per ADR-0053 (fitness-lane governance) and ADR-0055
> (impossible-to-fail measurement contract). Gap closure trajectory in
> §4 is a binding commitment tracked against ADR-0055 §6 score formula.

## 1. Failure-mode → quality-dimension(s) violated

| Failure mode | Violates dimension(s) | Current oyatie coverage | Status |
|---|---|---|---|
| AIS-001 Hallucinated package | D26 + D27 + D28 | `cargo-deny [licenses]` partial; `cargo-vet` not adopted; allowlist absent | **OPEN** (HIGH) |
| AIS-002 Hallucinated stdlib fn | D31 (compile-clean) | `cargo nextest` mandated → compile-fail blocked | CLOSED |
| AIS-003 Hallucinated CLI flag | D36 (doc freshness) | partial (rustdoc doctest) | OPEN (M) |
| AIS-010 `.unwrap()` in prod | D07 + D08 | clippy `unwrap_used` not workspace-denied | **OPEN (C)** |
| AIS-011 Empty catch / log-only | D06 | no lane today | **OPEN (C)** |
| AIS-012 `let _ = result` | D06 | clippy `let_underscore_drop` not denied | OPEN (H) |
| AIS-013 unwrap_or_default mask | D06 | no lane today | OPEN (M) |
| AIS-020 Premature abstraction | D03 + Linus good-taste | PR template "good-taste-audit" exists; not lane-enforced | OPEN (M) |
| AIS-021 Dead code | D03 | `dead_code` warn (not deny) | OPEN (M) |
| AIS-022 Trait explosion | D03 | no lane today | OPEN (L) |
| AIS-030 Missing edge cases | D32 (fuzz) | `cargo-fuzz` not adopted | **OPEN (H)** |
| AIS-031 Off-by-one | D33 (mutation) | `cargo-mutants` not adopted | OPEN (H) |
| AIS-032 Null-deref on input | D14 + D32 | partial | OPEN (H) |
| AIS-040 IO in kernel crate | D01..D04 + kernel-purity | flat-crates lane partial | OPEN (H) |
| AIS-041 Provider-specific in kernel | D04 | `governance-provider-coupling` | CLOSED |
| AIS-042 Logic in adapter | D04 | partial (inverse rule pending) | OPEN (M) |
| AIS-050 Deprecated Rust idiom | D31 | clippy `-D warnings` | CLOSED |
| AIS-051 Stale K8s API | D26 | no lane today | OPEN (M) |
| AIS-052 Stale crypto | D27 + D14 | partial via cargo-deny advisories | OPEN (H) |
| AIS-060 Forked code drift | D03 + D31 | no duplication lane | OPEN (M) |
| AIS-061 Boilerplate replication | D03 | no lane today | OPEN (L) |
| AIS-070 SQLi / shell-injection | D17 | no semgrep lane today | **OPEN (C)** |
| AIS-071 Hardcoded secret | D15 + D17 | gitleaks not adopted | **OPEN (C)** |
| AIS-072 Weak randomness | D14 + D17 | no clippy `disallowed_methods` config | OPEN (H) |
| AIS-073 AGPL/GPL trap | D27 | `governance-license` | CLOSED |
| AIS-074 Hidden-Unicode injection | D14 + D38 | no lane today | **OPEN (C)** |
| AIS-080 Orphan tokio::spawn | D09 | no lane today | OPEN (H) |
| AIS-081 Cancel-unsafe future | D08 | no lane today | **OPEN (C)** |
| AIS-082 Deadlock | D08 | no lane today | OPEN (C) |
| AIS-090 Unbounded alloc | D11 | no lane today | **OPEN (C)** |
| AIS-091 FD / pool leak | D12 | no lane today | OPEN (H) |
| AIS-092 Unbounded retry | D13 | no lane today | OPEN (H) |
| AIS-100 Tautological tests | D33 | no `cargo-mutants` | OPEN (M) |
| AIS-101 Mock-everything | D31 | partial | OPEN (H) |
| AIS-102 Snapshot-on-broken | D34 | `insta` not adopted | OPEN (M) |
| AIS-110 Lying docstring | D36 | `cargo doc` enforced — partial (doctests not always run) | OPEN (M) |
| AIS-111 Fabricated example | D36 | same | OPEN (M) |
| AIS-112 Architecture-map drift | D36 | `governance-architecture-map-freshness` | CLOSED |
| AIS-120 Workspace version skew | D26 | partial via Cargo.lock | OPEN (M) |
| AIS-121 Feature-flag mismatch | D26 | partial | OPEN (M) |
| AIS-122 Transitive license violation | D27 | `cargo-deny` | CLOSED |
| AIS-130 Missing tracing span | D18 | partial | OPEN (M) |
| AIS-131 Missing metric | D19 | no lane today | OPEN (M) |
| AIS-132 Missing audit emit | D21 | `governance-audit-emission` | CLOSED |
| AIS-140 Irreversible schema change | D22 + D23 | `governance-schema-migration` | CLOSED (extend to rollback) |
| AIS-141 Global rollout | D24 | no per-tenant lane today | **OPEN (C)** |
| AIS-150 PII in logs | D20 | `governance-data-class` | CLOSED |
| AIS-151 Missing consent check | D16 | `governance-capability-publish` | CLOSED |
| AIS-152 Cross-region data flow | D16 | no data-residency lane today | OPEN (H) |

## 2. Closed vs open

- **Failure modes already mechanically prevented**: 11 of 42 (26%).
  Existing lanes cover compile-clean, provider-coupling, deprecated
  idiom, license, architecture-map, audit-emission, schema-migration
  (partial), PII/data-class, consent (capability-publish),
  contract-parity, brand-residue.
- **Failure modes with partial coverage**: 9 of 42 (21%). Some clippy
  defaults catch a fraction; PR-template review can catch the rest, but
  there is no mechanical gate so the agent path is not blocked.
- **Failure modes with no mechanical prevention today**: 22 of 42
  (52%). These are the gap surface.

## 3. Top-3 most-dangerous open gaps

Ranked by (severity × likelihood under autonomous-agent operation ×
blast radius).

### Gap-1 — Panic-on-prod (AIS-010 / AIS-011 / AIS-012)

LLMs emit `.unwrap()` and empty catches as default style. Real-world
proof: the [Cloudflare 18-Nov-2025 global outage](https://blog.cloudflare.com/18-november-2025-outage/)
took down ~20% of the public web because a Rust FL2 proxy called
`Result::unwrap()` on a feature-file overflow. The Cloudflare engineers
explicitly noted clippy `unwrap_used = "deny"` would have prevented it
([Hackaday postmortem](https://hackaday.com/2025/11/20/how-one-uncaught-rust-exception-took-out-cloudflare/);
[Medium — Cloudflare unwrap](https://medium.com/@lordmoma/trust-me-bro-the-cloudflare-rust-unwrap-that-panicked-across-330-data-centers-a29f33ef1ba9)).

**Fix**: Adopt `governance-no-unwrap` lane immediately
(workspace `[lints]` with `clippy::unwrap_used = "deny"`,
`clippy::expect_used = "deny"`, `clippy::panic = "deny"`,
`clippy::todo = "deny"`, `clippy::unimplemented = "deny"`,
`clippy::unreachable = "deny"`) + `governance-error-fan-in`
(no empty/log-only catch). Blocker for M01.

### Gap-2 — Supply-chain slopsquat surface (AIS-001 / AIS-074)

Hallucinated dependency names + hidden-Unicode prompt injection are
both unmitigated today. USENIX 2025 measured 20% hallucination rate
across 16 models with 43% repeat-rate — a stable attacker-targetable
distribution
([USENIX 2025 spracklen et al](https://www.usenix.org/system/files/conference/usenixsecurity25/sec25cycle1-prepub-742-spracklen.pdf)).
The "Rules File Backdoor"
([Pillar Security 2025](https://www.pillar.security/blog/new-vulnerability-in-github-copilot-and-cursor-how-hackers-can-weaponize-code-agents))
plants invisible BiDi-control characters into `.cursorrules` /
`.claude/instructions.md` to exfiltrate secrets or alter generated
code. GitHub shipped Unicode-warning May 2025 but oyatie has neither
the warning nor a denial lane.

**Fix**: Adopt `cargo-vet` + dep-allowlist (`governance-dep-allowlist`)
+ Unicode-discipline lane (`governance-unicode-discipline`,
deny non-ASCII in `.cursorrules`, `.claude/`, `.omc/`, `AGENTS.md`,
`CLAUDE.md`, source files; allow only in `i18n/` and `docs/i18n/`).
Blocker for M02.

### Gap-3 — Async unsafety (AIS-080 / AIS-081 / AIS-082)

Orphan `tokio::spawn`, cancel-unsafe `select!`, deadlock — all three
are intrinsic LLM failure modes
([Cybernetist Tokio cancellation](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/);
[Tokio task tracker docs](https://docs.rs/tokio/latest/tokio/task/)).
No oyatie lane currently catches them. Foundry capability runtime +
agent orchestration are entirely async paths; a single orphan task
across 50K runs/week (per M04 done criterion) compounds.

**Fix**: Adopt `loom` model-check lane (`governance-cancel-safety`)
+ `clippy::disallowed_methods` deny of bare `tokio::spawn` (mandate
`intelligence-task-supervisor` wrapper which registers a `TaskTracker`).
Blocker for M02 (Foundry-Preview).

## 4. Coverage trajectory

If the 3 top-gap lanes ship at M01/M02 boundaries, **18 of 22 open
modes** (82%) close mechanically because the new lanes (no-unwrap,
error-fan-in, dep-allowlist, unicode-discipline, cancel-safety,
task-supervisor, fuzz-corpus, mutation-coverage, semgrep-injection,
secret-scan) cascade across multiple modes. The residual 4 modes
(stale K8s API, perf hot-path drift, regional-data crossing,
performance budget drift) close in M01 foundation phases.

## 5. Sources

- [Cloudflare 18-Nov-2025 outage postmortem](https://blog.cloudflare.com/18-november-2025-outage/)
- [Hackaday — Cloudflare unwrap](https://hackaday.com/2025/11/20/how-one-uncaught-rust-exception-took-out-cloudflare/)
- [Medium — Cloudflare panic deep-dive](https://medium.com/@lordmoma/trust-me-bro-the-cloudflare-rust-unwrap-that-panicked-across-330-data-centers-a29f33ef1ba9)
- [USENIX 2025 — Package Hallucinations](https://www.usenix.org/system/files/conference/usenixsecurity25/sec25cycle1-prepub-742-spracklen.pdf)
- [Pillar Security — Rules File Backdoor](https://www.pillar.security/blog/new-vulnerability-in-github-copilot-and-cursor-how-hackers-can-weaponize-code-agents)
- [Cybernetist — Tokio cancellation](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/)
- [Tokio task tracker docs](https://docs.rs/tokio/latest/tokio/task/)
- [The Coded Message — Why Rust should only have `expect`, not `unwrap`](https://www.thecodedmessage.com/posts/2022-07-14-programming-unwrap/)
- [DEV — Beyond the Panic: Hardening the Rust SDK](https://dev.to/yashksaini/beyond-the-panic-hardening-the-rust-sdk-53oj)
- [AWS S3 Feb 2017 outage postmortem](https://aws.amazon.com/message/41926/) — same shape (one typo, no validator).
- [Gremlin — After the Retrospective AWS S3 2017](https://www.gremlin.com/blog/the-2017-amazon-s-3-outage)
