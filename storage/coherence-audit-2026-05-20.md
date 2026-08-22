# cloud-storage ownership-coherence audit - 2026-05-20

Audit owner: single-agent cloud-storage lane.
Scope: `/Users/jasonlee/oyatie/microservices/cloud-storage/` only for write output.
Target counterparts: AWS S3, Google Cloud Storage, Azure Blob Storage.
Deployable contexts assumed in scope unless contradicted: oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, oyatie-as-cloud-provider.
Audit date: 2026-05-20.

## Anchor block

1. Canonical sequence: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2214` for §D-15 multi-context and cloud-storage ownership; `:2241-2494` for §D-16 OpenTofu; `:3441-3754` for §D-19 OCI Always Free; `:3756-4146` for §D-20 audit decision tree.
2. Machine-readable plan: `specs/master-plan-sequencing.json:704-867` for six deployment contexts, OpenTofu substrate, supported OS matrix, Rust-strict language policy, and OCI Always Free profile.
3. Service PRD citation: service-local `microservices/cloud-storage/PRD.md` is absent; closest product-purpose evidence read is `docs/decisions/ADR-0702-identity-authz-live-apex.md:8-11` plus root cloud PRD capability records `docs/products/cloud/PRD.md:1587-1665`.
4. Service architecture citation: service-local `microservices/cloud-storage/ARCHITECTURE.md` is absent; closest architecture evidence read is root crate map `docs/products/cloud/PRD.md:131-137` and local backend explanation `microservices/cloud-storage/faqs/storage-engineer-faq.md:15-22`.
5. Documentation-rigor citation: `docs/standards/documentation-rigor.md:62-81` sets full-platform requirements and `docs/standards/documentation-rigor.md:133-156` defines intern-buildability and hyperscaler-grade tests.

## §1 µservice Purpose Summary

cloud-storage is intended to be Oyatie's storage control and data plane for cloud-provider mode and for workloads hosted on other substrates.
The root product PRD assigns bucket, object, block volume, file share, archive, backup, restore, KMS binding, policy, and SLO-bearing storage API ownership to cloud-storage.
The root cloud PRD maps `cloud-storage-kernel` to bucket, object, volume, filesystem, snapshot, and archive-tier concepts at `docs/products/cloud/PRD.md:131-137`.
The local tenant_class policy narrows the first written service-local purpose to object storage: buckets, object lifecycle, versioning, replication, storage classes, S3 API compatibility, retention, encryption, quota, and billing.
The local FAQ says the physical object backend is MinIO Enterprise in the near term, with Reed-Solomon erasure coding and per-tenant KMS wrapping.
ADR-0328 makes that adapter framing subordinate to the product boundary: cloud-storage owns bucket/object/container semantics, retention, encryption, quota, evidence, and billing, while S3 or OCI Object Storage may only be backing adapters.
Therefore the coherent product purpose is not "an S3 wrapper."
The coherent product purpose is an object, block, and file storage service that can run as Oyatie public cloud, guest workload, on-prem managed service, colo service, or Oyatie as a cloud provider.
The current service-local corpus only documents the object-storage subset.
The root PRD includes block and file APIs, but the service path has no block-volume tutorial, file-share runbook, volume SLO, file-share contract, or archive-vault architecture.
The service-local docs emphasize S3-compatible behavior heavily.
The same docs also mention Azure Blob and Google Cloud Storage compatibility, but they do not define a provider-neutral storage resource model.
The missing service PRD is not a harmless absence because it is the artifact that should reconcile object, block, file, archive, tenancy, regions, contexts, and pricing into one boundary.
The missing service architecture is also not harmless because it should define adapters, data-plane isolation, erasure coding, object index layout, metadata store, KMS envelope model, cross-region replication, and failure handling.
The current artifacts are useful operational sketches.
They are not a buildable ownership package.
The top purpose risk is product-surface shrinkage: local docs say "not a public S3 alternative" while canonical ADR-0328 requires cloud-storage to support the Oyatie-as-cloud-provider context.
The second purpose risk is adapter leakage: tenant_class definitions name MinIO and an AWS Glacier adapter instead of an Oyatie storage substrate abstraction.
The third purpose risk is language and operations drift: onboarding and migration examples depend on AWS CLI, boto3, gcloud-python, jq, and imperative `./bin/oya` flows instead of OpenTofu and Rust-only implementation surfaces.
The fourth purpose risk is evidence drift: the benchmark file claims measured results and an evidence path that is not present in the audited service path or `.foundry/evidence` location.
The strongest existing material is the service FAQ's domain details around lifecycle, replication, KMS, inventory, durability, and recovery.
The weakest existing material is the missing canonical ownership spine: no PRD, no architecture, no manifest, no OpenTofu, no SLO, no contracts, no runbook, no OS matrix, no source, no tests.

## §2 Inventory Snapshot

Inventory command: `find microservices/cloud-storage -maxdepth 3 -type f | sort`.
Inventory result count: 7 files.
Directory count observed: 8 directories including root.
Service-local lines audited: 1340 total.
Canonical lines audited separately: ADR-0328 §D-15..§D-20, master plan constraint fields, brief-template §3.9..§3.12, documentation-rigor §1.1/§2, eight memory files, root product PRD anchors, registry anchors, and chat history references.

| File | Lines | Role | Coherent with purpose? |
|---|---:|---|---|
| `benchmarks/cloud-storage-vs-s3-vs-gcs-vs-azure-blob-vs-r2-vs-minio.md` | 133 | benchmark comparison against object stores | partial: useful dimensions but measured-evidence claim is unverifiable and non-top-3 providers dilute the assigned counterpart set |
| `ADR-0329/ADR-0330/ADR-0331` | 110 | demo_trial/paid baseline profile/paid production profile/paid regulated profile tier sketch | partial: tier limits exist but OCI Always Free, six contexts, block/file storage, and provider-neutral substrate are missing |
| `faqs/storage-engineer-faq.md` | 191 | engineer Q&A | partial: rich object-storage operations but no context matrix, no OpenTofu, no OS, and forbidden SDK/tool references |
| `migration-playbooks/from-s3-and-azure-blob.md` | 186 | migration guide | partial: concrete migration phases but relies on AWS CLI, jq, boto3, Azure SDK, and imperative setup rather than Rust/OpenTofu lanes |
| `onboarding/storage-engineer-first-week.md` | 211 | week-one onboarding | partial: practical object walkthrough but lacks canonical build, test, OS, IaC, and context setup |
| `reference-implementations/lifecycle-versioning-replication-rust-sdk.md` | 258 | embedded Rust example | yes for Rust example scope; partial for ownership because it is doc-embedded and not service source |
| `tutorials/multi-class-lifecycle-versioning-and-replication.md` | 251 | object-storage tutorial | partial: strong feature walkthrough but requires boto3/AWS CLI/jq and does not cover six deployment contexts |

Missing service-local artifact: `PRD.md`.
Missing service-local artifact: `ARCHITECTURE.md`.
Missing service-local artifact: `README.md`.
Missing service-local artifact: `decisions/ADR-MS-*.md`.
Missing service-local artifact: `implementation-plans/IP-*.md`.
Missing service-local artifact: `contracts/*.yaml`.
Missing service-local artifact: `contracts/*.json`.
Missing service-local artifact: `contracts/*.proto`.
Missing service-local artifact: `slos/*.openslo.yaml`.
Missing service-local artifact: `cross-microservice-handoffs.md`.
Missing service-local artifact: `capacity-model.md`.
Missing service-local artifact: `failure-modes.md`.
Missing service-local artifact: `incident-response.md`.
Missing service-local artifact: `cost-budget.md`.
Missing service-local artifact: `dpia.md`.
Missing service-local artifact: `compliance.md`.
Missing service-local artifact: `runbooks/*`.
Missing service-local artifact: `iac/oyatie-public-cloud/*`.
Missing service-local artifact: `iac/guest-on-aws/*`.
Missing service-local artifact: `iac/oci-guest/*`.
Missing service-local artifact: `iac/on-prem/*`.
Missing service-local artifact: `iac/colo/*`.
Missing service-local artifact: `iac/oyatie-iaas/*`.
Missing service-local artifact: `iac/oci-guest/always-free/*`.
Missing service-local artifact: `supported-oses.json`.
Missing service-local artifact: `manifest.json` or any equivalent service manifest.
Missing service-local artifact: `src/*`.
Missing service-local artifact: `tests/*`.
Missing service-local artifact: threat model.
Missing service-local artifact: service-local policy fragments.
Missing service-local artifact: service-local billing-meter schema.
Missing service-local artifact: service-local tenant onboarding evidence.
Missing service-local artifact: service-local CI lane spec.
Missing service-local artifact: service-local benchmark harness.

## §3 9-Dimension Audit

### §3.1 Dimension 1 - Internal Coherence

Finding D1-001: Internal corpus is coherent around object-storage vocabulary, but incomplete against root cloud-storage scope.
Citation: `docs/decisions/ADR-0702-identity-authz-live-apex.md:8-11` defines object-storage duties.
Citation: `docs/products/cloud/PRD.md:131-137` includes object, block, and file storage APIs.
Classification: partial, P2.
Internal reference check: `ADR-0329/ADR-0330/ADR-0331:3` cites ADR-0244; target exists under `docs/decisions/`.
Internal reference check: `ADR-0329/ADR-0330/ADR-0331:3` cites ADR-0245; target exists under `docs/decisions/`.
Internal reference check: `ADR-0329/ADR-0330/ADR-0331:3` cites ADR-0248; target exists under `docs/decisions/`.
Internal reference check: `ADR-0329/ADR-0330/ADR-0331:3` cites ADR-0251; target exists under `docs/decisions/`.
Internal reference check: the retired tenant_class citation is superseded by ADR-0329, ADR-0330, and ADR-0331 under `docs/decisions/`.
Internal reference check: `onboarding/storage-engineer-first-week.md:7-15` references the same ADR cluster and AWS S3 API; targets are external/root-level, not service-local.
Internal reference check: `reference-implementations/lifecycle-versioning-replication-rust-sdk.md:6-26` references an `cloud-storage-sdk`; no service-local crate source exists.
Internal reference check: `benchmarks/cloud-storage-vs-s3-vs-gcs-vs-azure-blob-vs-r2-vs-minio.md:124-133` references `.foundry/evidence/benchmarks/cloud-storage/2026-05-13T22:14:42Z`; local `ls` found no such directory.
Internal reference check: `migration-playbooks/from-s3-and-azure-blob.md:39-53` references `./bin/oya storage tenant register` and `storage bucket create`; no service-local CLI contract exists.
Internal reference check: `tutorials/multi-class-lifecycle-versioning-and-replication.md:7-10` references `make dev-cell.up`, `make dev-tenant.create`, AWS CLI, and boto3; no service-local make target or Rust-only bootstrap contract exists.
Internal reference check: `faqs/storage-engineer-faq.md:78-88` references Cedar-translated S3 JSON policies; no service-local Cedar files exist.
Internal reference check: `faqs/storage-engineer-faq.md:122-130` references cloud-kms KEK wrapping; no local handoff file documents required API shape.
Internal reference check: `faqs/storage-engineer-faq.md:150-151` references paid production-or-regulated profile inventory reports; no local schema or SLO defines inventory report contracts.
Contradiction probe 1: paid production profile replication is "asynchronous by default; lag <= 5s p95" in `faqs/storage-engineer-faq.md:64-66`, but migration says AWS RTC maps to "paid production profile sync-replica SLA" at `migration-playbooks/from-s3-and-azure-blob.md:81-83`.
Contradiction probe 1 classification: P1 because sync versus async materially changes durability, write latency, and counterpart parity.
Contradiction probe 2: Tier invariant says Hot->Warm transition <= 1h at `ADR-0329/ADR-0330/ADR-0331:107-110`, while FAQ says paid baseline profile lifecycle daemons scan every 6h at `faqs/storage-engineer-faq.md:134-136`.
Contradiction probe 2 classification: P1 for tier/SLO mismatch.
Contradiction probe 3: Benchmark claims paid production profile lifecycle Hot->Warm p95 24min at `benchmarks/...:48-52`, while the FAQ's paid baseline profile scan is 6h and no paid production profile measured harness is present.
Contradiction probe 3 classification: P2 because benchmark evidence is not present.
Contradiction probe 4: Benchmark says paid production profile replication is async 4.8s at `benchmarks/...:60-64`, conflicting with migration's paid production profile sync claim.
Contradiction probe 4 classification: P1 because migration cutover could promise wrong data-loss semantics.
Contradiction probe 5: FAQ says maximum object size paid baseline profile 100GB and paid production profile 5TB at `faqs/storage-engineer-faq.md:92-95`; tenant_class policy omits object-size ceilings entirely.
Contradiction probe 5 classification: P2 because this is an omission, not a direct contradiction.
Contradiction probe 6: tenant_class policy demo_trial cost is about $15/mo at `ADR-0329/ADR-0330/ADR-0331:29-31`, while ADR-0328 requires OCI demo_trial to be Always Free under the OCI sub-profile at `ADR-0328:3441-3466`.
Contradiction probe 6 classification: P1 for canonical cost-tier drift.
Contradiction probe 7: FAQ says existing apps use boto3, Azure Blob SDK, and gcloud-python at `faqs/storage-engineer-faq.md:7-11`; ADR-0328 Rust-strict policy forbids Python and other non-allowlisted implementation languages at `ADR-0328:3938-4001`.
Contradiction probe 7 classification: P1 when treated as implementation guidance; acceptable only if explicitly labeled external customer-client compatibility.
Contradiction probe 8: Local docs call the service "not a public S3 alternative" at `faqs/storage-engineer-faq.md:7-11`, but ADR-0328 includes `oyatie-as-cloud-provider` where cloud-storage is an IaaS product at `ADR-0328:1981-2050`.
Contradiction probe 8 classification: P1 product-position drift.
Contradiction probe 9: tenant_class policy paid baseline profile names `aws-s3-glacier-deep` as cold tier at `ADR-0329/ADR-0330/ADR-0331:37-40`, but ADR-0328 forbids cloud-vendor APIs in domain logic and requires provider-agnostic business logic at `ADR-0328:2194-2204`.
Contradiction probe 9 classification: P1 unless rewritten as an optional adapter outside tier baseline.
Contradiction probe 10: The reference implementation uses Rust and an S3 SDK, but it is embedded in Markdown and not in `src/`, so it cannot satisfy the canonical build invocation.
Contradiction probe 10 classification: P2.
Internal severity summary: P1=6, P2=4, P3=0 in sampled probes.
Dimension verdict: drifted-fixable.
Dimension remediation: create service-local PRD and ARCHITECTURE first, then normalize tier and migration semantics.

### §3.2 Dimension 2 - Outbound Cross-References

Outbound reference category: ADRs.
ADR references from service-local files include ADR-0244, ADR-0245, ADR-0248, ADR-0251, ADR-0329, ADR-0330, ADR-0331, ADR-0253, and ADR-0212.
All listed ADR names resolve in `docs/decisions/` by repository search.
Outbound reference issue: service-local docs do not cite ADR-0328 §D-15..§D-20 even though those sections now govern this audit.
Outbound reference issue severity: P2.
Outbound reference category: external providers.
AWS S3 is cited in every local artifact.
Google Cloud Storage is cited in benchmark and FAQ compatibility notes.
Azure Blob Storage is cited in benchmark, FAQ, and migration playbook.
Cloudflare R2, MinIO, and Backblaze B2 are cited, but they are outside the assigned top-3 counterpart set.
Outbound reference issue severity for extra providers: P3, because they are useful context but distract from union-coverage obligations.
Outbound reference category: root product docs.
Root cloud PRD assigns storage object/block/file surfaces at `docs/products/cloud/PRD.md:131-137` and API SLOs/contracts at `docs/products/cloud/PRD.md:161-167`.
Root cloud PRD assigns capabilities CLOUD-CAP-031 through CLOUD-CAP-047 to cloud-storage at `docs/products/cloud/PRD.md:1587-1669`.
Service-local docs do not reverse-link those root capabilities.
Outbound reference issue severity: P2.
Outbound reference category: registry.
`registry/microservices.json:105-114` includes `cloud-storage`.
`registry/openapi/schema-bindings.tsv:41-58` binds cloud-storage object/block schemas to Rust crates.
`registry/openapi/runtime-bindings.tsv:9-11` binds block create, object put, and object get runtime functions to tests.
Service-local docs do not link to these registry rows.
Outbound reference issue severity: P2.
Outbound reference category: other µservices.
cloud-billing tutorial writes FOCUS exports to a cloud-storage bucket at `microservices/cloud-billing/tutorials/meter-attribute-invoice-and-export-focus.md:177-196`.
cloud-billing FAQ promises tenant-controlled S3-compatible exports via cloud-storage at `microservices/cloud-billing/faqs/billing-engineer-faq.md:127-130`.
cloud-data FAQ streams WAL records to cloud-storage for PITR at `microservices/cloud-data/faqs/data-engineer-faq.md:114-121`.
cloud-data tenant_class policy declares cloud-storage backup/archive integration at `microservices/cloud-data/ADR-0329/ADR-0330/ADR-0331:38-44` and `:62-64`.
Service-local cloud-storage has no `cross-microservice-handoffs.md`.
Reverse-reference issue severity: P1 because cloud-billing and cloud-data depend on storage semantics for bill export and recovery.
Outbound reference category: governance and risk.
`docs/governance/risk-register-2026-05-20.md` mentions cloud-storage in residency, KMS, search, analytics, evidence, and audit-chain risks.
Service-local docs do not provide matching risk treatment or incident runbooks.
Outbound reference issue severity: P2.
Outbound reference category: quality gates.
`docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md:64` names provider-specific import outside an adapter crate as a failure mode using `aws_sdk_s3::Client` in `cloud-storage-kernel`.
Service-local docs do not include an adapter-boundary rule preventing provider SDK imports in domain/kernel layers.
Outbound reference issue severity: P1.
Reference-to-this-service category: documentation coverage.
`docs/DOC-COVERAGE.md:130-139` marks cloud-storage as a red stub.
`docs/architecture/agent-deliverable-verification-audit-2026-05-20.md:552-558` shows cloud-storage benchmarks and tenant_class profiles below floor.
`docs/architecture/agent-deliverable-verification-audit-2026-05-20.md:1804-1805` says cloud-storage benchmarks need object-store workload dimensions and tenant_class profiles need retention and object limits.
`docs/architecture/wave-3-final-scorecard-2026-05-20.md:2159-2167` says cloud-storage is thin, has 7 artifacts, missing threat model, and has low artifact count.
`docs/architecture/wave-3-final-scorecard-2026-05-20.md:4767-4886` names cloud-storage cross-service integration and threat-marker gaps.
Reference-to-this-service severity: P2 for coverage, P1 for threat and integration gaps in cloud-infra context.
Orphan reference: root contracts are bound to crates, but no service-local contract inventory points back to them.
Missing reverse reference: cloud-storage should reference cloud-billing export and cloud-data PITR expectations but does not.
Missing reverse reference: cloud-storage should reference cloud-kms key wrapping and cloud-iam/Cedar principal checks but only describes them informally.
Missing reverse reference: cloud-storage should reference cloud-region or residency constraints for bucket/replication placement.
Missing reverse reference: cloud-storage should reference cloud-observability and audit-chain event emission for data access logs.
Dimension verdict: partial with P1 handoff gaps.

### §3.3 Dimension 3 - Substance Bar: Intern Buildability

Intern-buildability question: can a cold intern build, test, deploy, and operate cloud-storage from the current service-local docs alone?
Answer: no.
Citation: documentation-rigor requires intern-buildability at `docs/standards/documentation-rigor.md:133-141`.
Citation: documentation-rigor requires full microservice suite artifacts at `docs/standards/documentation-rigor.md:62-81`.
Buildability gap 1: no service-local PRD exists, so a new engineer cannot determine authoritative scope across object, block, file, archive, and backup.
Buildability gap 2: no service-local ARCHITECTURE exists, so a new engineer cannot determine metadata-store, object-data, replication, erasure-coding, and control-plane boundaries.
Buildability gap 3: no README exists, so there is no entrypoint for local setup, dependency graph, build commands, or test commands.
Buildability gap 4: no source code exists under `microservices/cloud-storage/src`.
Buildability gap 5: no tests exist under `microservices/cloud-storage/tests`.
Buildability gap 6: the embedded reference implementation is Markdown, not a buildable crate.
Buildability gap 7: no `Cargo.toml` exists under the service path.
Buildability gap 8: no canonical `cargo build --workspace --release --all-features --locked` invocation is tied to this service.
Buildability gap 9: reference implementation uses `cargo run --release` at `reference-implementations/lifecycle-versioning-replication-rust-sdk.md:217-221`, which is narrower than the required workspace build.
Buildability gap 10: no local contract files exist despite root API bindings.
Buildability gap 11: no local OpenAPI/AsyncAPI/Protobuf schemas are owned under the service path.
Buildability gap 12: no SLO files exist despite root SLO claims for p99 metadata GET, S3 compatibility, and durability at `docs/products/cloud/PRD.md:161-167`.
Buildability gap 13: no failure-mode document exists, so split-brain replication, object-lock bypass, KMS deletion, erasure rebuild, and index corruption are not specified.
Buildability gap 14: no incident-response document exists, so on-call action during data unavailability, elevated error rate, or retention policy misfire is undefined.
Buildability gap 15: no capacity model exists, so bucket count, object count, object size, prefix rate, erasure overhead, replication queues, and metadata partitions are not computable.
Buildability gap 16: no cost-budget document exists, so demo_trial/paid baseline profile/paid production profile/paid regulated profile cost envelopes cannot reconcile with OCI Always Free.
Buildability gap 17: no DPIA/compliance document exists, although object lock, legal hold, residency, and evidence retention are compliance-heavy.
Buildability gap 18: no runbooks exist, despite storage being an operationally critical data service.
Buildability gap 19: no iac directory exists, so the intern cannot provision any context.
Buildability gap 20: no supported OS manifest exists, so packaging and CI targets cannot be inferred.
Buildability gap 21: onboarding starts with `make dev-cell.up` at `onboarding/storage-engineer-first-week.md:22-27`, but no service-local Make target is present.
Buildability gap 22: onboarding uses `./bin/oya` commands at `onboarding/storage-engineer-first-week.md:29-52`, but no command contract is service-local.
Buildability gap 23: tutorial requires AWS CLI and boto3 at `tutorials/multi-class-lifecycle-versioning-and-replication.md:7-10`, which conflicts with Rust-strict build doctrine unless framed as external compatibility only.
Buildability gap 24: migration requires AWS CLI, jq, and shell loops at `migration-playbooks/from-s3-and-azure-blob.md:10-18`.
Buildability gap 25: migration says reads from S3 via boto3 and Azure SDK at `migration-playbooks/from-s3-and-azure-blob.md:101-110`.
Buildability gap 26: FAQ says existing apps use boto3, Azure Blob SDK, and gcloud-python at `faqs/storage-engineer-faq.md:7-11`.
Buildability gap 27: no CI lane spec connects the root crate tests in `registry/openapi/runtime-bindings.tsv:9-11` back to service ownership.
Buildability gap 28: no service-local threat model exists; scorecard explicitly marks cloud-storage threat model absent at `docs/architecture/wave-3-final-scorecard-2026-05-20.md:2159-2167`.
Buildability gap 29: no cross-service integration scenario exists; scorecard names cross-service integration absent at `docs/architecture/wave-3-final-scorecard-2026-05-20.md:4767`.
Buildability gap 30: benchmark reproducibility claims a missing evidence directory.
Buildability gap 31: tenant_class policy lacks object-size ceilings even though FAQ names paid baseline profile and paid production profile object-size limits.
Buildability gap 32: tenant_class policy lacks retention duration bounds even though Object Lock is central.
Buildability gap 33: tenant_class policy lacks per-context capacity deltas.
Buildability gap 34: tenant_class policy lacks OCI demo_trial Always Free reconciliation.
Buildability gap 35: docs do not tell the intern when to use MinIO, Ceph, SeaweedFS, OCI Object Storage, S3, or Azure Blob as adapters.
Buildability gap 36: docs do not define bucket namespace uniqueness rules.
Buildability gap 37: docs do not define object key consistency and listing semantics.
Buildability gap 38: docs do not define multipart upload state recovery.
Buildability gap 39: docs do not define checksum algorithms beyond the example.
Buildability gap 40: docs do not define KMS key deletion or KMS unavailability behavior.
Buildability gap 41: docs do not define legal hold precedence over lifecycle expiration.
Buildability gap 42: docs do not define quota enforcement location.
Buildability gap 43: docs do not define event schema for write, read, delete, retention denial, lifecycle transition, replication lag, or inventory export.
Buildability gap 44: docs do not define billing meter names for object storage, block storage, replication, egress, lifecycle transitions, or restore operations.
Buildability gap 45: docs do not define local developer fixtures.
Buildability gap 46: docs do not define upgrade/migration between tiers.
Buildability gap 47: docs do not define data deletion verification and crypto-shred evidence.
Buildability gap 48: docs do not define restore test evidence.
Buildability gap 49: docs do not define air-gapped on-prem operation.
Buildability gap 50: docs do not define colo rack, power, or storage media assumptions.
Dimension verdict: not intern-buildable.
Dimension classification: P2 for missing doc set; P1 for wrong required tools and handoff gaps.

### §3.4 Dimension 4 - Canonical-Direction Alignment

Constraint 1: multi-context deployment.
Canonical citation: ADR-0328 requires every Phase-0 cloud service to declare support across six contexts at `ADR-0328:1730-2214`.
Current artifact state: no manifest and no context matrix exist under `microservices/cloud-storage`.
Classification: drifted-fixable, P1.
Constraint 2: OpenTofu IaC.
Canonical citation: ADR-0328 requires OpenTofu dirs per context at `ADR-0328:2241-2494`.
Current artifact state: no `iac/` directory exists under cloud-storage.
Classification: drifted-fixable, P1.
Constraint 3: OS support matrix.
Canonical citation: master plan names Tier-1, Tier-2, and out-of-scope OSes at `specs/master-plan-sequencing.json:777-815`.
Current artifact state: no `supported-oses.json`, no package matrix, and no OS CI lanes exist.
Classification: drifted-fixable, P2.
Constraint 4: Rust-strict language policy.
Canonical citation: ADR-0328 allows Rust backend and narrow frontend languages while forbidding Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, F#, and C++ at `ADR-0328:3938-4001`.
Current artifact state: no forbidden source files were found under the service path.
Current artifact state: docs prescribe boto3, gcloud-python, aws-sdk-go-v2, AWS CLI, shell, and jq in operational paths.
Classification: actual source aligned; docs and implementation path drifted-fixable, P1.
Constraint 5: OCI Always Free.
Canonical citation: OCI Always Free has 4 OCPU, 24GB memory, 200GB block, 10GB object, 10GB archive, and service module requirements at `ADR-0328:3441-3754`.
Current artifact state: no `iac/oci-guest/always-free/` exists.
Current artifact state: demo_trial tier says 50GB per tenant and about $15/mo at `ADR-0329/ADR-0330/ADR-0331:21-31`.
Classification: drifted-fixable, P1.
Constraint 6: audit decision tree.
Canonical citation: ADR-0328 says missing inspection artifacts should be recorded and not concealed at `ADR-0328:4041-4146`.
Current artifact state: this audit records the missing artifacts explicitly.
Classification: aligned for audit behavior.
Constraint 7: documentation-rigor.
Canonical citation: `docs/standards/documentation-rigor.md:62-81` expects full platforms.
Current artifact state: only seven docs exist.
Classification: drifted-fixable, P2.
Constraint 8: brief-template anti-patterns.
Canonical citation: `docs/standards/brief-template.md:1727-1793` rejects scaffold-only and line-count-as-completion.
Current artifact state: current service docs are not empty, but the suite is too thin and lacks ownership spine.
Classification: partial, P2.
Constraint 9: provider agnosticism.
Canonical citation: memory file `feedback_multi_context_provider_agnostic_2026_05_20.md:28-38` requires context-aware product docs.
Current artifact state: provider-specific commands dominate migration and tutorial content.
Classification: drifted-fixable, P1.
Constraint 10: zero handroll.
Canonical citation: memory file `feedback_zero_handroll_opentofu_only_2026_05_20.md:16-35` requires cloud-iac and OpenTofu, not manual setup.
Current artifact state: service docs use `./bin/oya` manual creation and shell loops.
Classification: drifted-fixable, P1.
Dimension verdict: not aligned enough for Wave-14 aggregation without remediation.

### §3.5 Dimension 5 - Industry-Counterpart Parity

Headline finding: partial.
AWS S3 counterpart surface includes buckets, objects, prefixes, tags, inventory, batch operations, versioning, MFA Delete, replication, RTC, multi-region access points, Object Lock, storage classes, access points, IAM integration, block public access, encryption, event notifications, CloudTrail/CloudWatch, query-in-place, transfer acceleration, Snow-family offline migration, 3,500 write requests per second per prefix, and 5,500 read requests per second per prefix.
AWS source citation: official AWS S3 overview at `https://aws.amazon.com/documentation-overview/s3/` and S3 storage-class guide at `https://docs.aws.amazon.com/AmazonS3/latest/userguide/storage-class-intro.html`.
Google Cloud Storage counterpart surface includes projects, buckets, objects, folders, managed folders, IAM, encryption, soft delete, object versioning, bucket lock, object retention lock, Pub/Sub notifications, audit logs, Storage Insights, batch operations, inventory, object contexts, Rapid Bucket, hierarchical namespace, FUSE, HMAC interoperability, strong consistency, 1000 write requests/sec initial bucket capacity, 5000 read requests/sec initial bucket capacity, turbo replication, and cross-bucket replication.
Google source citation: official Cloud Storage overview at `https://docs.cloud.google.com/storage/docs/introduction`, request-rate guide at `https://docs.cloud.google.com/storage/docs/request-rate`, and availability/durability guide at `https://docs.cloud.google.com/storage/docs/availability-durability`.
Azure Blob Storage counterpart surface includes storage accounts, containers, blobs, block/blob/page/append model, Data Lake Gen2 hierarchical namespace, hot/cool/cold/archive/smart tiers, lifecycle management, immutable storage, versioning, snapshots, change feed, object replication, private endpoints, SFTP, NFS 3.0, high-scale account limits, and 20k/40k requests per second account targets.
Azure source citation: official Azure Blob introduction at `https://learn.microsoft.com/en-us/azure/storage/blobs/storage-blobs-introduction`, access tiers at `https://learn.microsoft.com/en-us/azure/storage/blobs/access-tiers-overview`, scalability targets at `https://learn.microsoft.com/en-us/azure/storage/common/scalability-targets-standard-account`, and object replication overview at `https://learn.microsoft.com/en-us/azure/storage/blobs/object-replication-overview`.
Oyatie-present local capability: buckets.
Oyatie-present local capability: object put/get/list/delete.
Oyatie-present local capability: versioning.
Oyatie-present local capability: object lock and legal hold concepts.
Oyatie-present local capability: lifecycle transitions.
Oyatie-present local capability: storage classes Hot/Warm/Cold/Archive.
Oyatie-present local capability: replication.
Oyatie-present local capability: S3-compatible endpoint.
Oyatie-present local capability: presigned URLs.
Oyatie-present local capability: multipart upload.
Oyatie-present local capability: KMS wrapping.
Oyatie-present local capability: Cedar resource policy.
Oyatie-present local capability: inventory reports.
Oyatie-present local capability: durability targets.
Oyatie-present local capability: migration from S3 and Azure Blob.
Counterpart gap: S3 Batch Operations equivalent is not specified.
Counterpart gap: S3 Storage Lens or GCS Storage Insights equivalent is not specified.
Counterpart gap: GCS soft delete is not specified.
Counterpart gap: Azure change feed equivalent is not specified.
Counterpart gap: Azure snapshots are not specified.
Counterpart gap: Azure Data Lake hierarchical namespace equivalent is not specified.
Counterpart gap: SFTP and NFS access surfaces are not specified, despite root file API scope.
Counterpart gap: block volume API has root references but no service-local implementation plan.
Counterpart gap: file share API has root references but no service-local implementation plan.
Counterpart gap: S3 Object Lambda or object transform equivalent is absent.
Counterpart gap: S3 Multi-Region Access Points equivalent is absent.
Counterpart gap: GCS Rapid Bucket or zonal bucket performance model is absent.
Counterpart gap: Azure private endpoint and network ACL model is absent.
Counterpart gap: GCS bucket IP filtering/public access prevention equivalents are absent.
Counterpart gap: object tags/labels/object context model is under-specified.
Counterpart gap: storage analytics and recommendations are absent.
Counterpart gap: requester-pays or cost-attribution at access time is absent.
Counterpart gap: batch inventory export schema is absent.
Counterpart gap: bucket namespace/domain verification is absent.
Counterpart gap: data transfer acceleration/offline import is absent.
Additive Oyatie capability: Cedar policy as first-class resource policy.
Additive Oyatie capability: per-object KMS shred binding from root PRD.
Additive Oyatie capability: audit-chain evidence hooks.
Additive Oyatie capability: OCI Always Free demo_trial target, but not implemented locally.
Additive Oyatie capability: cross-service use by cloud-billing and cloud-data.
Dimension verdict: partial parity, no union-coverage claim allowed.

### §3.6 Dimension 6 - Multi-Context Deployment Support

Canonical citation: ADR-0328 §D-15 requires six contexts and service-local context support declarations.
Context `oyatie-public-cloud`: in scope.
Context `oyatie-public-cloud` current state: no `iac/oyatie-public-cloud/`, no context-specific tier deltas, no SLO overlay.
Context `oyatie-public-cloud` classification: supported in root intent, missing IaC locally, P1.
Context `guest-on-aws`: in scope.
Context `guest-on-aws` current state: AWS S3 migration is documented, but no `iac/guest-on-aws/` and no "AWS backing only through adapter" rule.
Context `guest-on-aws` classification: partial, missing IaC, P1.
Context `guest-on-oci`: in scope.
Context `guest-on-oci` current state: no `iac/oci-guest/`, no `iac/oci-guest/always-free/`, no OCI Object Storage 10GB object/10GB archive budget.
Context `guest-on-oci` classification: missing, P1.
Context `on-prem`: in scope.
Context `on-prem` current state: FAQ says MinIO Enterprise cluster, but no on-prem OpenTofu, hardware profile, offline update, or KMS/HSM dependency mapping.
Context `on-prem` classification: partial concept, missing IaC, P1.
Context `colo`: in scope.
Context `colo` current state: no rack, AZ, power, media, or network assumptions.
Context `colo` classification: missing, P1.
Context `oyatie-as-cloud-provider`: in scope.
Context `oyatie-as-cloud-provider` current state: FAQ says "not a public S3 alternative" at `faqs/storage-engineer-faq.md:7-11`, which conflicts with IaaS-provider posture.
Context `oyatie-as-cloud-provider` classification: product-position drift, P1.
Forbidden pattern check: provider SDK calls in business logic cannot be proven because service-local source is absent.
Forbidden pattern check: docs prescribe AWS S3 adapter and boto3 paths; this is acceptable as customer compatibility only if clearly separated from implementation.
Forbidden pattern check: paid baseline profile tier baseline names AWS Glacier adapter, making AWS a tier dependency rather than a context adapter.
Forbidden pattern check: no direct cloud-vendor API call in service-local Rust source was found because no source exists.
Forbidden pattern check: no deployment manifest lists correctly N/A contexts.
No context is correctly N/A under current canonical direction.
All six contexts require remediation artifacts.
Dimension verdict: P1 multi-context gap.

### §3.7 Dimension 7 - OpenTofu IaC Coverage

Canonical citation: ADR-0328 §D-16 requires OpenTofu and forbids Terraform/Pulumi/CloudFormation/ARM/Bicep/shell bootstrapping.
IaC directory inventory: no `microservices/cloud-storage/iac/` directory exists.
Expected directory missing: `iac/oyatie-public-cloud/`.
Expected directory missing: `iac/guest-on-aws/`.
Expected directory missing: `iac/oci-guest/`.
Expected directory missing: `iac/on-prem/`.
Expected directory missing: `iac/colo/`.
Expected directory missing: `iac/oyatie-iaas/`.
Expected directory missing: `iac/oci-guest/always-free/`.
Expected file missing per context: `main.tf`.
Expected file missing per context: `providers.tf`.
Expected file missing per context: `variables.tf`.
Expected file missing per context: `outputs.tf`.
Expected file missing per context: `README.md` or equivalent generated usage.
Expected signing missing: `module-signing.json` or equivalent sigstore/cosign evidence.
Expected state backend missing: object-store backend declaration per context.
Terraform references: Google official docs include Terraform, but service-local files do not contain Terraform strings.
Pulumi references: none found in service-local files.
CloudFormation references: none found in service-local files.
ARM/Bicep references: none found in service-local files.
`null_resource` references: none found.
`local-exec` references: none found.
`remote-exec` references: none found.
SSH provisioner references: none found.
Hand-edited tfstate references: none found.
Unsigned module evidence: all modules absent, so signing evidence is absent.
OpenTofu references: none found in service-local files.
Manual setup pattern: onboarding uses `make dev-cell.up` and `./bin/oya` commands.
Manual setup pattern: migration playbook uses AWS CLI discovery and imperative bucket creation.
Manual setup severity: P1 because cloud-storage is deployment substrate and docs need OpenTofu provisioning paths.
Sigstore wiring: absent.
ADR-0039 signing connection: absent.
State backend `oyatie-public-cloud`: absent.
State backend `guest-on-aws`: absent.
State backend `guest-on-oci`: absent.
State backend `on-prem`: absent.
State backend `colo`: absent.
State backend `oyatie-as-cloud-provider`: absent.
Tenant onboarding with `tofu init`: absent.
Tenant onboarding with `tofu plan`: absent.
Tenant onboarding with `tofu apply`: absent.
Tenant onboarding evidence events: absent.
Drift detection: absent.
Plan/apply CI: absent.
Policy-as-code gate for IaC: absent.
Capacity budget outputs: absent.
OCI zero-cost billing output: absent.
Dimension verdict: P1 OpenTofu coverage gap.

### §3.8 Dimension 8 - OS Support Matrix

Canonical citation: ADR-0328 §D-17 and master plan require Tier-1, Tier-2, and out-of-scope OS declarations.
Manifest presence: no `supported-oses.json`.
Manifest format: absent.
Alternative manifest field: no `supported_oses` field found under service path.
Tier-1 OS `talos-linux`: status absent.
Tier-1 OS `rhel`: status absent.
Tier-1 OS `oracle-linux`: status absent.
Tier-1 OS `sles`: status absent.
Tier-1 OS `ubuntu`: status absent.
Tier-1 OS `debian`: status absent.
Tier-1 OS `rocky-linux`: status absent.
Tier-1 OS `almalinux`: status absent.
Tier-1 OS `centos-stream`: status absent.
Tier-1 OS `amazon-linux`: status absent.
Tier-1 OS `flatcar`: status absent.
Tier-1 OS `photon-os`: status absent.
Tier-1 OS `macos-m5-apple-silicon`: status absent.
Tier-2 OS/arch `ppc64le`: status absent.
Tier-2 OS/arch `s390x`: status absent.
Out-of-scope `macos-intel`: no explicit exclusion.
Out-of-scope `macos-pre-m5`: no explicit exclusion.
Out-of-scope `freebsd`: no explicit exclusion.
Out-of-scope `openbsd`: no explicit exclusion.
Out-of-scope `windows-server`: no explicit exclusion.
Out-of-scope `solaris`: no explicit exclusion.
Package format `RPM`: absent.
Package format `DEB`: absent.
Package format `.pkg`: absent.
Package format `Homebrew`: absent.
Package format `Talos extension`: absent.
Package format `Flatcar extension`: absent.
Package format `container image`: absent.
CI lane `talos-linux`: absent.
CI lane `rhel`: absent.
CI lane `oracle-linux`: absent.
CI lane `sles`: absent.
CI lane `ubuntu`: absent.
CI lane `debian`: absent.
CI lane `rocky-linux`: absent.
CI lane `almalinux`: absent.
CI lane `centos-stream`: absent.
CI lane `amazon-linux`: absent.
CI lane `flatcar`: absent.
CI lane `photon-os`: absent.
CI lane `macos-m5`: absent.
Architecture coverage `x86_64`: absent.
Architecture coverage `aarch64`: absent.
Architecture coverage `ppc64le`: absent.
Architecture coverage `s390x`: absent.
Service-local implication: object data plane likely runs on Linux nodes, but no doc says which Tier-1 OSes are release targets.
Service-local implication: macOS M5 may be relevant only for SDK/CLI/frontend or dev tooling, but no exception is declared.
Dimension verdict: P2 OS matrix gap; P1 if any deployment claim is made without OS support.

### §3.9 Dimension 9 - Rust-Strict Language Coverage

Source-file grep scope: `microservices/cloud-storage`.
Forbidden file result: no `.py` files found.
Forbidden file result: no `.js` files found.
Forbidden file result: no `.ts` files found.
Forbidden file result: no `.rb` files found.
Forbidden file result: no `.go` files found.
Forbidden file result: no `.java` files found.
Forbidden file result: no `.scala` files found.
Forbidden file result: no `.groovy` files found.
Forbidden file result: no `.php` files found.
Forbidden file result: no `.fs` or `.fsx` files found.
Authorized file type result: `.md` files only.
Whitelisted status: Markdown docs are allowed.
Rust implementation status: embedded Rust appears in `reference-implementations/lifecycle-versioning-replication-rust-sdk.md:30-215`.
Rust implementation limitation: embedded Markdown code is not a buildable source tree.
Build invocation expected: `cargo build --workspace --release --all-features --locked` per ADR-0328 §D-20.
Build invocation present: not found.
Reference build invocation present: `cargo run --release` at `reference-implementations/lifecycle-versioning-replication-rust-sdk.md:217-221`.
Reference test invocation present: `cargo test --features hermetic` at `reference-implementations/lifecycle-versioning-replication-rust-sdk.md:246-253`.
Frontend directory status: no `frontend/` directory exists.
Swift frontend status: absent.
Kotlin frontend status: absent.
WinUI3 frontend status: absent.
Unauthorized implementation guidance: FAQ says boto3, Azure Blob SDK, gcloud-python at `faqs/storage-engineer-faq.md:7-11`.
Unauthorized implementation guidance: onboarding says boto3 and aws-sdk-go-v2 Just Work at `onboarding/storage-engineer-first-week.md:187`.
Unauthorized implementation guidance: tutorial requires boto3 at `tutorials/multi-class-lifecycle-versioning-and-replication.md:7-10`.
Unauthorized implementation guidance: migration uses boto3 and Azure SDK at `migration-playbooks/from-s3-and-azure-blob.md:101-110`.
Unauthorized shell tooling guidance: migration uses `jq` and shell loop at `migration-playbooks/from-s3-and-azure-blob.md:10-18`.
Unauthorized shell tooling guidance: tutorial uses `jq` at `tutorials/multi-class-lifecycle-versioning-and-replication.md:97-105` and `:184-195`.
Classification nuance: external customer SDK compatibility can mention non-Rust clients, but implementation and operator tooling must be Rust-first or explicitly external-only.
Current docs do not make that boundary explicit.
Dimension verdict: source tree aligned by absence; docs drift, P1.

## §4 Findings Summary

| Severity | Dimension | Finding | Citation | Remediation hint |
|---|---|---|---|---|
| P1 | D1 | paid production profile replication is both async and sync depending on artifact | `faqs/storage-engineer-faq.md:64-66`; `migration-playbooks/from-s3-and-azure-blob.md:81-83`; `benchmarks/...:60-64` | Pick async or sync per tenant_class and update benchmark, migration, and SLOs |
| P1 | D1 | Lifecycle scan cadence contradicts tier transition invariant | `ADR-0329/ADR-0330/ADR-0331:107-110`; `faqs/storage-engineer-faq.md:134-136` | Define per-tenant_class lifecycle SLO and daemon cadence |
| P1 | D1 | demo_trial cost conflicts with OCI Always Free doctrine | `ADR-0329/ADR-0330/ADR-0331:29-31`; `ADR-0328:3441-3466` | Split OCI demo_trial Always Free from paid baseline profile overlays |
| P1 | D1 | Product framing narrows away from public cloud provider posture | `faqs/storage-engineer-faq.md:7-11`; `ADR-0328:1981-2050` | Rewrite as provider-grade storage service with S3 compatibility |
| P1 | D1 | paid baseline profile tier hardcodes AWS Glacier adapter | `ADR-0329/ADR-0330/ADR-0331:37-40`; `ADR-0328:2194-2204` | Move AWS Glacier to optional adapter doc |
| P1 | D2 | cloud-billing depends on storage exports without reverse handoff | `microservices/cloud-billing/tutorials/meter-attribute-invoice-and-export-focus.md:177-196`; service inventory | Add `cross-microservice-handoffs.md` with billing export contract |
| P1 | D2 | cloud-data depends on storage PITR without reverse handoff | `microservices/cloud-data/faqs/data-engineer-faq.md:114-121`; `microservices/cloud-data/ADR-0329/ADR-0330/ADR-0331:62-64` | Add WAL/snapshot restore contract and SLO |
| P1 | D2 | Provider SDK import failure mode lacks service-local guardrail | `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md:64` | Add adapter-boundary policy and CI check |
| P1 | D3 | Required operational docs prescribe non-Rust/bespoke tooling | `tutorials/...:7-10`; `migration-playbooks/...:10-18`; `faqs/...:7-11` | Mark as external compatibility or replace operator path with Rust/OpenTofu |
| P1 | D4 | No six-context manifest exists | `ADR-0328:2079-2084`; service inventory | Add manifest with six contexts and no unsupported claim |
| P1 | D6 | All six contexts lack service-local IaC coverage | `ADR-0328:2277-2296`; service inventory | Add OpenTofu modules per context |
| P1 | D7 | No OpenTofu directory or state backend exists | `ADR-0328:2241-2494`; service inventory | Add `iac/*` with signed modules and context backends |
| P1 | D7 | Manual setup appears as deployment path | `onboarding/...:22-52`; `migration-playbooks/...:39-53` | Replace provisioning with cloud-iac/OpenTofu flow |
| P1 | D9 | Docs prescribe boto3, gcloud-python, aws-sdk-go-v2, jq | `faqs/...:7-11`; `onboarding/...:187`; `tutorials/...:97-105`; `migration-playbooks/...:101-110` | Separate customer SDK compatibility from implementation/operator tooling |
| P2 | D1 | No service-local PRD | `docs/standards/documentation-rigor.md:62-81`; service inventory | Author PRD with object/block/file scope |
| P2 | D1 | No service-local ARCHITECTURE | `docs/standards/documentation-rigor.md:62-81`; service inventory | Author architecture with adapters and data model |
| P2 | D1 | No README | `docs/standards/documentation-rigor.md:62-81`; service inventory | Add entrypoint README |
| P2 | D1 | Benchmark claims measured evidence but referenced path absent | `benchmarks/...:3-5`; `benchmarks/...:124-133`; missing `.foundry/evidence/...` | Reclassify as target numbers or land evidence bundle |
| P2 | D2 | Root cloud PRD capabilities lack local reverse index | `docs/products/cloud/PRD.md:1587-1669`; service inventory | Add capability ownership map |
| P2 | D2 | Root OpenAPI/runtime bindings lack local contract inventory | `registry/openapi/schema-bindings.tsv:41-58`; `registry/openapi/runtime-bindings.tsv:9-11` | Add service-local contracts index |
| P2 | D2 | Prior scorecard already names low artifact count and missing threat model | `docs/architecture/wave-3-final-scorecard-2026-05-20.md:2159-2167` | Add threat model and artifact suite |
| P2 | D3 | No local source tree | service inventory; `reference-implementations/...:30-215` | Move runnable example into Rust crate or link owning crate |
| P2 | D3 | No local test set | `registry/openapi/runtime-bindings.tsv:9-11`; service inventory | Add test plan and service-local test pointers |
| P2 | D3 | No capacity model | `docs/standards/documentation-rigor.md:62-81`; service inventory | Add object count, prefix, throughput, media model |
| P2 | D3 | No failure modes | `docs/standards/documentation-rigor.md:62-81`; service inventory | Add split-brain, KMS loss, lifecycle misfire cases |
| P2 | D3 | No incident response | `docs/standards/documentation-rigor.md:62-81`; service inventory | Add operational runbooks |
| P2 | D4 | Documentation-rigor suite incomplete | `docs/standards/documentation-rigor.md:62-81`; `docs/DOC-COVERAGE.md:130-139` | Build full platform |
| P2 | D5 | Union counterpart parity is partial, not complete | AWS/GCS/Azure official docs cited in §3.5; local inventory | Add missing batch, analytics, soft-delete, HNS, private networking |
| P2 | D8 | No supported OS manifest | `specs/master-plan-sequencing.json:777-815`; service inventory | Add `supported-oses.json` |
| P2 | D8 | No package or CI matrix | `ADR-0328:3877-3927`; service inventory | Add per-OS package formats and CI lanes |
| P2 | D9 | No canonical workspace build invocation | `ADR-0328:3992-3996`; `reference-implementations/...:217-253` | Add build/test contract |
| P3 | D2 | Non-top-3 providers dilute counterpart focus | `benchmarks/...:1-5` | Keep R2/MinIO/B2 appendix, not union bar |
| P3 | D5 | Additive Cedar/KMS/audit-chain surfaces are promising but undocumented | `docs/products/cloud/PRD.md:1587-1665`; local FAQ | Promote additive surfaces into PRD and contracts |
| P3 | D7 | No forbidden Terraform/Pulumi strings found because IaC is absent | local grep | Record clean grep after modules exist |
| P3 | D9 | No forbidden source files found because no source exists | local grep | Keep scan in CI once source lands |

Severity totals:
P0 total: 0.
P1 total: 14.
P2 total: 15.
P3 total: 4.

## §5 Open Questions for Wave 14 Aggregation

Open question 1: Should cloud-storage service-local scope include block and file storage now, or should root `cloud-storage` be split into object, block, and file subservices before PRD repair?
Open question 2: Should MinIO remain a default backing adapter, or should the canonical architecture name an Oyatie-native object engine with MinIO only as a compatibility backend?
Open question 3: Is S3-compatible API parity the public-provider contract, or is native Oyatie Storage API the primary public contract with S3 as one protocol facade?
Open question 4: What is the canonical tier semantic for paid production profile replication: async with p95 lag, sync quorum, or sync only at paid regulated profile?
Open question 5: Is the `aws-s3-glacier-deep` adapter acceptable as a paid baseline profile cold-tier fallback, or must cold/archive always resolve through a provider-neutral archive adapter?
Open question 6: Which service owns Storage Lens/Storage Insights-style analytics: cloud-storage, cloud-observability, cloud-finops, or analytics?
Open question 7: Which service owns billing-meter event schemas for object PUT/GET/storage bytes/replication/restore: cloud-storage or cloud-billing?
Open question 8: Should cloud-storage include GCS HMAC/XML API compatibility as a first-class protocol, or only S3 compatibility plus native API?
Open question 9: Should Azure Blob hierarchical namespace/Data Lake Gen2 parity be implemented in cloud-storage or cloud-data?
Open question 10: What is the minimum OCI Always Free demo_trial feature set when object storage budget is only 10GB and block budget is 200GB?
Open question 11: How should service-local docs reference root contracts and crates without duplicating source of truth?
Open question 12: Should the benchmark file be demoted from measured benchmark to target benchmark until ADR-0212 evidence exists?
Open question 13: Should external customer-client SDK examples mention Python/Go/JavaScript, or must all examples be Rust-only with external SDKs listed in compatibility tables?
Open question 14: Does Oyatie-as-cloud-provider require public object storage endpoint semantics equal to S3/GCS/Azure, including global namespace, DNS, CORS, signed URLs, access logs, and org-level analytics?
Open question 15: Should cloud-storage own a dedicated threat model before any other remediation, given root scorecard names threat markers absent?
Open question 16: Which OpenTofu module owner signs cloud-storage modules: cloud-storage directly or cloud-iac as shared module publisher?
Open question 17: Are R2/MinIO/B2 comparisons desired in appendix after top-3 union coverage, or should they be removed from headline parity docs?
Open question 18: Should block volume support be benchmarked against EBS/Persistent Disk/Azure Managed Disks rather than S3/GCS/Azure Blob?
Open question 19: Should file share support be benchmarked against EFS/Filestore/Azure Files rather than object-store counterparts?
Open question 20: Should this audit's P1 count become a Wave 14 blocker or a Wave 14 remediation queue input?

## Completion Summary

The audit found a narrow but real object-storage documentation cluster and a broad ownership-coherence gap.
The seven existing files provide useful details for versioning, lifecycle, replication, KMS wrapping, inventory, migration, and S3-compatible workflows.
The service path does not yet carry enough artifacts to own cloud-storage across object, block, file, six deployment contexts, OpenTofu IaC, OS support, Rust-strict implementation, or OCI Always Free.
No P0 finding was assigned because cloud-storage is not classified as a P0 HR/ERP/CRM service in the audit inputs.
P1 findings are concentrated in canonical-direction contradictions, provider/tooling drift, missing six-context/OpenTofu support, and cross-service dependency gaps.
P2 findings are concentrated in missing ownership artifacts and buildability gaps.
P3 findings are low-severity cleanup and future-proofing items.

<!-- ORCHESTRATOR REPORT
  µservice: cloud-storage
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/cloud-storage/coherence-audit-2026-05-20.md (617 lines)
    - /Users/jasonlee/oyatie/microservices/cloud-storage/feature-parity-matrix-2026-05-20.md (412 lines)
    - /Users/jasonlee/oyatie/microservices/cloud-storage/performance-benchmark-numbers-2026-05-20.md (355 lines)
    - /Users/jasonlee/oyatie/microservices/cloud-storage/tenant-class-deltas-vs-counterparts-2026-05-20.md (397 lines)
  inventory_files_seen: 7
  inventory_lines_read: 1340
  chat_history_matches_processed: 3
  findings_p0: 0
  findings_p1: 14
  findings_p2: 15
  findings_p3: 4
  top_3_counterparts_confirmed: AWS S3 / Google Cloud Storage / Azure Blob Storage
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1781
-->
