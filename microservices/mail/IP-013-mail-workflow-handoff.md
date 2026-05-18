---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-013-mail-workflow-handoff
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + axis-workflow + council-privacy
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-governance-workflow-event-registry, oya-governance-data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: mail-to-Workflow handoff (consent/policy-basis check; audit-chain)

## Intent

Implement the mail-to-Workflow handoff path per PRD FR-09 + capabilities/T1-assist.yaml T1-mail-handoff-to-workflow. Every handoff:
- Forbidden on Personal-pillar (Invariant DCI-06).
- Requires explicit user action OR tenant-declared policy basis.
- Emits `MailWorkflowHandoffCreated` audit-chain event linking source message + extracted payload digest + consent evidence + workflow item id.
- Receives `WorkflowHandoffCommitted` from workflow-engine and marks source message with handoff label.

## ChangeSet boundary

Sub-crates within `mailbox-store` BC: domain (extraction + redaction) + usecase (orchestrator) + rest (handler). Cross-µservice Workflow event integration verified at lane time.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-mailbox-store-domain/src/workflow_handoff.rs` | create | extraction logic + consent verification |
| `microservices/mail/src/crates/oya-mail-mailbox-store-usecase/src/workflow_handoff.rs` | create | orchestrator |
| `microservices/mail/src/crates/oya-mail-mailbox-store-rest/src/handoff_handler.rs` | create | POST /v1/assist/workflow-handoff |
| `microservices/mail/tests/e2e/workflow-handoff.sh` | create | happy + refused paths |

## Code Shape

```rust
// usecase/src/workflow_handoff.rs
pub async fn create_handoff(req: HandoffRequest, principal: &Principal, p: &Ports)
    -> Result<HandoffReceipt, HandoffError>
{
    let mb = p.mailbox.read(req.mailbox_id).await?;
    p.context_guard.assert(principal.context, mb.context_kind)?;
    if mb.context_kind == ContextKind::Personal {
        return Err(HandoffError::PersonalPillarForbidden);  // DCI-06
    }
    let msg = p.mailbox.read_message(req.mailbox_id, &req.message_id).await?;
    let consent = match req.consent_basis {
        ConsentBasis::UserExplicit { session_id } => p.consent.verify_user_explicit(session_id, principal).await?,
        ConsentBasis::TenantPolicy { basis_ref }  => p.consent.verify_tenant_policy(basis_ref, principal.tenant_id).await?,
    };
    let extracted = p.extractor.extract(&msg).await?;       // T1 capability: AI-assisted; user reviews
    let item = p.workflow_engine.create_item(extracted, &consent).await?;
    let seal = p.audit_chain.emit_seal("MailWorkflowHandoffCreated", &item).await?;
    p.events.emit_mail_workflow_handoff_created(MailWorkflowHandoffCreated {
        source_message_id: msg.id, workflow_item_id: item.id,
        consent_basis: consent, extracted_payload_digest: extracted.digest(),
        signature: seal,
    }).await?;
    Ok(HandoffReceipt { workflow_item_id: item.id })
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-mail-mailbox-store-usecase --test workflow_handoff
cargo run -p oya-dev-cli -- gate validate workflow-event-registry --microservice mail
bash microservices/mail/tests/e2e/workflow-handoff.sh
```

## Test Plan

- Happy: explicit user action + Professional context → handoff created, audit emitted.
- Refused: Personal context → `PersonalPillarForbidden`.
- Refused: no consent basis → `ConsentMissing`.
- Refused: tenant policy basis_ref invalid → `PolicyBasisInvalid`.
- Audit chain: `MailWorkflowHandoffCreated` event verified by audit-chain.
- Cross-µservice: workflow-engine returns `WorkflowHandoffCommitted` → mail labels source message.

## Halt Conditions

- Handoff allowed without consent → fail (regulatory critical).
- Personal-pillar handoff path → fail.

## Next IP

[`IP-014-hg-mail-authority-cohesion.md`](IP-014-hg-mail-authority-cohesion.md)

## References

- PRD FR-09; capabilities/T1-assist.yaml T1-mail-handoff-to-workflow
- `policy/dual-context-isolation.md` Invariant DCI-06
- ADR-0008 (data-use boundary)
- ADR-0140 (Cedar policy enforcement)
- GDPR Art. 6 (lawful basis) + Art. 7 (consent)
- KR PIPA Art. 15 (consent for collection)
