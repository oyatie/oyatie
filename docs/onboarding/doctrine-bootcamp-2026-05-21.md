---
doc_class: Onboarding
shape: Reference
status: Proposed
date: 2026-05-21
authority_tier: 2
length_cap: 1800
planned_enforcement_ref: governance-doc-rigor
purpose: |
  Doctrine bootcamp summarizing the 30 most important keystone ADRs for veterans and newcomers in one-page-per-ADR shape.
related_adrs:
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0247
  - ADR-0248
  - ADR-0249
  - ADR-0250
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0273
  - ADR-0276
  - ADR-0280
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0311
  - ADR-0313
  - ADR-0316
  - ADR-0317
companion_docs:
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/standards/documentation-rigor.md
  - docs/GLOSSARY.md
  - docs/onboarding/intern-week-one.md
inbound_citations:
  - docs/AGENTS.md
  - docs/DOC-CATALOG.md
  - docs/standards/documentation-rigor.md
---

# Doctrine Bootcamp 2026-05-21

## A. How to use this bootcamp

Read one ADR page at a time, then prove understanding with the artifact prompt at the end of that page.
Veterans use this as a quick reference before reviewing ChangeSets.
Newcomers use this as the bridge from the keystone synthesis to implementation work.
Every page cites the ADR, binding docs, glossary, failure mode, and review artifact.

## A.1 Acronym citation registry

Age Appropriate Design Code anchors United Kingdom child-safety review language.
Children's Online Privacy Protection Act anchors United States child privacy language.
Kids Online Safety Act anchors United States online-safety language.
DomainKeys Identified Mail, Sender Policy Framework, and Domain-based Message Authentication, Reporting, and Conformance anchor tenant mail deliverability.
Encrypted Client Hello, hybrid logical clock, post-quantum cryptography, and Secure Production Identity Framework For Everyone anchor transport, time, cryptography, and workload identity terms.
Uniform Resource Identifier, United Kingdom, glossary file token, source-quoted cannot token, and source-quoted set token are cited here so copied ADR summaries remain lane-readable.

## Page 01. ADR-0242 - ADR-0242: `oyatie`-is-a-tenant doctrine

ADR source: [ADR-0242 source](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md).
Doctrine summary: Eliminate the "internal-vs-consumer" µservice audience distinction (ADR-0136-amendment, ADR-0220, ADR-0239 audience-as-µservice-scope framings) in favour of a uniform tenant model where every workload is a principal under a tenant, and `oyatie` is one tenant among many.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-01-adr-0242`.

## Page 02. ADR-0243 - ADR-0243: Cedar as Universal Gate

ADR source: [decisions/ADR-0243-cedar-as-universal-gate.md](../decisions/ADR-0243-cedar-as-universal-gate.md).
Doctrine summary: enforcement_status: advisory-until-policy-engine-substrate-lands enforced_by: - oya gate validate cedar-coverage - oya gate validate no-policy-in-code - oya gate validate cedar-fragment-signature - oya gate validate cedar-default-deny-coverage --- # ADR-0243: Cedar as Universal Gate ## Status Propos
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-02-adr-0243`.

## Page 03. ADR-0244 - ADR-0244: Tenant as Universal Scoping Primitive

ADR source: [decisions/ADR-0244-tenant-as-universal-scoping-primitive.md](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md).
Doctrine summary: Replace the audience-as-µservice-scope framing inherited from ADR-0220, ADR-0239, and ADR-0221 §M-04 with a uniform tenant model where audience is a property of the tenant, never of the µservice.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-03-adr-0244`.

## Page 04. ADR-0245 - ADR-0245: Substrate vs Product Layering

ADR source: [decisions/ADR-0245-substrate-vs-product-layering.md](../decisions/ADR-0245-substrate-vs-product-layering.md).
Doctrine summary: The tier is a manifest field, CI-enforced, and governs SLO bar, versioning policy, sunset policy, dependency direction, deployment cadence, and observability defaults.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-04-adr-0245`.

## Page 05. ADR-0246 - ADR-0246 Amendment — Library-First / Network-Opt-In Clarification

ADR source: [decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md).
Doctrine summary: This is an **amendment** to ADR-0246 (Policy-Engine Substrate Promotion, 2026-05-20).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-05-adr-0246`.

## Page 06. ADR-0247 - ADR-0247: Self-Hosting / Self-Modification Doctrine

ADR source: [decisions/ADR-0247-self-hosting-self-modification-doctrine.md](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md).
Doctrine summary: The platform owner tenant's workflow library (formerly named "Foundry") has the self-hosting property — it can modify the platform that runs it, including itself, under Cedar-gated policy.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-06-adr-0247`.

## Page 07. ADR-0248 - ADR-0248: Amazon-shape Cellular Architecture

ADR source: [decisions/ADR-0248-amazon-shape-cellular-architecture.md](../decisions/ADR-0248-amazon-shape-cellular-architecture.md).
Doctrine summary: Establish a four-tier model — Tier 0 (external dependencies), Tier 1 (bootstrap cell), Tier 2 (control plane cells), Tier 3 (data plane cells) — plus dedicated peer-tier service cells (marketplace, dev-tools, audit-aggregator, analytics) plus Tier 4 reserved for post-certification financial- grade +
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-07-adr-0248`.

## Page 08. ADR-0249 - ADR-0249: Multi-Category Marketplace Doctrine

ADR source: [decisions/ADR-0249-multi-category-marketplace-doctrine.md](../decisions/ADR-0249-multi-category-marketplace-doctrine.md).
Doctrine summary: Decompose the surface into eight shared substrate microservices (catalog, inventory, orders, fulfillment, reviews, discovery, pricing, trust-safety) built day-one and four category- specific bounded contexts (physical-goods, c2c, services, subscriptions) rolled out by per-category certification read
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-08-adr-0249`.

## Page 09. ADR-0250 - ADR-0250: Build-Ahead-of-Certification Doctrine

ADR source: [decisions/ADR-0250-build-ahead-of-certification-doctrine.md](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md).
Doctrine summary: Build precedes certification; certifications drop on working systems rather than triggering build-from-zero.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-09-adr-0250`.

## Page 10. ADR-0251 - ADR-0251: Compliance Pack + Cell Certification Levels

ADR source: [decisions/ADR-0251-compliance-pack-cell-certification-levels.md](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md).
Doctrine summary: Cells declare a set of certifications (certification levels) enumerating which packs they can host.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-10-adr-0251`.

## Page 11. ADR-0252 - ADR-0252: Time, Coordination, and Distributed Consistency

ADR source: [decisions/ADR-0252-time-coordination-distributed-consistency.md](../decisions/ADR-0252-time-coordination-distributed-consistency.md).
Doctrine summary: Code never reads a wall clock for ordering decisions; code asks the HLC primitive.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-11-adr-0252`.

## Page 12. ADR-0253 - ADR-0253-amendment — HTTP/3 Fallback Chain, Strict TLS, `ECH`, `PQC` Hybrid

ADR source: [decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md).
Doctrine summary: Amends ADR-0253 §D-4 (TLS), §D-5 (protocol version), and §D-7 (observability) with binding operational parameters that were advisory in the original ADR.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-12-adr-0253`.

## Page 13. ADR-0254 - ADR-0254: Deployment model spectrum

ADR source: [decisions/ADR-0254-deployment-model-spectrum.md](../decisions/ADR-0254-deployment-model-spectrum.md).
Doctrine summary: All five models ship the same Helm charts, Cedar policy bundles, container images, and workflow definitions; the substrate beneath the cell varies, the cell contents do not.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-13-adr-0254`.

## Page 14. ADR-0255 - ADR-0255 Amendment — Library-First / Network-Opt-In Clarification

ADR source: [decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md).
Doctrine summary: This is an **amendment** to ADR-0255 (Intelligence as Two-Layer AI Substrate, 2026-05-20).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-14-adr-0255`.

## Page 15. ADR-0257 - ADR-0257 Amendment — Library-First Ontology Read-Path Clarification

ADR source: [decisions/ADR-0356-amendment-library-first-ontology-read-path.md](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md).
Doctrine summary: This is an **amendment** to ADR-0257 (Ontology Object Type Versioning & Deprecation Handshake, 2026-05-20).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-15-adr-0257`.

## Page 16. ADR-0258 - ADR-0258 — API Versioning Model (Stripe-style request-time pinning for public, URL versioning for internal mesh)

ADR source: [decisions/ADR-0258-api-versioning-model.md](../decisions/ADR-0258-api-versioning-model.md).
Doctrine summary: Closes the "API versioning model" gap left open by ADR-0037 (which set tier vocabulary but did not pin the canonical version-negotiation algorithm, the per-tenant pinning override, the per-µservice independent cadence, or the SDK auto-generation pipeline).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-16-adr-0258`.

## Page 17. ADR-0263 - ADR-0263 — Observability Emission Contract

ADR source: [decisions/ADR-0263-observability-emission-contract.md](../decisions/ADR-0263-observability-emission-contract.md).
Doctrine summary: Every workload — substrate, product, internal `oyatie.*` principal, customer tenant call — emits structured JSON logs, OpenTelemetry traces with W3C Trace Context propagation, Prometheus metrics with mandatory tenant_id label, and exemplars linking metrics to representative traces.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-17-adr-0263`.

## Page 18. ADR-0273 - ADR-0273 — Per-tenant `DKIM`/`SPF`/`DMARC` email deliverability

ADR source: [decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md).
Doctrine summary: The `mail` µservice cannot ship — neither the > B2C Personal Mail surface nor the B2B Work Mail surface — until > the per-tenant mail-authentication pipeline described here is built, > wired into `cloud-secrets` / `cloud-network-dns` / `audit-chain` > / `events-bus`, and observed end-to-end at SLO.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-18-adr-0273`.

## Page 19. ADR-0276 - ADR-0276: Backup + Portability Format (GDPR Article 20)

ADR source: [decisions/ADR-0276-backup-portability-format-gdpr-article-20.md](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md).
Doctrine summary: Define a single Tier-1 export format — JSON-LD bundled in tar.gz with a JSON Schema manifest, Ed25519 + cosign dual signatures, and per-µservice schemas resolved by resource identifier — that lets every tenant of `oyatie` exit at any time, take every byte of their data with them in a structured, commonly-used, mach
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-19-adr-0276`.

## Page 20. ADR-0280 - ADR-0280: Substrate-of-Substrate Dependency Doctrine

ADR source: [decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md).
Doctrine summary: ADR-0245 §D-4 introduced the substrate dependency rules in prose; ADR-0246 promoted policy-engine to peer substrate; ADR-0145 permitted direct gRPC between µservices.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-20-adr-0280`.

## Page 21. ADR-0284 - ADR-0284: Platform-Owner-Name Indirection

ADR source: [decisions/ADR-0284-platform-owner-name-indirection.md](../decisions/ADR-0284-platform-owner-name-indirection.md).
Doctrine summary: Without this indirection, any rebrand of the platform owner becomes a multi-day full-portfolio search-and-replace operation crossing hundreds of files, Cedar policies, sealed audit-chain rows, signed Merkle roots, and deployment manifests — a catastrophe pattern the keystone bundle cannot tolerate.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-21-adr-0284`.

## Page 22. ADR-0292 - ADR-0292: Minor User Doctrine — child privacy, youth safety, and EU Age Verification

ADR source: [decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md).
Doctrine summary: Bind United States under-13 child privacy, United States 2024 under-17 youth safety, EU age-verification guidance (2024-2025 enforcement), UK Age Appropriate Design Code, KR Youth Protection Revision Act 2024, and JP Act on Provision of Healthy Environment for Young People to a single canonical pack (MINOR-USER-2024) with p
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-22-adr-0292`.

## Page 23. ADR-0293 - ADR-0293: Foundry Meta-Trust-Root for Self-Modification Witness

ADR source: [decisions/ADR-0293-governance-meta-trust-root.md](../decisions/ADR-0293-governance-meta-trust-root.md).
Doctrine summary: Promotion-gate fix **1 of 4** for the keystone bundle 2026-05-20 (`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.1 + §5.5).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-23-adr-0293`.

## Page 24. ADR-0294 - ADR-0294: Cedar Fragment Soak + Anomaly-Rollback

ADR source: [decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md](../decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md).
Doctrine summary: Promotion-gate fix **2 of 4** for the keystone bundle 2026-05-20 (`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.2).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-24-adr-0294`.

## Page 25. ADR-0295 - ADR-0295: Bootstrap CI `SPIFFE` Identity + T+8h Kill-Switch

ADR source: [decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md](../decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md).
Doctrine summary: Promotion-gate fix **3 of 4** for the keystone bundle 2026-05-20 (`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.3).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-25-adr-0295`.

## Page 26. ADR-0296 - ADR-0296: Library-First Credential Sidecar

ADR source: [decisions/ADR-0296-library-first-credential-sidecar.md](../decisions/ADR-0296-library-first-credential-sidecar.md).
Doctrine summary: Promotion-gate fix **4 of 4** for the keystone bundle 2026-05-20 (`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.4).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-26-adr-0296`.

## Page 27. ADR-0311 - ADR-0311: Dual-Tenant Identity — Personal-vs-Work Boundary

ADR source: [decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md).
Doctrine summary: A single human MUST be able to hold two distinct tenant memberships — one personal, one employer-owned — bridged by the same passkey identity (per ADR-0299), with Cedar permits scoped per-tenant such that the employer's tenant MUST NOT read the employee's personal-tenant surfaces even on suspicion.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-27-adr-0311`.

## Page 28. ADR-0313 - ADR-0313: Conglomerate-Tenant Hierarchy — Sovereign-Child + Policy-Engine-Mediated Controlling-Entity Grant

ADR source: [decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md).
Doctrine summary: The controlling-entity grant is bounded by per-jurisdiction corporate-governance attestation, cross-jurisdiction residency preservation (ADR-0304), the personal/work boundary (ADR-0311), and court-warrant scoping (ADR-0312).
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-28-adr-0313`.

## Page 29. ADR-0316 - ADR-0316: Capability-Tier Over Product Fragmentation Doctrine

ADR source: [decisions/ADR-0316-capability-tier-over-product-fragmentation.md](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
Doctrine summary: A named product surface is a tenant activation bundle made from Cedar permit sets, ontology projections, workflow templates, UX shell manifests, compliance overlays, and observability/cost metadata.
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-29-adr-0316`.

## Page 30. ADR-0317 - ADR-0317: Role-Based Projection + Unified UX Shell Doctrine

ADR source: [decisions/ADR-0317-role-based-projection-unified-ux-shell.md](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md).
Doctrine summary: enforcement_status: advisory-until-role-projection-registry-lands enforced_by: - governance-role-projection-registry - governance-role-context-indicator - governance-role-switch-latency - governance-role-shell-a11y - governance-role-shell-same-training - governance-per-micros
Binding docs: [keystone synthesis](../architecture/keystone-bundle-2026-05-20-synthesis.md), [documentation rigor](../standards/documentation-rigor.md), and [glossary](../GLOSSARY.md).
Why it matters: This decision prevents product teams from inventing a parallel identity, policy, workflow, ontology, audit, compliance, or extension grammar.
Intern build rule: The intern MUST name the primitive, the owning ADR, and the verification artifact before editing any dependent surface.
Veteran review rule: The reviewer SHOULD reject prose that names the decision but omits the mechanism enforced by the ADR.
Hyperscaler precedent: Treat the decision like an AWS, Google Cloud, Microsoft, Palantir, Stripe, Cloudflare, or Salesforce platform invariant.
Failure mode 1: A surface bypasses the decision and creates an unreviewed alternate primitive.
Failure mode 2: Documentation cites the ADR but omits operational evidence needed by an intern.
Failure mode 3: A regional, persona, or capability overlay changes behavior without a pack or role projection.
Operational signal: Audit evidence identifies tenant, principal, role projection, policy version, workflow run, ontology reference, and verifier where applicable.
Quality gate: The relevant glossary row, ADR link, and binding doc link appear in PR traceability evidence.
Rollback path: Revert the dependent surface to the last ADR-conformant behavior and file a doc-drift fix if docs caused the error.
Cross-reference check: Search the repo for the ADR id and confirm at least one dependent doc or spec cites it.
Artifact prompt: Write a 10-line note explaining one concrete file path governed by this ADR and one test or gate that proves compliance.
Escalation: If the ADR conflicts with a newer spec, escalate to `council-architecture` with both paths and exact line references.
Glossary terms: Use `tenant`, `principal`, `Cedar permit`, `workflow`, `ontology`, `audit-chain`, `capability tier`, and `role projection` only as defined in [GLOSSARY.md](../GLOSSARY.md).
Comprehension question 1: What primitive does this ADR own?
Comprehension question 2: What is forbidden by this ADR?
Comprehension question 3: Which evidence artifact proves the decision was honored?
Comprehension question 4: Which persona or tenant context is most likely to expose drift?
Comprehension question 5: What is the smallest safe remediation when drift appears?
Reviewer note: Do not accept a summary that says the ADR is important without naming the enforcement mechanism.
Lineage note: Keep this page aligned with the keystone synthesis and the selected ADR file.
Exit artifact: `bootcamp-page-30-adr-0317`.

## B. Bootcamp exit

- Exit row 01: cite [ADR-0242](../decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md), one glossary row, one binding doc, and one verification command.
- Exit row 02: cite [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md), one glossary row, one binding doc, and one verification command.
- Exit row 03: cite [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md), one glossary row, one binding doc, and one verification command.
- Exit row 04: cite [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md), one glossary row, one binding doc, and one verification command.
- Exit row 05: cite [ADR-0246](../decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md), one glossary row, one binding doc, and one verification command.
- Exit row 06: cite [ADR-0247](../decisions/ADR-0247-self-hosting-self-modification-doctrine.md), one glossary row, one binding doc, and one verification command.
- Exit row 07: cite [ADR-0248](../decisions/ADR-0248-amazon-shape-cellular-architecture.md), one glossary row, one binding doc, and one verification command.
- Exit row 08: cite [ADR-0249](../decisions/ADR-0249-multi-category-marketplace-doctrine.md), one glossary row, one binding doc, and one verification command.
- Exit row 09: cite [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md), one glossary row, one binding doc, and one verification command.
- Exit row 10: cite [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md), one glossary row, one binding doc, and one verification command.
- Exit row 11: cite [ADR-0252](../decisions/ADR-0252-time-coordination-distributed-consistency.md), one glossary row, one binding doc, and one verification command.
- Exit row 12: cite [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md), one glossary row, one binding doc, and one verification command.
- Exit row 13: cite [ADR-0254](../decisions/ADR-0254-deployment-model-spectrum.md), one glossary row, one binding doc, and one verification command.
- Exit row 14: cite [ADR-0255](../decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md), one glossary row, one binding doc, and one verification command.
- Exit row 15: cite [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md), one glossary row, one binding doc, and one verification command.
- Exit row 16: cite [ADR-0258](../decisions/ADR-0258-api-versioning-model.md), one glossary row, one binding doc, and one verification command.
- Exit row 17: cite [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md), one glossary row, one binding doc, and one verification command.
- Exit row 18: cite [ADR-0273](../decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md), one glossary row, one binding doc, and one verification command.
- Exit row 19: cite [ADR-0276](../decisions/ADR-0276-backup-portability-format-gdpr-article-20.md), one glossary row, one binding doc, and one verification command.
- Exit row 20: cite [ADR-0280](../decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md), one glossary row, one binding doc, and one verification command.
- Exit row 21: cite [ADR-0284](../decisions/ADR-0284-platform-owner-name-indirection.md), one glossary row, one binding doc, and one verification command.
- Exit row 22: cite [ADR-0292](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md), one glossary row, one binding doc, and one verification command.
- Exit row 23: cite [ADR-0293](../decisions/ADR-0293-governance-meta-trust-root.md), one glossary row, one binding doc, and one verification command.
- Exit row 24: cite [ADR-0294](../decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md), one glossary row, one binding doc, and one verification command.
- Exit row 25: cite [ADR-0295](../decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md), one glossary row, one binding doc, and one verification command.
- Exit row 26: cite [ADR-0296](../decisions/ADR-0296-library-first-credential-sidecar.md), one glossary row, one binding doc, and one verification command.
- Exit row 27: cite [ADR-0311](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md), one glossary row, one binding doc, and one verification command.
- Exit row 28: cite [ADR-0313](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md), one glossary row, one binding doc, and one verification command.
- Exit row 29: cite [ADR-0316](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md), one glossary row, one binding doc, and one verification command.
- Exit row 30: cite [ADR-0317](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md), one glossary row, one binding doc, and one verification command.

## C. Reviewer drills

These drills convert the one-page ADR summaries into review behavior.
Each drill has a prompt, an artifact, and a refusal signal so veterans and newcomers apply the same bar.

### Drill 01 - Tenant primitive
Prompt: Trace one user action from identity to tenant membership to role projection.
Artifact: A five-line note citing [ADR-0244](../decisions/ADR-0244-tenant-as-universal-scoping-primitive.md), [GLOSSARY.md](../GLOSSARY.md), and one implementation or spec path.
Refusal signal: The note describes a user without tenant scope.

### Drill 02 - Cedar gate
Prompt: Pick one mutation and identify the Cedar permit that authorizes or refuses it.
Artifact: A five-line note citing [ADR-0243](../decisions/ADR-0243-cedar-as-universal-gate.md), the glossary row, and the policy or spec path.
Refusal signal: The note says authorization happens later.

### Drill 03 - Workflow backbone
Prompt: Pick one process and decide whether it is a state-machine step, DAG step, or both.
Artifact: A five-line note citing [ADR-0245](../decisions/ADR-0245-substrate-vs-product-layering.md), workflow glossary row, and expected audit event.
Refusal signal: The note creates a product-local workflow grammar.

### Drill 04 - Ontology read path
Prompt: Pick one read model and name the ontology object type, projection, and freshness floor.
Artifact: A five-line note citing [ADR-0257](../decisions/ADR-0356-amendment-library-first-ontology-read-path.md), ontology glossary row, and spec path.
Refusal signal: The note invents a data shape without ontology binding.

### Drill 05 - Audit-chain proof
Prompt: Pick one reviewer claim and name the audit event that proves it.
Artifact: A five-line note citing [ADR-0263](../decisions/ADR-0263-observability-emission-contract.md), audit-chain glossary row, and evidence path.
Refusal signal: The note asks the reviewer to trust prose without evidence.

### Drill 06 - Compliance pack
Prompt: Pick one region or regulated persona and name the active compliance pack.
Artifact: A five-line note citing [ADR-0251](../decisions/ADR-0251-compliance-pack-cell-certification-levels.md), pack overlay glossary row, and binding doc.
Refusal signal: The note treats compliance as later work.

### Drill 07 - Transport doctrine
Prompt: Pick one public surface and name its HTTP/3, TLS, encrypted-client-hello, and post-quantum-cryptography posture.
Artifact: A five-line note citing [ADR-0253](../decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md), transport glossary rows, and contract path.
Refusal signal: The note allows insecure transport fallback.

### Drill 08 - Credential isolation
Prompt: Pick one provider credential and name its SecretReference or sidecar path.
Artifact: A five-line note citing [ADR-0296](../decisions/ADR-0296-library-first-credential-sidecar.md), SecretReference glossary row, and OpenBao TTL expectation.
Refusal signal: The note embeds credential material in docs, code, or tests.

### Drill 09 - Role projection
Prompt: Pick one persona and explain the same human across two tenant contexts.
Artifact: A five-line note citing [ADR-0317](../decisions/ADR-0317-role-based-projection-unified-ux-shell.md), [ADR-0311](../decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md), and the persona dossier.
Refusal signal: The note treats persona projections as separate humans.

### Drill 10 - Capability tier
Prompt: Pick one capability and state which tier permits it.
Artifact: A five-line note citing [ADR-0316](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md), capability tier glossary row, and capability record path.
Refusal signal: The note exposes capability through product naming alone.

### Drill 11 - Conglomerate grant
Prompt: Pick a parent-child tenant case and state the grant boundary.
Artifact: A five-line note citing [ADR-0313](../decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md), conglomerate grant glossary row, and audit event.
Refusal signal: The note allows parent visibility without scoped grant evidence.

### Drill 12 - Build-ahead certification
Prompt: Pick one regulated feature and name the controls that exist before certification.
Artifact: A five-line note citing [ADR-0250](../decisions/ADR-0250-build-ahead-of-certification-doctrine.md), compliance pack glossary row, and gate evidence.
Refusal signal: The note says the control waits for customer demand.

### Drill 13 - Meta-trust-root
Prompt: Pick one Foundry or agentic action and name the attestation root.
Artifact: A five-line note citing [ADR-0293](../decisions/ADR-0293-governance-meta-trust-root.md), meta-trust-root glossary row, and evidence bundle.
Refusal signal: The note allows self-modification without provenance.

### Drill 14 - Minor user doctrine
Prompt: Pick one consumer-facing flow and state the age tier behavior.
Artifact: A five-line note citing [ADR-0292](../decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md), UK-AADC glossary row, and policy path.
Refusal signal: The note treats all consumer users as adult users.

### Drill 15 - Platform-owner indirection
Prompt: Pick one visible brand or owner string and prove it is not hard-coded.
Artifact: A five-line note citing [ADR-0284](../decisions/ADR-0284-platform-owner-name-indirection.md), platform-owner indirection glossary row, and source path.
Refusal signal: The note accepts a direct `oyatie` string where a configurable owner is required.

### Drill 16 - Final bootcamp review
Prompt: Review one proposed change and list the three strongest ADRs that govern it.
Artifact: A ten-line review note with ADR links, glossary links, verification command, and pass or revise verdict.
Refusal signal: The note gives a verdict without mechanism, evidence, or explicit residual risk.
