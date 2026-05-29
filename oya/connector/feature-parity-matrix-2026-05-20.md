# Feature Parity Matrix - 2026-05-20

Audit target: `microservices/connector/`.
Counterpart union bar: Twilio, Sendbird, Stream.
Product interpretation: communications integration substrate, connector catalog, OAuth broker, webhook receiver, signature verification, payload canonicalization, connector adapter, data mapping, retry/DLQ, and downstream handoff discipline.
Non-goal: turning connect into a standalone chat, messaging, contact-center, or workflow product.
Primary local anchors: `microservices/connector/PRD.md:29-39`, `microservices/connector/README.md:16-32`, `microservices/connector/ARCHITECTURE.md:26-38`, and chat history `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8923-8926`.

## §1 Counterpart 1 Capability Surface - Twilio

1. Twilio anchor: programmable communications APIs across messaging, conversations, voice, email-adjacent integrations, phone-number identity, webhooks, usage controls, and delivery state.
2. Public source used: Twilio Conversations limits, `https://www.twilio.com/docs/conversations-classic/conversations-limits`.
3. Public source used: Twilio Messaging throughput guidance, `https://help.twilio.com/articles/115002943027`.
4. relevance: Twilio is an external connector provider, not a replacement for connect.
5. Local evidence: Twilio already appears as a connector catalog record at `microservices/connector/catalog/connectors/twilio.yaml`.
6. Local evidence: connector PRD requires at least 500 connectors, `microservices/connector/PRD.md:156`.
7. Local evidence: connector PRD requires connector action invocation, `microservices/connector/PRD.md:128-137`.
8. Local evidence: connector PRD requires OAuth grant and webhook endpoint substrate, `microservices/connector/PRD.md:128-137`.
9. Surface TWI-01: connector catalog listing for Twilio product families.
10. Surface TWI-02: per-product connector actions for message send, conversation create, participant add, participant remove, and message status read.
11. Surface TWI-03: phone number, sender identity, and messaging service metadata ingestion.
12. Surface TWI-04: webhook registration for inbound message events.
13. Surface TWI-05: webhook registration for delivery status events.
14. Surface TWI-06: webhook signature verification and replay-window enforcement.
15. Surface TWI-07: provider retry and duplicate event handling.
16. Surface TWI-08: outbound action idempotency keys.
17. Surface TWI-09: provider-side rate-limit metadata modeled in catalog schema.
18. Surface TWI-10: tenant-side rate-limit overlay modeled in connect policy.
19. Surface TWI-11: conversation participant limit metadata.
20. Surface TWI-12: connectorion limit metadata.
21. Surface TWI-13: sender throughput profile metadata.
22. Surface TWI-14: delivery receipt normalization into Oyatie events.
23. Surface TWI-15: error-code normalization into connector failure taxonomy.
24. Surface TWI-16: credential rotation workflow through cloud-secrets handoff.
25. Surface TWI-17: least-privilege OAuth or API-key scoping profile.
26. Surface TWI-18: per-tenant webhook endpoint isolation.
27. Surface TWI-19: DLQ quarantine for invalid signature events.
28. Surface TWI-20: DLQ replay for transient provider errors.
29. Surface TWI-21: observability labels for provider, account, sender, tenant, and action.
30. Surface TWI-22: policy-engine decision point before external send.
31. Surface TWI-23: policy-engine decision point before webhook fanout.
32. Surface TWI-24: PII redaction in connector logs.
33. Surface TWI-25: data-residency metadata for provider payload storage.
34. Surface TWI-26: marketplace publisher workflow for Twilio adapter certification.
35. Surface TWI-27: contract versioning for Twilio adapter schema.
36. Surface TWI-28: provider outage circuit breaker.
37. Surface TWI-29: provider latency SLO dashboard.
38. Surface TWI-30: provider delivery-success dashboard.
39. Surface TWI-31: provider cost attribution fields.
40. Surface TWI-32: tenant-class usage cap hooks.
41. Surface TWI-33: demo_trial cap enforcement for outbound sends.
42. Surface TWI-34: paid tenant scaling with usage billing.
43. Surface TWI-35: revenue_share at-cost substrate accounting.
44. Surface TWI-36: onboarding tutorial for Twilio connector configuration.
45. Surface TWI-37: incident runbook for Twilio delivery degradation.
46. Surface TWI-38: migration playbook from direct Twilio integration into connect substrate.
47. Surface TWI-39: conformance test for signature verification.
48. Surface TWI-40: conformance test for webhook duplicate suppression.
49. Surface TWI-41: conformance test for provider rate-limit backoff.
50. Surface TWI-42: conformance test for DLQ replay.
51. Surface TWI-43: Rust adapter trait implementation.
52. Surface TWI-44: no provider credential in logs.
53. Surface TWI-45: outbound action audit event.
54. Surface TWI-46: inbound event audit event.
55. Surface TWI-47: connector catalog searchable fields for Twilio use cases.
56. Surface TWI-48: contract examples for send, inbound, delivery, and failure paths.
57. Surface TWI-49: provider SLA caveat captured as external dependency.
58. Surface TWI-50: synthetic canary for endpoint health.
59. Current connect coverage: catalog record present, broad connector substrate present in docs, exact Twilio adapter implementation evidence absent.
60. Current gap: no source tree or tests prove Twilio adapter behavior, as no `src/` or `tests/` directory exists under `microservices/connector/`.

## §2 Counterpart 2 Capability Surface - Sendbird

1. Sendbird anchor: application chat API, channels, users, messages, moderation, announcements, webhooks, push, rate limits, and channel scale controls.
2. Public source used: Sendbird core API rate limits, `https://sendbird.com/docs/chat/platform-api/v3/rate-limits/core-api-rate-limits`.
3. Public source used: Sendbird per-user rate limits, `https://sendbird.com/docs/chat/platform-api/v3/rate-limits/per-user-rate-limits`.
4. Public source used: Sendbird channel overview, `https://sendbird.com/docs/chat/platform-api/v3/channel/channel-overview`.
5. relevance: Sendbird should be modeled as a connector and event provider where Oyatie products need chat-provider integration.
6. Local evidence: no Sendbird connector catalog record appears in the 182-file inventory.
7. Local evidence: connector catalog has communication-adjacent providers such as Discord, Slack, SendGrid, Mailgun, and Twilio.
8. Surface SEN-01: connector catalog listing for Sendbird application, channel, user, message, moderation, and webhook capabilities.
9. Surface SEN-02: provider authentication schema for Sendbird application API tokens.
10. Surface SEN-03: application-level metadata ingestion.
11. Surface SEN-04: user create/update/read adapter actions.
12. Surface SEN-05: channel create/update/read adapter actions.
13. Surface SEN-06: message send/read/delete adapter actions.
14. Surface SEN-07: message reaction adapter actions.
15. Surface SEN-08: membership invite/join/leave adapter actions.
16. Surface SEN-09: moderation action adapter actions.
17. Surface SEN-10: announcement action adapter actions.
18. Surface SEN-11: webhook registration metadata.
19. Surface SEN-12: webhook event normalization.
20. Surface SEN-13: webhook signature verification where provider supports it.
21. Surface SEN-14: duplicate event suppression.
22. Surface SEN-15: provider rate-limit metadata for GET, POST, PUT, and DELETE classes.
23. Surface SEN-16: per-user rate-limit metadata.
24. Surface SEN-17: backoff policy for provider throttling.
25. Surface SEN-18: open-channel participant scale metadata.
26. Surface SEN-19: group-channel member scale metadata.
27. Surface SEN-20: push notification event normalization.
28. Surface SEN-21: file and media attachment metadata normalization.
29. Surface SEN-22: moderation evidence redaction.
30. Surface SEN-23: data residency handoff if Sendbird region settings are involved.
31. Surface SEN-24: tenant policy gate for user creation.
32. Surface SEN-25: tenant policy gate for channel creation.
33. Surface SEN-26: tenant policy gate for announcement fanout.
34. Surface SEN-27: DLQ quarantine for invalid provider events.
35. Surface SEN-28: DLQ replay for transient webhook handling failure.
36. Surface SEN-29: observability labels for application, channel, user, tenant, action, and provider event.
37. Surface SEN-30: catalog search facets for chat use cases.
38. Surface SEN-31: marketplace adapter certification flow.
39. Surface SEN-32: versioned adapter contract examples.
40. Surface SEN-33: tenant-class cap hooks for chat operations.
41. Surface SEN-34: demo_trial cap enforcement for users, channels, and messages.
42. Surface SEN-35: paid tenant scaling through usage and seat billing.
43. Surface SEN-36: revenue_share accounting for embedded SaaS reseller chat operations.
44. Surface SEN-37: provider health canary.
45. Surface SEN-38: provider webhook-lag dashboard.
46. Surface SEN-39: provider error taxonomy mapping.
47. Surface SEN-40: runbook for Sendbird webhook delivery degradation.
48. Surface SEN-41: runbook for Sendbird API throttling.
49. Surface SEN-42: migration playbook from direct Sendbird integration.
50. Surface SEN-43: Rust adapter trait implementation.
51. Surface SEN-44: conformance test for rate-limit behavior.
52. Surface SEN-45: conformance test for channel and user identity mapping.
53. Surface SEN-46: conformance test for webhook replay.
54. Surface SEN-47: conformance test for DLQ recovery.
55. Surface SEN-48: audit event for outbound provider operation.
56. Surface SEN-49: audit event for inbound provider event.
57. Surface SEN-50: least-privilege credential rotation handoff.
58. Current connect coverage: generic substrate docs cover many needed primitives, but no Sendbird-specific catalog record, contract example, tests, or runbook exists.
59. Current gap: Sendbird union coverage is a new explicit add.
60. Severity implication: P2 feature parity gap until the catalog and adapter contract express Sendbird.

## §3 Counterpart 3 Capability Surface - Stream

1. Stream anchor: chat API, channels, users, messages, moderation, reactions, attachments, push, webhooks, connection/rate limits, and client event sync.
2. Public source used: Stream rate limits, `https://getstream.io/chat/docs/node/rate_limits/`.
3. Public source used: Stream webhooks overview, `https://getstream.io/chat/docs/node/webhooks_overview/`.
4. Public source used: Stream channel limits, `https://getstream.io/chat/docs/ios-swift/ios_channel_limits/`.
5. relevance: Stream should be modeled as a connector and event provider with explicit rate-limit and webhook contracts.
6. Local evidence: no Stream connector catalog record appears in the 182-file inventory.
7. Local evidence: connector webhook receiver and connector action surfaces exist in PRD and contracts, cited by `microservices/connector/PRD.md:128-137` and `microservices/connector/contracts/openapi/connector-integration.yaml:176-218`.
8. Surface STR-01: connector catalog listing for Stream chat application, channels, users, messages, moderation, and webhooks.
9. Surface STR-02: provider authentication schema for API key, secret, and token flows.
10. Surface STR-03: channel create/update/read adapter actions.
11. Surface STR-04: user create/update/read adapter actions.
12. Surface STR-05: message send/update/delete adapter actions.
13. Surface STR-06: reaction create/delete adapter actions.
14. Surface STR-07: moderation and flagging adapter actions.
15. Surface STR-08: attachment upload metadata normalization.
16. Surface STR-09: unread-count behavior metadata.
17. Surface STR-10: push-notification behavior metadata.
18. Surface STR-11: provider webhook registration metadata.
19. Surface STR-12: webhook HMAC verification.
20. Surface STR-13: webhook delivery duplicate handling.
21. Surface STR-14: provider rate-limit metadata by endpoint and platform/user scope.
22. Surface STR-15: per-tenant rate-limit overlay.
23. Surface STR-16: backoff behavior for rate-limit responses.
24. Surface STR-17: channel-member scale metadata.
25. Surface STR-18: max message size metadata.
26. Surface STR-19: file attachment size metadata.
27. Surface STR-20: channel type capability metadata.
28. Surface STR-21: tenant policy gate for channel creation.
29. Surface STR-22: tenant policy gate for message send.
30. Surface STR-23: tenant policy gate for moderation action.
31. Surface STR-24: data-residency caveat field for provider storage.
32. Surface STR-25: connector catalog facets for Stream use cases.
33. Surface STR-26: provider outage circuit breaker.
34. Surface STR-27: webhook lag dashboard.
35. Surface STR-28: provider error taxonomy mapping.
36. Surface STR-29: DLQ quarantine for invalid webhook signature.
37. Surface STR-30: DLQ replay for transient receiver failure.
38. Surface STR-31: audit event for outbound provider action.
39. Surface STR-32: audit event for inbound provider event.
40. Surface STR-33: tenant-class cap hooks for chat operations.
41. Surface STR-34: demo_trial cap enforcement for users, channels, messages, and attachments.
42. Surface STR-35: paid tenant scaling through paid usage and contractual SLOs.
43. Surface STR-36: revenue_share accounting for embedded community/chat use.
44. Surface STR-37: marketplace adapter certification flow.
45. Surface STR-38: provider health canary.
46. Surface STR-39: provider compression and payload-size caveat metadata where relevant.
47. Surface STR-40: migration playbook from direct Stream integration.
48. Surface STR-41: runbook for Stream webhook delivery degradation.
49. Surface STR-42: runbook for Stream API throttling.
50. Surface STR-43: Rust adapter trait implementation.
51. Surface STR-44: conformance test for webhook HMAC.
52. Surface STR-45: conformance test for rate-limit backoff.
53. Surface STR-46: conformance test for channel-member scale metadata.
54. Surface STR-47: conformance test for attachment metadata limits.
55. Surface STR-48: conformance test for DLQ replay.
56. Surface STR-49: least-privilege secret rotation handoff.
57. Surface STR-50: provider-side service-level caveat surfaced in catalog record.
58. Current connect coverage: generic webhook and connector substrate exists, but no Stream-specific record, contract example, test, or runbook exists.
59. Current gap: Stream union coverage is a new explicit add.
60. Severity implication: P2 feature parity gap until catalog and adapter contracts express Stream.

## §4 UNION-Coverage Matrix

| # | Capability family | Twilio | Sendbird | Stream | Current connect evidence | Gap status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Connector catalog provider record | Present via catalog file | Missing | Missing | `catalog/connectors/twilio.yaml`; no Sendbird/Stream inventory hits | Partial |
| 2 | Provider auth metadata | Needed | Needed | Needed | PRD OAuth broker `PRD.md:128-137`; contracts OAuth `openapi:97-175` | Generic only |
| 3 | OAuth grant initiation | Applicable for OAuth-style providers | Sometimes not applicable | Token-based rather than OAuth in many flows | OpenAPI grant initiation `contracts/openapi/connector-integration.yaml:97-129` | Needs provider-specific profiles |
| 4 | API-key secret management | Needed | Needed | Needed | PRD secret rules `PRD.md:184-188`; cloud-secrets dependency `manifest.json:108-128` | Handoff not explicit |
| 5 | Webhook endpoint registration | Needed | Needed | Needed | OpenAPI webhook registration `contracts/openapi/connector-integration.yaml:176-203` | Generic only |
| 6 | Webhook rotation | Needed | Needed | Needed | OpenAPI rotate endpoint `contracts/openapi/connector-integration.yaml:205-218` | Generic only |
| 7 | Webhook signature verification | Needed | Provider-dependent | HMAC documented | Architecture BC `ARCHITECTURE.md:26-38`; policy files | Provider examples missing |
| 8 | Duplicate event suppression | Needed | Needed | Needed | Failure modes cover replay risks `failure-modes.md:32-48` | Test evidence missing |
| 9 | Webhook replay window | Needed | Needed | Needed | PRD security `PRD.md:184-188` | Provider examples missing |
| 10 | Outbound action idempotency | Needed | Needed | Needed | Contracts imply action requests; proto action service `contracts/proto/connector_integration.proto:123-125` | Explicit idempotency shape missing |
| 11 | Connector action invocation | Needed | Needed | Needed | PRD FR `PRD.md:128-137`; capability `capabilities/connector-invoke.yaml` | Generic only |
| 12 | Provider rate-limit metadata | Needed | Needed | Needed | Runbook `runbooks/connector-rate-limit-saturation.md`; capacity model | Needs counterpart-specific catalog fields |
| 13 | Tenant rate-limit overlay | Needed | Needed | Needed | Abuse policy and architecture mention adaptive controls `ARCHITECTURE.md:711-725` | Needs tenant_class rewrite |
| 14 | Demo usage caps | Needed | Needed | Needed | No exact `demo_trial` hit | Missing |
| 15 | Paid usage scaling | Needed | Needed | Needed | Product paid mention `PRD.md:47`; old paid-tier text | Needs tenant_class semantics |
| 16 | Revenue-share accounting | Possible | Possible | Possible | No `revenue_share` hit | Missing |
| 17 | Provider error taxonomy | Needed | Needed | Needed | Failure modes `failure-modes.md:15-84` | Needs provider mapping |
| 18 | DLQ quarantine | Needed | Needed | Needed | PRD FR and SLO `PRD.md:128-137`; `slos/dlq-overflow-prevention.openslo.yaml:4-38` | Generic only |
| 19 | DLQ replay | Needed | Needed | Needed | Backfill replay `backfill-replay.md:21-27` | Generic only |
| 20 | Connector catalog search | Needed | Needed | Needed | PRD NFR `PRD.md:143-150`; OpenAPI catalog endpoints | Provider facets incomplete |
| 21 | Marketplace publisher flow | Needed for adapters | Needed for adapters | Needed for adapters | PRD persona `PRD.md:47` | Handoff missing |
| 22 | Adapter certification | Needed | Needed | Needed | ADR-MS verification lines `ADR-MS-001...md:229-269` | Provider cases missing |
| 23 | Rust adapter trait | Needed | Needed | Needed | `contracts/connector-adapter-trait.md`; no source tree | Contract only |
| 24 | Conformance tests | Needed | Needed | Needed | No `tests/` directory | Missing |
| 25 | Provider health canary | Needed | Needed | Needed | Dashboards exist for broad service health | Provider canaries missing |
| 26 | Provider outage circuit breaker | Needed | Needed | Needed | Failure modes mention provider disruption | Implementation missing |
| 27 | Per-provider dashboard | Needed | Needed | Needed | Dashboard files exist | Provider dimensions need proof |
| 28 | Cost attribution | Needed | Needed | Needed | Cost budget `cost-budget.md:42-47` | Provider and tenant_class dimensions missing |
| 29 | Audit events | Needed | Needed | Needed | PRD observability `PRD.md:162-164`; AsyncAPI events | Provider examples missing |
| 30 | PII redaction | Needed | Needed | Needed | DPIA mitigations `dpia.md:60-69` | Provider examples missing |
| 31 | Data residency caveat | Needed | Needed | Needed | PRD data residency `PRD.md:194-196` | Provider caveat metadata missing |
| 32 | BYOK eligibility | Tenant-class dependent | Tenant-class dependent | Tenant-class dependent | No explicit tenant_class | Missing |
| 33 | Compliance pack eligibility | Tenant-class dependent | Tenant-class dependent | Tenant-class dependent | Compliance pack docs exist | Tenant_class binding missing |
| 34 | Participant/member scale metadata | Conversations participants | Channel members | Channel members | Current catalog schema not proven | Missing for Sendbird/Stream; unknown for Twilio |
| 35 | Message throughput metadata | Messaging throughput | API request throughput | Endpoint limits | Existing benchmark is stale | Needs new performance doc |
| 36 | Attachment metadata | Media limits | Media/file support | File attachment size | Contracts generic | Provider fields missing |
| 37 | Delivery receipts | Strong Twilio fit | Message events | Message events | AsyncAPI generic events | Provider event mapping missing |
| 38 | Moderation actions | Limited by product | Strong fit | Strong fit | No explicit moderation connector family | Missing |
| 39 | Announcement/broadcast | Twilio campaign/product-specific | Strong Sendbird fit | Product-specific | No explicit broadcast adapter policy | Missing |
| 40 | Push event handling | Provider-dependent | Strong fit | Strong fit | Generic webhook substrate | Provider mapping missing |
| 41 | User identity mapping | Needed | Needed | Needed | OAuth/user concepts in contracts | Provider mapping missing |
| 42 | Channel/conversation identity mapping | Needed | Needed | Needed | Connector action generic | Provider mapping missing |
| 43 | Provider migration playbook | Needed | Needed | Needed | Existing migration is Slack/Teams-focused | Need new playbooks |
| 44 | Provider onboarding tutorial | Needed | Needed | Needed | Existing tutorial is cross-tenant federation | Need integration tutorials |
| 45 | Provider FAQ | Needed | Needed | Needed | Existing FAQ is federation-focused | Need integration FAQ |
| 46 | Provider incident runbook | Needed | Needed | Needed | Generic runbooks exist | Need counterpart-specific entries |
| 47 | Webhook ack latency target | Needed | Needed | Needed | PRD NFR `PRD.md:143-150` | Good generic target |
| 48 | Connector action overhead target | Needed | Needed | Needed | PRD NFR `PRD.md:143-150` | Good generic target |
| 49 | Catalog latency target | Needed | Needed | Needed | PRD NFR `PRD.md:143-150` | Good generic target |
| 50 | Platform scale target | Needed | Needed | Needed | PRD scale `PRD.md:156-158` | Needs provider overlays |
| 51 | Six deployment context overlays | Needed | Needed | Needed | No context modules | Missing |
| 52 | OCI Always Free profile | Needed for demo | Needed for demo | Needed for demo | No profile directory | Missing |
| 53 | OS support manifest | Needed | Needed | Needed | No supported-oses file | Missing |
| 54 | OpenTofu modules | Needed | Needed | Needed | Flat IaC only | Missing |
| 55 | Contract versioning | Needed | Needed | Needed | Contract files versioned | Provider versions missing |
| 56 | Schema drift handling | Needed | Needed | Needed | Backfill/schema replay doc `backfill-replay.md:28-35` | Generic only |
| 57 | Credential revocation cascade | Needed | Needed | Needed | Runbook exists `runbooks/oauth-token-revocation-cascade.md` | Provider cases missing |
| 58 | PII leak runbook | Needed | Needed | Needed | Runbook exists `runbooks/pii-leak-via-connector.md` | Good generic base |
| 59 | Webhook replay attack runbook | Needed | Needed | Needed | Runbook exists `runbooks/webhook-replay-attack-detected.md` | Good generic base |
| 60 | Signature cascade runbook | Needed | Needed | Needed | Runbook exists `runbooks/signature-verification-cascade-failure.md` | Good generic base |
| 61 | Connector certification evidence | Needed | Needed | Needed | ADR-MS verification `ADR-MS-001...md:229-269` | Provider evidence missing |
| 62 | Contract examples | Needed | Needed | Needed | OpenAPI/AsyncAPI/proto present | Provider examples missing |
| 63 | SDK surface | Needed for developers | Needed for developers | Needed for developers | `sdk-plan.md` exists | Needs provider-specific SDK examples |
| 64 | Runtime source | Needed | Needed | Needed | No `src/` directory | Missing |
| 65 | Regression tests | Needed | Needed | Needed | No `tests/` directory | Missing |
| 66 | Manifest active purpose | Needed | Needed | Needed | Manifest says retiring | Misaligned |
| 67 | Handoff to workflow-engine | Needed | Needed | Needed | Dependencies listed | Handoff doc missing |
| 68 | Handoff to billing-ledger | Needed | Needed | Needed | Dependency listed | Tenant_class cost handoff missing |
| 69 | Handoff to policy-engine | Needed | Needed | Needed | Dependency listed | Provider policy matrix missing |
| 70 | Handoff to observability | Needed | Needed | Needed | Dependency listed | Provider metrics map missing |

## §5 Family Summary

1. Catalog family summary: connector has a credible generic catalog base, but the union bar requires Twilio, Sendbird, and Stream as explicit communications-provider records.
2. Catalog evidence: Twilio exists, Sendbird and Stream do not appear in the file inventory.
3. Catalog target: each counterpart needs provider metadata, auth pattern, event families, action families, rate limits, identity mapping, and data-handling caveats.
4. OAuth and secret family summary: the PRD and OpenAPI express generic OAuth grant lifecycle.
5. OAuth gap: Sendbird and Stream may use token or API-key patterns in common deployments, so connect needs provider-specific credential profiles, not a one-size OAuth assumption.
6. Webhook family summary: connector has the right generic receiver, signature, and DLQ concepts.
7. Webhook gap: provider-specific HMAC, retry, timeout, duplicate, and event mapping examples are missing.
8. Action invocation summary: connector owns outbound connector action invocation.
9. Action gap: no Rust adapter implementation or provider-specific conformance tests exist.
10. Data mapping summary: architecture names data-mapping as a bounded context.
11. Data mapping gap: no message/conversation/channel/user normalization examples for Twilio, Sendbird, or Stream exist.
12. Retry and DLQ summary: PRD, SLOs, backfill replay, runbooks, and failure modes give strong generic coverage.
13. Retry and DLQ gap: provider-specific backoff and retry semantics are not encoded.
14. Observability summary: dashboards and metric naming exist.
15. Observability gap: provider, tenant_class, action family, and deployment-context labels are not visibly enforced by tests.
16. Compliance summary: compliance, DPIA, and threat model are strong.
17. Compliance gap: provider data-residency caveats and BYOK/compliance-pack eligibility are not bound to tenant_class.
18. Deployment summary: all union coverage depends on six deployment contexts.
19. Deployment gap: canonical context modules are missing.
20. Language summary: no forbidden implementation-language files were found.
21. Language gap: no Rust implementation evidence exists.
22. Product-boundary summary: connector should not become chat UI or social product logic.
23. Product-boundary gap: federation tutorial, federation FAQ, and old benchmark can mislead future work.
24. Counterpart summary: Twilio adds phone/message sender and delivery-state discipline.
25. Counterpart summary: Sendbird adds channel, user, moderation, announcement, push, and API-rate-limit discipline.
26. Counterpart summary: Stream adds webhook HMAC, endpoint rate-limit, channel limit, attachment, unread, and client-event discipline.
27. Union summary: the common substrate is connector registry, credential profile, action invocation, webhook ingestion, event normalization, policy gate, rate-limit backoff, DLQ, observability, and audit.
28. Union summary: the additive substrate is communications-specific identity/channel/message metadata and provider-side rate-limit profiles.
29. Union summary: connector has documentation primitives but lacks provider-specific records and executable proof.
30. Union summary: feature parity work should start with provider catalog records and conformance fixtures before broad implementation.

## §6 Headline Gap Analysis

1. Gap H-01: active purpose conflict in manifest blocks automated discovery from trusting the integration substrate scope.
2. Evidence H-01: `microservices/connector/manifest.json:3-6` versus `microservices/connector/PRD.md:29-39`.
3. Impact H-01: provider parity automation may index retirement contracts rather than integration contracts.
4. Gap H-02: Sendbird and Stream catalog records are missing.
5. Evidence H-02: full inventory contains Twilio connector record and no Sendbird or Stream connector record.
6. Impact H-02: two of three required counterpart surfaces have no service-local provider anchor.
7. Gap H-03: no provider-specific conformance tests exist.
8. Evidence H-03: no `microservices/connector/tests/` directory exists.
9. Impact H-03: webhook, rate-limit, identity mapping, and DLQ behavior remain unproven.
10. Gap H-04: no Rust implementation source exists.
11. Evidence H-04: no `microservices/connector/src/` directory exists.
12. Impact H-04: parity remains design-only.
13. Gap H-05: current parity and benchmark docs use old counterpart sets.
14. Evidence H-05: `microservices/connector/competitor-parity-matrix.md:13-36` and `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:9-13`.
15. Impact H-05: current docs do not answer this batch’s union-coverage bar.
16. Gap H-06: old federation artifacts conflict with the integration substrate boundary.
17. Evidence H-06: chat `8f603fc7...jsonl:8923-8926` states not iPaaS/workflow-trigger/data-pipeline; federation docs remain in FAQ/tutorial/reference implementation.
18. Impact H-06: feature parity may drift into chat product implementation.
19. Gap H-07: tenant_class semantics are not expressed.
20. Evidence H-07: no `tenant_class`, `demo_trial`, or `revenue_share` hits; old paid-tier text in `ARCHITECTURE.md:715` and `policy/abuse-defence.cedar:80`.
21. Impact H-07: provider caps, commercial handling, and SLO overlays cannot be evaluated by class.
22. Gap H-08: six deployment context overlays are absent.
23. Evidence H-08: no canonical context directories under `microservices/connector/iac/`.
24. Impact H-08: provider parity cannot be claimed across public cloud, guest cloud, on-prem, colo, and provider contexts.
25. Gap H-09: OCI Always Free profile is absent.
26. Evidence H-09: no `microservices/connector/iac/oci-guest/always-free/` directory exists.
27. Impact H-09: demo_trial infrastructure caps cannot be enforced or benchmarked.
28. Gap H-10: OS support manifest is absent.
29. Evidence H-10: no `microservices/connector/supported-oses.json`.
30. Impact H-10: communication-provider adapter behavior cannot be mapped to the canonical OS and architecture matrix.
31. Gap H-11: cross-microservice handoff doc is absent.
32. Evidence H-11: no `cross-microservice-handoffs.md`, while dependencies are listed in `manifest.json:108-128`.
33. Impact H-11: workflow-engine, marketplace, billing, policy, and observability boundaries remain informal.
34. Gap H-12: provider rate-limit and scale metadata are not visible in catalog schema.
35. Evidence H-12: current catalog exists but no counterpart-specific rate-limit fields were verified in this audit.
36. Impact H-12: provider backoff and tenant caps cannot be compared to Twilio, Sendbird, or Stream.
37. Gap H-13: webhooks are generic, not counterpart-specific.
38. Evidence H-13: OpenAPI has generic endpoint registration `contracts/openapi/connector-integration.yaml:176-218`.
39. Impact H-13: HMAC, timeout, duplicate, and retry differences are not encoded.
40. Gap H-14: cost attribution exists generically, but provider and tenant_class dimensions are not explicit.
41. Evidence H-14: `microservices/connector/cost-budget.md:42-47`.
42. Impact H-14: paid and revenue-share economics cannot be enforced by connect evidence.
43. Gap H-15: current connector seed is far below product target.
44. Evidence H-15: PRD requires at least 500 connectors `microservices/connector/PRD.md:156`; prior audit says 30 seed connectors and 470-plus deferred `AUDIT-FINDINGS-2026-05-20.json:25-30`.
45. Impact H-15: union coverage can start with three counterparts but cannot be treated as catalog maturity.

## §7 Additive Surface For Connect

1. Add A-01: create Sendbird connector catalog record with provider auth, channel/user/message operations, webhook events, and rate-limit metadata.
2. Add A-02: create Stream connector catalog record with API key/secret profile, channel/user/message operations, webhook HMAC, endpoint limits, and attachment metadata.
3. Add A-03: enrich Twilio connector catalog record with Conversations/Messaging product families, delivery events, sender throughput, and participant limits.
4. Add A-04: define a provider capability schema for communications connectors.
5. Add A-05: define provider-rate-limit fields separate from tenant-class caps.
6. Add A-06: define tenant-class cap hooks for demo_trial, paid, and revenue_share.
7. Add A-07: define provider webhook signature profiles.
8. Add A-08: define provider webhook duplicate keys and replay-window expectations.
9. Add A-09: define provider error taxonomy mappings for Twilio, Sendbird, and Stream.
10. Add A-10: define provider identity mapping for user, participant, channel, conversation, and message identifiers.
11. Add A-11: define provider event normalization into AsyncAPI event families.
12. Add A-12: define provider conformance fixture format.
13. Add A-13: define Rust adapter trait examples for send message, receive webhook, rate-limit backoff, and DLQ replay.
14. Add A-14: define a provider health canary contract.
15. Add A-15: define provider observability label schema.
16. Add A-16: define provider cost attribution dimensions.
17. Add A-17: define provider data-residency caveat fields.
18. Add A-18: define marketplace adapter certification criteria.
19. Add A-19: define cross-microservice handoff to marketplace for listing and royalties.
20. Add A-20: define cross-microservice handoff to billing-ledger for usage and revenue-share accounting.
21. Add A-21: define cross-microservice handoff to policy-engine for outbound action authorization.
22. Add A-22: define cross-microservice handoff to cloud-secrets for credential storage and rotation.
23. Add A-23: define cross-microservice handoff to observability for provider dashboards and alerts.
24. Add A-24: define cross-microservice handoff to workflow-engine for consuming normalized events.
25. Add A-25: define deployment-context overlays for counterpart connector availability.
26. Add A-26: define OCI Always Free profile caps for communications connector demos.
27. Add A-27: define on-prem and colo egress constraints for external provider APIs.
28. Add A-28: define provider outage circuit breaker behavior.
29. Add A-29: define provider retry budget and DLQ escalation rules.
30. Add A-30: define provider-specific runbooks for delivery degradation and API throttling.
31. Add A-31: define migration playbooks from direct Twilio, Sendbird, and Stream usage into connect.
32. Add A-32: define onboarding tutorial for communications connector configuration.
33. Add A-33: define FAQ for provider limits, data handling, and operational responsibilities.
34. Add A-34: define audit events for outbound and inbound provider interactions.
35. Add A-35: define PII redaction requirements for provider payloads.
36. Add A-36: define provider contract examples in OpenAPI, AsyncAPI, and proto.
37. Add A-37: define service-local tests for provider signature, duplicate, rate-limit, and DLQ behavior.
38. Add A-38: define Rust implementation modules for catalog, credential profile, webhook verification, action invocation, and DLQ replay.
39. Add A-39: align manifest purpose/status to active integration substrate before automation consumes the feature matrix.
40. Add A-40: retire or archive old federation parity artifacts so they do not drive communications connector implementation.

## §8 Batch Conclusion

1. Twilio has partial local coverage because a connector record exists and the generic substrate docs match many Twilio integration needs.
2. Sendbird has no local connector record and should be treated as a P2 union-coverage gap.
3. Stream has no local connector record and should be treated as a P2 union-coverage gap.
4. The most important shared capabilities are provider credential profile, webhook signature profile, provider rate-limit metadata, event normalization, DLQ handling, policy gate, observability labels, and provider-specific conformance tests.
5. The current docs are strong enough to define the additive surface but not strong enough to prove implementation parity.
6. The next useful connect artifact is not another generic parity table; it is a provider-capability schema plus three counterpart connector records and conformance fixtures.
7. The parity target remains industry-leader grade across tenant classes, with caps expressed as billing and infrastructure overlays rather than old capability strata.
8. This report introduces no new capability-tier scaffold and does not create the retired fourth deliverable.

