# Identity microservice ownership-coherence audit - 2026-05-20

Citation anchors:
1. Canonical sequence and batch discipline: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-4234`, especially D-15 through D-20.
2. Machine-readable master plan: `specs/master-plan-sequencing.json:704-889`, including deployment contexts, OpenTofu substrate, OS matrix, language policy, and OCI Always Free.
3. Identity product requirements: `microservices/identity/PRD.md:1-1642`, read end to end for purpose, scope, constraints, and benchmark claims.
4. Identity architecture: `microservices/identity/ARCHITECTURE.md:1-880`, read end to end for topology, dependencies, operations, and edge cases.
5. Documentation rigor: `docs/standards/documentation-rigor.md:133-139` for intern-buildability and `docs/standards/documentation-rigor.md:58-83` for microservice doc-set completeness.

Constraint memory anchors read:
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md`.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md`.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md`.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md`.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md`.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md`.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md`.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md`.

Chat-history anchors processed:
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:552` records the doctrine that Oyatie internal actors authenticate as tenant-scoped principals instead of receiving shortcuts.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:571` names Identity as the service that binds CI agents, deploy actors, and service-to-service calls to tenant-scoped SPIFFE identities.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:602` places Identity under Tenancy and above Ontology in the substrate dependency graph.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:732` requires hot-path auth challenge and session state to remain per-cell and survive cross-cell isolation.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:4188-4206` records that the Identity PRD was expanded from a short stub to a 1642-line product requirement.

## 1. Microservice purpose summary

Identity is the platform authentication and principal-resolution substrate.
The PRD states that it owns OIDC, JWKS, WebAuthn, SCIM, MFA, passkeys, step-up, minors, recovery, and audit evidence across consumer, workforce, regulated, and emergency workflows (`microservices/identity/PRD.md:99-109`).
The PRD also names Auth0, Okta, and Microsoft Entra as comparator surfaces and positions Identity as a challenger to their hosted identity platforms (`microservices/identity/PRD.md:70-86`).
The architecture expands that scope into issuer, relying-party, SCIM, federation, HRIS, audit, risk, and Cedar coordination surfaces (`microservices/identity/ARCHITECTURE.md:22-29`).
The accepted passkey ADR makes passkeys primary, WebAuthn Level 3 the target, hardware-bound credentials mandatory for high-risk flows, and recovery envelope custody OpenBao-backed (`microservices/identity/decisions/ADR-ID-001-passkey-primary-webauthn-recovery-envelope.md:47-74`).
The service is therefore not merely a login component.
It is the principal root for tenant context, service-to-service calls, operator action, authorization claims, cross-microservice handoff, and audit-chain evidence.
The chat-history substrate graph makes this explicit: Tenancy depends downward on Cell, Identity depends on Tenancy, and Ontology depends on Identity (`8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:602`).
The current corpus has strong product and protocol detail for OIDC, WebAuthn, SCIM, and step-up.
The current corpus has weaker deployment, OpenTofu, OS, and OCI Always Free coherence.
The highest-risk gap is not missing identity-domain ambition.
The highest-risk gap is that deployment-substrate claims now lag ADR-0328's Wave 2 canonical constraints.

## 2. Inventory snapshot

Investigation inventory result: 237 files were listed under `microservices/identity/`.
Line-count audit result: 74,456 total lines were present under `microservices/identity/` at investigation time.
Core artifacts read in full or to substantial line counts: PRD, ARCHITECTURE, manifest, capability tenant_class adoption matrix, contracts, SLOs, capacity, cost, failure modes, incident response, DPIA, compliance, benchmark, migration playbook, decisions, IP plans, representative capability records, representative IaC, and test plans.
Absent expected files: `README.md`, `cross-microservice-handoffs.md`, `supported-oses.json`, `implementation-plans/`, `src/`.

| File or file family | Size observed | Role | Coherent with purpose? |
|---|---:|---|---|
| `PRD.md` | 1642 lines | Product requirements and scope authority | yes |
| `ARCHITECTURE.md` | 880 lines | Architecture narrative | partial |
| `manifest.json` | 433 lines | Service catalog and dependency graph | partial |
| `PHASE-01-OIDC-PASSKEY-SCIM-SUBSTRATE.md` | present | Phase substrate plan | partial |
| `competitor-parity-matrix.md` | 100 lines | Local competitor parity claim | partial |
| `ADR-0330 and ADR-0331 tenant_class model` | 181 lines | demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating definitions | partial |
| `capacity-model.md` | 158 lines | Traffic and scaling model | partial |
| `cost-budget.md` | 156 lines | TCO and vendor displacement model | partial |
| `failure-modes.md` | 132 lines | Failure catalog | yes |
| `incident-response.md` | 178 lines | Incident procedures | yes |
| `dpia.md` | 113 lines | Privacy impact analysis | partial |
| `compliance.md` | 1051 lines | Control mapping and evidence narrative | partial |
| `multi-region.md` | present | Region/cell operations | partial |
| `backfill-replay.md` | present | Replay semantics | partial |
| `sdk-plan.md` | present | SDK surface plan | partial |
| `threat-model.md` | present | Threat model root doc | yes |
| `security/threat-model.md` | present | Security threat detail | yes |
| `benchmarks/okta-auth0-entra-vs-oyatie.md` | 119 lines | Competitive benchmark narrative | partial |
| `migration-playbooks/from-okta.md` | 192 lines | Okta migration path | partial |
| `onboarding/identity-engineer-first-week.md` | present | First-week onboarding | yes |
| `faqs/identity-engineer-faq.md` | present | Engineer FAQ | yes |
| `tutorials/register-passkey-and-recovery-envelope.md` | present | User-facing implementation tutorial | yes |
| `reference-implementations/webauthn-passkey-flow-rust-sdk.md` | present | Rust SDK walkthrough | yes |
| `contracts/openapi/identity.yaml` | 641 lines | HTTP contract | yes |
| `contracts/openapi/multi-context-split.yaml` | 49 lines | Multi-context split HTTP contract | partial |
| `contracts/asyncapi/identity-events.yaml` | 196 lines | Event contract | yes |
| `contracts/asyncapi/multi-context-events.yaml` | 46 lines | Multi-context event contract | partial |
| `contracts/proto/identity.proto` | 171 lines | Internal gRPC/ext_authz/admin contract | yes |
| `contracts/proto/multi_context_split.proto` | 33 lines | Multi-context proto contract | partial |
| `slos/oidc-token-issue-latency.openslo.yaml` | 45 lines | OIDC issue latency SLO | yes |
| `slos/oidc-token-verify-latency.openslo.yaml` | present | OIDC verify latency SLO | yes |
| `slos/webauthn-authenticate-latency.openslo.yaml` | present | WebAuthn latency SLO | yes |
| `slos/scim-availability.openslo.yaml` | 36 lines | SCIM availability SLO | yes |
| `slos/jwks-availability.openslo.yaml` | 39 lines | JWKS availability SLO | yes |
| `slos/step-up-grant-latency.openslo.yaml` | present | Step-up latency SLO | yes |
| `slos/aaguid-refresh-freshness.openslo.yaml` | present | AAGUID freshness SLO | yes |
| `slos/audit-emit-completeness.openslo.yaml` | present | Audit emit completeness SLO | yes |
| `slos/zitadel-instance-health.openslo.yaml` | present | Zitadel health SLO | yes |
| `capabilities/oidc-token-issue.yaml` | 75 lines | OIDC capability record | yes |
| `capabilities/webauthn-authenticate.yaml` | 69 lines | WebAuthn capability record | yes |
| `capabilities/scim-user-provision.yaml` | 57 lines | SCIM capability record | yes |
| `capabilities/step-up-acr-grant.yaml` | present | Step-up capability record | yes |
| `capabilities/multi-context-principal-resolve.yaml` | present | Multi-context resolver record | partial |
| `catalog/*.yaml` | 10 files | Crate/control catalog records | yes |
| `policy/*.cedar` | 5 files | Cedar policy fragments | yes |
| `policy/data-residency.md` | present | Residency policy | yes |
| `dashboards/*.json` | 3 files | Grafana dashboards | yes |
| `scorecards/*.json` | 5 files | Scorecard evidence | yes |
| `runbooks/brute-force-mitigation.md` | present | Brute-force response | yes |
| `runbooks/idp-failover-drill.md` | present | IdP failover drill | yes |
| `runbooks/ip-block-incident.md` | present | IP blocking incident | yes |
| `runbooks/jwks-rotation.md` | present | JWKS rotation | yes |
| `runbooks/passkey-cross-device-debug.md` | present | Passkey debug | yes |
| `runbooks/passkey-replay-attack-response.md` | present | Replay response | yes |
| `runbooks/passkey-reset.md` | present | Passkey reset | yes |
| `runbooks/recovery-key-mass-issue-investigation.md` | present | Recovery-key incident | yes |
| `runbooks/scim-provisioning-debug.md` | present | SCIM debug | yes |
| `runbooks/tenant-admin-onboard.md` | present | Tenant admin onboarding | yes |
| `runbooks/webauthn-rp-id-rotation.md` | present | RP-ID rotation | yes |
| `decisions/ADR-ID-001-passkey-primary-webauthn-recovery-envelope.md` | 208+ lines read | Passkey primary ADR | yes |
| `decisions/ADR-identity-001-jwks-rotation-cadence.md` | 29 lines | JWKS rotation ADR | yes |
| `decisions/ADR-identity-002-passkey-attestation-policy.md` | present | Attestation ADR | yes |
| `decisions/ADR-identity-003-scim-rate-limits.md` | present | SCIM rate ADR | yes |
| `decisions/ADR-identity-004-session-class-tiers.md` | present | Session class ADR | yes |
| `decisions/ADR-identity-005-jit-it-approval-protocol.md` | 70 lines | Critical ACR approval ADR | yes |
| `IP-001-zitadel-helm-per-pack.md` | 99 lines | Helm deployment plan | partial |
| `IP-002-oidc-issuer-kernel.md` | present | OIDC issuer plan | yes |
| `IP-003-oidc-issuer-adapter-zitadel.md` | present | Zitadel adapter plan | yes |
| `IP-004-webauthn-relying-party-kernel.md` | present | WebAuthn kernel plan | yes |
| `IP-005-webauthn-rest.md` | present | WebAuthn REST plan | yes |
| `IP-006-aaguid-refresh-worker.md` | present | AAGUID worker plan | yes |
| `IP-007-scim-server-kernel.md` | present | SCIM kernel plan | yes |
| `IP-008-scim-adapter-zitadel.md` | present | SCIM adapter plan | yes |
| `IP-009-hris-adapter.md` | present | HRIS adapter plan | yes |
| `IP-010-step-up-orchestrator.md` | present | Step-up plan | yes |
| `IP-011-external-idp-federation.md` | present | Federation plan | yes |
| `IP-012-audit-emitter.md` | present | Audit emitter plan | yes |
| `IP-013-edge-authz-rules.md` | present | Edge authz plan | yes |
| `IP-014-continuous-risk-scoring.md` | present | Risk scoring plan | partial |
| `IP-015-shared-kernel-crates.md` | present | Shared crate plan | yes |
| `IP-016-zitadel-scale-validation-load-test.md` | 117 lines | Scale validation plan | drifted |
| `IP-017-multi-context-principal-resolver.md` | 28 lines | Multi-context resolver plan | drifted |
| `IP-journey-j*.md` | 100+ files | Journey-driven identity resolver slices | partial |
| `iac/helm/zitadel/*` | 10 files | Helm chart for Zitadel | partial |
| `iac/kustomize/components/edge-authz-rules/*` | 5 files | Kustomize edge-authz rules | partial |
| `iac/kustomize/overlays/pack-ae/values.yaml` | present | Regulatory pack overlay | yes |
| `iac/kustomize/overlays/pack-eu/values.yaml` | present | Regulatory pack overlay | yes |
| `iac/kustomize/overlays/pack-kr/values.yaml` | present | Regulatory pack overlay | yes |
| `iac/kustomize/overlays/pack-ksa/values.yaml` | present | Regulatory pack overlay | yes |
| `iac/kustomize/overlays/pack-us-healthcare/values.yaml` | present | Regulatory pack overlay | yes |
| `test-plans/unit-test-strategy.md` | present | Unit test plan | yes |
| `test-plans/integration-test-strategy.md` | present | Integration test plan | yes |
| `test-plans/contract-test-strategy.md` | present | Contract test plan | yes |

## 3. 9-dimension audit

### 3.1 Dimension 1 - internal coherence within the identity path

01. Purpose alignment is strong: PRD lines 99-109 describe universal OIDC, JWKS, residency, SSO, SCIM, MFA, passkey, step-up, minors, and audit needs, while the architecture inventory points to matching contracts, policies, SLOs, runbooks, and IaC surfaces (`PRD.md:99-109`; `ARCHITECTURE.md:22-29`).
02. Passkey coherence is strong between PRD, ADR, OpenAPI, and runbooks: PRD requires WebAuthn L3 and passkeys (`PRD.md:1460-1464`), ADR-ID-001 chooses passkeys as primary (`decisions/ADR-ID-001...md:47-74`), OpenAPI exposes registration/authentication endpoints (`contracts/openapi/identity.yaml:118-200`), and passkey runbooks exist.
03. OIDC coherence is mostly strong: PRD says the OIDC issuer wraps Zitadel (`PRD.md:1449-1456`), OpenAPI exposes discovery, JWKS, and token paths (`contracts/openapi/identity.yaml:49-83`), and the JWKS ADR sets 90-day scheduled and 15-minute emergency rotation (`decisions/ADR-identity-001-jwks-rotation-cadence.md:17-29`).
04. SCIM coherence is strong for inbound provisioning: PRD requires SCIM RFC 7643/7644 and Okta/Entra/Google compatibility (`PRD.md:1468-1480`), OpenAPI exposes `/scim/v2` user and group paths (`contracts/openapi/identity.yaml:234-372`), and capability `scim-user-provision.yaml` exists.
05. Federation coherence is strong for customer IdP connections: PRD lists SAML, OIDC, Apple, Google, Kakao, LINE, WeChat, and Naver (`PRD.md:1482-1507`), and OpenAPI exposes federation bindings (`contracts/openapi/identity.yaml:425-446`).
06. Step-up coherence is strong at concept level: PRD requires ACR and step-up (`PRD.md:730-738`), OpenAPI exposes step-up challenge/verify (`contracts/openapi/identity.yaml:374-423`), and ADR-identity-005 defines JIT approval for `acr=critical` (`decisions/ADR-identity-005-jit-it-approval-protocol.md:11-70`).
07. Contradiction probe 1: ARCHITECTURE repeatedly says the service cross-reference consumer is `identity` itself, creating a wrong-direction dependency claim where downstream services should be named (`ARCHITECTURE.md:290-296`, `ARCHITECTURE.md:788-794`, `compliance.md:153-159`).
08. Finding severity: P1 because self-consumption corrupts ownership handoff for a substrate service.
09. Contradiction probe 2: `manifest.json` marks IP-017 multi-context principal resolver as `ga` (`manifest.json:233-250`), but `capabilities/multi-context-principal-resolve.yaml` declares the capability `scaffolded` (`capabilities/multi-context-principal-resolve.yaml:9-12`).
10. Finding severity: P0 because Identity is a T0 substrate and the exact multi-context principal resolver is a canonical-context blocker.
11. Contradiction probe 3: ARCHITECTURE claims IaC evidence surfaces are present (`ARCHITECTURE.md:256-258`, `ARCHITECTURE.md:318-319`), while the actual `iac/` tree contains Helm and Kustomize only and no ADR-0328 context OpenTofu modules.
12. Finding severity: P0 because ADR-0328 D-15 and D-16 make per-context OpenTofu evidence mandatory for this service class (`ADR-0328:1730-2210`; `specs/master-plan-sequencing.json:747-775`).
13. Contradiction probe 4: compliance says the inventory spans Helm, Kustomize, and OpenTofu (`compliance.md:866-878`), but no `iac/<context>/versions.tf`, `main.tf`, `variables.tf`, or `outputs.tf` exists for identity.
14. Finding severity: P1 because the compliance evidence claim is broader than the actual file inventory.
15. Contradiction probe 5: `IP-001-zitadel-helm-per-pack.md` names a `pack-us` overlay (`IP-001-zitadel-helm-per-pack.md:21-38`), but the actual Kustomize overlays are `pack-ae`, `pack-eu`, `pack-kr`, `pack-ksa`, and `pack-us-healthcare`.
16. Finding severity: P2 because the wrong pack name is locally correctable and does not alter the authentication model.
17. Contradiction probe 6: PRD says benchmark modeling notes are expected in performance-budget docs (`PRD.md:779-793`), but no `docs/performance-budgets/identity-token-issuance.md` or `identity-webauthn-budget.md` exists under the identity path.
18. Finding severity: P2 because the main PRD is explicit about missing evidence and the local benchmark doc has unverified numbers.
19. Contradiction probe 7: `benchmarks/okta-auth0-entra-vs-oyatie.md` gives measured-looking latency and throughput values (`benchmarks/okta-auth0-entra-vs-oyatie.md:19-107`), but no benchmark result CSV path referenced by the doc was observed in inventory.
20. Finding severity: P1 because measured claims without attached results can mislead Wave 14 aggregation.
21. Contradiction probe 8: `migration-playbooks/from-okta.md` claims SDKs in Rust, TypeScript, Python, and Go (`migration-playbooks/from-okta.md:183-183`), while ADR-0328 forbids backend Python, TypeScript application logic, and Go unless justified (`specs/master-plan-sequencing.json:817-855`).
22. Finding severity: P1 because the doc prescribes forbidden future implementation surfaces.
23. Contradiction probe 9: `IP-016-zitadel-scale-validation-load-test.md` prescribes `k6-*.js`, `run.sh`, and Go-based tools (`IP-016-zitadel-scale-validation-load-test.md:49-68`), which conflicts with the Rust-strict doctrine when those scripts become build inputs.
24. Finding severity: P1 under ADR-0328 D-20.138 because docs prescribing a violating future path must be classified by the prescribed path (`ADR-0328:4170-4172`).
25. Contradiction probe 10: the demo_trial tenant_class requires four identity-api nodes, each 8 vCPU/32 GiB/500 GiB, plus multiple data services (`ADR-0330 and ADR-0331 tenant_class model:15-24`), which cannot be reconciled with OCI Always Free's 4 OCPU/24 GiB/200 GB block budget (`specs/master-plan-sequencing.json:857-867`).
26. Finding severity: P0 because the brief requires OCI Always Free demo_trial Always Free reconciliation and Identity is expected in all six contexts.
27. Internal cross-reference: PRD references `contracts/openapi/identity.yaml`; target exists and resolves (`PRD.md:751-765`; `contracts/openapi/identity.yaml:1-641`).
28. Internal cross-reference: PRD references `contracts/asyncapi/identity-events.yaml`; target exists and resolves (`PRD.md:751-765`; `contracts/asyncapi/identity-events.yaml:1-196`).
29. Internal cross-reference: PRD references `policy/cedar-acr-predicates.cedar`; target exists and resolves.
30. Internal cross-reference: ADR-ID-001 references `runbooks/jwks-rotation.md`; target exists (`decisions/ADR-ID-001...md:20-23`).
31. Internal cross-reference: ADR-ID-001 references `runbooks/passkey-reset.md`; target exists (`decisions/ADR-ID-001...md:20-23`).
32. Internal cross-reference: ADR-ID-001 references `policy/operator-recovery.cedar`; target exists (`decisions/ADR-ID-001...md:20-23`).
33. Internal cross-reference: ADR-identity-005 references `runbooks/passkey-reset.md`; target exists (`decisions/ADR-identity-005-jit-it-approval-protocol.md:67-70`).
34. Internal cross-reference: manifest references `decisions/ADR-identity-001-jwks-rotation-cadence.md`; target exists (`manifest.json:262-275`).
35. Internal cross-reference: manifest references `decisions/ADR-identity-005-jit-it-approval-protocol.md`; target exists (`manifest.json:262-275`).
36. Internal cross-reference: manifest references `slos/*.openslo.yaml`; targets exist for the declared SLO family (`manifest.json:180-230`; `slos/` inventory).
37. Internal cross-reference: manifest references `catalog/*.yaml`; ten catalog records exist (`manifest.json:8-118`; `catalog/` inventory).
38. Internal cross-reference: benchmark references a reproducibility command and results directory (`benchmarks/okta-auth0-entra-vs-oyatie.md:108-119`); the command/result artifact is not present in identity inventory.
39. Internal cross-reference: compliance references Helm and Kustomize surfaces; targets exist under `iac/helm/zitadel/` and `iac/kustomize/`.
40. Internal cross-reference: compliance references OpenTofu; target does not exist under identity IaC.
41. Internal cross-reference: PRD references a future benchmark note; target does not exist under identity.
42. Internal cross-reference: IP-001 references `pack-us`; actual target appears to be `pack-us-healthcare`.
43. Wrong-direction reference: ARCHITECTURE says `identity` consumes `identity`; that is wrong-direction.
44. Wrong-direction reference: compliance repeats the same `identity` consumed by `identity` row; that is wrong-direction.
45. Internal buildability risk: no README means the reader must infer entry points from PRD, architecture, and manifest, against the documentation-rigor roster that expects README (`docs/standards/documentation-rigor.md:64-67`).
46. Internal buildability risk: no `cross-microservice-handoffs.md` means service-to-service interfaces are spread across architecture, manifest, and contracts.
47. Internal buildability risk: no `src/` means all runtime statements are design-only; that is acceptable for audit phase but cannot support "measured" claims.
48. Internal buildability risk: no `implementation-plans/` directory despite the prompt asking for it; plans are top-level IP files instead.
49. Internal coherence score: strong product model, strong contracts, weak deployment substrate, weak OS substrate, weak benchmark evidence.
50. Dimension verdict: drifted-fixable with P0 deployment and OCI-tenant_class blockers.

### 3.2 Dimension 2 - outbound cross-references and inbound references

01. Outbound ADR references in ADR-ID-001 to ADR-0002, ADR-0003, ADR-0007, ADR-0008, and ADR-0043 exist by path convention and bind identity to tenant, audit, Cedar, data-use, and secret-custody doctrine (`decisions/ADR-ID-001...md:7-13`).
02. Outbound ADR references in manifest include identity-specific ADRs and related global ADRs (`manifest.json:262-275`).
03. Outbound cross-reference to Tenancy is explicit in PRD and architecture through tenant_id, tenant context, and home_cell claims (`PRD.md:751-765`; `contracts/proto/identity.proto:38-51`).
04. Outbound cross-reference to policy-engine/Cedar is explicit in PRD, architecture, proto, and policy fragments (`PRD.md:751-765`; `decisions/ADR-ID-001...md:160-164`).
05. Outbound cross-reference to audit-chain is explicit in PRD, AsyncAPI, and compliance (`contracts/asyncapi/identity-events.yaml:1-16`; `compliance.md:104-117`).
06. Outbound cross-reference to governance is explicit in ADR-identity-005, which notifies the governance microservice for JIT approval (`decisions/ADR-identity-005-jit-it-approval-protocol.md:27-32`).
07. Outbound cross-reference to OpenBao and HSM custody is explicit in ADR-ID-001 (`decisions/ADR-ID-001...md:64-65`, `decisions/ADR-ID-001...md:145-146`).
08. Outbound cross-reference to Zitadel is explicit in PRD and IP-001 (`PRD.md:1449-1456`; `IP-001-zitadel-helm-per-pack.md:17-38`).
09. Outbound cross-reference to Okta/Auth0/Entra is explicit in PRD benchmark/comparator sections (`PRD.md:70-86`).
10. Outbound cross-reference to regulatory packs is explicit in manifest and capability availability (`manifest.json:252-260`; `ADR-0330 and ADR-0331 tenant_class model:123-148`).
11. Outbound reference target check: `docs/standards/documentation-rigor.md` exists and applies retroactively to every microservice (`docs/standards/documentation-rigor.md:40-43`).
12. Outbound reference target check: `specs/master-plan-sequencing.json` exists and has the canonical six context IDs (`specs/master-plan-sequencing.json:704-745`).
13. Outbound reference target check: `docs/decisions/ADR-0328...md` exists and names six context IaC targets (`ADR-0328:1730-2210`).
14. Outbound reference target check: `docs/decisions/ADR-0039` is cited for sigstore/cosign by ADR-0328 D-20 but identity has no local OpenTofu module to sign (`ADR-0328:4196-4197`).
15. Outbound reference to `cloud-iac` is absent from local identity IaC despite ADR-0328 requiring cloud-iac plus OpenTofu for infrastructure provisioning (`ADR-0328:2095-2096`).
16. Outbound reference to deployment-context CI lanes is absent from identity docs; ADR-0328 requires per-context plan, contract, observability, IAM, billing, and failure fixtures (`ADR-0328:2154-2169`).
17. Inbound chat reference: Identity binds CI agents and deploy actors to tenant-scoped SPIFFE identities (`8f603fc7...jsonl:571`).
18. Inbound chat reference: Identity sits in the substrate dependency cascade under Tenancy and above Ontology (`8f603fc7...jsonl:602`).
19. Inbound chat reference: hot-path auth challenge/session state must be per-cell and isolation-capable (`8f603fc7...jsonl:732`).
20. Inbound chat reference: identity PRD expansion was explicitly recognized during prior doc waves (`8f603fc7...jsonl:4188-4206`).
21. Inbound docs reference from `docs/AGENTS.md` directs agents to root pointers and the operating contract; identity docs must stay within that workspace contract.
22. Inbound reference from documentation rigor: every microservice should have README and baseline docs, including architecture, runbooks, contracts, capabilities, dashboards, SLOs, IPs, catalog, IaC, and manifest (`docs/standards/documentation-rigor.md:64-79`).
23. Inbound reference from master plan: every microservice manifest requires supported OS metadata (`specs/master-plan-sequencing.json:777-815`).
24. Inbound reference from master plan: every microservice must obey Rust backend policy (`specs/master-plan-sequencing.json:817-855`).
25. Inbound reference from master plan: OCI Always Free requires per-microservice `iac/oci-guest/always-free/` (`specs/master-plan-sequencing.json:857-867`).
26. Orphan reference: identity architecture claims IaC evidence without a canonical OpenTofu module target.
27. Orphan reference: compliance claims OpenTofu inventory without OpenTofu files.
28. Orphan reference: PRD performance-budget notes reference absent docs.
29. Orphan reference: benchmark references result artifacts not present in inventory.
30. Orphan reference: IP-001 references `pack-us`, but inventory only shows `pack-us-healthcare`.
31. Missing reverse reference: no `cross-microservice-handoffs.md` means outbound-to-inbound handoff pairs are not centralized for downstream consumers.
32. Missing reverse reference: no supported OS manifest means OS claims cannot be consumed by packaging and CI owners.
33. Missing reverse reference: no per-context IaC module means cloud-iac cannot consume identity-specific context variables.
34. Missing reverse reference: no capability availability OCI note means cost and cloud-billing cannot consume demo_trial zero-cost constraints.
35. Missing reverse reference: no `src/` code means test plans cannot map to concrete crate paths.
36. Verified target: `contracts/openapi/identity.yaml` resolves and is substantial.
37. Verified target: `contracts/asyncapi/identity-events.yaml` resolves and is substantial.
38. Verified target: `contracts/proto/identity.proto` resolves and is substantial.
39. Verified target: `policy/*.cedar` fragments resolve and are in allowed file types.
40. Verified target: `runbooks/*.md` resolve for major incidents.
41. Verified target: `slos/*.openslo.yaml` resolve and align with service purpose.
42. Verified target: `dashboards/*.json` resolve, though dashboard semantic audit was not the main focus.
43. Verified target: `scorecards/*.json` resolve.
44. Verified target: `migration-playbooks/from-okta.md` resolves but is below rigor floor.
45. Verified target: `reference-implementations/webauthn-passkey-flow-rust-sdk.md` resolves and matches Rust preference.
46. Verified target: `onboarding/identity-engineer-first-week.md` resolves.
47. Verified target: `faqs/identity-engineer-faq.md` resolves.
48. Dimension risk: outbound references are numerous and mostly resolvable, but key canonical-direction references are either missing local implementation or wrong-direction.
49. Dimension severity: P1 for missing reverse handoff surface and OpenTofu orphan claims.
50. Dimension verdict: partial; product references resolve better than deployment references.

### 3.3 Dimension 3 - substance bar and intern-buildability

01. The PRD is above the documentation-rigor PRD floor by line count: 1642 lines, versus a 1500-line expectation in the rigor matrix summary and the task's own requirement to read it end to end.
02. The PRD gives a cold reader clear product purpose, personas, success metrics, protocols, and competitor weaknesses (`PRD.md:99-119`; `PRD.md:1051-1083`).
03. The PRD gives protocol-level details for OIDC issuer behavior, WebAuthn, SCIM, and federation (`PRD.md:1449-1507`).
04. The PRD includes explicit performance and availability budgets (`PRD.md:779-811`).
05. The OpenAPI contract is buildable enough for a first HTTP implementation of OIDC discovery, JWKS, token issuance, WebAuthn, SCIM, step-up, federation, and key rotation (`contracts/openapi/identity.yaml:49-462`).
06. The proto contract is buildable enough for internal verify and admin gRPC endpoints (`contracts/proto/identity.proto:15-99`).
07. The AsyncAPI contract is buildable enough for event class names and channels (`contracts/asyncapi/identity-events.yaml:32-75`, `contracts/asyncapi/identity-events.yaml:106-196`).
08. The ADR-ID-001 decision is buildable enough for passkey storage shapes, recovery envelope shapes, REST endpoints, Cedar permits, audit events, and verification tests (`decisions/ADR-ID-001...md:138-207`).
09. The failure-mode doc is buildable enough for first runbook branching across Postgres, JWKS, FIDO MDS, SCIM, Zitadel, DDoS, Cedar, residency, and OpenBao failures (`failure-modes.md:1-132`).
10. The incident-response doc is buildable enough for leaked key, tenant admin compromise, IdP outage, audit backlog, and post-incident handling (`incident-response.md:14-178`).
11. The capacity model gives concrete workload assumptions, per-replica throughput, quota ranges, and re-architecture triggers (`capacity-model.md:26-60`; `capacity-model.md:75-118`; `capacity-model.md:152-158`).
12. The compliance doc gives SOC2, ISO, NIST, evidence inventory, and control mapping detail (`compliance.md:15-117`).
13. Buildability gap: no `README.md` at the microservice root, despite documentation-rigor expecting README in every strategic/ops suite (`docs/standards/documentation-rigor.md:64-67`).
14. Buildability gap: no `supported-oses.json`, despite master plan requiring a per-microservice manifest (`specs/master-plan-sequencing.json:777-815`).
15. Buildability gap: no canonical context OpenTofu modules, despite ADR-0328 requiring all six context module paths (`ADR-0328:1730-2210`).
16. Buildability gap: no `iac/oci-guest/always-free/`, despite OCI Always Free demo_trial requiring that path (`specs/master-plan-sequencing.json:857-867`).
17. Buildability gap: no `src/` directory or crate layout in the identity path, so the contracts and tests cannot be tied to compile targets.
18. Buildability gap: manifest lists catalog records but no local Cargo manifest or crate source tree exists under identity.
19. Buildability gap: no CI lane file ties the OIDC/WebAuthn/SCIM tests to the canonical `cargo build --workspace --release --all-features --locked` invocation (`specs/master-plan-sequencing.json:853-854`).
20. Buildability gap: no per-context deployment variables, tenant onboarding input schema, or state backend wiring are present for any of six contexts.
21. Buildability gap: no plan-only IaC lane exists for context support claims, violating ADR-0328 D-15.125 (`ADR-0328:2154-2155`).
22. Buildability gap: no IAM fixture proves tenant principal access only in its allowed context, violating ADR-0328 D-15.128 (`ADR-0328:2162-2163`).
23. Buildability gap: no billing fixture proves usage attribution to tenant, context, cell, service, and tenant_class, violating ADR-0328 D-15.129 (`ADR-0328:2165-2166`).
24. Buildability gap: no failure fixture proves provider outage, cell loss, state-backend locking, quota exhaustion, or policy denial per context, despite ADR-0328 D-15.130 (`ADR-0328:2168-2169`).
25. Buildability gap: the benchmark doc provides numeric claims but not raw measurement artifacts, workloads, OS/arch, or tenant class disclosures at ADR-0328 D-20.152 depth (`ADR-0328:4208-4209`).
26. Buildability gap: the cost model includes vendor displacement numbers without enough citation to reproduce the pricing math (`cost-budget.md:123-146`).
27. Buildability gap: the migration playbook is only 192 lines, below the 500-line migration playbook rigor floor implied by the task and documentation-rigor doctrine.
28. Buildability gap: `ARCHITECTURE.md` is 880 lines and starts with an explicit anchor-sweep expansion note (`ARCHITECTURE.md:3`), below the deep architecture expectation and with a stub lineage marker.
29. Buildability gap: the architecture uses repeated boilerplate sections that assert evidence presence instead of showing context-specific module paths (`ARCHITECTURE.md:256-258`, `ARCHITECTURE.md:630-631`).
30. Buildability gap: no `cross-microservice-handoffs.md` centralizes handoff payloads to tenancy, policy-engine, audit-chain, governance, cloud-iam, cloud-iac, observability, or billing.
31. Buildability gap: the manifest lacks `deployment_contexts`, so a cold intern cannot know which contexts are supported, gated, or non-applicable.
32. Buildability gap: the manifest lacks `supported_oses`, so packaging and test matrix work must be inferred from ADR-0328.
33. Buildability gap: the demo_trial tenant_class cannot be implemented on OCI Always Free as written because its baseline machine envelope exceeds the canonical budget.
34. Buildability gap: `IP-016` prescribes non-Rust load tools without a Rust-strict exception ADR, so an intern following it would create forbidden build inputs.
35. Buildability gap: `IP-017` is too thin for the multi-context resolver it claims to complete; it lacks data model, state, failure semantics, API contract, CI lane, and deployment context details.
36. Buildability gap: `capabilities/multi-context-principal-resolve.yaml` scaffolded maturity conflicts with manifest GA status and leaves build-state ambiguous.
37. Buildability gap: per-OS package formats are not documented for RPM, DEB, `.pkg`, Homebrew, Talos extension, Flatcar extension, Photon, or container images.
38. Buildability gap: tenant_class-2 ppc64le/s390x test-only behavior is absent.
39. Buildability gap: out-of-scope OS declarations are absent.
40. Buildability gap: no explicit Rust backend build invocation is tied to Identity-specific artifacts.
41. Buildability gap: frontend scoping is not relevant because no `frontend/` subtree exists, but the absence should be declared rather than inferred.
42. Buildability gap: no OpenTofu state backend per context is documented locally.
43. Buildability gap: no sigstore/cosign signing wiring exists for OpenTofu modules, though scorecards mention cosign for supply-chain evidence.
44. Buildability gap: no tenant onboarding evidence event names exist for `requested`, `planned`, `approved`, `applied`, `verified`, and `rollback-ready` states.
45. Buildability gap: no context-specific ingress/DNS/certificate/WAF/rate-limit story exists outside generic Helm/Kustomize material.
46. Buildability gap: no context-specific audit and observability routing exists for all six contexts.
47. Buildability gap: no OCI Always Free downgrade path explains which demo_trial features become paid with per_seat billing_component.
48. Buildability gap: no local module explains disconnected on-prem or colo credential bootstrap.
49. Buildability conclusion: a cold intern could build the protocol surface, but not a compliant multi-context deployable service.
50. Dimension verdict: partial intern-buildability; product/protocol pass, canonical deployment substrate fails.

### 3.4 Dimension 4 - canonical-direction alignment

01. Multi-context alignment status: drifted-fixable.
02. Evidence: ADR-0328 requires six contexts and defaults Phase 0/1 services to all six (`ADR-0328:2116-2119`).
03. Evidence: identity is provider-control-plane prerequisite in context 6 (`ADR-0328:2015-2017`).
04. Evidence: master plan defines six canonical IDs and IaC targets (`specs/master-plan-sequencing.json:704-745`).
05. Drift: identity manifest does not declare all six context IDs.
06. Drift: identity IaC does not include `iac/oyatie-public-cloud/`.
07. Drift: identity IaC does not include `iac/guest-on-aws/`.
08. Drift: identity IaC does not include `iac/oci-guest/`.
09. Drift: identity IaC does not include `iac/oci-guest/always-free/`.
10. Drift: identity IaC does not include `iac/on-prem/`.
11. Drift: identity IaC does not include `iac/colo/`.
12. Drift: identity IaC does not include `iac/oyatie-iaas/`.
13. OpenTofu alignment status: incoherent at local module level.
14. Evidence: master plan engine is OpenTofu and forbids Terraform, Pulumi, and CloudFormation as primary (`specs/master-plan-sequencing.json:747-775`).
15. Evidence: ADR-0328 forbids context support claimed in prose without `iac/<context>/` module or N/A manifest (`ADR-0328:2198-2199`).
16. Drift: only Helm and Kustomize IaC exists in the identity path.
17. Positive: no Terraform, Pulumi, or CloudFormation files were found under identity.
18. Positive: no `null_resource`, `local-exec`, `remote-exec`, SSH provisioner, or tfstate pattern was found in identity IaC.
19. Negative: no `versions.tf` exists to pin OpenTofu and providers.
20. Negative: no state backend wiring exists per context.
21. Negative: no sigstore/cosign signing flow exists for OpenTofu modules.
22. OS alignment status: incoherent until a manifest is added.
23. Evidence: master plan requires tenant_class-1 OSes and per-microservice manifest (`specs/master-plan-sequencing.json:777-815`).
24. Drift: no `supported-oses.json` exists under identity.
25. Drift: no `supported_oses` field exists in `manifest.json`.
26. Drift: no tenant_class-1 OS list is declared locally.
27. Drift: no tenant_class-2 ppc64le/s390x test-only declaration is local.
28. Drift: no out-of-scope OS declaration is local.
29. Drift: no package format matrix is local.
30. Rust-strict alignment status: aligned for existing files, drifted for prescribed future test tooling.
31. Evidence: master plan backend language policy is Rust strict and only allows specific non-Rust extensions (`specs/master-plan-sequencing.json:817-855`).
32. Positive: grep found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, or similar source files under identity.
33. Positive: existing non-Rust files are Markdown, YAML, JSON, proto, Cedar, and OpenSLO-style YAML, all broadly within the allowed extension family.
34. Negative: IP-016 prescribes `k6-*.js`, `run.sh`, and Go-based tools as future load-test inputs (`IP-016-zitadel-scale-validation-load-test.md:49-68`).
35. OCI Always Free alignment status: incoherent for demo_trial.
36. Evidence: master plan maps OCI Always Free demo_trial to Always Free and requires `iac/oci-guest/always-free/` (`specs/master-plan-sequencing.json:857-867`).
37. Drift: no identity `iac/oci-guest/always-free/` exists.
38. Drift: tenant_class adoption matrix has no OCI Always Free demo_trial Always Free note.
39. Drift: demo_trial hardware exceeds Always Free capacity (`ADR-0330 and ADR-0331 tenant_class model:15-24`).
40. Documentation-rigor alignment status: partial.
41. Positive: artifact count is high, with 237 files and 74,456 lines.
42. Positive: PRD, contracts, SLOs, runbooks, capability records, dashboards, and scorecards exist.
43. Negative: README missing.
44. Negative: cross-service handoff doc missing.
45. Negative: architecture has stub-lineage marker and below deep architecture floor.
46. Negative: migration playbook below expected depth.
47. Anti-pattern alignment: mostly good because there is no generated hollow body in existing docs, but repeated evidence assertions in architecture and compliance should be replaced with concrete module paths.
48. Canonical-direction priority: add machine-readable manifest/context/OS/IaC records before polishing prose.
49. Severity summary for dimension: P0 for context/OpenTofu/OS/OCI Always Free demo_trial; P1 for prescribed forbidden load tooling.
50. Dimension verdict: product identity direction is correct; canonical substrate direction is not yet enforced.

### 3.5 Dimension 5 - industry-counterpart parity

01. Headline parity finding: partial.
02. Auth0 public docs confirm Organizations for B2B customers, federated login flows, organization membership, roles, APIs, and machine-to-machine access (`https://auth0.com/docs/manage-users/organizations/organizations-overview`).
03. Auth0 public docs confirm MFA with push, SMS, voice, OTP, WebAuthn security keys, WebAuthn device biometrics, email, Duo, and recovery codes (`https://auth0.com/docs/secure/multi-factor-authentication`).
04. Auth0 public docs confirm inbound SCIM for enterprise SAML, OIDC, Okta Workforce, and Microsoft Entra connections (`https://auth0.com/docs/authenticate/protocols/scim/configure-inbound-scim`).
05. Auth0 public docs confirm attack protection features including bot detection, suspicious IP throttling, brute-force protection, and breached password detection (`https://auth0.com/docs/secure/attack-protection`).
06. Okta public docs confirm app integrations with OIDC, SAML, SWA, WS-Fed, SCIM, OAuth service integrations, and Okta Integration Network submission paths (`https://developer.okta.com/docs/guides/create-an-app-integration/scim/main/`).
07. Okta public docs confirm MFA factor classes, authenticators, passkeys, Smart Card, Okta Verify, Okta FastPass, temporary access code, and YubiKey support (`https://help.okta.com/oie/en-us/content/topics/identity-engine/authenticators/about-authenticators.htm`).
08. Okta public docs confirm SCIM user lifecycle CRUD, deprovisioning, profile mapping, and profile sourcing semantics (`https://developer.okta.com/docs/concepts/scim/`).
09. Okta public docs confirm Identity Governance, lifecycle management, workflows, access certifications, access requests, entitlement management, resource owners, labels, and separation-of-duties rules (`https://help.okta.com/oie/en-us/content/topics/identity-governance/iga.htm`).
10. Microsoft public docs confirm Conditional Access with assignments, users/groups, target resources, network, conditions, sign-in risk, device platform, and access controls (`https://learn.microsoft.com/en-us/entra/identity/conditional-access/concept-conditional-access-policies`).
11. Microsoft public docs confirm Entra passkeys/FIDO2, synced passkeys, device-bound passkeys, attestation, AAGUID restrictions, and group-targeted passkey profiles (`https://learn.microsoft.com/en-us/entra/identity/authentication/how-to-authentication-passkeys-fido2`).
12. Microsoft public docs confirm Entra provisioning through SCIM 2.0 for automatic create/update/remove of users and groups (`https://learn.microsoft.com/en-us/entra/identity/app-provisioning/how-provisioning-works`).
13. Microsoft public docs confirm Identity Protection unified risk signals that can trigger risk-based Conditional Access (`https://learn.microsoft.com/en-us/entra/id-protection/id-protection-dashboard`).
14. Oyatie Identity present capability: OIDC/OAuth/JWKS via OpenAPI (`contracts/openapi/identity.yaml:49-83`).
15. Oyatie Identity present capability: WebAuthn/passkey via OpenAPI and ADR (`contracts/openapi/identity.yaml:118-200`; `decisions/ADR-ID-001...md:47-74`).
16. Oyatie Identity present capability: SCIM users/groups via OpenAPI (`contracts/openapi/identity.yaml:234-372`).
17. Oyatie Identity present capability: step-up/ACR via OpenAPI and ADR-identity-005 (`contracts/openapi/identity.yaml:374-423`; `decisions/ADR-identity-005...md:11-70`).
18. Oyatie Identity present capability: external IdP federation bindings via OpenAPI (`contracts/openapi/identity.yaml:425-446`).
19. Oyatie Identity present capability: AsyncAPI audit/event surface (`contracts/asyncapi/identity-events.yaml:1-196`).
20. Oyatie Identity present capability: tenant-scoped Cedar policy fragments (`policy/*.cedar` inventory).
21. Oyatie Identity present capability: runbooks for brute force, IdP failover, JWKS rotation, passkey reset, replay attack, and SCIM debug.
22. Oyatie gap: no self-service organization admin API depth comparable to Auth0 Organizations API detail, beyond roles and federation narrative.
23. Oyatie gap: no outbound SCIM lifecycle automation comparable to Okta and Entra; local competitor matrix already admits outbound SCIM deferred (`competitor-parity-matrix.md:47-49`).
24. Oyatie gap: continuous risk scoring is partial/deferred; local competitor matrix admits CAEP/risk scoring deferred to IP-014 (`competitor-parity-matrix.md:38-40`).
25. Oyatie gap: no Identity Governance equivalent for access certifications, access requests, entitlement management, and separation-of-duties campaign operations comparable to Okta Governance or Microsoft ID Governance.
26. Oyatie gap: no broad device posture management equivalent to Okta FastPass device assurance or Microsoft Conditional Access device/platform signals.
27. Oyatie gap: no rich admin product UI surface documented for policy, factor enrollment, recovery, organization admin, SCIM configuration, and logs.
28. Oyatie gap: no integration gallery/OIN equivalent for publishing identity integrations.
29. Oyatie gap: no tenant-level rate limit admin dashboard comparable to Okta rate-limit dashboard or Auth0 Management API limit surfaces.
30. Oyatie gap: no documented social login matrix beyond named provider list.
31. Oyatie gap: no delegated admin model at Auth0 Organizations level, despite roles and self-management being counterpart capabilities.
32. Oyatie gap: no workload identity product surface comparable to Microsoft Entra Workload ID; service-to-service SPIFFE doctrine exists but productized workload identity docs are thinner.
33. Oyatie gap: no privileged identity management equivalent beyond JIT approval protocol.
34. Oyatie gap: no access review/certification campaign equivalent.
35. Oyatie gap: no user risk remediation journey equivalent to Entra Identity Protection.
36. Oyatie gap: no passwordless compatibility matrix across browsers/native apps/OSes comparable to Entra FIDO2 compatibility.
37. Oyatie gap: no managed connector catalog comparable to Okta OIN or Entra gallery.
38. Oyatie additive surface: stronger sovereign-pack and air-gap self-hosting direction than Auth0/Okta hosted defaults (`competitor-parity-matrix.md:62-70`).
39. Oyatie additive surface: audit-chain Merkle/Ed25519 seal intent exceeds flat log claims in local competitor matrix (`competitor-parity-matrix.md:72-81`).
40. Oyatie additive surface: EU AI Act capability tagging appears in local matrix as an Oyatie-only feature (`competitor-parity-matrix.md:79-81`).
41. Oyatie additive surface: personal/work dual-context boundary and survivor/minor/emergency journeys are deeper than generic counterpart docs (`PRD.md:740-749`; `IP-journey-j*.md` inventory).
42. Oyatie additive surface: tenant-scoped principal doctrine for internal operators and CI agents is stronger than generic hosted IdP assumptions (`8f603fc7...jsonl:552-571`).
43. Counterpart parity risk: local `competitor-parity-matrix.md` claims parity on every must-have feature (`competitor-parity-matrix.md:98-100`), but this audit finds missing governance, outbound SCIM, risk, admin, and deployment evidence.
44. Finding severity: P1 because parity overclaim can distort Wave 14 aggregation.
45. Finding detail: competitor matrix should downgrade "parity on every must-have" to "partial parity with additive sovereignty/audit strengths."
46. Required implementation hook: extend contracts for organization admin, outbound SCIM, risk events, entitlement reviews, and device posture.
47. Required documentation hook: add capability records and IPs for governance parity surfaces, not just identity protocol surfaces.
48. Required deployment hook: counterpart parity cannot be claimed until multi-context deployability is real.
49. Dimension severity: P1 for overclaim, P2 for missing parity details.
50. Dimension verdict: partial union coverage; strongest in auth protocols, weakest in governance and product operations.

### 3.6 Dimension 6 - multi-context deployment support

01. Canonical requirement: six contexts are mandatory unless the service manifest marks explicit N/A reasons (`ADR-0328:1730-2210`; `specs/master-plan-sequencing.json:704-745`).
02. Context 1 `oyatie-public-cloud`: identity should be supported because every managed GA service needs identity/IAM seams.
03. Context 1 evidence found: no `microservices/identity/iac/oyatie-public-cloud/`.
04. Context 1 status: unsupported locally; missing IaC; no N/A reason.
05. Context 1 remediation: add OpenTofu module plus manifest declaration and CI plan lane.
06. Context 2 `guest-on-aws`: identity should be supported because AWS primitives must remain backing resources behind Oyatie identity.
07. Context 2 evidence found: no `microservices/identity/iac/guest-on-aws/`.
08. Context 2 status: unsupported locally; missing IaC; no N/A reason.
09. Context 2 remediation: add OpenTofu module using portable identity variables, not AWS IAM as app authority.
10. Context 3 `guest-on-oci`: identity should be supported because OCI guest includes demo, sandbox, trial, and dev tenants.
11. Context 3 evidence found: no `microservices/identity/iac/oci-guest/`.
12. Context 3 status: unsupported locally; missing IaC; no N/A reason.
13. Context 3 remediation: add OCI OpenTofu module and Always Free sub-profile.
14. Context 3 Always Free evidence found: no `microservices/identity/iac/oci-guest/always-free/`.
15. Context 3 Always Free status: unsupported and tenant_class-incoherent.
16. Context 4 `on-prem`: identity should be supported because regulated enterprise, healthcare, sovereign, and disconnected workflows need local identity integration (`ADR-0328:1913-1915`).
17. Context 4 evidence found: no `microservices/identity/iac/on-prem/`.
18. Context 4 status: unsupported locally; missing IaC; no N/A reason.
19. Context 4 remediation: add module for customer-controlled facility, local IdP/HSM, disconnected audit, and portable storage.
20. Context 5 `colo`: identity should be supported for sovereign cell, dedicated hardware, regulated low-latency, and facility-owned operation (`ADR-0328:1962-1968`).
21. Context 5 evidence found: no `microservices/identity/iac/colo/`.
22. Context 5 status: unsupported locally; missing IaC; no N/A reason.
23. Context 5 remediation: add module for Cilium/BGP/MetalLB/facility telemetry and HSM custody.
24. Context 6 `oyatie-as-cloud-provider`: identity is explicitly required as provider-control-plane prerequisite (`ADR-0328:2015-2017`).
25. Context 6 evidence found: no `microservices/identity/iac/oyatie-iaas/`.
26. Context 6 status: unsupported locally; missing IaC; no N/A reason.
27. Context 6 remediation: model identity as provider identity/security service, not cloud adapter.
28. Forbidden pattern check: no provider SDK calls were found in identity business logic because no source code exists under identity.
29. Forbidden pattern check: no `terraform`, `pulumi`, or `cloudformation` file was found under identity.
30. Forbidden pattern check: no `null_resource`, `local-exec`, `remote-exec`, or SSH provisioner was found under identity IaC.
31. Forbidden pattern: context support appears in prose without `iac/<context>/` modules; ADR-0328 explicitly forbids this pattern (`ADR-0328:2198-2199`).
32. Forbidden pattern: README-only manual setup cannot exist because README is absent, but the absence still prevents context support.
33. Tenant onboarding evidence: absent for all six contexts.
34. Tenant onboarding variables: absent for all six contexts.
35. Tenant onboarding signed plan artifact: absent for all six contexts.
36. Tenant onboarding audit events: absent for requested/planned/approved/applied/verified/rollback-ready states.
37. Observability routing per context: absent.
38. IAM fixture per context: absent.
39. Billing fixture per context: absent.
40. Failure-mode fixture per context: absent.
41. Context support conclusion: all six contexts are required, zero have canonical local IaC support.
42. Correctly N/A contexts: none identified.
43. Wrongly implied support: architecture/compliance imply deployment evidence exists but only Helm/Kustomize exists.
44. P0 finding: missing all six context modules for T0 identity.
45. P1 finding: manifest lacks context declarations and N/A reason fields.
46. P1 finding: no per-context CI lane evidence.
47. P1 finding: no context-specific ingress/DNS/cert/WAF/rate-limit behavior.
48. P1 finding: no context-specific tenant onboarding events.
49. Dimension severity: P0.
50. Dimension verdict: not aligned; all six context claims are blocked until local OpenTofu modules or explicit N/A reasons exist.

### 3.7 Dimension 7 - OpenTofu IaC coverage

01. Canonical requirement: OpenTofu is the engine and Terraform/Pulumi/CloudFormation are forbidden as primary engines (`specs/master-plan-sequencing.json:747-775`).
02. Canonical requirement: each context module must include `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and README (`ADR-0328:1730-2210`; `ADR-0328:4177-4218`).
03. Existing IaC directory: `iac/helm/zitadel/`.
04. Existing IaC directory: `iac/kustomize/components/edge-authz-rules/`.
05. Existing IaC directory: `iac/kustomize/overlays/pack-ae/`.
06. Existing IaC directory: `iac/kustomize/overlays/pack-eu/`.
07. Existing IaC directory: `iac/kustomize/overlays/pack-kr/`.
08. Existing IaC directory: `iac/kustomize/overlays/pack-ksa/`.
09. Existing IaC directory: `iac/kustomize/overlays/pack-us-healthcare/`.
10. Missing context module: `iac/oyatie-public-cloud/`.
11. Missing context module: `iac/guest-on-aws/`.
12. Missing context module: `iac/oci-guest/`.
13. Missing context module: `iac/oci-guest/always-free/`.
14. Missing context module: `iac/on-prem/`.
15. Missing context module: `iac/colo/`.
16. Missing context module: `iac/oyatie-iaas/`.
17. Existing Helm file: `iac/helm/zitadel/Chart.yaml`.
18. Existing Helm file: `iac/helm/zitadel/values.yaml`.
19. Existing Helm templates: configmap, deployment, HPA, ingress, networkpolicy, PDB, service, serviceaccount.
20. Existing Kustomize component files: Coraza WAF config, DDoS XDP policy, geo/ASN block, rate-limit config, and kustomization.
21. Existing Kustomize overlays: pack AE, EU, KR, KSA, and US healthcare values.
22. Positive evidence: Helm/Kustomize surfaces support Kubernetes deployment composition.
23. Negative evidence: Helm/Kustomize do not satisfy ADR-0328 OpenTofu substrate.
24. Negative evidence: no OpenTofu provider locks.
25. Negative evidence: no OpenTofu variable schema.
26. Negative evidence: no OpenTofu outputs.
27. Negative evidence: no OpenTofu state backend.
28. Negative evidence: no OpenTofu module README.
29. Negative evidence: no module signing metadata.
30. Negative evidence: no `tofu init`, `tofu plan`, or `tofu apply` workflow documented locally.
31. Search result: no Terraform engine files or Terraform Cloud use observed.
32. Search result: no Pulumi references observed.
33. Search result: no CloudFormation references observed.
34. Search result: no `null_resource` observed.
35. Search result: no `local-exec` observed.
36. Search result: no `remote-exec` observed.
37. Search result: no SSH provisioner observed.
38. Search result: no hand-edited tfstate observed.
39. Sigstore/cosign evidence: compliance references cosign and Sigstore (`compliance.md:866-878`, `compliance.md:897-907`).
40. Sigstore/cosign gap: no context module signing wiring exists for OpenTofu modules.
41. Scorecard evidence: `scorecards/cis-k8s-v1-10.json` and `scorecards/slsa-l3.json` mention cosign verification/signing in search results.
42. Scorecard gap: scorecards do not replace OpenTofu module signing.
43. State backend requirement: AWS should use S3+DynamoDB lock (`specs/master-plan-sequencing.json:758-765`).
44. State backend requirement: OCI should use Object Storage plus Autonomous DB lock (`specs/master-plan-sequencing.json:758-765`).
45. State backend requirement: on-prem and colo should use MinIO plus lock table (`specs/master-plan-sequencing.json:758-765`).
46. State backend requirement: Oyatie public cloud should use internal OCI (`specs/master-plan-sequencing.json:758-765`).
47. State backend requirement: Oyatie-as-cloud-provider should use internal `cloud-storage` (`specs/master-plan-sequencing.json:758-765`).
48. Finding: zero of five state backend mappings are present locally.
49. Dimension severity: P0 because identity is required in provider-control-plane context and has zero canonical OpenTofu modules.
50. Dimension verdict: not aligned; Helm/Kustomize are useful but insufficient.

### 3.8 Dimension 8 - OS support matrix

01. Canonical requirement: every microservice must have per-microservice OS manifest evidence (`specs/master-plan-sequencing.json:777-815`; `ADR-0328:4220-4221`).
02. Manifest check: `microservices/identity/supported-oses.json` is absent.
03. Manifest check: `manifest.json` has no `supported_oses` field.
04. tenant_class-1 OS `talos`: not declared locally.
05. tenant_class-1 OS `rhel-9.x+`: not declared locally.
06. tenant_class-1 OS `oracle-linux-9.x+`: not declared locally.
07. tenant_class-1 OS `sles-15-sp6+`: not declared locally.
08. tenant_class-1 OS `ubuntu-24.04-lts+`: not declared locally.
09. tenant_class-1 OS `debian-13+`: not declared locally.
10. tenant_class-1 OS `rocky-9.x+`: not declared locally.
11. tenant_class-1 OS `almalinux-9.x+`: not declared locally.
12. tenant_class-1 OS `centos-stream-10+`: not declared locally.
13. tenant_class-1 OS `amazon-linux-2023+`: not declared locally.
14. tenant_class-1 OS `flatcar`: not declared locally.
15. tenant_class-1 OS `photon-5.x+`: not declared locally.
16. tenant_class-1 OS `macos-apple-silicon-m5+`: not declared locally.
17. tenant_class-2 OS `linux-ppc64le`: not declared test-only locally.
18. tenant_class-2 OS `linux-s390x`: not declared test-only locally.
19. Out-of-scope `macos-intel`: not explicitly excluded locally.
20. Out-of-scope `macos-apple-silicon-pre-m5`: not explicitly excluded locally.
21. Out-of-scope `freebsd`: not explicitly excluded locally.
22. Out-of-scope `openbsd`: not explicitly excluded locally.
23. Out-of-scope `windows-server`: not explicitly excluded locally.
24. Out-of-scope `solaris`: not explicitly excluded locally.
25. Package format `RPM`: not declared.
26. Package format `DEB`: not declared.
27. Package format `.pkg`: not declared.
28. Package format `Homebrew`: not declared.
29. Package format `Talos extension`: not declared.
30. Package format `Flatcar extension`: not declared.
31. Package format `Photon image/package`: not declared.
32. Package format `container image`: implied by Helm/Kubernetes, not declared as OS support matrix.
33. CI lane for Talos: absent.
34. CI lane for RHEL: absent.
35. CI lane for Oracle Linux: absent.
36. CI lane for SLES: absent.
37. CI lane for Ubuntu: absent.
38. CI lane for Debian: absent.
39. CI lane for Rocky: absent.
40. CI lane for AlmaLinux: absent.
41. CI lane for CentOS Stream: absent.
42. CI lane for Amazon Linux: absent.
43. CI lane for Flatcar: absent.
44. CI lane for Photon: absent.
45. CI lane for macOS M5+: absent.
46. Positive evidence: identity docs mention tenant_class-1 substrate concept in security and architecture, showing awareness (`security/threat-model.md:25`; `ARCHITECTURE.md:328`).
47. Negative evidence: awareness is not a manifest and cannot feed CI/package automation.
48. Finding: OS support cannot be classified by platform until the manifest exists.
49. Dimension severity: P0 because ADR-0328 D-20.156 requires missing-manifest finding for every coherence audit.
50. Dimension verdict: not aligned; missing manifest.

### 3.9 Dimension 9 - Rust-strict language coverage

01. Canonical requirement: backend/runtime/CLI/validation/codegen/scripting/CI durable behavior is Rust (`specs/master-plan-sequencing.json:817-855`).
02. Canonical allowed extensions include `.tf`, `.cedar`, `.yaml`, `.json`, `.proto`, `.openapi.yaml`, `.asyncapi.yaml`, `.openslo.yaml`, `.sql`, and `.md` (`specs/master-plan-sequencing.json:828-839`).
03. Existing forbidden-source grep result: no `.py` files found under `microservices/identity/`.
04. Existing forbidden-source grep result: no `.js` files found under `microservices/identity/`.
05. Existing forbidden-source grep result: no `.ts` files found under `microservices/identity/`.
06. Existing forbidden-source grep result: no `.rb` files found under `microservices/identity/`.
07. Existing forbidden-source grep result: no `.go` files found under `microservices/identity/`.
08. Existing forbidden-source grep result: no `.java` files found under `microservices/identity/`.
09. Existing forbidden-source grep result: no `.scala` files found under `microservices/identity/`.
10. Existing forbidden-source grep result: no `.groovy` files found under `microservices/identity/`.
11. Existing forbidden-source grep result: no `.php` files found under `microservices/identity/`.
12. Existing forbidden-source grep result: no F# files found under `microservices/identity/`.
13. Existing forbidden-source grep result: no C/C++ source files found under `microservices/identity/`.
14. Existing forbidden-source grep result: no `package.json` found under `microservices/identity/`.
15. Existing forbidden-source grep result: no `pyproject.toml` found under `microservices/identity/`.
16. Existing forbidden-source grep result: no `go.mod` found under `microservices/identity/`.
17. Existing forbidden-source grep result: no Maven or Gradle build files found under `microservices/identity/`.
18. Existing files include Markdown; authorized as documentation.
19. Existing files include YAML; authorized for contracts, SLOs, capabilities, Helm, Kustomize, and policy records.
20. Existing files include JSON; authorized for manifest, dashboards, and scorecards.
21. Existing files include proto; authorized for gRPC contracts.
22. Existing files include Cedar; authorized for policy fragments.
23. Existing files include OpenSLO YAML; authorized as SLO documents.
24. Existing files include Helm/Kustomize YAML; allowed file type but not sufficient for OpenTofu coverage.
25. Existing frontend code: no `frontend/` subtree found.
26. Existing Swift code: none.
27. Existing Kotlin code: none.
28. Existing WinUI3 code: none.
29. Existing web Leptos code: none under identity path.
30. Build invocation required by master plan: `cargo build --workspace --release --all-features --locked` (`specs/master-plan-sequencing.json:853-854`).
31. Local build invocation gap: no identity-specific build doc ties that invocation to crate paths.
32. Local source gap: no `src/` directory exists under identity.
33. Local Cargo gap: no `Cargo.toml` exists under identity.
34. Catalog gap: catalog records imply Rust crates, but local crate source is not present under identity.
35. Authorized SDK evidence: `reference-implementations/webauthn-passkey-flow-rust-sdk.md` is Rust-aligned.
36. Forbidden future-path evidence: `migration-playbooks/from-okta.md` claims SDKs in TypeScript, Python, and Go (`migration-playbooks/from-okta.md:183-183`).
37. Forbidden future-path evidence: `IP-016-zitadel-scale-validation-load-test.md` prescribes `k6-*.js` (`IP-016...md:49-68`).
38. Forbidden future-path evidence: `IP-016` prescribes `run.sh` scripts.
39. Forbidden future-path evidence: `IP-016` prescribes Go-based `vegeta`.
40. ADR decision-tree evidence: docs prescribing a violating future path must be classified according to that path (`ADR-0328:4170-4172`).
41. Classification for current file corpus: aligned.
42. Classification for IP-016 future implementation path: drifted-fixable, P1.
43. Classification for migration playbook SDK statement: drifted-fixable, P1.
44. Classification for contracts/codegen: no generated SDK output observed.
45. Classification for Terraform/OpenTofu: no `.tf` files observed; absence is an OpenTofu coverage issue, not a language violation.
46. Classification for YAML IaC: allowed file type but not canonical engine compliance.
47. Remediation: rewrite load-test plan to use Rust harnesses or a documented exception ADR.
48. Remediation: rewrite migration playbook SDK claims to Rust-only backend and clearly scoped frontend/mobile generated clients if allowed.
49. Dimension severity: P1 because forbidden future implementation paths exist, despite current files being clean.
50. Dimension verdict: current corpus passes grep; prescribed future tooling does not.

## 4. Findings summary

| Severity | Dimension | Short description | Provenance | Remediation hint |
|---|---|---|---|---|
| P0 | 6/7 | No canonical OpenTofu context modules for all six required contexts | `ADR-0328:1730-2210`; identity `iac/` inventory | Add `iac/oyatie-public-cloud`, `guest-on-aws`, `oci-guest`, `oci-guest/always-free`, `on-prem`, `colo`, `oyatie-iaas` with OpenTofu files |
| P0 | 8 | Missing OS support manifest | `specs/master-plan-sequencing.json:777-815`; no `microservices/identity/supported-oses.json` | Add `supported-oses.json` and mirror manifest field |
| P0 | 4/6 | OCI Always Free demo_trial does not map to Always Free | `specs/master-plan-sequencing.json:857-867`; `ADR-0330 and ADR-0331 tenant_class model:15-24` | Split OCI Always Free demo_trial Always Free profile and move excess capacity to paid with per_seat billing_component |
| P0 | 1 | Multi-context resolver maturity conflict | `manifest.json:233-250`; `capabilities/multi-context-principal-resolve.yaml:9-12` | Align manifest GA status with capability maturity or finish resolver docs/contracts/tests |
| P1 | 1/7 | Architecture/compliance claim IaC evidence broader than inventory | `ARCHITECTURE.md:256-258`; `compliance.md:866-878` | Replace evidence claims with concrete module paths or downgrade claim |
| P1 | 2 | Missing cross-microservice handoff file | no `cross-microservice-handoffs.md`; `docs/standards/documentation-rigor.md:121-129` | Add handoff doc for tenancy, policy-engine, audit-chain, governance, observability, cloud-iac |
| P1 | 3 | Architecture still carries anchor-sweep/stub marker and is below deep architecture bar | `ARCHITECTURE.md:3`; `ARCHITECTURE.md:1-880` | Expand architecture into deployable control/data plane doc |
| P1 | 3/5 | Competitor parity matrix overclaims complete must-have parity | `competitor-parity-matrix.md:98-100` | Downgrade to partial parity and add missing governance/risk/outbound SCIM work |
| P1 | 3 | Benchmark doc presents numeric claims without observed raw result artifacts | `benchmarks/okta-auth0-entra-vs-oyatie.md:19-119` | Add raw result paths, methodology, OS/arch/context/tenant class |
| P1 | 9 | IP-016 prescribes JS/shell/Go load-test artifacts | `IP-016-zitadel-scale-validation-load-test.md:49-68`; `ADR-0328:4170-4172` | Rewrite to Rust load harness or add justified exception ADR |
| P1 | 9 | Migration playbook claims TypeScript/Python/Go SDKs | `migration-playbooks/from-okta.md:183-183` | Restrict backend SDK claims to Rust and explicitly scope any generated client exceptions |
| P1 | 4/6 | Manifest lacks deployment context declarations | `manifest.json:1-433`; `specs/master-plan-sequencing.json:704-745` | Add `deployment_contexts` with support/N/A/blocked states |
| P1 | 7 | No OpenTofu state backend or sigstore signing wiring | `specs/master-plan-sequencing.json:756-765`; `ADR-0328:4196-4218` | Add backend blocks and cosign/sigstore evidence per context |
| P2 | 1 | IP-001 references `pack-us`, actual overlay appears `pack-us-healthcare` | `IP-001-zitadel-helm-per-pack.md:21-38`; `iac/kustomize/overlays/` inventory | Correct pack name or add missing overlay |
| P2 | 3 | Migration playbook below rigor depth | `migration-playbooks/from-okta.md:1-192`; documentation-rigor doctrine | Expand to full migration runbook/playbook |
| P2 | 3 | PRD references performance-budget notes not present locally | `PRD.md:779-793` | Add referenced performance budget docs or update PRD to current benchmark source |
| P2 | 2 | Wrong-direction self-reference `identity` consumes `identity` | `ARCHITECTURE.md:290-296`; `compliance.md:153-159` | Replace with actual consuming services and reverse references |
| P2 | 8 | OS mentions are awareness only, not enforceable manifest | `ARCHITECTURE.md:328`; `security/threat-model.md:25` | Move OS matrix to machine-readable manifest |
| P2 | 5 | Missing productized integration-gallery/admin-console parity | public counterpart docs; `contracts/openapi/identity.yaml:1-641` | Add admin APIs, capability records, and IPs |
| P2 | 5 | Missing identity governance parity | Okta Governance docs; Microsoft Governance docs; no local equivalent | Add access reviews, access requests, entitlements, SOD |
| P3 | 3 | No local source tree under identity path | inventory | Clarify whether source lives in shared crates or add path pointers |
| P3 | 3 | No frontend subtree; acceptable but undeclared | inventory | Declare no frontend-owned code in manifest |
| P3 | 7 | Helm/Kustomize useful but not classified against OpenTofu contexts | `iac/helm/`; `iac/kustomize/` inventory | Keep as Kubernetes layer under OpenTofu-managed context modules |
| P3 | 5 | Social IdP support named but not matrixed | `PRD.md:1482-1507` | Add provider-by-provider capability record |

Severity totals:
- P0 findings: 4.
- P1 findings: 9.
- P2 findings: 7.
- P3 findings: 4.

## 5. Open questions for Wave 14 aggregation

1. Should Identity's source code be expected under `microservices/identity/src/`, or is the canonical source of implementation the shared crate catalog elsewhere in the repo?
2. Should the missing `cross-microservice-handoffs.md` be a service-local artifact for every substrate service, or will a generated handoff manifest replace it?
3. Should `capabilities/multi-context-principal-resolve.yaml` remain scaffolded while manifest says IP-017 is GA, or should Wave 14 downgrade IP-017 until implementation evidence exists?
4. Should demo_trial mean "generic demo_trial" across all contexts and "OCI Always Free demo_trial Always Free" only in OCI, or should the tenant_class adoption matrix define an explicit OCI sub-tenant_class?
5. Should load testing be implemented with a Rust-native harness only, or is a narrowly justified non-Rust benchmarking exception acceptable for external tools?
6. Should `competitor-parity-matrix.md` be updated in this wave to remove complete parity claims, or should parity correction wait for the feature-parity matrix aggregation?
7. Should every microservice adopt the exact path `iac/oyatie-iaas/` for context 6, given master plan text uses that path while the context id is `oyatie-as-cloud-provider`?
8. Should `pack-us` be added as a distinct overlay or should IP-001 be corrected to `pack-us-healthcare`?
9. Should Auth0/Okta/Entra governance parity live in Identity or be split with Governance and Policy Engine, with Identity only owning principal/session events?
10. Should OS support manifest be authored by each microservice owner or generated from a repo-level packaging matrix?

<!-- ORCHESTRATOR REPORT
  µservice: identity
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/identity/coherence-audit-2026-05-20.md (690 lines)
    - /Users/jasonlee/oyatie/microservices/identity/feature-parity-matrix-2026-05-20.md (416 lines)
    - /Users/jasonlee/oyatie/microservices/identity/performance-benchmark-numbers-2026-05-20.md (307 lines)
    - /Users/jasonlee/oyatie/microservices/identity/capability-adoption-deltas-vs-counterparts-2026-05-20.md (366 lines)
  inventory_files_seen: 237
  inventory_lines_read: 74456
  chat_history_matches_processed: 56
  findings_p0: 4
  findings_p1: 9
  findings_p2: 7
  findings_p3: 4
  top_3_counterparts_confirmed: Auth0 / Okta / Microsoft Entra ID (identity platform)
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1779
-->
