---
doc_status: published
---

# Checklist: Foundry capability publishing

> **When:** New Foundry capability ready to publish to the registry. Authoring agent or human follows this list before requesting merge.
> **Owner:** Capability author + reviewing team owner.
> **Validator:** `capability-schema-validator` + `capability-eval-coverage` + `oya agent capability publish` dry-run.
> **Template:** [`templates/capability-record-template.yaml`](../templates/capability-record-template.yaml)

---

## Authoring

1. ☐ **Capability ID + namespace** chosen per [GLOSSARY.md §10](../GLOSSARY.md) naming conventions; namespace matches owning axis.
2. ☐ **Description** written for both agent and human readers (MCP-discoverable).
3. ☐ **Input schema** defined as JSON Schema; `required` list explicit.
4. ☐ **Output schema** defined as JSON Schema.
5. ☐ **Side effects** declared: `reads_tenant_data`, `writes_tenant_data`, `reads_external`, `writes_external`, `emits_events`, `invokes_other_capabilities`.
6. ☐ **Autonomy tier required** declared per [ADR-0022 persona tier model](../decisions/) — T1/T2/T3/T4.
7. ☐ **Data classes touched** declared per [PRIVACY-PROGRAM §2.2.1](../PRIVACY-PROGRAM.md) — listed exhaustively.
8. ☐ **Regulatory packs consumed** declared (per COMPLIANCE-MATRIX); especially KR PIPA / GDPR / HIPAA / PCI / FSC.
9. ☐ **Cost profile** declared: per-invocation USD ceiling, monthly USD ceiling per tenant.
10. ☐ **Provider preference + failover** declared (Anthropic / OpenAI / Gemini × api / subscription; Oyatie-internal post W-AI-Model-Substrate).
11. ☐ **Evidence-emission topic** declared per ADR-0003.
12. ☐ **Sunset policy** declared per ADR-0001 / ADR-0040 (announce + EoL window + migration target).

---

## Eval set (mandatory; no capability without eval)

13. ☐ **Golden inputs** authored — minimum 20 across positive + negative + adversarial cases.
14. ☐ **Expected outputs** authored — exact match or scored metric.
15. ☐ **Eval metric** chosen — per-capability appropriate (BLEU / ROUGE / exact-match / per-class F1 / human-judged).
16. ☐ **Pass threshold** declared — capability publishes only if eval pass-rate ≥ threshold.
17. ☐ **Adversarial eval cases** include — prompt-injection attempts, data-class-violation attempts, autonomy-tier-bypass attempts.
18. ☐ **Per-region linguistic eval** — at minimum KR + EN; JP if pack onboarded.
19. ☐ **Eval set is checked-in + signed** — Cosign-signed per ADR-0039.

---

## Privacy + safety

20. ☐ **Class allowlist** — per-tenant per-capability data-class allowlist verified (Cedar policy).
21. ☐ **Tenant-class override** — vertical-pack overrides applied (e.g. healthcare tenant + PHI = HARD_DENY for this capability).
22. ☐ **Subject-class check** — minor / vulnerable / etc. checks applied.
23. ☐ **Inference-boundary check** — derived attributes inherit most-restrictive class per [PRIVACY-PROGRAM §2.2.5](../PRIVACY-PROGRAM.md).
24. ☐ **DSR cascade** — capability emits a hook into the DSR pipeline so revoked-consent records cascade-purge from any cache / retraining feed.
25. ☐ **Audit-chain emission** wired to the declared topic.
26. ☐ **Prompt-injection taint zones** — untrusted content marked; downstream tools refuse.

---

## Cost + budget

27. ☐ **Per-invocation cost ceiling** enforced at the router (hard stop).
28. ☐ **Per-tenant monthly budget** wired; soft warn at 80%, hard stop at 100%.
29. ☐ **Quota documented** for the capability's UI surface.

---

## Documentation

30. ☐ **Per-capability docs page** at `docs.oyatie.com/capabilities/<id>/` — tutorial + how-to + reference + concept (Diátaxis 4-quadrant per [DOCUMENTATION.md §3](../DOCUMENTATION.md)).
31. ☐ **MCP tool descriptor** auto-generated from capability YAML.
32. ☐ **Per-vertical examples** provided where applicable.

---

## Publish

33. ☐ Open PR with `## Issue / Summary / Verification / Code Review` (per CLAUDE.md).
34. ☐ Include eval-set output in `## Verification`.
35. ☐ Cross-axis review label set if capability touches a DESIGN §10 contract row.
36. ☐ `oya agent capability publish --dry-run` passes.
37. ☐ After merge, `oya agent capability publish` emits `EVT-CAPABILITY-AUTHORED` to audit chain.

## After publish

- ☐ Add to per-tenant marketplace if customer-facing
- ☐ Subscribe to capability-eval drift alerts
- ☐ Schedule next eval refresh per cadence (default: weekly for first month, then monthly)
