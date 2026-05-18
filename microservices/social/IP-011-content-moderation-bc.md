---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-011-content-moderation-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + axis-foundry-runtime + council-privacy
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location, oya-governance-eu-ai-act-conformance]
---

# IP-011: content-moderation BC (kernel → domain → usecase → adapter-clamav + adapter-opswat + worker + sdk)

## Intent

Author the `content-moderation` BC: AI-classifier verdicts via foundry-runtime
T2 capability; manual reviewer queue + appeal workflow per EU DSA Art. 20;
abuse-report ingestion per EU DSA Art. 16; Statement of Reasons emission per
EU DSA Art. 17; EU AI Act Art. 50 transparency-label on every verdict.

**This BC is high-risk per EU AI Act 2024/1689 Annex III §1(a). All Arts.
9-15 + 50 obligations operative.** See ADR-SOC-0003.

## ChangeSet boundary

`content-moderation` BC end-to-end.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-content-moderation-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-content-moderation-domain/src/{moderation_verdict,abuse_report,appeal,classifier_version,statement_of_reasons,eu_ai_act_label}.rs` | create |
| `src/crates/oya-social-content-moderation-usecase/src/{classify,emit_verdict,file_report,open_appeal,resolve_appeal}.rs` | create |
| `src/crates/oya-social-content-moderation-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-content-moderation-adapter-clamav/src/scanner.rs` | create |
| `src/crates/oya-social-content-moderation-adapter-opswat/src/scanner.rs` | create |
| `src/crates/oya-social-content-moderation-adapter/src/foundry_runtime_client.rs` | create |
| `src/crates/oya-social-content-moderation-worker/src/{classifier_loop,reviewer_queue,appeal_resolver}.rs` | create |
| `tests/content_moderation_e2e.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-social-content-moderation-kernel
cargo nextest run -p oya-social-content-moderation-domain
cargo run -p oya-dev-cli -- gate validate eu-ai-act-conformance --microservice social
```

## Test Plan

- AC-09 E2E: moderation classifier verdict → audit-chain seal within 2s + appeal-workflow opens.
- Statement of Reasons emitted with every verdict (EU DSA Art. 17 schema).
- EU AI Act Art. 50 transparency label populated on every verdict.
- Appeal resolution within 7-day SLA.
- pack-us-healthcare: auto-moderation DISABLED on PHI accounts by default.
- Classifier rollback drill per `runbooks/content-moderation-rollback.md`.
- Media scan: ClamAV + OPSWAT positive on synthetic infected file → quarantine.

## Halt Conditions

- EU AI Act golden-set eval regression (macro-F1 < 0.92) — block release.
- Classifier drift (FM-16) → runbook activates.

## Next IP

[`IP-012-search-and-cedar-filter.md`](IP-012-search-and-cedar-filter.md)

## References

- ADR-SOC-0003 (content-moderation-classifier-bounds).
- EU AI Act 2024/1689; EU DSA 2065/2022.
- `microservices/social/runbooks/content-moderation-rollback.md`.
- `microservices/social/runbooks/abuse-report-backlog-drain.md`.
