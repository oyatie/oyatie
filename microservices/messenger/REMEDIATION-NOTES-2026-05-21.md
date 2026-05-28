# Messenger remediation notes - 2026-05-21

## Completion log

- **F-MSGR-001 - deployment-context IaC modules:** Completed. All six messenger OpenTofu deployment-context directories now have `main.tf`, `versions.tf`, and `README.md`: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`. The shared provider constraints were moved into per-context `versions.tf` files, and the `oyatie-public-cloud` Grafana RBAC file no longer carries a duplicate provider constraint block.
- **F-MSGR-002 - legacy Terraform directory:** Confirmed closed. `microservices/messenger/iac/terraform/` remains absent and was not recreated.
- **F-MSGR-003 - mobile-app-bundle coordination:** Completed in `PRD.md` section `3.4.B Mobile-app-bundle coordination`. Messenger is documented as one pane in the single per-platform Oyatie app bundle alongside mail, social, and community while preserving four separate backend microservices.

## Files modified or created by this pass

- `microservices/messenger/PRD.md`
- `microservices/messenger/manifest.json`
- `microservices/messenger/compliance.md`
- `microservices/messenger/policy/tenant-class.cedar`
- `microservices/messenger/REMEDIATION-NOTES-2026-05-21.md`
- `microservices/messenger/iac/oyatie-public-cloud/main.tf`
- `microservices/messenger/iac/oyatie-public-cloud/grafana-rbac.tf`
- `microservices/messenger/iac/oyatie-public-cloud/versions.tf`
- `microservices/messenger/iac/oyatie-public-cloud/README.md`
- `microservices/messenger/iac/guest-on-aws/main.tf`
- `microservices/messenger/iac/guest-on-aws/versions.tf`
- `microservices/messenger/iac/guest-on-aws/README.md`
- `microservices/messenger/iac/guest-on-oci/main.tf`
- `microservices/messenger/iac/guest-on-oci/versions.tf`
- `microservices/messenger/iac/guest-on-oci/README.md`
- `microservices/messenger/iac/on-prem/main.tf`
- `microservices/messenger/iac/on-prem/versions.tf`
- `microservices/messenger/iac/on-prem/README.md`
- `microservices/messenger/iac/colo/main.tf`
- `microservices/messenger/iac/colo/versions.tf`
- `microservices/messenger/iac/colo/README.md`
- `microservices/messenger/iac/oyatie-as-cloud-provider/main.tf`
- `microservices/messenger/iac/oyatie-as-cloud-provider/versions.tf`
- `microservices/messenger/iac/oyatie-as-cloud-provider/README.md`
- `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`
- `microservices/messenger/slos/mention-fanout.openslo.yaml`
- `microservices/messenger/slos/message-send-availability.openslo.yaml`
- `microservices/messenger/slos/message-send-latency.openslo.yaml`
- `microservices/messenger/slos/presence-propagation.openslo.yaml`
- `microservices/messenger/slos/read-receipt-fanout.openslo.yaml`
- `microservices/messenger/slos/search-latency.openslo.yaml`
- `microservices/messenger/slos/voice-video-call-quality.openslo.yaml`
- `microservices/messenger/slos/voice-video-call-setup.openslo.yaml`
- `microservices/messenger/slos/websocket-fanout-latency.openslo.yaml`

## Tenant_class adoption checklist

- **Eligibility:** `manifest.json` and `PRD.md` declare `tenant_class_eligibility = ["demo_trial", "paid"]`.
- **Paid billing components:** `manifest.json`, `PRD.md`, and each deployment-context module use `paid_billing_components_emitted = ["per_seat", "per_usage"]`.
- **Principal claim binding:** `PRD.md`, `manifest.json`, and `policy/tenant-class.cedar` bind messenger authorization to `principal.tenant_class`, `principal.billing_components`, `principal.cap_breached`, and `principal.demo_trial_expires_at` from the shared cloud-iam session.
- **Demo trial caps:** `PRD.md`, `manifest.json`, and `policy/tenant-class.cedar` cover message volume, active channel count, message retention, huddle minutes, and attachment storage caps. The cap behavior preserves reads and denies write-heavy paths until conversion or grace-window recovery.
- **Paid Cedar gate:** `policy/tenant-class.cedar` makes compliance-pack activation, HSM escrow, work-mode MLS recovery escrow, BYOK, Slack pairing, and retention overrides paid-only. Compliance pack activation requires `tenant_class == "paid"` and `per_seat`; metered write-heavy operations require `per_usage` for paid tenants.
- **SLO surface:** All messenger OpenSLO files now label `tenant_class: demo_trial,paid`, expose eligibility/billing/principal/cap/gate references, and split best-effort demo objectives from contractual paid objectives.

## Mobile-app-bundle coordination summary

- Messenger, mail, social, and community remain separate backend microservices but ship as one client binary per platform.
- Cross-pane handoffs are documented as direct gRPC typed intents carrying tenant scope, `principal.tenant_class`, data-class labels, and audit-chain correlation IDs.
- Auth is one shared cloud-iam session across all panes; messenger must not mint pane-local identity.
- Push notifications use one APNs/FCM/WNS/Web Push envelope with pane classification, deduplication, and messenger sealed-sender constraints.
- Forbidden anti-patterns are explicit: LinkedIn-style engagement feed, influencer monetization, sponsored-post promotion, and metric-chasing social graph behavior.
- Client implementation targets are explicit: Swift for iOS/macOS, Kotlin for Android, WinUI 3 for Windows, and Leptos SSR plus selective hydration for web.

## Deferrals and scope notes

- No commits were made.
- Writes were kept under `microservices/messenger/`.
- ADR-MSG-001 and MLS-specific artifacts were not edited. MLS language added here is limited to tenant-class policy gates and push-envelope constraints already required by messenger compliance semantics.
- Full `tofu init` / provider-backed `tofu validate` was not run because this pass did not have deployment credentials or a provider-install requirement. Formatting and module-file completeness were verified locally.
- Cedar syntax validation was not run because no `cedar` CLI is installed in this workspace. The new policy follows existing repository Cedar idioms (`action in [...]`, `principal has ...`, and `.contains(...)`).

## Verification evidence

- `find microservices/messenger/iac -maxdepth 2 -type f | sort` showed every required deployment context with `main.tf`, `versions.tf`, and `README.md`.
- `test ! -d microservices/messenger/iac/terraform` passed.
- `jq empty microservices/messenger/manifest.json` passed.
- `tofu fmt -check -recursive microservices/messenger/iac` passed.
- `yq e 'true' microservices/messenger/slos/*.openslo.yaml >/dev/null` passed.
- `git diff --check -- microservices/messenger` passed.
- `git diff --name-only -- microservices/messenger | rg -i "ADR-MSG-001|mls"` returned no paths.

<!-- COMPLETION_REPORT: WAVE_15A_MESSENGER_FINALIZER status=complete findings=F-MSGR-001,F-MSGR-002,F-MSGR-003 scope=microservices/messenger no_commits=true mls_artifacts_touched=false -->

## Wave 15-IP-substance scrub (2026-05-21)

Assigned bucket: IP-BUCKET-H.

Scope interpreted as the `messenger` µservice's stamped short core IPs, not already-long journey IPs or already
substantive core slices.
Detection used line clustering, repeated heading inspection, and source grounding against `PRD.md`, `ARCHITECTURE.md`,
`manifest.json`, `competitor-parity-matrix.md`, policies, OpenSLO files, OpenAPI/AsyncAPI/proto contracts, IaC, and
runbooks.

Rewritten in place with bespoke Wave 15 substance sections:

- `IP-002-cargo-workspace-bootstrap.md`
- `IP-005-message-stream-kernel-domain.md`
- `IP-006-message-stream-adapters.md`
- `IP-007-presence-bc.md`
- `IP-008-file-attachment-bc.md`
- `IP-009-thread-tree-and-mention-router.md`
- `IP-010-read-receipt-tracker.md`
- `IP-011-rest-api-surface.md`
- `IP-012-websocket-frame-protocol.md`
- `IP-013-search-and-cedar-filter.md`
- `IP-014-huddles-livekit-signaling.md`
- `IP-015-hg-messenger-registration-and-branch-protection.md`

Preserved as already-substantive or outside the stamped 55-line core signature:

- `IP-001-iac-bootstrap.md`
- `IP-003-channel-store-kernel-domain.md`
- `IP-004-channel-store-adapter-postgres.md`
- `IP-NEW-hyperscaler-metric-emission.md`
- all `IP-journey-*.md` files.

Deleted as duplicative: none.

Counterpart references added across rewritten IPs include Slack, Microsoft Teams, Discord, Matrix/Element, Mattermost,
WhatsApp, Vertex-style contract comparison where relevant, and self-hosted messenger/product counterparts where the
IP scope demanded them.

Follow-up: preserved non-stamped IPs and journey IPs can still fail the broad grep counterpart-keyword command even
though they were not rewritten in this pass.

## Wave 15-journey-IP substance pass

Assigned µservice: messenger.

Inventory found 47 `microservices/messenger/IP-*.md` files over 200 lines. Template-loop detection found two journey
IPs with the stamped 68-deliverable pattern:

- `IP-journey-j105-cross-tenant-dm-boundary.md`
- `IP-journey-j113-cross-tenant-dm-boundary.md`

Both files were rewritten from numbered template loops into bespoke `Substantive journey rows` tables. The new rows cite
real messenger surfaces: `contracts/openapi/messenger.yaml`, `contracts/asyncapi/messenger-events.yaml`,
`contracts/proto/messenger.proto`, `policy/tenant-scope.cedar`, `policy/channel-scope.cedar`,
`policy/personal-dm-scope.cedar`, OpenSLO files, and the corresponding `docs/user-journeys/.../integration-test-plan.md`
artifacts.

Rows rewritten: 20 bespoke journey rows. Rows deleted as ungrounded: 116 stamped rows. Counterpart references added:
20 row-level references covering Slack Connect, Microsoft Teams cross-org/shared-channel/chat/eDiscovery flows, Slack
Workflow Builder, Teams Adaptive Cards, Microsoft Purview, and Slack/Teams archive/search/file-share equivalents.

Verification commands run:

```bash
for f in microservices/messenger/IP-*.md; do
  if [ $(wc -l < "$f") -gt 200 ]; then
    awk -F'|' 'NF>3 {print $2}' "$f" | sort | uniq -c | sort -rn | head -3
  fi
done
```

Follow-up: other long messenger journey IPs remain long because they did not contain the same stamped deliverable-loop
signature; future passes should target any non-messenger journey IPs that still contain repeated `IP row NNN` prose.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

- `microservices/messenger/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/messenger/IP-002-cargo-workspace-bootstrap.md`
- `microservices/messenger/IP-006-message-stream-adapters.md`
- `microservices/messenger/IP-007-presence-bc.md`
- `microservices/messenger/IP-010-read-receipt-tracker.md`
- `microservices/messenger/IP-journey-j01-emergency-911-dispatch-sender.md`
- `microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md`
- `microservices/messenger/PRD.md`
- `microservices/messenger/capacity-model.md`
- `microservices/messenger/catalog/oya-messenger-app.yaml`
- `microservices/messenger/catalog/oya-messenger-message-stream-adapter-valkey-streams.yaml`
- `microservices/messenger/catalog/oya-messenger-presence-adapter-valkey.yaml`
- `microservices/messenger/coherence-audit-2026-05-20.md`
- `microservices/messenger/iac/helm/messenger/templates/networkpolicy.yaml`
- `microservices/messenger/iac/helm/messenger/values.yaml`
- `microservices/messenger/manifest.json`
- `microservices/messenger/migration-from-connect.md`
- `microservices/messenger/policy/data-residency.md`
- `microservices/messenger/runbooks/presence-rebuild.md`
- `microservices/messenger/slos/message-send-availability.openslo.yaml`
- `microservices/messenger/slos/message-send-latency.openslo.yaml`
- `microservices/messenger/slos/presence-propagation.openslo.yaml`

Counterpart-fact preservations:

None.

Files renamed (git mv):

- `microservices/messenger/catalog/oya-messenger-message-stream-adapter-redis-streams.yaml` -> `microservices/messenger/catalog/oya-messenger-message-stream-adapter-valkey-streams.yaml`
- `microservices/messenger/catalog/oya-messenger-presence-adapter-redis.yaml` -> `microservices/messenger/catalog/oya-messenger-presence-adapter-valkey.yaml`

## Wave 15-doctrine-propagation-IPs (2026-05-21)

D4-BUCKET-1 trigger-based IP doctrine propagation.

- Root IPs scanned: 63
- Trigger A additions: 29
- Trigger B additions: 36
- Trigger C additions: 42
- Trigger D additions: 2
- Root IPs unmatched: 6
- Doctrine sources: ADR-0338, ADR-0342, ADR-0343, ADR-0344, ADR-0345; `specs/compliance-pack-floors.json`.
- Idempotence: skipped any IP section that already existed; no unmatched root IPs were edited.

IP-by-IP changes:
- `microservices/messenger/IP-001-iac-bootstrap.md`: added DR posture.
- `microservices/messenger/IP-007-presence-bc.md`: added DR posture.
- `microservices/messenger/IP-008-file-attachment-bc.md`: added DR posture.
- `microservices/messenger/IP-009-thread-tree-and-mention-router.md`: added DR posture.
- `microservices/messenger/IP-010-read-receipt-tracker.md`: added DR posture.
- `microservices/messenger/IP-011-rest-api-surface.md`: added API Versioning, DR posture.
- `microservices/messenger/IP-012-websocket-frame-protocol.md`: added API Versioning.
- `microservices/messenger/IP-013-search-and-cedar-filter.md`: added DR posture.
- `microservices/messenger/IP-014-huddles-livekit-signaling.md`: added DR posture.
- `microservices/messenger/IP-NEW-hyperscaler-metric-emission.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j01-emergency-911-dispatch-sender.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j03-crisis-chat-channel.md`: added API Versioning, Sustainability emission.
- `microservices/messenger/IP-journey-j04-silent-safe-channel.md`: added API Versioning, Sustainability emission.
- `microservices/messenger/IP-journey-j06-blind-reply-channel.md`: added API Versioning, Sustainability emission.
- `microservices/messenger/IP-journey-j08-trusted-contact-alert.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j09-trusted-contact-challenge.md`: added API Versioning, Sustainability emission.
- `microservices/messenger/IP-journey-j10-safe-channel-warning.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j100-pack-rollout-first-action.md`: added Sustainability emission.
- `microservices/messenger/IP-journey-j105-cross-tenant-dm-boundary.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j11-store-and-forward-queue.md`: added API Versioning, Sustainability emission.
- `microservices/messenger/IP-journey-j113-cross-tenant-dm-boundary.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j117-ops-war-room.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j123-launch-war-room.md`: added API Versioning, DR posture.
- `microservices/messenger/IP-journey-j124-emergency-war-room.md`: added API Versioning.
- `microservices/messenger/IP-journey-j127-thread-archive-on-leaver.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j130-thread-extract-for-whistleblower.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j133-manager-rif-1on1-dm.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j135-work-dm-investigation-read.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j14-read-scope-summarization.md`: added API Versioning, Sustainability emission.
- `microservices/messenger/IP-journey-j141-internal-audit-personal-tenant-boundary-deny-by-default.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j142-work-messenger-demotion.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j147-cohort-dm-anti-fraud.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j17-metadata-minimized-dm.md`: added API Versioning, Sustainability emission.
- `microservices/messenger/IP-journey-j21-first-e2ee-dm.md`: added Sustainability emission.
- `microservices/messenger/IP-journey-j34-work-channel-membership.md`: added Sustainability emission.
- `microservices/messenger/IP-journey-j49-omnichannel-thread.md`: added DR posture, Pod runtime tier.
- `microservices/messenger/IP-journey-j53-support-thread.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j55-support-thread.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j57-team-channel.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j59-channel-deprovision.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j65-message-export.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j69-message-triage.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j71-user-alert.md`: added API Versioning, DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j74-plugin-channel-actions.md`: added API Versioning, DR posture, Sustainability emission, Pod runtime tier.
- `microservices/messenger/IP-journey-j76-message-surface.md`: added API Versioning.
- `microservices/messenger/IP-journey-j85-message-surface.md`: added API Versioning.
- `microservices/messenger/IP-journey-j89-message-surface.md`: added API Versioning.
- `microservices/messenger/IP-journey-j91-us-msb-mtl-overlay.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j92-br-lgpd-us-parent-dsar.md`: added Sustainability emission.
- `microservices/messenger/IP-journey-j93-in-dpdpa-rbi-overlay.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j94-sox404-public-company-controls.md`: added DR posture, Sustainability emission.
- `microservices/messenger/IP-journey-j95-iso27001-soc2-annual-audit.md`: added Sustainability emission.
- `microservices/messenger/IP-journey-j96-ksa-uae-mena-onboarding.md`: added Sustainability emission.
- `microservices/messenger/IP-journey-j97-sg-pdpa-mas-tenant.md`: added Sustainability emission.
- `microservices/messenger/IP-journey-j98-au-privacy-apra-cps234.md`: added Sustainability emission.
- `microservices/messenger/IP-journey-j99-multi-pack-conflict-resolution.md`: added Sustainability emission.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set PRD target to manifest `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, and `runbooks/dr-failover.md` per ADR-0343, with HIPAA/KR-PIPA/SOC2/ISO27001/KR-CSAP floors cited. Rejected leaving only the old SLO paragraph because it did not name failover evidence or pack floors. Cost: active-active message/control replication, DR runbook ownership, and higher network/storage spend.
- Capacity model: declared manifest `0.20 vCPU`, `384 MiB RAM`, `8 GiB storage`, Valkey/Postgres/outbound baselines, `per_message` scaling, Tier-1 cell placement, `pod_runtime_tier=1`, and `3..80` replica bounds per ADR-0340. Rejected a pure WebSocket-count model because message fan-out and huddle paths scale on different dimensions. Cost: reserved idle socket capacity and shard-split automation.
- Sustainability + cost attribution: required `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on message, attachment, huddle, search, moderation, and notification audit rows per ADR-0344. Rejected aggregate-only finops tagging because CSRD/SB-253/SEC climate disclosures need tenant/capability/provider dimensions. Cost: per-call metering, finops rollup reconciliation, and carbon-routing guardrails.
- API versioning posture: adopted `YYYY-MM-DD` carrier triplet plus SDK semver, last 3 versions for at least 180 days, tenant pinning, and ADR-0145 internal-mesh exemption per ADR-0342. Rejected SDK-semver-only versioning because WebSocket, REST, proto, and federation consumers need a common public carrier. Cost: compatibility test matrix and per-tenant version pin storage.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345. ADR-0337 was not added because the PRD evidence shows ClickHouse analytics, not an Iceberg data-warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.2 vCPU, 384 MiB RAM, 8 GB storage per active tenant; connections valkey=6, postgres=4, outbound_http=6; scaling_dimension=per_message; cell_placement_class=Tier-1.
- ADR: ADR-0340 capacity declaration plus ADR-0248 cellular class.
- Rejected: template-stamped values copied from another service; messenger is declared Tier-1 because MLS/key-recovery and huddle signaling are tenant-data-plane surfaces; the rejected Tier-2 alternative would place recovery-sensitive work on the ordinary app pool.
- Cost: cell sizing and autoscaler budgets must reserve this per-tenant baseline before admitting more tenants.

### Block 2: dr
- Values: rto_p99_seconds=3600, rpo_p99_seconds=300, multi_region_active_active=true, backup_substrate=valkey_cluster, postgres_wal_g, object_storage_versioned, openbao_seal_unseal, audit_chain_merkle_seal, failover_runbook=runbooks/dr-failover.md.
- ADR: ADR-0343 plus compliance-pack floors; HIPAA/us-healthcare floors drive the 1h/5m baseline where applicable.
- Rejected: looser 24h PCI-only recovery because this service can serve healthcare or sensitive tenant workflows.
- Cost: warm cross-region replication and quarterly drill evidence are required for the declared runbook.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=microservices/messenger/PRD.md, microservices/messenger/IP-007-presence-bc.md, microservices/messenger/IP-014-huddles-livekit-signaling.md, microservices/messenger/runbooks/e2e-encryption-key-rotation.md.
- ADR: ADR-0338 runtime tiering; ADR-0340/ADR-0248 co-variance with cell_placement_class=Tier-1.
- Rejected: weaker runtime class that would contradict the documented tenant-data or first-party-app surface.
- Cost: runtime placement, nodepool capacity, and incident severity inherit this tier.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; supports_per_tenant_pinning=true.
- ADR: ADR-0342 date-versioned public APIs with per-tenant pinning.
- Rejected: internal-only exemption because this service has public OpenAPI, AsyncAPI, and proto surfaces.
- Cost: at least three supported public API windows and migration docs for any future breaking change.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=valkey, postgresql, cedar, openbao, kafka, opentelemetry; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 and /specs/oss-stewardship-registry.json registry authority.
- Rejected: local stewardship overrides because the registry default class is sufficient for each declared upstream.
- Cost: SBOM and CVE-response evidence must trace this service to each upstream owner team.

### Block 6: iac_module_invocations
- Values: aws-guest/k8s-namespace-bootstrap@v1, aws-guest/secrets-bootstrap@v1, oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1, on-prem/k8s-namespace-bootstrap@v1, on-prem/secrets-bootstrap@v1, colo/k8s-namespace-bootstrap@v1, colo/secrets-bootstrap@v1, oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1.
- ADR: ADR-0339 shared IaC module library.
- Rejected: unpinned local wrapper-only IaC because module reuse and pinning are the admission surface.
- Cost: module pins must be advanced deliberately when cloud-iac publishes new primitives.
