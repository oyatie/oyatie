---
doc_class: MultiRegionPlan
template_id: TPL-MULTI-REGION
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence + ops-sre-reliability + council-privacy
related_adrs: [ADR-0117, ADR-0255, ADR-0252, ADR-0253, ADR-0254]
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-region Plan — intelligence µservice

## Purpose

Define per-region provider routing, data-residency invariants per pack, DR pair topology, failover
procedure, and per-pack provider catalog. This document is the canonical residency artifact for EU
DPAs (per GDPR Arts. 44–50 + EU AI Act Art. 16), the Korean PIPC (per PIPA Art. 28 + Art. 23-2),
and equivalent supervisory authorities.

## Per-pack provider catalog

The provider catalog is pack-pinned. Dispatch from `pack-X` may only reach providers listed for
`pack-X`. Cross-pack provider routing is **forbidden by default**.

| Pack | Permitted providers | Region | Notes |
|---|---|---|---|
| pack-kr | Anthropic EU (KR egress via OCI EU mirror), OpenAI EU (KR egress via Azure EU), vLLM self-hosted KR, Naver HyperCLOVA-X (planned), Apple Foundation Models on-device | ap-seoul-1 | KR PIPA Art. 23-2 sensitive data refusal floor active |
| pack-eu | Anthropic EU, OpenAI EU, Vertex AI EU (eu-west4 / eu-central1), Azure OpenAI EU (Sweden Central / France Central), Mistral La Plateforme (EU-native), Cohere EU, vLLM EU self-hosted, Apple Foundation Models on-device | eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | EU AI Act Annex III refusal layer active |
| pack-us | Anthropic US, OpenAI US, Vertex AI US, AWS Bedrock US, Azure OpenAI US, Cohere US, vLLM US self-hosted, OpenRouter, Together, Groq, Replicate, Apple Foundation Models on-device | us-ashburn-1 + us-phoenix-1 (DR pair) | All non-restricted |
| pack-us-healthcare | Anthropic-via-AWS-Bedrock-BAA, Azure-OpenAI-BAA, AWS Titan-BAA, vLLM HIPAA-eligible self-hosted | us-ashburn-1 (HIPAA-eligible) | BAA-signed providers only |
| pack-us-federal (FedRAMP) | Azure-OpenAI-Gov-cloud, AWS Bedrock Gov-cloud | us-gov-ashburn-1 | FedRAMP High; CJIS overlay available |
| pack-jp | OpenAI Japan (via Azure JP), Vertex AI JP (asia-northeast1), Anthropic-via-Bedrock JP, Apple Foundation Models | ap-tokyo-1 | APPI compliance |
| pack-sg | OpenAI APAC, Vertex AI APAC (asia-southeast1), Anthropic-via-Bedrock APAC, vLLM SG self-hosted | ap-singapore-1 | PDPA + MAS-TRM |
| pack-au | OpenAI APAC, Vertex AI APAC (australia-southeast1), AWS Bedrock APAC (ap-southeast-2), Anthropic-via-Bedrock | ap-sydney-1 + ap-melbourne-1 | APP 8 cross-border |
| pack-in | Vertex AI Asia-South (asia-south1), AWS Bedrock APAC, Azure OpenAI India | ap-hyderabad-1 + ap-mumbai-1 | DPDPA 2023 §8(1)(g) |
| pack-br | Vertex AI Brazil (southamerica-east1), AWS Bedrock Brazil, Azure OpenAI Brazil South | sa-saopaulo-1 + sa-vinhedo-1 | LGPD Art. 16 |
| pack-ae | Azure OpenAI UAE Central, AWS Bedrock ME-Central | me-abudhabi-1 + me-dubai-1 | UAE PDPL |
| pack-ksa | AWS Bedrock KSA, Azure OpenAI KSA (planned) | me-jeddah-1 + me-riyadh-1 | KSA PDPL + NCA |
| pack-cn | Alibaba Qwen, Tencent Hunyuan, Baidu ERNIE | cn-shanghai-1 / cn-beijing-1 | PIPL Art. 55; outbound to US/EU providers REFUSED |
| pack-uk | OpenAI UK (via Azure UK South), Anthropic UK (via Bedrock eu-west-2), Vertex AI UK | uk-london-1 | UK GDPR |

## Pack-routing Cedar policy

`microservices/intelligence/policy/data-residency.md` carries the routing-policy details and links
to `policy/provider-routing.cedar` for enforcement.

## DR pair topology

Each pack with a DR pair maintains an active-active substrate:

```text
pack-eu:    eu-frankfurt-1  ↔  eu-amsterdam-1    (mTLS + async audit-tap replication)
pack-us:    us-ashburn-1    ↔  us-phoenix-1
pack-au:    ap-sydney-1     ↔  ap-melbourne-1
pack-in:    ap-hyderabad-1  ↔  ap-mumbai-1
pack-br:    sa-saopaulo-1   ↔  sa-vinhedo-1
pack-ae:    me-abudhabi-1   ↔  me-dubai-1
pack-ksa:   me-jeddah-1     ↔  me-riyadh-1
```

Single-region packs (pack-kr / pack-jp / pack-sg / pack-uk / pack-cn / pack-us-federal /
pack-us-healthcare) operate without intra-pack DR pair at MVP; multi-AZ within the region provides
intra-region failover. Cross-region DR for these packs requires tenant-executed SCC + ops-legal
sign-off.

## DR failover procedure

```text
Trigger: primary-region degraded (SLO breach > 15 min OR catastrophic outage signal)

1. on-call IC declares Sev-1
2. Verify DR-pair-availability (DR-side dispatch SLOs green for ≥ 5 min in the window)
3. Update Helm values: `intelligence-primary-region: <dr-region>` (the substrate's
   service-mesh routing reads this).
4. Re-bind tenant DNS via Cloudflare GeoDNS to the DR region.
5. Verify dispatch SLOs return to green on DR side within ≤ 15 min.
6. Update status page; tenant notification per `incident-response.md` template.
7. Audit-tap-worker continues emitting (audit-chain has its own DR pair).
8. Postmortem within 5 business days.
```

Failback procedure: reverse-apply once primary region is restored + audit-chain has reconciled.

## Cross-region replication policy

| Data class | Cross-region replication |
|---|---|
| Audit-tap records | Within-pack only (per pack's DR pair); never cross-pack |
| Per-call cost record | Same; projected to finops µservice which has its own replication |
| Eval canonicalen-set | Global (curated test data; no production PII) |
| Eval-online results | Within-pack only |
| Refusal Cedar fragments | Global (policy text; not personal data) |
| Provider credentials | Per-pack only (each pack has its own OpenBao instance) |

## Per-pack overlay

### pack-kr (KR PIPA + ISMS-P)

- All dispatch routes through KR-resident substrate.
- Provider-side EU routing requires explicit consent reference in dispatch envelope.
- Audit-tap retention ≥ 1 year per PIPA Enforcement Decree Art. 30; financial-sector tenants get
  5 years.

### pack-eu (GDPR + EU AI Act)

- All dispatch + audit-tap pinned to EU regions.
- Cross-pack transfer requires tenant-executed SCC + pack-router Cedar approval.
- EU AI Act Art. 16 provider obligations applied to oyatie as substrate provider.
- Art. 12 audit-tap retention ≥ 6 months minimum; 1y default; longer per tenant DPA.

### pack-us-healthcare (HIPAA)

- BAA-signed providers only.
- HIPAA-eligible OCI regions only.
- Audit-tap retention ≥ 6 years per §164.530(j) + §164.316(b)(2).

### pack-us-federal (FedRAMP High + CJIS)

- Azure-OpenAI-Gov-cloud + AWS Bedrock Gov-cloud only.
- All operators must be US persons (FedRAMP requirement).

### pack-cn (CN PIPL + Generative AI Service Provisions 2023)

- Alibaba Qwen / Tencent Hunyuan / Baidu ERNIE only.
- Outbound dispatch to US/EU providers REFUSED.
- Data-export Cedar gate blocks cross-pack on principle.

## Verification

- `cargo run -p oya-dev-cli -- gate validate pack-routing-conformance --microservice intelligence` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cross-pack-transfer-allowed-only-with-scc` — exit 0.
- Annual residency audit per pack.
- Quarterly chaos drill: induce cross-pack dispatch attempt; verify refusal.

## References

- ADR-0117 — Cloud-native infrastructure (residency).
- ADR-0255 — Intelligence as two-layer AI Substrate.
- ADR-0254 — Kubernetes + Cloud Hypervisor.
- `microservices/intelligence/policy/data-residency.md`.
- `microservices/intelligence/policy/provider-routing.cedar`.
- `microservices/intelligence/compliance.md`.
- `microservices/intelligence/legal/transfer-register.md`.
- Provider regional documentation.
- GDPR Arts. 44–50; EU AI Act Art. 16; KR PIPA Art. 23-2 + Art. 28; LGPD Art. 33; DPDPA 2023.
