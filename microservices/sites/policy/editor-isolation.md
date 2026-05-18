---
doc_class: PolicyContract
template_id: TPL-POLICY
microservice: sites
policy_id: POLICY-editor-isolation
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-sites
related_adrs: [ADR-0028, ADR-0117, ADR-0126, ADR-0140, ADR-SITES-0001]
related_artifacts:
  - microservices/sites/policy/tenant-scope.cedar
  - microservices/sites/policy/ci-scope.cedar
  - microservices/sites/policy/auditor-scope.cedar
  - microservices/sites/policy/public-read.cedar
doc_status: published
---

# Editor Isolation Policy — sites µservice

## Purpose

Define the per-tenant + per-role isolation invariants that bound how
editor activity flows across:
1. Tenant boundary (Tenant-A editor ↛ Tenant-B site).
2. Role boundary (tenant_editor ↛ admin actions).
3. Publish boundary (draft ↛ published until tenant_publisher acts).
4. AI-page-build boundary (T2 prompts never cross tenant; T2 never auto-publishes).
5. CRDT-session boundary (Loro CRDT log per-tenant; cross-tenant op refused).

This policy is enforced by:
- Postgres per-tenant Row-Level Security (RLS) — DB-layer prevention.
- Tenant-DEK envelope encryption — cryptographic prevention (for non-public content).
- Cedar policies (`tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`) — runtime authorization.
- Rust type-system separation (DraftPage ≠ PublishedPage at the type level) — compile-time prevention.
- LEAN CI lanes (`oya-check-tenant-isolation`, `oya-check-rls-coverage`, `oya-check-crdt-tenant-scope`, `oya-check-ai-page-build-tenant-isolation`) — change-time prevention.

## Isolation Invariants

### Invariant 1 — Per-tenant isolation

> No site / page / block / CMS-collection / search-index entry / Loro CRDT op / image asset row from Tenant-A is ever returned by any Postgres / Meilisearch / Loro / S3 query made on behalf of Tenant-B, except via:
> - Public read of a tenant-A page when `visibility=public` AND `public_opt_in_signed_by_tenant=true` (per `public-read.cedar`).
> - Public read of a tenant-A cms-collection entry when same flags.

**Enforcement:**
- Postgres RLS policy `site_tenant_isolation`:
  ```sql
  CREATE POLICY site_tenant_isolation ON sites_site
    USING (tenant_id = current_setting('app.current_tenant')::uuid);
  ```
- Application connection-pool sets `app.current_tenant` from OIDC subject; bypass requires DB superuser (audited via OpenBao JIT).
- LEAN check `oya-check-rls-coverage` refuses build if any new table lacks an RLS policy.
- Per-tenant Meilisearch index (one index per tenant); cross-tenant query at the search-rest layer is structurally impossible.
- Per-tenant Loro CRDT log namespace; CRDT relay refuses cross-tenant session-token.
- Per-tenant S3 prefix; IAM scoped per-tenant.

### Invariant 2 — Role isolation

> Tenant editor cannot publish; tenant_publisher cannot delete; tenant_admin cannot revoke certs without 2-person rule; compliance_officer cannot perform editor actions.

**Enforcement:**
- Cedar `tenant-scope.cedar` separates role permissions explicitly.
- 2-person rule for cert revoke + page delete on legal-hold.
- LEAN `oya-check-role-action-coverage` ensures every action has at least one role permitted.

### Invariant 3 — Draft / published isolation

> A page in `draft` state is NEVER served to an anonymous reader, even if the site is public. Only `published` pages are CDN-served. Preview-mode draft URLs require a signed-token URL.

**Enforcement:**
- Rust type-system: `DraftPage` and `PublishedPage` are distinct types in `oya-sites-page-kernel`; the rendering pipeline accepts only `PublishedPage`.
- Cedar `public-read.cedar` refuses draft/scheduled/internal_review state.
- Preview-mode URL signed-token check at rest layer; refused if invalid.
- LEAN `oya-check-draft-leak` refuses any code path that publishes draft state to CDN.

### Invariant 4 — AI-page-build T2 isolation

> T2 AI-page-build prompts are tenant-DEK wrapped before being sent to the foundry-runtime; the LLM provider sees ciphertext only. Cross-tenant training on tenant content is structurally forbidden by foundry-runtime; sites enforces no fine-tune-export of tenant data.

**Enforcement:**
- Per-tenant DEK wraps prompts.
- Foundry-runtime private-inference channel rejects cross-tenant requests.
- LEAN `oya-check-ai-page-build-tenant-isolation` refuses code paths that include cross-tenant context in prompt construction.
- T2 auto-publish refused; T2 requires explicit editor accept within 30s reversibility window.

### Invariant 5 — CRDT session isolation

> A Loro CRDT session is scoped to a single (tenant_id, site_id, page_id) triple. Cross-tenant or cross-page operation messages are refused at session-token validation.

**Enforcement:**
- Per-tenant Loro CRDT relay; relay validates `tenant_id == session.tenant_id` on every op.
- Cedar `tenant-scope.cedar` admits `block_write` only when `principal.tenant_id == resource.tenant_id`.
- LEAN `oya-check-crdt-tenant-scope` validates CRDT log entries.

### Invariant 6 — Tenant-DEK encryption-at-rest for non-public content

> All non-public page content (drafts, intranet, restricted) is encrypted at rest under the per-tenant DEK (Bominal ADR-0111 envelope-encryption). Published-public pages are stored unencrypted in S3 (the bytes are public by design) but their authorship metadata is encrypted.

**Enforcement:**
- DEKs issued by OpenBao (`cloud-secrets` µservice); rotated 90d.
- Page-store writer wraps content in `Encrypted<T>` type for non-public; ciphertext bound to DEK ID.
- DEK rotation event re-encrypts active non-public records; old DEKs available read-only.
- LEAN `oya-check-dek-binding-integrity` validates ciphertext binding.

### Invariant 7 — Image upload isolation

> Tenant-A's uploaded image MUST live under tenant-A's S3 prefix; the derived variants (WebP/AVIF/JPEG-XL) inherit the same prefix. EXIF stripped per ADR-SITES-0007.

**Enforcement:**
- S3 prefix bound to tenant_id at upload time.
- libvips strips EXIF + sanitises SVG.
- LEAN `oya-check-image-tenant-scope` refuses cross-tenant image URL resolution at render.

### Invariant 8 — Custom-domain isolation

> A custom domain bound to tenant-A's site CANNOT be used to publish tenant-B's content. Cert issuance refused unless `Domain{tenant_id} == publishing tenant_id` set.

**Enforcement:**
- Cedar refuses cross-tenant cert request.
- Publish-pipeline refuses to publish to a domain not in `Domain{tenant_id}` set.
- LEAN `oya-check-domain-tenant-scope` enforces.

### Invariant 9 — Search-index per-tenant isolation

> Each tenant has its own Meilisearch index. A search query against tenant-A's index NEVER returns tenant-B's results, even if the query string matches.

**Enforcement:**
- Meilisearch per-tenant index name (per ADR-SITES-0005).
- search-rest derives index name from request OIDC subject; cross-tenant index name refused.
- LEAN `oya-check-search-index-tenant-scope` validates.

### Invariant 10 — Audit-chain integrity

> Every state transition (site create, page write, page publish, page revert, page delete, domain bind, cert issue, cms-collection update, ai-page-build accept, legal-hold apply/release) emits an Ed25519-sealed audit-chain event. Bypass impossible at the kernel layer.

**Enforcement:**
- Kernel ports require an `&mut AuditChainEmitter` argument on every mutating method.
- LEAN `oya-check-audit-chain-emission-coverage` refuses code paths that mutate state without emission.

## Per-pack overlays

### pack-eu (GDPR Art. 17 + EU AI Act Art. 50 + EU DSA Art. 14)

- Right-to-erasure: page-usecase erasure orchestrator cascades to S3 + Meilisearch + Loro CRDT log + audit-chain (audit-chain retains erasure-flag, not original content).
- EU AI Act Art. 50: T2 AI-page-build UI label "AI is suggesting this page — review before publish".
- EU DSA Art. 14: publish-refusal carries policy citation; tenant can serve transparency report.

### pack-us-healthcare (HIPAA + ADA + Section 508)

- BAA required at tenant onboarding; LEAN refuses pack-us-healthcare without `baa_on_file=true`.
- WCAG 2.2 AA refuse-publish at < 100% (Section 508 + ADA aligned).
- PHI data-class on relevant CMS-collection fields.

### pack-kr (KR PIPA Art. 23 + 전자문서법)

- Special-category data class `SENSITIVE_PIPA_ART23` on flagged fields.
- 전자문서법 Art. 5 integrity via audit-chain Ed25519.

## Verification

| Check | Cadence | Owner |
|---|---|---|
| LEAN: tenant isolation coverage | per-PR | axis-sites |
| LEAN: RLS coverage | per-PR | axis-sites |
| LEAN: draft-leak refusal | per-PR | axis-sites |
| LEAN: AI-page-build tenant isolation | per-PR | ops-security |
| LEAN: CRDT tenant scope | per-PR | ops-security |
| LEAN: search-index tenant scope | per-PR | axis-sites |
| LEAN: image tenant scope | per-PR | axis-sites |
| LEAN: domain tenant scope | per-PR | axis-sites |
| LEAN: audit-chain emission coverage | per-PR | ops-security |
| Pen-test: cross-tenant page-read attempt | annually | external pen-test firm |
| Pen-test: AI-page-build cross-tenant prompt leak | annually | external pen-test firm |

## References

- ADR-0028 (Bominal audit-chain).
- ADR-0117 (data residency).
- ADR-0126 (Connect unbundle).
- ADR-0140 (Cedar policy).
- ADR-SITES-0001 (Loro CRDT).
- `policy/tenant-scope.cedar`, `policy/ci-scope.cedar`, `policy/auditor-scope.cedar`, `policy/public-read.cedar`.
- `threat-model.md`, `dpia.md`, `compliance.md`.
- HIPAA 45 CFR §164.502(b) (minimum-necessary).
- KR PIPA Art. 23.
- EU AI Act Regulation (EU) 2024/1689 Art. 50.
- EU DSA Regulation (EU) 2022/2065 Art. 14.
- WCAG 2.2 — w3.org/TR/WCAG22.
