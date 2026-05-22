# Ontology Feature-Parity Matrix

- Microservice: `ontology`
- Audit date: 2026-05-20
- Batch: Wave 3 Batch 3.2
- Required counterpart set: Palantir Foundry Ontology, Microsoft Dataverse, Salesforce Data Cloud
- Retired deliverable note: capability split-by-business-level delta output is intentionally omitted for this batch.
- Current Oyatie artifact base: `microservices/ontology/`
- Output scope: feature parity, union coverage, family summary, headline gap analysis, additive surface.

## Source Anchor Block

1. Palantir Foundry Ontology resources: https://www.palantir.com/docs/foundry/ontologies/ontologies-overview
2. Palantir action types and limits: https://www.palantir.com/docs/foundry/action-types/scale-property-limits/
3. Microsoft Dataverse custom APIs: https://learn.microsoft.com/en-us/powerapps/developer/data-platform/custom-api
4. Microsoft Dataverse service protection API limits: https://learn.microsoft.com/en-us/power-apps/developer/data-platform/api-limits
5. Salesforce Data Cloud object model: https://developer.salesforce.com/docs/data/data-cloud-dev/guide/dc-object-model.html
6. Salesforce Data Cloud ingestion API: https://developer.salesforce.com/docs/data/data-cloud-int/references/data-cloud-ingestionapi-ref/c360-a-api-get-started.html
7. Salesforce Data Cloud query API: https://developer.salesforce.com/docs/data/data-cloud-query-guide/references/data-cloud-query-api-reference/c360a-api-query.html
8. Oyatie ontology PRD purpose and requirements: `microservices/ontology/PRD.md:91-119`, `microservices/ontology/PRD.md:641-729`
9. Oyatie ontology architecture purpose and boundaries: `microservices/ontology/ARCHITECTURE.md:52-63`
10. Oyatie ontology contract inventory: `microservices/ontology/contracts/openapi/ontology.yaml:247-520`, `microservices/ontology/contracts/proto/ontology.proto:192-228`, `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85`

## Method

1. I treated the three named counterpart products as the required union-coverage bar.
2. I compared product-facing capabilities, control-plane capabilities, contract surfaces, operational features, and integration features.
3. I used local artifact citations for Oyatie coverage claims.
4. I used official counterpart documentation URLs for public counterpart surface claims.
5. I classified each capability as `covered`, `partial`, `absent`, or `owned-elsewhere`.
6. `covered` means current ontology artifacts define the feature with enough detail to implement or verify it.
7. `partial` means the feature exists in concept but misses contract, operational, source, deployment, or acceptance evidence.
8. `absent` means the audit found no current artifact evidence under `microservices/ontology/`.
9. `owned-elsewhere` means the capability is important to parity but should be provided by another Oyatie microservice with an explicit handoff.
10. I did not create a fourth retired delta deliverable.
11. I did not introduce new split-by-business-level feature headings.
12. I used tenant classes only where the current canonical doctrine requires them.
13. I kept all recommendations scoped to `microservices/ontology/`.
14. I treated product UI parity as out of ontology unless the data/model substrate requires an API, event, or policy hook.
15. I treated generated SDK ambitions as contract evidence only because the current microservice has no source tree.

## Counterpart 1: Palantir Foundry Ontology Capability Surface

1. Surface PFO-01: ontology resources include object types, link types, action types, interfaces, shared properties, and object type groups.
2. Source: Palantir ontology overview official documentation.
3. Oyatie evidence: README names ontology as the Palantir Foundry Ontology equivalent and lists canonical object-type registry, typed read substrate, Cedar writes, and ClickHouse projections: `microservices/ontology/README.md:17-21`.
4. Oyatie status: partial.
5. Gap: object types, relationships, actions, function types, and policy concepts exist, but interfaces, shared properties, and object type groups are not fully first-class in the current OpenAPI/proto contracts.
6. Repair: add explicit interface, shared-property, and object-type-group resources to contracts or document why they map to existing type definitions.
7. Surface PFO-02: ontology maps business concepts to operational data and makes that model usable by applications.
8. Source: Palantir ontology overview and core concept documentation.
9. Oyatie evidence: PRD says the service owns object types, relationship types, action schemas, function signatures, policy predicates, versioned schemas, and lineage metadata: `microservices/ontology/PRD.md:91-119`.
10. Oyatie status: covered for documented purpose, partial for executable evidence.
11. Gap: there is no `src/` or `tests/` tree to prove runtime behavior.
12. Repair: add Rust service implementation and contract tests or downgrade status language to documentation-only.
13. Surface PFO-03: object types have properties, backing data, and permissions.
14. Source: Palantir object and link type documentation.
15. Oyatie evidence: OpenAPI object-type endpoints define create/list/read shape: `microservices/ontology/contracts/openapi/ontology.yaml:247-288`.
16. Oyatie evidence: proto service defines `ObjectTypeService`: `microservices/ontology/contracts/proto/ontology.proto:192-201`.
17. Oyatie status: partial.
18. Gap: property semantics include a `PropertySensitivityLevel` vocabulary in proto and `property_sensitivity_level` in OpenAPI that should be renamed during doctrine cleanup: `microservices/ontology/contracts/proto/ontology.proto:31-37`, `microservices/ontology/contracts/openapi/ontology.yaml:90-100`.
19. Repair: replace sensitivity vocabulary with a non-business-level name and align OpenAPI/proto field names.
20. Surface PFO-04: link types model relationships between object types.
21. Source: Palantir ontology resources official documentation.
22. Oyatie evidence: OpenAPI relationship endpoints exist: `microservices/ontology/contracts/openapi/ontology.yaml:289-335`.
23. Oyatie evidence: proto service defines `RelationshipTypeService`: `microservices/ontology/contracts/proto/ontology.proto:203-210`.
24. Oyatie status: covered for contract presence.
25. Gap: no executable tests show one-to-one, one-to-many, many-to-many, temporal, or polymorphic relationship behavior.
26. Repair: add relationship contract examples and property-based tests.
27. Surface PFO-05: action types define transactional edits with user-facing side effects.
28. Source: Palantir action types overview and scale/property limit docs.
29. Oyatie evidence: PRD functional requirements include action type registry and Cedar-governed mutation: `microservices/ontology/PRD.md:653-662`.
30. Oyatie evidence: OpenAPI action endpoints exist: `microservices/ontology/contracts/openapi/ontology.yaml:336-383`.
31. Oyatie evidence: proto service defines `ActionTypeService`: `microservices/ontology/contracts/proto/ontology.proto:212-219`.
32. Oyatie status: partial.
33. Gap: contracts define action types, but current artifacts do not show action execution transactional semantics, side-effect events, or rollback behavior.
34. Repair: bind action schemas to workflow-engine/audit-chain handoffs and define failure modes for partial submission.
35. Surface PFO-06: action limits include 50 object types edited per action, 10,000 objects per action, 10,000 batch calls, and smaller function-backed limits when batching is not configured.
36. Source: Palantir action scale/property limits official documentation.
37. Oyatie evidence: PRD performance targets include write throughput and read throughput but not these action shape caps: `microservices/ontology/PRD.md:735-788`.
38. Oyatie status: absent.
39. Gap: action shape limits are not codified as request validation, performance budget, or abuse-prevention rule.
40. Repair: add action resource limits that meet or beat the public Palantir limits, then test those limits.
41. Surface PFO-07: interfaces abstract common object shape and capabilities across object types.
42. Source: Palantir interface overview official documentation.
43. Oyatie evidence: no `interface` resource appears in the OpenAPI path set; the OpenAPI visible paths are object types, relationship types, action types, function types, policy predicates, and graph traversal: `microservices/ontology/contracts/openapi/ontology.yaml:247-520`.
44. Oyatie status: absent.
45. Gap: without interfaces, Oyatie cannot fully match Palantir interface-driven workflows and generic action behavior.
46. Repair: add `InterfaceType`, interface properties, interface relationship constraints, and implementation links to contracts.
47. Surface PFO-08: shared properties centralize common metadata across object types.
48. Source: Palantir shared property documentation.
49. Oyatie evidence: current contracts define object type property lists but no shared-property catalog: `microservices/ontology/contracts/openapi/ontology.yaml:90-160`.
50. Oyatie status: absent.
51. Gap: shared-property reuse cannot be validated or migrated from Palantir without a first-class resource.
52. Repair: add shared-property catalog plus uniqueness and ownership rules.
53. Surface PFO-09: object type groups organize related types for governance and usability.
54. Source: Palantir ontology overview official documentation.
55. Oyatie evidence: no object type group path or proto service is present: `microservices/ontology/contracts/proto/ontology.proto:192-228`.
56. Oyatie status: absent.
57. Gap: grouping is left to external documentation or tags, which weakens admin-scale parity.
58. Repair: add type-group metadata or explicitly assign this to an application-builder service.
59. Surface PFO-10: Ontology Manager provides governance UI and type lifecycle management.
60. Source: Palantir ontology and interface official docs.
61. Oyatie evidence: PRD scope is service API and registry; no UI source exists under ontology: `microservices/ontology/PRD.md:641-729`.
62. Oyatie status: owned-elsewhere.
63. Gap: ontology must still expose lifecycle APIs for the UI owner to consume.
64. Repair: add handoff to application-builder/admin-console after creating cross-microservice handoff file.
65. Surface PFO-11: object explorer and views use ontology resources for application workflows.
66. Source: Palantir ontology official documentation.
67. Oyatie evidence: README positions ontology as typed read substrate for product surfaces: `microservices/ontology/README.md:17-21`.
68. Oyatie status: partial.
69. Gap: typed read substrate exists in concept, but query/read contracts do not cover object views or saved object sets.
70. Repair: add object set and saved view read resources, or document ownership by another service.
71. Surface PFO-12: functions can be attached to ontology workflows and actions.
72. Source: Palantir action/interface documentation.
73. Oyatie evidence: OpenAPI function-type endpoint exists: `microservices/ontology/contracts/openapi/ontology.yaml:384-430`.
74. Oyatie evidence: PRD includes function signature registry: `microservices/ontology/PRD.md:91-119`.
75. Oyatie status: partial.
76. Gap: runtime function execution, language policy, sandboxing, and workflow handoff are not evidenced.
77. Repair: bind function metadata to workflow-engine and intelligence with Rust-strict backend expectations.
78. Surface PFO-13: permissioning is embedded in ontology resources.
79. Source: Palantir ontology documentation describes private/shared organization-scoped ontologies.
80. Oyatie evidence: README and PRD emphasize Cedar writes and Cedar policy predicates: `microservices/ontology/README.md:17-21`, `microservices/ontology/PRD.md:653-662`.
81. Oyatie evidence: policy predicate endpoints exist: `microservices/ontology/contracts/openapi/ontology.yaml:431-475`.
82. Oyatie status: partial.
83. Gap: current artifacts do not encode tenant-class semantics or foundry principal inheritance.
84. Repair: add tenant_class and `oyatie.foundry.*` policy routing once canonical ownership is confirmed.
85. Surface PFO-14: ontology resources are versioned and proposed before activation.
86. Source: Palantir proposal/change-management model in ontology docs.
87. Oyatie evidence: PRD says versioned schemas and lineage metadata are owned here: `microservices/ontology/PRD.md:91-119`.
88. Oyatie evidence: AsyncAPI emits object type, relationship type, action type, function type, and policy predicate change events: `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85`.
89. Oyatie status: partial.
90. Gap: proposal lifecycle states, review gates, and rollback paths are not contract-complete.
91. Repair: add proposal state machine and event lifecycle.
92. Surface PFO-15: Palantir migration requires preserving business meaning, links, actions, and permissions.
93. Source: Palantir migration comparison is local Oyatie artifact plus Palantir docs.
94. Oyatie evidence: migration playbook exists: `microservices/ontology/migration-playbooks/from-palantir-foundry.md:34-94`.
95. Oyatie status: partial.
96. Gap: the playbook still uses retired business-level vocabulary and old capacity examples.
97. Repair: rewrite migration guidance around tenant classes and deployment contexts.

## Counterpart 2: Microsoft Dataverse Capability Surface

1. Surface DV-01: tables are the primary data model resource.
2. Source: Microsoft Dataverse table and relationship documentation.
3. Oyatie evidence: object types map naturally to Dataverse tables: `microservices/ontology/contracts/openapi/ontology.yaml:247-288`.
4. Oyatie status: partial.
5. Gap: current naming is ontology-first, which is appropriate, but migration docs should map table concepts to object types explicitly.
6. Repair: add a Dataverse migration appendix or crosswalk.
7. Surface DV-02: columns, data types, choices, calculated columns, rollups, and field constraints shape records.
8. Source: Microsoft Dataverse maker/developer documentation.
9. Oyatie evidence: OpenAPI property schema is present inside object type contracts: `microservices/ontology/contracts/openapi/ontology.yaml:90-160`.
10. Oyatie status: partial.
11. Gap: calculated columns and rollup semantics are not first-class.
12. Repair: either add derived-property definitions or assign calculated views to analytics/query services.
13. Surface DV-03: one-to-many and many-to-many relationships are first-class metadata.
14. Source: Microsoft Dataverse relationship documentation.
15. Oyatie evidence: relationship type service exists in OpenAPI and proto: `microservices/ontology/contracts/openapi/ontology.yaml:289-335`, `microservices/ontology/contracts/proto/ontology.proto:203-210`.
16. Oyatie status: partial.
17. Gap: eligibility checks, cascading behaviors, and relationship delete semantics are not fully visible.
18. Repair: add relationship cardinality and cascade fields.
19. Surface DV-04: business rules provide no-code validation and UI behavior.
20. Source: Microsoft business rule documentation.
21. Oyatie evidence: action schemas and policy predicates exist, but no business-rule resource exists: `microservices/ontology/contracts/openapi/ontology.yaml:336-475`.
22. Oyatie status: partial.
23. Gap: policy predicates are security/control semantics, not UI/business validation rules.
24. Repair: decide whether business rules map to ontology constraints, workflow-engine, or application-builder.
25. Surface DV-05: custom APIs allow developers to define actions/functions and expose business events.
26. Source: Microsoft custom API documentation.
27. Oyatie evidence: function type endpoint exists and action type endpoint exists: `microservices/ontology/contracts/openapi/ontology.yaml:336-430`.
28. Oyatie status: partial.
29. Gap: custom API binding types, plug-in step behavior, and event subscription semantics are not mirrored.
30. Repair: add operation-binding metadata and event handoff.
31. Surface DV-06: plug-ins provide synchronous and asynchronous extension points.
32. Source: Microsoft custom API and plug-in documentation.
33. Oyatie evidence: current ontology docs do not define plug-in execution: `microservices/ontology/PRD.md:641-729`.
34. Oyatie status: owned-elsewhere.
35. Gap: ontology should not run arbitrary plug-ins unless a Rust-strict workflow boundary is defined.
36. Repair: model plug-in-like extension as workflow-engine or function metadata.
37. Surface DV-07: solutions package schema, components, and custom APIs for ALM.
38. Source: Microsoft custom API and solution documentation.
39. Oyatie evidence: manifest exists, but there is no solution/package lifecycle contract: `microservices/ontology/manifest.json:1-80`.
40. Oyatie status: partial.
41. Gap: deployable schema bundles and rollback units are not defined.
42. Repair: add ontology package manifest with version and compatibility constraints.
43. Surface DV-08: security roles, teams, business units, and field security control access.
44. Source: Microsoft Dataverse security model documentation.
45. Oyatie evidence: Cedar policies and policy predicate contracts exist: `microservices/ontology/contracts/openapi/ontology.yaml:431-475`.
46. Oyatie status: partial.
47. Gap: Dataverse-style team/business-unit mapping is not documented.
48. Repair: add tenancy/identity/governance handoff rows for principal and organization mapping.
49. Surface DV-09: auditing and change tracking support governance and data lifecycle.
50. Source: Microsoft Dataverse auditing documentation.
51. Oyatie evidence: PRD requires audit-chain integration and AsyncAPI change events exist: `microservices/ontology/PRD.md:768-788`, `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85`.
52. Oyatie status: partial.
53. Gap: no source/tests prove event completeness or replay.
54. Repair: add audit-chain event contract tests.
55. Surface DV-10: service protection limits enforce 6,000 requests per 300-second window, 1,200 seconds combined execution time, and 52 or higher concurrent requests per web server.
56. Source: Microsoft service protection API limits official documentation.
57. Oyatie evidence: PRD has latency and throughput budgets but not service-protection behavior: `microservices/ontology/PRD.md:735-788`.
58. Oyatie status: partial.
59. Gap: the service lacks public per-user throttling contracts, retry headers, and overload error models.
60. Repair: add abuse-limit policy that meets single industry-leader targets and deployment overlays.
61. Surface DV-11: elastic tables support high-throughput CRUD with feature exclusions.
62. Source: Microsoft elastic table documentation.
63. Oyatie evidence: PRD targets large object and read/write scale: `microservices/ontology/PRD.md:760-767`.
64. Oyatie status: partial.
65. Gap: no split between standard object storage and high-throughput object storage is documented.
66. Repair: define storage class overlays without using retired business levels.
67. Surface DV-12: virtual tables expose external data without copying into Dataverse.
68. Source: Microsoft Dataverse virtual table documentation.
69. Oyatie evidence: architecture mentions ontology projections and external counterpart equivalence but not virtualized external objects: `microservices/ontology/ARCHITECTURE.md:52-59`.
70. Oyatie status: absent.
71. Gap: ontology has no virtual-object contract.
72. Repair: add external object type source binding or assign it to data-connector services.
73. Surface DV-13: alternate keys and duplicate detection support identity consistency.
74. Source: Microsoft Dataverse table metadata documentation.
75. Oyatie evidence: OpenAPI object type schemas include identifiers, but duplicate detection is not defined: `microservices/ontology/contracts/openapi/ontology.yaml:90-160`.
76. Oyatie status: partial.
77. Gap: no uniqueness, alternate key, or duplicate detection policy is visible.
78. Repair: add key model and merge semantics.
79. Surface DV-14: maker portal supports low-code schema authoring.
80. Source: Microsoft Power Apps maker documentation.
81. Oyatie evidence: no frontend/admin source exists under ontology.
82. Oyatie status: owned-elsewhere.
83. Gap: ontology still needs APIs for UI authoring to consume.
84. Repair: define admin authoring flows and assign UI implementation outside ontology.
85. Surface DV-15: Power Automate integration consumes Dataverse actions/events.
86. Source: Microsoft Dataverse connector/custom API documentation.
87. Oyatie evidence: AsyncAPI event channels exist for ontology resource changes: `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85`.
88. Oyatie status: partial.
89. Gap: event subscription guarantees, delivery policies, and workflow-engine binding are not fully documented.
90. Repair: add event handoff contract and failure-mode runbook.
91. Surface DV-16: metadata APIs expose schema for generated clients and tooling.
92. Source: Microsoft Dataverse metadata documentation.
93. Oyatie evidence: OpenAPI, AsyncAPI, and proto contracts exist: `microservices/ontology/contracts/openapi/ontology.yaml:1-520`, `microservices/ontology/contracts/asyncapi/ontology-events.yaml:1-202`, `microservices/ontology/contracts/proto/ontology.proto:1-280`.
94. Oyatie status: partial.
95. Gap: generator pipeline and schema compatibility checks are not evidenced under ontology.
96. Repair: bind contract generation to developer-sdk or CI gates.

## Counterpart 3: Salesforce Data Cloud Capability Surface

1. Surface SDC-01: Data Lake Objects ingest and store source data.
2. Source: Salesforce Data Cloud object model documentation.
3. Oyatie evidence: ontology maps typed business objects but does not own raw lake ingestion: `microservices/ontology/PRD.md:91-119`.
4. Oyatie status: owned-elsewhere.
5. Gap: ontology needs source-binding metadata if source data lineage affects object types.
6. Repair: add source mapping fields or a handoff to data-ingestion services.
7. Surface SDC-02: Data Model Objects provide harmonized canonical model shape.
8. Source: Salesforce Data Cloud object model documentation.
9. Oyatie evidence: canonical object-type registry is central to README and PRD: `microservices/ontology/README.md:17-21`, `microservices/ontology/PRD.md:91-119`.
10. Oyatie status: covered in concept, partial in executable evidence.
11. Gap: no implementation verifies harmonization behavior.
12. Repair: add contract tests and migration fixtures.
13. Surface SDC-03: Unified Data Model Objects represent identity-resolved unified entities.
14. Source: Salesforce Data Cloud object model documentation.
15. Oyatie evidence: object graph exists, but identity resolution is not explicit: `microservices/ontology/contracts/openapi/ontology.yaml:476-520`.
16. Oyatie status: partial.
17. Gap: identity graph and merge/conflict semantics are missing.
18. Repair: define whether identity resolution belongs to ontology, identity, intelligence, or customer-data services.
19. Surface SDC-04: data spaces partition data and metadata by organizational boundary.
20. Source: Salesforce Data Cloud data space documentation.
21. Oyatie evidence: multi-tenant posture exists, but tenant_class terms are absent and deployment contexts are not instantiated in IaC: `microservices/ontology/manifest.json:1-80`.
22. Oyatie status: partial.
23. Gap: data-space-like boundary semantics are not first-class.
24. Repair: add tenant, workspace, org, and data-space boundary metadata with Cedar binding.
25. Surface SDC-05: identity resolution maps source profiles into unified profiles.
26. Source: Salesforce Data Cloud identity resolution documentation.
27. Oyatie evidence: no identity-resolution resource appears in current contracts: `microservices/ontology/contracts/openapi/ontology.yaml:247-520`.
28. Oyatie status: absent or owned-elsewhere.
29. Gap: Data Cloud parity requires either owning this as an ontology identity graph or documenting an explicit handoff.
30. Repair: add identity-resolution handoff to identity/intelligence services.
31. Surface SDC-06: calculated insights define measures and dimensions for analytics.
32. Source: Salesforce Data Cloud query and calculated insight documentation.
33. Oyatie evidence: function types exist, but calculated insight resources do not: `microservices/ontology/contracts/openapi/ontology.yaml:384-430`.
34. Oyatie status: partial.
35. Gap: function metadata is not the same as a governed insight definition with measures and dimensions.
36. Repair: add derived-metric definitions or assign to analytics.
37. Surface SDC-07: SQL query APIs return Data Cloud records with row limits and pagination.
38. Source: Salesforce Query API official documentation.
39. Oyatie evidence: graph traversal endpoint exists, but generic SQL/query API is out of PRD scope: `microservices/ontology/contracts/openapi/ontology.yaml:476-520`, `microservices/ontology/PRD.md:1041`.
40. Oyatie status: partial.
41. Gap: current contract handles graph traversal, not all ad hoc semantic SQL.
42. Repair: keep ad hoc SQL out of ontology or explicitly define safe query API boundaries.
43. Surface SDC-08: ingestion API supports streaming and bulk ingestion with limits.
44. Source: Salesforce ingestion API official documentation.
45. Oyatie evidence: AsyncAPI captures ontology resource changes, not raw data ingestion: `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85`.
46. Oyatie status: owned-elsewhere.
47. Gap: ontology should not own raw source ingestion unless object-type source binding is in scope.
48. Repair: add source-binding handoff.
49. Surface SDC-09: activations export audiences or segments to destination systems.
50. Source: Salesforce Data Cloud activation documentation.
51. Oyatie evidence: no activation or audience export contract exists under ontology.
52. Oyatie status: owned-elsewhere.
53. Gap: activation is likely intelligence/marketing/workflow scope, but ontology must supply graph predicates.
54. Repair: add predicate handoff if customer segmentation consumes ontology.
55. Surface SDC-10: data actions trigger external systems from Data Cloud events.
56. Source: Salesforce webhook data action target documentation.
57. Oyatie evidence: action type schemas and AsyncAPI events exist: `microservices/ontology/contracts/openapi/ontology.yaml:336-383`, `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85`.
58. Oyatie status: partial.
59. Gap: external webhook delivery and retry policy are not documented.
60. Repair: assign delivery to workflow-engine/events and keep ontology as schema owner.
61. Surface SDC-11: data graph API exposes connected profile/business graph data.
62. Source: Salesforce Data Cloud object-specific and data graph documentation.
63. Oyatie evidence: graph traversal endpoint exists: `microservices/ontology/contracts/openapi/ontology.yaml:476-520`.
64. Oyatie status: partial.
65. Gap: traversal depth and authorization are present, but profile-specific graph convenience APIs are not.
66. Repair: add object-set/data-graph views or assign to product APIs.
67. Surface SDC-12: consent, privacy, and subject rights are core governance controls.
68. Source: Salesforce Data Cloud privacy/governance documentation.
69. Oyatie evidence: DPIA exists and PRD requires privacy and audit controls: `microservices/ontology/dpia.md:1-195`, `microservices/ontology/PRD.md:768-788`.
70. Oyatie status: partial.
71. Gap: DPIA signoffs remain open: `microservices/ontology/dpia.md:188-195`.
72. Repair: complete DPIA signoffs and bind subject-rights policy to contracts.
73. Surface SDC-13: connectors bring data from Salesforce and external systems.
74. Source: Salesforce Data Cloud integration documentation.
75. Oyatie evidence: no connector inventory lives under ontology.
76. Oyatie status: owned-elsewhere.
77. Gap: ontology needs only source metadata and lineage references.
78. Repair: add external source metadata fields.
79. Surface SDC-14: data transforms map source objects into model objects.
80. Source: Salesforce Data Cloud object model and ingestion documentation.
81. Oyatie evidence: migration playbook demonstrates Palantir transformation concerns, but not Salesforce transform mapping: `microservices/ontology/migration-playbooks/from-palantir-foundry.md:34-94`.
82. Oyatie status: partial.
83. Gap: no Data Cloud migration playbook exists.
84. Repair: add Salesforce Data Cloud migration mapping.
85. Surface SDC-15: governance/lineage over data lake and data model objects.
86. Source: Salesforce Data Cloud architecture and object model documentation.
87. Oyatie evidence: PRD states lineage metadata is in scope: `microservices/ontology/PRD.md:91-119`.
88. Oyatie status: partial.
89. Gap: lineage endpoints and lineage event examples are not first-class in current contract.
90. Repair: add lineage resource or link to audit-chain handoff.
91. Surface SDC-16: rebranded Data 360 documentation preserves Data Cloud functionality.
92. Source: Salesforce Data 360 query documentation.
93. Oyatie evidence: current audit still uses the user-required `Salesforce Data Cloud` label while citing Data 360 docs where Salesforce renamed documentation.
94. Oyatie status: covered for audit naming.
95. Gap: future docs should mention the rebrand once canonical direction allows it.
96. Repair: keep this report aligned to the user-specified counterpart name.

## Union-Coverage Matrix

| Capability ID | Union capability | Counterpart drivers | Oyatie status | Evidence | Gap class |
|---|---|---|---|---|---|
| U-001 | Object/table/model resource registry | Palantir object types; Dataverse tables; Salesforce DMOs | partial | `microservices/ontology/README.md:17-21`; `microservices/ontology/PRD.md:91-119` | executable evidence |
| U-002 | Relationship/link model | Palantir link types; Dataverse relationships; Data Cloud graph | partial | `microservices/ontology/contracts/openapi/ontology.yaml:289-335` | cascade and tests |
| U-003 | Action/custom operation schema | Palantir action types; Dataverse custom APIs; Data Cloud data actions | partial | `microservices/ontology/contracts/openapi/ontology.yaml:336-383` | execution and retry |
| U-004 | Function/custom API metadata | Palantir functions; Dataverse custom APIs | partial | `microservices/ontology/contracts/openapi/ontology.yaml:384-430` | runtime handoff |
| U-005 | Policy predicate model | Palantir permissions; Dataverse roles; Data Cloud governance | partial | `microservices/ontology/contracts/openapi/ontology.yaml:431-475` | tenant class and foundry principal inheritance |
| U-006 | Graph traversal/query | Palantir object sets; Data Cloud data graph; Dataverse relationships | partial | `microservices/ontology/contracts/openapi/ontology.yaml:476-520` | object views and identity graph |
| U-007 | Interface/abstract type model | Palantir interfaces | absent | no interface path in `microservices/ontology/contracts/openapi/ontology.yaml:247-520` | product parity |
| U-008 | Shared property catalog | Palantir shared properties | absent | no shared-property path in `microservices/ontology/contracts/openapi/ontology.yaml:247-520` | migration parity |
| U-009 | Object type groups/package groups | Palantir groups; Dataverse solutions | partial | manifest exists at `microservices/ontology/manifest.json:1-80` | lifecycle packaging |
| U-010 | Schema proposal and activation state machine | Palantir proposals; Dataverse solutions | partial | events at `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85` | state machine |
| U-011 | Bulk and streaming ingestion | Data Cloud ingestion | owned-elsewhere | ontology resource changes only at `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85` | handoff |
| U-012 | Identity resolution and unified profile | Salesforce Data Cloud | absent or owned-elsewhere | no identity path in `microservices/ontology/contracts/openapi/ontology.yaml:247-520` | ownership |
| U-013 | Calculated insights/derived metrics | Salesforce Data Cloud; Dataverse calculated columns | partial | function endpoint at `microservices/ontology/contracts/openapi/ontology.yaml:384-430` | semantic gap |
| U-014 | Business rules and validation rules | Dataverse business rules | partial | policy predicates at `microservices/ontology/contracts/openapi/ontology.yaml:431-475` | validation vs authorization |
| U-015 | Plug-in/custom extension lifecycle | Dataverse plug-ins; Palantir function-backed actions | owned-elsewhere | PRD requirements at `microservices/ontology/PRD.md:641-729` | workflow binding |
| U-016 | Audit and event emission | All three | partial | AsyncAPI channels at `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85` | test evidence |
| U-017 | Privacy and DPIA coverage | Salesforce governance; Dataverse audit | partial | `microservices/ontology/dpia.md:188-195` | signoffs |
| U-018 | Migration from Palantir | Palantir Foundry Ontology | partial | `microservices/ontology/migration-playbooks/from-palantir-foundry.md:34-94` | doctrine refresh |
| U-019 | Migration from Dataverse | Microsoft Dataverse | absent | inventory lacks Dataverse playbook | competitor coverage |
| U-020 | Migration from Salesforce Data Cloud | Salesforce Data Cloud | absent | inventory lacks Salesforce playbook | competitor coverage |
| U-021 | Public service protection limits | Dataverse limits; Salesforce limits | partial | PRD performance budgets at `microservices/ontology/PRD.md:735-788` | overload contract |
| U-022 | External data virtualization | Dataverse virtual tables; Data Cloud connectors | absent | no virtual object contract in `microservices/ontology/contracts/openapi/ontology.yaml:247-520` | source binding |
| U-023 | Generated client metadata | Dataverse metadata API; Palantir API; Salesforce APIs | partial | OpenAPI/proto/AsyncAPI contracts exist | generator pipeline |
| U-024 | Admin authoring UI support | Palantir Ontology Manager; Dataverse maker portal | owned-elsewhere | no ontology UI source tree | handoff |
| U-025 | Multi-organization sharing and data spaces | Palantir shared ontologies; Salesforce data spaces | partial | manifest bounded contexts at `microservices/ontology/manifest.json:6-25` | tenant/data-space model |
| U-026 | Deployment portability | Oyatie canonical direction | absent for IaC | no six context OpenTofu dirs under `microservices/ontology/iac/` | canonical gap |
| U-027 | OS support matrix | Oyatie canonical direction | absent | no `supported-oses.json` in inventory | canonical gap |
| U-028 | Rust backend implementation | Oyatie canonical direction | absent | no `src/` tree under ontology | executable gap |
| U-029 | Tenant classes | Oyatie canonical direction | absent | no `tenant_class`, `demo_trial`, or `revenue_share` terms in path | commercial model gap |
| U-030 | Foundry agent-state absorption | Wave 15I direction | absent | no `agent-state` term in ontology path | retirement blocker |
| U-031 | Cedar principal inheritance | Foundry retirement doctrine | partial | policy predicate contracts exist, but no `oyatie.foundry.*` mapping | authorization gap |
| U-032 | Contract consistency | All counterparts require stable APIs | partial | PRD/ADR/OpenAPI prefix conflict in coherence audit | API source-of-truth gap |
| U-033 | Graph depth and traversal safety | Palantir object graph; Data Cloud data graph | partial | OpenAPI traversal depth max 5 at `microservices/ontology/contracts/openapi/ontology.yaml:426` | documented rationale |
| U-034 | Object history and lineage | Palantir/audit; Salesforce lineage | partial | PRD lineage claim at `microservices/ontology/PRD.md:91-119` | lineage endpoint |
| U-035 | High-volume object scale | Dataverse elastic tables; Salesforce Data Cloud | partial | PRD scale target at `microservices/ontology/PRD.md:760-767` | benchmark evidence |
| U-036 | Schema package import/export | Dataverse solutions; Palantir Marketplace interfaces | partial | manifest exists but package lifecycle absent | ALM parity |
| U-037 | Backward compatibility and deprecation | All three | partial | ADR proposed at `microservices/ontology/decisions/ADR-ONT-001-rdf-shape-vs-property-graph-storage.md:4` | release governance |
| U-038 | External event delivery | Dataverse business events; Salesforce data actions | partial | AsyncAPI events exist | retry and subscription policy |
| U-039 | Object views and saved object sets | Palantir Object Explorer/Object Views | absent | no saved-view/object-set contract | application parity |
| U-040 | Source-system lineage mapping | Salesforce DLO to DMO; Palantir backing datasets | partial | PRD lineage claim | source binding |

## Family Summary: Modeling Substrate

1. Palantir sets the highest bar for ontology-specific modeling because it treats object types, link types, action types, interfaces, shared properties, and groups as ontology resources.
2. Dataverse sets the strongest low-code metadata bar because tables, relationships, business rules, custom APIs, plug-ins, and solutions are integrated into ALM.
3. Salesforce Data Cloud sets the strongest customer-data harmonization bar because DLO, DMO, unified DMO, identity resolution, data spaces, calculated insights, and activations are bundled.
4. Oyatie ontology currently matches the core idea of a typed semantic registry: object types, relationship types, action types, function types, policy predicates, and graph traversal.
5. The strongest Oyatie artifact evidence is contract presence across OpenAPI, AsyncAPI, and proto.
6. The weakest Oyatie evidence is executable reality: no source tree, no tests, no six-context OpenTofu modules, no OS manifest, and no tenant_class semantics.
7. Palantir-style interface support is the largest product-surface gap.
8. Dataverse-style business rules and solution ALM are the largest admin-governance gap.
9. Salesforce-style identity resolution and activation are the largest customer-data gap.
10. Some Salesforce capabilities should not be pushed into ontology, but their handoffs must be explicit.
11. Ontology should own the semantic schema, typed graph, policy predicate metadata, lineage references, and resource lifecycle.
12. Data ingestion should likely own raw source ingestion.
13. Identity should likely own source identity graph merge decisions unless ontology is explicitly assigned a generic identity graph substrate.
14. Workflow-engine should likely own execution of actions, functions, data actions, and plug-in-like extensions.
15. Audit-chain should own immutable replay and evidence retention.
16. Application-builder/admin-console should likely own authoring UI.
17. Developer-sdk should likely own generated client libraries and Stainless/OpenAPI generator policy.
18. The missing cross-microservice handoff file makes these boundaries less enforceable.
19. Current contracts are enough to start a semantic registry implementation but not enough to claim union parity.
20. The foundry `agent-state` absorption requirement adds a separate ownership blocker not present in ordinary counterpart comparison.

## Family Summary: Control Plane and Governance

1. Palantir governance pressure: shared/private ontology boundaries, object permissions, interface actions, and organization markings.
2. Dataverse governance pressure: security roles, teams, business units, field security, managed solutions, and custom API privileges.
3. Salesforce governance pressure: data spaces, consent, privacy, subject rights, and activation control.
4. Oyatie evidence: Cedar-first policy is present in README, PRD, OpenAPI, and policy artifacts.
5. Oyatie evidence: DPIA exists but signoffs are still open.
6. Oyatie evidence: OpenSLO files exist for availability and latency targets.
7. Oyatie gap: tenant_class semantics are absent.
8. Oyatie gap: Cedar principal inheritance for `oyatie.foundry.*` claims is absent.
9. Oyatie gap: six-context deployment control plane is absent under `iac/`.
10. Oyatie gap: OS matrix is absent.
11. Governance score: high conceptual ambition, medium contract presence, low executable deployment evidence.
12. Required next repair: make commercial model, deployment model, and principal model machine-readable.
13. Required next repair: decide sensitivity vocabulary rename and contract migration path.
14. Required next repair: add handoffs for identity, workflow-engine, audit-chain, developer-sdk, cloud-iac, tenancy, governance, and application-builder.
15. Required next repair: connect SLOs to load tests and source code.

## Family Summary: Developer and Operator Surface

1. Palantir provides user-facing ontology tooling and applications.
2. Dataverse provides maker/admin UI, solution ALM, API metadata, plug-in tooling, and service-protection behavior.
3. Salesforce provides ingestion/query APIs, SQL, object-specific APIs, data actions, and activation tooling.
4. Oyatie provides strong documentation artifacts but no executable service implementation under ontology.
5. Oyatie provides onboarding, FAQ, tutorials, migration playbooks, runbooks, and reference implementations.
6. Some of those docs still use retired business-level labels and must be rewritten.
7. Reference implementations include Rust SDK examples, which align with backend policy if treated as examples.
8. SDK plan references non-Rust languages and generators, which should move to developer-sdk governance or be scoped clearly as client outputs.
9. Operator runbooks exist, but no runbook covers the assigned foundry `agent-state` absorption slice.
10. Failure-mode docs reference a missing recovery playbook.
11. The operator surface is stronger than source reality.
12. The developer surface is stronger for design intent than for compilation or testability.
13. The parity matrix therefore scores many items as partial rather than absent.
14. The next implementation slice should not add more prose-only surfaces without testable contracts.
15. The next documentation slice should retire old commercial vocabulary and add handoffs.

## Headline Gap Analysis

1. Gap H-01: Palantir interfaces are not represented.
2. Evidence: no interface path is present in OpenAPI paths: `microservices/ontology/contracts/openapi/ontology.yaml:247-520`.
3. Impact: generic workflows over abstract shapes cannot match Palantir parity.
4. Severity: P2.
5. Repair: add interface resources, inheritance, link constraints, and action support boundaries.
6. Gap H-02: shared properties are not first-class.
7. Evidence: property schemas exist only inside object type schema: `microservices/ontology/contracts/openapi/ontology.yaml:90-160`.
8. Impact: migration from Palantir shared property catalogs is lossy.
9. Severity: P2.
10. Repair: add shared-property catalog with ownership, metadata, and version rules.
11. Gap H-03: Dataverse custom API parity is only conceptual.
12. Evidence: action/function endpoints exist, but no binding type, plug-in, or event step lifecycle is modeled: `microservices/ontology/contracts/openapi/ontology.yaml:336-430`.
13. Impact: Dataverse migration cannot preserve action/function extension semantics.
14. Severity: P2.
15. Repair: add operation binding metadata and workflow handoff.
16. Gap H-04: Dataverse business-rule parity is ambiguous.
17. Evidence: policy predicates exist, but business rules are not modeled separately: `microservices/ontology/contracts/openapi/ontology.yaml:431-475`.
18. Impact: UI validation, data validation, and security predicates can be conflated.
19. Severity: P2.
20. Repair: separate authorization predicates, validation rules, and presentation recommendations.
21. Gap H-05: Salesforce identity-resolution parity is not assigned.
22. Evidence: no identity-resolution endpoint exists in OpenAPI: `microservices/ontology/contracts/openapi/ontology.yaml:247-520`.
23. Impact: Data Cloud customer-profile parity remains incomplete.
24. Severity: P2.
25. Repair: assign identity graph ownership and define ontology references.
26. Gap H-06: Salesforce ingestion parity belongs elsewhere but lacks handoff.
27. Evidence: AsyncAPI covers ontology resource changes, not ingestion jobs: `microservices/ontology/contracts/asyncapi/ontology-events.yaml:27-85`.
28. Impact: external source mapping and lineage can fall between services.
29. Severity: P2.
30. Repair: add cross-microservice handoff.
31. Gap H-07: Foundry `agent-state` absorption is absent.
32. Evidence: current ontology path has no `agent-state` term; coherence audit records the search and ADR-0247 references.
33. Impact: Wave 15I foundry retirement is blocked for this assigned slice.
34. Severity: P1.
35. Repair: define agent-state object types, lifecycle, contracts, policies, runbooks, and tests.
36. Gap H-08: tenant_class adoption is absent.
37. Evidence: no `tenant_class`, `demo_trial`, or `revenue_share` terms under ontology.
38. Impact: commercial model and deployment overlays cannot be verified.
39. Severity: P1.
40. Repair: encode tenant_class into docs/contracts/policies where relevant.
41. Gap H-09: six-context deployment is absent.
42. Evidence: ontology `iac/` contains Helm/Kustomize rather than context OpenTofu modules.
43. Impact: deployable-context claim is not supportable.
44. Severity: P1.
45. Repair: add per-context OpenTofu modules or explicit approved non-applicability.
46. Gap H-10: OS support is absent.
47. Evidence: no `supported-oses.json` exists in ontology inventory.
48. Impact: supported OS claim cannot be verified.
49. Severity: P1.
50. Repair: add OS matrix aligned to canonical policy.
51. Gap H-11: status is incoherent.
52. Evidence: PRD proposed, doc_status published, README status ga, ADR proposed.
53. Impact: implementation readiness claims can mislead downstream plans.
54. Severity: P2.
55. Repair: normalize lifecycle status.
56. Gap H-12: endpoint source of truth diverges.
57. Evidence: PRD, ADR, and OpenAPI use different prefixes.
58. Impact: SDK generation and service implementation can drift.
59. Severity: P2.
60. Repair: choose one prefix and update all artifacts.

## Additive Surface Recommendations

1. Add `InterfaceType` as a first-class resource.
2. Add `SharedProperty` as a first-class resource.
3. Add `ObjectTypeGroup` or `OntologyPackage` as a first-class grouping resource.
4. Add schema proposal lifecycle: draft, review, approved, active, deprecated, retired.
5. Add operation binding metadata for actions and functions.
6. Add action execution constraints with transaction, idempotency, and rollback semantics.
7. Add validation-rule resource separate from authorization predicate.
8. Add derived-property or calculated-insight resource if analytics stays in ontology.
9. Add object-set or saved-view resource if application object views consume ontology directly.
10. Add lineage-source binding for object types and properties.
11. Add virtual-object source binding for external data without copy.
12. Add alternate-key and uniqueness constraints.
13. Add duplicate/merge policy references.
14. Add data-space or workspace boundary metadata.
15. Add tenant_class metadata where commercial model affects usage caps, SLO, BYOK, or compliance packs.
16. Add deployment-context overlays for read/write scale and availability.
17. Add OS support manifest.
18. Add six context OpenTofu module skeletons with real variables and outputs.
19. Add OCI Always Free profile under the guest-on-oci context.
20. Add Rust source implementation for the contract slice.
21. Add contract tests for OpenAPI/proto parity.
22. Add AsyncAPI event-completeness tests.
23. Add graph traversal authorization tests.
24. Add action shape limit tests.
25. Add migration fixtures for Palantir Foundry Ontology.
26. Add migration fixtures for Microsoft Dataverse.
27. Add migration fixtures for Salesforce Data Cloud.
28. Add cross-microservice handoff file.
29. Add handoff to tenancy for tenant boundaries and tenant_class source of truth.
30. Add handoff to governance for Cedar principal and claim issuance.
31. Add handoff to identity for identity resolution.
32. Add handoff to workflow-engine for action/function execution.
33. Add handoff to audit-chain for event replay and immutable evidence.
34. Add handoff to developer-sdk for generated clients.
35. Add handoff to cloud-iac/cell for deployment modules if those remain shared.
36. Add handoff to application-builder/admin-console for authoring UI.
37. Add foundry `agent-state` ownership section with schema and lifecycle.
38. Add `oyatie.foundry.*` Cedar principal routing if ontology owns agent-state policies.
39. Add agent-state events for create/update/lease/replay/compact/archive.
40. Add agent-state failure modes and incident response.
41. Add agent-state retention and privacy classification.
42. Add agent-state migration playbook from deprecated foundry.
43. Add ADR amendment if agent-state ownership moves away from ontology.
44. Remove or rewrite retired business-level vocabulary in onboarding docs.
45. Remove or rewrite retired business-level vocabulary in migration playbooks.
46. Remove or rewrite retired business-level vocabulary in tutorials.
47. Remove or rewrite retired business-level vocabulary in benchmark docs.
48. Retire the legacy commercial-capability directory after Wave 15J replacement exists.
49. Rename `max_sensitivity_level`, `property_sensitivity_level`, and `PropertySensitivityLevel` if they are business-level vocabulary rather than sensitivity vocabulary.
50. Add explicit non-business-level sensitivity vocabulary if sensitivity is still needed.

## Counterpart-by-Capability Notes

1. Palantir object types map most directly to Oyatie object types.
2. Palantir link types map most directly to Oyatie relationship types.
3. Palantir action types map partly to Oyatie action types and partly to workflow execution.
4. Palantir function-backed actions map to Oyatie function metadata plus workflow/intelligence execution.
5. Palantir interfaces do not yet map cleanly.
6. Palantir shared properties do not yet map cleanly.
7. Palantir object type groups do not yet map cleanly.
8. Palantir Ontology Manager maps to admin-console/application-builder, not ontology runtime.
9. Palantir object views map to application views plus ontology object-set contracts, which are absent.
10. Dataverse tables map to object types.
11. Dataverse relationships map to relationship types.
12. Dataverse columns map to object properties.
13. Dataverse choices map to enum-like property constraints, which are only partially visible.
14. Dataverse calculated columns map to derived properties or analytics functions, not yet first-class.
15. Dataverse rollups map to calculated insights or query projections, not yet first-class.
16. Dataverse business rules map ambiguously and need a boundary decision.
17. Dataverse custom APIs map to action/function metadata plus workflow execution.
18. Dataverse plug-ins map to workflow or function execution, not ontology core.
19. Dataverse solutions map to ontology package lifecycle, not yet present.
20. Dataverse security roles map to Cedar policy/governance integration, partially present.
21. Dataverse service protection limits map to overload contracts, partially present.
22. Salesforce DLOs map to data-ingestion owned source objects, not ontology core.
23. Salesforce DMOs map to ontology object types.
24. Salesforce unified DMOs map to identity-resolved object types, ownership unresolved.
25. Salesforce data spaces map to tenant/workspace/data-space boundary metadata, partially present.
26. Salesforce calculated insights map to derived metric resources or analytics, partially present.
27. Salesforce Query API maps to graph traversal and query surfaces, partially present.
28. Salesforce ingestion API maps to ingestion services, not ontology core.
29. Salesforce data actions map to action metadata and workflow delivery.
30. Salesforce activations map to marketing/workflow/customer-data products, not ontology core.

## Priority Repair Slice

1. First repair: add tenant_class vocabulary to ontology docs/contracts only where service behavior changes.
2. Reason: canonical doctrine requires replacement model and current artifacts have no adoption evidence.
3. Evidence: no tenant_class terms found under ontology; architecture still uses free/paid business-level wording at `microservices/ontology/ARCHITECTURE.md:174`.
4. Second repair: add six-context OpenTofu module plan or actual modules.
5. Reason: all six deployable contexts are default unless audited otherwise.
6. Evidence: ADR-0328 D-15/D-16 require the six-context and OpenTofu discipline; ontology `iac/` contains Helm/Kustomize only.
7. Third repair: add OS support manifest.
8. Reason: supported OS matrix is a canonical Wave 3 constraint.
9. Evidence: no `supported-oses.json` exists under ontology.
10. Fourth repair: define foundry `agent-state` absorption.
11. Reason: user directive explicitly assigns this slice to ontology for the audit.
12. Evidence: no agent-state artifacts under ontology.
13. Fifth repair: resolve interface/shared-property/object-group parity.
14. Reason: those are direct Palantir Foundry Ontology resources and the first counterpart is the closest product match.
15. Evidence: no interface/shared-property/object-group endpoints in OpenAPI path range.
16. Sixth repair: add cross-microservice handoff file.
17. Reason: many union capabilities are intentionally outside ontology but lack enforceable ownership.
18. Evidence: inventory contains no `cross-microservice-handoffs.md`.
19. Seventh repair: normalize endpoint prefixes.
20. Reason: OpenAPI/proto/PRD/ADR drift will break SDK and implementation alignment.
21. Evidence: prefix conflict in PRD, ADR, and OpenAPI citations.
22. Eighth repair: add Rust implementation and tests.
23. Reason: parity cannot be claimed from docs alone.
24. Evidence: no `src/` or `tests/` tree under ontology.
25. Ninth repair: rewrite old commercial vocabulary.
26. Reason: Wave 15J retirement candidates are known.
27. Evidence: 45 exact references cataloged in the coherence audit.
28. Tenth repair: refresh counterpart docs.
29. Reason: current local competitor matrix uses old counterparts and does not match this audit.
30. Evidence: `microservices/ontology/competitor-parity-matrix.md:21-22`, `microservices/ontology/competitor-parity-matrix.md:109-115`.

## Bottom-Line Parity Verdict

1. Palantir Foundry Ontology parity: partial.
2. Palantir reason: core object/link/action/function/policy ideas exist, but interfaces, shared properties, object groups, proposal lifecycle, object sets, and action execution limits are not complete.
3. Microsoft Dataverse parity: partial.
4. Dataverse reason: object/table, relationship, custom operation, security, and metadata concepts exist, but business rules, plug-in lifecycle, solution ALM, service-protection behavior, virtual tables, and maker/admin handoffs are incomplete.
5. Salesforce Data Cloud parity: partial and deliberately split across service boundaries.
6. Salesforce reason: ontology can own DMOs, semantic graph, lineage references, and policy metadata, but ingestion, identity resolution, activations, and raw Data Cloud-style connectors likely belong elsewhere.
7. Canonical Oyatie parity blocker: six-context OpenTofu is absent.
8. Canonical Oyatie parity blocker: OS support matrix is absent.
9. Canonical Oyatie parity blocker: tenant_class model is absent.
10. Canonical Oyatie parity blocker: foundry `agent-state` absorption is absent.
11. Canonical Oyatie parity blocker: no source/test tree exists.
12. Documentation parity strength: high.
13. Contract parity strength: medium.
14. Runtime parity strength: low.
15. Deployment parity strength: low.
16. Commercial doctrine alignment: low until tenant_class rewrite lands.
17. Industry counterpart surface coverage: incomplete but clearly recoverable.
18. Recommended next artifact: cross-microservice handoff file plus interface/shared-property contract amendment.
19. Recommended next implementation artifact: Rust contract skeleton with tests for object types, relationship types, action types, and policy predicates.
20. Recommended next retirement artifact: Wave 15J rewrite of old business-level vocabulary.
