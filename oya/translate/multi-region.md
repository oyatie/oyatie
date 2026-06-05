---
doc_class: MultiRegion
title: Multi-region + DR design
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-iac + axis-translate + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0139, ADR-0131, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/policy/data-residency.md
  - microservices/translate/capacity-model.md
  - microservices/translate/iac/kustomize/overlays/
review_cadence: annually + on every new pack activation
doc_status: published
---

# Multi-Region + DR Design — translate µservice

## Topology

### Per-pack regional pinning

Per `policy/data-residency.md`, each pack pins to a specific OCI region:

| Pack | Primary | DR pair | DR scope |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | none (intra-region AZ HA only; cross-region DR forbidden per PIPA Art. 28) | intra-region |
| pack-eu | OCI eu-frankfurt-1 | eu-amsterdam-1 | intra-EU DR pair (GDPR-compliant) |
| pack-us | us-ashburn-1 | us-phoenix-1 | intra-US |
| pack-us-healthcare | us-ashburn-1 (HIPAA-eligible) | us-phoenix-1 (HIPAA-eligible) | intra-US-HC |
| pack-jp | ap-tokyo-1 | ap-osaka-1 (when available) | intra-JP |
| pack-sg | ap-singapore-1 | none initially | intra-region |
| pack-au | ap-sydney-1 | ap-melbourne-1 | intra-AU |
| pack-in | ap-hyderabad-1 | ap-mumbai-1 | intra-IN (per DPDPA) |
| pack-br | sa-saopaulo-1 | sa-vinhedo-1 | intra-BR |
| pack-ae | me-abudhabi-1 | me-dubai-1 | intra-AE |
| pack-ksa | me-jeddah-1 | me-riyadh-1 | intra-KSA |
| pack-cn-stub | (CN region; scaffolding) | none | scaffolding only M01 |

### Per-pack components

Per pack, the translate stack is:

```
pack-<region>/
├── translate-router-rest    (3+ replicas across AZs)
├── translate-router-worker  (engine health monitor + cost roll-up)
├── translate-router-app     (composition root)
├── tm-rest + tm-worker      (TM service)
├── termbase-rest + worker   (termbase service)
├── qe-rest + qe-worker      (QE service)
├── langdetect-rest          (LangDetect service)
├── doc-translate-worker     (Pandoc/LibreOffice in gVisor)
├── bulk-translate-worker    (XLIFF/TMX/TBX bulk jobs)
├── stream-router            (real-time caption gateway)
├── adapter-foundry-runtime  (in-house MT/QE/LangDetect)
├── adapter-anthropic        (via foundry-providers)
├── adapter-openai           (via foundry-providers)
├── adapter-google-translate (via foundry-providers)
├── adapter-deepl            (via foundry-providers)
└── per-pack-storage:
    ├── postgres (HA primary + replica)
    ├── valkey (sentinel HA)
    ├── meilisearch (per pack)
    └── s3 (OCI Object Storage per pack)
```

## Cross-Pack Replication Policy

**Default: forbidden.** Same posture as observability + foundry-providers.

- Translation Memory units replicate intra-pack only.
- Termbase entries replicate intra-pack only.
- Bulk-job S3 artifacts replicate intra-pack only.
- Engine credentials (OpenBao) per-pack scope only.
- Audit events emit to per-pack audit-chain instance only.

### Exception: tenant-executed SCC (pack-eu intra-EU DR)

Intra-EU DR (eu-frankfurt-1 ↔ eu-amsterdam-1) is intra-region per GDPR; no SCC needed.

### Exception: HIPAA intra-US-HC DR

us-ashburn-1 ↔ us-phoenix-1 are both HIPAA-eligible OCI regions; failover is intra-region from a HIPAA perspective.

### Exception: BCDR exercise

Controlled per-pack DR drill quarterly; intra-pack only; cross-pack drill forbidden.

## DR Failover (per pack with DR pair)

When primary pack region degraded, failover to DR pair. Per `failure-modes.md` FM-04 (all vendors out) is the most common trigger; a region outage is rarer.

| Step | Action | Time |
|---|---|---|
| 1 | IC declares Sev-1 region-outage | t = 0 |
| 2 | Verify DR pair posture: `cargo run -p oya-dev-cli -- translate dr-status --pack <pack>` | ≤ 5 min |
| 3 | Drain primary: pause new bulk-jobs; let in-flight finish or timeout | ≤ 10 min |
| 4 | Update DNS + Istio VirtualService to route to DR pair | ≤ 5 min |
| 5 | Promote DR Postgres to primary (read-write); replica → standby | ≤ 5 min |
| 6 | Verify TM + termbase consistency on DR (RPO ≤ 5 min) | ≤ 5 min |
| 7 | Resume traffic on DR pair | ≤ 5 min |
| 8 | Tenant notification per `incident-response.md` | ≤ 30 min |
| 9 | Recovery to primary when region restored (reverse order) | per recovery |

**Total RTO budget: ≤ 35 min for full per-pack DR failover.**

## RPO

- Postgres: WAL-streaming + 5-min snapshot → RPO ≤ 5 min.
- Meilisearch: per-pack index re-built from Postgres → RPO ≤ 5 min (via re-index).
- Valkey: ephemeral session state; lost on failover (acceptable; user re-initiates).
- S3 bulk-job artifacts: replicated within OCI Object Storage (intra-pack RF-3); RPO ≈ 0 within pack.

## Engine Adapter Behavior on Region Outage

- Engine adapters (anthropic / openai / google / deepl) reach vendors via `foundry-providers`. If a pack region is down but vendors are reachable from DR pair, router can serve traffic from DR.
- Pack-cn-stub uses in-house ONLY (no external vendor); region outage = full outage (no DR pair in M01).

## Engine-Side Multi-Region

- Each external vendor offers per-region endpoints (api.anthropic.com EU/US/JP; translation.googleapis.com per Google Cloud region).
- `translate-router` selects per-region endpoint matching the tenant's pack.
- If vendor regional endpoint degraded but pack still up, router demotes that vendor only; continues serving via alternate vendor.

## Chaos Drill

Quarterly per pack:

| Drill | Goal | Acceptance |
|---|---|---|
| Region failover (eu-frankfurt → eu-amsterdam) | RTO ≤ 35 min; RPO ≤ 5 min | pass |
| All external vendors out (router → in-house only) | continue serving at degraded throughput | pass |
| Single vendor 429 (Anthropic rate-limit) | router demote + retry alternate | pass |
| Postgres primary loss | replica promote + DNS swap | pass |
| Meilisearch reindex | re-index from Postgres within 30 min | pass |
| Cross-region misroute attempt (FM-70) | block at decide; alert; no egress | pass (HARD) |

## SLO Impact

- Translate-request availability: 99.95 % monthly within pack.
- DR failover does NOT count against SLO budget for the duration of failover (per ADR-0139 ledger annotation).

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=multi-region --microservice translate` exits 0.
- Quarterly DR drill evidence under `evidence/dr-drills/translate-<pack>-<unix_ts>.json`.

## References

- ADR-0117 — pack residency model.
- ADR-0139 — SLO-gated promotion + rollback.
- ADR-TRANSLATE-0004 — residency-bound inference.
- `microservices/translate/failure-modes.md`.
- `microservices/translate/policy/data-residency.md`.
- OCI region availability + service class docs.
- AWS Well-Architected Reliability Pillar (multi-AZ + multi-region patterns).
- Google SRE Workbook ch. 10 (Disaster Recovery).
