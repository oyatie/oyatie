# Performance Benchmark Numbers - 2026-05-20

Audit target: `microservices/connector/`.
Counterpart benchmark set: Twilio, Sendbird, Stream.
Target model: one industry-leader-grade target set, with deployment-context overlays and tenant_class overlays.
No capability-tier rows or headings are used in this report.

Five-citation anchor block:
1. Local PRD performance and scale targets: `microservices/connector/PRD.md:143-158`.
2. Local capacity model and worker scaling assumptions: `microservices/connector/capacity-model.md:15-43`.
3. Canonical six deployment contexts, OpenTofu, OS, language, and OCI profile controls: `specs/master-plan-sequencing.json:704-868`.
4. Twilio public limits source: `https://www.twilio.com/docs/conversations-classic/conversations-limits`.
5. Sendbird and Stream public limits sources: `https://sendbird.com/docs/chat/platform-api/v3/rate-limits/core-api-rate-limits`, `https://sendbird.com/docs/chat/platform-api/v3/channel/channel-overview`, `https://getstream.io/chat/docs/node/rate_limits/`, `https://getstream.io/chat/docs/ios-swift/ios_channel_limits/`, and `https://getstream.io/chat/docs/node/webhooks_overview/`.

Methodology disclosure:
1. Public vendor limits are not identical to controlled benchmark results.
2. This report uses public limits as ceiling, default, or envelope evidence where vendors publish them.
3. Numbers marked `source` come directly from public vendor documentation or local Oyatie artifacts.
4. Numbers marked `estimated from` are deterministic conversions from a published unit, such as per-minute to per-second.
5. Numbers marked `Oyatie target` are target requirements for future implementation, not measured connect runtime results.
6. No runtime load test was run because `microservices/connector/src/`, `microservices/connector/tests/`, and canonical context IaC modules are absent.
7. The local target set therefore uses PRD targets and capacity-model assumptions as acceptance criteria.
8. All deployment-context overlays must be proven later with OpenTofu-deployed environments.
9. All tenant_class overlays must be proven later after `demo_trial`, `paid`, and `revenue_share` semantics exist in the service.

## §1 Methodology

1. Benchmark dimension: connector action latency.
2. Benchmark dimension: webhook acknowledgment latency.
3. Benchmark dimension: provider event to Oyatie workflow availability.
4. Benchmark dimension: OAuth or credential handoff latency.
5. Benchmark dimension: catalog search latency.
6. Benchmark dimension: outbound connector throughput.
7. Benchmark dimension: inbound webhook throughput.
8. Benchmark dimension: concurrent connection, channel, conversation, or participant scale where counterparts publish it.
9. Benchmark dimension: provider rate-limit envelope.
10. Benchmark dimension: retry, DLQ, and replay latency.
11. Benchmark dimension: attachment and payload size constraints where counterpart docs publish them.
12. Test workload W-01: catalog search for provider and capability metadata.
13. Test workload W-02: initiate credential or OAuth grant.
14. Test workload W-03: invoke outbound provider action.
15. Test workload W-04: receive signed webhook and acknowledge it.
16. Test workload W-05: normalize inbound event and publish it to downstream consumer.
17. Test workload W-06: reject invalid signature and quarantine event.
18. Test workload W-07: provider rate-limit response and backoff.
19. Test workload W-08: replay transient DLQ item.
20. Test workload W-09: rotate webhook secret.
21. Test workload W-10: catalog provider health canary.
22. OS disclosure: local service has no `supported-oses.json`, so OS-specific results are not available.
23. Architecture disclosure: canonical OS and architecture matrix must later follow ADR-0328 D-17 and `specs/master-plan-sequencing.json:777-816`.
24. Deployment-context disclosure: all six contexts are in scope unless later scoped down with evidence.
25. Context list: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
26. IaC disclosure: context-specific OpenTofu modules are absent, so context overlays are target constraints rather than measured results.
27. OCI disclosure: no `microservices/connector/iac/oci-guest/always-free/` profile exists, so demo_trial caps are target constraints.
28. Tenant_class disclosure: exact search found no `tenant_class`, `demo_trial`, or `revenue_share` in the service path.
29. Tenant_class target set: `demo_trial`, `paid`, and `revenue_share`.
30. Workload scale basis: PRD requires at least 500 connectors, 10M webhooks/day per tenant at p99, 100k concurrent OAuth grants, and 1M outbound connector actions/min, cited by `microservices/connector/PRD.md:156-158`.
31. Capacity basis: capacity model assumes 1,000 tenants at GA, 50,000 tenants at 24 months, and 100M webhooks/day platform-wide, cited by `microservices/connector/capacity-model.md:15-20`.
32. Worker basis: capacity model models 50 RPS per worker and 2,000 workers for 100k actions/sec, cited by `microservices/connector/capacity-model.md:25-27`.
33. Webhook basis: capacity model models 1,000 RPS per webhook receiver instance and 24 instances for peak webhook load, cited by `microservices/connector/capacity-model.md:31-33`.
34. SLO basis: webhook receiver OpenSLO target is 0.995, cited by `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml:4-39`.
35. SLO basis: connector availability OpenSLO target is 0.999, cited by `microservices/connector/slos/connector-availability.openslo.yaml:4-38`.
36. SLO basis: OAuth token health OpenSLO target is 0.995, cited by `microservices/connector/slos/oauth-token-health.openslo.yaml:4-38`.
37. SLO basis: DLQ overflow prevention OpenSLO target is 0.99, cited by `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml:4-38`.
38. Failure basis: failure modes document covers webhook retry, signature failure, adapter throttling, and DLQ failure, cited by `microservices/connector/failure-modes.md:32-74`.
39. Security basis: PRD requires no secret logging, signed webhooks, replay windows, and policy-gated connector actions, cited by `microservices/connector/PRD.md:184-188`.
40. Benchmark caveat: this report defines the numbers implementation must meet; it does not certify that current connect code meets them.

## §2 Counterpart Numbers

### §2.1 Twilio Public Numbers

1. Twilio number T-01: max participants per conversation is 1,000, source: Twilio Conversations limits.
2. Twilio number T-02: max non-chat participants per conversation is 50, source: Twilio Conversations limits.
3. Twilio number T-03: a user identity can participate in up to 1,000 conversations, source: Twilio Conversations limits.
4. Twilio number T-04: chat media size limit is 150 MB, source: Twilio Conversations limits.
5. Twilio number T-05: MMS media payload limit listed in source is 5 MB, source: Twilio Conversations limits.
6. Twilio number T-06: MMS attachments per message limit is 10, source: Twilio Conversations limits.
7. Twilio number T-07: default Conversations action rate is 30 actions per second, source: Twilio Conversations limits.
8. Twilio number T-08: concurrent connections per subaccount limit is 7,000, source: Twilio Conversations limits.
9. Twilio number T-09: concurrent connections per account limit is 100,000, source: Twilio Conversations limits.
10. Twilio number T-10: connectorion establishment rate per subaccount is 110 per second, source: Twilio Conversations limits.
11. Twilio number T-11: connectorion establishment rate per account is 1,000 per second, source: Twilio Conversations limits.
12. Twilio number T-12: upstream requests per connection limit is 500 per second, source: Twilio Conversations limits.
13. Twilio number T-13: upstream requests per subaccount limit is 20,000 per second, source: Twilio Conversations limits.
14. Twilio number T-14: short-code messaging throughput example is 100 messages per second, source: Twilio Messaging throughput guidance.
15. Interpretation: Twilio’s public numbers set a strong envelope for participants, media, concurrent connections, and request rates.
16. implication: Twilio adapter metadata must encode participant, media, sender, and request-rate limits rather than treating Twilio as generic webhook only.

### §2.2 Sendbird Public Numbers

1. Sendbird number S-01: core API GET rate limit for 100k MAU plan example is 600 requests per second, source: Sendbird core API rate limits.
2. Sendbird number S-02: core API POST rate limit for 100k MAU plan example is 200 requests per second, source: Sendbird core API rate limits.
3. Sendbird number S-03: core API PUT rate limit for 100k MAU plan example is 200 requests per second, source: Sendbird core API rate limits.
4. Sendbird number S-04: core API DELETE rate limit for 100k MAU plan example is 200 requests per second, source: Sendbird core API rate limits.
5. Sendbird number S-05: per-user message send limit is 5 requests per second, source: Sendbird per-user rate limits.
6. Sendbird number S-06: per-user channel create limit is 5 requests per second, source: Sendbird per-user rate limits.
7. Sendbird number S-07: per-user invite limit is 5 requests per second, source: Sendbird per-user rate limits.
8. Sendbird number S-08: per-user mark-as-read limit is 5 requests per second, source: Sendbird per-user rate limits.
9. Sendbird number S-09: shared-server open channel concurrent participant limit is 2,000, source: Sendbird channel overview.
10. Sendbird number S-10: classic group channel member limit is 100, source: Sendbird channel overview.
11. Sendbird number S-11: supergroup and dedicated server scaling can reach tens of thousands or higher by arrangement, source: Sendbird channel overview.
12. Sendbird number S-12: estimated sustained mutating API ceiling in 100k MAU example is one third of GET ceiling, estimated from 200 req/s versus 600 req/s public limits.
13. Interpretation: Sendbird’s envelope emphasizes API class rate limits, per-user limits, and channel member/participant scaling.
14. implication: Sendbird adapter metadata must encode both provider-global and per-user ceilings.

### §2.3 Stream Public Numbers

1. Stream number R-01: default `connector` endpoint rate limit example is 10,000 requests per minute at platform scope, source: Stream rate limits.
2. Stream number R-02: default `connector` endpoint rate limit example is 60 requests per minute at user scope, source: Stream rate limits.
3. Stream number R-03: estimated platform per-second rate for 10,000 requests per minute is about 333 requests per second, estimated from Stream rate limits.
4. Stream number R-04: estimated user per-second rate for 60 requests per minute is 2 requests per second under Stream’s per-second formula, estimated from Stream rate limits.
5. Stream number R-05: Stream describes per-second limit as the per-minute limit divided by 30, source: Stream rate limits.
6. Stream number R-06: message max length is 5,000 characters, source: Stream channel limits.
7. Stream number R-07: file attachment max size via Stream CDN is 100 MB, source: Stream channel limits.
8. Stream number R-08: unread count is only calculated for the first 2,000 channel members, source: Stream channel limits.
9. Stream number R-09: one push-notification behavior path applies only to the first 100 members, source: Stream channel limits.
10. Stream number R-10: Stream webhooks use HMAC SHA-256 verification, source: Stream webhooks overview.
11. Stream number R-11: Stream webhook events carry identifiers that can be used for duplicate handling, source: Stream webhooks overview.
12. Stream number R-12: Stream apps created after 2026-05-07 have compression enabled by default and compression can improve transfer times by 70-90 percent, source: Stream compression documentation.
13. Interpretation: Stream’s envelope emphasizes per-endpoint platform/user limits, message and attachment limits, and webhook verification.
14. implication: Stream adapter metadata must encode platform/user rates, member-count caveats, message size, attachment size, webhook HMAC, and dedupe identifiers.

## §3 Oyatie Target Numbers - Single Industry-Leader Target Set

1. Target O-01: connector action latency p50 <= 5 ms local overhead, Oyatie target from `microservices/connector/PRD.md:143-150`.
2. Target O-02: connector action latency p95 <= 20 ms local overhead, Oyatie target from `microservices/connector/PRD.md:143-150`.
3. Target O-03: connector action latency p99 <= 50 ms local overhead, Oyatie target from `microservices/connector/PRD.md:143-150`.
4. Target O-04: OAuth or credential handoff p50 <= 2 s, Oyatie target from `microservices/connector/PRD.md:143-150`.
5. Target O-05: OAuth or credential handoff p95 <= 8 s, Oyatie target from `microservices/connector/PRD.md:143-150`.
6. Target O-06: OAuth or credential handoff p99 <= 15 s, Oyatie target from `microservices/connector/PRD.md:143-150`.
7. Target O-07: webhook acknowledgment p50 <= 30 ms, Oyatie target from `microservices/connector/PRD.md:143-150`.
8. Target O-08: webhook acknowledgment p95 <= 100 ms, Oyatie target from `microservices/connector/PRD.md:143-150`.
9. Target O-09: webhook acknowledgment p99 <= 200 ms, Oyatie target from `microservices/connector/PRD.md:143-150`.
10. Target O-10: webhook-to-downstream availability p50 <= 200 ms, Oyatie target from `microservices/connector/PRD.md:143-150`.
11. Target O-11: webhook-to-downstream availability p95 <= 1 s, Oyatie target from `microservices/connector/PRD.md:143-150`.
12. Target O-12: webhook-to-downstream availability p99 <= 2 s, Oyatie target from `microservices/connector/PRD.md:143-150`.
13. Target O-13: connector catalog search p50 <= 50 ms, Oyatie target from `microservices/connector/PRD.md:143-150`.
14. Target O-14: connector catalog search p95 <= 150 ms, Oyatie target from `microservices/connector/PRD.md:143-150`.
15. Target O-15: connector catalog search p99 <= 300 ms, Oyatie target from `microservices/connector/PRD.md:143-150`.
16. Target O-16: catalog size >= 500 connectors, Oyatie target from `microservices/connector/PRD.md:156`.
17. Target O-17: per-tenant webhook capacity >= 10M webhooks/day at p99, Oyatie target from `microservices/connector/PRD.md:157`.
18. Target O-18: concurrent OAuth grants >= 100,000, Oyatie target from `microservices/connector/PRD.md:157`.
19. Target O-19: outbound connector action throughput >= 1M actions/min, Oyatie target from `microservices/connector/PRD.md:158`.
20. Target O-20: outbound connector action throughput >= 16,667 actions/sec, estimated from 1M actions/min PRD target.
21. Target O-21: platform-wide webhook capacity baseline >= 100M webhooks/day, source: `microservices/connector/capacity-model.md:15-20`.
22. Target O-22: webhook receiver instance capacity target >= 1,000 RPS per instance, source: `microservices/connector/capacity-model.md:31-33`.
23. Target O-23: peak webhook receiver fleet >= 24 instances for modeled load, source: `microservices/connector/capacity-model.md:31-33`.
24. Target O-24: connector worker target >= 50 RPS per worker, source: `microservices/connector/capacity-model.md:25-27`.
25. Target O-25: connector worker fleet target can scale to 2,000 workers for 100k actions/sec modeled load, source: `microservices/connector/capacity-model.md:25-27`.
26. Target O-26: connector availability SLO target >= 0.999, source: `microservices/connector/slos/connector-availability.openslo.yaml:4-38`.
27. Target O-27: webhook receiver throughput SLO target >= 0.995, source: `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml:4-39`.
28. Target O-28: OAuth token health SLO target >= 0.995, source: `microservices/connector/slos/oauth-token-health.openslo.yaml:4-38`.
29. Target O-29: DLQ overflow prevention SLO target >= 0.99, source: `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml:4-38`.
30. Target O-30: invalid signature reject path <= webhook p99 ack target of 200 ms, Oyatie target derived from security and webhook ack targets.
31. Target O-31: DLQ replay enqueue <= 1 s p50, <= 5 s p95, <= 10 s p99, source: `microservices/connector/PRD.md:143-150`.
32. Target O-32: provider rate-limit backoff must add no data loss and no duplicate downstream event under conformance tests, Oyatie target derived from failure modes and PRD replay requirements.
33. Target O-33: provider payload redaction must keep secret and PII fields out of logs, source: `microservices/connector/PRD.md:184-188` and `microservices/connector/dpia.md:60-69`.
34. Target O-34: connector catalog health canary p95 <= 150 ms for metadata read path, Oyatie target derived from catalog p95 target.
35. Target O-35: provider webhook health canary p95 <= 100 ms local ack path, Oyatie target derived from webhook p95 target.
36. Target O-36: provider action audit event emission p95 <= 20 ms local overhead, Oyatie target derived from connector action p95 target.
37. Target O-37: provider event audit event emission p95 <= 100 ms local ack path, Oyatie target derived from webhook p95 target.
38. Target O-38: event duplicate suppression false-negative target is zero under conformance fixtures, Oyatie target derived from replay security requirements.
39. Target O-39: connector credential secret exposure target is zero log events, source: `microservices/connector/PRD.md:184-188`.
40. Target O-40: all provider-specific limits must be catalog metadata, not hidden implementation constants.

### §3.1 Deployment-Context Overlay

1. Overlay D-01: `oyatie-public-cloud` should meet the full target set with elasticity guarantees once OpenTofu modules exist.
2. Overlay D-02: `guest-on-aws` should meet the full target set subject to customer account quotas and external-provider egress controls.
3. Overlay D-03: `guest-on-oci` should meet the full target set where paid capacity is available and should also include the OCI Always Free profile for demo_trial.
4. Overlay D-04: `on-prem` should meet latency targets inside facility constraints, with external-provider egress and compliance policies declared per facility.
5. Overlay D-05: `colo` should meet latency targets inside rack, carrier, and egress constraints declared per facility.
6. Overlay D-06: `oyatie-as-cloud-provider` should meet full target set through Oyatie-owned substrate and published provider capacity.
7. Overlay D-07: context modules must exist under `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/`.
8. Overlay D-08: OCI Always Free profile should be capped by 4 OCPU and 24 GB memory budget as stated in ADR-0328 D-19 and `specs/master-plan-sequencing.json:857-868`.
9. Overlay D-09: OCI Always Free profile should expose demo-scale connector catalog, OAuth, webhook, and DLQ paths without promising paid-scale throughput.
10. Overlay D-10: demo-profile throughput should be capped by documented OpenTofu variables, not by hidden service constants.
11. Overlay D-11: on-prem and colo throughput should be declared with facility-specific CPU, memory, storage, and egress variables.
12. Overlay D-12: all context overlays must preserve the same correctness and security targets even when throughput is capped.

### §3.2 Tenant_Class Overlay

1. Overlay C-01: `demo_trial` should use the same correctness and security bar with hard usage caps.
2. Overlay C-02: `demo_trial` should default to the OCI Always Free profile where feasible.
3. Overlay C-03: `demo_trial` should keep best-effort SLO language and no compliance-pack or BYOK entitlement.
4. Overlay C-04: `demo_trial` should cap provider actions/minute, active connector count, OAuth grant count, webhook endpoints, and DLQ storage.
5. Overlay C-05: `paid` should use per-seat plus usage-based billing and can scale across any supported deployment context.
6. Overlay C-06: `paid` should allow contractual SLO, compliance packs, and BYOK where the dependent services support them.
7. Overlay C-07: `paid` should scale toward the full target set by purchasing capacity rather than by changing feature quality.
8. Overlay C-08: `revenue_share` should run at-cost or zero-margin substrate with Oyatie taking a percentage of customer gross revenue.
9. Overlay C-09: `revenue_share` should expose the same connector quality but with cost attribution needed for revenue accounting.
10. Overlay C-10: all tenant classes should use the same industry-leader target set for correctness, security, and API semantics.
11. Overlay C-11: tenant_class should affect caps, entitlement, billing, and infrastructure profile, not baseline feature quality.
12. Overlay C-12: tenant_class values are not yet encoded in the current service path, so all overlay targets are adoption requirements.

## §4 Comparison Narrative

1. Connector action latency: Oyatie target p99 local overhead 50 ms is aggressive and appropriate because external-provider latency should be measured separately from local substrate overhead.
2. Twilio comparison: Twilio publishes action-rate and request-rate limits, not Oyatie-local overhead, so Oyatie should track local overhead and provider-wall-clock separately.
3. Sendbird comparison: Sendbird publishes API class rate limits; Oyatie should be at parity by modeling those ceilings and preventing local retry storms.
4. Stream comparison: Stream publishes endpoint and user rate limits; Oyatie should be at parity by encoding both platform and user ceilings in catalog metadata.
5. Webhook acknowledgment: Oyatie target p99 200 ms is strong for local receiver ack and should be preserved across contexts.
6. Twilio comparison: Twilio webhook delivery behavior requires fast receiver ack and duplicate tolerance; Oyatie target is appropriate if signature verification stays inside the ack budget.
7. Sendbird comparison: Sendbird webhook tracking requires receiver reliability; Oyatie target is parity if event normalization and DLQ enqueue do not slow ack.
8. Stream comparison: Stream webhooks use HMAC SHA-256, so Oyatie should keep HMAC verification inside p99 200 ms.
9. Catalog latency: Oyatie p99 300 ms is reasonable for provider catalog search and should be met from local indexed metadata.
10. Twilio comparison: Twilio’s product breadth makes catalog facets important; Oyatie should expose product-family search without provider API calls.
11. Sendbird comparison: Sendbird’s channel/user/message/moderation families require searchable capability tags.
12. Stream comparison: Stream’s endpoint/user limits require searchable rate-limit metadata.
13. Throughput: Oyatie PRD target of 1M actions/min is ahead of individual documented endpoint examples but must be proven by implementation and context IaC.
14. Twilio comparison: Twilio upstream request ceiling of 20,000/s per subaccount is higher than Oyatie’s 16,667/s PRD target when mapped to one high-capacity provider account.
15. Sendbird comparison: Sendbird 100k-MAU example mutating API ceiling of 200/s is much lower than Oyatie’s platform action target, so Oyatie must rate-limit per provider rather than push raw platform capacity into one provider.
16. Stream comparison: Stream default platform `connector` example of about 333/s is lower than Oyatie’s platform action target, so provider-specific backoff is required.
17. Concurrent scale: Oyatie target of 100k concurrent OAuth grants aligns with Twilio account-level concurrent connection magnitude but measures different object types.
18. Participant/member scale: Twilio 1,000 participants per conversation, Sendbird 2,000 shared open-channel participants, and Stream 2,000 unread-count caveat show provider-specific scale semantics must stay in catalog.
19. Attachment scale: Twilio 150 MB chat media and Stream 100 MB file attachment should become provider metadata and validation constraints.
20. Event normalization: Oyatie can be ahead if it normalizes provider events into one AsyncAPI surface without hiding provider-specific limits.
21. DLQ replay: Oyatie PRD replay targets give a good local acceptance bar; counterpart docs do not offer directly comparable DLQ targets.
22. OCI Always Free profile: Oyatie demo_trial profile will trail paid-scale counterpart envelopes by design, but must preserve correctness and security.
23. paid tenant class: paid tenants should scale to full target set subject to provider-side rate limits and purchased infrastructure.
24. revenue_share tenant class: revenue-share tenants should scale by at-cost economics, not by reduced connector semantics.
25. Overall: Oyatie is ahead in integrated policy, DLQ, and multi-context ambition; at parity target in webhook and catalog latency; catch-up in provider-specific catalog records, tests, and executable implementation; blocked on deployment/IaC proof.

## §5 Benchmark Acceptance Checklist

1. Acceptance A-01: provider catalog includes Twilio, Sendbird, and Stream records.
2. Acceptance A-02: provider rate-limit metadata includes platform, account, endpoint, and user scopes as applicable.
3. Acceptance A-03: provider webhook signature metadata includes algorithm and replay-window rules.
4. Acceptance A-04: provider identity metadata maps account, user, channel, conversation, participant, and message identifiers.
5. Acceptance A-05: connector action p99 local overhead load test proves <= 50 ms.
6. Acceptance A-06: webhook ack p99 load test proves <= 200 ms with signature verification enabled.
7. Acceptance A-07: catalog search p99 load test proves <= 300 ms with at least 500 connector records.
8. Acceptance A-08: OAuth or credential handoff p99 test proves <= 15 s.
9. Acceptance A-09: DLQ replay p99 proves <= 10 s for transient replay.
10. Acceptance A-10: invalid signature path proves reject plus quarantine inside p99 ack budget.
11. Acceptance A-11: provider rate-limit response proves backoff without duplicate downstream event.
12. Acceptance A-12: demo_trial profile caps are enforced by OpenTofu variables and service policy.
13. Acceptance A-13: paid class scales by capacity and billing, not by alternate feature semantics.
14. Acceptance A-14: revenue_share class emits cost and gross-revenue attribution hooks.
15. Acceptance A-15: all six deployment context modules can plan successfully with OpenTofu.
16. Acceptance A-16: OCI Always Free profile can plan under the canonical resource envelope.
17. Acceptance A-17: supported OS manifest exists and CI covers the declared Tier-1 set.
18. Acceptance A-18: Rust implementation exists for catalog, credential profile, webhook verification, connector action, and DLQ replay.
19. Acceptance A-19: Rust tests exist for Twilio, Sendbird, and Stream conformance fixtures.
20. Acceptance A-20: dashboards expose provider, tenant_class, deployment_context, action_family, and error_family dimensions.

## §6 Batch Conclusion

1. The strongest local numeric anchors are the PRD latency and scale targets in `microservices/connector/PRD.md:143-158`.
2. The strongest capacity anchors are `microservices/connector/capacity-model.md:15-43`.
3. Twilio sets high public ceilings for participants, media, connections, and upstream requests.
4. Sendbird sets clear API class and per-user rate-limit ceilings.
5. Stream sets clear endpoint/user rate-limit, message-size, attachment-size, and webhook-verification expectations.
6. Oyatie’s target set is credible if implemented as local substrate overhead plus provider-specific rate-limit enforcement.
7. Oyatie cannot claim measured parity yet because implementation source, tests, OpenTofu context modules, OS manifest, and tenant_class adoption are absent.
8. The correct next benchmark step is Rust conformance plus OpenTofu-deployed load tests, not another documentation-only number table.

## §7 First Measurement Plan

1. Measurement M-01: build a Rust catalog benchmark that loads at least 500 connector records and measures search p50, p95, and p99 against O-13 through O-15.
2. M-01 pass condition: p99 <= 300 ms with provider, action_family, auth_type, webhook_event, and rate-limit facets enabled.
3. M-01 evidence path: service-local Rust test output plus catalog fixture count.
4. Measurement M-02: build a Rust connector-action benchmark with mocked provider adapters and measure local overhead p50, p95, and p99 against O-01 through O-03.
5. M-02 pass condition: p99 <= 50 ms excluding external provider network time.
6. M-02 evidence path: benchmark output showing local serialization, policy decision point, adapter dispatch, audit emission, and queue handoff.
7. Measurement M-03: build webhook acknowledgment benchmark with signature verification enabled.
8. M-03 pass condition: p50 <= 30 ms, p95 <= 100 ms, and p99 <= 200 ms for valid signed webhook.
9. M-03 evidence path: benchmark output showing signature verify, replay-window check, enqueue, and HTTP ack.
10. Measurement M-04: build invalid-signature rejection benchmark.
11. M-04 pass condition: invalid signature rejects inside the same p99 200 ms webhook ack budget and emits quarantine evidence.
12. M-04 evidence path: benchmark output plus DLQ/quarantine event fixture.
13. Measurement M-05: build duplicate webhook event benchmark.
14. M-05 pass condition: duplicate event produces no duplicate downstream event under repeated delivery.
15. M-05 evidence path: event-id fixture and downstream event count.
16. Measurement M-06: build provider rate-limit benchmark for Twilio-style account and connection ceilings.
17. M-06 pass condition: local scheduler respects provider metadata and does not exceed configured account, sender, or connection ceilings.
18. M-06 evidence path: Twilio fixture based on public limits and scheduler output.
19. Measurement M-07: build provider rate-limit benchmark for Sendbird-style API method and per-user ceilings.
20. M-07 pass condition: GET, mutating method, and per-user limits are enforced independently.
21. M-07 evidence path: Sendbird fixture based on public limits and scheduler output.
22. Measurement M-08: build provider rate-limit benchmark for Stream-style endpoint, platform, and user ceilings.
23. M-08 pass condition: endpoint platform and user limits enforce both per-minute and per-second derived ceilings.
24. M-08 evidence path: Stream fixture based on public limits and scheduler output.
25. Measurement M-09: build DLQ replay benchmark.
26. M-09 pass condition: replay p50 <= 1 s, p95 <= 5 s, and p99 <= 10 s for transient failures.
27. M-09 evidence path: DLQ fixture, replay attempt log, and terminal event state.
28. Measurement M-10: build OAuth or credential handoff benchmark.
29. M-10 pass condition: p50 <= 2 s, p95 <= 8 s, and p99 <= 15 s for grant or credential-profile handoff.
30. M-10 evidence path: local handoff benchmark and cloud-secrets mock or integration fixture.
31. Measurement M-11: build provider payload redaction benchmark.
32. M-11 pass condition: secrets, tokens, and PII fields never appear in structured logs under success, failure, and DLQ paths.
33. M-11 evidence path: log capture scan and redaction fixture.
34. Measurement M-12: build deployment-context plan checks.
35. M-12 pass condition: all six OpenTofu context modules plan without drift once modules exist.
36. M-12 evidence path: `tofu plan` output captured per context.
37. Measurement M-13: build OCI Always Free profile plan check.
38. M-13 pass condition: demo_trial profile plans within CPU, memory, storage, load balancer, and database constraints from the canonical OCI profile.
39. M-13 evidence path: OpenTofu variables, plan output, and capacity cap documentation.
40. Measurement M-14: build tenant_class cap tests.
41. M-14 pass condition: demo_trial caps hard-stop usage, paid tenants scale by purchased capacity, and revenue_share tenants emit cost-accounting evidence.
42. M-14 evidence path: policy fixture, billing handoff event, and rate-limit counter output.
43. Measurement M-15: build OS support verification.
44. M-15 pass condition: service-local OS manifest exists and CI declares coverage for canonical supported OSes.
45. M-15 evidence path: `supported-oses.json` plus CI matrix output.
46. Measurement M-16: build provider canary checks.
47. M-16 pass condition: Twilio, Sendbird, and Stream canaries measure provider health without consuming tenant production quota.
48. M-16 evidence path: canary output with provider, tenant_class, deployment_context, and action_family labels.
49. Measurement M-17: build dashboard dimension checks.
50. M-17 pass condition: dashboards expose p50, p95, p99, error rate, rate-limit count, DLQ count, replay latency, and provider outage state.
51. M-17 evidence path: dashboard JSON or generated panel assertions.
52. Measurement M-18: build acceptance rollup.
53. M-18 pass condition: all local latency targets, provider metadata checks, deployment-context plan checks, and tenant_class cap checks pass in one repeatable Rust-driven verification command.
54. M-18 evidence path: final benchmark report generated by the Rust harness and checked into the connect evidence path.
55. Stop condition: measured parity can be claimed only after M-01 through M-18 have current evidence; until then this document remains a target-number report.
