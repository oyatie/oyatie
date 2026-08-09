---
doc_class: DesignNote
title: Workload-Identity Failure Modes (Fail-Closed Posture)
microservice: identity
bounded_context: workload-identity
status: Proposed
date: 2026-05-26
owner_team: axis-identity + ops-security
related_adrs: [ADR-0002, ADR-0183]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Workload-Identity — Failure Modes

The governing rule, from the cited brief (§10): **fail closed everywhere on the
authorization path.** No key → reject. Store unreachable → embedded default-deny.
Bad token → reject. Uncertainty is denial. This document enumerates each failure
mode, the resolved behavior, and why.

## Principle

A workload-identity substrate that fails *open* hands the fleet to an attacker
during exactly the moments it is most stressed. Cedar's formally-proven
default-deny (brief §10) and AVP's implicit-deny-unless-explicit-permit are the
models we follow: the *absence* of a positive signal is a denial, never a pass.

## Failure mode table

| # | Failure | Resolved behavior | Why (brief ref) |
|---|---|---|---|
| F1 | **JWKS fetch fails, valid cache present** | Use the last-known-good cached keys; serve normally; schedule proactive refresh | JWKS resilience: respect cache-control, last-known-good in memory (§10) |
| F2 | **JWKS fetch fails, cache empty/expired** | Hard-deny: `503` / `jwks-unavailable`; validation refused | Total-fail-no-cache → fail closed (§10) |
| F3 | **JWKS key-set exceeds cap** | Cap the held set at ≤100 keys; refuse to grow unbounded | Azure stores only first 100 signing keys (§10) |
| F4 | **Unknown `kid`** | Reject; never try all keys | Pipeline step 1 (§1) |
| F5 | **Token header `alg` ≠ server-bound alg** | Reject `algorithm-mismatch`; algorithm chosen server-side | RFC 8725 §3.1 (#1 vuln class) |
| F6 | **`alg:none`** | Reject `alg-none-rejected` unconditionally | RFC 8725 §3.2 |
| F7 | **Clock skew at boundary** | Enforce ≤60s skew; never disable | RFC 9068 (§10) |
| F8 | **Policy store unreachable** | Embedded in-process Cedar renders a decision; if even that cannot, default-deny | Policy-store-unavailable → embedded Cedar default-deny (§10) |
| F9 | **No policy matches** | `DENY` with empty `determiningPolicies` (implicit deny) | AVP implicitly denies unless explicit permit (§10, §2) |
| F10 | **Principal suspended/retired** | `DENY` via `forbid-suspended-principal`; consulted from the fast denylist | Suspend/retire → denylist at validate-time (§1) |
| F11 | **Lifecycle write lag (control-plane eventually consistent)** | Do NOT gate hot-path authorize on a just-written *activation*; DO gate on the denylist for suspend/retire | Decouple control-plane from data-plane; accept brief activation lag (§10) |
| F12 | **Malformed compact JWS** | Reject `malformed-token`; no partial parsing trusted | Pipeline fail-fast (§1) |
| F13 | **`jku`/`x5u` not allowlisted** | Reject `jku-not-allowlisted` | SSRF defense (§5, §10) |
| F14 | **Audit-chain emission stalls** | Decision still returned (availability), but a stalled seal is alerted and back-filled; never silently dropped | Decision log is the primary audit substrate, emit unconditionally (§9) |
| F15 | **Batch contains one bad tuple** | That item gets a per-item `errors[]` entry; the batch is not failed wholesale | Batch independence (§8 batching) |

## Activation lag is the one acceptable looseness

The only intentionally-loose path is **activation** (provision→active): because
the control plane is eventually consistent ("several seconds", brief §4), a
brand-new principal may briefly not authorize. That is acceptable and safe — it
errs toward denial. Revocation (suspend/retire) is NOT loose: it is enforced via
the fast denylist at validate time so a compromised identity is cut off within
`token_ttl + denylist_propagation` (brief §1, §10).

## What the PEP must do

A PEP MUST treat any non-`ALLOW` answer — including transport errors, `503`, and
`DENY` — as **deny**. An unreachable PDP is a denied request, never an allowed
one. This is stated in the OpenAPI `503` response descriptions and is an
acceptance criterion (AC-W-11, AC-W-12).

## References

Brief §1, §4, §9, §10; RFC 8725 §3.1–3.2; RFC 9068. Operational drill detail:
`runbooks/jwks-fetch-failure.md`, `runbooks/policy-store-unavailable.md`.
