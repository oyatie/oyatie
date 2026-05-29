# whiteboard remediation notes
## Wave 15-IP-substance scrub (2026-05-21)
- Scope: IP-BUCKET-O review for `whiteboard`.
- IPs rewritten in place: 0.
- IPs deleted as duplicative: 0.
- IPs preserved as already-substantive: 30.
- Counterpart anchors were made explicit where the verification regex lacked the service's native benchmark vocabulary.
- Follow-up: none for stamp-shell conversion; future service owners may still improve individual IP depth outside this bucket.

## Wave 15J-final-cleanup

- Bucket: F-BUCKET-3.
- Action: deleted stale 2026-05-20 coherence audit and feature parity artifacts; replaced manifest `capability_tier*` fields with tenant-class doctrine and scrubbed architecture wording.
- Verification: tier-name grep and `capability_tier|max_tier|tier_threshold` grep both return 0 outside remediation notes.
- Follow-ups: none.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-6. PRD updated: `microservices/whiteboard/PRD.md`. Related ADRs added: ADR-0338, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345.
- DR posture (ADR-0343): values: manifest RTO p99 1800s, RPO p99 120s, multi_region_active_active=true, `active-active-multi-az-cross-region-warm`, `runbooks/dr-failover.md`, HIPAA-2024 floor exceeded. Alternative rejected: active-passive-only CRDT board history after D-2 declared active-active. Cost: reconnect UX and replay validation during promotion.
- Capacity model (ADR-0340): values: manifest 0.12 vCPU, 384 MiB RAM, 8 GB storage, valkey=3, postgres=2, outbound_http=4, `per_user` scaling, Tier-3 placement, 2-80 collaboration/presence pods and 2-20 export pods. Alternative rejected: sizing only by board count. Cost: session-aware placement and export backpressure.
- Sustainability and cost attribution (ADR-0344): values: per-call `cost_usd_minor_units`, `co2_grams`, `watt_hours` on board open, canvas op, template import, export render, and replay rows. Alternative rejected: pooling live collaboration emissions into workspace overhead. Cost: hot-path audit writes carry extra dimensions.
- API versioning posture (ADR-0342): values: public `YYYY-MM-DD` carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for board clients and migration adapters, ADR-0145 internal mesh exemption. Alternative rejected: client-build version as API contract. Cost: board-client compatibility harness.
