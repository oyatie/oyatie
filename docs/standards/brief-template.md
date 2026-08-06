---
doc_class: Standard
shape: Reference
length_cap: 1300
authority_tier: 2
status: Accepted
date: 2026-05-20
owner: council-documentation
planned_enforcement_ref: oya-governance-brief-template
related_adrs:
  - ADR-0321
  - ADR-0322
  - ADR-0323
  - ADR-0324
  - ADR-0328
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/standards/anti-patterns.md
  - .omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md
inbound_citations:
  - .omc/plans/realign-oyatie-corpus-plan-2026-05-20.md
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Brief Template Standard

## §1 Purpose — Why every future agent dispatch must use this template

This standard defines the required shape for every future Oyatie authoring, audit,
and remediation dispatch brief.

The goal is not cosmetic uniformity.

The goal is to prevent agent drift before an agent writes a single artifact.

Every brief MUST carry a five-citation header because the 2026-05-20 realignment
failure was not caused by missing effort.

It was caused by effort aimed through underspecified briefs.

The canonical realignment spec identifies three converging failure lanes:
authoring briefs did not encode canonical direction, parallel work lacked clear
ownership, and verification treated line counts plus self-reports as completion.

This template attacks all three lanes at dispatch time.

A compliant brief makes the agent answer five questions before writing content:

1. Which canonical anchors bind this artifact class?
2. What exact file path and line floor must exist at the end?
3. Which substance signals make this artifact buildable from cold?
4. Which branch, verification commands, PR, Jenkins contexts, and reviewer/governance checks prove readiness to merge?
5. Which condition requires HALT-CLEANLY instead of improvising a bad artifact?

The template exists because "write a doc about X" is too weak for this corpus.

Oyatie's docs are build inputs for agents and humans.

If a dispatch brief permits generic wording, the resulting artifact becomes a
source of future implementation error.

If a dispatch brief permits template substitution, the corpus gains shape
without substance.

If a dispatch brief permits line-count-only verification, an orchestrator can
declare completion while downstream agents still cannot build.

Therefore every brief MUST be bespoke to the agent class, scope, and deliverable.

The header template is mandatory.

The body examples in this standard are not copy-paste prose bodies.

They are checklists for what a future brief must decide and cite.

The agent receiving the brief must still write the artifact directly, in its own
scope, against the named anchors.

This standard is also a coordination primitive.

A brief declares ownership, allowed parallelism, and the stop condition.

That declaration prevents two agents from editing the same ADR section, prevents
a microservice audit from spreading across five uncoordinated agents, and
prevents remediation from starting before audit evidence exists.

The brief author owns specificity.

The dispatched agent owns execution.

The orchestrator owns verification.

All three are visible in the brief.

## §2 Brief Anatomy — Required sections of every brief

Every dispatch brief MUST use the sections in this anatomy.

Section names MAY be adapted for the execution surface, but the content fields
are not optional.

### §2.1 Header: five-citation block

The first visible block MUST be `CANONICAL ANCHORS`.

The block MUST contain exactly five anchors.

The five anchors MUST be agent-class-specific, not merely the same five global
links pasted into every brief.

The global realignment anchors usually appear in most brief classes, but the
fifth anchor must point at the concrete microservice, ADR, journey, runbook, or
pack being authored.

Each anchor line MUST include a path and a section, subsection, schema pointer,
or named rule where one exists.

Bare file paths are allowed only when the target file has no stable sections.

Good anchor lines:

- `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 hyperscaler-grade rigor sub-test`
- `/Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md D-2 allowed vs forbidden tool uses`
- `/Users/jasonlee/oyatie/microservices/messenger/PRD.md §E non-functional requirements`

Weak anchor lines:

- `docs`
- `ADR-0321`
- `the microservice docs`

The header MUST also name the agent class.

The header MUST name whether the task is authoring, audit, or remediation.

The header MUST name whether the dispatch is safe to run in parallel.

### §2.2 Identity: agent class, slug, and scope

The brief MUST name the agent class exactly.

The brief MUST include a slug suitable for logs, VCS claim, and bundle naming.

The brief MUST name the smallest ownership scope that gives the agent enough
authority to finish.

For a microservice audit, the scope is one `microservices/<name>/` tree.

For an ADR-0321 dossier author, the scope is one vendor dossier range or one
named vendor family.

For an IP-slice author, the scope is one implementation plan file and its direct
acceptance criteria.

For a journey author, the scope is one journey directory.

For a runbook author, the scope is one operational primitive and its upstream
SLO or incident class.

The identity section MUST state what the agent does not own.

An agent that owns a runbook does not own the parent ADR unless explicitly
assigned.

An agent that owns one vendor dossier does not reorder the whole ADR-0321 file.

An agent that audits one microservice does not remediate another microservice
because it noticed a related flaw.

### §2.3 Deliverable: file path, line floor, and substance bar

The brief MUST state the sole or primary deliverable path.

The path MUST be absolute when dispatched from an orchestrator.

The path MAY also include the repository-relative path for git/PR and `oya gate` evidence.

The line floor MUST be explicit.

The line floor is a floor, not the quality bar.

The substance bar MUST name what an intern can build or verify from the final
artifact.

For example, ">=500 lines" is incomplete.

">=500 lines that let an agent author a compliant future dispatch without
asking which anchors, VCS steps, stop rules, or anti-patterns apply" is complete.

For vendor dossiers, the deliverable section MUST require vendor-specific API
versions, endpoint shapes, auth model, rate-limit behavior, migration hazards,
and rollback branches.

For Cedar policy content, the deliverable section MUST require principal,
action, resource, and context.

For SLO content, the deliverable section MUST require numeric p50, p95, p99,
availability, durability, error-budget, and measurement-window values when
applicable.

For regulatory content, the deliverable section MUST require article, section,
or clause numbers such as GDPR Article 5, GDPR Article 6, GDPR Article 20,
GDPR Article 32, HIPAA 45 CFR §164.312(b), EU AI Act Article 9, EU AI Act
Article 12, Korea PIPA Article 29, or Korea PIPA Article 30.

### §2.4 Procedure: numbered steps

Every brief MUST include a numbered procedure.

The first procedure step MUST be anchor reading.

The second procedure step MUST be path existence and ownership check.

The third procedure step MUST create or verify the isolated worktree branch scope.

The middle procedure steps MUST be artifact-class-specific.

The final procedure steps MUST be verification, done, and promote.

The procedure MUST forbid background "self-report is done" as evidence.

The procedure MUST say how the agent proves the file exists.

The procedure MUST say how the agent proves the file meets the line floor.

The procedure MUST say how the agent proves the file meets substance beyond the
line floor.

### §2.5 Substance Requirements: bespoke content checklist

Every brief MUST contain a checklist titled `Substance Requirements`.

The checklist MUST be written for the specific deliverable.

The checklist MUST reject template-stamping.

The checklist MUST reject generic "service handles X" prose.

Example: the checklist MUST reject deferred-delivery sentinels (the literal
strings `TBD`, the bigram `future` plus `work`, and `see code`) when the
artifact is canonical. This guidance line is itself an `Example:` so the
honest-claims gate does not treat it as an active deferral.

The checklist MUST require named vendor versions when vendors are involved.

Acceptable vendor-version examples include Salesforce REST API v59.0, ServiceNow
Washington DC family release APIs, Workday REST API v1 / SOAP tenant endpoints,
Microsoft Graph v1.0 plus beta-only caveat where needed, Atlassian Jira Cloud
REST API v3, Snowflake SQL API v2, Databricks Jobs API 2.1, Kubernetes 1.35 LTS,
Cilium 1.18, OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, PostgreSQL 17, and Cedar
v4.2 LTS.

The checklist MUST require real regulatory citations when compliance is involved.

Regulatory citations MUST include article or section numbers.

The checklist MUST require Cedar examples with all four permit coordinates.

An acceptable Cedar requirement names:

- `principal`: the user, agent, service, group, tenant role, or Foundry principal.
- `action`: the exact operation such as `OpportunityStageChange` or `RunFailover`.
- `resource`: the protected object such as `SalesOpportunity::<id>` or `Cell::<id>`.
- `context`: tenant, pack, data class, purpose, region, risk score, and break-glass state.

An acceptable Cedar permit example is concrete:

```cedar
permit(
  principal == Oyatie::User::"user_01J9M7F2Z7M1M6YV3HZQK4P8TP",
  action == Oyatie::Action::"OpportunityStageChange",
  resource == Oyatie::Resource::SalesOpportunity::"opp_01J9M7J7R7T7ZQHD2VXP48K2RM"
) when {
  context.tenant == "tenant_acme_us" &&
  context.purpose == "sales_pipeline_management" &&
  context.data_class == "customer_confidential" &&
  context.region in ["us-east-1", "us-west-2"] &&
  context.pack == "us-commercial" &&
  context.risk_score <= 40
};
```

The checklist MUST require real SLO numbers when operational behavior is described.

SLO examples MUST include rationale.

For example, "p95 write latency <=250 ms in-region because interactive CRM stage
changes block a sales-rep workflow" is acceptable.

"Fast" is not acceptable.

An acceptable SLO requirement is concrete:

```yaml
slo:
  operation: crm.opportunity.stage_change
  availability_30d: "99.95%"
  latency_p50_ms: 80
  latency_p95_ms: 250
  latency_p99_ms: 700
  error_budget_30d_minutes: 21.6
  rationale: "Stage changes are interactive sales-console mutations; p95 above 250 ms interrupts pipeline review and p99 above 700 ms causes duplicate-click risk."
```

The checklist MUST require named failure modes.

Failure modes should be specific, such as Bulk API governor limit exhaustion,
Cedar policy explosion, region-isolated OpenBao unseal delay, webhook replay
duplication, DKIM key rotation drift, or workflow replay divergence.

### §2.6 Repository lifecycle: branch -> verify -> PR -> merge

Every brief MUST include the plain-git + PR + Jenkins governance lifecycle.

The lifecycle is not optional for documentation-only work.

The lifecycle begins with an isolated worktree branch before editing:

```bash
git worktree add -b <branch> <isolated-worktree> origin/dev
git status --short --branch
```

The lifecycle verifies after the artifact exists:

```bash
./bin/oya verify --ci-required
./bin/oya gate run-all
```

The lifecycle opens or updates the PR only after verification evidence exists:

```bash
git push -u origin <branch>
gh pr create --base dev --head <branch>
```

The lifecycle merges only after required checks and review/governance gates are green:

```bash
gh pr merge <number> --squash --delete-branch
```

For this standard's own authoring slice, the evidence id is
`brief-template-2026-05-20`.

Future briefs MUST set their own evidence id in the identity section.

### §2.7 HALT-CLEANLY rule

Every brief MUST include a `HALT-CLEANLY` rule.

HALT-CLEANLY means the agent stops the authoring action, reports the blocker,
and leaves partial work in a reviewable state.

HALT-CLEANLY does not mean the agent hides the problem.

HALT-CLEANLY does not mean the agent invents missing authority.

HALT-CLEANLY is required when:

1. One of the five anchors is missing and no canonical replacement exists.
2. The target file is already owned by another active agent claim.
3. The task asks for remediation but the required audit artifact does not exist.
4. The agent cannot meet the substance bar without fabricating vendor, regulatory,
   Cedar, SLO, or failure-mode details.
5. The only apparent path uses scripting, metaprogramming, or template substitution
   to author substantive content.
6. The agent finds a hard contradiction between two authority-tier peers and the
   brief does not say which source wins.
7. Verification fails after the agent has made a bounded correction pass.

The HALT-CLEANLY report MUST include:

- Current file path.
- Completed sections.
- Missing anchor or failed gate.
- Exact command or manual check that failed.
- Proposed next owning agent class.

### §2.8 Forbidden patterns

The brief MUST explicitly forbid the following patterns:

- No scripting the body of substantive content.
- No metaprogramming the body of substantive content.
- No template-substitution authoring.
- No shell loops over artifact names to write prose.
- No `jq`, `awk`, `sed`, Python, Node, Ruby, or generator-driven prose bodies.
- No global find-replace on a sibling artifact to create a new artifact.
- No line-count padding.
- No clause-loop rows that repeat the same assertion with different numbers.
- No vendor-variable swaps.
- No "service handles X" without named handler, route, event, policy, SLO, and failure mode.
- No verification based only on self-report.
- No promote before verify and done.

Tools MAY be used for reading files, checking line counts, searching references,
and running validators.

Tools MUST NOT be used to generate the substantive body of the artifact.

## §3 Agent-Class Catalogue with five-anchor templates per class

Each agent class below defines the default five-anchor set.

A future brief MAY replace one class-specific anchor with a narrower artifact
anchor when the replacement is more authoritative for the exact scope.

A future brief MUST NOT remove the realignment backbone, the substance bar, or
the anti-script doctrine unless a newer accepted ADR supersedes them.

### Anchor Set — §3.1 µservice-ownership-coherence-audit-agent

Purpose: audit one microservice end-to-end for internal coherence, outbound
references, substance, canonical-direction alignment, and industry parity.

Anchor 1: `/Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md §Audit Wave Specification`.

Anchor 2: `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-4 through D-7`.

Anchor 3: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 hyperscaler-grade rigor sub-test and completeness invariants`.

Anchor 4: `/Users/jasonlee/oyatie/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md ownership-coherence directive`.

Anchor 5: `/Users/jasonlee/oyatie/microservices/<service>/PRD.md plus microservices/<service>/ARCHITECTURE.md when present`.

Required deliverables:

- `microservices/<service>/coherence-audit-2026-05-20.md`
- `microservices/<service>/feature-parity-matrix-2026-05-20.md`
- `microservices/<service>/performance-benchmark-numbers-2026-05-20.md`
- `microservices/<service>/capability-tier-deltas-vs-counterparts-2026-05-20.md`

The brief MUST state that this is audit-only unless remediation is explicitly
assigned.

The brief MUST name the service's top three counterparts before writing parity
claims.

The brief MUST require the agent to inspect all docs inside the service path.

The brief MUST classify contradictions as hard or soft using §4.3.

The brief MUST require at least three service-specific failure modes.

The brief MUST require SLO numbers tied to the service's user-facing or
platform-facing role.

### Anchor Set — §3.2 ADR-0321-dossier-author-agent

Purpose: author or repair one ADR-0321 vendor dossier without duplicating,
reordering, or template-stamping other dossiers.

Anchor 1: `/Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md §D dossier pattern`.

Anchor 2: `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-13 ADR-0321 in-scope universe`.

Anchor 3: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 hyperscaler-grade rigor sub-test`.

Anchor 4: `/Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md D-1 and D-2`.

Anchor 5: vendor official documentation for the named version or API surface, plus the relevant Oyatie destination microservice PRD.

The brief MUST state whether the vendor is Big 8, long-tail B2B SaaS,
cloud-infra, PaaS, developer tool, B2C consumer, or IaaS hyperscaler.

The brief MUST apply the §4.1 vendor in-scope decision tree before writing.

The brief MUST require vendor-specific endpoints, objects, rate limits,
migration steps, auth surfaces, UX surfaces, failure modes, and rollback.

The brief MUST require at least one Cedar permit whose action verbs are specific
to the vendor's operational surface.

The brief MUST forbid editing unrelated vendor sections.

The brief MUST require duplicate-section detection before adding a new dossier.

The brief MUST require monotonic section numbering when the dossier lands inside
ADR-0321.

### Anchor Set — §3.3 IP-slice-author-agent

Purpose: author one implementation-plan slice that is small enough for one PR
and strong enough for a future executor to build without invention.

Anchor 1: `/Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md §Canonical Build Sequence`.

Anchor 2: `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-1 through D-2`.

Anchor 3: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §2 Doc-class rigor matrix`.

Anchor 4: `/Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md AP-1 through AP-8`.

Anchor 5: the parent phase spec, parent PRD, or microservice manifest that owns the slice.

The brief MUST name the implementation boundary.

The brief MUST name the exact files a future executor is expected to touch.

The brief MUST name preconditions, acceptance tests, rollback, and evidence.

The brief MUST include at least one failure branch.

The brief MUST include SLO or performance implications when the slice changes a
runtime path.

The brief MUST not allow the agent to write multiple sibling IP bodies through a
shared pattern.

The brief MUST state whether the slice is substrate, capability substrate,
communication/collaboration, distribution, or B2B SaaS.

### Anchor Set — §3.4 per-µservice-ADR-author-agent

Purpose: author one microservice-scoped ADR that binds a real design decision
inside the service's path.

Anchor 1: `/Users/jasonlee/oyatie/microservices/<service>/PRD.md`.

Anchor 2: `/Users/jasonlee/oyatie/microservices/<service>/ARCHITECTURE.md` or service manifest if architecture is not present.

Anchor 3: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 and §2 ADR row`.

Anchor 4: `/Users/jasonlee/oyatie/docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md Decision Summary S-1 through S-8`.

Anchor 5: the root ADR that owns the decision family, such as ADR-0244 for tenant scope, ADR-0243 for Cedar, ADR-0263 for audit emission, ADR-0316 for capability tiers, or ADR-0328 for sequencing.

The brief MUST state the decision question in one sentence.

The brief MUST name at least two rejected alternatives.

The brief MUST include a Cedar hook when authorization changes.

The brief MUST include audit-chain event classes when behavior emits evidence.

The brief MUST include versioning and deprecation when the decision touches a
public contract.

The brief MUST not restate the root ADR as a local ADR.

The brief MUST explain the service-specific delta.

### Anchor Set — §3.5 journey-author-agent

Purpose: author one user journey with real actor, tenant, policy, workflow,
failure, evidence, and rollback details.

Anchor 1: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §2 User stories row`.

Anchor 2: `/Users/jasonlee/oyatie/docs/decisions/ADR-0702-identity-authz-live-apex.md tenant scoping model`.

Anchor 3: `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md policy gate model`.

Anchor 4: `/Users/jasonlee/oyatie/docs/decisions/ADR-0706-observability-live-apex.md audit emission contract`.

Anchor 5: the journey's owning product PRD, persona dossier, and involved microservice PRDs.

The brief MUST name the primary persona.

The brief MUST name the tenant context and audience type.

The brief MUST name the workflow state machine or direct interaction path.

The brief MUST include a Cedar permit or forbid example for the critical action.

The brief MUST include at least one accessibility acceptance criterion.

The brief MUST include at least one localization or regulatory pack implication
when the journey crosses region, identity, payments, employment, health, child,
or consent boundaries.

The brief MUST include observability evidence: event class, trace span, metric,
and log schema.

### Anchor Set — §3.6 runbook-author-agent

Purpose: author one operational runbook that an on-call engineer can execute
under pressure without asking for tribal knowledge.

Anchor 1: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §2 Runbook row`.

Anchor 2: `/Users/jasonlee/oyatie/docs/standards/on-call.md escalation and rotation rules`.

Anchor 3: `/Users/jasonlee/oyatie/docs/standards/observability-slo.md or service SLO file`.

Anchor 4: `/Users/jasonlee/oyatie/docs/decisions/ADR-0706-observability-live-apex.md`.

Anchor 5: the owning microservice runbook index, SLO, dashboard, or incident class.

The brief MUST require trigger conditions.

The brief MUST require pre-checks.

The brief MUST require a numbered procedure with commands or API surfaces.

The brief MUST require verification after every risky operation.

The brief MUST require rollback.

The brief MUST require post-incident evidence.

The brief MUST name timing budgets such as detection within 5 minutes, mitigation
within 15 minutes, and customer-visible status update within 30 minutes when the
incident class is user-facing.

The brief MUST include a Cedar permit for any privileged operational action.

### Anchor Set — §3.7 pack-overlay-author-agent

Purpose: author one regional, regulatory, or tenant-tier overlay pack without
weakening the canonical base.

Anchor 1: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 sovereign-cell awareness`.

Anchor 2: `/Users/jasonlee/oyatie/specs/compliance-pack-schema.json`.

Anchor 3: `/Users/jasonlee/oyatie/docs/decisions/ADR-0702-identity-authz-live-apex.md`.

Anchor 4: `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-1 phase sequence`.

Anchor 5: the specific pack directory, regulator source, or microservice overlay target.

The brief MUST require article-number citations.

The brief MUST require pack activation conditions.

The brief MUST require data residency, retention, consent, audit, and incident
handling deltas.

The brief MUST require at least one example policy overlay.

The brief MUST require a rollback or deactivation path.

The brief MUST reject "compliance handled elsewhere" language.

The brief MUST state whether the pack is KR, EU, US, JP, IN, BR, AU, MX, or a
sovereign-cell overlay.

### Anchor Set — §3.8 cross-handoff-matrix-author-agent

Purpose: author or repair a cross-microservice handoff matrix with clear
producer, consumer, contract, policy, SLO, and failure semantics.

Anchor 1: `/Users/jasonlee/oyatie/docs/decisions/ADR-0701-monorepo-capability-live-apex.md direct gRPC and boundary doctrine`.

Anchor 2: `/Users/jasonlee/oyatie/docs/decisions/ADR-0706-observability-live-apex.md`.

Anchor 3: `/Users/jasonlee/oyatie/docs/standards/cross-microservice-latency-budget.md`.

Anchor 4: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.2 engineering-rigor dimensions`.

Anchor 5: the producer and consumer microservice PRDs, contracts, and SLOs.

The brief MUST name the handoff direction.

The brief MUST name the contract type: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3,
event schema, Cedar policy, or workflow template.

The brief MUST name the owner of retries, idempotency, replay, and compensation.

The brief MUST name p95 and p99 latency budget by hop.

The brief MUST name audit events on both producer and consumer sides.

The brief MUST classify each contradiction found as hard or soft.

The brief MUST not create a broker, adapter, or suite folder unless a binding ADR
already authorizes it.

### Anchor Template — §3.9 multi-context deployment anchor

Purpose: force every brief that touches deployment IaC, runtime hosting, tenant
onboarding, network seams, IAM seams, observability seams, or billing seams to
name the deployment contexts it supports before an agent writes or audits.

Mandatory citation line:
`This µservice supports deployment_contexts <X>/<Y>/<Z> per specs/master-plan-sequencing.json#deployment_contexts and ADR-0328 D-15.`

Required anchor 1:
`/Users/jasonlee/oyatie/specs/master-plan-sequencing.json#deployment_contexts`.

Required anchor 2:
`/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-15`.

Required anchor 3:
`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md`.

Required anchor 4:
`ADR-0215 multi-context platform doctrine`.

Required anchor 5:
`ADR-0218 tenant granular control and per-tenant deployment-context choice`.

Brief field:
`SUPPORTED_CONTEXTS: [oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, oyatie-as-cloud-provider]`.

Brief field:
`CONTEXTS_NA: [{id: <context>, reason: <specific primitive or product reason>, revisit_gate: <gate>}]`.

Brief field:
`CLOUD_SURFACE_POSTURE: cloud-* µservices are Oyatie's IaaS surface, not AWS/OCI wrappers`.

Brief field:
`TENANT_ONBOARDING: tofu init -> tofu plan -> tofu apply through cloud-iac for every supported context`.

Decision tree step 1:
If the µservice is in the cloud-* family, require all six contexts unless a
root ADR says the service cannot apply to a context.

Decision tree step 2:
If the µservice owns IAM, KMS, secrets, storage, compute, network, DNS, billing,
capacity, marketplace, cell, dcops, or fsh, treat `oyatie-as-cloud-provider` as
mandatory.

Decision tree step 3:
If the µservice is identity, tenancy, audit-chain, governance, compliance,
observability, payments, api-gateway, network, or cell, require all six contexts
unless the brief records a hard blocker.

Decision tree step 4:
If the µservice is intelligence, ontology, workflow-engine, workflow-studio,
consent-graph, or detection, require all six contexts because Foundry-like
capability must not be public-cloud-only.

Decision tree step 5:
If the µservice is a collaboration surface, require public cloud, AWS guest, OCI
guest, and Oyatie-as-cloud-provider by default.

Decision tree step 6:
For collaboration on-prem or colo, require the brief to decide whether push,
email, media, recording, or retention dependencies make the context required or
N/A.

Decision tree step 7:
If the µservice is Phase 4 enterprise app surface, decide required contexts from
target buyer expectations and data-residency posture.

Decision tree step 8:
If a context is N/A, the reason must identify the missing primitive, not say
"not applicable" alone.

Decision tree step 9:
If a µservice claims a context, require an `iac/<context>/` module or a linked
remediation finding.

Decision tree step 10:
If a brief says AWS, OCI, on-prem, or colo without naming network, IAM,
observability, and billing seams, fail the anchor before dispatch.

Example: `cloud-iam`.

`cloud-iam` required contexts:
`oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`,
`oyatie-as-cloud-provider`.

`cloud-iam` brief language:
`cloud-iam owns Oyatie cloud identities and maps to AWS roles, OCI dynamic groups,
customer IdPs, and Oyatie provider principals only as backing adapters.`

`cloud-iam` dispatch check:
all six `iac/<context>/` paths or N/A rows are inspected before the audit writes
a pass verdict.

Example: `messenger`.

`messenger` required contexts:
`oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, and
`oyatie-as-cloud-provider`.

`messenger` conditional contexts:
`on-prem` and `colo` require explicit treatment of push notification, media
retention, abuse monitoring, and disconnected operation seams.

`messenger` dispatch check:
if on-prem is marked N/A, the brief must say whether the blocker is push,
retention, abuse, identity federation, or media processing.

Example: foundry-replacement capability.

Foundry replacement spans `intelligence`, `workflow-engine`, `workflow-studio`,
`ontology`, `governance`, and `tenancy`.

Foundry replacement required contexts:
all six contexts, because agentic workflow, ontology, and policy substrate are
platform capabilities.

Foundry replacement dispatch check:
do not permit a brief that makes the capability public-cloud-only.

Required audit output:
`New Constraint Dimensions - Dim 6 Multi-context: PASS | FINDING | N/A`.

Required audit evidence:
name supported context ids, N/A context ids, `iac/<context>/` paths inspected,
tenant onboarding flow, and the four seams.

Forbidden brief language:
`wraps AWS`, `wraps OCI`, `uses the cloud provider's IAM as the product IAM`,
`manual setup`, or `operator provisions the context`.

Forbidden brief omission:
claiming multi-cloud or on-prem support without naming deployment context ids.

Severity cue:
P0 for HR/Payroll, ERP, or CRM context violations; P1 for other in-scope
µservices; P2 for documentation-only gaps.

Stop condition:
the brief can be handed to a fresh audit agent and that agent can determine
required contexts, N/A contexts, owned seams, and tenant onboarding evidence
without asking follow-up questions.

### Anchor Template — §3.10 OpenTofu IaC anchor

Purpose: make OpenTofu the only provisioning path in every brief that touches
deployment, bootstrap, tenant onboarding, cloud resources, local facility
resources, or µservice infrastructure state.

Mandatory citation line:
`Provisioned via OpenTofu modules under microservices/<name>/iac/<context>/ per specs/master-plan-sequencing.json#iac_substrate and ADR-0328 D-16.`

Required anchor 1:
`/Users/jasonlee/oyatie/specs/master-plan-sequencing.json#iac_substrate`.

Required anchor 2:
`/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-16`.

Required anchor 3:
`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md`.

Required anchor 4:
`ADR-0039 supply-chain hardening for sigstore and cosign module signing`.

Required anchor 5:
`microservices/cloud-iac/` as IaC orchestrator surface when present in the local
checkout.

Brief field:
`IAC_ENGINE: OpenTofu`.

Brief field:
`FORBIDDEN_IAC_ENGINES: [Terraform (HashiCorp), Pulumi, CloudFormation as primary, ARM templates as primary]`.

Brief field:
`REQUIRED_MODULE_PATHS: microservices/<name>/iac/<context>/`.

Brief field:
`REQUIRED_FILES: main.tf, variables.tf, outputs.tf, versions.tf, README.md`.

Brief field:
`MODULE_SIGNING: sigstore + cosign per ADR-0039`.

Brief field:
`TENANT_ONBOARDING: tofu init -> tofu plan -> tofu apply`.

Sub-anchor: version pinning.

The brief must require `versions.tf` to pin OpenTofu and provider versions.

Sub-anchor: provider pinning.

The brief must require provider source, provider version, and provider lock
evidence.

Sub-anchor: module signing.

The brief must require signed module package, digest, signer identity, timestamp,
and verification result.

Sub-anchor: state backend.

The brief must map `guest-on-aws` to S3 plus DynamoDB lock.

Sub-anchor: state backend.

The brief must map `guest-on-oci` to OCI Object Storage plus Autonomous DB lock
or approved lock equivalent.

Sub-anchor: state backend.

The brief must map `on-prem` and `colo` to MinIO plus lock table or an approved
customer object store plus lock.

Sub-anchor: state backend.

The brief must map `oyatie-as-cloud-provider` to internal `cloud-storage`
semantics.

Pre-flight check:
search the target scope for `terraform`.

Pre-flight check:
search the target scope for `null_resource`.

Pre-flight check:
search the target scope for `local-exec`.

Pre-flight check:
search the target scope for `remote-exec`.

Pre-flight check:
search the target scope for `provisioner "file"`.

Pre-flight check:
search the target scope for `provisioner "remote-exec"`.

Pre-flight check:
search the target scope for `ssh`.

Pre-flight check:
search the target scope for `pulumi`.

Pre-flight check:
search the target scope for `cloudformation`.

Pre-flight check:
search the target scope for `tfstate` instructions that require hand editing.

Decision tree step 1:
If the µservice claims a deployment context, require an OpenTofu module for that
context or a concrete N/A reason.

Decision tree step 2:
If the brief proposes manual cloud-console steps, reject the brief before
dispatch.

Decision tree step 3:
If a provisioning step cannot be expressed declaratively, route the gap to
`cloud-iac` provider/plugin work instead of allowing `local-exec`.

Decision tree step 4:
If module signing evidence is missing, classify as finding even when module
shape is otherwise complete.

Decision tree step 5:
If state backend differs from the context policy, classify as finding.

Decision tree step 6:
If tenant onboarding omits `tofu init`, `tofu plan`, or `tofu apply`, classify
as finding.

Decision tree step 7:
If the code uses Terraform naming only in historical retired docs, record
provenance and do not fail live scope unless current docs point to it.

Decision tree step 8:
If a README says "Terraform compatible", rewrite the brief requirement to
OpenTofu and record the naming mismatch.

Example acceptable sentence:
`cloud-network guest-on-aws deployment uses OpenTofu under microservices/cloud-network/iac/guest-on-aws/ with S3+DynamoDB state, signed modules, and cloud-iac plan/apply orchestration.`

Example unacceptable sentence:
`Run Terraform, then SSH into the load balancer host and finish setup.`

Required audit output:
`New Constraint Dimensions - Dim 7 OpenTofu IaC: PASS | FINDING | N/A`.

Required audit evidence:
list module directories, required files present, version pins, signing evidence,
state backend, forbidden-pattern search result, and tenant onboarding command.

Severity cue:
P0 for HR/Payroll, ERP, or CRM IaC violations; P1 for other in-scope µservices;
P2 for missing docs with otherwise compliant modules.

Stop condition:
the brief proves zero-handroll provisioning through OpenTofu and gives the audit
agent exact paths and forbidden patterns to inspect.

### Anchor Template — §3.11 OS support anchor

Purpose: require every µservice brief to expose the OS and architecture contract
that downstream auditors must enforce before deployment portability claims.

Mandatory citation line:
`Supported OSes per microservices/<name>/supported-oses.json against specs/master-plan-sequencing.json#supported_oses Tier-1 and ADR-0328 D-17.`

Required anchor 1:
`/Users/jasonlee/oyatie/specs/master-plan-sequencing.json#supported_oses`.

Required anchor 2:
`/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-17`.

Required anchor 3:
`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md`.

Required anchor 4:
`microservices/<name>/supported-oses.json`.

Required anchor 5:
the µservice CI workflow, package manifest, or build evidence path that proves
Tier 1 and Tier 2 lane status.

Brief field:
`SUPPORTED_OSES_MANIFEST: microservices/<name>/supported-oses.json`.

Brief field:
`TIER_1_BLOCKING: [talos, rhel-9.x+, oracle-linux-9.x+, sles-15-sp6+, ubuntu-24.04-lts+, debian-13+, rocky-9.x+, almalinux-9.x+, centos-stream-10+, amazon-linux-2023+, flatcar, photon-5.x+, macos-apple-silicon-m5+]`.

Brief field:
`TIER_2_TEST_ONLY: [linux-ppc64le, linux-s390x]`.

Brief field:
`OUT_OF_SCOPE_EXPLICIT: [macos-intel, macos-apple-silicon-pre-m5, freebsd, openbsd, windows-server, solaris]`.

Brief field:
`ARCH_MATRIX: [linux/amd64, linux/arm64, darwin/arm64-m5+, linux/ppc64le-test-only, linux/s390x-test-only]`.

Sub-anchor: package formats.

RPM applies to RHEL, Oracle Linux, SLES, Rocky, AlmaLinux, CentOS Stream, Amazon
Linux, and Photon host-install cases.

Sub-anchor: package formats.

DEB applies to Ubuntu and Debian host-install cases.

Sub-anchor: package formats.

Container images apply to every Linux Tier 1 OS as the primary Kubernetes
deployment unit.

Sub-anchor: package formats.

Talos extension applies only when Talos host integration needs more than a
container image.

Sub-anchor: package formats.

Flatcar ignition or extension applies only when Flatcar host integration needs
more than a container image.

Sub-anchor: package formats.

macOS `.pkg` and Homebrew apply only to Apple Silicon M5+ macOS support.

Sub-anchor: CI gates.

Tier 1 lanes block release claims.

Sub-anchor: CI gates.

Tier 2 ppc64le and s390x lanes are soft-gates unless a later ADR promotes them.

Sub-anchor: portability.

Rust binaries and containers must not depend on a Python interpreter.

Sub-anchor: portability.

No service may claim generic Linux support without distro-specific rows.

Pre-flight check:
open `microservices/<name>/supported-oses.json`.

Pre-flight check:
verify every Tier 1 OS is present or explicitly N/A with service-local reason.

Pre-flight check:
verify Tier 2 rows are marked test-only.

Pre-flight check:
verify Intel macOS and pre-M5 Apple Silicon are excluded.

Pre-flight check:
verify FreeBSD, OpenBSD, Windows Server, and Solaris are excluded.

Pre-flight check:
verify package formats match OS rows.

Pre-flight check:
verify CI lane names and blocking/soft-gate behavior are stated.

Pre-flight check:
verify no Python, Node, or shell runtime is required for build or install.

Decision tree step 1:
If the µservice ships only a hosted control-plane API, it still needs Linux
container and host compatibility evidence for its runtime.

Decision tree step 2:
If the µservice ships a host agent, require per-distro package and kernel
assumption details.

Decision tree step 3:
If the µservice supports local developer tooling, decide whether macOS M5+
support applies.

Decision tree step 4:
If macOS support applies, require `darwin/arm64-m5+` and explicit Intel/pre-M5
exclusion.

Decision tree step 5:
If a service claims on-prem or colo support, require Linux distro support to be
explicit rather than inherited from public-cloud containers.

Decision tree step 6:
If a service needs Oracle Linux because of OCI, require arm64/Ampere evidence.

Decision tree step 7:
If a service marks an OS N/A, require the blocker and revisit gate.

Decision tree step 8:
If no manifest exists, classify at least P2 and P1 when deployment support is
claimed.

Example acceptable sentence:
`cloud-iac declares Tier 1 support in microservices/cloud-iac/supported-oses.json, blocks on Oracle Linux arm64 for OCI, and excludes Intel macOS and Windows Server.`

Example unacceptable sentence:
`Works on Linux and macOS.`

Required audit output:
`New Constraint Dimensions - Dim 8 OS support: PASS | FINDING | N/A`.

Required audit evidence:
manifest path, Tier 1 rows, Tier 2 rows, exclusions, architecture matrix,
package formats, CI lane status, and portability check result.

Severity cue:
P0 for HR/Payroll, ERP, or CRM OS violations; P1 for other in-scope µservices;
P2 for missing manifest or docs when behavior appears compliant.

Stop condition:
the brief gives the audit agent enough OS, architecture, package, CI, and
exclusion detail to reject vague portability claims.

### Anchor Template — §3.12 language policy anchor

Purpose: ensure every brief enforces Rust-only backend, µservice, scripting,
validation, codegen, and CI behavior while preserving the narrow native frontend
allowlist.

Mandatory citation line:
`Backend code in Rust per specs/master-plan-sequencing.json#language_policy and ADR-0328 D-18; frontend native bundle per platform allowlist only.`

Required anchor 1:
`/Users/jasonlee/oyatie/specs/master-plan-sequencing.json#language_policy`.

Required anchor 2:
`/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-18`.

Required anchor 3:
`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md`.

Required anchor 4:
`/Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md`.

Required anchor 5:
the µservice Cargo manifest, workspace membership, frontend directory, or
per-µservice non-Rust exception ADR.

Brief field:
`BACKEND_LANGUAGE: rust`.

Brief field:
`CANONICAL_BACKEND_BUILD: cargo build --workspace --release --all-features --locked`.

Brief field:
`AUTHORIZED_BACKEND_NON_RUST_EXTENSIONS: [.tf, .cedar, .yaml, .json, .proto, openapi.yaml, asyncapi.yaml, .openslo.yaml, .sql, .md]`.

Brief field:
`FRONTEND_ALLOWLIST: frontend/ios=Swift, frontend/macos=Swift, frontend/android=Kotlin, frontend/windows=WinUI 3 C#/.NET net8.0+, frontend/web=Leptos (Rust→WASM, mandatory SSR with selective island-scoped WebAssembly hydration; static sections SSR-only, hydration opt-in per island; CSR-only and whole-page-hydration forbidden)`.

Brief field:
`EXCEPTION_PROTOCOL: microservices/<name>/decisions/ADR-MS-NNN-non-rust-justification.md`.

Sub-anchor: Rust-only backend.

Runtime handlers, workers, CLIs, validation tools, codegen tools, scripting, and
durable CI behavior are Rust.

Sub-anchor: authorized IaC.

OpenTofu `.tf` files are infrastructure declarations only.

Sub-anchor: authorized policy.

Cedar `.cedar` files are policy declarations only.

Sub-anchor: authorized config.

YAML and JSON are configuration, contract, manifest, spec, or evidence data, not
application logic.

Sub-anchor: authorized contracts.

OpenAPI, AsyncAPI, and proto3 define contracts and generated clients, not
alternate source-of-truth logic.

Sub-anchor: authorized SLO.

OpenSLO YAML defines SLO data.

Sub-anchor: authorized SQL.

SQL files are sqlx migrations and schema fixtures, not workflow engines.

Sub-anchor: authorized docs.

Markdown is documentation and cannot prescribe forbidden scripts as required
operations.

Sub-anchor: frontend scoping.

Swift, Kotlin, and WinUI 3 C#/.NET are allowed only under their frontend
platform directories.

Pre-flight check:
search backend and µservice paths for `*.py`.

Pre-flight check:
search backend and µservice paths for `*.js`.

Pre-flight check:
search backend and µservice paths for `*.ts` and `*.tsx`.

Pre-flight check:
search backend and µservice paths for `*.rb`.

Pre-flight check:
search backend and µservice paths for `*.pl`.

Pre-flight check:
search backend and µservice paths for `*.php`.

Pre-flight check:
search backend and µservice paths for `*.java`.

Pre-flight check:
search backend and µservice paths for `*.scala`.

Pre-flight check:
search backend and µservice paths for `*.groovy`.

Pre-flight check:
search backend and µservice paths for `*.go`.

Pre-flight check:
search backend and µservice paths for `*.fs` and `*.fsx`.

Pre-flight check:
search for `*.cs` outside `frontend/windows/`.

Pre-flight check:
search for `*.kt` outside `frontend/android/` or ADR-approved frontend shared
code.

Pre-flight check:
search for `*.swift` outside `frontend/ios/` and `frontend/macos/`.

Pre-flight check:
search for backend `package.json`, `pyproject.toml`, `Gemfile`, `go.mod`,
`pom.xml`, and `build.gradle`.

Decision tree step 1:
If a forbidden file exists in backend or µservice runtime scope, require an
exception ADR or classify a violation.

Decision tree step 2:
If JavaScript or TypeScript appears as generated SDK output, require generation
provenance before clearing it.

Decision tree step 3:
If Kotlin appears outside Android frontend scope, require a frontend shared-code
ADR or classify a violation.

Decision tree step 4:
If C# appears outside Windows frontend scope, classify a backend language
violation unless an ADR-approved FFI/frontend boundary exists.

Decision tree step 5:
If Swift appears outside iOS or macOS frontend scope, classify a violation.

Decision tree step 6:
If a build doc requires `make`, `npm run`, `python setup.py`, or `gradle` for
backend release, classify a violation.

Decision tree step 7:
If a Codex brief proposes generating Rust or Markdown through scripts, reject it
under ADR-0324 before dispatch.

Decision tree step 8:
If only docs omit the Rust-strict boundary while code is compliant, classify P2
and add a doc remediation row.

Example acceptable sentence:
`cloud-iac orchestration is Rust, its IaC modules are .tf OpenTofu declarations, and any frontend code is outside the backend path.`

Example unacceptable sentence:
`Use a Python helper to validate modules before cargo build.`

Required audit output:
`New Constraint Dimensions - Dim 9 Rust-strict: PASS | FINDING | N/A`.

Required audit evidence:
Cargo/workspace evidence, forbidden-language scan result, allowed non-Rust file
classification, frontend path classification, build invocation, and exception
ADR references.

Severity cue:
P0 for HR/Payroll, ERP, or CRM language violations; P1 for other in-scope
µservices; P2 for missing docs when code appears compliant.

Stop condition:
the brief makes Rust the backend source of truth, restricts frontend languages
to native bundle paths, and gives audit agents exact grep targets.

#### §3.12 Amendment summary for Wave 1 Task 1.4

`docs/decisions/ADR-0700-ci-admission-live-apex.md`
was amended with §D-15 through §D-20.

§D-15 adds the six-context deployment matrix, context seams, µservice surface
rules, CI lanes, tenant onboarding, cloud-* IaaS rationale, forbidden patterns,
and severity cues.

§D-16 adds OpenTofu-only IaC policy, required module files, version/provider
pinning, sigstore/cosign signing, state backend policy, `cloud-iac`
orchestration, forbidden patterns, and severity cues.

§D-17 adds Tier 1 and Tier 2 OS support policy, explicit exclusions,
architecture matrix, package formats, `supported-oses.json` schema, CI lane
policy, portability gates, and severity cues.

§D-18 adds Rust-only backend policy, allowed non-Rust extensions, frontend-only
Swift/Kotlin/WinUI scopes, forbidden languages, canonical Cargo build
invocation, anti-script authoring rules, grep targets, and severity cues.

`iac/oci-guest/always-free/` contract, zero-cost billing events, OCI state
backend, strategic rationale, provider-agnostic boundary, and severity cues.

§D-20 adds audit-agent application rules for new Dimensions 6 through 9,
constraint-specific finding examples, P0/P1/P2 decision tree, and memory-file
cross-references.

`specs/master-plan-sequencing.json` was amended with five top-level keys.

`deployment_contexts` adds the six canonical context ids, scope, IaC target, and
default tenant class.

`iac_substrate` adds OpenTofu engine policy, forbidden engines, pinning, signing,
state backend mapping, `cloud-iac`, forbidden patterns, and onboarding command.

`supported_oses` adds Tier 1, Tier 2, explicit exclusions, architecture matrix,
CI lane policy, manifest requirement, and ADR pointer.

`language_policy` adds Rust backend strictness, frontend language allowlist,
frontend scoping, authorized backend extensions, forbidden backend languages,
build invocation, exception protocol, and ADR pointer.

budgets, per-µservice module path, cross-cloud prohibition, and ADR pointer.

`docs/standards/brief-template.md` was amended with §3.9 through §3.12.

§3.9 gives the copy-paste multi-context anchor, decision tree, cloud-* examples,
audit evidence, forbidden language, severity cue, and stop condition.

§3.10 gives the copy-paste OpenTofu anchor, module and state sub-anchors,
forbidden pattern pre-flight checks, decision tree, audit evidence, severity
cue, and stop condition.

§3.11 gives the copy-paste OS support anchor, Tier 1/Tier 2/exclusion fields,
package and CI sub-anchors, pre-flight checks, decision tree, audit evidence,
severity cue, and stop condition.

§3.12 gives the copy-paste language policy anchor, Rust/backend/frontend
scoping, authorized and forbidden file classes, pre-flight checks, decision
tree, audit evidence, severity cue, stop condition, and this summary.

Measured line-count delta:

- `docs/decisions/ADR-0700-ci-admission-live-apex.md` -> `§D-15 through §D-20` -> `+2425 lines (1888 -> 4313)`.
- `specs/master-plan-sequencing.json` -> `deployment_contexts`, `iac_substrate`, `supported_oses`, `language_policy`, `oci_always_free` -> `+164 lines (705 -> 869)`.
- `docs/standards/brief-template.md` -> `§3.9 through §3.12` -> `+710 lines (1181 -> 1891)`.

## §4 Decision Trees

Decision trees MUST be copied into future briefs when they govern the task.

They are operational checks, not background reading.

### §4.1 Is vendor X in-scope for ADR-0321?

Start with the vendor's buyer and product shape.

If the vendor is B2B SaaS, answer YES.

Examples: Salesforce, ServiceNow, Workday, HubSpot, Atlassian, Zendesk, DocuSign,
Notion, Linear, Pendo, Figma, Miro, Snowflake, Databricks, Box, Okta, and Stripe.

If the vendor is cloud-infra software or managed platform surface, answer YES
when the dossier maps to capabilities Oyatie must subsume or interoperate with.

Examples: Cloudflare R2, Cloudflare Workers, Vercel, Netlify, Fly.io, MongoDB
Atlas, Confluent Cloud, PlanetScale, Supabase, Sentry, Datadog, HashiCorp Cloud,
and Elastic Cloud.

If the vendor is PaaS, answer YES.

Examples: Heroku, Render, Railway, Vercel, Netlify, Fly.io, Supabase, Firebase,
and Cloudflare Workers.

If the vendor is a developer tool, answer YES.

Examples: GitHub, GitLab, JetBrains, Snyk, Sonar, Linear, LaunchDarkly, Postman,
PagerDuty, Sentry, Datadog, Docker Hub, and Buildkite.

If the vendor is B2C consumer with no B2B operational control-plane relevance,
answer NO for ADR-0321.

Examples: TikTok consumer app, Netflix consumer streaming, Spotify consumer
music, consumer games, or consumer dating apps.

If the vendor is IaaS hyperscaler compute rental as a whole, answer NO for
ADR-0321 because hyperscaler primitives belong in architecture and deployment
doctrine, not B2B SaaS industry-leader dossiers.

Examples: AWS as a whole, Azure as a whole, Google Cloud as a whole, and OCI as
a whole.

If a hyperscaler sub-surface behaves like a B2B SaaS or PaaS product, classify
the sub-surface, not the parent company.

Example: GitHub is YES even though Microsoft is a hyperscaler-adjacent company.

Example: Firebase is YES as PaaS; Google Cloud as a whole is NO for ADR-0321.

If the answer is unclear, HALT-CLEANLY and ask the orchestrator to classify the
vendor before authoring.

### §4.2 Is content substance-bar?

Ask whether an intern can build, audit, or operate from the artifact without
asking for private context.

If the artifact names exact vendors and versions, continue.

If the artifact says "the vendor API" without version or endpoint shape, fail.

If the artifact names real regulatory citations with article or section numbers,
continue.

If the artifact says "privacy law applies" without citation, fail.

If the artifact includes Cedar permits with principal, action, resource, and
context, continue.

If the artifact says "authorization is checked" without a policy shape, fail.

If the artifact includes real SLO numbers and rationale, continue.

If the artifact says "fast", "reliable", "scalable", or "secure" without
numeric or mechanical evidence, fail.

If the artifact names at least three real failure modes and the system behavior
for each, continue.

If the artifact lists only happy-path steps, fail.

If each paragraph contributes a new decision, example, branch, citation, or
mechanic, continue.

If paragraphs can be moved to another artifact by changing only the noun, fail.

If the artifact's line floor is met but the bespoke substance is not, fail.

If the artifact is shorter than the floor but fully specific, still fail the
line-floor gate and extend with real substance.

If the artifact cannot be extended without padding, HALT-CLEANLY rather than
write filler.

### §4.3 Is contradiction hard or soft?

A hard contradiction exists when one authority says X and another authority says
NOT-X.

Example hard contradiction: a service PRD says `mail` owns DKIM custody, while
the runbook says `comms-email` owns DKIM key rotation for the same tenant path.

Example hard contradiction: one ADR says a mutation is Cedar-gated, while the
journey says the same mutation bypasses policy for convenience.

Example hard contradiction: one SLO says p95 <=250 ms, while the runbook treats
900 ms as normal for the same operation and window.

Hard contradictions MUST be logged as blockers.

Hard contradictions require remediation, authority selection, or explicit
supersession.

A soft contradiction exists when one authority is silent on X.

Example soft contradiction: a PRD names the user-facing flow but does not specify
the audit event class.

Example soft contradiction: a runbook has rollback but does not name the
dashboard panel.

Example soft contradiction: an ADR names Cedar but does not include the service's
exact action verbs.

Soft contradictions become substance gaps.

Soft contradictions can be remediated by adding the missing detail without
overruling another source.

If a brief does not authorize remediation, the agent records the contradiction
and HALTs or completes the audit-only deliverable according to scope.

## §5 Code-block examples of well-formed brief headers per agent class

The examples below show header blocks only.

They are not full briefs.

They demonstrate anchor specificity, identity, deliverable, VCS bundle, and
HALT-CLEANLY placement.

### §5.1 Header example: µservice-ownership-coherence-audit-agent

```markdown
AGENT CLASS: µservice-ownership-coherence-audit-agent
AGENT SLUG: codex-audit-messenger-coherence
MODE: audit-only
PARALLELISM: safe with other microservices; do not edit outside microservices/messenger/
BUNDLE: messenger-coherence-audit-2026-05-20

CANONICAL ANCHORS:
1. /Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md §Audit Wave Specification
2. /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-4 through D-7
3. /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 hyperscaler-grade rigor sub-test
4. /Users/jasonlee/oyatie/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md
5. /Users/jasonlee/oyatie/microservices/messenger/PRD.md + ARCHITECTURE.md

DELIVERABLE:
/Users/jasonlee/oyatie/microservices/messenger/coherence-audit-2026-05-20.md, plus the three parity/benchmark/tier delta companion docs named in the brief.
Line floor: >=500 lines across the audit doc; no line-padding.
Substance bar: identify hard contradictions, soft gaps, top-3 counterpart parity, Cedar/SLO/audit-event implications, and exact remediation candidates.

HALT-CLEANLY:
If the messenger PRD or architecture file is missing, report the missing anchor and do not invent service ownership.
```

### §5.2 Header example: ADR-0321-dossier-author-agent

```markdown
AGENT CLASS: ADR-0321-dossier-author-agent
AGENT SLUG: codex-dossier-servicenow-itsm
MODE: authoring
PARALLELISM: not safe against other ADR-0321 writers unless section range is pre-claimed
BUNDLE: adr0321-servicenow-itsm-dossier-2026-05-20

CANONICAL ANCHORS:
1. /Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md §D dossier pattern
2. /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-13 ADR-0321 in-scope universe
3. /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 hyperscaler-grade rigor sub-test
4. /Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md D-1 through D-2
5. ServiceNow Washington DC release API docs + /Users/jasonlee/oyatie/microservices/itsm/PRD.md

DELIVERABLE:
One ServiceNow ITSM dossier section in ADR-0321.
Line floor: >=130 lines for Big-8-priority density when assigned as hero dossier; otherwise >=80 lines with no generic row reuse.
Substance bar: named ServiceNow table APIs, CMDB and incident objects, assignment rules, Flow Designer migration, Cedar permit verbs, SLOs, and failure modes.

HALT-CLEANLY:
If a ServiceNow dossier already exists, repair the existing section instead of adding a duplicate.
```

### §5.3 Header example: IP-slice-author-agent

```markdown
AGENT CLASS: IP-slice-author-agent
AGENT SLUG: codex-ip-mail-dkim-rotation
MODE: authoring
PARALLELISM: safe only if no sibling agent edits the same microservice/mail phase file
BUNDLE: mail-dkim-rotation-ip-2026-05-20

CANONICAL ANCHORS:
1. /Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md §Canonical Build Sequence
2. /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-1 through D-2
3. /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §2 Migration playbook and Spec rows where applicable
4. /Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md AP-1 through AP-8
5. /Users/jasonlee/oyatie/microservices/mail/PRD.md + /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md

DELIVERABLE:
/Users/jasonlee/oyatie/microservices/mail/IP-###-dkim-key-rotation.md
Line floor: >=250 lines.
Substance bar: exact key lifecycle, OpenBao path, DNS propagation wait, rollback, p95 rotation control-plane latency, evidence events, and tests.

HALT-CLEANLY:
If ADR-0273 and the mail PRD conflict on owner, classify as hard contradiction and stop before writing implementation steps.
```

### §5.4 Header example: per-µservice-ADR-author-agent

```markdown
AGENT CLASS: per-µservice-ADR-author-agent
AGENT SLUG: codex-adr-ms-observability-cardinality-budget
MODE: authoring
PARALLELISM: safe only within the claimed service ADR path
BUNDLE: observability-cardinality-budget-adr-2026-05-20

CANONICAL ANCHORS:
1. /Users/jasonlee/oyatie/microservices/observability/PRD.md
2. /Users/jasonlee/oyatie/microservices/observability/ARCHITECTURE.md
3. /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 and §2 ADR row
4. /Users/jasonlee/oyatie/docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md S-1 through S-8
5. /Users/jasonlee/oyatie/docs/decisions/ADR-0706-observability-live-apex.md

DELIVERABLE:
/Users/jasonlee/oyatie/microservices/observability/decisions/ADR-MS-###-cardinality-budget.md
Line floor: >=200 lines.
Substance bar: decision-specific alternatives, metric cardinality math, tenant budget enforcement, audit events, failure modes, and rollback.

HALT-CLEANLY:
If the decision is already made in a root ADR, write only the service-specific delta or stop.
```

### §5.5 Header example: journey-author-agent

```markdown
AGENT CLASS: journey-author-agent
AGENT SLUG: codex-journey-contract-signing
MODE: authoring
PARALLELISM: safe if the journey directory is uniquely claimed
BUNDLE: j38-contract-signing-journey-2026-05-20

CANONICAL ANCHORS:
1. /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §2 User stories row
2. /Users/jasonlee/oyatie/docs/decisions/ADR-0702-identity-authz-live-apex.md
3. /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md
4. /Users/jasonlee/oyatie/docs/decisions/ADR-0706-observability-live-apex.md
5. /Users/jasonlee/oyatie/docs/user-journeys/j38-b2b-e-signing-contract/README.md plus involved microservice PRDs

DELIVERABLE:
/Users/jasonlee/oyatie/docs/user-journeys/j38-b2b-e-signing-contract/story.md
Line floor: >=250 lines.
Substance bar: persona, tenant, Cedar, workflow state, audit event, accessibility, localization, failure, and rollback detail.

HALT-CLEANLY:
If the persona or owning microservice is ambiguous, record the ambiguity and stop instead of writing a generic journey.
```

### §5.6 Header example: runbook-author-agent

```markdown
AGENT CLASS: runbook-author-agent
AGENT SLUG: codex-runbook-cloud-kms-unseal-delay
MODE: authoring
PARALLELISM: safe if the runbook path is uniquely claimed
BUNDLE: cloud-kms-unseal-delay-runbook-2026-05-20

CANONICAL ANCHORS:
1. /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §2 Runbook row
2. /Users/jasonlee/oyatie/docs/standards/on-call.md escalation and rotation rules
3. /Users/jasonlee/oyatie/microservices/cloud-kms/slos/control-plane.openslo.yaml
4. /Users/jasonlee/oyatie/docs/decisions/ADR-0706-observability-live-apex.md
5. /Users/jasonlee/oyatie/microservices/cloud-kms/runbooks/README.md

DELIVERABLE:
/Users/jasonlee/oyatie/microservices/cloud-kms/runbooks/openbao-unseal-delay.md
Line floor: >=250 lines.
Substance bar: trigger, pre-checks, commands, Cedar permit, OpenBao path, timing budget, verification, rollback, and post-incident evidence.

HALT-CLEANLY:
If the privileged action lacks a Cedar permit model, stop and file the missing policy gap.
```

### §5.7 Header example: pack-overlay-author-agent

```markdown
AGENT CLASS: pack-overlay-author-agent
AGENT SLUG: codex-pack-kr-pipa-consent-overlay
MODE: authoring
PARALLELISM: safe only for the claimed pack path
BUNDLE: kr-pipa-consent-overlay-2026-05-20

CANONICAL ANCHORS:
1. /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 sovereign-cell awareness
2. /Users/jasonlee/oyatie/specs/compliance-pack-schema.json
3. /Users/jasonlee/oyatie/docs/decisions/ADR-0702-identity-authz-live-apex.md
4. /Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md D-1 phase sequence
5. Korea PIPA Article 29 and Article 30 plus /Users/jasonlee/oyatie/packs/kr/

DELIVERABLE:
/Users/jasonlee/oyatie/packs/kr/overlays/consent-graph-pipa.md
Line floor: >=300 lines.
Substance bar: article-number citations, activation condition, data-class impact, Cedar overlay, retention, audit, and deactivation.

HALT-CLEANLY:
If the pack would weaken canonical-base behavior, stop and require an ADR amendment.
```

### §5.8 Header example: cross-handoff-matrix-author-agent

```markdown
AGENT CLASS: cross-handoff-matrix-author-agent
AGENT SLUG: codex-handoff-mail-comms-email
MODE: authoring
PARALLELISM: not safe if producer or consumer contracts are being edited
BUNDLE: mail-comms-email-handoff-2026-05-20

CANONICAL ANCHORS:
1. /Users/jasonlee/oyatie/docs/decisions/ADR-0701-monorepo-capability-live-apex.md
2. /Users/jasonlee/oyatie/docs/decisions/ADR-0706-observability-live-apex.md
3. /Users/jasonlee/oyatie/docs/standards/cross-microservice-latency-budget.md
4. /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.2 engineering-rigor dimensions
5. /Users/jasonlee/oyatie/microservices/mail/PRD.md + /Users/jasonlee/oyatie/microservices/comms-email/PRD.md

DELIVERABLE:
/Users/jasonlee/oyatie/microservices/mail/cross-microservice-handoffs.md
Line floor: >=300 lines for a full handoff rewrite or >=120 lines for a single handoff section.
Substance bar: producer, consumer, contract, retry owner, idempotency owner, p95/p99 hop budget, audit events, failure modes, and compensation.

HALT-CLEANLY:
If producer and consumer both claim the same state transition, classify as hard contradiction and stop.
```

## §6 Anti-patterns to Avoid

The anti-patterns in this section are drawn from the 2026-05-20 feedback
directives, ADR-0324, and the standards anti-pattern catalog.

They are mandatory review checks for every future brief.

### §6.1 Scaffold without substance

Pattern: the artifact has frontmatter, headings, and a section count, but the
body does not let an intern build, operate, audit, or migrate anything.

Why it fails: documentation-rigor §1.1 requires cold intern-buildability and
hyperscaler-grade mechanics.

Brief prevention: require exact APIs, policies, SLOs, failure modes, and
references in the deliverable line.

Review cue: delete the headings and ask what remains that is unique.

### §6.2 Line count as completion

Pattern: the agent proves only `wc -l` and declares done.

Why it fails: the verification feedback directive explicitly rejects line count
as a proxy for deliverable quality.

Brief prevention: require line count plus manual anchor cross-check plus
substance checklist.

Review cue: read three random sections and require each to cite a named anchor or
describe a buildable mechanic.

### §6.3 Cross-vendor variable swap

Pattern: vendor dossiers differ only by vendor name, category, destination, or
tier.

Why it fails: a Salesforce migration, ServiceNow migration, Workday migration,
and Databricks migration have different APIs, object models, auth semantics,
limits, and rollback paths.

Brief prevention: require vendor-specific endpoint, object, auth challenge,
rate-limit behavior, workflow template, permit verbs, and failure mode.

Review cue: if the dossier still makes sense after replacing the vendor name
with another vendor, it is not substance-bar.

### §6.4 Recycled boilerplate per microservice

Pattern: each microservice doc claims the service owns generic intake, policy,
audit, notification, and closure without naming service-local objects.

Why it fails: microservice ownership coherence requires one agent to understand
the actual service boundary.

Brief prevention: require service-local PRD and architecture anchors, top-three
counterpart parity, and internal contradiction classification.

Review cue: ask which exact event, route, policy, dashboard, SLO, and failure
branch belongs to this service only.

### §6.5 Scripted substantive bodies

Pattern: a shell loop, jq expression, Python script, Node script, or template
engine writes the prose body of content artifacts.

Why it fails: ADR-0324 forbids scripting or metaprogramming substantive content,
even when the generated text might appear clever.

Brief prevention: include `NO SCRIPTING`, `NO METAPROGRAMMING`, and `NO
TEMPLATE-SUBSTITUTION` in the forbidden-patterns section.

Review cue: inspect provenance, command logs, and body similarity.

### §6.6 Clause-loop padding

Pattern: a document repeats "Thesis clause N" or "Problem clause N" with minor
word changes to reach a quota.

Why it fails: the artifact has token mass without new decisions.

Brief prevention: require every paragraph to introduce a decision, example,
failure branch, metric, policy, or citation.

Review cue: collapse repeated clauses; if the meaning survives unchanged, the
artifact is padded.

### §6.7 Parent table-of-contents pretending to be doctrine

Pattern: a parent ADR or standard lists child artifacts but makes no parent-level
decision.

Why it fails: downstream agents cannot infer sequencing, precedence, or
acceptance criteria from a list.

Brief prevention: require decision, mechanics, consequences, and stop rules in
the parent artifact.

Review cue: ask what changes if the parent document is deleted.

### §6.8 Soft contradiction left unclassified

Pattern: an audit notes a missing detail but does not distinguish silence from
actual disagreement.

Why it fails: remediation priority becomes noisy.

Brief prevention: require the hard-versus-soft decision tree.

Review cue: identify the two statements; if only one statement exists, it is a
soft gap.

### §6.9 Hard contradiction normalized as style

Pattern: two authority-tier docs disagree, and the agent writes prose that tries
to make both sound acceptable.

Why it fails: builders need one source of truth.

Brief prevention: require HALT-CLEANLY on unresolved hard contradictions.

Review cue: look for "depending on context" language where the context is not
actually specified by an ADR or spec.

### §6.10 Missing VCS lifecycle

Pattern: an agent writes a deliverable without claim, verify, done, and promote.

Why it fails: the corpus loses ownership and promotion evidence.

Brief prevention: include lifecycle commands with the exact agent id, intent,
bundle, and path.

Review cue: VCS evidence must exist before a completion claim.

## §7 References

Primary realignment backbone:

- `/Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md`
- `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md`
- `/Users/jasonlee/oyatie/specs/master-plan-sequencing.json`
- `/Users/jasonlee/oyatie/.omc/plans/realign-oyatie-corpus-plan-2026-05-20.md`

Substance and anti-pattern authority:

- `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1`
- `/Users/jasonlee/oyatie/docs/decisions/ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md`
- `/Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md`
- `/Users/jasonlee/oyatie/docs/standards/anti-patterns.md`

Feedback anchors required by future brief headers:

- `/Users/jasonlee/oyatie/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md`
- `/Users/jasonlee/oyatie/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md`
- `feedback_docs_substance_not_scaffold_2026_05_20.md` as cited by ADR-0322, ADR-0323, ADR-0324, and the realignment spec.

Root agent and VCS authority:

- `/Users/jasonlee/oyatie/specs/root-hub-pointers.json`
- `/Users/jasonlee/oyatie/docs/AGENTS.md`
- `/Users/jasonlee/oyatie/docs/decisions/ADR-0709-general-live-apex.md`

Protocol reminder:

- Claim before editing.
- Author directly.
- Verify file existence, line floor, anchor count, and bespoke substance.
- Mark done only after verification.
- Promote only after done.
- HALT-CLEANLY rather than fabricate, pad, script, or template-substitute.
