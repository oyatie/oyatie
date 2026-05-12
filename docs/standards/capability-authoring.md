# Oyatie — Foundry Capability Authoring Standard

> **Owner:** `axis-foundry`.
> **Companion:** [`templates/capability-record-template.yaml`](../templates/capability-record-template.yaml), [`checklists/foundry-capability-publishing.md`](../checklists/foundry-capability-publishing.md), [DESIGN §3.0](../DESIGN.md), ADR-0021 (Foundry capability registry + MCP gateway), ADR-0022 (autonomy ceiling), ADR-0024 (eval harness).

## 1. What is a capability

A discrete unit of agent-invocable functionality with declared inputs / outputs / autonomy / data-class / cost / sunset. Per ADR-0021, capabilities are the unit of consumption from MCP clients (Claude Desktop / Cursor / Continue / Cline / OpenAI Apps SDK / etc.).

## 2. Authoring

Per [`templates/capability-record-template.yaml`](../templates/capability-record-template.yaml):

1. Choose `id` per [GLOSSARY.md §10](../GLOSSARY.md) naming conventions; namespace matches owning axis
2. Write description for both agent and human readers (MCP-discoverable)
3. Define input + output schemas as JSON Schema; `required` list explicit
4. Declare side effects (`reads_tenant_data`, `writes_tenant_data`, `reads_external`, `writes_external`, `emits_events`, `invokes_other_capabilities`)
5. Declare autonomy tier required per ADR-0022 (T1/T2/T3/T4)
6. Declare data classes touched per ADR-0008 (exhaustive)
7. Declare regulatory packs consumed per [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md)
8. Declare cost profile (per-invocation USD ceiling, monthly USD ceiling per tenant)
9. Declare provider preference + failover (Anthropic / OpenAI / Gemini × api / subscription; oya-internal post W-AI-Model-Substrate)
10. Declare evidence-emission topic per ADR-0003
11. Declare sunset policy per ADR-0037 + ADR-0019 (announce + EoL window + migration target)

## 3. Eval set (mandatory)

Per ADR-0024:
- Golden inputs ≥ 20 across positive + negative + adversarial cases
- Expected outputs (exact match or scored metric)
- Eval metric chosen (BLEU / ROUGE / exact-match / per-class F1 / human-judged)
- Pass threshold declared
- Adversarial cases include prompt-injection, data-class violation, autonomy-tier bypass attempts
- Per-region linguistic eval (KR + EN minimum; JP if pack onboarded)
- Eval set Cosign-signed per ADR-0039

## 4. Privacy + safety

- Class allowlist (per-tenant per-capability via Cedar policy)
- Tenant-class override applied (e.g. healthcare tenant + PHI = HARD_DENY)
- Subject-class check (minor / vulnerable)
- Inference-boundary check (derived attributes inherit most-restrictive class per ADR-0008 §2.2.5)
- DSR cascade hook into the DSR pipeline so revoked-consent records cascade-purge from cache
- Audit-chain emission wired to declared topic per ADR-0003
- Prompt-injection taint zones (untrusted content marked; downstream tools refuse)

## 5. Cost + budget

- Per-invocation cost ceiling enforced at the router (hard stop)
- Per-tenant monthly budget wired; soft warn at 80%, hard stop at 100%
- Quota documented for the capability's UI surface

## 6. Documentation

- Per-capability docs page at `docs.oyatie.com/capabilities/<id>/` with Diátaxis 4-quadrant per [DOCUMENTATION.md §3](../DOCUMENTATION.md): tutorial + how-to + reference + concept
- MCP tool descriptor auto-generated from capability YAML
- Per-vertical examples where applicable

## 7. Publish

Per [`checklists/foundry-capability-publishing.md`](../checklists/foundry-capability-publishing.md).

## 8. Anti-patterns

- Skipping eval set — never; CI gate fails
- Class allowlist set to "all" — never; explicit list required
- Autonomy tier T4 (auto-execute) for regulated capability — never (T2 max for fintech / T1 for healthcare safety)
- Cost ceiling unset — never; default ceiling applies
- Side effects undeclared — never; CI gate detects via runtime trace
- Capability published without per-language linguistic eval — never (KR + EN minimum)

## 9. Sources
ADR-0003/0008/0019/0021/0022/0024/0037/0039, [DESIGN §3.0](../DESIGN.md), [`templates/capability-record-template.yaml`](../templates/capability-record-template.yaml), [`checklists/foundry-capability-publishing.md`](../checklists/foundry-capability-publishing.md).
