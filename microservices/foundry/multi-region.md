---
doc_class: MULTI-REGION
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: ops-sre-reliability + axis-foundry
related_adrs: [ADR-0117, ADR-0136, ADR-0137]
---

# Multi-Region Plan — foundry (consolidated)

## Scope

Cross-BC, cross-pack regional topology for foundry. Per-BC multi-region
docs preserved at `bc-sources/<bc>/multi-region.md`.

## M01 Launch (2026-Q3)

- **Single pack: pack-kr** on OCI ap-seoul-1.
- All 6 BCs deployed in single Kubernetes cluster.
- HA: per-BC Helm subchart declares `minReplicas: 3` for stateless tiers;
  6-shard Valkey cluster (runtime session-state); 3-replica Postgres
  (per-BC); 3-replica ClickHouse (eval); 3-region S3 (evidence blob).

## Post-M01 expansion sequence

| Wave | Pack | Trigger | Notes |
|---|---|---|---|
| 1 | pack-eu (Frankfurt) | first EU tenant signed; GDPR + EU AI Act conformity | per-pack overlay applied across 6 BCs |
| 2 | pack-us (us-east + us-west) | first US tenant signed | CCPA overlay |
| 3 | pack-us-healthcare | first BAA signed | HIPAA overlay; 6y retention |
| 4 | pack-jp (Tokyo) | first JP tenant; APPI overlay | |
| 5 | pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | per-tenant trigger | per-jurisdiction overlay |

## Cross-pack invariants

- **Cross-pack data flow is forbidden by default.** Per ADR-0117 + per-pack
  Cedar fragments. Per-BC adapters refuse cross-pack reads/writes; CI lane
  `per-pack-residency` blocks misconfiguration at PR-time.
- **Per-pack independence**: each pack ships a complete foundry stack;
  outage in one pack does not degrade another.
- **Per-pack KMS keys**: never shared across packs; rotation per pack;
  audit-chain seal binds pack identity.

## Per-BC multi-region notes

| BC | Pack-local state | Cross-pack flow | Latency sensitivity |
|---|---|---|---|
| runtime | session-state Valkey + capability-cache Postgres | none | dispatch p99 ≤50ms — co-locate with caller |
| supervisor | fleet-state Postgres | none | command propagation ≤5s — same-pack |
| eval | baseline-store S3 + ClickHouse | optional cross-pack read of baseline store (signed, content-addressed) | scheduling ≤500ms |
| evidence | pack-builder Postgres + blob S3 | regulator-export across packs (signed envelope only) | pack assembly ≤2s/100MB |
| guardrails | rule-store Postgres + ONNX serving | none | inline ≤20ms |
| providers | router Postgres + Valkey rate-limit + OpenBao | provider call goes to provider-side endpoint (provider's own region) | router ≤5ms |

## DR / failover

- **RPO**: ≤30s per BC (sync-replicated state in adapters).
- **RTO**: ≤5 min per BC (HA failover; Kubernetes deployment-controller).
- **Pack-level outage**: declare in incident-response.md; tenant SLA carve-
  out per `cost-budget.md` and tenant contract; cross-pack failover is
  manual + tenant-approved (per ADR-0117 cross-pack-forbidden).

## Per-BC multi-region archives

- `bc-sources/runtime/multi-region.md`
- `bc-sources/supervisor/multi-region.md`
- `bc-sources/eval/multi-region.md`
- `bc-sources/evidence/multi-region.md`
- `bc-sources/guardrails/multi-region.md`
- `bc-sources/providers/multi-region.md`

## References

- ADR-0117: Data-residency + jurisdiction codes.
- ADR-0136 / ADR-0137: foundry topology.

---

## ADR-0158 Multi-Region Disposition Statement

**Disposition: `single_region` per cell (GPU pool pinned to region).**

Per ADR-0158, the foundry µservice is declared `single_region`. The GPU pool is physically pinned to the region; cross-region GPU pool replication is operationally infeasible and economically dispositive. Tenant routing pins the tenant to a foundry GPU pool in their home region.

| Property | Value |
|---|---|
| Disposition | `single_region` |
| RPO (intra-cell) | ≤ 60 seconds (model-weight checkpoint replication) |
| RTO (intra-cell) | ≤ 5 minutes (GPU pool failover within region) |
| Cross-region GPU pool replication | FORBIDDEN (cost + sovereignty) |
| Sovereign-pin behavior | tenant routes only to in-region foundry; pack-ksa tenant never reaches non-KSA GPU pool |

## ADR-0164 Sovereign Cloud / Air-Gapped Deployment Variant

Per ADR-0164, the foundry µservice ships a per-pack air-gap variant. In air-gap mode:

### On-prem LLM only

- External LLM provider calls (Anthropic, OpenAI, Google Gemini) are FORBIDDEN.
- `foundry-providers` adapter code for external providers is ABSENT from air-gap pack image builds.
- Egress NetworkPolicy + Cilium L7 egress policy deny external hosts.
- Istio `ServiceEntry` for external LLM hosts is absent.

### vLLM serving on cell GPU pool

- vLLM 0.6+ serves Llama 3.x / DeepSeek / Qwen / Mistral / Falcon (G42) on the cell's GPU pool.
- Ollama for smaller models / dev tier.
- Per-pack model selection in `microservices/foundry/iac/kustomize/components/pack-{name}/values.yaml`.

### Pack matrix (foundry perspective)

| Pack | `air_gap` | LLM strategy |
|---|---|---|
| `pack-eu-sovereign-airgap` | true | vLLM Llama 3 + Mistral (EU-region GPUs) |
| `pack-kr-fsc` | true | vLLM HyperCLOVA-X (Naver Cloud) + Llama 3 |
| `pack-kr-public` | true | vLLM Llama 3 |
| `pack-ksa` | true | vLLM Falcon (G42) + Llama 3 |
| `pack-uae` | true | vLLM Falcon + Llama 3 |
| `pack-us-gov` | true | vLLM Llama 3 |
| `pack-us-shared` | false | external Anthropic / OpenAI default + on-prem fallback |
| `pack-eu` | false | external EU-region only (Anthropic EU / OpenAI EU) |
| `pack-kr` | false | external KR-region only (HyperCLOVA-X) |
| `pack-jp` | false | external JP-region only |

### vLLM Helm chart

vLLM Helm chart at `microservices/foundry/iac/helm/vllm/` with per-pack model selection. Per-pack values.yaml selects model + GPU SKU + quantization tier.

CI lane `oya gate validate air-gap-overlay` enforces (a) air-gap packs reference no external LLM host, (b) foundry image build excludes external-provider adapter binaries, (c) vLLM Helm chart present in air-gap pack manifest.

See `/specs/sovereign-cloud-air-gapped-canonical.json` for the canonical declaration.
