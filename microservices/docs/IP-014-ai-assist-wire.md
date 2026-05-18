---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-014-ai-assist-wire
status: pending
execution_unit: ChangeSet
owner: axis-docs + foundry-runtime + council-privacy
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-ai-act-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: AI writing-assist wire (T0/T1/T2 per ADR-DOCS-0005)

## Intent

Wire T0/T1/T2 capabilities from `capabilities/*.yaml` to the editor REST + worker. Tenant-DEK-wrapped prompts via foundry-runtime SDK. Cedar policy gates per capability tier. EU AI Act Annex III §3 HR-context REFUSED at Cedar layer for pack-eu per ADR-DOCS-0005.

## ChangeSet boundary

Capabilities surface crates: shared AI-assist port + per-capability adapters under document-store-usecase + cross-cutting Cedar policy.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-ai-assist-shared/src/{lib,prompt_envelope,capability_dispatcher,reversibility_window}.rs` | create |
| `microservices/docs/src/crates/oya-docs-document-store-usecase/src/{ai_grammar_suggest,ai_auto_summary,ai_expand_rewrite,ai_citation_suggest,ai_grammar_bulk_fix,ai_auto_translate,ai_auto_format,ai_auto_cite}.rs` | create |
| `microservices/docs/policy/ai-act-hr-scope.cedar` | create (NEW; refuses T1/T2 HR-context in pack-eu) |
| `microservices/docs/policy/t1-tenant-tier-gate.cedar` | create |
| `microservices/docs/policy/t2-tenant-tier-gate.cedar` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-ai-assist-shared -- prompt_envelope_tenant_dek_wrapped
cargo nextest run -p oya-docs-ai-assist-shared -- reversibility_window_audit_emit
cargo run -p oya-dev-cli -- gate validate ai-act-conformance --microservice docs
```

## References

- ADR-DOCS-0005 (AI writing-assist EU AI Act bounds).
- `capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`.
- EU AI Act Regulation (EU) 2024/1689.
