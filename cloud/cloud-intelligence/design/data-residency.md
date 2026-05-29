# Cloud Intelligence service — Data Residency

**Authority:** ADR-0373 (audit + default-off body logging), ADR-0373 (per-tenant isolation)
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §7 (data residency — logging opt-in, access-controlled, redaction-aware, region-pinned).
**Last reviewed:** 2026-05-26
**Applicable regulatory packs:** GDPR / EU-AI-ACT-2024-HIGH-RISK, KR-PIPA-2023-amendment, SOC2-T2, ISO27001-2022 (see `manifest.json#regulatory_packs`).

## The residency problem for an LLM proxy

Prompts and completions are the highest-sensitivity data a gateway touches — they can contain
arbitrary tenant PII, secrets, or regulated content. The brief (§7) found the mature pattern across
vendors: **logging is opt-in, access-controlled, redaction-aware, and region-pinned** (AWS Bedrock
invocation logging is off-by-default and account/region-scoped; blocked content appears plaintext
unless logging is disabled → IAM-restrict + SIEM; Bedrock PII block/mask filters; GCP Model Armor /
Cloudflare DLP).

## Adopted posture (brief §7 "Adopt")

### 1. Prompt/completion body logging default-OFF
- The gateway **does not persist** prompt or completion bodies. By default, `llm.audit.v1` and
  `llm.usage.v1` carry only token counts + hashed refs (see `audit-evidence-emission.md`), never
  body text.
- Body capture is **per-tenant opt-in** only, with an explicit retention TTL.

### 2. Residency-pinned storage, separate from metering
- When a tenant opts in, the redacted body spills to a **residency-pinned object-storage bucket**
  (the tenant's region), referenced by `prompt_uri` / `completion_uri` (Bedrock >100KB
  externalization pattern). This bucket is **separate** from the metering stream (brief §7 — keep
  residency-pinned bodies out of the low-PII metering path).
- The audit record carries `residency_region`; a mismatch between the tenant's region and the
  target bucket **blocks the spill** (fail-closed; the body stays un-persisted).

### 3. Redaction / masking before persistence
- A redaction pass runs **before** any body is written (brief §7 — PII block/mask, Model Armor /
  Cloudflare DLP analog). v1 provides the hook; a DLP/guardrail provider can be wired (the same PEP
  as the threat-model LLM01/LLM02 hook).

### 4. Never persist tokens
- Provider keys and bearer tokens are never written to the body store or the audit record — only
  non-reversible hash fingerprints (brief §5, §7).

### 5. Audit-read gated behind a distinct authority + SIEM
- Reading the audit stream / body store requires the `AuditReader` Cedar role (distinct from admin),
  and reads are SIEM-forwarded (brief §7). See `policy/cloud-intelligence.cedar`.

### 6. Per-tenant region
- Each tenant has a residency region; the gateway records it on every audit record and pins any
  body-spill to it. This is the same residency model the rest of the Oyatie fleet uses (KR / EU
  strict packs).

## Data classes touched

| Data | Class | Persistence | Residency control |
|---|---|---|---|
| Prompt body | `AI_PROMPT` (may contain PII) | default-OFF; opt-in → redacted, region-pinned bucket | `residency_region`, fail-closed on mismatch |
| Completion body | `AI_COMPLETION` (may contain PII) | default-OFF; opt-in → redacted, region-pinned bucket | same |
| Token counts / cost | `BILLING` | `llm.usage.v1` (low-PII) | metering stream (not body-pinned) |
| Provider key | `SECRET_REFERENCE` | never persisted; in-memory only | OpenBao region (cloud-kms) |
| Ingress/admin token | `SECRET_REFERENCE` | never persisted; hashed ref only | n/a |
| Audit record | `AUDIT_INTERNAL` | `evidence/audit-chain.jsonl` (hash-chained) | tenant + `residency_region` recorded |

## Regulatory pack mapping

- **GDPR / EU-AI-ACT-2024-HIGH-RISK:** default-OFF body logging + redaction + per-tenant region +
  audit-read authority satisfy data-minimization and access-control expectations; the EU-AI-Act
  high-risk pack additionally drives the audit completeness (every model invocation recorded).
- **KR-PIPA-2023-amendment:** KR-resident tenants pin body-spill + audit to the KR region; no
  cross-region body movement.
- **SOC2-T2 / ISO27001-2022:** immutable, hash-chained, access-controlled audit + alert-if-disabled
  (brief §9) provides the control evidence.

## Non-claims

- No live region-pinned bucket is provisioned by the current foundation
  (`CS-CLOUD-INTELLIGENCE-AGENT-DISPATCH-001`); the bucket lifecycle is owned by cloud-iac / cloud-secrets
  (PRD open-question 3). This document specs the posture the rest crate enforces, not a deployed
  bucket.

## References

- `design/hyperscaler-best-practice-brief.md` §7, §9.
- `design/audit-evidence-emission.md`, `design/tenant-isolation.md`, `policy/cloud-intelligence.cedar`.
- AWS Bedrock model-invocation logging (off-by-default, account/region-scoped); GCP Model Armor; Cloudflare DLP (brief §7).
