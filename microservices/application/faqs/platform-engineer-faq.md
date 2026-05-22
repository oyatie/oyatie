# `application` µservice — Platform Engineer FAQ

Real questions raised against `application` by platform engineers (Q1-Q22). Answers are normative — they cite the controlling ADR or
test/lane that enforces the rule.

---

**Q1. Why does `application` exist if we already have `api-gateway`?**

`api-gateway` is the protocol surface — HTTP/3 vs HTTP/2 vs gRPC, mTLS termination, anycast routing, WAF, rate limit. It is
product-agnostic and tenant-naive. `application` is the **product-aware, tenant-aware** dispatch layer that sits behind it.
ADR-0215 §2 makes this split binding. Concretely: `api-gateway` answers "is this a valid HTTP/3 request from a known anycast PoP?";
`application` answers "for tenant T, in product P, in context C, what downstream µservice handles intent I?".

---

**Q2. What is `dispatch::Pipeline::run` and why is it pinned?**

It is the **single** public contract of `oya-application-kernel`. Every inbound request becomes an `Intent`, passes Cedar (`oya-application-port-cedar`),
gets enriched with the tenant + pack overlay, and is routed to exactly one downstream µservice. Pinning means signature changes
require an ADR amendment + a `lean-a10-no-silent-regression` lane sign-off (per `feedback_no_silent_regression.md`).

---

**Q3. How do I add a new `Intent` variant?**

You don't, alone. The `Intent` enum is closed by Cedar policy (`application::IntentSchema`). The workflow:
1. File an ADR amendment under `docs/decisions/`.
2. Get reviewer-agent approval through the Foundry admission gate.
3. Add the variant in `crates/oya-application-domain/src/intent.rs`.
4. Add the Cedar permit in `crates/oya-application-port-cedar/policies/`.
5. Add the dispatch arm in `crates/oya-application-kernel/src/dispatch.rs`.
6. Add a test in `crates/oya-application-app/tests/foundation_flow.rs`.

---

**Q4. Does `application` hold state?**

No. The µservice is stateless across requests. Per-tenant config is hot-loaded from `tenancy` and cached for ≤ 60 s (config-cache TTL).
Audit trail is appended to `audit-chain`. All hard state belongs in `cockroachdb` clusters owned by downstream µservices.

---

**Q5. How do I make a tenant-specific dispatch override?**

Edit `tenancy::TenantConfig::application.dispatch_overrides[intent_name] = downstream_service_id`. The override is hot-applied within
the config-cache TTL. The override is Cedar-gated — a tenant cannot override into a service it doesn't have a permit on.

---

**Q6. What's the difference between `Tier::tenant_class demo_trial` and "dev-cell"?**

Tier::tenant_class demo_trial is a **production** tenant_class with full SLOs. dev-cell is a developer loopback environment that runs the same binary
with `permit_mode = dev-permissive` and a single-node Cockroach. They are not interchangeable; you can't promote a dev-cell to tenant_class demo_trial.

---

**Q7. What protocol does `application` speak?**

HTTP/3 (QUIC v1, RFC 9000) by default per ADR-0253. HTTP/2 fallback is allowed only via per-tenant flag `application.legacy_protocol_allowlist`.
gRPC is HTTP/3 multiplexed (ADR-0145 inter-microservice communication reform).

---

**Q8. How is multi-cell routing decided?**

By shuffle-sharded consistent hashing on `(tenant_id, cell_pool)`. The pool of cells a tenant can land in is determined by tenant_class;
within the pool the route is HRW (Highest Random Weight) hashed on `(tenant_id, intent_kind)`. This matches AWS cellular topology (ADR-0248).

---

**Q9. What is "emergency dispatch" and who can call it?**

`application::Action::EmergencyDispatch` is compliance_pack-bound paid-only. It bypasses normal Cedar permits using a break-glass principal
(`oyatie.governance.break-glass-operator.*`) and writes a high-priority audit event tagged `governance.break_glass = true`. The break-glass
mode auto-expires after 15 minutes and requires reviewer-agent post-approval within 24 hours or the action is auto-reversed.

---

**Q10. How does the µservice handle a Cedar evaluator outage?**

It hard-fails closed by default. `application.fail_mode_on_permit_outage = closed` is the only Cedar-validated value in production.
A regional "permit-outage emergency mode" exists (`open-with-audit-and-cap`) but requires a signed governance ticket + a 5-minute rate cap.

---

**Q11. Can I run `application` outside Cloud Hypervisor?**

In dev, yes. In production, no — ADR-0254 requires Cloud Hypervisor + Kata containers for any µservice that handles cross-tenant traffic.
Bare Linux containers are forbidden.

---

**Q12. What's the canonical timeout chain?**

`api-gateway` (15 s) → `application` (10 s) → downstream µservice (5 s) → DB (1 s). Each layer subtracts 5 s of headroom. If a request
budgets less than the downstream deadline, `application` short-circuits to `503` + audit-chain entry `application.deadline_exceeded`.

---

**Q13. Where do I look for a dropped request?**

The trace ID propagates via `traceparent` (W3C Trace Context, level 2). Query `observability` with:
```bash
oya-obs query --trace $traceparent --service application --window 1h
```

If the request never reached `application`, it died at `api-gateway`. If it died inside, the span tag `application.dispatch.outcome` will be
non-`success` and the `application.dispatch.error_code` will explain why.

---

**Q14. What's the cardinality of `tenant_id`?**

Practically unbounded (every reserved-namespace tenant from `oyatie.gov.kr.k-finance-sec` to `oyatie.b2c.indie.alice@example.com`). The
µservice indexes tenant config by a 128-bit BLAKE3 of the canonical tenant string. There is no per-tenant table — config is keyed by the hash.

---

**Q15. How do I run application against a sovereign cell from a non-sovereign laptop?**

You don't. Sovereign cells (`KR k-finance-sec`, `EU eu-gdpr-strict`) reject any non-attested client. Pair-program over an attested
sovereign jumphost; the jumphost itself runs on Cloud Hypervisor with Nitro Enclaves attestation. The `governance` µservice handles
the human-approval flow for jumphost issuance.

---

**Q16. Can I add a new downstream adapter?**

Yes — add a crate `crates/oya-application-adapter-<service>/`, wire it via the `DispatchTarget` enum in `oya-application-port`,
and add a permit + dispatch arm. Adding adapters is **not** an ADR-class change unless the downstream µservice itself is new.

---

**Q17. Is the µservice horizontally scalable?**

Yes, stateless behind a load balancer. Per-tier autoscale ranges live in `application.autoscale.<tier>` in tenant config. The
load balancer is HRW hashing on `(tenant_id, intent_kind)` to keep cache warm.

---

**Q18. How is per-pack overlay applied?**

`tenancy` ships the active pack set with the config blob. `application` merges per-pack overlays in pack precedence order
(stricter pack wins). The merged config is hashed and cached; a change in any pack invalidates the cache for that tenant.

---

**Q19. What is `application.dispatch.intent_hash` used for?**

It's a BLAKE3-256 of the canonical intent JSON, written into `audit-chain`. Auditors use it to prove that an inbound request matches
an outbound dispatch event one-to-one without seeing tenant payload.

---

**Q20. Where do I find tier definitions in code?**

`crates/oya-application-domain/src/tier.rs` — `Tier::{retired four-label ladder}`. The tier-matrix.md doc cites this enum.

---

**Q21. What happens if a tenant's pack set is incoherent (e.g. SOC2 active but GDPR removed mid-flight)?**

`application` refuses to dispatch with `application::Error::IncoherentPackSet` and emits an audit-chain entry. `governance` then
quarantines the tenant until the pack set is reconciled. Reconciliation is a tenancy-side workflow, not an application-side one.

---

**Q22. Where do I escalate a real production incident?**

`microservices/application/runbooks/incident-dispatch-failure.md` — paged within 60 s, oncall sees the structured runbook with copy-pasteable
commands. The runbook is owned by the `oya-application-*` lane PR-reviewer rotation, not by SRE.
