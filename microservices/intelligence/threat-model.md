# Intelligence Threat Model

## Assets

- Tenant context, consent grants, retrieval citations, advisory drafts, refusal reasons, and audit events.

## Threats

- Prompt or retrieval context crosses tenant boundaries.
- Draft output is treated as an automatic mutation.
- Consent or budget checks are bypassed.
- Citation leakage reveals restricted tenant data.

## Mitigations

- Tenant, context, consent, and budget are required before model or retrieval invocation.
- Drafts remain advisory and require deterministic builder import.
- Policy refusals emit audit-chain evidence.
- Citations are scoped to approved context boundaries.
