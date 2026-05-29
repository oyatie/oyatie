# IP-010: projection-gateway-adapter-pulsar — topic minting + tenant-aware ACL

- Bounded context: projection-gateway
- Layers: adapter (Pulsar), api, app
- Crates:
  - `oya-consent-graph-projection-gateway-adapter`
  - `oya-consent-graph-projection-gateway-adapter-pulsar`
  - `oya-consent-graph-projection-gateway-api`
  - `oya-consent-graph-projection-gateway-app`
- Acceptance status: ga
- Authority: ADR-0214 §2.5, ADR-SVC-CG-004 (grantor-region topic ownership), ADR-0078 (Pulsar substrate).
- Depends on: `oya-consent-graph-projection-gateway-{kernel, domain}`, `pulsar-rs = "6"`,
  `oya-consent-graph-agreement-sdk`, `openbao-rs = "0.8"`.

## 1. Goal

Implement the Pulsar-backed `ProjectionMinter`: at agreement-acceptance, mint a per-(grantor, grantee,
entity) projection topic in the **grantor's** Pulsar cluster with tenant-aware ACLs that allow only
the grantee to subscribe. On revocation/expiration, destroy the topic and revoke the grantee's token.

## 2. Topic naming

```
oya.consent-graph.projection.v1.<grantor_short>.<grantee_short>.<entity_short>.<agreement_short>
```

Where:
- `grantor_short` = grantor tenant-id truncated to first 8 chars (collision-safe per ULID prefix).
- `grantee_short` = grantee tenant-id ditto.
- `entity_short` = entity_type lowercased + sanitized (e.g., `finishedgoodsinventory` → `fgi`).
- `agreement_short` = agreement_id first 8 chars.

Example:
```
oya.consent-graph.projection.v1.tn-acme.tn-retail.fgi.01HXYZ12
```

This name is reversible to (grantor, grantee, entity, agreement) for ops debugging via a deterministic
local lookup table.

## 3. Pulsar tenancy model

Each oyatie tenant maps to a Pulsar tenant in their grantor-region cluster:
- Pulsar tenant: `oya-tenant-<grantor_tenant_id>`
- Pulsar namespace: `oya-tenant-<grantor_tenant_id>/cross-tenant-projection`
- Topic: `<namespace>/<topic-name>`

The grantee receives a JWT token bound to:
- Issuer: grantor-region OpenBao-signed key (per ADR-0072).
- Audience: this specific topic.
- Subject: grantee tenant + (optionally) grantee principal.
- Expiry: agreement.expiration (or 24h auto-renew, whichever sooner).
- Claim: `oya:agreement_id = <agreement_id>`.

Pulsar's authentication plugin verifies the JWT signature + audience + expiry; authorization plugin
verifies the `oya:agreement_id` claim matches a known active agreement (queried via cached SDK call
to consent-graph).

## 4. Minting algorithm

```rust
async fn mint(&self, agreement: &DataSharingAgreement) -> Result<ProjectionTopic, MintError> {
    // 4.1: sovereignty pre-check
    let topic_region = agreement.sovereignty.grantor_region;
    let pulsar_admin = self.pulsar_admin_for_region(topic_region)?;

    // 4.2: idempotent tenant/namespace
    pulsar_admin.create_tenant_if_absent(&format!("oya-tenant-{}", agreement.grantor)).await?;
    pulsar_admin.create_namespace_if_absent(&format!("oya-tenant-{}/cross-tenant-projection", agreement.grantor)).await?;

    // 4.3: topic
    let topic_name = build_topic_name(agreement);
    pulsar_admin.create_partitioned_topic(&topic_name, partition_count_for(agreement.terms.mode)).await?;

    // 4.4: namespace policies
    pulsar_admin.set_retention(&namespace, Retention { time_minutes: 60 * 24 * 7, size_mb: 10_000 }).await?;  // 7d, 10GB
    pulsar_admin.set_compaction_threshold(&namespace, 100_000_000).await?;  // 100MB triggers compaction

    // 4.5: ACL — grantee only on Subscribe
    pulsar_admin.grant_permission(
        &topic_name,
        &format!("oya-tenant-{}", agreement.grantee),
        vec![PulsarAction::Consume],   // explicit: no Produce
    ).await?;

    // 4.6: mint JWT token, store ref in OpenBao
    let token = self.jwt_issuer.mint_subscribe_token(agreement, &topic_name).await?;
    self.openbao.write(&format!("secret/consent-graph/projection-token/{}", agreement.agreement_id),
        SecretValue::Token(token)).await?;

    // 4.7: assert sovereignty invariant before returning
    let topic = ProjectionTopic { topic_id, topic_name, region: topic_region, agreement_id: agreement.agreement_id, ... };
    assert_grantor_region(&topic, agreement.sovereignty.grantor_region)?;

    // 4.8: emit audit
    self.audit_bridge.emit_projection_mint(&topic, &agreement).await?;
    Ok(topic)
}
```

## 5. Token delivery to grantee

The mint returns a `ProjectionTopic` with topic-name and metadata; the grantee fetches the JWT token
via `partner-directory-rest::get_projection_credentials(agreement_id)` (mTLS-authenticated). The token
is short-lived (24h); the SDK auto-renews via a refresh call.

## 6. Partition count heuristic

| Mode | Default partitions | Rationale |
|------|--------------------|-----------|
| Projection | 16 | row-level events, ordered by `entity_id` partition key |
| Aggregate | 4 | aggregate events are fewer + chunkier |
| AttestedQuery | 4 | low-volume request/response pattern |

Partitions may be re-sized via `pulsar-admin topics update-partitioned-topic`; consent-graph monitors
`pulsar_topic_msg_rate` and warns at 80% partition saturation.

## 7. Destruction

On revocation/expiration:
```rust
async fn destroy(&self, topic_id: ProjectionTopicId) -> Result<(), DestroyError> {
    let topic = self.repo.read(topic_id)?;
    let topic_name = &topic.topic_name;
    let pulsar_admin = self.pulsar_admin_for_region(topic.region)?;

    // 7.1: revoke grantee permission (immediate effect)
    pulsar_admin.revoke_permission(topic_name, &format!("oya-tenant-{}", topic.grantee)).await?;

    // 7.2: revoke JWT token in OpenBao (rotate signing-key-version invalidates all tokens, but per-token
    //      revocation is faster; we maintain a per-agreement revocation list checked by Pulsar auth plugin)
    self.jwt_issuer.add_to_revocation_list(topic.agreement_id).await?;

    // 7.3: stop any in-flight consumers
    pulsar_admin.unload_topic(topic_name).await?;   // forces clients to reconnect; reconnection denied by step 7.1

    // 7.4: schedule topic deletion (grace period 1h for audit-chain inspection)
    self.schedule_delete(topic_name, Duration::from_secs(3600)).await?;

    // 7.5: emit audit
    self.audit_bridge.emit_projection_destroy(topic_id).await?;
    Ok(())
}
```

Steps 7.1 + 7.3 give us ≤500ms wallclock to "grantee can no longer read" once revocation event
arrives at projection-gateway-app. Combined with the IP-008 1s revocation propagation budget, the
total time-to-lose-access is ≤1.5s p99.

## 8. Cross-region considerations

The grantor's Pulsar cluster lives in the grantor's region. The grantee subscribes to that cluster
*from* the grantee's region — i.e., a cross-region Pulsar consumer. Pulsar supports this natively
(`messageRoutingMode=RoundRobin` + cross-region TLS). Latency cost: grantee's read p99 includes the
cross-region round-trip.

For latency-critical use cases, grantor may *opt-in* to geo-replication of the topic to the grantee's
region; the sovereignty contract requires this is a per-agreement explicit decision (a
`SharingTerms::geo_replicate_to_grantee_region: bool` flag — defaults to false). When true:
- Pulsar geo-replicates the topic to grantee region (one-way; grantee region cannot publish back).
- Sovereignty invariant: replicated topic is *read-only in grantee region*; grantor-region remains
  the authoritative source.
- Pack overlay rules may forbid geo-replication entirely (KR PIPA strict-residency pack: forbidden).

## 9. Tests

- `mint_creates_tenant_namespace_topic_idempotent` — re-mint is no-op.
- `mint_acl_grantee_consume_only` — verify Pulsar permission list.
- `mint_topic_in_grantor_region_only` — mint with wrong region → sovereignty invariant violation.
- `destroy_revokes_permission_and_unloads` — destroy + immediate consumer reconnect denied.
- `geo_replicate_when_explicit` — flag=true triggers georep; flag=false does not.
- `partition_count_default_by_mode` — 16/4/4 per mode.
- `jwt_token_expiry_matches_agreement` — token expiry = min(agreement.expiration, now+24h).

## 10. App composition

`projection-gateway-app` wires:
- gRPC service (port 9445) for `mint`/`destroy`/`set_acl`.
- Pulsar admin clients per region (configured in helm values).
- OpenBao client for token store.
- Audit-bridge SDK.
- Revocation subscriber (subscribes to revocation topic; on event, runs `destroy`).
- Health probes: requires Pulsar admin connection healthy in all configured regions.

## 11. Verification

- `cargo build` + `cargo test` clean.
- Integration test against Pulsar dev cluster: mint → consumer subscribes successfully; revoke →
  consumer reads denied within 500ms.
- Sovereignty check: try to mint topic in non-grantor region → returns invariant violation, never
  reaches Pulsar.

## 12. Risk

- **R**: Pulsar admin API rate-limited.
  **M**: Mint is one-time per agreement-accept (≤100K/day at peak); cached admin client per region.
- **R**: JWT token leak.
  **M**: 24h expiry + per-agreement revocation list + audit-chain emission on every token issuance.
- **R**: Geo-replication misconfigured → topic accidentally replicated cross-region.
  **M**: `mint` reads agreement's `geo_replicate_to_grantee_region` flag, refuses to set replication
  if false; sovereignty audit job nightly checks Pulsar's replication-cluster-set matches agreement
  config.
- **R**: Pulsar cluster outage in grantor region.
  **M**: Active-active across 3 AZs; ADR-0078 mandates regional HA. Single-region outage means
  cross-tenant reads from that grantor halt — failure mode is correct (deny-by-default).

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: Snowflake/Databricks mint share access inside their own control planes; Cookiebot/OneTrust/TrustArc do not mint real-time projection topics. This IP's service-specific substance is tenant-aware Pulsar topic naming, ACL token delivery, destruction, and sovereignty-safe replication rather than generic data-share permissioning.
