# Hyperscaler-pattern attribution matrix (consolidated)

> Consolidated source-of-truth for the F2 (hyperscaler-fitness) facet of
> multispectrum review v2.4.0. Aggregates every per-ADR Appendix A row from
> the 14-ADR keystone bundle (ADR-0242 through ADR-0255) into a single
> reviewable matrix.
>
> **Authority:** required appendix referenced by every keystone ADR. Every
> new ADR (≥0256) MUST extend the matrix below per the maintenance protocol
> in §8.
>
> **Last verified:** 2026-05-20 against ADR-0242..ADR-0255 Appendix A.
>
> **Cross-cuts:** ADR-0242 (oyatie-as-tenant), ADR-0243 (Cedar-as-universal-
> gate), ADR-0244 (tenant scoping), ADR-0245 (substrate-vs-product),
> ADR-0246 (policy-engine substrate), ADR-0247 (self-hosting),
> ADR-0248 (Amazon-shape cellular), ADR-0249 (multi-category marketplace),
> ADR-0250 (build-ahead-of-certification), ADR-0251 (compliance pack +
> cell certification levels), ADR-0252 (time + distributed consistency),
> ADR-0253 (network topology + edge + service mesh), ADR-0254 (deployment
> model spectrum), ADR-0255 (intelligence two-layer AI substrate).

---

## 1. Purpose

The 14-ADR keystone bundle established a portfolio-wide audit pattern:
**every architectural decision in a keystone ADR must declare a named
hyperscaler pattern, cite a public source, and identify the anti-pattern
that is being avoided.** That rule was originally introduced in the
ADR-0242 Appendix A and is now binding on every subsequent ADR.

The per-ADR Appendix A tables are authoritative for their decisions, but
they are scattered across 14 documents. Reviewers running the F2
(hyperscaler-fitness) facet of multispectrum review v2.4.0 should be able
to:

1. Look up any decision in the portfolio in O(1) and find its named
   pattern, its citation, and the anti-pattern it avoids.
2. See the full grouping by hyperscaler source (AWS Builder's Library,
   Stripe Engineering, Google SRE, etc.) so that source-density audits
   are tractable.
3. See which decisions claim "novel" patterns (no direct hyperscaler
   precedent) and verify that those novelty claims are defensible.
4. See per-ADR citation-quality scorecards so weak appendices are
   visible.

This document is the consolidated source-of-truth for those four needs.

The matrix is the single fan-out input to the F2 facet. F2 reviewers
DO NOT re-derive citations from scratch; they verify against this
matrix, then verify the matrix against the linked sources.

**Scope and non-scope.** This document is descriptive — it catalogues
attributions that already appear in the keystone ADRs. It does not
re-litigate the decisions themselves (that is the ADRs' job) and it does
not invent attributions (the per-ADR Appendix A tables remain
authoritative for their own decisions). When the matrix below
disagrees with a per-ADR Appendix A, the per-ADR Appendix A wins and
this document is the bug.

---

## 2. How to use this matrix

### 2.1 For F2 reviewers running multispectrum review v2.4.0

When a ChangeSet touches a decision rooted in ADR-0242..ADR-0255 (or
any subsequent ADR that has extended this matrix), the F2 facet review
loop is:

1. **Look up the decision ID** in §3 (master matrix). Decision IDs are
   formatted `ADR-XXXX D-Y` (e.g., `ADR-0248 D-7` = ADR-0248 decision
   section D-7).
2. **Verify the citation is real.** Each row carries a source citation
   like "Vogels 2016 '10 Lessons'" or "AWS Builder's Library 'Static
   Stability'". F2 should spot-check that the citation actually exists
   and that the named pattern is faithfully represented in it. If the
   citation is dead, file an F2 finding and propose a fresh source.
3. **Verify the anti-pattern characterisation.** The anti-pattern column
   is the load-bearing column for F2: it tells the reviewer what
   architectural failure mode is being closed. F2 should confirm that
   the proposed design genuinely avoids the named anti-pattern, not
   just decorates around it.
4. **Cross-reference §4 grouped views.** If a decision claims, e.g., an
   AWS Builder's Library pattern, §4.1 should also list that decision
   under AWS Builder's Library. Inconsistency between §3 and §4
   indicates a documentation bug.
5. **For novel-pattern claims:** consult §5. If the decision is in §5,
   verify the why-novel-but-defensible justification holds. Novel
   patterns require especially careful F2 review because there is no
   external precedent to lean on.

### 2.2 For ADR authors

When authoring a new ADR (≥0256) with a hyperscaler-pattern attribution
appendix:

1. Author your per-ADR Appendix A as before, with one row per decision.
2. Append your rows to §3 of this document in the same commit. Use the
   maintenance protocol in §8.
3. If your ADR cites a source not yet listed in §4, add the source
   bucket to §4. If your ADR claims a novel pattern, add it to §5 with
   defensibility justification.
4. Update §6 (citation-quality scorecard) and §7 (freshness audit)
   with your ADR's row counts.

### 2.3 For reviewers running source-density audits

The portfolio is healthier when citations span multiple hyperscaler
sources (avoiding mono-vendor risk) and when sources are current. §4
groups every row by source bucket; §7 audits source freshness by year.
Run those audits quarterly per §8 maintenance protocol.

---

## 3. Master matrix

The matrix has 205 rows covering every architectural decision in the
14-keystone bundle. Columns:

- **Decision ID:** `ADR-XXXX D-Y[.Z]` — references the section heading
  in the source ADR's Decision section.
- **Decision Summary:** one-line gist of what was decided (for context
  without re-reading the ADR).
- **Hyperscaler Pattern (named):** the canonical pattern name (often in
  the form a hyperscaler / academic paper would use).
- **Source Citation:** paper / blog / talk / RFC with year. Multiple
  citations indicate the pattern shows up across multiple
  hyperscalers — a stronger attribution than single-vendor.
- **Anti-Pattern Avoided:** the architectural failure mode that the
  decision is closing.

Rows are ordered by ADR number, then by decision number within ADR.

### 3.1 ADR-0242 — oyatie-is-a-tenant doctrine (8 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0242 D-1 | oyatie as canonical org-tenant slug | Eat-Your-Own-Dogfood at Platform Level | Vogels 2016 "10 Lessons"; Stripe Engineering Quora 2013; Apple WWDC 2024 keynote; Palantir Apollo product docs | Internal Carve-Out — bypass paths for platform-owner ops |
| ADR-0242 D-1.r | Reserved-namespace protection for `oyatie.*` | Reserved Identifier Namespace + IDN Homograph Defence | AWS `arn:aws:iam::aws:` reserved partition; UTS#39 Unicode Security; UTR#36 Security Considerations | Typosquatting Tenant Impersonation — third-party registers `oyatie-foo` to imply affiliation |
| ADR-0242 D-2 | Dotted hierarchical sub-scopes (`oyatie.security.<role>.<id>`) | Hierarchical Principal Path | AWS IAM principal ARN paths; GCP IAM resource hierarchy; Azure RBAC scope hierarchy | Flat Namespace Drift — inheritance + rollup require explicit cross-namespace queries |
| ADR-0242 D-3 | No internal-only µservices | Unified Multi-Tenant Substrate | Salesforce multi-tenant architecture; AWS shared-substrate model; Microsoft 365 multi-tenant Exchange Online | Audience-As-Service-Scope — explicitly retired by every named hyperscaler reference |
| ADR-0242 D-4 | Uniform compliance machinery covers oyatie too | Dogfooded Compliance Pipeline | Stripe SOC 2 includes Stripe's internal use; AWS Audit Manager covers AWS-on-AWS; Microsoft 365 includes Microsoft IT | Compliance Carve-Out — platform owner outside audit scope (regulator red flag) |
| ADR-0242 D-5 | Bootstrap sequence with audited replay | Audited Bootstrap Replay | rustc stage0 bootstrap; Kubernetes kubeadm certificate chain; Certificate Transparency log bootstrap | Untraceable Genesis — original deployment lacks audit trail |
| ADR-0242 D-6 | Reserved-namespace enforcement via Cedar | Defence-in-Depth via Cedar Fragment | AWS Service Control Policy enforcing partition; GCP Org Policy constraints | Application-Layer-Only Check — bypass via direct database write |
| ADR-0242 D-7 | `oyatie` tenant properties (first-class platform-owner) | First-Class Platform-Owner Account | AWS `aws` system account; GCP `google` system project; Microsoft "First-Party Tenant" pattern in Azure AD | Implicit Platform Account — undocumented service-principal sprawl |
| ADR-0242 D-8 | Sandbox + preview tenants | Ephemeral Tenant Pattern | Vercel preview deployments; Stripe test mode; Heroku review apps | Production-Only Testing — risk to live tenants from CI tests |

### 3.2 ADR-0243 — Cedar-as-universal-gate (13 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0243 D-1 | Cedar evaluates every policy-class decision | Single Policy Engine Consolidation | AWS Verified Permissions design; GCP Org Policy consolidation; Netflix OPA-at-scale | Multiple Policy Engines Drift — each subsystem hand-rolls policy |
| ADR-0243 D-2 | Fragment lifecycle (author → sign → publish → activate → sunset) | Signed Policy Authoring Lifecycle | AWS Verified Permissions policy store; Sigstore + cosign attestations | Imperative Policy Patching — ad-hoc policy changes without provenance |
| ADR-0243 D-3 | Minimum gate set per µservice | Coverage-Required Authorization | NIST SP 800-162 ABAC; AWS Well-Architected SEC | Implicit Permit — actions without explicit permit policy |
| ADR-0243 D-4 | Per-tenant overlay composition | Layered Policy Composition | AWS SCP + IAM intersection; Cedar fragment union | Per-Tenant Code Branch — per-tenant logic embedded in shared code |
| ADR-0243 D-5 | Bootstrap chain of trust | PKI Root + Intermediate Certificate Chain | RFC 5280 X.509; Sigstore Rekor; AWS KMS key hierarchy | Implicit Bootstrap Trust — undocumented signing key emergence |
| ADR-0243 D-6 | In-cell cache + sub-millisecond p99 evaluator | Edge-Cached Policy Evaluation | AWS Verified Permissions production cache; Cloudflare Workers KV | Synchronous Round-Trip to Global Policy Store — cross-region policy fetch on hot path |
| ADR-0243 D-7 | Audit emission on every decision | Audit-Every-Decision | NIST SP 800-92 audit log standards; SOC 2 CC7.2 | Audit Sampling — only some decisions audited |
| ADR-0243 D-8 | Multispectrum review integration for fragments | Multi-Facet Policy Review | oyatie multispectrum review v2.4.0 doctrine | Single-Reviewer Policy Change — drift via insufficient review |
| ADR-0243 D-9 | Coverage CI lane | Coverage-Enforced Policy | Google SRE Workbook ch. 4 (SLO coverage); AWS Config conformance packs | Untested Policy Surface — gates discovered missing in production |
| ADR-0243 D-10 | Hot-reload semantics for fragments | Hot-Reload Configuration Distribution | etcd watch pattern; Kubernetes ConfigMap watch; Apollo / Argo CD sync | Restart-To-Apply — policy changes require service restart |
| ADR-0243 D-11 | Fail-closed default + static stability | Static Stability + Fail-Closed | AWS Builder's Library "Static stability"; NIST SP 800-207 deny-by-default | Fail-Open on Policy Unavailable — security holes during outage |
| ADR-0243 D-12 | Per-tenant fragment authoring boundary | Restricted Tenant Self-Policy | AWS SCP + IAM permission boundary pattern | Tenant Privilege Escalation — tenant raises own permissions above baseline |
| ADR-0243 D-13 | Feature flag replacement via Cedar | Unified Policy + Feature Gate | AWS Verified Permissions docs explicitly cover feature gates | Separate Feature-Flag System — LaunchDarkly + similar |

### 3.3 ADR-0244 — tenant-as-universal-scoping-primitive (16 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0244 D-1 | Tenant ID format (DNS-segment-compatible slug) | Globally Unique Slug + DNS-Compatible Segments | RFC 1035; AWS account-alias rules; Stripe account ID conventions | Auto-Incrementing Integer Tenant ID — leaks customer count + ordering |
| ADR-0244 D-1.r | Reserved namespace (`oyatie.*`) | Reserved Identifier Namespace + IDN Homograph Defence | AWS `arn:aws:iam::aws:`; UTS #39; UTR #36 | Typosquatting Tenant Impersonation — partner registers `oyatie-fake` to imply affiliation |
| ADR-0244 D-2 | Dotted hierarchical sub-scope | Hierarchical Principal Path | AWS IAM principal paths; GCP resource hierarchy; Azure RBAC scope; Kubernetes namespace hierarchy | Flat Namespace Drift — inheritance requires explicit cross-namespace queries at scale |
| ADR-0244 D-2.d | Max depth 5 | Bounded-Depth Hierarchy | AWS IAM path limit; Azure subscription nesting limit; GCP folder depth limit (10 in practice; 5 recommended) | Unbounded Tree Depth — policy evaluation exponential in depth |
| ADR-0244 D-3 | Tenant table schema (single source of truth) | Single Source of Truth Tenant Registry | AWS Organizations master account table; GCP Resource Manager hierarchy table; Stripe Accounts table | Per-µservice Tenant View Drift — each µservice rolls its own tenant concept |
| ADR-0244 D-3.c | Capability flags column | Capability-Based Authorization | Stripe account capabilities; AWS IAM permission boundaries; Linux capabilities(7) | Role-Based-Only — coarse role assignment misses per-capability gating |
| ADR-0244 D-3.dr | DR pair strategy enum | Tier-Aware DR Strategy | AWS Resilience Hub tiers; Azure Site Recovery patterns | One-Size-Fits-All DR — premium tier RTO applied to every tenant |
| ADR-0244 D-4 | Cedar entity-types for tenants + sub-scopes | Typed Entity Policy Schema | AWS Verified Permissions Cedar entity schema; OPA structured-data policies | Untyped String Match Policy — fragile per-string conditions |
| ADR-0244 D-5 | Manifest schema; drop audience field | Caller-Side Attribute Resolution | AWS principal-attribute policy conditions; Azure AAD claims-based; Stripe webhook tenant_id in payload | Callee-Side Audience Declaration — category error retired |
| ADR-0244 D-6 | Cross-tenant grants (time-bounded) | Time-Bounded Cross-Tenant Grant | AWS STS AssumeRole cross-account; Azure AAD B2B Collaboration; Stripe platform-on-behalf-of | Permanent Cross-Tenant Trust — perpetual elevation; bypass-path acquisition |
| ADR-0244 D-6.3 | Partner on-behalf-of pattern | Platform-on-Behalf-Of Pattern | Stripe Connect; Salesforce Partner Portal; AWS Marketplace partner accounts | Direct Customer Credential Sharing — partner holds customer secrets |
| ADR-0244 D-7 | Tenant lifecycle state machine + soft-delete window | Multi-State Tenant Lifecycle with Soft-Delete Window | AWS Organizations account close (90-day grace); GCP Project delete (30-day soft-delete); Azure AD tenant delete (30-day recovery) | Hard-Delete-Only Lifecycle — accidental deletes irrecoverable |
| ADR-0244 D-7.h | Hard delete cascade + tombstone | Cascade-Plus-Tombstone Deletion | AWS Organizations CLOSED account preserves audit; GCP Project SOFT_DELETED preserves logs | Total Erasure Including Audit — regulatory violation; tamper detection broken |
| ADR-0244 D-8 | Sandbox tenants (per-engineer) | Per-Engineer Sandbox Tenant | AWS Cloud9 + AWS Sandboxes; Stripe Test Mode; Heroku development apps | Shared Development Tenant — engineers step on each other's data |
| ADR-0244 D-9 | Preview tenants (per-PR ephemeral) | Per-PR Ephemeral Tenant | Vercel preview deployments; Heroku Review Apps; Render preview environments | Manual Pre-Production Promotion — slow review cycle |
| ADR-0244 D-10 | Cross-cell migration (signed ledger + drain + cutover) | Signed Migration Ledger + Drain + Cutover | AWS Database Migration Service patterns; Google Spanner re-shard; Cassandra token migration | Big-Bang Migration — irreversible if cutover fails |
| ADR-0244 D-11 | Audience-type enum (closed) | Closed-Enum Tenant Classification | Stripe account type enum; Salesforce customer-vs-partner-vs-internal; Azure AAD tenant type | Free-Form Audience Tags — drift across tenants |
| ADR-0244 D-12 | Reserved namespace enforcement via Cedar fragment | Defence-in-Depth via Cedar Fragment | AWS Service Control Policy; GCP Org Policy constraints; Kubernetes admission controller | Application-Layer-Only Check — bypass via direct DB write |

### 3.4 ADR-0245 — substrate-vs-product-layering (11 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0245 D-1 | Two-rule doctrine — substrate vs product | Foundational-vs-Application Service Tier | AWS Well-Architected v2024-Q4 Pillar 4; Apple Platform Architecture 2024; Google Cloud Deprecation Policy 2024; Salesforce Trust Documentation 2024; Microsoft Cloud Adoption Framework 2024 | Mixed-Tier Service — both substrate and product, SLO/versioning/observability drift |
| ADR-0245 D-2 | Manifest `tier` + `tier_subtype` fields | Manifest-Declared Service Tier | AWS Service Health Dashboard tier classification; CNCF Landscape per-project tier metadata; GCP service tier API | Inferred Tier — tier inferred per-µservice rather than declared |
| ADR-0245 D-3 | Full classification table for the portfolio | Per-Service Tier Registration | AWS Service Health Dashboard registry; GCP service catalog; Apple Framework Index | Lazy Tier Classification — services emerge without tier classification |
| ADR-0245 D-4 | Cross-tier dependency rules | Layered Service Tier DAG | AWS Builders' Library service-layering pattern; GCP service dependency graph; Apple Framework dependency rules | Inverted Dependency — substrate depends on product (architectural inversion) |
| ADR-0245 D-4.B | Substrate DAG ordering | Foundational Dependency DAG | AWS Builders' Library "Static stability"; GCP Borg/Omega layering (Verma et al. 2016); Apple Frameworks Reference dependency layers | Cyclic Substrate Dependency — chicken-egg bootstrap failure |
| ADR-0245 D-5 | Service-cell deep-dive (peer-cell pattern) | Peer-Cell Service Pattern | AWS Marketplace + AWS Activate peer-cell pattern; Salesforce AppExchange peer-cell; Stripe peer-cell | Forced Two-Tier — service cells classified as substrate or product creating ambiguity |
| ADR-0245 D-6 | Reserved µservice rules (built-but-pre-cert) | Build-Ahead-of-Certification | AWS pre-launch service pattern; FedRAMP reserved namespace; Apple beta-framework pattern | Live-Before-Certified — uncertified service deployed live, missing regulatory gate |
| ADR-0245 D-7 | CI lane coherence enforcement | Coverage-Required Tier Classification | AWS Config conformance packs; Google SRE Workbook ch. 4 SLO coverage; Apple Xcode static analysis | Honour-System Tier — tier classification not enforced at CI time |
| ADR-0245 D-8 | Substrate SLO floor 99.99% minimum | Per-Tier SLO Floor | Google SRE Workbook ch. 2 SLO composition; AWS Well-Architected v2024-Q4 Pillar 4; Microsoft Azure Well-Architected | Uniform SLO — substrate underprovisioning + product overprovisioning |
| ADR-0245 D-8.c | Cross-tier SLO composition (Markov) | Markov-Chain Availability Composition | Pinheiro et al. 2007; Google SRE Workbook ch. 2; AWS Well-Architected Reliability Pillar | Unverified Composition — product SLO higher than substrates' composed SLO |
| ADR-0245 D-9 | Per-tier versioning + deprecation policy | Tier-Aware Deprecation | Google Cloud Deprecation Policy 2024 (12+ months foundational); AWS deprecation policy; Apple framework SemVer | Uniform Deprecation Window — single window across tiers |

### 3.5 ADR-0246 — policy-engine substrate promotion (11 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0246 D-1 | Promote bounded contexts to peer substrate µservice | Centralized Policy Service | AWS Verified Permissions (re:Invent 2023 BOA303); Google Org Policy; Netflix authz service (Netflix Tech Blog 2024) | Embedded Policy in Application Service — policy evolution coupled to host service deploy cycle |
| ADR-0246 D-2 | 8 BCs (fragment-registry, evaluator, signing-chain, hot-reload, coverage-audit, pack-overlay, tenant-overlay, bootstrap-genesis) | Single-Concern Bounded Contexts | DDD (Evans 2003); ADR-0132 no-grouping forward policy | Bundle Bounded Context — multi-concern BCs prone to coupling drift |
| ADR-0246 D-3 | 47-crate redistribution per BNF v4.1 + ADR-0105 | Hexagonal Architecture with Port-in-Kernel | Cockburn 2005 Hexagonal; ADR-0105 13-value canonical enum | Anemic Layered Architecture — ports defined outside kernel, leaking I/O concerns |
| ADR-0246 D-4 | gRPC + OpenAPI 3.2.0 dual surface; 10 operations | gRPC-Primary with REST Compat | Google API Design Guide; Stripe API design (REST primary with gRPC for internal); Cloudflare Workers gRPC | Single-Protocol Lock-in — REST-only locks out efficient inter-µservice calls; gRPC-only locks out browser callers |
| ADR-0246 D-5 | Per-cell deployment: 3+ replicas + HPA + PDB + cross-region paired DR cell | Cell-Sharded Stateless Tier with HA | AWS cell-based architecture (Bryan Liston re:Invent 2018); ADR-0009 cell architecture; ADR-0048 cell sharding | Global Singleton Service — single-region or single-replica policy service is a portfolio-wide blast radius |
| ADR-0246 D-6 | Hot path p99 < 1ms via in-cell evaluator + Valkey hot cache + circuit breaker fallback | Static Stability + Edge-Cached Evaluation | AWS Builder's Library "Static Stability" (Weiss + Furr); AWS Verified Permissions production cache; Cloudflare Workers KV | Synchronous Round-Trip to Global Policy Store — cross-region policy fetch on hot path |
| ADR-0246 D-7 | Postgres + Citus shard on (scope, fragment_id); Cedar AST cache covers hot path | Distributed Relational with Application-Aware Sharding | Citus design (Microsoft acquired 2019); AWS Aurora Limitless; Google Spanner external consistency | Single-Instance Relational Bottleneck — single Postgres for global policy doesn't scale write-side |
| ADR-0246 D-8 | Bootstrap chain of trust: org root in HSM → genesis fragment → intermediate keys → publisher fragments | PKI Root + Intermediate Certificate Chain | RFC 5280 X.509; Sigstore Rekor; AWS KMS key hierarchy; Let's Encrypt CA hierarchy | Implicit Bootstrap Trust — undocumented signing key emergence |
| ADR-0246 D-9 | Ontology amendment: drop BC, rewrite "universal mediator" framing, rename agent-gateway BC | Substrate Cohesion via PRD Amendment | DDD context-mapping (Evans 2003); ADR pattern (Nygard 2011) | Stale PRD — PRDs drift behind architectural reality |
| ADR-0246 D-10 | SLO targets: T1, 5min RTO, 0 RPO, 99.99% availability | Tiered DR + Per-Microservice SLO Ownership | ADR-0241 DR + BC portfolio policy; Google SRE Workbook ch. 2 | Implicit SLO — µservices ship without explicit SLO declaration |
| ADR-0246 D-11 | CI lane rename: governance-* → governance-* | Coverage-Enforced Substrate Doctrine | Google SRE Workbook ch. 4 (SLO coverage); AWS Config conformance packs | Untested Substrate Surface — substrate gates discovered missing in production |

### 3.6 ADR-0247 — self-hosting / self-modification doctrine (12 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0247 D-1 | Foundry-as-µservice dissolves; BCs redistribute to primitives | Substrate Primitive De-duplication | AWS Bedrock + Step Functions + IAM as separate substrates; GCP Vertex AI + Workflows + IAM Conditions; Azure AI Foundry + Logic Apps + Azure Policy | Primitive Duplication Across Sibling µservices — same primitive drifts over time |
| ADR-0247 D-2 | Workflow library replaces Foundry-as-product | Internal-CI as Tenant-of-Platform | AWS internal CI as AWS IAM principal; Stripe internal CI as Stripe tenant; Google internal CI as Borg tenant | Internal-CI as Separate µservice — bypass paths + drift loops |
| ADR-0247 D-3 | Self-modification mechanics under Cedar gates | Policy-Gated Reflective Tower | AWS Verified Permissions self-modification; Anthropic Console self-modification under Cedar-equivalent gates | Unrestricted Reflection — system can authorise any modification of itself |
| ADR-0247 D-4 | Bootstrap Tier 0 minimum | Audited Bootstrap Replay | rustc stage0; Kubernetes kubeadm certificate chain; Certificate Transparency log bootstrap | Untraceable Genesis — original deployment lacks audit trail |
| ADR-0247 D-5 | 5-stage bootstrap sequence | Multi-Stage Self-Host Bootstrap | rustc stage0/1/2 chain; LFS Chapter 5/6 cross-compile pattern; kubeadm Phase 1/2 design | Big-Bang Bootstrap — single step from zero to steady-state with no audit |
| ADR-0247 D-6 | Dev/staging/prod self-modification environments | Three-Tier CD with Auto-Rollback | AWS internal dev/gamma/prod fleets; Google canary → fleet rollout; Spinnaker bake-to-prod pipeline | Single-Environment Self-Modification — production drift without rehearsal |
| ADR-0247 D-7 | Workflow versioning + atomic swap | Immutable Workflow Version Pinning | Temporal workflow versioning; AWS Step Functions versioning; GCP Workflows versioning | Mutable Workflow Drift — running instances change underfoot |
| ADR-0247 D-8 | Cedar fragment gating self-modification | Policy-Engine-Gated Self-Modification | AWS Verified Permissions + KMS chain; Sigstore signed-fragment provenance | Trust-On-First-Use Self-Modification — first publisher acquires unbounded modification rights |
| ADR-0247 D-9 | Artifact migration plan (lossless lineage) | Lossless Substrate Distribution | rustc stage0/1/2 maintains historical artifact lineage; Nix flakes preserve input provenance | Lossy Migration — primitives drop during reorganisation |
| ADR-0247 D-10 | retired external agent harness name retirement | Inherited-Term Decommission | Glossary discipline; canonical-glossary enforcement | Vestigial Terminology Sprawl — inherited names persist with no canonical meaning |
| ADR-0247 D-11 | CI lanes for self-modification | Coverage-Required Self-Modification | Google SRE Workbook ch. 4 (SLO coverage); AWS Config conformance packs | Untested Self-Modification Surface — paths discovered missing in production |
| ADR-0247 D-12 | Failure modes + bootstrap-replay runbook | Documented Recovery Procedure | AWS Builder's Library "Static Stability"; NIST SP 800-34 contingency planning; Google SRE incident-response runbooks | Tribal Recovery Knowledge — only one engineer knows how to recover |

### 3.7 ADR-0248 — Amazon-shape cellular architecture (17 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0248 D-1 | Tier 0 external dependencies inventory | External Dependency Inventory | AWS Well-Architected Reliability pillar; Google CRE Book ch. 8 | Undocumented External Coupling — surprise outage when external dependency drops |
| ADR-0248 D-2 | Tier 1 bootstrap cell with retirement plan | Bootstrap-and-Retire | rustc stage0 bootstrap; Kubernetes kubeadm; Certificate Transparency log bootstrap | Eternal Bootstrap — bootstrap cell never retires; becomes architectural sediment |
| ADR-0248 D-3 | Tier 2 control plane cells | Control Plane / Data Plane Separation | AWS Route 53 control plane; GCP Spanner zone-master separation; Stripe Cells 2024 control plane | Co-Located Control + Data — control plane saturation propagates to data plane |
| ADR-0248 D-4 | Tier 3 data plane cells (per-tenant-group) | Per-Tenant-Group Cell | AWS S3, Lambda cellular; Stripe Cells 2024 per-account cell; Salesforce Pods | One Cell Per Service — per-service cells produce per-cell-fixed-cost × N anti-pattern |
| ADR-0248 D-5 | Service cells (peer-tier dedicated function) | Peer-Tier Dedicated-Function Cell | AWS Marketplace, AWS IAM Access Analyzer; Stripe Connect; Salesforce AppExchange | Service-Cell-Sprawl — every µservice gets a service cell |
| ADR-0248 D-6 | Per-cell vs cross-cell bright line | Hot-Path-Intra-Cell | AWS S3 partition boundary; Google Spanner replica locality; Stripe per-account locality | Cross-Cell Hot Path — synchronous cross-cell call introduces fault correlation |
| ADR-0248 D-7 | Shuffle sharding `S=8` | Shuffle Sharding | MacCárthaigh 2014 AWS Architecture Blog; Route 53 production; AWS Lambda concurrency model | Single-Cell-Per-Tenant — cell failure = tenant fully offline |
| ADR-0248 D-8 | Static stability 24h tolerance | Static Stability | Weiss/Furr 2020 AWS Builder's Library; AWS 2024 ARC404 | Fail-Fast-On-Control-Plane-Outage — data plane shuts down when control plane goes |
| ADR-0248 D-9 | Constant work (Route-53-style health propagation) | Constant Work | Brooker 2020 AWS Builder's Library; Route 53 health propagation | Push-Per-Change Delta — control plane scales with change rate × fleet size |
| ADR-0248 D-10 | Cell sizing + auto-spawn at 70% utilisation | Capacity-Aware Auto-Spawn | AWS Lambda concurrency scaling; Kubernetes HPA + Karpenter; Stripe Cells 2024 sizing | Manual Cell Provisioning — operations bottleneck at scale |
| ADR-0248 D-11 | Cross-region routing via GeoDNS | GeoDNS + Edge Failover | Cloudflare GeoDNS + edge POPs; AWS Route 53 latency-based routing; Akamai EdgeDNS | Centralised DNS Hot-Spot — single DNS point of failure |
| ADR-0248 D-12 | Planned migration workflow (audit-trail-backed) | Audit-Trail-Backed Tenant Migration | AWS Outposts migration; Stripe Cells 2024 account migration | Live Tenant Migration Without Audit — migration drops data; no rollback |
| ADR-0248 D-13 | K8s-everything except edge | Workload-In-Pod Default | Google Kubernetes Engine; AWS EKS; Microsoft Azure AKS | Snowflake Workload — bespoke deployment per-µservice |
| ADR-0248 D-14 | Cloud Hypervisor + Kata Containers for untrusted | VM-Per-Workload Isolation | AWS Firecracker; Kata Containers at Bytedance + Tencent + Microsoft; Confidential Computing Consortium | Container-Only Isolation For Untrusted — container escape risk |
| ADR-0248 D-14.alt | NOT gVisor (rejected) | KVM-Backed Isolation | AWS Firecracker; Kata + Cloud Hypervisor; Linux KVM hardware-backed | User-Space Syscall Interception — gVisor Sentry bugs become escape vectors |
| ADR-0248 D-15 | Cloudflare → Pingora; HTTP/3 default | Distributed Edge POP | Cloudflare edge ~300 POPs; AWS CloudFront; Fastly POPs | Centralised Ingress — single ingress point; geographic latency |
| ADR-0248 D-16 | Tier 4 reserved for sovereign/classified | Build-Ahead-of-Certification | AWS GovCloud (built before FedRAMP-High cert); AWS HealthLake (built before HIPAA cert) | Graft-On-After-Cert — topology refactor under regulator deadline |

### 3.8 ADR-0249 — multi-category marketplace doctrine (24 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0249 D-1 | 8 marketplace substrates (catalog, inventory, orders, fulfillment, reviews, discovery, pricing, trust-safety) | Substrate-Shared, Surface-Specialised | Amazon ASIN evolution (Bezos 1997 + Bryar/Carr 2021); Stripe Tenant/RBAC Packaging docs; Apple One subscription bundle | Per-Category Stack Duplication — N stacks for N commerce shapes |
| ADR-0249 D-1.1 | Catalog universal Listing identifier | Universal Product Identifier | Amazon ASIN; eBay Item ID; Walmart Marketplace Item ID; Etsy Listing ID | Per-Category Catalog Fragmentation — categories share no identity primitive |
| ADR-0249 D-1.2 | Inventory per-warehouse stock | Per-Warehouse Stock State | Amazon Fulfilled-By-Amazon docs; Shopify Inventory API; ShipBob docs | Single Global Inventory — no warehouse dimension makes 3PL impossible |
| ADR-0249 D-1.3 | Orders as durable saga | Saga Pattern for Distributed Order Workflow | Temporal.io docs (Workflow Engine inheritance); AWS Step Functions Saga blueprint; Stripe Order API | Synchronous Order Pipeline — long pipeline blocks under failure |
| ADR-0249 D-1.4 | Fulfillment 3PL adapters | Pluggable Carrier + 3PL Adapter Layer | ShipBob, ShipStation, Easypost adapter patterns; Shopify Shipping App docs | Carrier-Specific Direct Integration — vendor lock per carrier |
| ADR-0249 D-1.5 | Reviews + Q&A + moderation | Multi-Surface Reputation System | Amazon Reviews + Q&A; eBay Feedback; Yelp Reviews | Standalone Review Database — no cross-surface reputation signal |
| ADR-0249 D-1.6 | Discovery (tantivy + Quickwit + ClickHouse) | Three-Tier Search + Ranking Stack | Algolia engineering blog; Elasticsearch + ClickHouse hybrid patterns; Amazon Search architecture talks | Single-Engine Search — can't carry analytics alone |
| ADR-0249 D-1.7 | Pricing rules + promo + tax substrate | Pricing-Promotion-Tax Substrate | Shopify Pricing + Discounts APIs; Amazon Price Rules; Stripe Tax | Per-Category Pricing Logic — promotion rules duplicated per category |
| ADR-0249 D-1.8 | Trust-safety + cold-start | Multi-Signal Trust Score with Cold-Start | Stripe Radar; Sift Engineering blog; Airbnb Trust + Safety; Meta Marketplace fraud talks | Single Trust Score — categories share no nuance |
| ADR-0249 D-2 | 4 consumer-surface BCs sharing substrate | Per-Category Surface BC, Shared Substrate | Apple Music + Apple TV+ + Apple Arcade on shared App Store ID; Stripe Connect's diverse partners | One Surface for All Categories — UX mismatch |
| ADR-0249 D-3 | Plugin-app-store refactor onto shared substrate | Existing-Product Refactor onto Shared Substrate | Salesforce AppExchange evolution onto Salesforce Lightning Platform | Parallel Stack for Plugins — plugin-app-store as separate commerce engine |
| ADR-0249 D-4 | `marketplace_roles[]` (multi-role tenant) | Multi-Role Tenant | AWS IAM multi-policy attachment; GCP IAM role binding | Single-Role Tenant — tenant can only be buyer OR seller |
| ADR-0249 D-5 | `seller_categories[]` (per-category verification) | Per-Category Seller Verification | Etsy Shop categories; Amazon Seller Central category approval; Shopify Partner verification | All-or-Nothing Seller — open every category at once without verification |
| ADR-0249 D-6 | `fulfillment_capabilities[]` | Declared Fulfillment Capabilities | Shopify Locations API; Amazon Seller Fulfilled vs FBA distinction | Implicit Fulfillment — no declared shape leads to fulfillment failures |
| ADR-0249 D-7 | Trust + reputation + cold-start with graduated limits | Cold-Start with Graduated Limits | Stripe Radar; Airbnb New Host program; eBay 100-feedback restriction history | No Cold-Start Friction — fraud floods new accounts |
| ADR-0249 D-8 | Marketplace cell pinning + cross-cell projection | Cell-Local Surface + Cross-Cell Projection | Amazon's per-region retail with cross-region catalog; AWS Marketplace cross-region catalog | Single Global Cell — single point of failure |
| ADR-0249 D-9 | Per-category certification-readiness wave | Phased Activation by Certification | Stripe Atlas + + Treasury phased launches; Apple Pay country-by-country activation | Big-Bang Multi-Category Launch — fails when one cert blocks others |
| ADR-0249 D-10 | Cross-tenant + cross-cell saga | Compensating Saga across Cells | AWS Step Functions Saga; Temporal cross-cluster workflow | Two-Phase Commit Across Cells — synchronous 2PC blocking |
| ADR-0249 D-11 | Marketplace facilitator tax (per-jurisdiction activation) | Marketplace Facilitator with Per-Jurisdiction Activation | Amazon's MTL implementation; Etsy's state-by-state activation; Shopify Tax | Seller Self-Reports Tax — collection failure + compliance risk |
| ADR-0249 D-12 | Per-category Cedar gating | Per-Category Policy Overlay | Apple App Store category-specific review (Health/Medical); Shopify per-category requirements | Single Policy for All Categories — under-restricts healthcare etc. |
| ADR-0249 D-13 | Discovery substrate stack (federated search + OLAP) | Federated Search + OLAP Ranking | Algolia federation; Elasticsearch + ClickHouse hybrid in Yelp/Airbnb | Manual Per-Cell Index — cells diverge in ranking |
| ADR-0249 D-14 | Reviews moderation (per-jurisdiction overlay) | Per-Jurisdiction Moderation Overlay | DSA Article 16 implementation by Meta/Google/Amazon EU; KR-방심위 compliance pattern | Single Global Moderation Policy — DSA non-compliance |
| ADR-0249 D-15 | Returns + disputes + escrow | Workflow Saga with Compensating Action + Escrow | Amazon A-to-Z Guarantee + Escrow; Upwork milestone escrow; eBay Money Back Guarantee | Manual Refund + No Escrow — buyer protection failure |

### 3.9 ADR-0250 — build-ahead-of-certification doctrine (12 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0250 D-1 | Three-state lifecycle (ARCHITECTED → BUILT → LAUNCHED) | Architected-Built-Launched Tri-State | AWS service-availability progression (preview → public → cert-authorized); Apple Pay country progression; Microsoft Azure Vertical Industry Solutions launch shape | Build-on-cert-grant — single-state launched/not-launched with no built-but-unlaunched intermediate |
| ADR-0250 D-2 | Build quality bar (operationally-ready-before-launch) | Operationally-Ready-Before-Launch | AWS Builder's Library "Ten things we wish we'd known sooner"; Apple Pay "we launched only when we were ready" Vogels-equivalent quote; Stripe Engineering blog 2014-2020 | Demo-quality launch — launching with insufficient ops-readiness |
| ADR-0250 D-3 | Per-market launch gates | Per-Market Launch Gate Matrix | Apple Pay per-country launch playbook; Stripe per-country onboarding sequence; AWS per-region service launch sequencing | Global simultaneous launch — incompatible with per-market certification |
| ADR-0250 D-4 | Capability + certification matrix as canonical catalog | Certification Catalog as Canonical Source | AWS Service Authorisation Reference; Microsoft Trust Center compliance offerings catalog; Salesforce Trust + Compliance | Ad-hoc per-market certification lookup — discovery via emails + tribal knowledge |
| ADR-0250 D-5 | Three-state lifecycle in tenant model | Eligibility-as-Derived-State | AWS Organization OU eligibility; GCP IAM eligibility from inherited permissions; Apple App Store entitlement-as-derived | Imperative eligibility checks — per-µservice if/else against tenant flags |
| ADR-0250 D-6 | Cedar gate composition for launch eligibility | Layered Policy Composition | AWS Verified Permissions cell-policy composition; AWS Cedar policy union semantics; ADR-0243 D-4 layered overlay | Single monolithic policy — non-composable per-tenant or per-market policy |
| ADR-0250 D-7 | Launch runbook template | Pre-Launch Runbook Discipline | AWS Operational Readiness Review (ORR) checklist; Google SRE Workbook ch. 18 Production Readiness Review (PRR); Microsoft Operations Manual | Tribal launch knowledge — launch executes from informal team memory |
| ADR-0250 D-8 | Multi-year roadmap timeline | Multi-Year Capability Roadmap | Apple Pay 2014-2024 country launch progression; Stripe 2011-2025 product + country progression; AWS 2006-2024 service + region progression | Quarterly product roadmap — incompatible with 18-36-month certification timelines |
| ADR-0250 D-9 | Anti-bypass guarantees (no admin override) | No-Bypass Defense in Depth | AWS SCP + IAM + permission boundary triple-check; NIST SP 800-207 Zero Trust no-bypass; ADR-0242 D-3 no-internal-bypass | Admin override loophole — bypass paths acquired during incidents, retained afterward |
| ADR-0250 D-10 | Pre-launch testing on sandbox/preview tenants | Sandbox-Tenant Pilot | Vercel preview deploys; Stripe test mode; Heroku review apps; ADR-0242 D-8 ephemeral tenants | Production-only test — live tenant exposure to built-but-unlaunched capability |
| ADR-0250 D-11 | Evidence retention anchored on audit chain | Audit-Chain-Anchored Compliance Evidence | AWS Audit Manager evidence packs; Sigstore Rekor immutability; Sedona Conference legal-hold supersession | Evidence siloed per certification — non-uniform retention; lost evidence under audit |
| ADR-0250 D-12 | Graceful rollback model on cert lapse | Graceful Capability Sunset | Salesforce End of Life roadmap for legacy products; AWS deprecation policy (12-month notice); Stripe API version sunset cadence | Hard kill on cert lapse — abrupt customer service interruption |

### 3.10 ADR-0251 — compliance pack + cell certification levels (16 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0251 D-1 | Compliance Pack schema | Compliance-as-Packaged-Bundle | AWS Audit Manager framework catalog; Microsoft Purview assessment templates; Google Assured Workloads compliance regimes; Salesforce HealthCloud / GovCloud SKUs; Databricks Compliance Security Profile | Ad-Hoc Per-Regulation Implementation — N+1 regulation cost scales superlinearly |
| ADR-0251 D-2 | Pack lifecycle (author → review → sign → publish → activate → audit → sunset → tombstone) | Signed Policy Bundle Lifecycle with Transparency Log | Sigstore Rekor + cosign; AWS Verified Permissions policy store; AWS IAM Policy versioning + history | Imperative Policy Patching — pack content changes without provenance |
| ADR-0251 D-3 | Tenant pack installation (eligibility + KYB/KYC + jurisdiction + DPIA + agreements + cell pinning + Cedar activation + audit + onboarding) | Tenant-Installs-Compliance-Regime | Google Assured Workloads regime activation; AWS Control Tower compliance guardrail installation; Microsoft Purview tenant assessment activation | Implicit Compliance Inheritance — tenant assumed-compliant without explicit opt-in + verification |
| ADR-0251 D-4 | Cell certification level matrix (general / hipaa / fedramp-mod / fedramp-high / dod-il5 / dod-il6) | Cell-Certification-as-Discrete-Levels | AWS regions partitioned (Commercial vs GovCloud vs ISO vs China vs Top Secret); Google Cloud regional compliance designations; Azure compliance offerings per region; AWS Outposts compliance binding | Single-Tier Substrate — one substrate must satisfy all regulations |
| ADR-0251 D-5 | Tenant → cell pinning rule (mandatory) | Mandatory-Compliance-Pinning | AWS Organizations + Control Tower account-to-OU pinning; Azure Subscription compliance binding; Google Cloud Project assured-workloads binding | Drift via Tenant Movement — tenant migrates to incompatible cell silently |
| ADR-0251 D-6 | Cross-pack traffic Cedar-gated; full deny matrix | Cross-Tenant Policy Gate | AWS Verified Permissions cross-account evaluation; AWS Resource Access Manager (RAM) cross-account share gating; Azure RBAC cross-tenant guest access; Google IAM cross-project deny | Implicit Cross-Tenant Trust — data flows freely across compliance domains |
| ADR-0251 D-7 | BAA/DPA agreement lifecycle saga | Durable-Workflow-Driven Compliance-Agreement Lifecycle | AWS Artifact agreement automation; DocuSign + Adobe Sign integration; Stripe onboarding workflow | Manual-Email-PDF Agreement Lifecycle — agreements lost in inboxes |
| ADR-0251 D-8 | Breach notification per-jurisdiction workflow | Per-Jurisdiction Breach-Notification Workflow | AWS GuardDuty + Detective + Security Hub integrated incident response; Microsoft Sentinel + Compliance Manager breach playbook; Atlassian + PagerDuty + ServiceNow incident response | First-Breach Scramble — workflow built under deadline pressure with errors |
| ADR-0251 D-9 | De-identification engine substrate (tokenization + k-anon + l-div + t-close + DP + HIPAA Safe Harbor + GDPR pseudonymization) | Shared De-Identification Substrate | AWS Glue DataBrew PII transforms; Google Cloud DLP API; Microsoft Presidio; Privitar (acquired by Informatica 2023) | Per-Use-Case De-ID Implementation — HIPAA Safe Harbor implemented incorrectly |
| ADR-0251 D-10 | Encryption substrate (per-data-class + encryption-key BYOK + HYOK + FIPS 140-2/3 + HSM-rooted + PQ-hybrid) | Hierarchical-Key-Management Substrate | AWS KMS + CloudHSM + per-service-key hierarchy; Google Cloud KMS + Cloud HSM + External Key Manager; Azure Key Vault Managed HSM | Per-Service KMS — keys reimplemented per service inconsistently |
| ADR-0251 D-11 | Consent management substrate (per-purpose) | Per-Purpose Consent Substrate | OneTrust Consent Management; TrustArc Consent Manager; Cookiebot (Usercentrics) | Boolean Consent Field — consent collapsed to one column; granularity lost |
| ADR-0251 D-12 | Per-pack DPIA template | Per-Regulation DPIA Template | ICO DPIA template (UK); CNIL PIA tool (France); HHS HIPAA Risk Analysis tool; EU AI Act FRIA template | Free-Form DPIA Document — DPIA inconsistent across deployments |
| ADR-0251 D-13 | Audit chain per-pack stream with per-jurisdiction retention | Per-Stream Audit-Chain with Per-Pack Retention | AWS CloudTrail Lake per-event-data-store retention; Google Cloud Audit Logs retention buckets; Microsoft Sentinel retention tiers | Single Audit Stream — retention overshooting cost; under-shooting compliance |
| ADR-0251 D-14 | Pack composition semantics (deny wins, retention MAX, cross-tenant MOST RESTRICTIVE) | Compositional Policy Semantics | AWS SCP + IAM policy intersection (deny wins); GCP Org Policy + Project Policy union; Cedar fragment composition | Per-Pack Re-Implementation of Composition — composition rules drift per pack |
| ADR-0251 D-15 | Certification level inheritance (FedRAMP High ⊇ Moderate ⊇ Low etc.) | Hierarchical Certification Inheritance | FedRAMP High ⊇ FedRAMP Moderate ⊇ FedRAMP Low; CMMC Level 3 ⊇ Level 2 ⊇ Level 1; ISO 27001 implies ISO 27002 controls | Flat Certification Catalog — prerequisites not enforced |
| ADR-0251 D-16 | Auto-emit auditor evidence package per-pack per-cadence | Auto-Emit Auditor Evidence Package | AWS Audit Manager evidence collection + assessment report; Microsoft Purview Compliance Manager scorecard export; Google Cloud Assured Workloads evidence | Manual Audit-Evidence Compilation — quarterly engineering scramble |

### 3.11 ADR-0252 — time coordination + distributed consistency (16 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0252 D-1 | HLC default clock primitive | Hybrid Logical Clock | Demirbas + Kulkarni OPODIS 2014; CockroachDB design doc 2015; MongoDB Atlas Causal Consistency docs; YugabyteDB consistency docs; TiDB Percolator+HLC docs | Wall-Clock Ordering — every leap-second + NTP-skew outage in §Context |
| ADR-0252 D-2 | TrueTime tier for Tier-4 cells | Atomic-Clock-Backed External Consistency | Spanner OSDI 2012; AWS Aurora DSQL re:Invent 2024 | TrueTime Everywhere — cost prohibitive at fleet scale |
| ADR-0252 D-3 | Causal default; strict total order opt-in | Tiered Consistency Model | MongoDB Atlas tiers; Azure Cosmos DB 5-level model; CockroachDB causality propagation | One Size Fits All Consistency — over-coordination cost or under-consistency bugs |
| ADR-0252 D-4 | Caller-supplied idempotency keys | Stripe Idempotency Key | Brandur Leach 2014; Stripe API docs; AWS SDK retryable APIs 2018+; IETF draft `idempotency-key-header-09` | Retry-Without-Dedup — every AWS Lambda duplication / Atlassian cleanup class incident |
| ADR-0252 D-5 | No distributed locks; saga + idempotency replacement | Saga + Compensation, Not Lock | Google SRE Workbook Ch.24; Kleppmann fencing token essay; AWS Distributed Sagas; Temporal docs; ADR-0222 | Distributed Lock — GitHub Redis-lock-zombie 2023; "lock-held-too-long" class incidents |
| ADR-0252 D-6 | Per-cell cron with jitter | Per-Cell Periodic Scheduling | SRE Workbook Ch.24; Klein "Cron at scale" 2018; AWS EventBridge per-region scheduler model | Global Cron Service — single-point-of-failure scheduler cascading across cells |
| ADR-0252 D-7 | Google Smear leap second handling | Linear Time Smear | Google blog 2008+2011; AWS announcement 2015+2016; Meta blog 2022; chronyd `leapsectz slew` | Step-At-Leap-Boundary — Linux kernel livelock 2012-06-30 |
| ADR-0252 D-8 | Idempotency key format spec | Opaque Self-Describing Key | Stripe key shape; Square Idempotency-Key; Twilio Idempotency-Key | Server-Generated Idempotency Key — defeats the retry-safety guarantee |
| ADR-0252 D-9 | Cross-cell idempotency replication | Per-Cell Idempotency Store | AWS Step Functions per-region state; Temporal per-cluster idempotency | Global Idempotency Store — single-point-of-failure for retry safety |
| ADR-0252 D-10 | Audit-chain HLC ordering + cross-cell gossip | HLC-Ordered Audit Chain | CockroachDB CDC ordering; Cassandra cluster timestamp resolution post-HLC adoption | Wall-Clock Audit Ordering — forensic ambiguity during cross-region investigations |
| ADR-0252 D-11 | Time bound in Cedar context | Policy-Enforced Deadline | NIST SP 800-207 ZTA per-call evaluation; AWS Builders Library "request budgets" | Implicit Infinite Timeout — unbounded latency from missing deadline |
| ADR-0252 D-12 | Replay safety via idempotency + saga + HLC | Deterministic Workflow Replay | Temporal replay model; AWS Step Functions checkpoint replay; Cadence replay | Non-Replayable Workflow — manual recovery from arbitrary failure |
| ADR-0252 D-13 | Clock skew tolerance bounds + alerts | Uncertainty-Bounded Time | Spanner TT uncertainty bound; CockroachDB max_offset config | Silent Clock Drift — incorrect timestamps without warning |
| ADR-0252 D-14 | Time-based feature flags in Cedar | Policy-as-Feature-Flag | AWS Verified Permissions feature gates; ADR-0243 unification | Separate Feature-Flag SDK — LaunchDarkly-class parallel policy |
| ADR-0252 D-15 | Per-µservice HLC integration contract | Uniform Clock Abstraction | CockroachDB hlc.Clock interface; MongoDB Cluster Time integration | Per-µservice Clock Reimplementation — drift between services |
| ADR-0252 D-16 | Postgres REPEATABLE READ default; SERIALIZABLE Cedar gate | Tiered Isolation with Policy Opt-In | Postgres SSI (Cahill 2008); CockroachDB SERIALIZABLE; Azure Cosmos consistency tiers | SERIALIZABLE-Everywhere — performance tax for ops that don't need it |

### 3.12 ADR-0253 — network topology + edge + service mesh (24 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0253 D-1.a | Anycast + GeoDNS apex | Anycast Apex DNS | Cloudflare DNS Anycast 300+ POPs; AWS Route 53 Anycast; Google Cloud DNS Anycast 100+ POPs | Single-region DNS — apex single point of failure |
| ADR-0253 D-1.b | DNSSEC zone integrity | Zone integrity attestation | Cloudflare DNSSEC + AWS Route 53 DNSSEC + RFC 4034/4035 | Unauthenticated DNS — DNS spoofing risk |
| ADR-0253 D-1.c | DoH/DoT (DNS over HTTPS/TLS) | Client DNS privacy | Cloudflare 1.1.1.1 DoH + Google 8.8.8.8 DoH + RFC 8484 + RFC 7858 | Plaintext DNS surveillance — ISP DNS surveillance |
| ADR-0253 D-2.a | Cloudflare Workers + WAF + Bot Management | Planetary Edge POP | Cloudflare Workers 300+ POPs + Akamai 4000+ POPs + AWS CloudFront 350+ POPs | Cloud-provider-LB-only — POP density inadequate |
| ADR-0253 D-2.b | Pingora migration Year 3+ | Rust-based Edge Proxy at Scale | Cloudflare Pingora open-source 2024 (Rust; powers Cloudflare's own edge) | Forever vendor edge — vendor lock-in at hyperscaler scale |
| ADR-0253 D-3.a | TLS 1.3 only | Modern Crypto at Edge | Mozilla SSL Configuration Generator "modern"; Stripe API TLS 1.3 only; Cloudflare zones TLS 1.3 by default | TLS 1.2 downgrade attack surface — POODLE, BEAST, CRIME, HEARTBLEED legacy |
| ADR-0253 D-3.b | Per-cell ingress Envoy | L7 Ingress Termination | Istio + Envoy at GKE + EKS + AKS | Cloud-LB-as-L7 — limited mutability surface |
| ADR-0253 D-3.c | Certificate management via signed config | Certificate-as-Code | ADR-0223 oya git signed config + cosign attestations + Let's Encrypt ACME automation | Ad-hoc cert deployment — outage from cert rotation drift |
| ADR-0253 D-4 | Post-quantum hybrid ML-KEM-768 | Post-Quantum Hybrid KEX | Cloudflare X25519MLKEM768 2024 + AWS s2n-tls kyber-tls13 + NIST FIPS 203 | Harvest-now-decrypt-later — recorded traffic decrypted at CRQC arrival |
| ADR-0253 D-5 | HTTP/3 universal | Modern Transport at Edge | Cloudflare HTTP/3 since 2020 + Google HTTP/3 universal + AWS CloudFront HTTP/3 since 2022 | TCP-only — head-of-line blocking + slow session resumption |
| ADR-0253 D-6 | Cilium ambient + Istio Ambient mesh | Layered L3/L4 + L7 mesh | ADR-0148 + Google GKE Dataplane V2 + Solo.io reference architecture | Sidecar tax — Istio-classic 2× CPU + 30% memory overhead |
| ADR-0253 D-7 | SPIFFE/SPIRE workload identity | Workload identity primitive | Google ALTS + SPIFFE/SPIRE CNCF Graduated 2022 + IRSA at AWS EKS | Static service accounts — credentials stolen and used indefinitely |
| ADR-0253 D-8 | Cross-cell async slow path | Cellular Architecture Async | Amazon cells (Werner Vogels 2019 re:Invent + Pat Helland) | Cross-cell sync hot path — cell failure cascades |
| ADR-0253 D-9 | Cross-provider WireGuard tunnels | Per-pair encrypted tunnel | Google cross-cloud Anthos + IBM Satellite | Public-internet plaintext cross-provider — eavesdropping risk |
| ADR-0253 D-10 | Cilium default-deny egress | Zero-trust egress | NIST SP 800-207 zero-trust architecture + AWS VPC security groups + Cilium 1.14+ identity policy | Default-allow egress — exfiltration risk |
| ADR-0253 D-11 | Per-cell L4 + L7 load balancing | Layered load balancing | Envoy + Cilium kube-proxy replacement + GKE service mesh | Single-tier LB — limited control |
| ADR-0253 D-12 | Year 5+ self-managed BGP (own ASN + RPKI) | Own ASN + RPKI | Cloudflare ASN 13335 + Google ASN 15169 + RIPE NCC ROA | Forever-cloud-BGP — vendor lock-in at planetary scale |
| ADR-0253 D-13 | GeoDNS routes to home_cell with residency fallback | Tenant-aware residency routing | AWS Route 53 GeoDNS + Cloudflare GeoSteering | Residency-blind failover — EU data crosses Atlantic on EU cell failure |
| ADR-0253 D-14 | OpenAPI 3.2 + GraphQL Federation + gRPC + AsyncAPI | Multi-protocol API surface | Stripe API (REST 3.x) + Netflix GraphQL Federation + Google gRPC + Slack AsyncAPI | Single-protocol bottleneck — REST-only forces RPC-over-REST patterns |
| ADR-0253 D-15 | SSE primary + WebSocket bidirectional + WebTransport | Realtime push tier | Slack WebSocket + Discord SSE + ChatGPT SSE for token stream + Zoom WebTransport | Polling — high request volume + latency |
| ADR-0253 D-16 | Dual-signed webhook + saga + idempotency | Webhook reliability triplet | Stripe webhook HMAC + Slack request signing + GitHub Ed25519 + AWS EventBridge retry + Stripe idempotency | Single-sig + fire-and-forget — replay attacks + lost events |
| ADR-0253 D-17 | Pingora migration plan (phased self-hosting) | Phased self-hosting migration | Stripe + Cloudflare + GitHub all reached self-hosted edge by year 5-7 | Forever-hosted-edge — vendor margin compounds at planetary scale |
| ADR-0253 D-18 | PowerDNS migration plan (phased self-hosting DNS) | Phased self-hosting DNS | Cloudflare DNS + Akamai + Hurricane Electric all self-host authoritative DNS | Forever-hosted-DNS — vendor lock-in for foundational primitive |

### 3.13 ADR-0254 — deployment model spectrum (16 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0254 D-1.1 | Shared-cloud multi-tenant SaaS | Shuffle-Sharded Multi-Tenant SaaS | AWS Builders' Library — *Workload isolation using shuffle sharding* (2017-2024); Salesforce Trust Cloud architecture | Noisy-Neighbor Tenant Sprawl — uncapped tenant placement on cells |
| ADR-0254 D-1.2 | Dedicated-cloud single-tenant | Dedicated Cell Pattern | AWS Outposts (dedicated variant); Salesforce Government Cloud; Snowflake Virtual Private Snowflake | Shared Substrate Sovereign-Risk — sovereign tenants on multi-tenant substrate |
| ADR-0254 D-1.3 | Hybrid / BYO-cloud (BYOC) | Bring-Your-Own-Cloud (BYOC) | Snowflake BYOC (2022); Confluent BYOC (2023); Astronomer BYOC (2024); Databricks Customer-Managed VPC (2021) | Mandatory Vendor Cloud Lock-In — tenant forced onto vendor's cloud contract |
| ADR-0254 D-1.4 | On-prem connected | Connected Edge / Hybrid On-Prem | AWS Outposts connected; Azure Arc; Google Anthos; Palantir Apollo connected; Anduril Lattice tactical edge | Disconnected Forever On-Prem — on-prem with no upgrade path |
| ADR-0254 D-1.5 | On-prem air-gapped | Air-Gapped Bundle Delivery | Palantir Apollo air-gapped; GitHub Enterprise Server with TUF; Anduril Lattice classified; defense IL5/6 reference architectures | Online-Only Update Required — air-gap incompatibility |
| ADR-0254 D-2 | Same architecture across models (single build) | Single-Build Multi-Deployment | Palantir Apollo one-Foundry-build; Snowflake's "build Snowflake on Snowflake" blog 2022; Confluent Platform vs Cloud single-codebase | N Parallel Codebases — per-deployment code branches; CVE patching gaps |
| ADR-0254 D-3 | Cell topology per model | Cell as Unit of Deployment | Amazon's cellular architecture (Werner Vogels, re:Invent 2018); ADR-0248 inheritance | Pre-Cellular Deployment Unit — service-level deployment without isolation |
| ADR-0254 D-4 | Deployment control plane | Palantir Apollo Pattern | Palantir Forward 2023+2024 keynotes; Apollo product page | Per-Customer Bespoke Deployment Tooling — no canonical deployment substrate |
| ADR-0254 D-5 | `.oab` signed artifact bundle | TUF + Cosign + SLSA L3 Distribution | The Update Framework spec; Sigstore Cosign 2021+; SLSA L3 specification | Unsigned Distribution — supply-chain attack surface |
| ADR-0254 D-6 | Air-gap one-way diode (Cross-Domain Solution) | Cross-Domain Solution (CDS) Bundle Delivery | NSA RTB guidance; NCDSMO approved CDS products list; DoD SRG IL5/6 reference | Bidirectional Channel Across Air-Gap — covert exfil risk |
| ADR-0254 D-7 | Update + rollback per model (Flagger progressive) | Flagger Progressive Delivery + Per-Model Pull Cadence | ADR-0040 inheritance; Flagger production deployments | Big-Bang Cross-Cell Update — fleet-wide outage on bad release |
| ADR-0254 D-8 | Per-model SLO + support tier | Tiered Support Matrix | AWS Support tiers; Salesforce Premier vs Standard; Palantir Mission Support | Single Support Tier — under-served enterprise + over-priced B2C |
| ADR-0254 D-9 | Per-model pricing (cost-aligned) | Cost-Aligned Pricing | AWS On-Demand vs Reserved vs Savings Plan; Snowflake credit consumption; Confluent per-model SKU | Single Pricing Across Heterogeneous Deployments — cost-misaligned tenant base |
| ADR-0254 D-10 | Compliance per model (uniform pack application) | Compliance-Pack Uniform Application | ADR-0251 inheritance | Per-Model Compliance Carve-Out — uneven compliance posture |
| ADR-0254 D-11 | BYO-cloud setup via IAM-delegated provisioning | IAM-Delegated Customer-Account Provisioning | Snowflake BYOC IAM pattern; Databricks Customer-Managed VPC IAM | Customer-Provides-Root-Credentials — over-privileged access |
| ADR-0254 D-12 | Observability in BYO-cloud (anonymized Cedar-gated telemetry) | Anonymized Cedar-Gated Telemetry | ADR-0253 inheritance; Snowflake telemetry shipping pattern | Raw Tenant Data Egress — privacy violation in BYOC |
| ADR-0254 D-13 | Migration between models (workflow-saga durable) | Workflow-Saga Durable Migration | AWS Step Functions saga pattern; Confluent Cluster Linking migration | Lossy Migration — tenant data corruption across models |
| ADR-0254 D-14 | Air-gap audit-chain reconciliation (Merkle bundled) | Merkle-Sealed Bundled Audit Export | Palantir Apollo air-gap audit pattern; Bitcoin block reconciliation analogue | Lost Air-Gap Audit Continuity — tamper detection gap |
| ADR-0254 D-15 | On-prem hardware requirements (vendor reference architecture) | Vendor Reference Architecture | Dell PowerEdge / HPE ProLiant / Lenovo ThinkSystem reference architectures; AWS Outposts hardware spec | Hardware-Agnostic Spec — under-specified tenant procurement |
| ADR-0254 D-16 | Tenant onboarding per model | Per-Model Onboarding Workflow | Salesforce Onboarding flows; AWS Partner-Led Onboarding; Palantir Mission Specialist onboarding | Generic Onboarding — model-specific risk untracked |

### 3.14 ADR-0255 — intelligence as two-layer AI substrate (18 rows)

| Decision ID | Decision Summary | Hyperscaler Pattern (named) | Source Citation | Anti-Pattern Avoided |
|---|---|---|---|---|
| ADR-0255 D-1 | Two-layer model — AI Substrate + Consumer Brand Surface | Substrate + Brand Surface Layering | Apple Intelligence WWDC 2024 keynote; AWS Bedrock + product UI separation; Azure AI Foundry per-deployment endpoint scoping | Audience-as-µservice-scope — explicitly retired by ADR-0242 + every named reference |
| ADR-0255 D-2 | AI Substrate BCs are audience-neutral | Audience-Neutral Substrate | AWS Bedrock model invocation API; Azure AI Foundry inference endpoint; Apple Foundation Models API | Consumer-Only Substrate — substrate refusing internal-platform-ops calls |
| ADR-0255 D-3 | Consumer Brand Surface BCs are consumer-only | Layered Brand Surface | Apple Intelligence consumer surfaces; Salesforce Einstein consumer UI atop Einstein substrate | Brand Concerns in Substrate — consumer UX leaking into substrate code |
| ADR-0255 D-4 | provider-credential BYOK SecretReference + owner declaration | provider-credential BYOK SecretReference + Owner Declaration | AWS Bedrock customer-managed credentials; provider credential sidecar patterns; Stripe provider credential docs; HashiCorp Vault dynamic secrets | Substrate-Owned Credentials — provider secrets persisted in substrate code or DB |
| ADR-0255 D-5 | Multi-modal transport day-one (text/image/audio/video) | Multi-Modal Day-One Provider Adapter | GPT-4o multi-modal 2024-05; Claude 3.5 Sonnet 2024-06; Gemini 1.5 Pro 2024-02; Azure AI Foundry multi-modal | Text-First, Modality-Later — Apple Intelligence's late image-generation surface scramble 2024 |
| ADR-0255 D-6 | Stateless dispatch + Workflow durability | Stateless Substrate + Durable Composition | AWS Bedrock stateless API + Step Functions durability; Anthropic message batches + workflow caller composition | Stateful Substrate — substrate growing session storage + retry + checkpointing |
| ADR-0255 D-7 | Caller-side RAG (via Ontology) | Caller-Side Retrieval | AWS Bedrock Knowledge Bases (opt-in caller side); Anthropic prompt-caching with caller-built context; Azure AI Foundry retrieval connectors | Substrate-Side Retrieval Coupling — substrate growing vector DB + tenant-data dependencies |
| ADR-0255 D-8 | Embeddings as separate substrate | Embeddings Substrate Promotion | Pinecone-as-substrate; Milvus-as-substrate; AWS Bedrock Knowledge Bases retrieval substrate | Embeddings Embedded in Inference Substrate — coupled lifecycle + scaling |
| ADR-0255 D-9 | Fine-tuning as separate substrate | Fine-Tuning Substrate Promotion | AWS Bedrock model customization; Azure AI Foundry fine-tune; OpenAI fine-tune API | Fine-Tuning Embedded in Dispatch Substrate — coupled training + inference paths |
| ADR-0255 D-10 | External + own-hosted hybrid model serving | Hybrid Model Serving | AWS Bedrock (external + Bedrock-hosted) + custom model import; Azure AI Foundry (Azure OpenAI + Llama + custom) | Single-Tier Model Serving — pure-external (vendor lock-in) or pure-self-hosted (no frontier) |
| ADR-0255 D-11 | Per-cell deployment | Per-Cell Substrate Deployment | AWS region + AZ Bedrock deployment; Apple per-region private cloud compute; Azure AI Foundry per-region endpoint | Global Singleton Substrate — cross-region call hops + sovereignty violation |
| ADR-0255 D-12 | Tool calling split Intelligence/Ontology | Tool-Call Ingress + Dispatcher Separation | Anthropic MCP architecture (server + host separation); AWS Bedrock Agents Action Groups | Tool-Call Logic in LLM Substrate — tenant-data authorization leaking into dispatch |
| ADR-0255 D-13 | Streaming via SSE + WebSocket bi-directional | Streaming via SSE Conventions | Anthropic streaming API; OpenAI streaming API; Stripe SSE conventions | Long-Polling Streaming — connection thrash + audit-row loss |
| ADR-0255 D-14 | Conversation state — stateless default + opt-in session-store | Opt-In Session State | OpenAI Assistants v2 (opt-in stateful); AWS Bedrock Agents conversational state | Always-Stateful Substrate — every caller pays session-storage cost |
| ADR-0255 D-15 | Audience as call tag (not service scope) | Audience-As-Call-Tag | AWS Bedrock Guardrails per-policy attachment; Azure AI Foundry per-deployment scoping; Apple Intelligence per-surface brand | Audience-As-Service-Boundary — explicitly retired by ADR-0220 alternative reconsidered |
| ADR-0255 D-16 | Foundry BC absorption (substrate consolidation) | Substrate Consolidation Under Universal Tenancy | AWS Bedrock absorbing prior per-team provider adapters; Azure AI Foundry consolidating Azure AI Studio + Azure ML + Azure OpenAI | Doubled Provider Adapter Surface — duplicated Anthropic/OpenAI/Google adapters across internal+consumer |
| ADR-0255 D-17 | ADR-0220 fate — substantially rewritten | ADR Drift-Loop Closure via Keystone Rewrite | Internal portfolio practice; comparable to AWS Bedrock product page evolution 2023→2024 | Silent ADR Drift — keeping a contradictory ADR in force |
| ADR-0255 D-18 | provider-credential BYOK + ToS interaction (owner-declared clearance) | Owner-Declared ToS Clearance | AWS Bedrock per-customer ToS; Anthropic AUP per-organization acceptance; OpenAI per-account ToS | Substrate-Implicit ToS Coverage — assumed ToS without per-credential attestation |

**Total master matrix rows:** 205 (8 + 13 + 18 + 11 + 11 + 12 + 17 + 23 + 12 + 16 + 16 + 23 + 20 + 18 = 218; numbering above includes some sub-decisions split out for separability).

---

## 4. Grouped views by source bucket

A pattern attribution is stronger when the same pattern appears across
multiple unrelated hyperscaler / academic sources. The groupings below
let F2 reviewers run a source-density audit: a decision citing only
one source is weaker than one citing three.

Each row format: `Decision ID — Pattern Name`.

### 4.1 AWS Builder's Library + AWS Well-Architected + AWS engineering blogs

The AWS Builder's Library + AWS Well-Architected pillar pieces +
re:Invent talks are cited across the bundle.

- ADR-0242 D-3 — Unified Multi-Tenant Substrate (AWS shared-substrate)
- ADR-0242 D-7 — First-Class Platform-Owner Account (AWS `aws` system account)
- ADR-0243 D-1 — Single Policy Engine Consolidation (AWS Verified Permissions)
- ADR-0243 D-3 — Coverage-Required Authorization (AWS Well-Architected SEC)
- ADR-0243 D-4 — Layered Policy Composition (AWS SCP + IAM intersection)
- ADR-0243 D-5 — PKI Root + Intermediate Certificate Chain (AWS KMS hierarchy)
- ADR-0243 D-6 — Edge-Cached Policy Evaluation (AWS Verified Permissions production cache)
- ADR-0243 D-9 — Coverage-Enforced Policy (AWS Config conformance packs)
- ADR-0243 D-11 — Static Stability + Fail-Closed (AWS Builder's Library "Static stability")
- ADR-0243 D-12 — Restricted Tenant Self-Policy (AWS SCP + IAM permission boundary)
- ADR-0243 D-13 — Unified Policy + Feature Gate (AWS Verified Permissions feature gates)
- ADR-0244 D-1 — Globally Unique Slug (AWS account-alias rules)
- ADR-0244 D-2 — Hierarchical Principal Path (AWS IAM principal paths)
- ADR-0244 D-2.d — Bounded-Depth Hierarchy (AWS IAM path limit)
- ADR-0244 D-3 — Single Source of Truth Tenant Registry (AWS Organizations master)
- ADR-0244 D-3.c — Capability-Based Authorization (AWS IAM permission boundaries)
- ADR-0244 D-3.dr — Tier-Aware DR Strategy (AWS Resilience Hub tiers)
- ADR-0244 D-4 — Typed Entity Policy Schema (AWS Verified Permissions Cedar entity schema)
- ADR-0244 D-5 — Caller-Side Attribute Resolution (AWS principal-attribute policy conditions)
- ADR-0244 D-6 — Time-Bounded Cross-Tenant Grant (AWS STS AssumeRole)
- ADR-0244 D-7 — Multi-State Tenant Lifecycle with Soft-Delete Window (AWS Organizations close)
- ADR-0244 D-7.h — Cascade-Plus-Tombstone Deletion (AWS Organizations CLOSED account preserves audit)
- ADR-0244 D-8 — Per-Engineer Sandbox Tenant (AWS Cloud9 + AWS Sandboxes)
- ADR-0244 D-10 — Signed Migration Ledger (AWS Database Migration Service patterns)
- ADR-0244 D-12 — Defence-in-Depth via Cedar Fragment (AWS Service Control Policy)
- ADR-0245 D-1 — Foundational-vs-Application Service Tier (AWS Well-Architected v2024-Q4 Pillar 4)
- ADR-0245 D-2 — Manifest-Declared Service Tier (AWS Service Health Dashboard tier classification)
- ADR-0245 D-3 — Per-Service Tier Registration (AWS Service Health Dashboard registry)
- ADR-0245 D-4 — Layered Service Tier DAG (AWS Builders' Library service-layering)
- ADR-0245 D-4.B — Foundational Dependency DAG (AWS Builders' Library "Static stability")
- ADR-0245 D-5 — Peer-Cell Service Pattern (AWS Marketplace + AWS Activate)
- ADR-0245 D-7 — Coverage-Required Tier Classification (AWS Config conformance packs)
- ADR-0245 D-8 — Per-Tier SLO Floor (AWS Well-Architected v2024-Q4 Pillar 4)
- ADR-0245 D-8.c — Markov-Chain Availability Composition (AWS Well-Architected Reliability Pillar)
- ADR-0245 D-9 — Tier-Aware Deprecation (AWS deprecation policy)
- ADR-0246 D-1 — Centralized Policy Service (AWS Verified Permissions re:Invent 2023 BOA303)
- ADR-0246 D-4 — gRPC-Primary with REST Compat (Stripe API design with gRPC for internal)
- ADR-0246 D-5 — Cell-Sharded Stateless Tier with HA (AWS cell-based architecture)
- ADR-0246 D-6 — Static Stability + Edge-Cached Evaluation (AWS Builder's Library "Static Stability")
- ADR-0246 D-7 — Distributed Relational with Application-Aware Sharding (AWS Aurora Limitless)
- ADR-0246 D-8 — PKI Root + Intermediate Certificate Chain (AWS KMS key hierarchy)
- ADR-0246 D-11 — Coverage-Enforced Substrate Doctrine (AWS Config conformance packs)
- ADR-0247 D-1 — Substrate Primitive De-duplication (AWS Bedrock + Step Functions + IAM)
- ADR-0247 D-2 — Internal-CI as Tenant-of-Platform (AWS internal CI as AWS IAM principal)
- ADR-0247 D-3 — Policy-Gated Reflective Tower (AWS Verified Permissions self-modification)
- ADR-0247 D-6 — Three-Tier CD with Auto-Rollback (AWS internal dev/gamma/prod fleets)
- ADR-0247 D-7 — Immutable Workflow Version Pinning (AWS Step Functions versioning)
- ADR-0247 D-8 — Policy-Engine-Gated Self-Modification (AWS Verified Permissions + KMS chain)
- ADR-0247 D-12 — Documented Recovery Procedure (AWS Builder's Library "Static Stability")
- ADR-0248 D-1 — External Dependency Inventory (AWS Well-Architected Reliability pillar)
- ADR-0248 D-3 — Control Plane / Data Plane Separation (AWS Route 53 control plane)
- ADR-0248 D-4 — Per-Tenant-Group Cell (AWS S3, Lambda cellular)
- ADR-0248 D-5 — Peer-Tier Dedicated-Function Cell (AWS Marketplace, AWS IAM Access Analyzer)
- ADR-0248 D-6 — Hot-Path-Intra-Cell (AWS S3 partition boundary)
- ADR-0248 D-7 — Shuffle Sharding (MacCárthaigh 2014 AWS Architecture Blog)
- ADR-0248 D-8 — Static Stability (Weiss/Furr 2020 AWS Builder's Library; ARC404)
- ADR-0248 D-9 — Constant Work (Brooker 2020 AWS Builder's Library)
- ADR-0248 D-10 — Capacity-Aware Auto-Spawn (AWS Lambda concurrency scaling)
- ADR-0248 D-11 — GeoDNS + Edge Failover (AWS Route 53 latency-based routing)
- ADR-0248 D-12 — Audit-Trail-Backed Tenant Migration (AWS Outposts migration)
- ADR-0248 D-13 — Workload-In-Pod Default (AWS EKS)
- ADR-0248 D-14 — VM-Per-Workload Isolation (AWS Firecracker)
- ADR-0248 D-14.alt — KVM-Backed Isolation (AWS Firecracker)
- ADR-0248 D-15 — Distributed Edge POP (AWS CloudFront)
- ADR-0248 D-16 — Build-Ahead-of-Certification (AWS GovCloud built before FedRAMP-High)
- ADR-0249 D-1.3 — Saga Pattern for Distributed Order Workflow (AWS Step Functions Saga blueprint)
- ADR-0249 D-10 — Compensating Saga across Cells (AWS Step Functions Saga)
- ADR-0250 D-1 — Architected-Built-Launched Tri-State (AWS service-availability progression)
- ADR-0250 D-2 — Operationally-Ready-Before-Launch (AWS Builder's Library "Ten things we wish we'd known sooner")
- ADR-0250 D-3 — Per-Market Launch Gate Matrix (AWS per-region service launch sequencing)
- ADR-0250 D-4 — Certification Catalog as Canonical Source (AWS Service Authorisation Reference)
- ADR-0250 D-5 — Eligibility-as-Derived-State (AWS Organization OU eligibility)
- ADR-0250 D-6 — Layered Policy Composition (AWS Verified Permissions cell-policy composition)
- ADR-0250 D-7 — Pre-Launch Runbook Discipline (AWS Operational Readiness Review checklist)
- ADR-0250 D-8 — Multi-Year Capability Roadmap (AWS 2006-2024 service + region progression)
- ADR-0250 D-9 — No-Bypass Defense in Depth (AWS SCP + IAM + permission boundary triple-check)
- ADR-0250 D-11 — Audit-Chain-Anchored Compliance Evidence (AWS Audit Manager evidence packs)
- ADR-0250 D-12 — Graceful Capability Sunset (AWS deprecation policy 12-month notice)
- ADR-0251 D-1 — Compliance-as-Packaged-Bundle (AWS Audit Manager framework catalog)
- ADR-0251 D-2 — Signed Policy Bundle Lifecycle (AWS Verified Permissions policy store)
- ADR-0251 D-3 — Tenant-Installs-Compliance-Regime (AWS Control Tower compliance guardrail installation)
- ADR-0251 D-4 — Cell-Certification-as-Discrete-Levels (AWS regions partitioned Commercial vs GovCloud)
- ADR-0251 D-5 — Mandatory-Compliance-Pinning (AWS Organizations + Control Tower account-to-OU pinning)
- ADR-0251 D-6 — Cross-Tenant Policy Gate (AWS Verified Permissions cross-account evaluation; AWS RAM)
- ADR-0251 D-7 — Durable-Workflow-Driven Compliance-Agreement Lifecycle (AWS Artifact agreement automation)
- ADR-0251 D-8 — Per-Jurisdiction Breach-Notification Workflow (AWS GuardDuty + Detective + Security Hub)
- ADR-0251 D-9 — Shared De-Identification Substrate (AWS Glue DataBrew PII transforms)
- ADR-0251 D-10 — Hierarchical-Key-Management Substrate (AWS KMS + CloudHSM)
- ADR-0251 D-13 — Per-Stream Audit-Chain with Per-Pack Retention (AWS CloudTrail Lake)
- ADR-0251 D-14 — Compositional Policy Semantics (AWS SCP + IAM policy intersection)
- ADR-0251 D-16 — Auto-Emit Auditor Evidence Package (AWS Audit Manager evidence collection)
- ADR-0252 D-2 — Atomic-Clock-Backed External Consistency (AWS Aurora DSQL re:Invent 2024)
- ADR-0252 D-4 — Stripe Idempotency Key (AWS SDK retryable APIs 2018+)
- ADR-0252 D-5 — Saga + Compensation, Not Lock (AWS Distributed Sagas)
- ADR-0252 D-6 — Per-Cell Periodic Scheduling (AWS EventBridge per-region scheduler model)
- ADR-0252 D-7 — Linear Time Smear (AWS announcement 2015+2016)
- ADR-0252 D-9 — Per-Cell Idempotency Store (AWS Step Functions per-region state)
- ADR-0252 D-11 — Policy-Enforced Deadline (AWS Builders Library "request budgets")
- ADR-0252 D-12 — Deterministic Workflow Replay (AWS Step Functions checkpoint replay)
- ADR-0252 D-14 — Policy-as-Feature-Flag (AWS Verified Permissions feature gates)
- ADR-0253 D-1.a — Anycast Apex DNS (AWS Route 53 Anycast)
- ADR-0253 D-1.b — Zone integrity attestation (AWS Route 53 DNSSEC)
- ADR-0253 D-2.a — Planetary Edge POP (AWS CloudFront 350+ POPs)
- ADR-0253 D-4 — Post-Quantum Hybrid KEX (AWS s2n-tls kyber-tls13)
- ADR-0253 D-5 — Modern Transport at Edge (AWS CloudFront HTTP/3 since 2022)
- ADR-0253 D-7 — Workload identity primitive (IRSA at AWS EKS)
- ADR-0253 D-10 — Zero-trust egress (AWS VPC security groups)
- ADR-0253 D-13 — Tenant-aware residency routing (AWS Route 53 GeoDNS)
- ADR-0253 D-16 — Webhook reliability triplet (AWS EventBridge retry)
- ADR-0254 D-1.1 — Shuffle-Sharded Multi-Tenant SaaS (AWS Builders' Library 2017-2024)
- ADR-0254 D-1.2 — Dedicated Cell Pattern (AWS Outposts dedicated)
- ADR-0254 D-1.4 — Connected Edge / Hybrid On-Prem (AWS Outposts connected)
- ADR-0254 D-3 — Cell as Unit of Deployment (Werner Vogels re:Invent 2018)
- ADR-0254 D-8 — Tiered Support Matrix (AWS Support tiers)
- ADR-0254 D-9 — Cost-Aligned Pricing (AWS On-Demand vs Reserved vs Savings Plan)
- ADR-0254 D-13 — Workflow-Saga Durable Migration (AWS Step Functions saga pattern)
- ADR-0254 D-15 — Vendor Reference Architecture (AWS Outposts hardware spec)
- ADR-0254 D-16 — Per-Model Onboarding Workflow (AWS Partner-Led Onboarding)
- ADR-0255 D-1 — Substrate + Brand Surface Layering (AWS Bedrock + product UI separation)
- ADR-0255 D-2 — Audience-Neutral Substrate (AWS Bedrock model invocation API)
- ADR-0255 D-4 — provider-credential BYOK SecretReference + Owner Declaration (AWS Bedrock customer-managed provider credentials)
- ADR-0255 D-6 — Stateless Substrate + Durable Composition (AWS Bedrock stateless API + Step Functions durability)
- ADR-0255 D-7 — Caller-Side Retrieval (AWS Bedrock Knowledge Bases opt-in caller side)
- ADR-0255 D-8 — Embeddings Substrate Promotion (AWS Bedrock Knowledge Bases retrieval substrate)
- ADR-0255 D-9 — Fine-Tuning Substrate Promotion (AWS Bedrock model customization)
- ADR-0255 D-10 — Hybrid Model Serving (AWS Bedrock external + Bedrock-hosted + custom model import)
- ADR-0255 D-11 — Per-Cell Substrate Deployment (AWS region + AZ Bedrock deployment)
- ADR-0255 D-12 — Tool-Call Ingress + Dispatcher Separation (AWS Bedrock Agents Action Groups)
- ADR-0255 D-15 — Audience-As-Call-Tag (AWS Bedrock Guardrails per-policy attachment)
- ADR-0255 D-16 — Substrate Consolidation (AWS Bedrock absorbing prior per-team provider adapters)
- ADR-0255 D-18 — Owner-Declared ToS Clearance (AWS Bedrock per-customer ToS)

**Source-bucket count:** ~120 decision-citations into AWS sources. AWS is
the densest source, reflecting AWS's role as the broadest hyperscaler-
pattern catalog (Builder's Library + Well-Architected + service docs).

### 4.2 Stripe Engineering

- ADR-0242 D-1 — Eat-Your-Own-Dogfood at Platform Level (Stripe Engineering Quora 2013)
- ADR-0242 D-4 — Dogfooded Compliance Pipeline (Stripe SOC 2 includes Stripe's internal use)
- ADR-0242 D-8 — Ephemeral Tenant Pattern (Stripe test mode)
- ADR-0243 — (general Stripe API design lineage in D-1 through D-13)
- ADR-0244 D-1 — Globally Unique Slug + DNS-Compatible Segments (Stripe account ID conventions)
- ADR-0244 D-3 — Single Source of Truth Tenant Registry (Stripe Accounts table)
- ADR-0244 D-3.c — Capability-Based Authorization (Stripe account capabilities)
- ADR-0244 D-5 — Caller-Side Attribute Resolution (Stripe webhook tenant_id in payload)
- ADR-0244 D-6 — Time-Bounded Cross-Tenant Grant (Stripe platform-on-behalf-of)
- ADR-0244 D-6.3 — Platform-on-Behalf-Of Pattern (Stripe Connect)
- ADR-0244 D-8 — Per-Engineer Sandbox Tenant (Stripe Test Mode)
- ADR-0244 D-11 — Closed-Enum Tenant Classification (Stripe account type enum)
- ADR-0246 D-4 — gRPC-Primary with REST Compat (Stripe API design)
- ADR-0247 D-2 — Internal-CI as Tenant-of-Platform (Stripe internal CI as Stripe tenant)
- ADR-0248 D-3 — Control Plane / Data Plane Separation (Stripe Cells 2024 control plane)
- ADR-0248 D-4 — Per-Tenant-Group Cell (Stripe Cells 2024 per-account cell)
- ADR-0248 D-5 — Peer-Tier Dedicated-Function Cell (Stripe Connect)
- ADR-0248 D-6 — Hot-Path-Intra-Cell (Stripe per-account locality)
- ADR-0248 D-10 — Capacity-Aware Auto-Spawn (Stripe Cells 2024 sizing)
- ADR-0248 D-12 — Audit-Trail-Backed Tenant Migration (Stripe Cells 2024 account migration)
- ADR-0249 D-1 — Substrate-Shared, Surface-Specialised (Stripe Tenant/RBAC Packaging docs)
- ADR-0249 D-1.3 — Saga Pattern for Distributed Order Workflow (Stripe Order API)
- ADR-0249 D-1.7 — Pricing-Promotion-Tax Substrate (Stripe Tax)
- ADR-0249 D-1.8 — Multi-Signal Trust Score with Cold-Start (Stripe Radar)
- ADR-0249 D-2 — Per-Category Surface BC (Stripe Connect's diverse partners)
- ADR-0249 D-7 — Cold-Start with Graduated Limits (Stripe Radar)
- ADR-0249 D-9 — Phased Activation by Certification (Stripe Atlas + + Treasury phased launches)
- ADR-0250 D-2 — Operationally-Ready-Before-Launch (Stripe Engineering blog 2014-2020)
- ADR-0250 D-3 — Per-Market Launch Gate Matrix (Stripe per-country onboarding sequence)
- ADR-0250 D-8 — Multi-Year Capability Roadmap (Stripe 2011-2025 product + country progression)
- ADR-0250 D-10 — Sandbox-Tenant Pilot (Stripe test mode)
- ADR-0250 D-12 — Graceful Capability Sunset (Stripe API version sunset cadence)
- ADR-0251 D-7 — Durable-Workflow-Driven Compliance-Agreement Lifecycle (Stripe onboarding)
- ADR-0252 D-4 — Stripe Idempotency Key (Brandur Leach 2014; Stripe API docs)
- ADR-0252 D-8 — Opaque Self-Describing Key (Stripe key shape)
- ADR-0253 D-3.a — Modern Crypto at Edge (Stripe API TLS 1.3 only)
- ADR-0253 D-14 — Multi-protocol API surface (Stripe API REST 3.x)
- ADR-0253 D-15 — Realtime push tier (Stripe SSE conventions)
- ADR-0253 D-16 — Webhook reliability triplet (Stripe webhook HMAC + idempotency)
- ADR-0253 D-17 — Phased self-hosting migration (Stripe reached self-hosted edge by year 5-7)
- ADR-0255 D-4 — provider-credential BYOK SecretReference + Owner Declaration (Stripe provider credential docs)
- ADR-0255 D-13 — Streaming via SSE Conventions (Stripe SSE conventions)

**Source-bucket count:** ~42 decision-citations into Stripe sources.
Stripe is the second densest bucket — reflecting Stripe's role as the
canonical commerce-substrate reference pattern.

### 4.3 Google SRE Workbook / Google Cloud / Google research

- ADR-0242 D-7 — First-Class Platform-Owner Account (GCP `google` system project)
- ADR-0243 D-1 — Single Policy Engine Consolidation (GCP Org Policy consolidation)
- ADR-0243 D-4 — Layered Policy Composition (GCP Org Policy)
- ADR-0243 D-9 — Coverage-Enforced Policy (Google SRE Workbook ch. 4)
- ADR-0244 D-2 — Hierarchical Principal Path (GCP resource hierarchy)
- ADR-0244 D-2.d — Bounded-Depth Hierarchy (GCP folder depth limit)
- ADR-0244 D-3 — Single Source of Truth Tenant Registry (GCP Resource Manager hierarchy)
- ADR-0244 D-7 — Multi-State Tenant Lifecycle (GCP Project delete 30-day soft-delete)
- ADR-0244 D-7.h — Cascade-Plus-Tombstone Deletion (GCP Project SOFT_DELETED preserves logs)
- ADR-0244 D-12 — Defence-in-Depth via Cedar Fragment (GCP Org Policy constraints)
- ADR-0245 D-1 — Foundational-vs-Application Service Tier (Google Cloud Deprecation Policy 2024)
- ADR-0245 D-2 — Manifest-Declared Service Tier (GCP service tier API)
- ADR-0245 D-3 — Per-Service Tier Registration (GCP service catalog)
- ADR-0245 D-4 — Layered Service Tier DAG (GCP service dependency graph)
- ADR-0245 D-4.B — Foundational Dependency DAG (GCP Borg/Omega layering, Verma et al. 2016)
- ADR-0245 D-7 — Coverage-Required Tier Classification (Google SRE Workbook ch. 4)
- ADR-0245 D-8 — Per-Tier SLO Floor (Google SRE Workbook ch. 2)
- ADR-0245 D-8.c — Markov-Chain Availability Composition (Google SRE Workbook ch. 2)
- ADR-0245 D-9 — Tier-Aware Deprecation (Google Cloud Deprecation Policy 2024)
- ADR-0246 D-1 — Centralized Policy Service (Google Org Policy)
- ADR-0246 D-7 — Distributed Relational with Application-Aware Sharding (Google Spanner external consistency)
- ADR-0246 D-10 — Tiered DR + Per-Microservice SLO Ownership (Google SRE Workbook ch. 2)
- ADR-0246 D-11 — Coverage-Enforced Substrate Doctrine (Google SRE Workbook ch. 4)
- ADR-0247 D-1 — Substrate Primitive De-duplication (GCP Vertex AI + Workflows + IAM Conditions)
- ADR-0247 D-2 — Internal-CI as Tenant-of-Platform (Google internal CI as Borg tenant)
- ADR-0247 D-6 — Three-Tier CD with Auto-Rollback (Google canary → fleet rollout)
- ADR-0247 D-7 — Immutable Workflow Version Pinning (GCP Workflows versioning)
- ADR-0247 D-11 — Coverage-Required Self-Modification (Google SRE Workbook ch. 4)
- ADR-0247 D-12 — Documented Recovery Procedure (Google SRE incident-response runbooks)
- ADR-0248 D-1 — External Dependency Inventory (Google CRE Book ch. 8)
- ADR-0248 D-3 — Control Plane / Data Plane Separation (GCP Spanner zone-master separation)
- ADR-0248 D-6 — Hot-Path-Intra-Cell (Google Spanner replica locality)
- ADR-0248 D-13 — Workload-In-Pod Default (Google Kubernetes Engine)
- ADR-0249 D-13 — Federated Search + OLAP Ranking (Algolia federation; hybrid stack)
- ADR-0250 D-7 — Pre-Launch Runbook Discipline (Google SRE Workbook ch. 18 PRR)
- ADR-0251 D-3 — Tenant-Installs-Compliance-Regime (Google Assured Workloads regime activation)
- ADR-0251 D-4 — Cell-Certification-as-Discrete-Levels (Google Cloud regional compliance designations)
- ADR-0251 D-5 — Mandatory-Compliance-Pinning (Google Cloud Project assured-workloads binding)
- ADR-0251 D-6 — Cross-Tenant Policy Gate (Google IAM cross-project deny)
- ADR-0251 D-9 — Shared De-Identification Substrate (Google Cloud DLP API)
- ADR-0251 D-10 — Hierarchical-Key-Management Substrate (Google Cloud KMS + Cloud HSM + EKM)
- ADR-0251 D-13 — Per-Stream Audit-Chain with Per-Pack Retention (Google Cloud Audit Logs retention buckets)
- ADR-0251 D-14 — Compositional Policy Semantics (GCP Org Policy + Project Policy union)
- ADR-0251 D-16 — Auto-Emit Auditor Evidence Package (Google Cloud Assured Workloads evidence)
- ADR-0252 D-1 — Hybrid Logical Clock (CockroachDB / Spanner lineage)
- ADR-0252 D-2 — Atomic-Clock-Backed External Consistency (Spanner OSDI 2012)
- ADR-0252 D-5 — Saga + Compensation, Not Lock (Google SRE Workbook Ch.24)
- ADR-0252 D-6 — Per-Cell Periodic Scheduling (SRE Workbook Ch.24; Klein "Cron at scale" 2018)
- ADR-0252 D-7 — Linear Time Smear (Google blog 2008+2011)
- ADR-0252 D-13 — Uncertainty-Bounded Time (Spanner TT uncertainty bound)
- ADR-0253 D-1.a — Anycast Apex DNS (Google Cloud DNS Anycast 100+ POPs)
- ADR-0253 D-1.c — Client DNS privacy (Google 8.8.8.8 DoH)
- ADR-0253 D-5 — Modern Transport at Edge (Google HTTP/3 universal)
- ADR-0253 D-6 — Layered L3/L4 + L7 mesh (Google GKE Dataplane V2)
- ADR-0253 D-7 — Workload identity primitive (Google ALTS)
- ADR-0253 D-9 — Per-pair encrypted tunnel (Google cross-cloud Anthos)
- ADR-0253 D-11 — Layered load balancing (GKE service mesh)
- ADR-0253 D-12 — Own ASN + RPKI (Google ASN 15169)
- ADR-0253 D-14 — Multi-protocol API surface (Google gRPC)
- ADR-0254 D-1.4 — Connected Edge / Hybrid On-Prem (Google Anthos)

**Source-bucket count:** ~60 decision-citations into Google sources.

### 4.4 Apple — WWDC, Platform Architecture, App Store

- ADR-0242 D-1 — Eat-Your-Own-Dogfood at Platform Level (Apple WWDC 2024 keynote)
- ADR-0245 D-1 — Foundational-vs-Application Service Tier (Apple Platform Architecture 2024)
- ADR-0245 D-2 — Manifest-Declared Service Tier (Apple Framework Index)
- ADR-0245 D-3 — Per-Service Tier Registration (Apple Framework Index)
- ADR-0245 D-4 — Layered Service Tier DAG (Apple Framework dependency rules)
- ADR-0245 D-4.B — Foundational Dependency DAG (Apple Frameworks Reference dependency layers)
- ADR-0245 D-6 — Build-Ahead-of-Certification (Apple beta-framework pattern)
- ADR-0245 D-7 — Coverage-Required Tier Classification (Apple Xcode static analysis)
- ADR-0245 D-9 — Tier-Aware Deprecation (Apple framework SemVer)
- ADR-0249 D-2 — Per-Category Surface BC (Apple Music + Apple TV+ + Apple Arcade on shared App Store ID)
- ADR-0249 D-1 — Substrate-Shared, Surface-Specialised (Apple One subscription bundle)
- ADR-0249 D-9 — Phased Activation by Certification (Apple Pay country-by-country activation)
- ADR-0249 D-12 — Per-Category Policy Overlay (Apple App Store category-specific review: Health/Medical)
- ADR-0250 D-1 — Architected-Built-Launched Tri-State (Apple Pay country progression)
- ADR-0250 D-2 — Operationally-Ready-Before-Launch (Apple Pay "we launched only when we were ready" quote)
- ADR-0250 D-3 — Per-Market Launch Gate Matrix (Apple Pay per-country launch playbook)
- ADR-0250 D-5 — Eligibility-as-Derived-State (Apple App Store entitlement-as-derived)
- ADR-0250 D-8 — Multi-Year Capability Roadmap (Apple Pay 2014-2024 country launch progression)
- ADR-0255 D-1 — Substrate + Brand Surface Layering (Apple Intelligence WWDC 2024 keynote)
- ADR-0255 D-2 — Audience-Neutral Substrate (Apple Foundation Models API)
- ADR-0255 D-3 — Layered Brand Surface (Apple Intelligence consumer surfaces)
- ADR-0255 D-5 — Multi-Modal Day-One Provider Adapter (Apple Intelligence late image-gen scramble = anti-pattern)
- ADR-0255 D-11 — Per-Cell Substrate Deployment (Apple per-region private cloud compute)
- ADR-0255 D-15 — Audience-As-Call-Tag (Apple Intelligence per-surface brand)

**Source-bucket count:** ~24 decision-citations into Apple sources.

### 4.5 Cloudflare Engineering

- ADR-0243 D-6 — Edge-Cached Policy Evaluation (Cloudflare Workers KV)
- ADR-0246 D-4 — gRPC-Primary with REST Compat (Cloudflare Workers gRPC)
- ADR-0246 D-6 — Static Stability + Edge-Cached Evaluation (Cloudflare Workers KV)
- ADR-0248 D-11 — GeoDNS + Edge Failover (Cloudflare GeoDNS + edge POPs)
- ADR-0248 D-15 — Distributed Edge POP (Cloudflare edge ~300 POPs)
- ADR-0253 D-1.a — Anycast Apex DNS (Cloudflare DNS Anycast 300+ POPs)
- ADR-0253 D-1.b — Zone integrity attestation (Cloudflare DNSSEC)
- ADR-0253 D-1.c — Client DNS privacy (Cloudflare 1.1.1.1 DoH)
- ADR-0253 D-2.a — Planetary Edge POP (Cloudflare Workers 300+ POPs)
- ADR-0253 D-2.b — Rust-based Edge Proxy at Scale (Cloudflare Pingora open-source 2024)
- ADR-0253 D-3.a — Modern Crypto at Edge (Cloudflare zones TLS 1.3 by default)
- ADR-0253 D-4 — Post-Quantum Hybrid KEX (Cloudflare X25519MLKEM768 2024)
- ADR-0253 D-5 — Modern Transport at Edge (Cloudflare HTTP/3 since 2020)
- ADR-0253 D-12 — Own ASN + RPKI (Cloudflare ASN 13335)
- ADR-0253 D-13 — Tenant-aware residency routing (Cloudflare GeoSteering)
- ADR-0253 D-17 — Phased self-hosting migration (Cloudflare reached self-hosted edge by year 5-7)
- ADR-0253 D-18 — Phased self-hosting DNS (Cloudflare DNS self-host)

**Source-bucket count:** ~17 decision-citations into Cloudflare sources.

### 4.6 Palantir Apollo / Forward

- ADR-0242 D-1 — Eat-Your-Own-Dogfood at Platform Level (Palantir Apollo product docs)
- ADR-0254 D-1.4 — Connected Edge / Hybrid On-Prem (Palantir Apollo connected)
- ADR-0254 D-1.5 — Air-Gapped Bundle Delivery (Palantir Apollo air-gapped)
- ADR-0254 D-2 — Single-Build Multi-Deployment (Palantir Apollo one-Foundry-build)
- ADR-0254 D-4 — Palantir Apollo Pattern (Palantir Forward 2023+2024 keynotes; Apollo product page)
- ADR-0254 D-8 — Tiered Support Matrix (Palantir Mission Support)
- ADR-0254 D-14 — Merkle-Sealed Bundled Audit Export (Palantir Apollo air-gap audit pattern)
- ADR-0254 D-16 — Per-Model Onboarding Workflow (Palantir Mission Specialist onboarding)

**Source-bucket count:** 8 decision-citations into Palantir sources.

### 4.7 Microsoft Azure / Microsoft Purview / Microsoft 365

- ADR-0242 D-3 — Unified Multi-Tenant Substrate (Microsoft 365 multi-tenant Exchange Online)
- ADR-0242 D-4 — Dogfooded Compliance Pipeline (Microsoft 365 includes Microsoft IT)
- ADR-0242 D-7 — First-Class Platform-Owner Account (Microsoft "First-Party Tenant" pattern in Azure AD)
- ADR-0244 D-2 — Hierarchical Principal Path (Azure RBAC scope)
- ADR-0244 D-2.d — Bounded-Depth Hierarchy (Azure subscription nesting limit)
- ADR-0244 D-3.dr — Tier-Aware DR Strategy (Azure Site Recovery patterns)
- ADR-0244 D-6 — Time-Bounded Cross-Tenant Grant (Azure AAD B2B Collaboration)
- ADR-0244 D-7 — Multi-State Tenant Lifecycle (Azure AD tenant delete 30-day recovery)
- ADR-0244 D-11 — Closed-Enum Tenant Classification (Azure AAD tenant type)
- ADR-0245 D-1 — Foundational-vs-Application Service Tier (Microsoft Cloud Adoption Framework 2024)
- ADR-0245 D-8 — Per-Tier SLO Floor (Microsoft Azure Well-Architected)
- ADR-0246 D-7 — Distributed Relational with Application-Aware Sharding (Citus design, Microsoft acquired 2019)
- ADR-0247 D-1 — Substrate Primitive De-duplication (Azure AI Foundry + Logic Apps + Azure Policy)
- ADR-0248 D-13 — Workload-In-Pod Default (Microsoft Azure AKS)
- ADR-0250 D-1 — Architected-Built-Launched Tri-State (Microsoft Azure Vertical Industry Solutions launch shape)
- ADR-0250 D-4 — Certification Catalog as Canonical Source (Microsoft Trust Center compliance offerings catalog)
- ADR-0250 D-7 — Pre-Launch Runbook Discipline (Microsoft Operations Manual)
- ADR-0251 D-1 — Compliance-as-Packaged-Bundle (Microsoft Purview assessment templates)
- ADR-0251 D-3 — Tenant-Installs-Compliance-Regime (Microsoft Purview tenant assessment activation)
- ADR-0251 D-4 — Cell-Certification-as-Discrete-Levels (Azure compliance offerings per region)
- ADR-0251 D-5 — Mandatory-Compliance-Pinning (Azure Subscription compliance binding)
- ADR-0251 D-6 — Cross-Tenant Policy Gate (Azure RBAC cross-tenant guest access)
- ADR-0251 D-8 — Per-Jurisdiction Breach-Notification Workflow (Microsoft Sentinel + Compliance Manager)
- ADR-0251 D-9 — Shared De-Identification Substrate (Microsoft Presidio)
- ADR-0251 D-10 — Hierarchical-Key-Management Substrate (Azure Key Vault Managed HSM)
- ADR-0251 D-13 — Per-Stream Audit-Chain with Per-Pack Retention (Microsoft Sentinel retention tiers)
- ADR-0251 D-16 — Auto-Emit Auditor Evidence Package (Microsoft Purview Compliance Manager scorecard export)
- ADR-0252 D-3 — Tiered Consistency Model (Azure Cosmos DB 5-level model)
- ADR-0252 D-16 — Tiered Isolation with Policy Opt-In (Azure Cosmos consistency tiers)
- ADR-0253 D-3.b — L7 Ingress Termination (AKS Envoy)
- ADR-0254 D-1.4 — Connected Edge / Hybrid On-Prem (Azure Arc)
- ADR-0255 D-1 — Substrate + Brand Surface Layering (Azure AI Foundry per-deployment endpoint scoping)
- ADR-0255 D-2 — Audience-Neutral Substrate (Azure AI Foundry inference endpoint)
- ADR-0255 D-4 — provider-credential BYOK SecretReference + Owner Declaration (Azure provider credential SecretReference pattern)
- ADR-0255 D-5 — Multi-Modal Day-One Provider Adapter (Azure AI Foundry multi-modal)
- ADR-0255 D-7 — Caller-Side Retrieval (Azure AI Foundry retrieval connectors)
- ADR-0255 D-9 — Fine-Tuning Substrate Promotion (Azure AI Foundry fine-tune)
- ADR-0255 D-10 — Hybrid Model Serving (Azure AI Foundry, Azure OpenAI + Llama + custom)
- ADR-0255 D-11 — Per-Cell Substrate Deployment (Azure AI Foundry per-region endpoint)
- ADR-0255 D-15 — Audience-As-Call-Tag (Azure AI Foundry per-deployment scoping)
- ADR-0255 D-16 — Substrate Consolidation (Azure AI Foundry consolidating Azure AI Studio + Azure ML + Azure OpenAI)

**Source-bucket count:** ~41 decision-citations into Microsoft sources.

### 4.8 OPA + Cedar + Sigstore + formal policy

- ADR-0243 D-1 — Single Policy Engine Consolidation (Netflix OPA-at-scale)
- ADR-0243 D-2 — Signed Policy Authoring Lifecycle (Sigstore + cosign attestations)
- ADR-0243 D-3 — Coverage-Required Authorization (NIST SP 800-162 ABAC)
- ADR-0243 D-5 — PKI Root + Intermediate Certificate Chain (Sigstore Rekor)
- ADR-0243 D-10 — Hot-Reload Configuration Distribution (etcd watch pattern; Apollo / Argo CD sync)
- ADR-0243 D-11 — Static Stability + Fail-Closed (NIST SP 800-207 deny-by-default)
- ADR-0244 D-4 — Typed Entity Policy Schema (OPA structured-data policies)
- ADR-0246 D-2 — Single-Concern Bounded Contexts (DDD, Evans 2003; ADR-0132 no-grouping forward policy)
- ADR-0246 D-3 — Hexagonal Architecture with Port-in-Kernel (Cockburn 2005 Hexagonal)
- ADR-0246 D-8 — PKI Root + Intermediate Certificate Chain (Sigstore Rekor)
- ADR-0246 D-9 — Substrate Cohesion via PRD Amendment (DDD context-mapping, Evans 2003; ADR pattern, Nygard 2011)
- ADR-0247 D-8 — Policy-Engine-Gated Self-Modification (Sigstore signed-fragment provenance)
- ADR-0250 D-9 — No-Bypass Defense in Depth (NIST SP 800-207 Zero Trust no-bypass)
- ADR-0250 D-11 — Audit-Chain-Anchored Compliance Evidence (Sigstore Rekor immutability)
- ADR-0251 D-2 — Signed Policy Bundle Lifecycle with Transparency Log (Sigstore Rekor + cosign)
- ADR-0252 D-11 — Policy-Enforced Deadline (NIST SP 800-207 ZTA per-call evaluation)
- ADR-0253 D-10 — Zero-trust egress (NIST SP 800-207)
- ADR-0254 D-5 — TUF + Cosign + SLSA L3 Distribution (The Update Framework spec; Sigstore Cosign 2021+; SLSA L3)
- ADR-0254 D-6 — CDS Bundle Delivery (NSA RTB guidance; NCDSMO approved CDS products list)

**Source-bucket count:** ~19 decision-citations into formal policy /
Sigstore / OPA / NIST sources.

### 4.9 IETF RFCs + Mozilla + Web Standards

- ADR-0243 D-5 — PKI Root + Intermediate Certificate Chain (RFC 5280 X.509)
- ADR-0244 D-1 — Globally Unique Slug + DNS-Compatible Segments (RFC 1035)
- ADR-0252 D-4 — Stripe Idempotency Key (IETF draft `idempotency-key-header-09`)
- ADR-0253 D-1.b — Zone integrity attestation (RFC 4034/4035)
- ADR-0253 D-1.c — Client DNS privacy (RFC 8484 + RFC 7858)
- ADR-0253 D-3.a — Modern Crypto at Edge (Mozilla SSL Configuration Generator "modern")
- ADR-0253 D-4 — Post-Quantum Hybrid KEX (NIST FIPS 203)

**Source-bucket count:** 7 decision-citations into IETF + Mozilla.

### 4.10 NIST + FedRAMP + DoD standards

- ADR-0243 D-3 — Coverage-Required Authorization (NIST SP 800-162 ABAC)
- ADR-0243 D-7 — Audit-Every-Decision (NIST SP 800-92 audit log standards; SOC 2 CC7.2)
- ADR-0243 D-11 — Static Stability + Fail-Closed (NIST SP 800-207 deny-by-default)
- ADR-0247 D-12 — Documented Recovery Procedure (NIST SP 800-34 contingency planning)
- ADR-0248 D-16 — Build-Ahead-of-Certification (AWS GovCloud + FedRAMP-High pattern)
- ADR-0250 D-9 — No-Bypass Defense in Depth (NIST SP 800-207 Zero Trust)
- ADR-0250 D-11 — Audit-Chain-Anchored Compliance Evidence (Sedona Conference legal-hold)
- ADR-0251 D-15 — Hierarchical Certification Inheritance (FedRAMP High ⊇ Moderate ⊇ Low; CMMC L3 ⊇ L2 ⊇ L1; ISO 27001 implies ISO 27002)
- ADR-0252 D-11 — Policy-Enforced Deadline (NIST SP 800-207 ZTA)
- ADR-0253 D-4 — Post-Quantum Hybrid KEX (NIST FIPS 203)
- ADR-0253 D-10 — Zero-trust egress (NIST SP 800-207)
- ADR-0254 D-1.5 — Air-Gapped Bundle Delivery (defense IL5/6 reference architectures)
- ADR-0254 D-6 — CDS Bundle Delivery (NSA RTB guidance; NCDSMO approved CDS products list)
- ADR-0254 D-15 — Vendor Reference Architecture (defense IL5/6 hardware spec)

**Source-bucket count:** 14 decision-citations into NIST / FedRAMP / DoD.

### 4.11 Salesforce + Snowflake + Databricks + Confluent

- ADR-0242 D-3 — Unified Multi-Tenant Substrate (Salesforce multi-tenant architecture)
- ADR-0244 D-11 — Closed-Enum Tenant Classification (Salesforce customer-vs-partner-vs-internal)
- ADR-0244 D-6.3 — Platform-on-Behalf-Of Pattern (Salesforce Partner Portal)
- ADR-0245 D-1 — Foundational-vs-Application Service Tier (Salesforce Trust Documentation 2024)
- ADR-0245 D-5 — Peer-Cell Service Pattern (Salesforce AppExchange peer-cell)
- ADR-0248 D-4 — Per-Tenant-Group Cell (Salesforce Pods)
- ADR-0248 D-5 — Peer-Tier Dedicated-Function Cell (Salesforce AppExchange)
- ADR-0249 D-3 — Existing-Product Refactor onto Shared Substrate (Salesforce AppExchange onto Lightning Platform)
- ADR-0250 D-4 — Certification Catalog as Canonical Source (Salesforce Trust + Compliance)
- ADR-0250 D-12 — Graceful Capability Sunset (Salesforce End of Life roadmap)
- ADR-0251 D-1 — Compliance-as-Packaged-Bundle (Salesforce HealthCloud / GovCloud SKUs; Databricks Compliance Security Profile)
- ADR-0254 D-1.1 — Shuffle-Sharded Multi-Tenant SaaS (Salesforce Trust Cloud architecture)
- ADR-0254 D-1.2 — Dedicated Cell Pattern (Salesforce Government Cloud; Snowflake Virtual Private Snowflake)
- ADR-0254 D-1.3 — Bring-Your-Own-Cloud (BYOC) (Snowflake BYOC 2022; Confluent BYOC 2023; Astronomer BYOC 2024; Databricks Customer-Managed VPC 2021)
- ADR-0254 D-2 — Single-Build Multi-Deployment (Snowflake "build Snowflake on Snowflake" blog 2022; Confluent Platform vs Cloud single-codebase)
- ADR-0254 D-8 — Tiered Support Matrix (Salesforce Premier vs Standard)
- ADR-0254 D-9 — Cost-Aligned Pricing (Snowflake credit consumption; Confluent per-model SKU)
- ADR-0254 D-11 — IAM-Delegated Customer-Account Provisioning (Snowflake BYOC IAM pattern; Databricks Customer-Managed VPC IAM)
- ADR-0254 D-13 — Workflow-Saga Durable Migration (Confluent Cluster Linking migration)
- ADR-0254 D-16 — Per-Model Onboarding Workflow (Salesforce Onboarding flows)
- ADR-0255 D-3 — Layered Brand Surface (Salesforce Einstein consumer UI atop Einstein substrate)

**Source-bucket count:** ~21 decision-citations into Salesforce/Snowflake/Databricks/Confluent.

### 4.12 Anthropic + OpenAI + Google Gemini (model-vendor docs)

- ADR-0247 D-3 — Policy-Gated Reflective Tower (Anthropic Console self-modification)
- ADR-0247 D-7 — Immutable Workflow Version Pinning (Temporal versioning; mentioned alongside)
- ADR-0255 D-5 — Multi-Modal Day-One Provider Adapter (GPT-4o multi-modal 2024-05; Claude 3.5 Sonnet 2024-06; Gemini 1.5 Pro 2024-02)
- ADR-0255 D-6 — Stateless Substrate + Durable Composition (Anthropic message batches + workflow composition)
- ADR-0255 D-7 — Caller-Side Retrieval (Anthropic prompt-caching with caller-built context)
- ADR-0255 D-9 — Fine-Tuning Substrate Promotion (OpenAI fine-tune API)
- ADR-0255 D-12 — Tool-Call Ingress + Dispatcher Separation (Anthropic MCP architecture)
- ADR-0255 D-13 — Streaming via SSE Conventions (Anthropic streaming API; OpenAI streaming API)
- ADR-0255 D-14 — Opt-In Session State (OpenAI Assistants v2)
- ADR-0255 D-18 — Owner-Declared ToS Clearance (Anthropic AUP per-organization acceptance; OpenAI per-account ToS)

**Source-bucket count:** ~10 decision-citations into model-vendor docs.

### 4.13 Temporal + AWS Step Functions + Workflow engines

- ADR-0247 D-7 — Immutable Workflow Version Pinning (Temporal workflow versioning)
- ADR-0249 D-1.3 — Saga Pattern for Distributed Order Workflow (Temporal.io docs)
- ADR-0249 D-10 — Compensating Saga across Cells (Temporal cross-cluster workflow)
- ADR-0252 D-5 — Saga + Compensation, Not Lock (Temporal docs)
- ADR-0252 D-9 — Per-Cell Idempotency Store (Temporal per-cluster idempotency)
- ADR-0252 D-12 — Deterministic Workflow Replay (Temporal replay; Cadence replay)
- ADR-0254 D-13 — Workflow-Saga Durable Migration (AWS Step Functions saga pattern)

**Source-bucket count:** 7 decision-citations into Temporal-class
workflow engines.

### 4.14 Distributed systems academia + textbooks

- ADR-0245 D-8.c — Markov-Chain Availability Composition (Pinheiro et al. 2007)
- ADR-0245 D-4.B — Foundational Dependency DAG (Verma et al. 2016 — Borg/Omega)
- ADR-0246 D-3 — Hexagonal Architecture with Port-in-Kernel (Cockburn 2005)
- ADR-0246 D-9 — Substrate Cohesion via PRD Amendment (Evans 2003 DDD; Nygard 2011 ADR)
- ADR-0252 D-1 — Hybrid Logical Clock (Demirbas + Kulkarni OPODIS 2014)
- ADR-0252 D-2 — Atomic-Clock-Backed External Consistency (Spanner OSDI 2012)
- ADR-0252 D-5 — Saga + Compensation, Not Lock (Kleppmann fencing token essay)
- ADR-0252 D-16 — Tiered Isolation with Policy Opt-In (Postgres SSI, Cahill 2008)

**Source-bucket count:** 8 decision-citations into academic + textbook
sources.

### 4.15 Other (community + open-source + niche references)

- ADR-0242 D-1.r — Reserved Identifier Namespace + IDN Homograph Defence (UTS#39 + UTR#36 Unicode)
- ADR-0244 D-1.r — Reserved Identifier Namespace + IDN Homograph Defence (UTS #39; UTR #36)
- ADR-0242 D-5 — Audited Bootstrap Replay (rustc stage0; kubeadm; Certificate Transparency)
- ADR-0244 D-3.c — Capability-Based Authorization (Linux capabilities(7))
- ADR-0244 D-10 — Signed Migration Ledger + Drain + Cutover (Cassandra token migration)
- ADR-0244 D-2 — Hierarchical Principal Path (Kubernetes namespace hierarchy)
- ADR-0244 D-9 — Per-PR Ephemeral Tenant (Vercel preview deployments; Heroku Review Apps; Render preview)
- ADR-0246 D-7 — Distributed Relational with Application-Aware Sharding (Citus design)
- ADR-0247 D-5 — Multi-Stage Self-Host Bootstrap (rustc stage0/1/2; LFS Chapter 5/6; kubeadm Phase 1/2)
- ADR-0247 D-6 — Three-Tier CD with Auto-Rollback (Spinnaker bake-to-prod pipeline)
- ADR-0247 D-9 — Lossless Substrate Distribution (Nix flakes preserve input provenance)
- ADR-0248 D-13 — Workload-In-Pod Default (Kubernetes everywhere)
- ADR-0248 D-14 — VM-Per-Workload Isolation (Kata Containers at Bytedance/Tencent/Microsoft)
- ADR-0248 D-15 — Distributed Edge POP (Fastly POPs)
- ADR-0249 D-1.1 — Universal Product Identifier (eBay Item ID; Walmart Marketplace Item ID; Etsy Listing ID)
- ADR-0249 D-1.2 — Per-Warehouse Stock State (Shopify Inventory API; ShipBob docs)
- ADR-0249 D-1.4 — Pluggable Carrier + 3PL Adapter Layer (ShipBob; ShipStation; Easypost)
- ADR-0249 D-1.5 — Multi-Surface Reputation System (eBay Feedback; Yelp Reviews)
- ADR-0249 D-1.6 — Three-Tier Search + Ranking Stack (Algolia engineering blog; Elasticsearch + ClickHouse)
- ADR-0249 D-1.7 — Pricing-Promotion-Tax Substrate (Shopify Pricing + Discounts APIs)
- ADR-0249 D-1.8 — Multi-Signal Trust Score (Sift Engineering blog; Airbnb Trust + Safety; Meta Marketplace fraud)
- ADR-0249 D-5 — Per-Category Seller Verification (Etsy Shop categories; Shopify Partner verification)
- ADR-0249 D-7 — Cold-Start with Graduated Limits (Airbnb New Host program; eBay 100-feedback restriction history)
- ADR-0249 D-11 — Marketplace Facilitator with Per-Jurisdiction Activation (Etsy state-by-state activation; Shopify Tax)
- ADR-0249 D-12 — Per-Category Policy Overlay (Shopify per-category requirements)
- ADR-0249 D-13 — Federated Search + OLAP Ranking (Elasticsearch + ClickHouse in Yelp/Airbnb)
- ADR-0249 D-14 — Per-Jurisdiction Moderation Overlay (DSA Article 16; KR-방심위)
- ADR-0249 D-15 — Workflow Saga with Compensating Action + Escrow (Upwork milestone escrow; eBay Money Back Guarantee)
- ADR-0251 D-9 — Shared De-Identification Substrate (Privitar, acquired by Informatica 2023)
- ADR-0251 D-10 — Hierarchical-Key-Management Substrate (HashiCorp Vault Transit)
- ADR-0251 D-11 — Per-Purpose Consent Substrate (OneTrust Consent Management; TrustArc; Cookiebot)
- ADR-0251 D-12 — Per-Regulation DPIA Template (ICO DPIA template; CNIL PIA tool; HHS HIPAA Risk Analysis)
- ADR-0252 D-1 — Hybrid Logical Clock (CockroachDB design doc 2015; MongoDB Atlas; YugabyteDB; TiDB Percolator+HLC)
- ADR-0252 D-7 — Linear Time Smear (Meta blog 2022; chronyd `leapsectz slew`)
- ADR-0252 D-8 — Opaque Self-Describing Key (Square Idempotency-Key; Twilio Idempotency-Key)
- ADR-0252 D-10 — HLC-Ordered Audit Chain (CockroachDB CDC ordering; Cassandra cluster timestamp)
- ADR-0252 D-15 — Uniform Clock Abstraction (CockroachDB hlc.Clock interface; MongoDB Cluster Time)
- ADR-0253 D-1.a — Anycast Apex DNS (Akamai 4000+ POPs)
- ADR-0253 D-2.a — Planetary Edge POP (Akamai 4000+ POPs)
- ADR-0253 D-6 — Layered L3/L4 + L7 mesh (Solo.io reference architecture)
- ADR-0253 D-7 — Workload identity primitive (SPIFFE/SPIRE CNCF Graduated 2022)
- ADR-0253 D-8 — Cellular Architecture Async (Pat Helland)
- ADR-0253 D-9 — Per-pair encrypted tunnel (IBM Satellite)
- ADR-0253 D-12 — Own ASN + RPKI (RIPE NCC ROA)
- ADR-0253 D-14 — Multi-protocol API surface (Netflix GraphQL Federation; Slack AsyncAPI)
- ADR-0253 D-15 — Realtime push tier (Slack WebSocket; Discord SSE; ChatGPT SSE for token stream; Zoom WebTransport)
- ADR-0253 D-16 — Webhook reliability triplet (Slack request signing; GitHub Ed25519)
- ADR-0253 D-18 — Phased self-hosting DNS (Akamai; Hurricane Electric)
- ADR-0254 D-1.4 — Connected Edge / Hybrid On-Prem (Anduril Lattice tactical edge)
- ADR-0254 D-1.5 — Air-Gapped Bundle Delivery (GitHub Enterprise Server with TUF; Anduril Lattice classified)
- ADR-0254 D-7 — Flagger Progressive Delivery (Flagger production deployments)
- ADR-0254 D-14 — Merkle-Sealed Bundled Audit Export (Bitcoin block reconciliation analogue)
- ADR-0254 D-15 — Vendor Reference Architecture (Dell PowerEdge / HPE ProLiant / Lenovo ThinkSystem)
- ADR-0255 D-8 — Embeddings Substrate Promotion (Pinecone-as-substrate; Milvus-as-substrate)

**Source-bucket count:** ~50 decision-citations into Other community /
open-source / niche sources.

---

## 5. Novel architectural primitives

A "novel" primitive is one for which no direct hyperscaler precedent
exists. The keystone bundle explicitly identifies the following 5
oyatie-novel patterns. Each is novel BUT defensible because it
composes existing hyperscaler primitives in a new way OR responds to a
constraint that hyperscalers do not face.

For F2 reviewers: novel-pattern decisions require extra scrutiny.
Verify that the why-novel-but-defensible justification holds, and that
no existing hyperscaler precedent was missed.

### 5.1 Foundry dissolution (ADR-0247 D-1, ADR-0255 D-16)

**The novelty.** Most platform companies that have a "Foundry"-style
internal CI / self-modification surface keep it as a peer µservice
forever (Palantir Foundry, Salesforce Setup, AWS Internal Tools).
Oyatie dissolves it.

**Why novel.** No hyperscaler has publicly dissolved their equivalent
of a Foundry µservice. The closest analogue is AWS Bedrock's absorption
of prior per-team provider adapters, but Bedrock is consumer-facing —
not the substrate of platform self-modification.

**Why defensible.** The dissolution follows ADR-0242's "oyatie is a
tenant" doctrine to its logical conclusion: if oyatie is a tenant,
then the internal-CI workflow surface should be a *workflow library*
running on Workflow Engine (a substrate) under Cedar gates (a
substrate), not a separate µservice. The dissolution reduces primitive
duplication and closes a class of bypass-path acquisition risks. The
substrate primitives it depends on (Workflow Engine + Cedar +
Audit Chain + Git-µservice) are all hyperscaler-precedented; the
composition is novel.

**F2 verification checklist for novel-claim:**

- Cite at least one hyperscaler precedent for *each substrate* the
  dissolved Foundry depends on. ✓ (AWS Bedrock + Step Functions + IAM
  cited in ADR-0247 D-1 row.)
- Demonstrate that the composition closes a measurable risk that the
  pre-dissolution shape exposed. ✓ (Primitive Duplication Across
  Sibling µservices anti-pattern.)
- Demonstrate that the alternative (keep Foundry) was rejected on
  load-bearing grounds. ✓ (ADR-0247 §Alternatives.)

### 5.2 Intelligence-as-substrate two-layer model (ADR-0255 D-1 through D-18)

**The novelty.** Most AI platforms either have a single tier (consumer-
only: Claude.ai, ChatGPT, Gemini) or split internal-vs-consumer at the
service boundary (Apple Intelligence partly, OpenAI Enterprise). Oyatie
unifies internal-platform-ops, B2B tenant, and B2C consumer calls
behind one substrate, with audience encoded as a Cedar context
attribute rather than a service boundary.

**Why novel.** AWS Bedrock is the closest hyperscaler precedent — it
serves AWS internal teams + AWS Console + AWS customers from one
substrate. But AWS does not expose Bedrock to consumers in a Brand
Surface layered model; their consumer surfaces (Amazon Q,
CodeWhisperer) are separate products with their own substrate plumbing.

Apple Intelligence is the closest consumer-product precedent (substrate
+ brand-surface separation), but Apple Intelligence is consumer-only
and does not serve internal Apple platform-ops or external B2B tenants
through the same substrate.

**Why defensible.** The substrate uniformity property (Calls 1 + 2 in
Appendix B traverse identical substrate code paths, distinguished only
by Cedar context attributes) is exactly the property ADR-0242 demands:
no bypass paths for platform-owner ops. Apple Intelligence's substrate
+ brand-surface separation is hyperscaler-precedented; the extension
of that to audience-neutral substrate is the novel composition.

**F2 verification checklist for novel-claim:**

- Identify the composed hyperscaler patterns. ✓ (AWS Bedrock for
  audience-neutral substrate + Apple Intelligence for brand-surface
  layering + Stripe provider-credential BYOK for credential primitives.)
- Demonstrate the substrate uniformity property holds with worked
  example. ✓ (ADR-0255 Appendix B Call 1 vs Call 2.)
- Verify that the alternative (separate substrates per audience) was
  rejected with load-bearing rationale. ✓ (ADR-0255 §Alternatives;
  retired ADR-0220 path.)

### 5.3 oyatie-as-tenant slug (ADR-0242 D-1, ADR-0244 D-1.r)

**The novelty.** Hyperscalers reserve namespaces (AWS `aws`, GCP
`google`, Microsoft "First-Party Tenant") for platform-owner ops, but
typically those namespaces are *outside* the multi-tenant audit-
compliance machinery — they're administrative carve-outs.

Oyatie's `oyatie` tenant slug is *inside* the multi-tenant machinery:
SOC 2 audits cover oyatie's use of oyatie; GDPR DSAR cascades cover
oyatie engineers; the audit-chain emits oyatie events on the same
streams as customer events. The reserved-namespace protection is
defensive (preventing typosquatting), but the slug itself is fully
participating, not carved out.

**Why novel.** No hyperscaler has publicly documented their platform-
owner account as a fully audited tenant in their compliance scope.
AWS, GCP, and Microsoft all have first-party tenants but explicitly
treat them as administrative scope, not customer scope. The closest
public precedent is Stripe's SOC 2 covering Stripe's internal use —
but Stripe's internal use is at a different layer (using Stripe to
pay Stripe contractors), not Stripe's internal CI as a Stripe tenant.

**Why defensible.** The doctrine closes a regulator-red-flag risk
(platform-owner outside audit scope). The reserved-namespace
protection is hyperscaler-precedented (AWS `arn:aws:iam::aws:`); the
extension to "and the reserved namespace IS audited as a tenant" is
the novel composition. The worked example in ADR-0242 Appendix B
(DSAR for EU-resident oyatie engineer) demonstrates that the uniform
machinery works.

**F2 verification checklist for novel-claim:**

- Identify the composed hyperscaler patterns. ✓ (AWS reserved
  namespace + Stripe dogfooded compliance + Salesforce multi-tenant
  substrate.)
- Demonstrate the regulator-red-flag risk being closed. ✓ (Compliance
  Carve-Out anti-pattern explicitly named.)
- Verify the worked example demonstrates uniformity. ✓ (ADR-0242
  Appendix B DSAR cascade.)

### 5.4 Compliance Pack abstraction (ADR-0251 D-1, D-2, D-3, D-14, D-15)

**The novelty.** Compliance frameworks (AWS Audit Manager, Microsoft
Purview, Google Assured Workloads) provide *catalogs* of regulations
mapped to controls, but they typically don't package the full
regulation as an installable "pack" with its own lifecycle (author →
sign → publish → activate → audit → sunset → tombstone) + its own
Cedar fragments + its own onboarding workflow + its own breach-
notification workflow + its own DPIA template + its own audit-stream
retention + its own per-pack composition semantics.

Oyatie's Compliance Pack treats a regulation as a fully self-
contained installable bundle. Tenant installs a pack via a
deterministic workflow; the pack carries everything needed for that
regulation; cross-pack composition follows declared semantics (deny
wins, retention MAX, cross-tenant MOST RESTRICTIVE).

**Why novel.** AWS Audit Manager comes closest with framework
catalogs + assessment reports, but it doesn't carry Cedar fragments
or onboarding workflows. Google Assured Workloads activates a regime
at the project level but doesn't expose per-pack lifecycle. Microsoft
Purview tenant assessments are template-based, not installable bundles.

**Why defensible.** The pack abstraction is composed from hyperscaler-
precedented primitives:

- Compliance-as-Packaged-Bundle (AWS Audit Manager catalogs, Google
  Assured Workloads regimes).
- Signed Policy Bundle Lifecycle (Sigstore Rekor + cosign).
- Compositional Policy Semantics (AWS SCP + IAM intersection).
- Hierarchical Certification Inheritance (FedRAMP High ⊇ Moderate).

The novel composition is the *unification* of all four into a single
installable abstraction with deterministic lifecycle. The worked
example (ADR-0251 Appendix B Acme Healthcare installing HIPAA +
SOC 2 T2) demonstrates that the abstraction is mechanically operable.

**F2 verification checklist for novel-claim:**

- Identify the composed hyperscaler patterns. ✓ (Four named above.)
- Demonstrate the mechanical-operability property. ✓ (ADR-0251
  Appendix B 10-step walkthrough.)
- Verify that the cross-pack composition semantics are deterministic.
  ✓ (D-14 names deny-wins + retention-MAX + cross-tenant-MOST-
  RESTRICTIVE.)

### 5.5 provider-credential BYOK plus encryption-key BYOK SecretReference boundaries (ADR-0255 D-4, with reach into ADR-0251 D-10)

**The novelty.** Hyperscalers offer encryption-key BYOK for *specific surfaces* —
AWS Bedrock customer-managed encryption keys, Azure Key Vault CMK,
GCP CMEK. But each of these is scoped to a single substrate (e.g.,
AWS Bedrock CMK protects Bedrock model invocations, not the broader
platform's secret material).

Oyatie's SecretReference model treats provider-credential BYOK (ADR-0255 D-4) and encryption-key BYOK (ADR-0251 D-10) as disjoint concern families while representing every secret
the substrate touches (LLM API keys, payment processor keys, KMS keys,
storage encryption keys, etc.) as a SecretReference with owner
declaration (`oyatie-subscription` / `tenant-byok` / `tenant-hyok`)
and Cedar-evaluated permission to use. The Cedar evaluation selects
the SecretReference at dispatch time based on call context.

**Why novel.** No hyperscaler offers a unified SecretReference abstraction across provider-credential BYOK and encryption-key BYOK
for every secret class — each substrate (Bedrock, KMS, etc.) has its own
provider-credential or encryption-key BYOK pattern with subtle differences. Oyatie unifies them under one
SecretReference primitive.

**Why defensible.** The primitives composed are hyperscaler-
precedented:

- provider-credential BYOK and encryption-key BYOK SecretReference (AWS Bedrock CMK + Azure Key Vault CMK + GCP
  CMEK + Stripe provider credentials + HashiCorp Vault Transit).
- Owner-Declared ToS Clearance (AWS Bedrock per-customer ToS +
  Anthropic AUP per-organization).
- Hierarchical-Key-Management Substrate (AWS KMS + CloudHSM
  hierarchy; GCP Cloud KMS + Cloud HSM + EKM; Azure Key Vault Managed
  HSM).

The novel composition is the *uniformity* of SecretReference across
every secret class, with Cedar-driven selection at call time. The
worked example (ADR-0255 Appendix B Call 2's provider-credential BYOK tenant selection)
demonstrates that the abstraction is mechanically operable.

**F2 verification checklist for novel-claim:**

- Identify the composed hyperscaler patterns. ✓ (Three named above.)
- Demonstrate the uniformity-across-secret-classes property. ✓
  (ADR-0255 §SecretReference + ADR-0251 D-10 encryption substrate.)
- Verify that the Cedar-driven selection prevents substrate-owned
  credentials. ✓ (D-4 Substrate-Owned Credentials anti-pattern.)

---

## 6. Citation-quality scorecard

Per-ADR scorecard. Columns:

- **Rows:** total decision rows in the per-ADR Appendix A.
- **Named-pattern coverage:** rows where the pattern has a defensible
  named-pattern column (target: 100%).
- **Multi-source citations:** rows that cite 2+ unrelated sources
  (target: ≥ 80%; multi-source is stronger than single-vendor).
- **Single-source rows:** rows where only one source backs the claim
  (a soft watch-list, not a fail).
- **Novel-pattern rows:** rows where the pattern is novel-but-
  defensible (justified in §5).
- **Weak rows:** rows where the citation is to an internal artifact
  (an ADR, an oyatie convention) rather than an external source. Some
  weak-row count is expected because keystone ADRs reference each
  other; but ≥ 50% weak rows would indicate the ADR is leaning too
  much on internal scaffolding.

| ADR | Rows | Named-pattern coverage | Multi-source | Single-source | Novel-pattern | Weak rows |
|---|---|---|---|---|---|---|
| ADR-0242 | 8 | 8/8 (100%) | 7/8 (88%) | 1/8 (D-7 cites 3 sources) | 0 | 0 |
| ADR-0243 | 13 | 13/13 (100%) | 11/13 (85%) | 2/13 (D-8 internal; D-12 single AWS) | 0 | 1 (D-8 internal doctrine) |
| ADR-0244 | 18 | 18/18 (100%) | 17/18 (94%) | 1/18 (D-4 OPA + Cedar; OK) | 0 | 0 |
| ADR-0245 | 11 | 11/11 (100%) | 11/11 (100%) | 0 | 0 | 0 |
| ADR-0246 | 11 | 11/11 (100%) | 10/11 (91%) | 1/11 (D-7 Citus + AWS Aurora + Spanner) | 0 | 1 (D-9 internal DDD + ADR) |
| ADR-0247 | 12 | 12/12 (100%) | 12/12 (100%) | 0 | 1 (D-1 Foundry dissolution) | 1 (D-10 internal glossary) |
| ADR-0248 | 17 | 17/17 (100%) | 15/17 (88%) | 2/17 (D-7 MacCárthaigh primary; D-9 Brooker primary) | 0 | 0 |
| ADR-0249 | 24 | 24/24 (100%) | 22/24 (92%) | 2/24 (D-3 Salesforce only; D-15 multi but eBay-led) | 0 | 0 |
| ADR-0250 | 12 | 12/12 (100%) | 12/12 (100%) | 0 | 0 | 1 (D-6 references ADR-0243 internal) |
| ADR-0251 | 16 | 16/16 (100%) | 15/16 (94%) | 1/16 (D-15 FedRAMP + CMMC + ISO — multi-cert but all gov-cert) | 1 (D-1 Compliance Pack abstraction; D-3/D-14/D-15 supporting) | 0 |
| ADR-0252 | 16 | 16/16 (100%) | 15/16 (94%) | 1/16 (D-13 Spanner + CockroachDB — multi but close family) | 0 | 0 |
| ADR-0253 | 24 | 24/24 (100%) | 22/24 (92%) | 2/24 (D-2.b Pingora only; D-12 own ASN + RPKI single industry pattern) | 0 | 1 (D-3.c references ADR-0223 internal) |
| ADR-0254 | 20 | 20/20 (100%) | 19/20 (95%) | 1/20 (D-10 ADR-0251 inheritance) | 0 | 2 (D-7 ADR-0040; D-10 ADR-0251; D-12 ADR-0253) |
| ADR-0255 | 18 | 18/18 (100%) | 17/18 (94%) | 1/18 (D-17 internal portfolio practice) | 2 (D-1 two-layer; D-4 provider-credential BYOK SecretReference) | 1 (D-17 internal practice) |

**Aggregate scorecard:**

- **Total rows:** 220 (counting sub-decisions split out for readability,
  ~218 unique decisions).
- **Named-pattern coverage:** 220/220 = 100%. Every decision has a
  named pattern.
- **Multi-source coverage:** 209/220 ≈ 95%. Strong cross-vendor
  validation.
- **Novel-pattern coverage:** 5 architectural primitives explicitly
  justified in §5.
- **Weak rows:** ~7/220 ≈ 3%. Acceptable; all weak rows reference
  other oyatie ADRs (internal-but-canonical), not "AGENTS.md said
  so" hand-waves.

**Watch-list:**

- ADR-0246 D-9 (Substrate Cohesion via PRD Amendment) cites DDD + ADR
  pattern only — should pick up a hyperscaler precedent at next
  multispectrum review.
- ADR-0247 D-10 (retired external agent harness name retirement) cites internal glossary
  discipline only — defensible (the term is internal); accept as-is.
- ADR-0255 D-17 (ADR-0220 fate) cites internal portfolio practice;
  defensible because it's an ADR-on-ADR meta decision.

---

## 7. Source freshness audit (2024-2026 emphasis)

Citation freshness matters because hyperscaler patterns evolve. A
citation to a 2008 paper is acceptable if the pattern is foundational
(e.g., Postgres SSI 2008 is still canonical); a citation to a 2008
blog post about service architecture is suspect because the field has
moved.

### 7.1 Citation count by year bucket

| Year bucket | Approximate count | Notable sources |
|---|---|---|
| 2024-2026 | ~90 | AWS Bedrock + Step Functions (re:Invent 2023+2024); AWS Verified Permissions (re:Invent 2023 BOA303 + 2024); AWS Cells 2024 (Stripe Cells 2024); AWS Aurora DSQL (re:Invent 2024); AWS Builders' Library 2024-Q4; Apple Intelligence WWDC 2024; Apple Pay 2024; AWS Outposts 2024 connected; Azure AI Foundry 2024; Cloudflare Pingora 2024; Cloudflare X25519MLKEM768 2024; GPT-4o 2024-05; Claude 3.5 Sonnet 2024-06; Gemini 1.5 Pro 2024-02; Meta marketplace 2024; NIST FIPS 203 2024; Google Cloud Deprecation Policy 2024; AWS Well-Architected v2024-Q4; Salesforce Trust Documentation 2024; Microsoft Cloud Adoption Framework 2024; Stripe Atlas + Treasury 2024; AWS 2024 ARC404 + ARC405; Snowflake BYOC 2024; Confluent BYOC 2024; Astronomer BYOC 2024; Databricks 2024; Palantir Forward 2023+2024; DSA Article 16 2024; KR-PIPA-2023-amendment; EU-AI-ACT-2024; HHS HIPAA-2024; etc. |
| 2021-2023 | ~40 | Sigstore Cosign (2021+); SLSA L3 (2021); Snowflake BYOC (2022); Confluent BYOC (2023); Apple Foundation Models (2024 wrap-up); Bryar/Carr 2021 "Working Backwards"; Privitar acquired by Informatica 2023; ADR-0148 (oyatie 2024); Databricks Customer-Managed VPC (2021); SPIFFE/SPIRE CNCF Graduated 2022; Cloudflare HTTP/3 since 2020; AWS CloudFront HTTP/3 since 2022; Meta leap-second blog 2022; Cilium 1.14+ identity policy 2023; Mozilla SSL Configuration Generator (current); etc. |
| 2018-2020 | ~25 | Brooker "Constant Work" 2020 AWS Builder's Library; Weiss/Furr "Static Stability" 2020 AWS Builder's Library; Bryan Liston cell-based architecture re:Invent 2018; AWS SDK retryable APIs 2018+; Klein "Cron at scale" 2018; Werner Vogels cell talk re:Invent 2018; Werner Vogels cells re:Invent 2019; etc. |
| 2014-2017 | ~25 | MacCárthaigh shuffle sharding 2014 AWS Architecture Blog; Brandur Leach Stripe Idempotency Key 2014; Demirbas + Kulkarni HLC OPODIS 2014; Verma et al. Borg/Omega 2016; Apple Pay 2014 launch playbook; Vogels "10 Lessons" 2016; etc. |
| 2008-2013 | ~10 | Google Smear blog 2008+2011; Postgres SSI Cahill 2008; Pinheiro et al. 2007; Spanner OSDI 2012; Stripe Engineering Quora 2013; etc. |
| Pre-2008 | ~5 | Cockburn Hexagonal Architecture 2005; Evans DDD 2003; Bezos 1997 ASIN history; Linux capabilities(7) (1999+); RFC 1035 (1987); RFC 5280 X.509 (2008 revision of older). |

### 7.2 Pre-2020 source review

For each pre-2020 source cited:

| Source | Year | Decision(s) citing it | Still canonical? | Action |
|---|---|---|---|---|
| RFC 1035 DNS | 1987 | ADR-0244 D-1 | Yes; foundational | No action. |
| Bezos 1997 ASIN | 1997 | ADR-0249 D-1 | Yes; pattern is canonical | No action; supplement with Bryar/Carr 2021 (already done). |
| Linux capabilities(7) | 1999+ | ADR-0244 D-3.c | Yes; foundational | No action. |
| Evans DDD 2003 | 2003 | ADR-0246 D-2, D-9 | Yes; foundational textbook | No action. |
| Cockburn Hexagonal Arch 2005 | 2005 | ADR-0246 D-3 | Yes; foundational pattern | No action; supplement with ADR-0105 internal. |
| Pinheiro Availability 2007 | 2007 | ADR-0245 D-8.c | Yes; canonical paper | No action. |
| RFC 5280 X.509 | 2008 | ADR-0243 D-5, ADR-0246 D-8 | Yes; foundational PKI | No action. |
| Postgres SSI Cahill 2008 | 2008 | ADR-0252 D-16 | Yes; canonical SSI paper | No action. |
| Google Smear blog 2008+2011 | 2008-2011 | ADR-0252 D-7 | Yes; pattern still in production | No action; supplement with Meta 2022 (already done). |
| Nygard ADR pattern 2011 | 2011 | ADR-0246 D-9 | Yes; canonical ADR pattern | No action. |
| Spanner OSDI 2012 | 2012 | ADR-0252 D-2, D-13 | Yes; canonical paper | No action. |
| Vogels "10 Lessons" 2016 | 2016 | ADR-0242 D-1 | Yes; canonical reference | No action. |
| Stripe Engineering Quora 2013 | 2013 | ADR-0242 D-1 | Source URL still resolves; pattern unchanged | Watch; re-cite from a more current Stripe Engineering blog if available. |

**Verdict:** Pre-2020 sources are predominantly foundational (RFCs,
canonical papers, foundational textbooks). No source is outdated in
the sense of describing a pattern no longer in production. The
Stripe Engineering Quora 2013 citation is the only watch-item; should
be supplemented at next multispectrum review.

### 7.3 Source-density invariants to maintain

Going forward:

- **2024-2026 source share should remain ≥ 30%** of total citations.
  Currently ≈ 40% (90/220) — healthy.
- **Single-vendor citation share should remain ≤ 10%** of total
  citations. Currently ≈ 5% (11/220) — healthy.
- **Pre-2010 source share should remain ≤ 10%** of total citations.
  Currently ≈ 4% (8-10 / 220) — healthy.

---

## 8. Maintenance protocol

When a new ADR (≥0256) lands with its own Appendix A:

### 8.1 Mandatory steps

1. **Author the per-ADR Appendix A** in the new ADR with one row per
   decision (per ADR-0242 inherited pattern).
2. **Add a new §3.X subsection** to this document for the new ADR with
   the same row format. Cross-reference the source ADR.
3. **Update §4 grouped views** for any new sources cited. If the new
   ADR cites, e.g., Datadog Engineering for the first time, add a
   new §4.16 subsection.
4. **Update §5 novel-pattern section** if the new ADR claims a novel
   pattern. Provide the F2 verification checklist (4-bullet
   defensibility justification).
5. **Update §6 citation-quality scorecard** with the new ADR's row
   counts.
6. **Update §7 source freshness audit** with any new pre-2020 source
   citations.
7. **Increment the row count in §3** preamble.
8. **Update the "Last verified" date** at the top of this document.

### 8.2 CI verification

A CI lane (`check-hyperscaler-pattern-matrix-coherence`) verifies:

- Every keystone ADR (≥0242 and any ADR with an `## Appendix A:
  Hyperscaler-pattern attribution matrix` header) has a corresponding
  §3.X subsection in this document.
- The decision counts in §3 match the decision counts in the source
  ADRs.
- Every decision ID in §3 resolves to a real decision section in the
  source ADR.
- Every citation in §3 also appears in §4's appropriate source bucket.
- Every novel-pattern claim in any ADR's Appendix A appears in §5.

The CI lane is BLOCKER-class: merge is refused if this document drifts
from the per-ADR Appendix A tables.

### 8.3 Periodic audit (quarterly)

Quarterly (aligned with multispectrum review v2.4.0 cycle):

- **Source freshness sweep:** verify every cited URL still resolves;
  open replacement-citation PRs for dead links.
- **Source-density audit:** confirm the invariants in §7.3 still hold.
- **Novel-pattern review:** for each pattern in §5, verify it remains
  novel — if a new hyperscaler precedent has emerged, demote the
  novelty claim and document the precedent.
- **Watch-list resolution:** address any items on the §6 watch-list.

### 8.4 ADR retirement / replacement

When a keystone ADR is superseded:

- Mark the §3.X subsection with `**SUPERSEDED BY ADR-XXXX —**` at the
  top of the subsection.
- Keep the rows for historical reference.
- Add the successor ADR's §3.X subsection.
- If the superseded ADR had novel-pattern claims, mark them in §5 as
  `**SUPERSEDED**` and update the active claims.

### 8.5 Disagreement resolution

If this document disagrees with a per-ADR Appendix A, **the per-ADR
Appendix A wins** and this document is the bug. Open a fix PR against
this document. Do NOT silently update the per-ADR Appendix A from this
document.

### 8.6 Ownership

- **Document owner:** council-architecture.
- **Reviewer for F2 facet:** assigned per-PR per multispectrum review
  v2.4.0 doctrine.
- **CI lane owner:** council-engineering.
- **Source-freshness sweep owner:** council-architecture + council-
  security (security-and-hardening skill).

---

## Appendix I — Canonical source dossier

This appendix is the per-source dossier referenced by §4. For each
source bucket, it lists the canonical entry points reviewers should
consult when verifying a citation from §3. The dossier is intentionally
narrow (canonical entry points only) so the F2 reviewer doesn't have
to hunt for the right URL or paper.

### I.1 AWS Builder's Library + Well-Architected

- **AWS Builder's Library — Static stability** (Weiss + Furr, 2020):
  the canonical entry point for static-stability decisions. Cited by
  ADR-0243 D-11, ADR-0245 D-4.B, ADR-0246 D-6, ADR-0247 D-12,
  ADR-0248 D-8.
- **AWS Builder's Library — Constant work** (Brooker, 2020): the
  canonical entry point for constant-work + Route-53-style health
  propagation. Cited by ADR-0248 D-9.
- **AWS Builder's Library — Workload isolation using shuffle
  sharding** (MacCárthaigh, 2014; reaffirmed 2017+2024): the canonical
  entry point for shuffle-sharding decisions. Cited by ADR-0248 D-7,
  ADR-0254 D-1.1.
- **AWS Builder's Library — Ten things we wish we'd known sooner**:
  cited by ADR-0250 D-2.
- **AWS Builder's Library — Request budgets**: the canonical entry
  point for policy-enforced deadlines. Cited by ADR-0252 D-11.
- **AWS Well-Architected v2024-Q4 Pillar 4 (Reliability)** + Pillar
  5 (Security): cited by ADR-0245 D-1, ADR-0245 D-8, ADR-0245 D-8.c,
  ADR-0248 D-1.
- **AWS Verified Permissions (re:Invent 2023 BOA303)** + AWS
  Verified Permissions docs 2024: the canonical entry point for
  Cedar-as-a-substrate. Cited by ADR-0243 D-1, D-2, D-6, D-12, D-13;
  ADR-0246 D-1, D-6; ADR-0247 D-3, D-8; ADR-0250 D-6, D-11;
  ADR-0251 D-2, D-6; ADR-0252 D-14.
- **AWS Step Functions** + AWS Step Functions Saga blueprint: cited
  by ADR-0249 D-1.3, D-10; ADR-0252 D-9, D-12; ADR-0254 D-13.
- **AWS Bedrock** (model invocation API; Knowledge Bases; Agents;
  Guardrails): cited extensively in ADR-0255 D-1 through D-18.
- **AWS Aurora DSQL re:Invent 2024**: cited by ADR-0252 D-2.
- **AWS Cells re:Invent 2018 (Bryan Liston)** + Stripe Cells 2024
  + re:Invent 2024 ARC404 + ARC405: cited by ADR-0246 D-5,
  ADR-0248 D-3 through D-12, ADR-0253 D-8, ADR-0254 D-3.
- **AWS Firecracker**: cited by ADR-0248 D-14, D-14.alt.
- **AWS Outposts** (dedicated + connected variants): cited by
  ADR-0248 D-12, ADR-0254 D-1.2, D-1.4, D-15.
- **AWS Route 53** (Anycast; GeoDNS; latency-based routing): cited
  by ADR-0248 D-3, D-11; ADR-0253 D-1.a, D-1.b, D-13.
- **AWS KMS + CloudHSM** (key hierarchy): cited by ADR-0243 D-5,
  ADR-0246 D-8, ADR-0251 D-10.
- **AWS Audit Manager** (framework catalog; evidence packs;
  assessment reports): cited by ADR-0242 D-4, ADR-0250 D-11,
  ADR-0251 D-1, D-16.
- **AWS Organizations / Control Tower** (account hierarchy; SCP;
  Compliance Tower): cited by ADR-0244 D-3, D-7, D-7.h;
  ADR-0250 D-9, ADR-0251 D-3, D-5; ADR-0250 D-5.

### I.2 Stripe Engineering

- **Stripe Engineering — Idempotency Keys** (Brandur Leach, 2014):
  the canonical entry point for caller-supplied idempotency. Cited
  by ADR-0252 D-4, D-8.
- **Stripe API docs**: cited by ADR-0244 D-1, D-3, D-3.c, D-5;
  ADR-0246 D-4; ADR-0249 D-1.7; ADR-0252 D-4; ADR-0253 D-3.a, D-14,
  D-15, D-16.
- **Stripe Connect** (platform-on-behalf-of; account capabilities;
  diverse partners): cited by ADR-0244 D-3.c, D-6, D-6.3; ADR-0249
  D-1, D-2; ADR-0251 D-7.
- **Stripe Engineering blog 2014-2020**: cited by ADR-0250 D-2.
- **Stripe Atlas + Treasury + phased launches**: cited by
  ADR-0249 D-9; ADR-0250 D-3, D-8.
- **Stripe Radar**: cited by ADR-0249 D-1.8, D-7.
- **Stripe Cells 2024**: cited by ADR-0248 D-3, D-4, D-10, D-12.

### I.3 Google SRE Workbook + Google Cloud + Google research

- **Google SRE Workbook ch. 2 (SLO composition)**: cited by ADR-0245
  D-8, D-8.c; ADR-0246 D-10.
- **Google SRE Workbook ch. 4 (SLO coverage)**: cited by ADR-0243
  D-9, ADR-0245 D-7, ADR-0246 D-11, ADR-0247 D-11.
- **Google SRE Workbook ch. 18 (Production Readiness Review)**:
  cited by ADR-0250 D-7.
- **Google SRE Workbook ch. 24 (Distributed Periodic Scheduling)**:
  cited by ADR-0252 D-5, D-6.
- **Google CRE Book ch. 8**: cited by ADR-0248 D-1.
- **Google Cloud Deprecation Policy 2024**: cited by ADR-0245 D-1,
  D-9.
- **Google Cloud Spanner OSDI 2012 (TrueTime + external
  consistency)**: cited by ADR-0252 D-2, D-13.
- **Google ALTS (workload identity)**: cited by ADR-0253 D-7.
- **Google Cloud KMS + Cloud HSM + EKM**: cited by ADR-0251 D-10.
- **Google Cloud DLP API**: cited by ADR-0251 D-9.
- **Google Cloud Assured Workloads**: cited by ADR-0251 D-3, D-4,
  D-5, D-16.
- **Google Borg/Omega (Verma et al. 2016)**: cited by ADR-0245
  D-4.B.
- **Google leap-second smear blog 2008+2011**: cited by ADR-0252
  D-7.
- **Google Anthos cross-cloud**: cited by ADR-0253 D-9, ADR-0254
  D-1.4.
- **Google ASN 15169 + RIPE NCC ROA**: cited by ADR-0253 D-12.

### I.4 Apple WWDC + Platform Architecture

- **Apple Intelligence WWDC 2024 keynote**: cited by ADR-0255 D-1,
  D-2, D-3, D-5, D-11, D-15.
- **Apple Platform Architecture 2024** + Apple Framework Index +
  Apple Frameworks Reference: cited by ADR-0245 D-1 through D-9.
- **Apple Pay 2014-2024 country launch progression** (the canonical
  reference for per-market phased launches): cited by ADR-0249 D-9;
  ADR-0250 D-1, D-2, D-3, D-8.
- **Apple App Store category-specific review (Health/Medical)**:
  cited by ADR-0249 D-12, ADR-0250 D-5.
- **Apple One subscription bundle**: cited by ADR-0249 D-1.
- **Apple Foundation Models API**: cited by ADR-0255 D-2.

### I.5 Cloudflare Engineering

- **Cloudflare Workers + WAF + Bot Management**: cited by ADR-0243
  D-6; ADR-0246 D-4, D-6; ADR-0253 D-2.a.
- **Cloudflare DNS Anycast 300+ POPs**: cited by ADR-0253 D-1.a.
- **Cloudflare 1.1.1.1 DoH**: cited by ADR-0253 D-1.c.
- **Cloudflare DNSSEC**: cited by ADR-0253 D-1.b.
- **Cloudflare Pingora open-source 2024**: cited by ADR-0253 D-2.b,
  D-17.
- **Cloudflare X25519MLKEM768 2024**: cited by ADR-0253 D-4.
- **Cloudflare HTTP/3 since 2020**: cited by ADR-0253 D-5.
- **Cloudflare GeoSteering**: cited by ADR-0253 D-13.
- **Cloudflare ASN 13335 + RPKI**: cited by ADR-0253 D-12.
- **Cloudflare zones TLS 1.3 by default**: cited by ADR-0253 D-3.a.
- **Cloudflare edge ~300 POPs**: cited by ADR-0248 D-15.

### I.6 Palantir

- **Palantir Apollo product page**: cited by ADR-0242 D-1;
  ADR-0254 D-1.4, D-1.5, D-2, D-4, D-14.
- **Palantir Forward 2023 + 2024 keynotes**: cited by ADR-0254 D-4.
- **Palantir Mission Support + Mission Specialist**: cited by
  ADR-0254 D-8, D-16.

### I.7 Microsoft Azure + Purview + 365

- **Azure AI Foundry**: cited by ADR-0247 D-1; ADR-0255 D-1, D-2,
  D-5, D-7, D-9, D-10, D-11, D-15, D-16.
- **Microsoft Purview** (assessment templates; Compliance Manager
  scorecard): cited by ADR-0251 D-1, D-3, D-16.
- **Microsoft Sentinel + Compliance Manager**: cited by ADR-0251
  D-8, D-13.
- **Azure Key Vault** (Managed HSM; CMK): cited by ADR-0251 D-10;
  ADR-0255 D-4.
- **Azure RBAC + AAD B2B + AAD tenant type**: cited by ADR-0244
  D-2, D-2.d, D-6, D-7, D-11; ADR-0251 D-6.
- **Azure Cosmos DB 5-level consistency**: cited by ADR-0252 D-3,
  D-16.
- **Azure Site Recovery**: cited by ADR-0244 D-3.dr.
- **Microsoft Cloud Adoption Framework 2024**: cited by ADR-0245 D-1.
- **Microsoft Azure Well-Architected**: cited by ADR-0245 D-8.
- **Microsoft 365 multi-tenant Exchange Online**: cited by ADR-0242
  D-3, D-4, D-7.
- **Microsoft Trust Center**: cited by ADR-0250 D-4.
- **Microsoft Operations Manual**: cited by ADR-0250 D-7.
- **Azure Arc**: cited by ADR-0254 D-1.4.
- **AKS Envoy ingress**: cited by ADR-0253 D-3.b.
- **Citus (Microsoft acquired 2019)**: cited by ADR-0246 D-7.

### I.8 OPA + Cedar + Sigstore + formal policy

- **Cedar policy language docs** (AWS Verified Permissions Cedar
  Reference): cited by ADR-0243 D-4, ADR-0244 D-4, ADR-0251 D-14.
- **OPA policies**: cited by ADR-0243 D-1 (Netflix OPA-at-scale);
  ADR-0244 D-4.
- **Sigstore + cosign + Rekor**: cited by ADR-0243 D-2, D-5;
  ADR-0246 D-8; ADR-0247 D-8; ADR-0250 D-11; ADR-0251 D-2;
  ADR-0254 D-5.
- **DDD (Evans 2003)**: cited by ADR-0246 D-2, D-9.
- **ADR pattern (Nygard 2011)**: cited by ADR-0246 D-9.
- **Hexagonal Architecture (Cockburn 2005)**: cited by ADR-0246 D-3.

### I.9 IETF + RFCs + Web Standards

- **RFC 1035 (DNS)**: cited by ADR-0244 D-1.
- **RFC 4034/4035 (DNSSEC)**: cited by ADR-0253 D-1.b.
- **RFC 5280 (X.509)**: cited by ADR-0243 D-5, ADR-0246 D-8.
- **RFC 7858 (DoT)**: cited by ADR-0253 D-1.c.
- **RFC 8484 (DoH)**: cited by ADR-0253 D-1.c.
- **IETF draft `idempotency-key-header-09`**: cited by ADR-0252 D-4.
- **NIST FIPS 203 (ML-KEM)**: cited by ADR-0253 D-4.
- **UTS #39 + UTR #36 (Unicode Security)**: cited by ADR-0242
  D-1.r, ADR-0244 D-1.r.
- **Mozilla SSL Configuration Generator** (modern profile): cited
  by ADR-0253 D-3.a.

### I.10 NIST + FedRAMP + DoD

- **NIST SP 800-92 (Audit log standards)**: cited by ADR-0243 D-7.
- **NIST SP 800-162 (ABAC)**: cited by ADR-0243 D-3.
- **NIST SP 800-207 (Zero Trust Architecture)**: cited by ADR-0243
  D-11, ADR-0250 D-9, ADR-0252 D-11, ADR-0253 D-10.
- **NIST SP 800-34 (Contingency Planning)**: cited by ADR-0247 D-12.
- **NIST SP 800-53 Rev 5** (FedRAMP baseline): cited indirectly
  via ADR-0251 D-15 + ADR-0254 D-1.5 scenarios.
- **FedRAMP High ⊇ Moderate ⊇ Low**: cited by ADR-0251 D-15.
- **DoD SRG IL5 / IL6**: cited by ADR-0254 D-1.5, D-6.
- **NSA RTB guidance + NCDSMO approved CDS list**: cited by
  ADR-0254 D-6.
- **CMMC L3 ⊇ L2 ⊇ L1**: cited by ADR-0251 D-15.
- **ISO 27001 / ISO 27002 / ISO 22301 / ISO 42001**: cited by
  ADR-0251 D-15 + per-pack mappings in ADR-0251 Appendix E.

### I.11 Salesforce + Snowflake + Databricks + Confluent

- **Salesforce multi-tenant architecture**: cited by ADR-0242 D-3.
- **Salesforce Pods**: cited by ADR-0248 D-4.
- **Salesforce AppExchange**: cited by ADR-0245 D-5; ADR-0248
  D-5; ADR-0249 D-3.
- **Salesforce HealthCloud / GovCloud SKUs**: cited by ADR-0251
  D-1.
- **Salesforce Premier vs Standard support**: cited by ADR-0254
  D-8.
- **Salesforce Trust Documentation 2024 + Trust + Compliance**:
  cited by ADR-0245 D-1; ADR-0250 D-4.
- **Salesforce Onboarding flows**: cited by ADR-0254 D-16.
- **Salesforce End of Life roadmap**: cited by ADR-0250 D-12.
- **Salesforce Einstein**: cited by ADR-0255 D-3.
- **Snowflake BYOC (2022)**: cited by ADR-0254 D-1.3.
- **Snowflake Virtual Private Snowflake**: cited by ADR-0254 D-1.2.
- **Snowflake "build Snowflake on Snowflake" blog (2022)**: cited
  by ADR-0254 D-2.
- **Snowflake credit consumption pricing**: cited by ADR-0254 D-9.
- **Confluent BYOC (2023)**: cited by ADR-0254 D-1.3.
- **Confluent Platform vs Cloud single-codebase**: cited by ADR-0254
  D-2.
- **Confluent Cluster Linking migration**: cited by ADR-0254 D-13.
- **Databricks Customer-Managed VPC (2021)**: cited by ADR-0254
  D-1.3, D-11.
- **Databricks Compliance Security Profile**: cited by ADR-0251
  D-1.
- **Astronomer BYOC (2024)**: cited by ADR-0254 D-1.3.

### I.12 Model-vendor docs (Anthropic + OpenAI + Google Gemini)

- **Anthropic Console + Claude 3.5 Sonnet 2024-06**: cited by
  ADR-0247 D-3, ADR-0255 D-5.
- **Anthropic streaming API + message batches**: cited by ADR-0255
  D-6, D-13.
- **Anthropic prompt-caching with caller-built context**: cited by
  ADR-0255 D-7.
- **Anthropic MCP architecture (server + host separation)**: cited
  by ADR-0255 D-12.
- **Anthropic AUP per-organization acceptance**: cited by ADR-0255
  D-18.
- **OpenAI Assistants v2 (opt-in stateful)**: cited by ADR-0255
  D-14.
- **OpenAI streaming API + fine-tune API**: cited by ADR-0255 D-9,
  D-13.
- **OpenAI per-account ToS**: cited by ADR-0255 D-18.
- **GPT-4o multi-modal 2024-05**: cited by ADR-0255 D-5.
- **Gemini 1.5 Pro 2024-02 multi-modal**: cited by ADR-0255 D-5.

### I.13 Temporal + Cadence + Workflow engines

- **Temporal workflow versioning + replay model + per-cluster
  idempotency + cross-cluster workflow**: cited by ADR-0247 D-7;
  ADR-0249 D-1.3, D-10; ADR-0252 D-5, D-9, D-12.
- **Cadence replay**: cited by ADR-0252 D-12.

### I.14 Distributed systems academia + textbooks

- **Cahill 2008 (Postgres SSI)**: cited by ADR-0252 D-16.
- **Cockburn 2005 (Hexagonal Architecture)**: cited by ADR-0246
  D-3.
- **Demirbas + Kulkarni OPODIS 2014 (HLC)**: cited by ADR-0252
  D-1.
- **Evans 2003 (Domain-Driven Design)**: cited by ADR-0246 D-2,
  D-9.
- **Kleppmann fencing token essay**: cited by ADR-0252 D-5.
- **Nygard 2011 (ADR pattern)**: cited by ADR-0246 D-9.
- **Pinheiro et al. 2007 (Markov-Chain Availability)**: cited by
  ADR-0245 D-8.c.
- **Spanner OSDI 2012 (Corbett et al.)**: cited by ADR-0252 D-2,
  D-13.
- **Verma et al. 2016 (Borg/Omega/Kubernetes lineage)**: cited by
  ADR-0245 D-4.B.

### I.15 Other industry references

- **CockroachDB design doc 2015 (HLC + SERIALIZABLE)**: cited by
  ADR-0252 D-1, D-3, D-10, D-13, D-15, D-16.
- **MongoDB Atlas Causal Consistency + Cluster Time**: cited by
  ADR-0252 D-1, D-3, D-15.
- **YugabyteDB + TiDB Percolator+HLC**: cited by ADR-0252 D-1.
- **Cassandra cluster timestamp resolution post-HLC**: cited by
  ADR-0252 D-10.
- **Vercel preview deployments**: cited by ADR-0242 D-8, ADR-0244
  D-9, ADR-0250 D-10.
- **Heroku Review Apps**: cited by ADR-0242 D-8, ADR-0244 D-9,
  ADR-0250 D-10.
- **Render preview environments**: cited by ADR-0244 D-9.
- **Spinnaker bake-to-prod pipeline**: cited by ADR-0247 D-6.
- **Nix flakes provenance**: cited by ADR-0247 D-9.
- **rustc stage0/1/2**: cited by ADR-0242 D-5, ADR-0247 D-4, D-5,
  D-9; ADR-0248 D-2.
- **Linux From Scratch (LFS) Chapter 5/6 cross-compile**: cited by
  ADR-0247 D-5.
- **Kubernetes kubeadm certificate chain**: cited by ADR-0242 D-5,
  ADR-0247 D-4, D-5; ADR-0248 D-2.
- **Certificate Transparency log bootstrap**: cited by ADR-0242
  D-5, ADR-0247 D-4, ADR-0248 D-2.
- **HashiCorp Vault Transit**: cited by ADR-0255 D-4, ADR-0251
  D-10.
- **OneTrust + TrustArc + Cookiebot (Usercentrics)**: cited by
  ADR-0251 D-11.
- **ICO DPIA template (UK) + CNIL PIA tool (France) + HHS HIPAA
  Risk Analysis tool + EU AI Act FRIA template**: cited by
  ADR-0251 D-12.
- **DSA Article 16** (Meta/Google/Amazon EU implementation): cited
  by ADR-0249 D-14.
- **KR-방심위 compliance pattern**: cited by ADR-0249 D-14.
- **Amazon A-to-Z Guarantee + Escrow + Upwork milestone escrow +
  eBay Money Back Guarantee**: cited by ADR-0249 D-15.
- **Etsy Shop categories + Etsy state-by-state activation**: cited
  by ADR-0249 D-5, D-11.
- **Shopify Inventory API + Pricing + Locations API + Shipping
  Apps + Tax + Partner verification + per-category requirements**:
  cited by ADR-0249 D-1.2, D-1.4, D-1.7, D-5, D-6, D-11, D-12.
- **ShipBob + ShipStation + Easypost adapter patterns**: cited by
  ADR-0249 D-1.4.
- **Algolia engineering blog + Elasticsearch + ClickHouse hybrid
  in Yelp/Airbnb**: cited by ADR-0249 D-1.6, D-13.
- **Sift Engineering blog + Airbnb Trust + Safety + Meta
  Marketplace fraud talks**: cited by ADR-0249 D-1.8, D-7.
- **eBay Item ID + Walmart Marketplace Item ID + Etsy Listing ID**:
  cited by ADR-0249 D-1.1.
- **Bryar/Carr 2021 "Working Backwards" (ASIN history)**: cited by
  ADR-0249 D-1.
- **Privitar (acquired by Informatica 2023)**: cited by ADR-0251
  D-9.
- **SPIFFE/SPIRE CNCF Graduated 2022**: cited by ADR-0253 D-7.
- **Cilium 1.14+ identity policy**: cited by ADR-0253 D-10.
- **GKE Dataplane V2 + Solo.io reference architecture + Istio
  Ambient**: cited by ADR-0253 D-6.
- **Pat Helland (cells)**: cited by ADR-0253 D-8.
- **IBM Satellite cross-cloud**: cited by ADR-0253 D-9.
- **Netflix GraphQL Federation**: cited by ADR-0253 D-14.
- **Slack request signing + Slack WebSocket + Slack AsyncAPI**:
  cited by ADR-0253 D-14, D-15, D-16.
- **Discord SSE + ChatGPT SSE for token stream + Zoom
  WebTransport**: cited by ADR-0253 D-15.
- **GitHub Ed25519 webhook signing + GitHub Enterprise Server with
  TUF**: cited by ADR-0253 D-16, ADR-0254 D-1.5.
- **Akamai 4000+ POPs + Akamai EdgeDNS + Hurricane Electric DNS**:
  cited by ADR-0248 D-11, ADR-0253 D-1.a, D-2.a, D-18.
- **Fastly POPs**: cited by ADR-0248 D-15.
- **Anduril Lattice tactical edge + classified**: cited by
  ADR-0254 D-1.4, D-1.5.
- **Flagger production deployments**: cited by ADR-0254 D-7.
- **Pinecone-as-substrate + Milvus-as-substrate**: cited by
  ADR-0255 D-8.
- **The Update Framework (TUF) spec + SLSA L3**: cited by ADR-0254
  D-5.
- **Dell PowerEdge / HPE ProLiant / Lenovo ThinkSystem reference
  architectures**: cited by ADR-0254 D-15.
- **Bitcoin block reconciliation analogue**: cited by ADR-0254
  D-14.
- **Square Idempotency-Key + Twilio Idempotency-Key**: cited by
  ADR-0252 D-8.
- **Let's Encrypt CA hierarchy + ACME automation**: cited by
  ADR-0246 D-8, ADR-0253 D-3.c.
- **etcd watch pattern + Kubernetes ConfigMap watch + Apollo /
  Argo CD sync**: cited by ADR-0243 D-10.
- **chronyd `leapsectz slew`**: cited by ADR-0252 D-7.
- **Meta leap-second blog 2022 + Meta Marketplace fraud talks**:
  cited by ADR-0252 D-7, ADR-0249 D-1.8.
- **DocuSign + Adobe Sign integration**: cited by ADR-0251 D-7.
- **Atlassian + PagerDuty + ServiceNow incident response**: cited
  by ADR-0251 D-8.
- **Sedona Conference legal-hold supersession**: cited by ADR-0250
  D-11.
- **Coalfire QSA + Deloitte auditor + Persona + Plaid + World-Check
  + Surety One**: cited via ADR-0250 Appendix B scenario.

---

## Appendix II — Anti-pattern reverse index

This appendix lists the canonical anti-patterns named across the
keystone bundle and which decision rows close them. F2 reviewers
worried about a specific failure mode (e.g., "cross-cell sync hot
path") can search this appendix to find which decision defends against
it and which hyperscaler-named pattern is the defense.

Anti-patterns are grouped by failure class.

### II.1 Multi-tenant / scoping anti-patterns

- **Internal Carve-Out** (bypass paths for platform-owner ops) —
  ADR-0242 D-1. Defense: Eat-Your-Own-Dogfood at Platform Level.
- **Typosquatting Tenant Impersonation** — ADR-0242 D-1.r, ADR-0244
  D-1.r. Defense: Reserved Identifier Namespace + IDN Homograph
  Defence.
- **Flat Namespace Drift** — ADR-0242 D-2, ADR-0244 D-2. Defense:
  Hierarchical Principal Path.
- **Unbounded Tree Depth** — ADR-0244 D-2.d. Defense: Bounded-Depth
  Hierarchy.
- **Audience-As-Service-Scope** — ADR-0242 D-3, ADR-0255 D-1,
  D-15. Defense: Unified Multi-Tenant Substrate + Audience-As-Call-
  Tag.
- **Audience-As-Service-Boundary** — ADR-0255 D-15. Defense:
  Audience-As-Call-Tag (deprecates ADR-0220 alternative).
- **Per-µservice Tenant View Drift** — ADR-0244 D-3. Defense:
  Single Source of Truth Tenant Registry.
- **Auto-Incrementing Integer Tenant ID** — ADR-0244 D-1. Defense:
  Globally Unique Slug + DNS-Compatible Segments.
- **Role-Based-Only** — ADR-0244 D-3.c. Defense: Capability-Based
  Authorization.
- **One-Size-Fits-All DR** — ADR-0244 D-3.dr. Defense: Tier-Aware
  DR Strategy.
- **Callee-Side Audience Declaration** — ADR-0244 D-5. Defense:
  Caller-Side Attribute Resolution.
- **Permanent Cross-Tenant Trust** — ADR-0244 D-6. Defense: Time-
  Bounded Cross-Tenant Grant.
- **Direct Customer Credential Sharing** — ADR-0244 D-6.3. Defense:
  Platform-on-Behalf-Of Pattern.
- **Hard-Delete-Only Lifecycle** — ADR-0244 D-7. Defense: Multi-
  State Tenant Lifecycle with Soft-Delete Window.
- **Total Erasure Including Audit** — ADR-0244 D-7.h. Defense:
  Cascade-Plus-Tombstone Deletion.
- **Shared Development Tenant** — ADR-0244 D-8, ADR-0242 D-8.
  Defense: Per-Engineer Sandbox Tenant.
- **Manual Pre-Production Promotion** — ADR-0244 D-9. Defense:
  Per-PR Ephemeral Tenant.
- **Free-Form Audience Tags** — ADR-0244 D-11. Defense: Closed-
  Enum Tenant Classification.
- **Production-Only Testing** — ADR-0242 D-8, ADR-0250 D-10.
  Defense: Ephemeral Tenant + Sandbox-Tenant Pilot.

### II.2 Policy + authorization anti-patterns

- **Multiple Policy Engines Drift** — ADR-0243 D-1, ADR-0246 D-1.
  Defense: Single Policy Engine Consolidation / Centralized Policy
  Service.
- **Imperative Policy Patching** — ADR-0243 D-2, ADR-0251 D-2.
  Defense: Signed Policy Authoring Lifecycle.
- **Implicit Permit** — ADR-0243 D-3. Defense: Coverage-Required
  Authorization.
- **Per-Tenant Code Branch** — ADR-0243 D-4. Defense: Layered
  Policy Composition.
- **Implicit Bootstrap Trust** — ADR-0243 D-5, ADR-0246 D-8.
  Defense: PKI Root + Intermediate Certificate Chain.
- **Synchronous Round-Trip to Global Policy Store** — ADR-0243
  D-6, ADR-0246 D-6. Defense: Edge-Cached Policy Evaluation /
  Static Stability + Edge-Cached Evaluation.
- **Audit Sampling** — ADR-0243 D-7. Defense: Audit-Every-Decision.
- **Single-Reviewer Policy Change** — ADR-0243 D-8. Defense: Multi-
  Facet Policy Review.
- **Untested Policy Surface** — ADR-0243 D-9. Defense: Coverage-
  Enforced Policy.
- **Restart-To-Apply** — ADR-0243 D-10. Defense: Hot-Reload
  Configuration Distribution.
- **Fail-Open on Policy Unavailable** — ADR-0243 D-11. Defense:
  Static Stability + Fail-Closed.
- **Tenant Privilege Escalation** — ADR-0243 D-12. Defense:
  Restricted Tenant Self-Policy.
- **Separate Feature-Flag System** — ADR-0243 D-13, ADR-0252 D-14.
  Defense: Unified Policy + Feature Gate / Policy-as-Feature-Flag.
- **Untyped String Match Policy** — ADR-0244 D-4. Defense: Typed
  Entity Policy Schema.
- **Application-Layer-Only Check** — ADR-0242 D-6, ADR-0244 D-12.
  Defense: Defence-in-Depth via Cedar Fragment.
- **Embedded Policy in Application Service** — ADR-0246 D-1.
  Defense: Centralized Policy Service.
- **Trust-On-First-Use Self-Modification** — ADR-0247 D-8. Defense:
  Policy-Engine-Gated Self-Modification.
- **Unrestricted Reflection** — ADR-0247 D-3. Defense: Policy-Gated
  Reflective Tower.

### II.3 Tier + dependency anti-patterns

- **Mixed-Tier Service** — ADR-0245 D-1. Defense: Foundational-vs-
  Application Service Tier.
- **Inferred Tier** — ADR-0245 D-2. Defense: Manifest-Declared
  Service Tier.
- **Lazy Tier Classification** — ADR-0245 D-3. Defense: Per-Service
  Tier Registration.
- **Inverted Dependency** — ADR-0245 D-4. Defense: Layered Service
  Tier DAG.
- **Cyclic Substrate Dependency** — ADR-0245 D-4.B. Defense:
  Foundational Dependency DAG.
- **Forced Two-Tier** — ADR-0245 D-5. Defense: Peer-Cell Service
  Pattern.
- **Live-Before-Certified** — ADR-0245 D-6. Defense: Build-Ahead-
  of-Certification.
- **Honour-System Tier** — ADR-0245 D-7. Defense: Coverage-Required
  Tier Classification.
- **Uniform SLO** — ADR-0245 D-8. Defense: Per-Tier SLO Floor.
- **Unverified Composition** — ADR-0245 D-8.c. Defense: Markov-
  Chain Availability Composition.
- **Uniform Deprecation Window** — ADR-0245 D-9. Defense: Tier-
  Aware Deprecation.

### II.4 Cellular + topology anti-patterns

- **Undocumented External Coupling** — ADR-0248 D-1. Defense:
  External Dependency Inventory.
- **Eternal Bootstrap** — ADR-0248 D-2. Defense: Bootstrap-and-
  Retire.
- **Co-Located Control + Data** — ADR-0248 D-3. Defense: Control
  Plane / Data Plane Separation.
- **One Cell Per Service** — ADR-0248 D-4. Defense: Per-Tenant-
  Group Cell.
- **Service-Cell-Sprawl** — ADR-0248 D-5. Defense: Peer-Tier
  Dedicated-Function Cell.
- **Cross-Cell Hot Path** — ADR-0248 D-6, ADR-0253 D-8. Defense:
  Hot-Path-Intra-Cell / Cellular Architecture Async.
- **Single-Cell-Per-Tenant** — ADR-0248 D-7. Defense: Shuffle
  Sharding.
- **Fail-Fast-On-Control-Plane-Outage** — ADR-0248 D-8. Defense:
  Static Stability.
- **Push-Per-Change Delta** — ADR-0248 D-9. Defense: Constant Work.
- **Manual Cell Provisioning** — ADR-0248 D-10. Defense: Capacity-
  Aware Auto-Spawn.
- **Centralised DNS Hot-Spot** — ADR-0248 D-11. Defense: GeoDNS +
  Edge Failover.
- **Live Tenant Migration Without Audit** — ADR-0248 D-12. Defense:
  Audit-Trail-Backed Tenant Migration.
- **Snowflake Workload** — ADR-0248 D-13. Defense: Workload-In-Pod
  Default.
- **Container-Only Isolation For Untrusted** — ADR-0248 D-14.
  Defense: VM-Per-Workload Isolation.
- **User-Space Syscall Interception** — ADR-0248 D-14.alt. Defense:
  KVM-Backed Isolation.
- **Centralised Ingress** — ADR-0248 D-15. Defense: Distributed
  Edge POP.
- **Graft-On-After-Cert** — ADR-0248 D-16. Defense: Build-Ahead-
  of-Certification.

### II.5 Commerce + marketplace anti-patterns

- **Per-Category Stack Duplication** — ADR-0249 D-1. Defense:
  Substrate-Shared, Surface-Specialised.
- **Per-Category Catalog Fragmentation** — ADR-0249 D-1.1.
  Defense: Universal Product Identifier.
- **Single Global Inventory** — ADR-0249 D-1.2. Defense: Per-
  Warehouse Stock State.
- **Synchronous Order Pipeline** — ADR-0249 D-1.3. Defense: Saga
  Pattern for Distributed Order Workflow.
- **Carrier-Specific Direct Integration** — ADR-0249 D-1.4.
  Defense: Pluggable Carrier + 3PL Adapter Layer.
- **Standalone Review Database** — ADR-0249 D-1.5. Defense: Multi-
  Surface Reputation System.
- **Single-Engine Search** — ADR-0249 D-1.6. Defense: Three-Tier
  Search + Ranking Stack.
- **Per-Category Pricing Logic** — ADR-0249 D-1.7. Defense:
  Pricing-Promotion-Tax Substrate.
- **Single Trust Score** — ADR-0249 D-1.8. Defense: Multi-Signal
  Trust Score with Cold-Start.
- **One Surface for All Categories** — ADR-0249 D-2. Defense: Per-
  Category Surface BC, Shared Substrate.
- **Parallel Stack for Plugins** — ADR-0249 D-3. Defense: Existing-
  Product Refactor onto Shared Substrate.
- **Single-Role Tenant** — ADR-0249 D-4. Defense: Multi-Role
  Tenant.
- **All-or-Nothing Seller** — ADR-0249 D-5. Defense: Per-Category
  Seller Verification.
- **Implicit Fulfillment** — ADR-0249 D-6. Defense: Declared
  Fulfillment Capabilities.
- **No Cold-Start Friction** — ADR-0249 D-7. Defense: Cold-Start
  with Graduated Limits.
- **Single Global Cell** — ADR-0249 D-8. Defense: Cell-Local
  Surface + Cross-Cell Projection.
- **Big-Bang Multi-Category Launch** — ADR-0249 D-9. Defense:
  Phased Activation by Certification.
- **Two-Phase Commit Across Cells** — ADR-0249 D-10. Defense:
  Compensating Saga across Cells.
- **Seller Self-Reports Tax** — ADR-0249 D-11. Defense: Marketplace
  Facilitator with Per-Jurisdiction Activation.
- **Single Policy for All Categories** — ADR-0249 D-12. Defense:
  Per-Category Policy Overlay.
- **Manual Per-Cell Index** — ADR-0249 D-13. Defense: Federated
  Search + OLAP Ranking.
- **Single Global Moderation Policy** — ADR-0249 D-14. Defense:
  Per-Jurisdiction Moderation Overlay.
- **Manual Refund + No Escrow** — ADR-0249 D-15. Defense: Workflow
  Saga with Compensating Action + Escrow.

### II.6 Build-ahead + certification anti-patterns

- **Build-on-cert-grant** — ADR-0250 D-1. Defense: Architected-
  Built-Launched Tri-State.
- **Demo-quality launch** — ADR-0250 D-2. Defense: Operationally-
  Ready-Before-Launch.
- **Global simultaneous launch** — ADR-0250 D-3. Defense: Per-
  Market Launch Gate Matrix.
- **Ad-hoc per-market certification lookup** — ADR-0250 D-4.
  Defense: Certification Catalog as Canonical Source.
- **Imperative eligibility checks** — ADR-0250 D-5. Defense:
  Eligibility-as-Derived-State.
- **Single monolithic policy** — ADR-0250 D-6. Defense: Layered
  Policy Composition.
- **Tribal launch knowledge** — ADR-0250 D-7. Defense: Pre-Launch
  Runbook Discipline.
- **Quarterly product roadmap** — ADR-0250 D-8. Defense: Multi-
  Year Capability Roadmap.
- **Admin override loophole** — ADR-0250 D-9. Defense: No-Bypass
  Defense in Depth.
- **Production-only test** — ADR-0250 D-10. Defense: Sandbox-
  Tenant Pilot.
- **Evidence siloed per certification** — ADR-0250 D-11. Defense:
  Audit-Chain-Anchored Compliance Evidence.
- **Hard kill on cert lapse** — ADR-0250 D-12. Defense: Graceful
  Capability Sunset.

### II.7 Compliance pack anti-patterns

- **Ad-Hoc Per-Regulation Implementation** — ADR-0251 D-1.
  Defense: Compliance-as-Packaged-Bundle.
- **Implicit Compliance Inheritance** — ADR-0251 D-3. Defense:
  Tenant-Installs-Compliance-Regime.
- **Single-Tier Substrate** — ADR-0251 D-4. Defense: Cell-
  Certification-as-Discrete-Levels.
- **Drift via Tenant Movement** — ADR-0251 D-5. Defense: Mandatory-
  Compliance-Pinning.
- **Implicit Cross-Tenant Trust** — ADR-0251 D-6. Defense: Cross-
  Tenant Policy Gate.
- **Manual-Email-PDF Agreement Lifecycle** — ADR-0251 D-7.
  Defense: Durable-Workflow-Driven Compliance-Agreement Lifecycle.
- **First-Breach Scramble** — ADR-0251 D-8. Defense: Per-
  Jurisdiction Breach-Notification Workflow.
- **Per-Use-Case De-ID Implementation** — ADR-0251 D-9. Defense:
  Shared De-Identification Substrate.
- **Per-Service KMS** — ADR-0251 D-10. Defense: Hierarchical-Key-
  Management Substrate.
- **Boolean Consent Field** — ADR-0251 D-11. Defense: Per-Purpose
  Consent Substrate.
- **Free-Form DPIA Document** — ADR-0251 D-12. Defense: Per-
  Regulation DPIA Template.
- **Single Audit Stream** — ADR-0251 D-13. Defense: Per-Stream
  Audit-Chain with Per-Pack Retention.
- **Per-Pack Re-Implementation of Composition** — ADR-0251 D-14.
  Defense: Compositional Policy Semantics.
- **Flat Certification Catalog** — ADR-0251 D-15. Defense:
  Hierarchical Certification Inheritance.
- **Manual Audit-Evidence Compilation** — ADR-0251 D-16. Defense:
  Auto-Emit Auditor Evidence Package.

### II.8 Time + consistency anti-patterns

- **Wall-Clock Ordering** — ADR-0252 D-1. Defense: Hybrid Logical
  Clock.
- **TrueTime Everywhere** — ADR-0252 D-2. Defense: Atomic-Clock-
  Backed External Consistency (Tier-4 only).
- **One Size Fits All Consistency** — ADR-0252 D-3. Defense: Tiered
  Consistency Model.
- **Retry-Without-Dedup** — ADR-0252 D-4. Defense: Stripe
  Idempotency Key.
- **Distributed Lock** — ADR-0252 D-5. Defense: Saga + Compensation,
  Not Lock.
- **Global Cron Service** — ADR-0252 D-6. Defense: Per-Cell
  Periodic Scheduling.
- **Step-At-Leap-Boundary** — ADR-0252 D-7. Defense: Linear Time
  Smear.
- **Server-Generated Idempotency Key** — ADR-0252 D-8. Defense:
  Opaque Self-Describing Key.
- **Global Idempotency Store** — ADR-0252 D-9. Defense: Per-Cell
  Idempotency Store.
- **Wall-Clock Audit Ordering** — ADR-0252 D-10. Defense: HLC-
  Ordered Audit Chain.
- **Implicit Infinite Timeout** — ADR-0252 D-11. Defense: Policy-
  Enforced Deadline.
- **Non-Replayable Workflow** — ADR-0252 D-12. Defense:
  Deterministic Workflow Replay.
- **Silent Clock Drift** — ADR-0252 D-13. Defense: Uncertainty-
  Bounded Time.
- **Per-µservice Clock Reimplementation** — ADR-0252 D-15. Defense:
  Uniform Clock Abstraction.
- **SERIALIZABLE-Everywhere** — ADR-0252 D-16. Defense: Tiered
  Isolation with Policy Opt-In.

### II.9 Network + edge + mesh anti-patterns

- **Single-region DNS** — ADR-0253 D-1.a. Defense: Anycast Apex DNS.
- **Unauthenticated DNS** — ADR-0253 D-1.b. Defense: Zone integrity
  attestation.
- **Plaintext DNS surveillance** — ADR-0253 D-1.c. Defense: Client
  DNS privacy.
- **Cloud-provider-LB-only** — ADR-0253 D-2.a. Defense: Planetary
  Edge POP.
- **Forever vendor edge** — ADR-0253 D-2.b. Defense: Rust-based
  Edge Proxy at Scale.
- **TLS 1.2 downgrade attack surface** — ADR-0253 D-3.a. Defense:
  Modern Crypto at Edge.
- **Cloud-LB-as-L7** — ADR-0253 D-3.b. Defense: L7 Ingress
  Termination.
- **Ad-hoc cert deployment** — ADR-0253 D-3.c. Defense: Certificate-
  as-Code.
- **Harvest-now-decrypt-later** — ADR-0253 D-4. Defense: Post-
  Quantum Hybrid KEX.
- **TCP-only** — ADR-0253 D-5. Defense: Modern Transport at Edge.
- **Sidecar tax** — ADR-0253 D-6. Defense: Layered L3/L4 + L7 mesh.
- **Static service accounts** — ADR-0253 D-7. Defense: Workload
  identity primitive.
- **Public-internet plaintext cross-provider** — ADR-0253 D-9.
  Defense: Per-pair encrypted tunnel.
- **Default-allow egress** — ADR-0253 D-10. Defense: Zero-trust
  egress.
- **Single-tier LB** — ADR-0253 D-11. Defense: Layered load
  balancing.
- **Forever-cloud-BGP** — ADR-0253 D-12. Defense: Own ASN + RPKI.
- **Residency-blind failover** — ADR-0253 D-13. Defense: Tenant-
  aware residency routing.
- **Single-protocol bottleneck** — ADR-0253 D-14. Defense: Multi-
  protocol API surface.
- **Polling** — ADR-0253 D-15. Defense: Realtime push tier.
- **Single-sig + fire-and-forget** — ADR-0253 D-16. Defense: Webhook
  reliability triplet.
- **Forever-hosted-edge** — ADR-0253 D-17. Defense: Phased self-
  hosting migration.
- **Forever-hosted-DNS** — ADR-0253 D-18. Defense: Phased self-
  hosting DNS.

### II.10 Deployment + delivery anti-patterns

- **Noisy-Neighbor Tenant Sprawl** — ADR-0254 D-1.1. Defense:
  Shuffle-Sharded Multi-Tenant SaaS.
- **Shared Substrate Sovereign-Risk** — ADR-0254 D-1.2. Defense:
  Dedicated Cell Pattern.
- **Mandatory Vendor Cloud Lock-In** — ADR-0254 D-1.3. Defense:
  Bring-Your-Own-Cloud.
- **Disconnected Forever On-Prem** — ADR-0254 D-1.4. Defense:
  Connected Edge / Hybrid On-Prem.
- **Online-Only Update Required** — ADR-0254 D-1.5. Defense: Air-
  Gapped Bundle Delivery.
- **N Parallel Codebases** — ADR-0254 D-2. Defense: Single-Build
  Multi-Deployment.
- **Pre-Cellular Deployment Unit** — ADR-0254 D-3. Defense: Cell
  as Unit of Deployment.
- **Per-Customer Bespoke Deployment Tooling** — ADR-0254 D-4.
  Defense: Palantir Apollo Pattern.
- **Unsigned Distribution** — ADR-0254 D-5. Defense: TUF + Cosign
  + SLSA L3 Distribution.
- **Bidirectional Channel Across Air-Gap** — ADR-0254 D-6.
  Defense: Cross-Domain Solution (CDS) Bundle Delivery.
- **Big-Bang Cross-Cell Update** — ADR-0254 D-7. Defense: Flagger
  Progressive Delivery + Per-Model Pull Cadence.
- **Single Support Tier** — ADR-0254 D-8. Defense: Tiered Support
  Matrix.
- **Single Pricing Across Heterogeneous Deployments** — ADR-0254
  D-9. Defense: Cost-Aligned Pricing.
- **Per-Model Compliance Carve-Out** — ADR-0254 D-10. Defense:
  Compliance-Pack Uniform Application.
- **Customer-Provides-Root-Credentials** — ADR-0254 D-11. Defense:
  IAM-Delegated Customer-Account Provisioning.
- **Raw Tenant Data Egress** — ADR-0254 D-12. Defense: Anonymized
  Cedar-Gated Telemetry.
- **Lossy Migration** — ADR-0254 D-13, ADR-0247 D-9. Defense:
  Workflow-Saga Durable Migration / Lossless Substrate
  Distribution.
- **Lost Air-Gap Audit Continuity** — ADR-0254 D-14. Defense:
  Merkle-Sealed Bundled Audit Export.
- **Hardware-Agnostic Spec** — ADR-0254 D-15. Defense: Vendor
  Reference Architecture.
- **Generic Onboarding** — ADR-0254 D-16. Defense: Per-Model
  Onboarding Workflow.

### II.11 AI + intelligence anti-patterns

- **Audience-as-µservice-scope** — ADR-0255 D-1. Defense: Substrate
  + Brand Surface Layering (with audience as call tag).
- **Consumer-Only Substrate** — ADR-0255 D-2. Defense: Audience-
  Neutral Substrate.
- **Brand Concerns in Substrate** — ADR-0255 D-3. Defense: Layered
  Brand Surface.
- **Substrate-Owned Credentials** — ADR-0255 D-4. Defense: provider-credential BYOK
  SecretReference + Owner Declaration.
- **Text-First, Modality-Later** — ADR-0255 D-5. Defense: Multi-
  Modal Day-One Provider Adapter.
- **Stateful Substrate** — ADR-0255 D-6. Defense: Stateless
  Substrate + Durable Composition.
- **Substrate-Side Retrieval Coupling** — ADR-0255 D-7. Defense:
  Caller-Side Retrieval.
- **Embeddings Embedded in Inference Substrate** — ADR-0255 D-8.
  Defense: Embeddings Substrate Promotion.
- **Fine-Tuning Embedded in Dispatch Substrate** — ADR-0255 D-9.
  Defense: Fine-Tuning Substrate Promotion.
- **Single-Tier Model Serving** — ADR-0255 D-10. Defense: Hybrid
  Model Serving.
- **Global Singleton Substrate** — ADR-0255 D-11. Defense: Per-
  Cell Substrate Deployment.
- **Tool-Call Logic in LLM Substrate** — ADR-0255 D-12. Defense:
  Tool-Call Ingress + Dispatcher Separation.
- **Long-Polling Streaming** — ADR-0255 D-13. Defense: Streaming
  via SSE Conventions.
- **Always-Stateful Substrate** — ADR-0255 D-14. Defense: Opt-In
  Session State.
- **Doubled Provider Adapter Surface** — ADR-0255 D-16. Defense:
  Substrate Consolidation Under Universal Tenancy.
- **Silent ADR Drift** — ADR-0255 D-17. Defense: ADR Drift-Loop
  Closure via Keystone Rewrite.
- **Substrate-Implicit ToS Coverage** — ADR-0255 D-18. Defense:
  Owner-Declared ToS Clearance.

### II.12 Self-hosting + self-modification anti-patterns

- **Primitive Duplication Across Sibling µservices** — ADR-0247
  D-1. Defense: Substrate Primitive De-duplication.
- **Internal-CI as Separate µservice** — ADR-0247 D-2. Defense:
  Internal-CI as Tenant-of-Platform.
- **Untraceable Genesis** — ADR-0242 D-5, ADR-0247 D-4. Defense:
  Audited Bootstrap Replay.
- **Big-Bang Bootstrap** — ADR-0247 D-5. Defense: Multi-Stage
  Self-Host Bootstrap.
- **Single-Environment Self-Modification** — ADR-0247 D-6. Defense:
  Three-Tier CD with Auto-Rollback.
- **Mutable Workflow Drift** — ADR-0247 D-7. Defense: Immutable
  Workflow Version Pinning.
- **Vestigial Terminology Sprawl** — ADR-0247 D-10. Defense:
  Inherited-Term Decommission.
- **Untested Self-Modification Surface** — ADR-0247 D-11. Defense:
  Coverage-Required Self-Modification.
- **Tribal Recovery Knowledge** — ADR-0247 D-12. Defense:
  Documented Recovery Procedure.

---

## Appendix III — F2 reviewer quick reference

A 1-page distillation of what an F2 reviewer needs in order to verify
a single PR's hyperscaler-pattern claims.

### III.1 Trigger

Whenever a ChangeSet touches:

- a manifest field tied to a keystone-ADR decision (e.g., `tier`,
  `tier_subtype`, `home_cell`, `dr_cell`, `compliance_packs`,
  `audience`, `marketplace_roles`, `deployment_model`, etc.); or
- a Cedar fragment in `baseline/`, `pack/`, or `tenant/`; or
- an ADR draft that adds, modifies, or removes a hyperscaler-pattern
  attribution row; or
- a substrate µservice's PRD that touches the bounded contexts listed
  in any keystone ADR.

…F2 review fires.

### III.2 Verification checklist

For each touched decision:

1. **Find the row.** Search this document's §3 for the decision ID
   (e.g., `ADR-0248 D-7`).
2. **Verify the pattern name.** Is the name the same one used in
   the per-ADR Appendix A row? If they disagree, the per-ADR row
   wins and this document is the bug.
3. **Verify the source citation.** Click through to at least one of
   the cited sources. Does it describe the named pattern in
   substantially the same way? If the citation is dead, file an F2
   finding.
4. **Verify the anti-pattern.** Does the proposed change genuinely
   avoid the named anti-pattern? Specifically — would a hostile
   reviewer be able to claim the change is decorating around the
   anti-pattern rather than closing it?
5. **For novel patterns:** apply the 4-bullet defensibility checklist
   from §5. If the novelty claim is new and not yet in §5, propose
   adding it.
6. **For cross-references to other keystone ADRs:** verify the
   referenced ADR's decision is consistent with how this ChangeSet
   uses it. E.g., if the ChangeSet relies on ADR-0243 D-4 (Layered
   Policy Composition), is the new Cedar fragment authored in the
   correct overlay scope (baseline / pack / tenant)?

### III.3 Outputs

- **PASS:** all rows verified; F2 verdict APPROVE.
- **PASS with note:** rows verified but a source URL is stale; F2
  verdict APPROVE-WITH-NOTE; open a follow-up PR.
- **FAIL — dead citation:** F2 verdict REQUEST-CHANGES; specify the
  affected row and ask for fresh source.
- **FAIL — pattern name disagreement:** F2 verdict REQUEST-CHANGES;
  reference per-ADR Appendix A vs this document; fix the document.
- **FAIL — anti-pattern not avoided:** F2 verdict REQUEST-CHANGES;
  specify how the proposed change still exposes the anti-pattern.

### III.4 Tools

- `oya check hyperscaler-pattern-matrix-coherence` — CI lane
  verifying §3 mirrors per-ADR Appendix A.
- `oya check citation-url-liveness` — quarterly CI lane verifying
  cited URLs still resolve (200 OK; not redirected to a stub).
- `oya search-pattern <pattern-name>` — local CLI looking up a
  pattern name in the master matrix and returning the row.
- `oya search-anti-pattern <anti-pattern-name>` — local CLI looking
  up an anti-pattern name in Appendix II.

---

## Appendix IV — Pattern-name inventory (alphabetical)

For quick lookup by pattern name. (Decision IDs referenced; sources
in §3.)

- ADR Drift-Loop Closure via Keystone Rewrite — ADR-0255 D-17.
- Air-Gapped Bundle Delivery — ADR-0254 D-1.5.
- Anonymized Cedar-Gated Telemetry — ADR-0254 D-12.
- Anycast Apex DNS — ADR-0253 D-1.a.
- Architected-Built-Launched Tri-State — ADR-0250 D-1.
- Atomic-Clock-Backed External Consistency — ADR-0252 D-2.
- Audience-Neutral Substrate — ADR-0255 D-2.
- Audience-As-Call-Tag — ADR-0255 D-15.
- Audit-Chain-Anchored Compliance Evidence — ADR-0250 D-11.
- Audit-Every-Decision — ADR-0243 D-7.
- Audit-Trail-Backed Tenant Migration — ADR-0248 D-12.
- Audited Bootstrap Replay — ADR-0242 D-5, ADR-0247 D-4.
- Auto-Emit Auditor Evidence Package — ADR-0251 D-16.
- Bootstrap-and-Retire — ADR-0248 D-2.
- Bounded-Depth Hierarchy — ADR-0244 D-2.d.
- Bring-Your-Own-Cloud (BYOC) — ADR-0254 D-1.3.
- Build-Ahead-of-Certification — ADR-0245 D-6, ADR-0248 D-16.
- provider-credential BYOK SecretReference + Owner Declaration — ADR-0255 D-4.
- Caller-Side Attribute Resolution — ADR-0244 D-5.
- Caller-Side Retrieval — ADR-0255 D-7.
- Capability-Aware Auto-Spawn — ADR-0248 D-10.
- Capability-Based Authorization — ADR-0244 D-3.c.
- Cascade-Plus-Tombstone Deletion — ADR-0244 D-7.h.
- Cell as Unit of Deployment — ADR-0254 D-3.
- Cell-Certification-as-Discrete-Levels — ADR-0251 D-4.
- Cell-Local Surface + Cross-Cell Projection — ADR-0249 D-8.
- Cell-Sharded Stateless Tier with HA — ADR-0246 D-5.
- Cellular Architecture Async — ADR-0253 D-8.
- Centralized Policy Service — ADR-0246 D-1.
- Certificate-as-Code — ADR-0253 D-3.c.
- Certification Catalog as Canonical Source — ADR-0250 D-4.
- Client DNS privacy — ADR-0253 D-1.c.
- Closed-Enum Tenant Classification — ADR-0244 D-11.
- Cold-Start with Graduated Limits — ADR-0249 D-7.
- Compensating Saga across Cells — ADR-0249 D-10.
- Compliance-as-Packaged-Bundle — ADR-0251 D-1.
- Compliance-Pack Uniform Application — ADR-0254 D-10.
- Compositional Policy Semantics — ADR-0251 D-14.
- Connected Edge / Hybrid On-Prem — ADR-0254 D-1.4.
- Constant Work — ADR-0248 D-9.
- Control Plane / Data Plane Separation — ADR-0248 D-3.
- Cost-Aligned Pricing — ADR-0254 D-9.
- Coverage-Enforced Policy — ADR-0243 D-9.
- Coverage-Enforced Substrate Doctrine — ADR-0246 D-11.
- Coverage-Required Authorization — ADR-0243 D-3.
- Coverage-Required Self-Modification — ADR-0247 D-11.
- Coverage-Required Tier Classification — ADR-0245 D-7.
- Cross-Domain Solution (CDS) Bundle Delivery — ADR-0254 D-6.
- Cross-Tenant Policy Gate — ADR-0251 D-6.
- Declared Fulfillment Capabilities — ADR-0249 D-6.
- Dedicated Cell Pattern — ADR-0254 D-1.2.
- Defence-in-Depth via Cedar Fragment — ADR-0242 D-6, ADR-0244
  D-12.
- Deterministic Workflow Replay — ADR-0252 D-12.
- Distributed Edge POP — ADR-0248 D-15.
- Distributed Relational with Application-Aware Sharding — ADR-0246
  D-7.
- Documented Recovery Procedure — ADR-0247 D-12.
- Dogfooded Compliance Pipeline — ADR-0242 D-4.
- Durable-Workflow-Driven Compliance-Agreement Lifecycle — ADR-0251
  D-7.
- Eat-Your-Own-Dogfood at Platform Level — ADR-0242 D-1.
- Edge-Cached Policy Evaluation — ADR-0243 D-6.
- Eligibility-as-Derived-State — ADR-0250 D-5.
- Embeddings Substrate Promotion — ADR-0255 D-8.
- Ephemeral Tenant Pattern — ADR-0242 D-8.
- Existing-Product Refactor onto Shared Substrate — ADR-0249 D-3.
- External Dependency Inventory — ADR-0248 D-1.
- Federated Search + OLAP Ranking — ADR-0249 D-13.
- Fine-Tuning Substrate Promotion — ADR-0255 D-9.
- First-Class Platform-Owner Account — ADR-0242 D-7.
- Flagger Progressive Delivery + Per-Model Pull Cadence — ADR-0254
  D-7.
- Foundational-vs-Application Service Tier — ADR-0245 D-1.
- Foundational Dependency DAG — ADR-0245 D-4.B.
- GeoDNS + Edge Failover — ADR-0248 D-11.
- Globally Unique Slug + DNS-Compatible Segments — ADR-0244 D-1.
- Graceful Capability Sunset — ADR-0250 D-12.
- gRPC-Primary with REST Compat — ADR-0246 D-4.
- Hexagonal Architecture with Port-in-Kernel — ADR-0246 D-3.
- Hierarchical Certification Inheritance — ADR-0251 D-15.
- Hierarchical-Key-Management Substrate — ADR-0251 D-10.
- Hierarchical Principal Path — ADR-0242 D-2, ADR-0244 D-2.
- HLC-Ordered Audit Chain — ADR-0252 D-10.
- Hot-Path-Intra-Cell — ADR-0248 D-6.
- Hot-Reload Configuration Distribution — ADR-0243 D-10.
- Hybrid Logical Clock — ADR-0252 D-1.
- Hybrid Model Serving — ADR-0255 D-10.
- IAM-Delegated Customer-Account Provisioning — ADR-0254 D-11.
- Immutable Workflow Version Pinning — ADR-0247 D-7.
- Inherited-Term Decommission — ADR-0247 D-10.
- Internal-CI as Tenant-of-Platform — ADR-0247 D-2.
- Internal Carve-Out (anti-pattern) — closed by ADR-0242 D-1.
- KVM-Backed Isolation — ADR-0248 D-14.alt.
- L7 Ingress Termination — ADR-0253 D-3.b.
- Layered Brand Surface — ADR-0255 D-3.
- Layered L3/L4 + L7 mesh — ADR-0253 D-6.
- Layered load balancing — ADR-0253 D-11.
- Layered Policy Composition — ADR-0243 D-4, ADR-0250 D-6.
- Layered Service Tier DAG — ADR-0245 D-4.
- Linear Time Smear — ADR-0252 D-7.
- Lossless Substrate Distribution — ADR-0247 D-9.
- Mandatory-Compliance-Pinning — ADR-0251 D-5.
- Manifest-Declared Service Tier — ADR-0245 D-2.
- Markov-Chain Availability Composition — ADR-0245 D-8.c.
- Marketplace Facilitator with Per-Jurisdiction Activation —
  ADR-0249 D-11.
- Merkle-Sealed Bundled Audit Export — ADR-0254 D-14.
- Modern Crypto at Edge — ADR-0253 D-3.a.
- Modern Transport at Edge — ADR-0253 D-5.
- Multi-Facet Policy Review — ADR-0243 D-8.
- Multi-Modal Day-One Provider Adapter — ADR-0255 D-5.
- Multi-protocol API surface — ADR-0253 D-14.
- Multi-Role Tenant — ADR-0249 D-4.
- Multi-Signal Trust Score with Cold-Start — ADR-0249 D-1.8.
- Multi-Stage Self-Host Bootstrap — ADR-0247 D-5.
- Multi-State Tenant Lifecycle with Soft-Delete Window — ADR-0244
  D-7.
- Multi-Surface Reputation System — ADR-0249 D-1.5.
- Multi-Year Capability Roadmap — ADR-0250 D-8.
- No-Bypass Defense in Depth — ADR-0250 D-9.
- Operationally-Ready-Before-Launch — ADR-0250 D-2.
- Opt-In Session State — ADR-0255 D-14.
- Opaque Self-Describing Key — ADR-0252 D-8.
- Owner-Declared ToS Clearance — ADR-0255 D-18.
- Own ASN + RPKI — ADR-0253 D-12.
- Palantir Apollo Pattern — ADR-0254 D-4.
- Per-Cell Idempotency Store — ADR-0252 D-9.
- Per-Cell Periodic Scheduling — ADR-0252 D-6.
- Per-Cell Substrate Deployment — ADR-0255 D-11.
- Per-Category Policy Overlay — ADR-0249 D-12.
- Per-Category Seller Verification — ADR-0249 D-5.
- Per-Category Surface BC, Shared Substrate — ADR-0249 D-2.
- Per-Engineer Sandbox Tenant — ADR-0244 D-8.
- Per-Jurisdiction Breach-Notification Workflow — ADR-0251 D-8.
- Per-Jurisdiction Moderation Overlay — ADR-0249 D-14.
- Per-Market Launch Gate Matrix — ADR-0250 D-3.
- Per-Model Onboarding Workflow — ADR-0254 D-16.
- Per-pair encrypted tunnel — ADR-0253 D-9.
- Per-PR Ephemeral Tenant — ADR-0244 D-9.
- Per-Purpose Consent Substrate — ADR-0251 D-11.
- Per-Regulation DPIA Template — ADR-0251 D-12.
- Per-Service Tier Registration — ADR-0245 D-3.
- Per-Stream Audit-Chain with Per-Pack Retention — ADR-0251 D-13.
- Per-Tenant-Group Cell — ADR-0248 D-4.
- Per-Tier SLO Floor — ADR-0245 D-8.
- Per-Warehouse Stock State — ADR-0249 D-1.2.
- Peer-Cell Service Pattern — ADR-0245 D-5.
- Peer-Tier Dedicated-Function Cell — ADR-0248 D-5.
- Phased Activation by Certification — ADR-0249 D-9.
- Phased self-hosting DNS — ADR-0253 D-18.
- Phased self-hosting migration — ADR-0253 D-17.
- PKI Root + Intermediate Certificate Chain — ADR-0243 D-5,
  ADR-0246 D-8.
- Planetary Edge POP — ADR-0253 D-2.a.
- Platform-on-Behalf-Of Pattern — ADR-0244 D-6.3.
- Pluggable Carrier + 3PL Adapter Layer — ADR-0249 D-1.4.
- Policy-as-Feature-Flag — ADR-0252 D-14.
- Policy-Enforced Deadline — ADR-0252 D-11.
- Policy-Engine-Gated Self-Modification — ADR-0247 D-8.
- Policy-Gated Reflective Tower — ADR-0247 D-3.
- Post-Quantum Hybrid KEX — ADR-0253 D-4.
- Pre-Launch Runbook Discipline — ADR-0250 D-7.
- Pricing-Promotion-Tax Substrate — ADR-0249 D-1.7.
- Realtime push tier — ADR-0253 D-15.
- Reserved Identifier Namespace + IDN Homograph Defence — ADR-0242
  D-1.r, ADR-0244 D-1.r.
- Restricted Tenant Self-Policy — ADR-0243 D-12.
- Rust-based Edge Proxy at Scale — ADR-0253 D-2.b.
- Saga + Compensation, Not Lock — ADR-0252 D-5.
- Saga Pattern for Distributed Order Workflow — ADR-0249 D-1.3.
- Shared De-Identification Substrate — ADR-0251 D-9.
- Shuffle Sharding — ADR-0248 D-7.
- Shuffle-Sharded Multi-Tenant SaaS — ADR-0254 D-1.1.
- Signed Migration Ledger + Drain + Cutover — ADR-0244 D-10.
- Signed Policy Authoring Lifecycle — ADR-0243 D-2.
- Signed Policy Bundle Lifecycle with Transparency Log — ADR-0251
  D-2.
- Single-Build Multi-Deployment — ADR-0254 D-2.
- Single-Concern Bounded Contexts — ADR-0246 D-2.
- Single Policy Engine Consolidation — ADR-0243 D-1.
- Single Source of Truth Tenant Registry — ADR-0244 D-3.
- Stateless Substrate + Durable Composition — ADR-0255 D-6.
- Static Stability — ADR-0248 D-8.
- Static Stability + Edge-Cached Evaluation — ADR-0246 D-6.
- Static Stability + Fail-Closed — ADR-0243 D-11.
- Streaming via SSE Conventions — ADR-0255 D-13.
- Substrate Cohesion via PRD Amendment — ADR-0246 D-9.
- Substrate Consolidation Under Universal Tenancy — ADR-0255 D-16.
- Substrate Primitive De-duplication — ADR-0247 D-1.
- Substrate-Shared, Surface-Specialised — ADR-0249 D-1.
- Substrate + Brand Surface Layering — ADR-0255 D-1.
- Tenant-Installs-Compliance-Regime — ADR-0251 D-3.
- Tenant-aware residency routing — ADR-0253 D-13.
- Three-Tier CD with Auto-Rollback — ADR-0247 D-6.
- Three-Tier Search + Ranking Stack — ADR-0249 D-1.6.
- Tier-Aware DR Strategy — ADR-0244 D-3.dr.
- Tier-Aware Deprecation — ADR-0245 D-9.
- Tiered Consistency Model — ADR-0252 D-3.
- Tiered DR + Per-Microservice SLO Ownership — ADR-0246 D-10.
- Tiered Isolation with Policy Opt-In — ADR-0252 D-16.
- Tiered Support Matrix — ADR-0254 D-8.
- Time-Bounded Cross-Tenant Grant — ADR-0244 D-6.
- Tool-Call Ingress + Dispatcher Separation — ADR-0255 D-12.
- TUF + Cosign + SLSA L3 Distribution — ADR-0254 D-5.
- Typed Entity Policy Schema — ADR-0244 D-4.
- Uncertainty-Bounded Time — ADR-0252 D-13.
- Unified Multi-Tenant Substrate — ADR-0242 D-3.
- Unified Policy + Feature Gate — ADR-0243 D-13.
- Uniform Clock Abstraction — ADR-0252 D-15.
- Universal Product Identifier — ADR-0249 D-1.1.
- Vendor Reference Architecture — ADR-0254 D-15.
- VM-Per-Workload Isolation — ADR-0248 D-14.
- Webhook reliability triplet — ADR-0253 D-16.
- Workflow-Saga Durable Migration — ADR-0254 D-13.
- Workflow Saga with Compensating Action + Escrow — ADR-0249 D-15.
- Workload identity primitive — ADR-0253 D-7.
- Workload-In-Pod Default — ADR-0248 D-13.
- Zero-trust egress — ADR-0253 D-10.
- Zone integrity attestation — ADR-0253 D-1.b.

---

*End of hyperscaler-pattern attribution matrix.*
