---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-013-translation-bc
status: pending
owner: axis-recordings + axis-translate
acceptance_lanes: [lean-a2]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: Translation BC — cross-µservice handoff to `translate` µservice

## Intent

Land the translation BC that calls the `translate` µservice via Workflow
events (no direct call per LEAN-A2). Translation request → translate
µservice → translated transcript JSON stored alongside the source.

## Concrete crates

`oya-recordings-translation-{kernel,domain,usecase,api,adapter,worker,sdk,app}`.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice recordings   # cross-product through Workflow
```

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-013-translation-bc
depends_on_changesets: [CS-RECORDINGS-IP-006-transcript-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-012-export-ediscovery-bcs]
enables: []
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Translation request emitted via Workflow event (no direct cross-product call) | `cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice recordings` |
| AC-02 | Translated transcript stored alongside source with language tag (BCP 47) | `cargo nextest run -p oya-recordings-translation-domain -- bcp47_tag` |
| AC-03 | Translation cancellation propagates if source transcript revoked | `cargo nextest run -p oya-recordings-translation-usecase -- cancel_on_revoke` |
| AC-04 | EU AI Act risk class declared (medium-risk per ADR-MEET-0006 analogue) | capability YAML declares risk class |

## Build Sequence

1. Kernel: `TranslationRequester`, `TranslationStore` ports.
2. Domain: `TranslationRequest`, `TranslatedTranscript`, `BCP47LanguageTag`.
3. Usecase: `RequestTranslation`, `IngestTranslation`, `CancelTranslation`.
4. Workflow event subscriber for `translate.translation.v1.completed`.
5. `cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice recordings`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-13 (auto-translated transcript) |
| ADR | ADR-MEET-0006 (EU AI Act bounds for translation — mirrored) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Direct call to `translate` µservice (LEAN-A2 violation) | Lane refuses crate-level cross-product imports |
| Translation request orphaned if `translate` µservice is down | Retry with exponential backoff; max-attempts in event metadata |
| Translation race vs transcript redaction | Cancel on source-transcript revocation |

## References

- BCP 47 (Tags for Identifying Languages) — RFC 5646.
- EU AI Act final text — Regulation (EU) 2024/1689.
- ADR-MEET-0006 (AI feature bounds).
- ADR-0105 / LEAN-A2 cross-product-refusal lane.

## Next IP

[`IP-014-strangler-migration-adapter.md`](IP-014-strangler-migration-adapter.md)
