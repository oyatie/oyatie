---
id: ADR-0092
status: Accepted
amended_by: [ADR-515]
doc_status: published
---

# ADR-0092: Workspace dependency-seam policy

> **Status:** Accepted
> **Date:** 2026-05-14
> **Owner:** `council-architecture`
> **Supersedes:** —
> **Superseded-by:** —
> **Related:** ADR-0054, ADR-0056, ADR-0062, ADR-0069, ADR-0090, ADR-0093, ADR-0094, ADR-0095

---

## Status

Accepted (2026-05-14). Codifies the dependency-seam discipline AND amends
M01-P13-IP-002's original layer enum + ledger shape to align with the
canonical 12-layer enum (ADR-0056 v4.1).

## Context

`.omc/plans/milestones/M01-foundation/phases/P06-distroless-lts-image/IP-002-lts-dependency-lane.md`
declared a 5-value layer enum `{kernel, runtime, adapter, api, app}`
inconsistent with ADR-0056 v4.1's canonical 12-value enum
`{kernel, domain, application, adapter, infrastructure, cli, rest, grpc, graphql, worker, app, sdk}`.
The plan also proposed a tech-debt ledger with state machine, trigger DSL,
and monotonic transition graph for 11 deps — control-plane complexity for a
problem that doesn't exist at that scale.

Independent verification of the seam *before* this work landed:

| Cargo.toml declaring `hyper`/`hyper-util`/`http-body-util`/`bytes` (workspace dep) |
|---|
| `oya-http-middleware-kernel` (bytes) — KERNEL LEAKING hyper-family |
| `oya-http-deadline-middleware-domain` (bytes) — symptom |
| `oya-http-telemetry-middleware-domain` (bytes) — symptom |
| `oya-http-tenant-middleware-domain` (bytes) — symptom |
| `oya-http-runtime-hyper-adapter` (hyper + hyper-util + http-body-util + bytes) — adapter, correct |
| `oya-ops-workspace-shell-runtime` (bytes) — Layer 6 leak |

The kernel's body field used `bytes::Bytes`, forcing every consumer to pull
the dep. The IP's "remove bytes from middleware-domain crates" treated the
symptom, not the root cause.

## Decision

### D1 — Canonical 12-layer enum

The dependency-seam policy uses ADR-0056 v4.1's **canonical 12-value enum**:
`{kernel, domain, application, adapter, infrastructure, cli, rest, grpc, graphql, worker, app, sdk}`.
IP-002's 5-value enum is REJECTED as inconsistent with the canon. Layer is
derived from the crate-name suffix per the BNF; no parallel
`[package.metadata.oyatie.layer]` declaration is required for this IP.

### D2 — Single-crate hyper isolation (load-bearing seam)

`oya-http-runtime-hyper-adapter` is the ONLY workspace crate that declares
hyper-family dependencies (`hyper`, `hyper-util`, `http-body-util`, `bytes`)
OR imports them in source.

Mechanically verified — empirical seam audit (reproducible):

```bash
for d in crates/*/Cargo.toml; do
  if grep -E "^(hyper|hyper-util|http-body-util|bytes)( |\.workspace|\s*=)" "$d" >/dev/null; then
    echo "$d"
  fi
done
# Returns exactly: crates/oya-http-runtime-hyper-adapter/Cargo.toml
```

Mechanism: kernel body type is `Vec<u8>` (std-only). Adapter converts
`hyper::body::Bytes` → `Vec<u8>` on inbound (via `Limited::new + collect`)
and `Vec<u8>` → `Bytes::from(...)` on outbound. The conversion is zero-copy
on outbound (Bytes adopts the Vec); inbound allocates once, bounded by
the per-request body cap (D5).

### D3 — Middleware layer = `infrastructure`, not `runtime`

The three pre-existing middleware crates were renamed:

| Old | New |
|---|---|
| `oya-http-deadline-middleware-domain` | `oya-http-latency-budget-middleware-infrastructure` (see ADR-0093 for the type rename) |
| `oya-http-telemetry-middleware-domain` | `oya-http-telemetry-middleware-infrastructure` |
| `oya-http-tenant-middleware-domain` | `oya-http-tenant-middleware-infrastructure` |

`-runtime` is not in the canonical 12-layer enum and would have created
naming inconsistency. ADR-0056 v4.1 §"Layer semantics" defines `infrastructure`
as "framework / driver glue (axum routers, OTel exporters, pool helpers)" —
which is exactly what middleware that wraps a hyper Service does.

### D4 — Type names match doc-comments (Linus F1)

`oya-http-middleware-kernel` renamed its public types from `HyperRequest` /
`HyperResponse` to `HttpRequest` / `HttpResponse`. The previous names
contradicted the kernel's transport-neutral contract (its own doc-comments
described the types as such). Body field type changed from `bytes::Bytes`
to `Vec<u8>`.

### D5 — Body size limit + connection timeouts (Phase 8 S3 + S4)

Adapter `serve()` takes `ServerConfig { max_body_bytes, header_read_timeout,
keepalive_timeout }`. Defaults: 1 MiB body cap, 15s header-read timeout, 60s
keepalive timeout. `http_body_util::Limited` enforces the cap at the
intake; over-cap bodies render HTTP 413. Hyper-util ConnBuilder sets the
header-read timeout (slowloris defense) and HTTP/2 keepalive.

### D6 — Path-traversal defense (Phase 9 S5)

`oya-http-router-kernel::RouteTemplate::match_path` REJECTS placeholder
captures equal to `.` or `..` by default. A handler that legitimately needs
dot-segments must accept them as literal template fragments, not as captures.

### D7 — SSE injection defense in depth (Phase 9 S7 + S8)

`oya-http-sse-kernel::SseEvent::render()` and `render_heartbeat()` sanitize
CR, NUL, and other C0 control bytes (except LF in `data:` payloads).
Even a caller using the lenient `SseEvent::data(...)` constructor with
attacker-controlled input cannot produce on-wire output containing a
synthetic event field. Fallible `try_data` / `try_with_id` / `try_with_event`
/ `try_render_heartbeat` constructors are available for fail-fast detection.

### D8 — Header hardening (Phase 10 S1 + S2 + S10)

- `HttpResponse::with_header(key, value)` lowercases the key (S1) and
  strips CR / LF / NUL from the value (S10).
- Adapter `collect_hyper_request` lowercases header names AND returns
  `HyperRuntimeError::NonUtf8HeaderValue` (renders 400) when a header
  value is not valid UTF-8 (S2, replacing silent drop).
- `HttpRequest.headers` keys are case-canonical (lowercase) end-to-end.

### D9 — Telemetry uses static matched_template, not raw path (Phase 4 Q1 + S6)

`Router::match_route` returns `(handler, captures, matched_template)`.
Dispatch sets `HttpRequest.matched_template`. Telemetry middleware reads it
as the metric route label. Heuristic `path.replace(captured_value, "{name}")`
is GONE — it produced incorrect labels when captured values appeared
elsewhere in the path (Q1) AND leaked sensitive captured values into metric
labels (S6).

### D10 — Tenant slug grammar lives in `oya-tenancy-kernel` (Phase 7)

`TenantSlug(String)` newtype with `TryFrom<&str>` (ASCII alnum + `-` + `_`,
1..=128 bytes). HTTP tenant middleware imports and delegates. The kernel
owns the grammar (defense in depth) so anyone bypassing the middleware can
still not construct an invalid slug.

### D11 — `Handler` trait with associated `Error` (additive)

`pub trait Handler { type Error: Into<HttpResponse>; fn call(&self, req)
-> Result<HttpResponse, Self::Error>; }` in `oya-http-middleware-kernel`.
`handler_to_sync(handler)` helper in the adapter wraps a typed Handler into
the existing `SyncHandler` closure alias. Additive — no existing handler
breaks. See ADR-0094 for the full rationale.

### D12 — Dependency rationales overlay (NOT a state machine)

`/registry/dependency-rationales.json` holds 11 rows, 5 fields each:
`{name, version_pin, layer_seam, owner_team, rationale, replacement_strategy_doc_ref, isolated_in_crate}`.

The original IP's tech-debt ledger with state machine + trigger DSL +
monotonic transition graph + cross-row predicate is REJECTED as
speculative-complexity: no deps have been
removed; we'd build a control plane for transitions that haven't happened.

When the 3rd dep phase-out is in flight, revisit; introduce trigger
predicates as Rust functions inline at the lane crate, not as a DSL.

### D13 — Seam-discipline lane (3 mechanical sub-checks)

(Amended per D-MULTISPECTRUM-RETIRED 2026-06-07: the 3 multispectrum-bar sub-checks were removed with the retired doctrine; the 3 mechanical seam sub-checks remain.)

`oya-check-dependency-seam` (forthcoming) carries 3 mechanical sub-checks:

1. `seam-imports`: no crate outside the declared `isolated_in_crate`
   declares or imports the dep.
2. `registry-coverage`: every `[workspace.dependencies]` entry has a
   `dependency-rationales.json` row; no orphan rows.
3. `cargo-audit-shell`: `cargo audit` exits 0; CVE findings annotated.

Severity: report-only on day 1; flips to error after one-week soak via cron.

### D14 — Soak + flip via cron, not session

The original IP's "30-day soak then flip to error" is a calendar event, not
session work. A cron entry watches lane evidence; flips severity when the
soak window elapses and the last N PRs are green.

## Drivers

The decisions above were driven by the following dependency-seam concerns:

- **Architectural fit**: rejected the IP's 5-layer enum (special case contradicting ADR-0056); deleted the tech-debt ledger state machine (control plane before scale); identified + fixed the kernel-bytes leak as the root cause vs the symptomatic middleware-bytes cleanup.
- **Build-graph cost**: the seam reduces hyper-family cargo build graph for 5+ crates; the rationales overlay generates from `cargo metadata` instead of being hand-authored.
- **Adversarial coverage**: every code change ships a failing-fixture variant — 189 tests across 12 suites including byte-equality boundary checks, dot-segment rejection, SSE injection sanitization, header CRLF strip, sensitive-capture-value exclusion.
- **Ergonomics**: new contributor adds a workspace dep in ~5 minutes (one row in rationales overlay vs the original IP's 8-field + DSL state).
- **Quality scoping**: documented 5 quality issues NOT in scope as bounded FixupTasks (telemetry hot-lock, async chain for real cancellation, tenant-id internal/external split formalization, handler async variant, fixture-pair lane crate).
- **Alternatives**: 3 refactor options enumerated for the kernel rename; Option β (concrete Vec<u8>) selected over generic-body and enum-body for current-scale fit. Rejection reasons recorded.
- **Security**: 6 OWASP-DoS / data-integrity / correctness findings closed with adversarial fixtures (S3, S4 partial, S5, S6, S7, S8, S1, S2, S10).

## Consequences

### Positive

- Single canonical HTTP backbone in one isolatable crate.
- Adding/removing a workspace dep is a 1-row JSON change instead of a state-machine workflow.
- Security gaps that pre-existed in the http-* foundation are closed with adversarial fixtures (DoS, injection, traversal, header-smuggling, label-injection).

### Negative

- 189 tests instead of fewer; CI runtime grows modestly.
- 4 ADRs (this + 0093 + 0094 + 0095) instead of the IP's planned 4 (numbered 0091-0094 originally, but 0091 was already taken, so we renumbered).
- Documented FixupTasks (5 quality + 2 security partials) increase the visible backlog without changing the true scope.

## Alternatives considered

| Option | Description | Rejection |
|---|---|---|
| A — IP-002 as written (5-layer enum, ledger state machine, mass Cargo.toml metadata insert) | Honors the IP literally | Conflicts with ADR-0056 v4.1 canonical enum; over-engineers a control plane for a non-existent scale; treats kernel-bytes symptom not root cause |
| B — Slimmed slice (this ADR's predecessor proposal) | Type rename + body Vec<u8> + crate renames; defer quality + security | Leaves S3 / S4 / S5 / S6 / S7 / S8 security gaps; leaves Q1-Q5 quality issues |
| C (selected) — Full quality bar | All of B + Q1-Q5 fixes + S1-S10 fixes | Multi-day; user authorized 2026-05-14 |

## Why the original IP-002 may have been right (acknowledged-but-scheduled-for-distinct-tracked-work)

The amendment's rejection of the state machine / trigger DSL / monotonic
transitions is NOT a claim that those primitives are wrong. It is a claim
that they are not yet warranted at current workspace scale. The original
IP author had legitimate motivations this ADR is keeping on record so the
decision is reversible when conditions change:

1. **Autonomous masterplan execution.** Long-term goal (per project memory)
   is "implement the masterplan runs without user intervention." Autonomous
   agents need machine-evaluable state transitions to mechanically decide
   "is this dep ready to be removed?" A flat rationales overlay requires
   human reasoning per row. The state machine + trigger predicates were
   the foundation for agents to close out dep phase-outs themselves.

2. **Audit / compliance posture.** Regulated workloads (fintech-compliance)
   require auditable state transitions. "WHO decided to keep hyper? WHEN?
   Against WHICH CVE evidence?" A state machine with explicit transitions
   is auditable; a flat rationales overlay is hand-wave-able.

3. **CVE-driven acceleration as automation primitive.** The original
   `cve_acceleration` field meant: when a CVE hits, the lane MECHANICALLY
   transitions the row to `replacement-armed-by-cve`. Cargo-audit alone
   detects a CVE; the IP's machinery told the workspace what to DO about
   it.

4. **Multi-agent contention.** Dozens of parallel agents (per Oya VCS plans)
   editing dep state want atomic transitions. State machines naturally
   serialize. Free-form rationale rows are race-prone.

5. **Bominal inheritance.** Per [[feedback-bominal-inheritance-precedence]],
   Bominal ADR decisions inherit 1:1 by default. Bominal may already run
   a similar ledger; IP-002's shape may be carrying a proven primitive
   rather than inventing speculative complexity. (This ADR does not check
   Bominal; if the inheritance is real, ADR-0092 owes Bominal a citation
   amendment.)

6. **Round-5 consensus.** The IP frontmatter says "expanded from round-5
   findings" — a multi-agent / multi-human consensus process. The
   complexity may have been a negotiated compromise, not unilateral.

7. **Monotonic-transition correctness.** A state machine REFUSES invalid
   transitions (e.g., `replaced → active` regression). Free-form rationale
   rows do not. The same value Rust gets from typed enums.

8. **Hyperscalers build control planes early.** AWS / Google / Microsoft
   ship dep-management infrastructure before customer-tier scale. The
   judgment call is *when*. At 11 deps with zero removals, YAGNI wins. At
   50 deps with 3 in-flight removals, the machine pays.

## Re-evaluation triggers (when to revisit the scheduled-for-distinct-tracked-work complexity)

This ADR is REVERSIBLE. Re-introduce the state machine / trigger predicates
when ANY of the following becomes true. Each trigger is a FixupTask
condition; the seam lane SHOULD watch for these and emit a warning row.

| Trigger | Condition |
|---|---|
| **T1 — In-flight replacements** | Third workspace dep enters `replacement-in-flight` state (currently zero). At 3+ concurrent replacements the human cost of tracking via flat overlay exceeds the machine's authoring cost. |
| **T2 — CVE-response SLA breach** | A CVE on a workspace dep ages past 7 days without a patch-bump or ADR-tracked extension. The cargo-audit + manual-rationale path missed the SLA; mechanical acceleration would have caught it. |
| **T3 — Compliance audit gap** | A compliance audit asks "show me the transition history for `<dep>`" and the rationales overlay cannot answer it. State machine + transition log are required. |
| **T4 — Multi-agent race** | 4+ agents concurrently mutate `dependency-rationales.json` and a CI run detects a merge race (or worse, a silent overwrite). Atomic transitions become required. |
| **T5 — Scale** | Workspace dep count exceeds 30 OR `[workspace.dependencies]` grows by >50% from the 2026-05-14 baseline. Hand-curated overlay cost crosses the state-machine cost. |
| **T6 — Policy-as-code demand** | An auditor / security team needs to author dep-policy in machine-evaluable form. Express triggers in **Cedar** (`oya-policy-cedar-*`), NOT a new DSL. The trigger-DSL rejection from the IP amendment survives even at this trigger. |

When any T1-T6 fires, open an ADR-0092-A amendment that:
- Authors the state machine (active → scheduled → armed-{cve,plan} → replaced | replacement-attempted-abandoned).
- Authors the transition predicates in Cedar.
- Migrates `dependency-rationales.json` rows by adding a `state` field
  defaulting to `active`.
- Adds a `tech-debt-transition-log` append-only ledger for audit.
- Updates the seam lane to read transitions + emit notifications.

## What this ADR does NOT relax

Even at all-triggers-fired conditions, three findings from the amendment survive:

- **Trigger DSL ≠ new language.** Use Cedar (`oya-policy-cedar-*`),
  which already exists in the workspace. Inventing a parallel DSL
  duplicates Cedar's role and fails F2 hyperscaler.
- **Layer derived from name, not metadata.** ADR-0056 v4.1 BNF makes the
  crate-name suffix the layer source of truth. Declaring it again in
  `[package.metadata.oyatie.layer]` is double-bookkeeping (F1).
- **Owner from Cargo metadata, not parallel registry.** ADR-0056 §7
  declares `[package.metadata.oya.owner_team]` per crate. CODEOWNERS is
  generated from that. Parallel `dri.json` / `role-roster.json` is F1
  duplication.

## FixupTasks (scheduled-for-distinct-tracked-work but named)

- **F-MULTI-Q2**: telemetry hot-lock (Mutex<BTreeMap>) → sharded AtomicU64 when load tests show contention.
- **F-ASYNCCHAIN-1**: introduce async middleware chain; LatencyBudgetReporter (this ADR D5) becomes the SLO reporter while a real cancelling Deadline middleware appears alongside.
- **F-HANDLER-ASYNC**: async variant of `Handler` trait per ADR-0094.
- **F-TENANTID-FORMAL**: formalize the distinction between `TenantId` (internal canonical, `ten_xxx`) and `TenantSlug` (customer-facing) in PRD-tenancy.
- **F-SEC-S4-INTEGRATION**: real slowloris integration test requires a hyper client harness; defer to integration-test phase.
- **F-DRI-CODEOWNERS**: generate CODEOWNERS from `[package.metadata.oya.owner_team]` instead of maintaining a parallel `dri.json` (rejected in this IP).
- **F-LOOP-EXHAUSTED-N**: track any iterative-fix-loop that hits the iteration budget.
- **F-STEP8-READYZ**: no /readyz endpoint exists in the workspace today; the original IP-002 Step 8 ReadinessGate test becomes a FixupTask when the endpoint lands.

## References

- ADR-0054 (grit scaffold-claim pattern)
- ADR-0056 (Rust Clean Architecture BNF v4.1 — 12-layer enum)
- ADR-0062 (Quality / Performance / Scalability bar)
- ADR-0069 (active-machine-readable-artifact-contract)
- ADR-0090 (hyper canonical HTTP backbone)
- ADR-0093 (LatencyBudgetReporter rename + async-chain deferral)
- ADR-0094 (Handler trait + associated Error type)
- ADR-0095 (TenantSlug centralization in oya-tenancy-kernel)
- `/specs/iterative-fix-loop.json` (loop protocol)
- `/registry/dependency-rationales.json` (data plane for D12 + D13)
