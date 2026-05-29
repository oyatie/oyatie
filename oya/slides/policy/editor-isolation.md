---
doc_class: PolicyDoc
template_id: TPL-POLICY-DOC
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-security
related_artifacts:
  - microservices/slides/threat-model.md
  - microservices/slides/policy/tenant-scope.cedar
  - microservices/workflow-studio/policy/editor-isolation.md
  - microservices/docs/policy/editor-isolation.md
  - microservices/sheets/policy/editor-isolation.md
doc_status: published
---

# Editor isolation — slides µservice

This policy defines multi-tenant editor session safety guarantees for the slides µservice. It mirrors the equivalent policy in `workflow-studio` + `docs` + `sheets` (Loro CRDT family) and refines for slides-specific concerns (per-slide ACL, broadcast-mode, AI-content-generation, embed-bridges).

## Invariants

1. **Tenant binding rebound at every WS message dispatch**. Server cannot trust client-supplied `tenant_id` mid-stream; the WS-upgrade OIDC sub is the binding authority.
2. **Per-cell Valkey cluster cell-locality**. CRDT state for a deck NEVER crosses cells.
3. **Per-tenant Postgres RLS**. Deck rows tagged with `tenant_id`; every query passes through RLS.
4. **Per-tenant S3 prefix**. Deck content snapshots + assets under `<tenant_id>/...` prefix; per-tenant IAM condition.
5. **Per-tenant CDN cache key**. Cache partitioned by `(tenant_hash, pack, version)`.
6. **Per-slide ACL evaluated additionally** to deck-level (ADR-SLIDES-0007).
7. **Speaker-notes scope = presenter-view only**. Never broadcast frame. Never embed-bridged.
8. **Per-pack residency overlay**. Cross-pack collab refused at admission gate.
9. **Per-session HMAC** on CRDT op envelopes. Tampering surfaces as Sev-1 alarm.
10. **Single-writer lease per deck** via Valkey. Split-brain detected + reconciled.
11. **WASM bundle SRI**. Every chunk SHA-384 hash; mismatch refuses load.
12. **Strict CSP**. No inline scripts; no eval; WASM-bootstrap nonce per request.
13. **Embed-bridge sanitization at boundary**. Cross-µservice embed content sanitized at slides-side bridge before render.
14. **Broadcast presenter token bound to OIDC sub + Cedar evaluation**. Messenger-issued one-time signed token; revoked after session end.
15. **AI prompt + completion sealed by audit-chain**. EU AI Act risk-class stamped per invocation.

## Forbidden patterns

- `innerHTML` on any deck content.
- `eval()` anywhere in the slides client bundle.
- Direct DB / Valkey / S3 read or write from cross-µservice consumers (must go through SDK).
- Loro types leaked through SDK boundaries (per ADR-SLIDES-0001).
- LiveKit types leaked through SDK boundaries (per ADR-SLIDES-0005; messenger-SDK is the only client).
- Cross-pack collab.
- Per-slide ACL bypass via deck-level grant alone.
- Speaker-notes embedded in broadcast frame.
- Mid-render per-tenant branding injection (CSP + sanitization refusal).
- T2 AI-content-generation without explicit human-accept gate (Annex III high-risk refusal default; per-pack override required).

## Enforcement

- `oya gate validate per-microservice-layout --microservice slides` — verifies layout.
- `oya gate validate cedar-preview-required --microservice slides` — verifies every save path exercises Cedar preview.
- `oya gate validate wasm-bundle-sri --microservice slides` — verifies SRI.
- `oya gate validate ai-act-risk-class-stamp --microservice slides` — verifies every T2 invocation carries a risk-class.
- `oya gate validate reduced-motion-fallback-mandatory --microservice slides` — verifies animations BC honors `prefers-reduced-motion`.
- Runtime: per-session HMAC verification + Sev-1 alarm on mismatch.
- Runtime: per-tenant CDN cache key audit.
- Runtime: cross-pack op refusal at admission gate.

## References

- `threat-model.md` §"Information Disclosure" + §"Elevation of Privilege".
- ADR-SLIDES-0001 (Loro types containment).
- ADR-SLIDES-0005 (LiveKit reuse pattern).
- ADR-SLIDES-0007 (per-slide ACL).
- ADR-SLIDES-0006 (AI risk-class).
- `workflow-studio/policy/editor-isolation.md` (parallel pattern; collab-CRDT family).
- `docs/policy/editor-isolation.md` (parallel; docs collab).
- `sheets/policy/editor-isolation.md` (parallel; sheets collab).
