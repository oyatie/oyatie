---
doc_class: Reference
shape: review-packet
length_cap: 500
authority_tier: 3
status: Draft-Pending-Independent-Review
date: 2026-07-01
purpose: |
  Security, privacy, and customer-safe documentation review packet for the
  Trust Center / Compliance Evidence Portal first implementation slice. This
  packet records launch-claim boundaries, threat model, data-class/redaction
  matrix, tenant-isolation checklist, auditor-room/export procedure, and source
  implementation evidence for Kanban task t_e615a913.
canonical_authority: /specs/trust-center-compliance-evidence-portal.json
authority_chain_declaration: |
  docs/AGENTS.md + /specs/root-hub-pointers.json >
  /specs/trust-center-compliance-evidence-portal.json > this review packet.
companion_docs:
  - docs/AGENTS.md
  - docs/standards/security-review.md
  - docs/standards/privacy-review.md
  - docs/standards/data-class.md
  - docs/templates/threat-model-template.md
  - specs/hyperscaler-production-readiness-claim-contract.json
kanban_task: t_e615a913
doc_status: draft-pending-review
---

# Trust Center Security, Privacy, and Docs Review Packet - 2026-07-01

## 0. Review control

This packet covers TRUSTCENTER-DOCS-SEC-001 (`t_e615a913`). It is a review and
customer-safe documentation artifact, not a launch approval, certification, or
runtime implementation.

Does NOT cover:

- API, UI, ingestion, storage, export-package, or live collector implementation.
- SOC 2, ISO, PCI, CSAP, ISMS-P, KISA, KMCC, KCC, FedRAMP, or other external
  certification claims.
- Production-ready, hyperscaler-grade, or tenant-facing launch approval.
- Raw scanner output, exploit payloads, secrets, PII, private incident notes,
  tenant identifiers from other tenants, or unredacted operator findings.

Launch-claim disposition: NOT APPROVED. The first slice has useful API,
ingestion, and UI evidence, but tenant-facing launch claims remain blocked until
all open review/fix and verification gates in §8 are complete.

## 1. Source implementation and review evidence

| Surface | Card / file evidence | Review status | Launch effect |
|---|---|---|---|
| Planning contract | `t_157e833c`; `specs/trust-center-compliance-evidence-portal.json` | Approved by `t_8dd778a9` as planning/product-surface contract. | Supports spec-ready planning only. |
| API/read model | `t_3af64a26`; `oya/trust/crates/oya-trust-center-api/*` | `t_27602eee` APPROVE after fixing cross-tenant export-download audit emission. | Good first-slice API evidence; cursor hardening gap remains in §8. |
| Ingestion/publishability | `t_3a144f8c`; `oya/trust/crates/oya-trust-center-ingest/*` | `t_2b194edb` APPROVE after fixing missing tenant assertion and policy-N/A redaction. | Good first-slice ingestion evidence. |
| UX first slice | `t_c9fba41f`; app shell `app.rs` and `app.css` | Review/fix `t_03bcb795` is ready, not complete at packet time. | Launch remains blocked. |
| PR / protected CI | No PR recorded in parent handoffs. | No `oya-ci-required` current-head evidence in this packet. | Launch remains blocked. |
| Browser evidence | UX worker ran SSR-focused Rust tests; no live browser E2E attached. | Pending independent UX/browser review. | Launch remains blocked. |

Verification cited by parent cards:

- API: `cargo fmt -p oya-trust-center-api --check`; `cargo test --locked -p
  oya-trust-center-api`; static scan `static_scan_findings=0`.
- Ingestion: `cargo fmt -p oya-trust-center-api -p oya-trust-center-ingest
  --check`; `cargo test -p oya-trust-center-api -p oya-trust-center-ingest`;
  `cargo clippy -p oya-trust-center-api -p oya-trust-center-ingest --all-targets
  -- -D warnings`; actionable static scan `0`.
- UX: `rustfmt --check` on `app.rs`; scoped `git diff --check`; focused
  `trust_center_slice` tests; full app-shell SSR lib tests. UX review is still
  pending in `t_03bcb795`.

## 2. Customer-safe claim language

Every customer-visible Trust Center statement MUST render a claim tier and an
evidence state. A missing/stale/unreviewed artifact is not green and is not
silently hidden.

| Claim tier | Customer-safe wording | Forbidden wording |
|---|---|---|
| `target_non_claim` | Planned target; evidence gap is visible. | Ready, certified, enforced, complete. |
| `spec_ready` | Contracted or fixture-backed; implementation can be tested. | Enforced in production or live tenant-ready. |
| `mechanically_enforced` | Branch-protected required context and RED/GREEN gate prove the invariant. | Local-only check or advisory script as authority. |
| `production_ready` | Per-service production evidence exists: SLO, rollout, rollback, isolation, observability, and operations. | Program-wide adjective or one-time test pass. |
| `hyperscaler_grade` | Sustained multi-cell/region, tenant isolation, release, SLO, capacity, and security evidence exists. | Synonym for good architecture. |
| `externally_certified` | Approved external attestation exists, applies to the tenant/pack, and legal permits display. | Fake SOC 2/ISO/CSAP/ISMS-P certification language. |

Minimum customer-safe banner:

> Trust Center evidence is scoped to your tenant and current evidence state.
> Target or spec-ready entries are not certifications. Missing, stale, pending,
> or review-blocked evidence is shown as such and does not count as a pass.

## 3. Threat model

### 3.1 Subject

The Trust Center / Compliance Evidence Portal exposes tenant-scoped evidence
summaries, control freshness, SBOM/VEX posture, compliance-pack views,
reviewer-room grants, export-request stubs, access audit, and public status
links. Source evidence comes from security-validation, vulnerability/SBOM/VEX,
compliance-pack, SLO/DR/status/incident, quality-kit, release, and audit-chain
families.

### 3.2 Logical boundaries

```text
public visitor -> public status page boundary -> public status summary only

authenticated tenant admin/reviewer/operator
  -> trusted auth boundary + Cedar/PDP decision
  -> Trust Center API/read model
  -> tenant-scoped evidence records and audit-chain refs
  -> redacted customer-safe UI/API/export request

Oyatie operator
  -> publishability queue and redaction policy
  -> append-only publishability decision records
  -> access/export audit records
```

### 3.3 STRIDE and abuse-case table

| Abuse case | Required controls | Evidence observed | Residual / gate |
|---|---|---|---|
| Cross-tenant access | Trusted-boundary tenant scope; payload tenant is assertion only; detail/export/audit deny mismatches. | API tests cover every spec endpoint payload tenant mismatch and detail cross-tenant denial. | Keep requiring API review on any storage/list/cache adapter. |
| Reviewer grants | Purpose, scope, actor, expiry, revocation, audit event; reviewers cannot self-extend or administer. | API has grant audit events; UI fixture has create/revoke expiring reviewer controls. | UX review `t_03bcb795` pending; live grant persistence absent. |
| Stale cursors | Cursor must be tenant, actor, filter, and expiry bound; replay outside scope fails closed. | First API slice binds cursor prefix to tenant only. | BLOCKER before launch; create hardening work for actor/filter/expiry binding. |
| Export package leakage | Purpose, framework, time window, approval, manifest, revocation, and audit-chain refs. | API export is a stub with `manifest_ref: None`; export/download audit is tenant-checked after review fix. | No real package assembly; launch claim blocked until package manifest controls exist. |
| Source-evidence spoofing | Source records need trusted producer identity, source refs, audit refs, minimum fields, and tenant assertion checks. | Ingest validates minimum fields and tenant assertions across six source families. | Future live collectors need signed/fetched receipts and producer identity. |
| Redaction bypass | Operator-only raw output, PII, secrets, exploit detail, private incident detail, and cross-tenant ids never reach customer payloads. | Ingest redaction tests remove synthetic secret/PII/exploit/raw/cross-tenant markers; API denies operator-only detail. | UX review still needs visual/customer-safe check. |
| Public/auth boundary | Public status remains public-only; authenticated evidence room never depends on public status for private data. | Spec and UI separate public status affordance from evidence room. | No live status integration in first slice. |
| Publishability tampering | Publishability decisions append, cite source/audit refs, principal, reason, and expiry where temporary. | Ingest emits append-only decision records; API operator-only publishability mutation appends audit. | Future storage must preserve append-only semantics. |
| Claim inflation | Every posture row carries claim tier and evidence state. | Spec/API/UI model claim tiers including `target_non_claim` and blocked states. | Docs/review gate must reject external-certification or production words without evidence. |

## 4. Privacy and data-class / redaction matrix

| Data class | Allowed surfaces | Retention | Redaction | Export controls |
|---|---|---|---|---|
| `PUBLIC_STATUS` | Public status page/link and public incident summary only. | Status-system policy; do not mix with private Trust Center retention. | No tenant names, raw logs, private incident detail, reviewer identities, or cell occupancy. | Public links only; never a regulated export package by itself. |
| `TENANT_TRUST_EVIDENCE` | Authenticated tenant admin, approved reviewer room, operator scoped to tenant. | Access audit at least 400 days; evidence per source/pack policy. | Collapse paths, scanner payloads, employee identity, secrets, exploit detail, and other-tenant refs to evidence refs and roles. | Tenant admin approval; bounded purpose/scope; audit-chain refs. |
| `REGULATED_EXPORT_EVIDENCE` | Explicit export grant or auditor room only. | Compliance-pack retention floor and legal hold policy. | Apply residency, consent, regulator, and pack-specific redaction before export. | Purpose, actor, time window, framework, manifest, approval, expiry, revocation, and audit-chain refs required. |
| `OPERATOR_SECURITY_INTERNAL` | Operator-only queues and internal review surfaces. | Security program / source-system policy; not customer retention. | Never directly customer-visible; only policy-approved derived summaries. | Not exportable except through separately approved legal/security disclosure. |

Privacy review checklist:

- Data classes are named in the spec and API enum.
- Ingest marks security-validation and SBOM/VEX summaries as tenant trust
  evidence, compliance-pack views as regulated export evidence, and raw operator
  detail as redacted/not exposed.
- DSR/export impact is not complete for real storage because this is an in-memory
  first slice; live storage/export work must add DSR and retention adapters.
- Privacy council review remains required before customer-visible launch.

## 5. Tenant-isolation and redaction checklist

| Check | Status | Evidence / note |
|---|---|---|
| Server-side tenant scope is source of authority. | PASS for first API slice. | `validate_api_request` validates boundary, principal, authorization, and tenant assertions. |
| Payload `tenant_id` cannot become authority. | PASS for API and ingest. | API endpoint-wide tests and ingest source assertion tests. |
| Cross-tenant evidence detail is denied. | PASS for first API slice. | API test denies `ev_other_current`. |
| Operator-only detail is not customer visible. | PASS for API/ingest. | API denies operator-only detail; ingest emits `raw_operator_payload_exposed=false`. |
| Stale or missing evidence fails closed. | PASS for API/ingest first slice. | API rejects stale detail; ingest maps stale/missing/parser/unknown/expired to blocked states. |
| Cursor replay is stale-scope safe. | FAIL/PENDING. | Current cursor binds only to tenant prefix and offset. Actor, filters, and expiry are not encoded. |
| Export download audit is tenant-bound. | PASS after review fix. | `t_27602eee` enforced tenant match before download audit event. |
| UX states are non-green/non-color-only. | PENDING independent review. | UI has text markers; review `t_03bcb795` not complete. |
| No raw secrets/PII/exploit markers in tests/docs. | PASS for scoped static scans. | API/ingest review comments record static-scan findings of 0. |
| Generated JSON untouched. | PASS for this packet. | This packet only adds a Markdown audit/review artifact. |

## 6. Auditor-room and export operating procedure

This is the required operating procedure for any future auditor-room/export
implementation. The current first slice has an export request stub only.

1. Intake
   - Actor is tenant admin or explicitly authorized operator.
   - Purpose is specific: procurement review, audit fieldwork, regulator request,
     DSR/export support, or named compliance-pack review.
   - Framework and tenant/compliance-pack scope are selected before assembly.

2. Scope and time window
   - Time window has trusted start and end timestamps.
   - Evidence IDs are tenant-scoped and source-record backed.
   - Public status refs are included as links, not private-status authority.

3. Approval
   - Regulated export evidence requires operator/security/privacy approval until
     policy permits self-service for a narrower class.
   - Approval record includes actor, reviewer, decision id, reason, expiry, and
     audit-chain reference.

4. Manifest
   - Export package has a manifest with evidence refs, source system, data class,
     redaction policy id, retention/expiry, audit_event_ref, and package digest.
   - Screenshots are not sole proof.

5. Delivery
   - Auditor room is time-limited and revocable.
   - Reviewer cannot self-extend access.
   - Download emits `trust_center.evidence_export_downloaded` with tenant match.

6. Revocation and expiry
   - Expired grants fail closed.
   - Revocation emits `trust_center.access_grant_revoked`.
   - Export links no longer resolve after expiry unless a legal hold workflow
     explicitly preserves package metadata.

7. Audit chain
   - Emit requested, approved, downloaded, revoked, redaction-applied, and
     publishability-changed events.
   - Retain access audit at least 400 days or the pack floor, whichever is
     stricter.

## 7. Customer-facing documentation copy

Use this copy for first-slice docs and UI until later review supersedes it:

> Oyatie Trust Center shows tenant-scoped evidence status for security controls,
> SBOM/VEX posture, compliance-pack coverage, status/SLO/DR posture, quality-kit
> evidence, release evidence, and audit-chain references. Entries labeled
> `target_non_claim`, `spec_ready`, stale, missing, pending review, or blocked are
> not certifications and do not indicate production readiness. External
> certification labels appear only when an approved external attestation applies
> to your tenant or compliance pack and legal/compliance permits display.

Use this export/auditor-room copy:

> Evidence rooms and export packages are bounded by tenant, purpose, framework,
> time window, actor, expiry, approval, and manifest. They include customer-safe
> summaries and provenance references, not raw scanner output, secrets, exploit
> payloads, private incident notes, or evidence from other tenants.

Forbidden customer-facing wording unless separately proven:

- "Certified", "SOC 2 compliant", "ISO 27001 certified", "CSAP certified",
  "ISMS-P certified", "KISA approved", or equivalent external assurance.
- "Production ready", "hyperscaler grade", "fully enforced", or "complete".
- "All controls passed" when any source is stale, missing, pending review,
  policy-N/A, or target/spec-only.
- "Export package ready" when manifest, approval, expiry, revocation, and
  audit-chain references are absent.

## 8. Review outcomes and launch blockers

Security/privacy/API outcome:

- API review `t_27602eee`: APPROVE after fixing cross-tenant export-download
  audit emission.
- Ingestion review `t_2b194edb`: APPROVE after fixing security-validation
  tenant assertion and sensitive policy-N/A redaction.
- Packet-level finding: API pagination cursor is tenant-bound but not yet
  actor/filter/expiry-bound. That is a launch blocker for stale cursor replay.

Product/UX/docs outcome:

- UI implementation `t_c9fba41f` exists and has focused SSR tests.
- Independent UI review/fix `t_03bcb795` is ready and not complete at packet
  time.
- This packet supplies customer-safe docs language, but independent
  security/privacy/docs review is still required before launch claims.

Launch remains blocked until all of these are true:

1. UI review/fix `t_03bcb795` completes with product/UX, accessibility,
   security/compliance, and regression approval.
2. API cursor hardening `t_41610e0a` binds pagination cursor replay to tenant,
   actor, filters, and expiry, with regression tests.
3. Independent review/fix `t_95d311cb` of this packet approves the threat model, data-class
   matrix, export runbook, and claim-boundary copy.
4. A PR exists with current-head `oya-ci-required` green and scoped local
   verification for API, ingestion, UI, docs, and any generated artifacts.
5. Browser/API user-story evidence covers tenant admin evidence read, reviewer
   room access, SBOM/VEX view, export request, and access-audit review.
6. Release-governance/release-note, rollout, rollback, observability, and
   product-completion packet evidence exist for any tenant-facing launch.

## 9. Follow-up work to queue

- `t_41610e0a` TRUSTCENTER-API-CURSOR-HARDENING: bind Trust Center list cursors to tenant,
  actor/principal, filters, and expiry. Add stale/replayed/cross-scope cursor
  regression tests.
- `t_95d311cb` Review/fix: TRUSTCENTER-DOCS-SEC-001 packet. Required lenses: security threat
  model, privacy/data-class/export controls, docs claim-boundary/customer-safe
  language, and launch-gate evidence completeness.
- `t_cf64824f` TRUSTCENTER-LAUNCH-CLAIM-GATE: after implementation reviews complete, gather
  current PR/CI/browser/API evidence and either approve a limited claim tier or
  explicitly keep launch blocked.
