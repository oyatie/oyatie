---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P04-connect-pro-mail
status: Proposed
acceptance_lanes: []
entry_gate: 'M02b-substrate-schema-foundation complete (oya-ontology-entity-kernel
  + oya-workflow-engine-kernel ship);

  oya-kms-kernel ships (tenant DEK envelope encryption per ADR-0111);

  EmployeeHired Workflow event registered (from M03/P01-hr);

  Cedar policy engine bootstrapped.

  '
exit_gate: "All IP acceptance gates green; mail send/receive round-trip test green;\
  \ DKIM signature valid;\ndual-context boundary test passes (Professional unreachable\
  \ from Personal);\nlegal hold blocks deletion; eDiscovery export \u22645 min for\
  \ 100k messages;\n`oya gate validate lean-a2 --ms connect` exits 0;\n`oya gate validate\
  \ audit-chain --ms connect` exits 0;\nk6 smoke mail send p99 \u22642s; messenger\
  \ p99 \u2264200ms;\ngrit done on all P04 symbols; ICM phase-handoff row emitted.\n"
depends_on:
- milestone: M02
  phase: P22-substrate-ready
  reason: "KMS \xB5service (tenant DEK), Ontology entity types, Cedar policy engine,\
    \ Workflow event bus all required before Connect can store encrypted mail or enforce\
    \ dual-context boundary."
- milestone: M03
  phase: P01-hr
  reason: EmployeeHired Workflow event triggers Connect account provisioning; requires
    HR to ship first.
parallel_wave: 2
owner_team: council-connect
purpose: "Delivers Connect Professional Mail: hosted corporate email for tenant domains with SMTP ingest, IMAP access, JMAP protocol support, SPF/DKIM/DMARC enforcement, and full compliance stack (legal hold, eDiscovery export in PST/MBOX."
---
# P04-connect-pro-mail: Connect Professional Mail — SMTP/IMAP, tenant DEK encryption, legal hold, eDiscovery, retention

## Purpose

Delivers Connect Professional Mail: hosted corporate email for tenant domains
with SMTP ingest, IMAP access, JMAP protocol support, SPF/DKIM/DMARC enforcement,
and full compliance stack (legal hold, eDiscovery export in PST/MBOX, retention
policies, immutable audit log). All mail content encrypted at rest under tenant
DEK (AES-256-GCM, KMS-wrapped per ADR-0111). Dual-context boundary (ADR-0208 /
ADR-0215) enforced: org-admin APIs return 403 for any Personal-context resource.

Scaffolds the Connect µservice shared binary (`oya-connect-app`) that P05-connect-pro-messenger
extends; both contexts share the same deployable but have physically isolated
Postgres schemas (`connect_pro` / `connect_personal`).

---

## Scope

### In-scope

| µservice | Bounded Contexts | Crate family (BNF v4.1) |
|---|---|---|
| `connect` | `mail` | `oya-connect-mail-{kernel,domain,application,adapter,rest,grpc}` |
| `connect` | `legal-hold` | `oya-connect-legal-hold-{kernel,domain,application,adapter,rest}` |
| `connect` | `provisioning` | `oya-connect-provisioning-{domain,application}` |
| `connect` | `workflow-handoff` | `oya-connect-workflow-handoff-{domain,application}` |
| `connect` | `personal` | `oya-connect-personal-{domain,application,infrastructure}` (scaffolded; NOT GA until post-M03 crypto audit) |
| `connect` | `app` | `oya-connect-app` |

Naming justifications:

```
NAME: oya-connect-mail-kernel
JUSTIFICATION:
- microservice = connect: Connect µservice (dual-context communication); registered; ADR-0056 v4.1
- bc-tokens = mail: connect has multiple BCs (mail/legal-hold/provisioning/workflow-handoff/personal/messenger); mail BC owns Mailbox + Message + Thread entities + MailboxStore/LegalHoldStore port-traits; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure MailboxId/MessageId value types + MailboxStore port declaration; zero logic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-mail-domain
JUSTIFICATION:
- microservice = connect; bc-tokens = mail; layer = domain: Mailbox aggregate + Message entity + Thread entity + DKIM signature validation rules + dual-context_kind enforcement invariant; calls through MailboxStore; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-mail-adapter
JUSTIFICATION:
- microservice = connect; bc-tokens = mail; layer = adapter: PostgresMailboxStore (implements MailboxStore), OciObjectStorageMessageBody (message body to object storage), SmtpIngestAdapter, ImapAdapter; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-mail-grpc
JUSTIFICATION:
- microservice = connect; bc-tokens = mail; layer = grpc: tonic gRPC service for internal mail delivery bus; Connect-internal only; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-legal-hold-kernel
JUSTIFICATION:
- microservice = connect; bc-tokens = legal-hold: legal-hold BC owns LegalHold entity + RetentionPolicy + eDiscovery export port + LegalHoldStore port-trait; ADR-0215 contracts; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure HoldId value types + LegalHoldStore/ExportPort port declarations; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-provisioning-domain
JUSTIFICATION:
- microservice = connect; bc-tokens = provisioning: provisioning BC owns ConnectAccount entity + lifecycle rules (hire → provision, terminate → suspend + retention hold); domain layer only — application wires Workflow event subscription; no infrastructure; ADR-0056 v4.1 BC-optionality
- layer = domain: ConnectAccount aggregate + provisioning invariants; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-workflow-handoff-domain
JUSTIFICATION:
- microservice = connect; bc-tokens = workflow-handoff: workflow-handoff BC owns ApprovalCard entity + action card delivery logic; domain layer only; ADR-0056 v4.1 BC-optionality
- layer = domain: ApprovalCard entity + card delivery rules; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-personal-domain
JUSTIFICATION:
- microservice = connect; bc-tokens = personal: personal BC scaffolded but NOT GA; PersonalConversation entity + person-pillar boundary invariants; deferred until post-M03 crypto audit; ADR-0056 v4.1 BC-optionality
- layer = domain: PersonalConversation aggregate (stub; no adapter wired); ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-app
JUSTIFICATION:
- microservice = connect; bc-tokens: OMITTED — composition-root; ADR-0056 §"BC optionality"
- layer = app: main.rs + DI wiring; hosts both Professional context (M03) and Personal context scaffold (post-M03); ADR-0056 §"Layer semantics"
- exemptions: none
```

### Out-of-scope

- Connect Personal GA — deferred post-M03 pending crypto audit per PRD-connect open question #2.
- S/MIME / E2E encryption for Professional mail — post-M03 per ADR-0215 §"Future Direction".
- eDiscovery search UX (advanced filters, relevance ranking) — deferred per ADR-0215 §"Future Direction #1".
- watchOS/Wear OS mail clients — post-M03 per ADR-0210.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full Connect Professional Mail: SMTP/IMAP/JMAP, tenant DEK encryption, SPF/DKIM/DMARC, legal hold, eDiscovery export (PST/MBOX), retention policies, dual-context Cedar policies, ConnectAccount provisioning from EmployeeHired, Workflow approval action cards, load tests | pending | council-connect |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features                                        # exit 0
cargo build -p oya-connect-app --all-features                                 # exit 0
cargo clippy -p oya-connect-mail-domain -p oya-connect-legal-hold-domain -- -D warnings  # exit 0
cargo nextest run --test test_mail_send_receive                               # exit 0; DKIM signature valid
cargo nextest run --test dual_context_isolation                               # exit 0; Professional data unreachable from Personal
cargo nextest run --test test_legal_hold_export_100k                         # exit 0; ≤5 min for 100k messages
cargo nextest run --test test_provisioning_workflow                           # exit 0; EmployeeHired → ConnectAccount
cargo nextest run -p oya-connect-legal-hold-domain                           # exit 0; retention + deletion enforced
cargo deny check                                                              # exit 0
```

### Fitness lane gates

```bash
oya gate validate lean-a2 --ms connect            # no imports from hr/payroll/accounting
oya gate validate lean-a1 --ms connect            # layer ordering
oya gate validate port-location --ms connect      # port traits in kernel
oya gate validate shardability --ms connect       # tenant_id partition key
oya gate validate audit-chain --ms connect        # Ed25519 seal per (tenant_id, period)
oya gate validate jurisdiction-overlay --ms connect  # jurisdiction_code=KR; 전자문서법 retention
```

### Cedar policy gate

```bash
oya gate validate cedar-policy --ms connect  # dual-context forbid rules from ADR-0215; org-admin 403 on person-pillar
```

### Performance gates

```bash
# k6: mail send p99 ≤2s
k6 run tests/load/smoke-connect-mail-send.js --env BASE_URL=http://localhost:8084
# k6: messenger message p99 ≤200ms at 5k concurrent users
k6 run tests/load/smoke-connect-messenger.js --env BASE_URL=http://localhost:8084
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-connect-mail-kernel` | `kernel` | Yes — `MailboxStore`, `LegalHoldStore` | N/A |
| `oya-connect-mail-domain` | `domain` | N/A | N/A |
| `oya-connect-mail-application` | `application` | N/A | N/A |
| `oya-connect-mail-adapter` | `adapter` | N/A | Yes — `PostgresMailboxStore`, `OciObjectStorageMessageBody`, `SmtpIngestAdapter` |
| `oya-connect-mail-rest` | `rest` | N/A | No direct adapter import |
| `oya-connect-mail-grpc` | `grpc` | N/A | No direct adapter import |
| `oya-connect-legal-hold-kernel` | `kernel` | Yes — `LegalHoldStore`, `ExportPort` | N/A |
| `oya-connect-legal-hold-adapter` | `adapter` | N/A | Yes — `PostgresLegalHoldStore`, `PstExportAdapter`, `MboxExportAdapter` |
| `oya-connect-provisioning-domain` | `domain` | N/A | N/A |
| `oya-connect-provisioning-application` | `application` | N/A | N/A |
| `oya-connect-workflow-handoff-domain` | `domain` | N/A | N/A |
| `oya-connect-personal-domain` | `domain` | N/A (stub) | N/A |
| `oya-connect-app` | `app` | N/A | Unrestricted inward |

Cross-product: Connect NEVER imports `oya-hr-*`, `oya-payroll-*`, `oya-accounting-*`.
Employee reads via `oya-ontology-entity-kernel::ObjectStore` port only.

### Dual-context Cedar policies (ADR-0208 + ADR-0215)

```cedar
// Reject cross-pillar queries
forbid(principal, action in [Action::"Read", Action::"ExportForEDiscovery", Action::"InitiateLegalHold"], resource)
when {
  principal.ownership_pillar != resource.ownership_pillar
};

// Org-admin forbidden from person-pillar
forbid(principal is Admin, action, resource)
when { resource.ownership_pillar == Person };
```

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `mail` | `connect` | pending |
| `legal-hold` | `connect` | pending |
| `provisioning` | `connect` | pending |
| `workflow-handoff` | `connect` | pending |
| `personal` | `connect` | pending (scaffold only) |

---

## Grit Claim Symbols

```
crates/oya-connect-mail-kernel/src/ports.rs::MailboxStore
crates/oya-connect-mail-kernel/src/ports.rs::LegalHoldStore
crates/oya-connect-mail-domain/src/mailbox.rs::Mailbox
crates/oya-connect-legal-hold-domain/src/legal_hold.rs::LegalHold
crates/oya-connect-legal-hold-domain/src/retention_policy.rs::RetentionPolicy
crates/oya-connect-provisioning-domain/src/connect_account.rs::ConnectAccount
contracts/connect.openapi.yaml::sendMail
contracts/connect.openapi.yaml::initiateLegalHold
docs/standards/bounded-contexts.md::connect.mail
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P04-connect-pro-mail started; depends on M02 substrate + M03/P01-hr EmployeeHired; scope: Professional mail (SMTP/IMAP/JMAP), tenant DEK encryption, legal hold (ADR-0215), eDiscovery, retention, dual-context boundary (ADR-0208)" \
  -i high \
  -k "M03,P04,phase-start,connect,mail"

icm store \
  -t context-oyatie \
  -c "Phase P04-connect-pro-mail complete; Connect Professional Mail shipped; SMTP/IMAP/JMAP; tenant DEK (ADR-0111); legal hold + eDiscovery (ADR-0215); dual-context Cedar policies (ADR-0208); ConnectAccount provisioning from EmployeeHired; next: P05-connect-pro-messenger" \
  -i high \
  -k "M03,P04,phase-complete,connect,mail"
```

---

## References

- PRD: `docs/prds/connect.md`
- Bominal ADRs inherited: ADR-0208 (dual-context), ADR-0215 (retention/legal hold), ADR-0210 (M3 mail launch), ADR-0132 (data pillars), ADR-0111 (tenant DEK), ADR-0028 (audit chain)
- oyatie ADRs: ADR-0056 (BNF v4.1)
