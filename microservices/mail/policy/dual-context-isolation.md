---
doc_class: PolicySpec
title: Dual-Context Isolation Specification
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-mail + council-privacy + ops-security
deciders: council-architecture, ops-security, axis-mail, council-privacy, ops-legal
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145), ADR-0208, ADR-0215]
related_artifacts:
  - microservices/mail/threat-model.md (Trust Boundary 6; T-I-02, T-L-05, T-L-08)
  - microservices/mail/dpia.md (R-02, R-06)
  - microservices/mail/policy/tenant-scope.cedar
  - microservices/mail/policy/auditor-scope.cedar
review_cadence: quarterly + on every change to dual-context kernel
doc_status: published
---

# Dual-Context Isolation Specification (mail µservice)

## Purpose

Define the load-bearing invariants of mail's dual-context model: **Personal context (B2C; person-pillar)** vs **Professional context (B2B; org-pillar)**. These contexts share one user identity but are isolated at the kernel layer per parallel ADR-0238 (which codifies the 7-sub-product Connect expansion's dual-context isolation rule) and Bominal ADR-0208 (dual-context unified channel hub) + ADR-0215 (retention/legal-hold dual-context).

This document is the authoritative reference for SOC 2 examiners (CC6.1 / CC6.2 / CC6.6), ISO 27001 auditors (A.5.15 / A.8.3 / A.8.12), GDPR Art. 32 reviewers, KR PIPA Art. 23 reviewers, HIPAA §164.502(b) reviewers, and ops-legal asking *"how does mail prevent an employer from reading an employee's personal mail?"*

## Context Model

### Context kind enum (kernel-immutable)

```text
context_kind:
  Professional      # B2B; org-pillar; tenant org owns; org admin sees; legal-hold + eDiscovery applicable
  Personal          # B2C; person-pillar; user owns; org cannot decrypt; legal-hold + eDiscovery FORBIDDEN
```

Set at mailbox creation; **immutable for the mailbox lifetime**. Re-classification requires deletion + re-creation (which the user must initiate) + audit-chain emission documenting the transition.

### Ownership pillar mapping

```text
ownership_pillar:
  Org      ⇔ context_kind = Professional
  Person   ⇔ context_kind = Personal
```

Pillar is derived from context_kind; together they form the (context, pillar) tuple that gates every mail operation.

### Identity model

A single oyatie identity (`user_id`) can own mailboxes in BOTH contexts:
- `user_id=alice` owns `mailbox:alice@acme.com` (Professional, pillar=Org, tenant=acme)
- `user_id=alice` ALSO owns `mailbox:alice@gmail-equivalent.oya` (Personal, pillar=Person, tenant=null)

The user switches persona via the `MailContextSwitched` event; switching does NOT cross data between contexts.

## Invariants

### Invariant DCI-01: Context immutability at the entity layer

Every `Mailbox`, `Thread`, `MailMessage`, `Folder`, `MimeBlob` entity carries an immutable `context_kind` field. The kernel-layer struct uses `#[non_exhaustive]` and exposes no setter; deserialization rejects mutation.

LEAN check `oya-check-mail-context-immutability` greps for any `set_context_kind` or assignment to the field outside of constructors; presence fails the lane.

### Invariant DCI-02: Cross-context routing refused at API boundary

Every public API surface (REST, JMAP, IMAP, SDK, internal port) takes a `ContextKind` parameter or derives it from the principal's session. The `ContextBoundaryGuard` port is called BEFORE any data is read:

```rust
async fn read_mailbox(principal: &Principal, mailbox_id: MailboxId) -> Result<Mailbox> {
    let mailbox = mailbox_store.read(mailbox_id).await?;
    context_boundary_guard.assert(principal.context, mailbox.context_kind)?;  // <- FAIL HERE
    Ok(mailbox)
}
```

Guard refuses when:
- `principal.context == Professional` AND `mailbox.context_kind == Personal` → 403 + audit-emit `mail_cross_context_routing_refused_total{from=professional,to=personal}`
- `principal.context == Personal` AND `mailbox.context_kind == Professional` → same shape

LEAN check `oya-check-dual-context-cross-boundary` greps source for any `mailbox_store.read(...)` call NOT preceded by a `context_boundary_guard.assert(...)` call within the same function (excluding test helpers); presence fails the lane.

### Invariant DCI-03: Personal-context encryption key is user-derived; org cannot decrypt

Professional-context blobs are encrypted under tenant DEK in KMS; org admin (via JIT elevation) can decrypt under audit.

Personal-context blobs are encrypted under a user-derived DEK (when user opts into E2E per Bominal ADR-0208 personal-pillar policy):
- DEK wrapping key derived from the user's passphrase + per-user salt (PBKDF2-HMAC-SHA256, 600,000 iterations).
- Wrapped DEK stored in KMS with access scope = `subject==user.user_id` only.
- Org admin's JIT elevation never includes `subject==user.user_id` for personal-pillar; KMS refuses.
- KMS policy auto-validates: any read of personal-pillar DEK by a non-user principal is refused + audit-emitted.

LEAN check `oya-check-personal-pillar-kms-scope` validates KMS policy + asserts no role can decrypt personal-pillar DEK other than the user's subject.

Open Question 4 (PRD): key-recovery design for personal-pillar — user-held-only vs escrow-with-2-person-rule. Default at M03 launch: **user-held-only** with QR-code paper recovery. registry/placeholder-debt/adr-follow-ups.yaml#personal-mail-key-recovery to finalise.

### Invariant DCI-04: Legal hold + eDiscovery forbidden on personal-pillar

Per Bominal ADR-0215, scoped legal holds and eDiscovery export are forbidden on personal-pillar mailboxes. The `legal-hold` BC's `engage_hold` API refuses any scope that resolves to a personal-pillar mailbox:

```rust
async fn engage_hold(scope: HoldScope, approval: HoldApproval) -> Result<LegalHold> {
    let resolved_mailboxes = resolve_scope(scope).await?;
    for mb in &resolved_mailboxes {
        if mb.context_kind == ContextKind::Personal {
            return Err(LegalHoldError::PersonalPillarForbidden);
        }
    }
    // ... proceed with hold
}
```

Audit-emit on attempt (`mail_personal_pillar_hold_attempt_total`); ops-legal paged.

LEAN check `oya-check-personal-pillar-hold-forbidden` greps the legal-hold engine source for any code path that does NOT include this pillar check; presence fails the lane.

### Invariant DCI-05: Search index per-context-partition

Search index is partitioned by `(tenant_id, context_kind)`:
- Professional context: index per-tenant, queryable by tenant employees + admins via tenant-scoped principal.
- Personal context: index per-user, queryable only by the user (no admin scope can query).

A Professional-context search query for a query term that exists in a user's Personal mailbox returns ZERO matches; the user's Personal mailbox is structurally unreachable from the Professional index.

LEAN check `oya-check-search-index-context-partition` validates per-tenant + per-user Tantivy index layout.

### Invariant DCI-06: Cross-context routing in cross-channel context (mail ↔ messenger ↔ calendar)

A user's Personal mailbox does not appear in Professional context's messenger-suggested-contacts; Professional calendar invites cannot escalate to Personal mailbox. The cross-channel coordinator in `audit-chain` µservice respects pillar per ADR-0215; mail's role is to refuse cross-channel reads that violate pillar:

- `messenger` requesting a mail-to-action-card on a personal-pillar message → refused.
- `calendar` adding a personal-pillar email to a Professional meeting invite → refused at API.

### Invariant DCI-07: Context switch audit-chained

Every `MailContextSwitched` event (user toggles Personal ↔ Professional in UI) emits an audit-chain record. The new context is bound to the session token; the prior context's state is cleared (no cross-context state in same session).

### Invariant DCI-08: Personal-pillar onboarding requires explicit user activation

A tenant onboarding does NOT auto-provision Personal mailboxes for its employees. Personal mailbox creation is initiated by the user (via consumer-side onboarding); tenant onboarding cannot create it on behalf of the user.

This invariant prevents the failure mode where an employer onboards itself + its employees into a tenant and inadvertently captures personal mailboxes under the org pillar.

## Cross-Context Operations (the small allowed set)

There is a tightly bounded set of cross-context operations:

| Operation | Direction | Permission | Audit |
|---|---|---|---|
| User views own mailbox list across contexts | Personal ↔ Professional (same user) | OIDC scope `mail.view_persona_list` (per-user, not per-tenant) | `MailContextListViewed` |
| User initiates persona switch | Either direction | Always allowed for own contexts | `MailContextSwitched` |
| User forwards a Personal mail to their Professional address (or vice versa) | Personal → Professional or reverse | Explicit user action via Compose UI; treated as a new mail at recipient context | `MessageSent` from source context + `MessageReceived` at destination context (two separate audit records) |
| User links a Personal-pillar contact (vcard) into Professional address book | Personal → Professional | Explicit user action; address-book entry only; never the underlying message | `MailContactLinked` |

NO other cross-context operation is allowed. In particular:
- Org admin reading user's Personal mailbox → FORBIDDEN at kernel.
- Search across both contexts simultaneously → FORBIDDEN (search-index partition invariant).
- Backup/restore moving data across contexts → FORBIDDEN (each context's backup is scoped to that pillar).
- Legal hold engaging on personal-pillar → FORBIDDEN per Invariant DCI-04.
- eDiscovery export of personal-pillar → FORBIDDEN per Invariant DCI-04.

## Four-Eyes Legal Hold (Professional Pillar Only)

Per Bominal ADR-0215 contract 1 (sealed_ediscovery_export):

1. **Compliance officer A** requests a scoped hold. Scope is recorded.
2. **Compliance officer B** (distinct OIDC subject) reviews + approves OR rejects.
3. Approval is Ed25519-signed by both A and B.
4. Hold engaged only when both signatures present + signature times within 5min window.
5. Plaintext disclosure (eDiscovery export bundle decryption) requires four-eyes co-signing identical to hold engage.
6. Hold release requires four-eyes co-signing.
7. Pillar check (Invariant DCI-04) enforced at every step.

`legal-hold` BC's state machine refuses any operation lacking these signatures; audit-chain emits at every state transition.

## Failure Modes

### FM-DCI-01: Developer error: API surface missing context_boundary_guard call

**Behaviour:** LEAN check refuses merge; if somehow merged, runtime guard at later layer (Cedar policy or RLS) catches; defence-in-depth.

**Tenant impact:** Caught pre-merge.

**Detection:** `oya-check-dual-context-cross-boundary` LEAN lane + Cedar policy `dual-context-isolation` deny rule.

**Recovery:** Block merge; root-cause in PR review.

### FM-DCI-02: Org admin gains access to personal-pillar KMS DEK

**Behaviour:** KMS policy refuses; emits `mail_personal_pillar_kms_violation_total` + Sev-1 page.

**Tenant impact:** None (KMS refuses); investigation determines how the request was made.

**Detection:** KMS audit log + LEAN check.

**Recovery:** Revoke whatever credentials made the request; forensic trace; incident response.

### FM-DCI-03: Migration adapter incorrectly tags imported personal mail as professional

**Behaviour:** Migration adapter requires explicit `context_kind` per imported message; defaults to Professional only when migration is initiated by tenant admin (not user); per-user migration defaults to Personal.

**Tenant impact:** Caught at migration test; if not caught, audit-chain reveals; user can re-tag with manual workflow.

**Detection:** `oya-check-migration-context-tagging` LEAN lane.

**Recovery:** Re-import with correct context.

### FM-DCI-04: Legal hold engaged on personal-pillar (developer bypass attempt)

**Behaviour:** legal-hold engine refuses; audit-emit + page.

**Tenant impact:** None.

**Detection:** Audit-chain + `mail_personal_pillar_hold_attempt_total` metric.

**Recovery:** Investigation; harden if a code path attempted to bypass.

### FM-DCI-05: Cross-pillar mailbox listing exposed in unified UI

**Behaviour:** Persona switcher UI uses per-context API endpoints; mixing is via explicit user action only.

**Tenant impact:** Caught in integration test.

**Detection:** E2E test `tests/e2e/cross-context-refusal.sh`.

**Recovery:** UI patch.

## Audit Trail

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| Cross-context routing refused | API surface (any BC) | `principal_id, source_context, target_context, mailbox_id, timestamp` | ≥ 1y |
| Personal-pillar hold attempt refused | legal-hold | `principal_id, scope, attempted_at` | ≥ 1y |
| Personal-pillar KMS access refused | KMS audit forwarder | `principal_id, attempted_mailbox_id, timestamp` | ≥ 1y |
| Context switch | imap-frontend / REST | `user_id, from_context, to_context, session_id, timestamp` | ≥ 1y |
| Migration context tagging | migration worker | `import_batch_id, message_count, context_kind_assigned, reason` | ≥ 1y |
| Cross-context forward (Personal → Professional or reverse) | outbound-smtp | `from_context, to_context, source_message_id, new_message_id, principal_id` | ≥ 1y |

Audit log retention per `policy/data-residency.md` per-pack overlays (HIPAA 6y; KR-FSS 5y; default 1y).

## Per-Pack Overlay

### pack-kr (KR PIPA + ISMS-P)

- KR PIPA Art. 28 + 29 mapping: dual-context invariant satisfies PIPA's "collection within necessary scope" + "encrypted storage of sensitive PII".
- KR PIPA Art. 22-2 (sensitive PII special protections): personal-pillar mailbox treated as sensitive; KMS-in-KR; 2-person rule for any access escalation (which cannot include personal-pillar anyway).
- Audit log retention ≥ 1y per PIPA Enforcement Decree Art. 30; 5y for KR-FSS tenant.

### pack-us-healthcare (HIPAA)

- §164.502(b) Minimum Necessary: dual-context invariant directly satisfies — personal-pillar PHI (if any) is inaccessible to professional context per Invariant DCI-04.
- §164.504(e) BAA: BAA covers professional-pillar only; personal-pillar PHI is the user's own responsibility (oyatie acts as cloud provider, not BA, for personal-pillar).
- §164.312(a)(1) Access Control: Cedar + ContextBoundaryGuard implement Unique User Identification + Automatic Logoff + Encryption-and-Decryption.

### pack-eu (GDPR + ePrivacy)

- Art. 25 by design + default: pillar invariant is the load-bearing privacy-by-design control.
- Art. 32(1)(a) pseudonymisation + encryption: per-tenant DEK for org pillar; user-derived DEK for personal pillar.
- ePrivacy Directive Art. 5 (e-mail confidentiality): satisfied for personal-pillar (org cannot read); professional-pillar is processor activity per DPA.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `regional-packs/<pack>/mail-dual-context-overlay.md` maps the local PII law's confidentiality + minimum-necessary requirements to DCI-01..DCI-08.

## Verification

- `cargo run -p oya-dev-cli -- gate validate dual-context-cross-boundary --microservice mail` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate mail-context-immutability --microservice mail` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate personal-pillar-kms-scope` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate personal-pillar-hold-forbidden` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate search-index-context-partition --microservice mail` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate migration-context-tagging --microservice mail` — exit 0.
- Annual pen-test against pillar boundary: documented in `runbooks/pillar-boundary-pentest.md`.
- Quarterly chaos drill: induce cross-context routing attempt + personal-pillar hold attempt; verify rejection + alerting.

## References

- ADR-0028 (Bominal): audit-chain.
- ADR-0117: residency.
- ADR-0135: Connect dissolution; dual-context invariant.
- ADR-0139: SLO gate.
- ADR-0131: per-microservice flat layout.
- ADR-0132: no-suite forward policy.
- ADR-0140: Cedar policy enforcement.
- Bominal ADR-0208: dual-context unified channel hub.
- Bominal ADR-0215: retention/legal-hold dual-context.
- `microservices/mail/threat-model.md` Trust Boundary 6; T-I-02, T-L-05, T-L-08.
- `microservices/mail/dpia.md` R-02, R-06.
- `microservices/mail/policy/tenant-scope.cedar`, `auditor-scope.cedar`.
- KR PIPA Arts. 23, 28, 29, 22-2.
- HIPAA §164.502(b), §164.504(e), §164.312(a)(1).
- GDPR Arts. 25, 32; ePrivacy Directive Art. 5.
