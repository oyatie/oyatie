# Cloud Intelligence service — Data Residency

**Authority:** ADR-0373 (audit + default-off body logging), ADR-0373 (per-tenant isolation)
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §7 (data residency — access-controlled, redaction-aware, region-pinned logging/evidence).
**Last reviewed:** 2026-06-10
**Applicable regulatory packs:** GDPR / EU-AI-ACT-2024-HIGH-RISK, KR-PIPA-2023-amendment, SOC2-T2, ISO27001-2022 (see `manifest.json#regulatory_packs`).

## The residency problem for an LLM proxy

Prompts and completions are the highest-sensitivity data a gateway touches — they can contain
arbitrary tenant PII, secrets, or regulated content. The brief (§7) found the mature pattern across
vendors: **logging/evidence must be access-controlled, redaction-aware, and region-pinned** (AWS
Bedrock invocation logging is off-by-default and account/region-scoped; blocked content can appear
plaintext unless logging is disabled or tightly restricted → IAM-restrict + SIEM; Bedrock PII
block/mask filters; GCP Model Armor / Cloudflare DLP). Oyatie adopts a stricter posture: raw prompt,
completion, and tool payloads are not stored on the normal path.

## Adopted posture (brief §7 "Adopt")

### 1. Prompt/completion body storage forbidden on the normal path
- The gateway **does not persist** prompt, completion, or tool-call payload bodies during normal
  request handling. `llm.audit.v1` and `llm.usage.v1` carry only token counts + hashed/redacted refs
  (see `audit-evidence-emission.md`), never body text.
- Tenant policy cannot opt into normal-path raw body storage. Product teams may add product-layer
  records, but cloud-intelligence events remain redacted and handle-based.

### 2. Guardrail-triggered encrypted evidence handles
- Critical guardrail triggers create an encrypted `evidence-ref://` handle through the owned
  secret-provider/KMS ports. The evidence handle is **separate** from metering; metering never gains
  body access.
- Evidence is region-pinned by tenant and data class. A mismatch between the tenant's region, data
  class, and evidence target **blocks capture** fail-closed; the model call remains blocked and the
  incident escalates with redacted metadata.
- Default reviewer material is redacted structured evidence. Raw payload access requires audited
  break-glass with reason, approver, fixed TTL, and SIEM-forwarded access event.

### 3. In-transit redaction and reversible tokens
- Sensitive personal/security classes are blocked before provider, routing-advisor, or secondary
  review boundaries.
- Trivial personal data is redacted or replaced with tenant-policy-approved reversible tokens before
  the model call. Tokens are ephemeral per run/task by default and restored only after model output.
  Longer-lived token maps require a named workflow policy, TTL, and audit trail.

### 4. Never persist tokens
- Provider keys and bearer tokens are never written to the body store or the audit record — only
  non-reversible hash fingerprints (brief §5, §7).

### 5. Audit-read gated behind a distinct authority + SIEM
- Reading the audit stream / body store requires the owned policy-engine `AuditReader`
  role (distinct from admin), and reads are SIEM-forwarded (brief §7). The current
  Cedar policy file is a transient fixture for that port.

### 6. Per-tenant region
- Each tenant has a residency region; the gateway records it on every audit record and pins any
  evidence handle to it. This is the same residency model the rest of the Oyatie fleet uses (KR / EU
  strict packs).

## Data classes touched

| Data | Class | Persistence | Residency control |
|---|---|---|---|
| Prompt body | `AI_PROMPT` (may contain PII) | normal path: never persisted; guardrail trigger: encrypted evidence handle, redacted default view, fixed TTL | `residency_region`, data-class TTL, fail-closed on mismatch |
| Completion body | `AI_COMPLETION` (may contain PII) | normal path: never persisted; guardrail trigger: encrypted evidence handle, redacted default view, fixed TTL | same |
| Tool payload / scheduled-task input | `AI_TOOL_PAYLOAD` (may contain secrets/PII) | normal path: never persisted; guardrail trigger: encrypted evidence handle, redacted default view, fixed TTL | same |
| Token counts / cost | `BILLING` | `llm.usage.v1` (low-PII) | metering stream (not body-pinned) |
| Provider key | `SECRET_REFERENCE` | never persisted; in-memory only | cloud-secrets/cloud-kms region |
| Ingress/admin token | `SECRET_REFERENCE` | never persisted; hashed ref only | n/a |
| Audit record | `AUDIT_INTERNAL` | `evidence/audit-chain.jsonl` (hash-chained) | tenant + `residency_region` recorded |

## Regulatory pack mapping

- **GDPR / EU-AI-ACT-2024-HIGH-RISK:** no normal-path raw body storage + redaction + per-tenant region +
  audit-read authority satisfy data-minimization and access-control expectations; the EU-AI-Act
  high-risk pack additionally drives the audit completeness (every model invocation recorded).
- **KR-PIPA-2023-amendment:** KR-resident tenants pin evidence handles + audit to the KR region; no
  cross-region body movement.
- **SOC2-T2 / ISO27001-2022:** immutable, hash-chained, access-controlled audit + alert-if-disabled
  (brief §9) provides the control evidence.

## Non-claims

- No live region-pinned evidence store is provisioned by the current foundation
  (`CS-CLOUD-INTELLIGENCE-AGENT-DISPATCH-001`); the evidence-store lifecycle is owned by cloud-iac / cloud-secrets
  (PRD open-question 3). This document specs the posture the rest crate enforces, not a deployed
  evidence store.

## References

- `design/hyperscaler-best-practice-brief.md` §7, §9.
- `design/audit-evidence-emission.md`, `design/tenant-isolation.md`, `policy/cloud-intelligence.cedar`.
- AWS Bedrock model-invocation logging (off-by-default, account/region-scoped); GCP Model Armor; Cloudflare DLP (brief §7).
