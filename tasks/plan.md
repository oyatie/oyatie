# Implementation Plan: Messenger, Mail, Community, Social Microservice Foundations

## Authority and command mapping

SPEC-equivalent inputs are the machine-readable specs/manifests and implementation plans, not a new root `SPEC.md` (root Markdown is pointer-hub only): `/specs/microservices/{messenger,mail,social}.json`, `microservices/{messenger,mail,community,social}/manifest.json`, and the relevant IP files under each microservice directory.

Command flow mapped to repo artifacts:
- `/spec`: canonical JSON specs/manifests plus microservice IP files.
- `/plan`: this file + `tasks/todo.md`.
- `/build`: TDD domain-foundation and usecase/API ChangeSets below.
- `/code-simplify` / ai-slop-cleaner: keep changed files pure domain/API/usecase, no I/O/provider SDKs.
- `/review` / `/ship`: production ship remains NO-GO until REST/OpenAPI/AsyncAPI/proto handlers, persistence, runtime, SLO, CI/CD, and promotion evidence exist.

## Best-practice research handoff

Official/upstream constraints applied: Google SRE SLOs/error budgets (targets need measured SLIs), OpenTelemetry messaging conventions (future boundary telemetry), AWS Well-Architected reliability (tested failover/backpressure before reliability claims), W3C ActivityPub (social federation stays adapter-layer), and IETF mail/JMAP/DMARC standards (protocol parsing stays API/adapter-layer; domain/usecase owns policy invariants).

Honest claim boundary: completed slices can claim **domain/API/usecase foundation invariants implemented and tested** only. They cannot claim production readiness, hyperscaler maturity, SLO attainment, compliance certification, runtime deployment, or cloud deployability.

## Completed parallelizable ChangeSets

### CS-BACKBONE-MSG-001 — Messenger governance invariants
Scope: `crates/oya-connect-messenger-domain/src/{lib.rs,governance.rs}`.
Acceptance: personal E2E non-discoverability; work tenant-DEK/four-eyes audit; attachment/thread/reaction retention inheritance; presence cross-pillar denial; verified anonymity; cross-org DEK isolation.
Verification: `cargo test -p oya-connect-messenger-domain`; clippy for package.

### CS-BACKBONE-MAIL-001 — Mail governance invariants
Scope: `crates/oya-connect-mail-domain/src/{lib.rs,governance.rs}`.
Acceptance: personal mail non-exportability; work tenant-DEK+retention; legal-hold precedence; DMARC quarantine/reject evidence; tracker blocking; Workflow handoff policy basis; AI assist no plaintext/training signal.
Verification: `cargo test -p oya-connect-mail-domain`; clippy for package.

### CS-BACKBONE-SOCIAL-001 — Social domain scaffold
Scope: `crates/oya-connect-social-domain/**`, `Cargo.toml`, catalog, social manifest binding.
Acceptance: immutable context/pillar; context-switch no cross refs; story TTL purge targets; Workflow consent for work crosspost; AR no biometric persistence; collab consent.
Verification: `cargo test -p oya-connect-social-domain`; catalog YAML parse; cargo prefix gate.

### CS-BACKBONE-COMMUNITY-001 — Community post-store domain scaffold
Scope: `crates/oya-community-post-store-domain/**`, `Cargo.toml`, catalog.
Acceptance: employment-sensitive anonymity policy; idempotent votes/self-vote denial; moderation evidence required.
Verification: `cargo test -p oya-community-post-store-domain`; catalog YAML parse; cargo prefix gate.

### CS-BACKBONE-API-USECASE-001 — Protocol-neutral API + usecase foundations
Scope:
- `crates/oya-messenger-message-stream-{api,usecase}/**`
- `crates/oya-mail-mailbox-store-{api,usecase}/**`
- `crates/oya-social-post-composition-{api,usecase}/**`
- `crates/oya-community-post-store-{api,usecase}/**`
- `Cargo.toml`, relevant manifests, root catalog rows, evidence.

Acceptance:
- Messenger: send-message usecase binds principal to author, requires tenant scope/idempotency/audit correlation, preserves personal non-discoverability, and enforces work four-eyes tenant-DEK governance.
- Mail: workflow handoff is refused for Personal context, requires lawful basis/policy snapshot, rejects plaintext/training AI assist, and maps DMARC reject/quarantine decisions.
- Social: post composition binds creator to principal, enforces work crosspost consent, and converts expired stories to explicit CDN/search/ontology purge targets.
- Community: post creation carries tenant/policy/idempotency context, Teamblind disclosure policy, idempotent votes, and moderation evidence.

Verification:
- `cargo test` across 12 touched domain/API/usecase packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- `oya gate validate cargo-prefix`, `dependency-seam`, and `honest-claims` on the scoped corpus.

### CS-BACKBONE-REST-001 — Framework-free REST/OpenAPI boundary foundations
Scope:
- `crates/oya-messenger-message-stream-rest/**`
- `crates/oya-mail-mailbox-store-rest/**` plus the mail API/usecase submit-message seam
- `crates/oya-social-post-composition-rest/**`
- `crates/oya-community-post-store-rest/**`
- `Cargo.toml`, relevant manifests, root catalog rows, evidence.

Acceptance:
- Messenger: `POST /channels/{channel_id}/messages` route constant and handler map OpenAPI-shaped headers/body to the message-stream usecase, require idempotency/audit correlation, and return 201 receipts.
- Mail: `POST /messages` route constant and handler map OpenAPI-shaped submit-message requests to mailbox-store usecase, require work tenant scope, and return 202 submission receipts; hard DMARC reject remains refused.
- Social: `POST /posts` route constant and handler map OpenAPI-shaped publish requests to post-composition usecase, return 201 receipts, and preserve work crosspost consent refusal.
- Community: `POST /spaces/{space_id}/posts`, `POST /posts/{post_id}/vote`, and `POST /moderation/actions` route constants and handlers map to post-store usecases, preserving principal-bound votes and moderation evidence requirements.

Verification:
- `cargo test` across 16 touched domain/API/usecase/REST packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- `oya gate validate cargo-prefix`, `dependency-seam`, and `honest-claims` on the scoped corpus.

### CS-BACKBONE-POLICY-001 — Cedar policy-decision seams at API boundaries
Scope:
- `crates/oya-messenger-message-stream-{api,usecase,rest}/**`
- `crates/oya-mail-mailbox-store-{api,usecase,rest}/**`
- `crates/oya-social-post-composition-{api,usecase,rest}/**`
- `crates/oya-community-post-store-{api,usecase,rest}/**`
- evidence.

Acceptance:
- Messenger, mail, and social authorized contexts now require a policy decision reference and reject cross-context personal requests that carry tenant scope.
- Community authorized context now has a typed missing-policy-decision error instead of falling through to generic invalid input.
- REST contexts propagate policy decision references into usecase receipts for audit-chain continuity.
- Negative API/REST tests cover missing policy decisions and personal/work scope drift.

Verification:
- `cargo test` across 12 touched API/usecase/REST packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.

### CS-BACKBONE-ROUTE-CATALOG-001 — OpenAPI operation catalogs for REST crates
Scope:
- `crates/oya-messenger-message-stream-rest/**`
- `crates/oya-mail-mailbox-store-rest/**`
- `crates/oya-social-post-composition-rest/**`
- `crates/oya-community-post-store-rest/**`
- evidence.

Acceptance:
- Messenger REST publishes metadata for all 26 OpenAPI operations, while marking only the implemented send-message handler as implemented.
- Mail REST publishes metadata for all 15 OpenAPI operations, while marking only the implemented submit-message handler as implemented.
- Social REST publishes metadata for all 27 OpenAPI operations, while marking only the implemented publish-post handler as implemented.
- Community REST publishes metadata for all 22 OpenAPI operations, while marking only the implemented create/vote/moderation handlers as implemented.
- Contract-only routes are explicit non-claims, preventing hidden stub/false-green handler claims.

Verification:
- `cargo test` across the 4 touched REST packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.

### CS-BACKBONE-PERSISTENCE-001 — Postgres/Citus migration bundles for write stores
Scope:
- `crates/oya-messenger-message-stream-adapter-postgres/**`
- `crates/oya-mail-mailbox-store-adapter-postgres/**`
- `crates/oya-social-post-composition-adapter-postgres/**`
- `crates/oya-community-post-store-adapter-postgres/**`
- `Cargo.toml`, root catalog rows, evidence.

Acceptance:
- Messenger message-stream Postgres/Citus adapter crate publishes a migration bundle for `messages` and `message_receipts`, with `tenant_id`, `home_cell`, `shard_key`, `jurisdiction_code`, audit event class, RLS, forced RLS, and Citus distribution by tenant.
- Mail mailbox-store Postgres/Citus adapter crate publishes a migration bundle for `mail_messages` and `mail_submission_receipts`, with the same tenant/cell/shard and RLS/distribution invariants.
- Social post-composition Postgres/Citus adapter crate publishes a migration bundle for `posts` and `story_purge_targets`, with explicit story purge storage and the same storage invariants.
- Community post-store Postgres/Citus adapter crate publishes a migration bundle for `posts`, `votes`, and `moderation_actions`, with vote/moderation evidence storage and the same storage invariants.
- Each crate has deterministic unit checks that fail if Citus, tenant RLS, forced RLS, distribution columns, required tenant/cell/shard columns, or composite primary keys are removed.

Verification:
- `cargo test` across the 4 touched adapter-postgres packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- `oya gate validate shardability --migrations-dir <crate>/migrations` for each new migration directory.

### CS-BACKBONE-REPOSITORY-COMMANDS-001 — Tenant-scoped repository SQL command seams
Scope:
- `crates/oya-shared-postgres-command-kernel/**`
- `crates/oya-messenger-message-stream-adapter-postgres/**`
- `crates/oya-mail-mailbox-store-adapter-postgres/**`
- `crates/oya-social-post-composition-adapter-postgres/**`
- `crates/oya-community-post-store-adapter-postgres/**`
- `Cargo.toml`, root catalog row, evidence.

Acceptance:
- Shared Postgres command kernel models tenant-session setup and typed parameter values without opening a database connection or embedding tenant data in SQL text.
- Messenger, mail, social, and community Postgres adapter crates build deterministic write batches that run tenant scoping before parameterized insert statements.
- Social story purge targets and community post/vote/moderation writes have explicit command builders and fail closed on missing required audit/policy fields.
- Unit tests verify tenant-scope command ordering, parameterized SQL shape, and required-field rejection.

Verification:
- `cargo test` across the shared kernel plus 4 adapter-postgres packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- `oya gate validate cargo-prefix`, `dependency-seam`, `honest-claims`, and `api-semver` on the scoped evidence corpus.

### CS-BACKBONE-MANIFEST-CONSISTENCY-001 — Implemented-layer manifest alignment
Scope:
- `microservices/messenger/manifest.json`
- `microservices/mail/manifest.json`
- `microservices/social/manifest.json`
- evidence and task tracking.

Acceptance:
- Messenger, mail, and social manifests list the `api` layer now that API crates are implemented and cataloged.
- Social manifest also lists the `usecase` layer now that post-composition usecase crates are implemented and cataloged.
- Manifest JSON remains parseable, and every newly declared layer is backed by at least one bounded-context crate suffix.

Verification:
- JSON parse of the three touched manifests.
- Manifest layer/crate consistency check for messenger, mail, social, and community.
- `git diff --check`.

### CS-BACKBONE-TELEMETRY-BINDINGS-001 — Canonical request telemetry bindings for write routes
Scope:
- `crates/oya-shared-hyperscaler-metrics-kernel/src/lib.rs`
- `crates/oya-messenger-message-stream-rest/**`
- `crates/oya-mail-mailbox-store-rest/**`
- `crates/oya-social-post-composition-rest/**`
- `crates/oya-community-post-store-rest/**`
- evidence and task tracking.

Acceptance:
- Shared metrics kernel exposes `RequestTelemetryBinding` so REST crates do not hand-format `oya_<microservice>_*` request/response metric names.
- Messenger send-message, mail submit-message, social publish-post, and community create/vote/moderation routes bind low-cardinality operation IDs to canonical request, success, total response, 5xx, and 429 metric names.
- Operation IDs are validated as lowercase/dotted/dashed/underscored tokens, rejecting empty/uppercase/adjacent separator labels.
- Unit tests verify canonical metric names for all write routes.

Verification:
- `cargo test` across shared metrics kernel and 4 REST crates.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- dependency-seam, honest-claims, data-class known-blocker audit, and diff hygiene.

### CS-BACKBONE-REST-DISPATCH-001 — Fail-closed dispatch for contract-only REST routes
Scope:
- `crates/oya-messenger-message-stream-rest/**`
- `crates/oya-mail-mailbox-store-rest/**`
- `crates/oya-social-post-composition-rest/**`
- `crates/oya-community-post-store-rest/**`
- evidence and task tracking.

Acceptance:
- Every REST crate exposes `dispatch_contract_only_route(method, path)` for OpenAPI-declared but not-yet-implemented routes.
- Known contract-only routes return HTTP 501 with an explicit non-claim reason instead of silently pretending to be implemented.
- Typed implemented routes are refused by the contract-only dispatcher so callers must use the typed handler and cannot bypass validation/policy seams.
- Unknown routes return a distinct unknown-route error.

Verification:
- `cargo test` across the 4 REST packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-REST-WRITE-DISPATCH-001 — Framework-free dispatch for implemented REST write routes
Scope:
- `crates/oya-messenger-message-stream-rest/**`
- `crates/oya-mail-mailbox-store-rest/**`
- `crates/oya-social-post-composition-rest/**`
- `crates/oya-community-post-store-rest/**`
- evidence and task tracking.

Acceptance:
- Messenger, mail, social, and community REST crates expose method/path write-route dispatch functions for already-implemented business handlers.
- Implemented routes no longer require out-of-band typed entrypoint selection: the framework-free dispatcher maps OpenAPI method/path plus typed payload enum to the existing validated handler.
- Contract-only routes still fail closed and are refused by the write dispatcher instead of returning accidental success.
- Community dispatch covers create-post, cast-vote, and moderation-action handlers; messenger/mail/social dispatch cover their implemented write route handlers.
- Honest non-claim: this remains framework-free typed dispatch; it does not parse JSON, bind Hyper, deploy a gateway, provide TLS, or implement still-contract-only endpoints.

Verification:
- `cargo test` across the 4 touched REST packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- broad backbone cargo check/test/clippy/fmt, dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-PROTOCOL-PARITY-001 — AsyncAPI/proto parity bindings for write receipts
Scope:
- `crates/oya-shared-protocol-parity-kernel/**`
- `crates/oya-messenger-message-stream-api/**`
- `crates/oya-mail-mailbox-store-api/**`
- `crates/oya-social-post-composition-api/**`
- `crates/oya-community-post-store-api/**`
- `Cargo.toml`, root catalog row, evidence and task tracking.

Acceptance:
- Shared protocol parity kernel validates binding metadata that ties one implemented REST operation to one AsyncAPI event operation/channel/message and one proto package/service/RPC.
- Messenger, mail, social, and community API crates expose write-receipt protocol bindings for the implemented write paths.
- Protocol event envelopes carry schema version, tenant/person scope reference, aggregate ID, audit correlation, idempotency, and policy decision references without publishing events or serializing protobufs.
- Unit tests verify AsyncAPI/proto binding names and fail-closed receipt event-type checks.

Verification:
- `cargo test` across shared protocol parity kernel plus the 4 touched API packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.

### CS-BACKBONE-REST-PROBE-READINESS-001 — REST probe readiness and Kubernetes probe path alignment
Scope:
- `crates/oya-messenger-message-stream-rest/**`
- `crates/oya-mail-mailbox-store-rest/**`
- `crates/oya-social-post-composition-rest/**`
- `crates/oya-community-post-store-rest/**`
- `microservices/{messenger,mail,social,community}/iac/k8s/helm/templates/deployment.yaml`
- `microservices/{mail,community}/contracts/openapi/*.yaml`
- evidence and task tracking.

Acceptance:
- Messenger, mail, social, and community REST crates expose framework-free `GET /health` liveness and `GET /ready` readiness dispatch functions.
- Liveness returns a minimal 200 response and explicitly does not claim downstream SQL/outbox/policy/OTel readiness.
- Readiness returns 200 only when caller-supplied dependency evidence is ready, otherwise 503; this keeps readiness testable before a Hyper/Kubernetes runtime exists.
- Mail and community OpenAPI contracts declare `/health` and `/ready` so all four backbone services have consistent probe routes.
- Helm readiness probes point at `/ready`, and liveness probes point at `/health`, matching the declared REST probe routes instead of undeclared `/healthz`/`/livez` paths.

Verification:
- `cargo test` across the 4 touched REST packages.
- `cargo check`, `cargo clippy -D warnings`, and `cargo fmt --check` across the same packages.
- YAML parse for the 4 OpenAPI contracts.
- Static grep proves the four Helm deployments use `/ready` and `/health` and no longer use `/healthz` or `/livez`.
- dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-CI-MATRIX-001 — CI matrix coverage for backbone package slices
Scope:
- `.github/workflows/backbone-microservices-ci.yml`
- `microservices/messenger/ci/Jenkinsfile`
- `microservices/mail/ci/Jenkinsfile`
- `microservices/social/ci/Jenkinsfile`
- `microservices/community/ci/Jenkinsfile`
- evidence and task tracking.

Acceptance:
- GitHub Actions has a dedicated backbone microservice package matrix for messenger, mail, social, and community package slices.
- Per-microservice Jenkinsfiles test the concrete implemented crates instead of a non-existent umbrella package placeholder.
- Both CI surfaces run fmt/check/clippy/test or nextest lanes with fail-fast disabled at the matrix layer where applicable.
- Static checks verify every Jenkins package name exists in the Cargo workspace and that Jenkins parity markers remain present.

Verification:
- YAML parse of the GitHub Actions workflow.
- Static Jenkins package matrix validation against `Cargo.toml` workspace packages.
- `cargo check`, `cargo test`, `cargo clippy -D warnings`, and `cargo fmt --check` across the 23 touched backbone packages.
- diff hygiene and honest-claims on the scoped corpus.

### CS-BACKBONE-APP-ORCHESTRATION-001 — Runtime-neutral app write plans
Scope:
- `crates/oya-messenger-app/**`
- `crates/oya-mail-mailbox-store-app/**`
- `crates/oya-social-app/**`
- `crates/oya-community-post-store-app/**`
- `Cargo.toml`, root catalog rows, CI package matrices, evidence and task tracking.

Acceptance:
- Messenger app plan composes authorized API/usecase send-message validation, tenant-scoped Postgres write batch, and protocol event envelope for the implemented message write path.
- Mail mailbox-store app plan composes submit-message validation, DMARC action mapping, tenant-scoped Postgres write batch, and protocol event envelope.
- Social app plan composes publish-post validation, tenant-scoped Postgres write batch, protocol event envelope, and deterministic story purge-target command planning when expiry has passed.
- Community post-store app plans compose create-post, cast-vote, and moderation-action usecases with tenant-scoped Postgres write batches and protocol event envelopes.
- Every app plan rejects API scope vs tenant routing drift before building persistence commands.

Verification:
- `cargo test` across the 4 app packages.
- `cargo check`, `cargo test`, `cargo clippy -D warnings`, and `cargo fmt --check` across the 27 touched backbone packages.
- YAML/catalog parse, CI/Jenkins package-matrix validation, dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-REST-APP-PLAN-001 — REST write-plan entrypoints for app orchestration
Scope:
- `crates/oya-messenger-message-stream-rest/**`
- `crates/oya-mail-mailbox-store-rest/**`
- `crates/oya-social-post-composition-rest/**`
- `crates/oya-community-post-store-rest/**`
- evidence and task tracking.

Acceptance:
- Messenger, mail, social, and community REST crates expose write-plan entrypoints that map REST-shaped contexts/requests into app-layer plans.
- REST write-plan responses include the same status codes as the receipt-only handlers plus app plans containing receipts, tenant-scoped persistence batches, and protocol event envelopes.
- Existing receipt-only REST handlers remain intact for compatibility while the stronger write-plan functions prepare later gateway/runtime integration.
- Tests verify persistence and protocol event outputs for every implemented write route, including community vote and moderation.

Verification:
- `cargo test` across the 4 touched REST packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- broad 27-package cargo check/test/clippy/fmt, dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-TRANSPORT-PLAN-001 — Runtime-neutral broker/gRPC transport plans
Scope:
- `crates/oya-shared-protocol-transport-kernel/**`
- `crates/oya-messenger-message-stream-grpc/**`
- `crates/oya-mail-mailbox-store-grpc/**`
- `crates/oya-social-post-composition-grpc/**`
- `crates/oya-community-post-store-grpc/**`
- app write-plan crates, root catalog rows, CI package matrices, evidence and task tracking.

Acceptance:
- Shared protocol transport kernel derives AsyncAPI broker publish plans and proto/gRPC unary descriptors from validated protocol event envelopes.
- App write plans now include deterministic broker publish and gRPC unary transport bundles alongside receipts, persistence batches, and protocol event envelopes.
- Messenger, mail, social, and community gRPC crates expose framework-free write-plan entrypoints for the implemented RPCs.
- Tests verify broker channel/event metadata and fully-qualified proto RPC descriptors for messenger, mail, social, and community create/vote/moderation paths.

Verification:
- `cargo test` across the shared transport kernel, 4 app packages, and 4 gRPC packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- broad backbone cargo check/test/clippy/fmt, catalog/CI parsing, dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-PROTOCOL-PAYLOAD-CODEC-001 — Deterministic protocol payload codec seam
Scope:
- `crates/oya-shared-protocol-transport-kernel/**`
- task/evidence tracking.

Acceptance:
- Shared protocol transport kernel encodes validated protocol event envelopes into deterministic `proto-json-v1` payload bytes before transport planning records broker payload size.
- Codec escapes JSON string characters, represents absent idempotency keys as `null`, and enforces broker payload byte budgets against actual encoded bytes.
- Existing transport planning uses encoded payload length instead of a symbolic field-length estimate.
- Honest non-claim: this is a deterministic payload serialization seam only; it is not protobuf code generation, a live broker publish, a gRPC network call, or a delivery guarantee.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-protocol-transport-kernel`.
- broad backbone cargo check/test/clippy/fmt with the transport kernel included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.


### CS-BACKBONE-PROTO-CONTRACTS-001 — Source-controlled proto3 write service contracts
Scope:
- `specs/proto/backbone/**`
- `crates/oya-shared-backbone-proto-contracts-kernel/**`
- workspace/catalog/CI/Jenkins task/evidence wiring.

Acceptance:
- Messenger, mail, social, and community write-service proto3 files declare the implemented package/service/RPC names already used by parity and transport planning.
- The shared proto-contract kernel embeds the source files and validates proto3 syntax, package declarations, service/RPC descriptors, request/response messages, and common tenant/principal/idempotency/policy/audit metadata fields.
- CI and Jenkins package matrices include the proto-contract kernel and path filters cover source-controlled proto contract changes.
- Honest non-claim: these are source-controlled contracts plus deterministic validation only; this is not prost/tonic code generation, generated Rust stubs, protobuf binary serialization, a live gRPC server/client, or a delivery guarantee.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-backbone-proto-contracts-kernel`.
- broad backbone cargo check/test/clippy/fmt with the proto-contract kernel included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, catalog/data-class known-blocker capture, diff hygiene, and Oya VCS verify/done/promote.


### CS-BACKBONE-GRPC-GENERATED-001 — Compile-time tonic/prost generated bindings
Scope:
- `crates/oya-shared-backbone-grpc-generated-adapter/**`
- workspace dependency pins/rationales for prost/tonic/protoc code generation.
- catalog/CI/Jenkins task/evidence wiring.

Acceptance:
- A shared adapter crate compiles the source-controlled backbone proto3 contracts at build time with vendored `protoc`, `tonic-prost-build`, and adapter-scoped prost/tonic dependencies.
- Generated Rust message/service modules exist for messenger, mail, social, and community packages, with package and fully qualified method constants preserved for future runtime adapters.
- Unit tests encode/decode generated prost request messages for every implemented write RPC family, including community create/vote/moderation.
- Dependency rationales keep prost/tonic/protoc tooling isolated to the generated adapter/build script; domain/API/usecase crates remain framework-free.
- Honest non-claim: this compiles generated bindings and proves local prost binary round trips only; it is not a live gRPC server/client, gateway deployment, broker delivery path, database integration, or production compatibility certification.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-backbone-grpc-generated-adapter`.
- broad backbone cargo check/test/clippy/fmt with the generated adapter included in every per-service matrix.
- cargo-deny, license-policy, cargo-prefix, dependency-seam, honest-claims, catalog/data-class known-blocker capture, diff hygiene, and Oya VCS verify/done/promote.


### CS-BACKBONE-GRPC-PLAN-ADAPTER-001 — Generated gRPC request to write-plan adapters
Scope:
- `crates/oya-shared-backbone-grpc-generated-adapter/**`
- catalog/task/evidence tracking.

Acceptance:
- Generated messenger, mail, social, and community request messages convert into the existing framework-free gRPC write-plan boundaries without opening sockets.
- Conversion maps generated enum/oneof fields into API request/envelope types, rejects unspecified/invalid enums and missing generated messages before app execution, and preserves tenant/principal/idempotency/policy/audit metadata.
- Community generated adapters cover create-post, cast-vote, and moderation-action write plans when supplied with the already-loaded post/ledger state required by existing usecases.
- Honest non-claim: this is a generated request-to-write-plan adapter seam only; it is not a tonic server implementation, live client, service gateway, broker publish, database execution, or deployment evidence.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-backbone-grpc-generated-adapter`.
- broad backbone cargo check/test/clippy/fmt with the generated adapter included in every per-service matrix.
- catalog/data-class known-blocker capture, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.


### CS-BACKBONE-GRPC-TRANSPORT-ADAPTER-001 — Live tonic transport server/client socket seam
Scope:
- `crates/oya-shared-backbone-grpc-transport-adapter/**`
- workspace/catalog/CI/task/evidence tracking.

Acceptance:
- Tonic server trait implementations wrap the generated messenger, mail, social, and community write-plan adapters and return generated proto responses without executing SQL or publishing broker messages.
- Public client factories expose generated clients over `tonic::transport::Channel`, and a loopback TCP test exercises messenger, mail, social, community create, community vote, and community moderation RPCs against a real tonic server on `127.0.0.1:0`.
- Community socket handling keeps explicit in-memory post/ledger state for the adapter-only vote/moderation dependency and rejects vote/moderation before a post is loaded.
- Honest non-claim: this is a local tonic transport seam only; it is not TLS/mTLS, gateway admission, production deployment, database execution, broker delivery, long-running daemon supervision, or hyperscaler certification evidence.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-backbone-grpc-transport-adapter`.
- broad backbone cargo check/test/clippy/fmt with the transport adapter included in every per-service matrix.
- cargo-deny, license-policy, cargo-prefix, catalog/data-class known-blocker capture, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-TRANSPORT-ACK-001 — Recording broker/gRPC acknowledgement contract
Scope:
- `crates/oya-shared-protocol-transport-kernel/**`
- task/evidence tracking.

Acceptance:
- Shared protocol transport kernel exposes a `ProtocolTransportExecutor` trait and `RecordingProtocolTransportExecutor` that execute a planned transport bundle into deterministic broker and gRPC acknowledgement reports without network I/O.
- Acknowledgement reports preserve tenant scope, audit correlation, policy decision ref, idempotency key, broker channel/message/partition details, gRPC method/deadline, and deterministic ack refs.
- Negative tests detect broker/gRPC tenant-scope and idempotency drift before recording an acknowledgement.
- Honest non-claim: this is a recording executor contract for runtime integration tests, not a live broker, gRPC server, protobuf serializer, retry loop, or async runtime.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-protocol-transport-kernel`.
- broad backbone cargo check/test/clippy/fmt with the transport kernel included in every per-service matrix.
- dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-TRANSPORT-RETRY-APP-001 — Bounded broker/gRPC transport retry seam
Scope:
- `crates/oya-shared-protocol-transport-retry-app/**`
- workspace/catalog/CI/Jenkins task/evidence wiring.

Acceptance:
- Transport retry app executes an already-planned broker/gRPC transport bundle through an injected retry executor with explicit retryable vs permanent attempt errors.
- Retry policy validates max attempts, base backoff, and max backoff before execution; retryable failures receive capped exponential planned backoff between attempts.
- Permanent errors stop immediately without retry; exhausted retryable attempts stop without a terminal backoff; successful attempts preserve the final transport execution report.
- Adapter wraps the existing protocol transport executor and maps kernel invariant errors to permanent retry outcomes.
- Honest non-claim: this is a bounded retry/dead-letter decision seam only; it performs no live broker publish, gRPC network call, async sleep, process supervision, or delivery guarantee.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-protocol-transport-retry-app`.
- broad backbone cargo check/test/clippy/fmt with the retry app included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.


### CS-BACKBONE-BROKER-HTTP-PUBLISHER-001 — Live HTTP broker-gateway publisher adapter
Scope:
- `crates/oya-shared-outbox-broker-http-adapter/**`
- workspace/catalog/CI/task/evidence tracking.

Acceptance:
- Adapter performs a real TCP HTTP/1.1 POST of deterministic outbox metadata to a configured broker-gateway endpoint using outbox transport-plan channel, event, tenant, policy, idempotency, and audit headers.
- Tests run a local TCP HTTP broker-gateway fixture and verify success ack capture, non-2xx rejection, payload budget enforcement, and config/plan validation before network I/O.
- CI/Jenkins matrices include the broker HTTP adapter in all four backbone service lanes.
- Honest non-claim: this is a local HTTP broker-gateway publish seam only; it is not Kafka/NATS/PubSub vendor integration, TLS/mTLS, broker durability, retries, gRPC execution, database mutation, production deployment, or delivery SLO evidence.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-outbox-broker-http-adapter`.
- broad backbone cargo check/test/clippy/fmt with the broker HTTP adapter included in every per-service matrix.
- cargo-deny, license-policy, cargo-prefix, catalog/data-class known-blocker capture, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-HTTP-OUTBOX-EXECUTOR-001 — HTTP broker publisher outbox executor bridge
Scope:
- `crates/oya-shared-outbox-broker-http-adapter/src/lib.rs`
- `crates/oya-shared-transactional-outbox-dispatch-app/src/lib.rs`
- catalog/task/evidence tracking.

Acceptance:
- `HttpBrokerPublisher` implements `OutboxTransportExecutor` so existing worker-cycle injection can execute a planned outbox broker publish over the live local HTTP broker-gateway adapter.
- HTTP publish success maps the broker ack header into `OutboxTransportAck::broker_ack_ref`, preserves event/tenant/audit/policy/idempotency refs, and records an explicit `grpc:not-executed:<method>:<sequence>` ack ref rather than claiming a gRPC call occurred.
- HTTP non-2xx, invalid config/plan, response parse, and I/O failures map to a first-class dispatch execution failure so the existing worker dead-letter path can classify the event without pretending it was published.
- The HTTP publisher accepts AsyncAPI-style relative channel addresses from outbox metadata while still rejecting absolute URLs, authority-form paths, control characters, spaces, and header injection.
- Honest non-claim: this wires a live HTTP broker publish executor into the outbox dispatch seam only; it is not Kafka/NATS/PubSub vendor integration, not gRPC execution, not SQL mutation, not a running poller daemon, not TLS/mTLS, and not delivery SLO evidence.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-outbox-broker-http-adapter` and `oya-shared-transactional-outbox-dispatch-app`.
- broad backbone cargo check/test/clippy/fmt with the updated executor bridge included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-SQL-EXECUTION-CONTRACT-001 — Postgres pool and SQL execution contracts
Scope:
- `crates/oya-shared-postgres-command-kernel/**`
- app write-plan crates and task/evidence tracking.

Acceptance:
- Shared Postgres command kernel defines bounded pool configuration, ordered SQL execution plans, and a recording executor contract that preserves tenant-scope command ordering.
- Messenger, mail, social, and community app write plans attach SQL execution plans derived from their tenant-scoped write batches.
- Tests verify pool validation, tenant-scope-first execution ordering, and app-level command counts for implemented write routes.

Verification:
- `cargo test` across shared Postgres command kernel and 4 app packages.
- `cargo clippy` across the same packages with `-D warnings`.
- `cargo fmt --check` across the same packages.
- broad backbone cargo check/test/clippy/fmt, dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-SQLX-EXECUTOR-001 — SQLx-backed Postgres command executor adapter seam
Scope:
- `crates/oya-shared-postgres-command-adapter-sqlx/**`
- workspace/dependency registry/catalog rows.
- backbone CI package matrices and task/evidence tracking.

Acceptance:
- Shared SQLx adapter crate owns the external `sqlx` dependency and exposes `SqlxPostgresBatchExecutor` over `PgPool` for executing tenant-scoped `SqlExecutionPlan` values inside one transaction.
- The adapter validates tenant `set_config` ordering, bounded pool configuration, TLS-required database URL posture, SQL statement shape, and `$N` placeholder/parameter count before touching the pool.
- SQLx remains adapter-isolated: command kernel and service domain/API/usecase/app crates remain SQLx-free.
- Backbone GitHub Actions and Jenkins matrices include the SQLx adapter package.
- Honest non-claim: this compiles the real SQLx execution path and validates plans, but live database/RLS/backup/Citus drills still require an environment with Postgres/Citus.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-postgres-command-adapter-sqlx`.
- broad backbone cargo check/test/clippy/fmt with the SQLx adapter included in every per-service matrix.
- dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-LIVE-POSTGRES-RLS-HARNESS-001 — Env-gated live Postgres RLS/Citus harness
Scope:
- `crates/oya-shared-postgres-command-adapter-sqlx/**`
- `registry/dependency-rationales.json`
- task/evidence tracking.

Acceptance:
- SQLx adapter exposes an environment-gated live harness controlled by `OYA_BACKBONE_LIVE_POSTGRES`, `OYA_BACKBONE_POSTGRES_URL`, `OYA_BACKBONE_POSTGRES_REQUIRE_TLS`, and optional `OYA_BACKBONE_REQUIRE_CITUS`.
- Harness creates a disposable probe schema/table, enables and forces PostgreSQL row-level security, creates a tenant policy based on the same `oyatie.tenant_id` transaction setting used by production write plans, inserts two tenant rows through `SqlxPostgresBatchExecutor`, and proves tenant A, tenant B, and unset-tenant visibility counts.
- When Citus is required, the harness first verifies the `citus` extension is installed and then calls `create_distributed_table(..., 'tenant_id', colocate_with => 'none')` before inserting probe rows.
- Default local/CI tests skip the live probe unless explicitly enabled, while still compile-checking the async harness, env parsing, generated insert plan, and safety gates.
- Honest non-claim: this adds a runnable harness but does not claim a live database was available in the default evidence run, nor backup/restore drills, Citus rebalance drills, production RLS rollout, or live data-plane SLO evidence.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-postgres-command-adapter-sqlx`.
- broad backbone cargo check/test/clippy/fmt with the harness compile-checked in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-TRANSACTIONAL-OUTBOX-001 — Transactional outbox command seam
Scope:
- `crates/oya-shared-transactional-outbox-kernel/**`
- messenger/mail/social/community Postgres migration bundles and app write-plan crates.
- backbone CI package matrices, catalog rows, and task/evidence tracking.

Acceptance:
- Shared transactional outbox kernel appends a parameterized `protocol_outbox_events` insert to an existing tenant-scoped SQL write batch after business statements and before SQL execution planning.
- Messenger, mail, social, and community app write plans persist protocol-event dispatch metadata in the same `SqlExecutionPlan` transaction as their business rows.
- Each service migration bundle declares a tenant-distributed, RLS-forced `protocol_outbox_events` table with dispatch state, attempt count, audit, policy, idempotency, AsyncAPI, and proto metadata.
- Tests prove outbox command ordering, tenant/envelope scope-drift refusal, parameterized SQL shape, app-level command-count updates, and migration distribution/RLS validation.
- Honest non-claim: this is a transactional outbox write seam only; no live outbox poller, broker publisher, gRPC server, retry worker, or exactly-once delivery proof is claimed.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for the shared outbox kernel and 4 touched app/adapter packages.
- broad backbone cargo check/test/clippy/fmt with the outbox kernel included in every per-service matrix.
- catalog/workflow/Jenkins parsing, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-OUTBOX-SQLX-DRAIN-001 — SQLx transactional outbox drain adapter seam
Scope:
- `crates/oya-shared-transactional-outbox-adapter-sqlx/**`
- workspace/dependency registry/catalog rows.
- backbone CI package matrices and task/evidence tracking.

Acceptance:
- Shared SQLx outbox adapter crate owns the live Postgres row-claim queries for service-scoped `protocol_outbox_events` tables while keeping kernels/domain/usecase crates SQLx-free.
- Claim SQL uses a fixed table allowlist from `BackboneOutboxTable`, tenant filtering, pending dispatch-state filtering, bounded batch limits, `ORDER BY created_at`, and `FOR UPDATE SKIP LOCKED` for queue-style concurrent consumers.
- The adapter validates worker refs, tenant refs, claim limits, claimed event tenant/service drift, and event-id keys before state mutations.
- Mutation SQL marks claimed rows publishing, published, or dead-letter using tenant/event-id parameters without interpolating runtime input into SQL.
- Honest non-claim: this compiles the outbox drain path and query contract, but no live database, live poller loop, broker publisher, gRPC server, retry worker, or delivery guarantee is claimed.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-transactional-outbox-adapter-sqlx`.
- broad backbone cargo check/test/clippy/fmt with the outbox SQLx adapter included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-OUTBOX-DISPATCH-APP-001 — Runtime-neutral outbox dispatch app seam
Scope:
- `crates/oya-shared-transactional-outbox-dispatch-app/**`
- workspace/catalog/CI/Jenkins task/evidence wiring.

Acceptance:
- Dispatch app maps already-claimed `protocol_outbox_events` rows into dynamic broker publish and gRPC unary transport plans without contacting a broker, gRPC server, or database.
- Batch validation rechecks table, tenant, event-id, service-id, audit, policy, idempotency, AsyncAPI, and proto metadata before executing an injected transport executor.
- Recording executor emits deterministic broker/gRPC acknowledgement refs and recommends `Published` only after transport invariant checks pass.
- Honest non-claim: this is a runtime-neutral dispatch seam over claimed rows; no background poller loop, live broker publisher, live gRPC call, SQL state mutation, retry worker, or delivery guarantee is claimed.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-transactional-outbox-dispatch-app`.
- broad backbone cargo check/test/clippy/fmt with the dispatch app included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-OUTBOX-WORKER-APP-001 — SQLx outbox worker-cycle seam
Scope:
- `crates/oya-shared-transactional-outbox-worker-app/**`
- workspace/catalog/CI/Jenkins task/evidence wiring.

Acceptance:
- Worker app composes one explicit SQLx claim → runtime-neutral dispatch → SQL state mutation cycle over the outbox drain adapter and dispatch app.
- Successful dispatches are marked published only after broker/gRPC transport invariant checks pass through the injected executor.
- Dispatch-planning or transport errors are classified into dead-letter mutation reports with non-empty reasons; worker reports reject count/event-id/state drift.
- Honest non-claim: this compiles a single-cycle worker seam, but no daemon, scheduler, continuously running poller, live broker client, live gRPC client, retry policy, or delivery guarantee is claimed.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-transactional-outbox-worker-app`.
- broad backbone cargo check/test/clippy/fmt with the worker app included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-OUTBOX-POLLER-APP-001 — Bounded outbox poller policy seam
Scope:
- `crates/oya-shared-transactional-outbox-poller-app/**`
- workspace/catalog/CI/Jenkins task/evidence wiring.

Acceptance:
- Poller app owns deterministic scheduler policy for repeated worker cycles: max ticks, idle shutdown, consecutive-error shutdown, and capped error backoff.
- Poller reports per-tick work/idle/error outcomes, planned delays, total claimed/published/dead-letter counts, and total runner errors.
- Work ticks reset idle/error streaks; error ticks reset idle streaks and use capped exponential backoff; idle ticks stop after the configured idle threshold.
- Honest non-claim: this is a bounded scheduler-policy seam only; it performs no sleeping, process spawning, live database I/O, live broker/gRPC I/O, process supervision, or production delivery guarantee.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-transactional-outbox-poller-app`.
- broad backbone cargo check/test/clippy/fmt with the poller app included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-OUTBOX-TOKIO-RUNTIME-001 — Tokio runtime sleeper for bounded outbox poller
Scope:
- `crates/oya-shared-transactional-outbox-runtime-tokio-app/**`
- workspace/dependency-rationale/catalog/CI/Jenkins task/evidence wiring.

Acceptance:
- Tokio runtime app provides an async sleeper backed by `tokio::time::sleep` and an async bounded poller loop over an injected async worker-cycle runner.
- Runtime loop sleeps between non-terminal ticks using the deterministic delay/backoff policy and does not sleep after idle/error/max-tick stop conditions.
- Reports preserve the poller stop reason, per-tick outcomes, planned delays, and aggregate claimed/published/dead-letter/error counts.
- Honest non-claim: this wires a real Tokio delay seam, but no process daemon, live SQLx pool, live broker/gRPC client, process supervisor, deployment, or delivery guarantee is claimed.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-transactional-outbox-runtime-tokio-app`.
- broad backbone cargo check/test/clippy/fmt with the Tokio runtime app included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-OUTBOX-SERVICE-LOOP-001 — Bounded outbox service-loop lifecycle seam
Scope:
- `crates/oya-shared-transactional-outbox-runtime-tokio-app/**`
- catalog task/evidence wiring.

Acceptance:
- Runtime app exposes bounded service-loop config/report types that run one or more Tokio poller epochs over the existing injected async runner, sleeper, and shutdown-signal traits.
- Service loop validates max epochs, epoch pause, and embedded poller config before work starts.
- Service loop checks shutdown before the first epoch and after completed epochs, pauses only between non-terminal epochs, and never pauses after max-epoch or shutdown stop conditions.
- Reports preserve epoch reports plus aggregate tick, claimed, published, dead-letter, and error counts.
- Honest non-claim: this is a lifecycle seam for future daemons; it does not install an OS process daemon, signal handler, supervisor, live SQLx pool, live broker/gRPC client, deployment, or delivery guarantee.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-transactional-outbox-runtime-tokio-app`.
- broad backbone cargo check/test/clippy/fmt with the service-loop tests included in every per-service matrix.
- cargo-deny, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.


### CS-BACKBONE-OUTBOX-SUPERVISOR-LIFECYCLE-001 — Supervised outbox service lifecycle envelope
Scope:
- `crates/oya-shared-transactional-outbox-runtime-tokio-app/src/lib.rs`
- catalog/task/evidence tracking.

Acceptance:
- Tokio outbox runtime exposes a supervisor config/report wrapper that records starting, ready, shutdown-requested, and stopped lifecycle events around the existing bounded service loop.
- Supervisor validation rejects empty service/worker refs and zero readiness/shutdown timeouts before the service loop starts.
- Tests cover max-epoch stop, shutdown-before-work, and invalid supervisor config while preserving the existing no-live-DB/no-live-broker runtime boundary.
- Honest non-claim: this is supervisor lifecycle metadata around the local service loop only; it is not an OS daemon install, systemd/launchd/Kubernetes controller, real SIGTERM handler, broker publish, database execution, or production delivery evidence.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-transactional-outbox-runtime-tokio-app`.
- broad backbone cargo check/test/clippy/fmt with the runtime app included in every per-service matrix.
- catalog/data-class known-blocker capture, dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-CEDAR-RUNTIME-001 — Cedar runtime evaluator and backbone write-policy conformance
Scope:
- `crates/oya-policy-cedar-domain/**`
- backbone CI package matrices and task/evidence tracking.

Acceptance:
- Cedar policy domain exposes a runtime evaluator adapter that maps `AuthzRequest` values into policy-set authorization decisions, deterministic decision refs, and append-only evaluation log entries.
- Backbone write-policy pack covers every implemented messenger, mail, social, and community write action with tenant-scoped allow policies and required backbone data-plane context.
- Conformance tests prove allow coverage, tenant default-deny, explicit-deny precedence, log filtering, missing-audit refusal, and invalid role context refusal.
- Backbone GitHub Actions and Jenkins matrices include the Cedar policy domain package.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-policy-cedar-domain`.
- broad backbone cargo check/test/clippy/fmt with the policy package included in every per-service matrix.
- CI/Jenkins package validation, dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-RUNTIME-OBSERVABILITY-001 — Runtime metrics exercise, SLO burn model, and Prometheus adapter coverage
Scope:
- `crates/oya-shared-hyperscaler-metrics-kernel/**`
- `crates/oya-shared-hyperscaler-metrics-adapter-prometheus/**`
- backbone GitHub Actions/Jenkins package matrices and task/evidence tracking.

Acceptance:
- Shared metrics kernel exposes a `RequestTelemetryOutcome` runtime-emission seam that fans one REST/gRPC outcome into request-total, request-success, responses-total, 429, 5xx, circuit-state, and retry-budget-exhausted canonical metric families.
- Shared metrics kernel exposes deterministic runtime telemetry exercise reports and integer SLO burn-rate assessments for pre-prod drills.
- Prometheus reference adapter proves the runtime outcome emitter increments canonical counters/gauges for success, 429 backpressure, open circuit, retry-budget exhaustion, and 5xx outcomes.
- Backbone GitHub Actions and Jenkins matrices include the Prometheus metrics adapter package alongside the metrics kernel.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-hyperscaler-metrics-kernel` and `oya-shared-hyperscaler-metrics-adapter-prometheus`.
- broad backbone cargo check/test/clippy/fmt with the Prometheus adapter included in every per-service matrix.
- CI/Jenkins package validation, dependency-seam, honest-claims, and diff hygiene.

### CS-BACKBONE-OTLP-METRICS-HARNESS-001 — Env-gated OTLP metrics exporter harness
Scope:
- `crates/oya-shared-hyperscaler-metrics-adapter-otlp/**`
- `registry/catalog/oya-shared-hyperscaler-metrics-adapter-otlp.yaml`
- `registry/dependency-rationales.json`
- backbone GitHub Actions/Jenkins package matrices and task/evidence tracking.

Acceptance:
- Adds a separate shared OTLP/HTTP adapter crate implementing the existing `HyperscalerMetrics` trait with OpenTelemetry SDK counters/gauges and canonical low-cardinality labels.
- Export is disabled unless `OYA_BACKBONE_OTLP_METRICS` parses true and an OTLP endpoint is supplied through `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` or `OTEL_EXPORTER_OTLP_ENDPOINT`.
- Endpoint validation rejects whitespace/control characters, requires `http://` or `https://`, and enforces `https://` when `OYA_BACKBONE_OTLP_REQUIRE_TLS` is true.
- Unit tests exercise env gating, endpoint precedence, TLS posture, timeout validation, disabled builder behavior, and canonical runtime-outcome recording without a live collector.
- Backbone GitHub Actions and Jenkins matrices include the OTLP adapter package alongside the metrics kernel and Prometheus adapter.
- Honest non-claim: this is an SDK/exporter harness and compile/test proof only; it does not claim a live collector, live alert firing, production SLO, or backpressure drill.

Verification:
- `cargo test` / `cargo clippy -D warnings` / `cargo fmt --check` for `oya-shared-hyperscaler-metrics-adapter-otlp`.
- broad backbone cargo check/test/clippy/fmt with the OTLP adapter included in every per-service matrix.
- CI/Jenkins package validation, dependency-seam, honest-claims, cargo-deny/license/cargo-prefix/catalog/data-class known-blocker probes, and diff hygiene.

### CS-BACKBONE-ARGOCD-PROMOTION-001 — ArgoCD ApplicationSet promotion manifest for backbone services
Scope:
- `microservices/cloud-iac/iac/oyatie-cloud-provider/argocd/apps/backbone-microservices-applicationset.yaml`
- `.github/workflows/backbone-microservices-ci.yml`
- task/evidence tracking.

Acceptance:
- ArgoCD ApplicationSet enumerates messenger, mail, social, and community as separate tenant-isolated namespaces under the oyatie-cloud-provider ArgoCD project.
- Every generated Application points at the existing per-service `iac/k8s/helm` chart, pins `targetRevision: dev`, requires cosign image promotion, and records deploy audit-chain event annotations.
- Backbone GitHub Actions static governance smoke parses the ApplicationSet alongside existing evidence/catalog files.
- Honest non-claim: this is a static GitOps promotion manifest only, not evidence of a live ArgoCD sync, branch-protection rollout, or deployed runtime.

Verification:
- YAML parse and structural assertions for the ApplicationSet.
- GitHub Actions workflow parse after adding the ApplicationSet to path filters/governance smoke.
- dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-EDGE-TLS-HARDENING-001 — Static edge WAF/ECH/PQC hardening for messenger and community
Scope:
- `microservices/messenger/iac/edge-waf.yaml`
- `microservices/messenger/iac/ech-config.yaml`
- `microservices/messenger/iac/pqc-cert.yaml`
- `microservices/community/iac/edge-waf.yaml`
- `microservices/community/iac/ech-config.yaml`
- `microservices/community/iac/pqc-cert.yaml`
- task/evidence tracking.

Acceptance:
- Messenger and community gain static EdgeWAFConfig manifests with route-scoped rate limits, passive/challenge bot-management posture, honeypot routes, abuse-detection controls, audit emission classes, tenant visibility metrics, and a TLS 1.3/ECH/PQC posture matching existing mail/social patterns.
- Messenger and community gain ECHConfig manifests with 90-day rotation windows, Cloudflare HTTPS RR publication intent, standard TLS 1.3 fallback, and explicit no-hard-fail UX fallback.
- Messenger and community gain cert-manager Certificate manifests for hybrid PQC/classical server cert intent with X25519MLKEM768 and ed25519+ml_dsa_65 preferences plus classical fallback.
- Honest non-claim: these are source-controlled static hardening manifests only; no DNS record, certificate issuance, Cloudflare application, Gateway sync, or production TLS rollout is claimed.

Verification:
- YAML parse of the six new static manifests.
- Structural checks for EdgeWAFConfig/ECHConfig/Certificate kind coverage, TLS 1.3, ECH enabled, PQC hybrid fields, honeypot routes, and non-claim task/evidence markers.
- dependency-seam, honest-claims, diff hygiene, and Oya VCS verify/done/promote.

### CS-BACKBONE-GOVERNANCE-VALIDATOR-UNBLOCK-001 — Catalog/data-class validator unblock
Scope:
- `registry/catalog/oya-audit-chain-*.yaml`, `registry/catalog/oya-payments-*.yaml`, and `registry/catalog/oya-tenancy-*.yaml` records for existing workspace crates that already had workspace membership but lacked global catalog coverage.
- Existing kernel field comments in audit-chain, tenancy, OIDC, and SCIM crates plus stale legacy data-class allowances.
- PR metadata/evidence for the remote GitHub Actions budget failure mode.

Acceptance:
- Global catalog validation covers every current workspace crate with conservative preview/unreviewed/source-only records and does not promote live/security/supply-chain claims.
- Data-class validation passes by semantically annotating existing exposed kernel fields and deleting only stale legacy allowances for fields now annotated.
- Remote GitHub Actions failures are recorded honestly as budget-prevented jobs with no runner steps, not implementation test failures or remote green evidence.

Verification:
- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog`.
- `./bin/oya gate validate data-class --workspace Cargo.toml --legacy registry/data-class/legacy-unannotated-fields.tsv`.
- Focused cargo check/fmt/test/clippy for the touched kernel crates.
- GitHub check-run annotation audit for PR #179 confirms 36 failed checks share the Actions budget-prevented message.
- Full `./bin/oya verify --ci-required` is recorded honestly as non-green: local catalog/data-class blockers are resolved, while repo-global/local mirror lanes still fail on recursive `oya verify` nextest cases, doc/readme catalog coverage, ADR citation/index metadata, design/spec maturity, glossary/placeholder debt, dependency-seam, architecture/layered-architecture discipline, and VCS admission audit-chain coverage.


### CS-BACKBONE-ARCHITECTURE-BOUNDARY-ALIGNMENT-001 — Transport/catalog role alignment
Scope:
- `crates/oya-dev-cli/src/commands/gate/architecture_boundaries.rs` role matrix and focused tests.
- Catalog role truth-down for backbone REST/gRPC/API DTO/outbox composition crates plus audit-chain DTO crates that are imported by domain code.
- Evidence/task tracking for the architecture-boundaries lane outcome.

Acceptance:
- `architecture-boundaries` recognizes REST/gRPC/adapter crates as outer composition surfaces that may depend inward on usecase/app/application/domain/kernel roles without allowing kernel/domain outward dependencies.
- API DTO crates that are consumed by domain/usecase/app code are cataloged as `domain` contract crates rather than outward API facades.
- Local architecture-boundaries validation passes for the current workspace/catalog snapshot, with no production runtime or remote CI green claim.

Verification:
- `cargo fmt --check -p oya-dev-cli`.
- `cargo test -p oya-dev-cli architecture_boundaries -- --nocapture`.
- `./bin/oya gate validate architecture-boundaries --repo-root . --registry registry/catalog`.
- `./bin/oya catalog validate --workspace Cargo.toml --registry registry/catalog`.

### CS-BACKBONE-VERIFY-RECURSION-FIXTURE-001 — Local verify nested-test unblock
Scope:
- `crates/oya-dev-cli/tests/oya_verify_ci_mirror.rs` fixture environment handling for nested `oya verify --ci-required` integration tests.
- Evidence/task tracking for the D-4 nextest local full-verify blocker resolution.

Acceptance:
- The production `OYA_VERIFY_RUNNING` recursion guard remains in `verify.rs`; only the test fixture removes an inherited outer guard before intentionally spawning the real `oya verify` binary.
- The oya verify CI mirror tests pass when the parent test process inherits `OYA_VERIFY_RUNNING=1`, matching the full `oya verify --ci-required` execution environment.
- Workspace nextest passes under the inherited guard, while the broader full verify remains honestly non-green on documented D-5/D-6 governance lanes.

Verification:
- `OYA_VERIFY_RUNNING=1 cargo test -p oya-dev-cli --test oya_verify_ci_mirror -- --nocapture`.
- `OYA_VERIFY_RUNNING=1 cargo nextest run -p oya-dev-cli --test oya_verify_ci_mirror`.
- `OYA_VERIFY_RUNNING=1 cargo nextest run --workspace --no-fail-fast`.
- `./bin/oya verify --ci-required` records D-4 nextest as pass and remains non-green on D-5/D-6 lanes.

### CS-BACKBONE-GATEWAY-HTTPROUTE-001 — Static Gateway API HTTPRoute chart exposure
Scope:
- `microservices/{messenger,mail,social,community}/iac/k8s/helm/templates/httproute.yaml`.
- `microservices/{messenger,mail,social,community}/iac/k8s/helm/values.yaml`.
- `.github/workflows/backbone-microservices-ci.yml` path filters and static smoke coverage.
- Evidence/task tracking for the static Gateway API route seam.

Acceptance:
- Messenger, mail, social, and community Helm charts include an optional `gateway.networking.k8s.io/v1` `HTTPRoute` template attaching to a pack-owned HTTPS Gateway parent and routing only the OpenAPI base path to the in-namespace Service.
- Values declare explicit default hostnames/path prefixes aligned with the current OpenAPI server surfaces: messenger/social `/api/v1`, mail/community `/v1`.
- The route template annotates its OpenAPI contract, TLS/ECH/PQC manifest references, and audit-chain event class while remaining disabled by default to avoid accidental exposure before Gateway/DNS/TLS provisioning.
- Backbone GitHub Actions path filters and governance smoke now cover Helm chart changes and check the static HTTPRoute template shape.
- Honest non-claim: these are chart-ready static HTTPRoute templates only; no live Gateway, DNS record, TLS termination, ECH publication, PQC certificate issuance, or production route rollout is claimed.

Verification:
- YAML parse of the four updated `values.yaml` files.
- Static HTTPRoute template checks for `apiVersion`, `kind`, `parentRefs`, `backendRefs`, OpenAPI/TLS/ECH/PQC annotations, disabled-by-default values, and service/path/hostname coverage.
- Backbone governance smoke Ruby command parses values and asserts HTTPRoute template shape.
- dependency-seam, honest-claims, diff hygiene, full `./bin/oya verify --ci-required`, and Oya VCS verify/done/promote.

### CS-BACKBONE-REST-HYPER-RUNTIME-001 — Shared REST Hyper loopback runtime adapter
Scope:
- `crates/oya-http-runtime-hyper-adapter/src/lib.rs` listener-bound serve entrypoint.
- `crates/oya-shared-backbone-rest-runtime-adapter/**` shared runtime adapter crate.
- `Cargo.toml`, `registry/catalog/oya-shared-backbone-rest-runtime-adapter.yaml`, backbone GitHub Actions/Jenkins matrices, and task/evidence tracking.

Acceptance:
- Messenger, mail, social, and community OpenAPI route catalogs are all registered into the canonical Hyper runtime seam via a shared adapter crate without importing Hyper outside `oya-http-runtime-hyper-adapter`.
- `/health` and `/ready` dispatch through the service-owned probe handlers; contract-only OpenAPI routes return explicit 501 responses; typed write-plan routes remain honest 501 responses that state generic Hyper JSON-body binding is not yet claimed.
- A local TCP loopback test reaches all four REST service catalogs through Hyper for liveness and contract-only routes.
- The backbone package matrices include the shared REST runtime adapter in every microservice lane.
- Honest non-claim: this proves local Hyper loopback/runtime binding only; it is not production Gateway/TLS rollout evidence and does not claim generic JSON write-body execution.

Verification:
- Focused cargo check/test/clippy/fmt for `oya-http-runtime-hyper-adapter` and `oya-shared-backbone-rest-runtime-adapter`.
- Backbone CI governance smoke parses the new catalog row.
- dependency-seam, honest-claims, diff hygiene, full `./bin/oya verify --ci-required`, and Oya VCS verify/done/promote.

### CS-BACKBONE-REST-JSON-WRITE-BINDING-001 — Stateless REST JSON write binding over Hyper
Scope:
- `crates/oya-shared-backbone-rest-runtime-adapter/**` JSON body/header binding for stateless write routes.
- `registry/dependency-rationales.json` adapter-layer `serde_json` allowance for this runtime edge only.
- Task/evidence tracking.

Acceptance:
- The shared REST runtime adapter parses bounded Hyper-collected JSON bodies at the adapter edge and maps them into existing service-owned typed REST dispatchers for messenger post-message, mail submit-message, social publish-post, and community create-post.
- Common request context is supplied through explicit headers (`x-oya-scope-ref`, `x-oya-context-kind`, `x-oya-principal-ref`, `Idempotency-Key`, `x-oya-policy-decision-ref`, `x-request-id`) and missing/invalid context fails closed with 400.
- Path/body identifier drift fails closed before the typed handler runs.
- Community vote/moderation routes remain explicit 501 stateful-write-plan-required seams because they require backing post/ledger state; no database/broker/live deployment claim is made.
- Local TCP loopback tests execute all four stateless JSON write routes over Hyper and assert service receipt event types.

Verification:
- Focused cargo check/test/clippy/fmt for `oya-shared-backbone-rest-runtime-adapter`.
- dependency-seam, honest-claims, diff hygiene, full `./bin/oya verify --ci-required`, and Oya VCS verify/done/promote.

### CS-BACKBONE-CLOUD-TENANT-WORKLOAD-LABELS-001 — Static Oyatie Cloud dogfood tenant workload labels
Scope:
- `microservices/{messenger,mail,social,community}/iac/k8s/helm/values.yaml` workload label defaults for FD-001 dogfood tenant identity, cost center, workload class, regulatory pack, dogfood substrate, and product-goal correlation.
- `microservices/{messenger,mail,social,community}/iac/k8s/helm/templates/deployment.yaml` Deployment and Pod-template labels for workload-level cost, tenant, regulatory, and dogfood correlation.
- `.github/workflows/backbone-microservices-ci.yml` governance smoke coverage for Deployment templates as a separate static-template class from Gateway `HTTPRoute` templates.
- Task/evidence/audit tracking.

Acceptance:
- Messenger, mail, social, and community Helm charts carry source-controlled labels `oya.io/tenant-id`, `oya.io/cost-center`, `oya.io/workload-class`, `oya.io/regulatory-pack`, `oyatie.com/dogfood-substrate`, and `oyatie.com/product-goal` on both Deployment metadata and Pod-template metadata.
- The labels are values-driven and are intentionally not added to `spec.selector.matchLabels`, preserving selector stability if future tenant/cost overlays change.
- CI governance smoke parses the values and asserts Deployment templates include readiness/liveness probes, `runtimeClassName`, resources, security context, and the required tenant/cost/regulatory/dogfood label contract.
- Honest non-claim: this proves a static chart contract only. No rendered live manifests, Kubernetes admission result, OpenCost allocation report, ArgoCD sync/health result, branch-protection result, or production tenant workload deployment is claimed.

Verification:
- YAML parse of the four updated `values.yaml` files.
- Static Deployment template checks for required probes, runtime class, resources, security context, and tenant/cost/regulatory/dogfood labels.
- Backbone governance smoke Ruby command parses values, distinguishes HTTPRoute templates from Deployment templates, and asserts both static template classes.
- tenant-cost-label static source checks, dependency-seam, honest-claims, diff hygiene, full `./bin/oya verify --ci-required`, and Oya VCS verify/done/promote.

### CS-BACKBONE-RENDERED-TENANT-COST-LABEL-COVERAGE-001 — Static rendered tenant-cost-label coverage snapshots
Scope:
- `microservices/{messenger,mail,social,community}/iac/helm/oyatie-cloud-dogfood/rendered/deployment.yaml` static rendered Deployment manifest snapshots generated from the current k8s Helm values/template defaults.
- `.github/workflows/backbone-microservices-ci.yml` path filters and governance smoke coverage for the rendered snapshots.
- Task/evidence/audit tracking.

Acceptance:
- The ADR-0199 tenant-cost-label advisory gate scans four FD-001 backbone rendered workload manifests and reports zero findings.
- Each rendered Deployment snapshot carries the tenant/cost/regulatory label block, dogfood substrate/product-goal labels, probes, runtime class, resources, security context, and an explicit non-claim annotation.
- CI path filters include the rendered snapshot directories, and governance smoke parses the rendered YAML while asserting the same Deployment label/probe/resource/security shape.
- Honest non-claim: these are static rendered snapshot files only. No Helm controller, Kubernetes API server, admission webhook, OpenCost collector/report, ArgoCD sync/health result, branch-protection result, or production tenant workload deployment is claimed.

Verification:
- YAML parse and static required-string checks for the four rendered Deployment snapshots.
- `./bin/oya gate validate tenant-cost-labels-coverage` reports four manifests scanned and zero findings.
- Backbone governance smoke Ruby command parses and checks the rendered snapshots along with the chart templates.
- dependency-seam, honest-claims, audit-chain replay, diff hygiene, full `./bin/oya verify --ci-required`, and Oya VCS verify/done/promote.

### CS-BACKBONE-ARGOCD-FD001-TENANT-METADATA-001 — ArgoCD FD-001 tenant metadata propagation
Scope:
- `microservices/cloud-iac/iac/oyatie-cloud-provider/argocd/apps/backbone-microservices-applicationset.yaml` metadata, list-generator elements, Helm parameter overrides, and managed namespace labels for the FD-001 dogfood tenant/cost/workload/regulatory label block.
- `.github/workflows/backbone-microservices-ci.yml` governance smoke semantic assertions for the ApplicationSet tenant metadata, chart paths, four-service list generator, and workload label Helm parameters.
- Task/evidence/audit tracking.

Acceptance:
- The static ArgoCD ApplicationSet carries the same FD-001 tenant ID, cost center, workload class, regulatory pack, dogfood substrate, and product-goal metadata used by the source Helm charts and rendered Deployment snapshots.
- Each generated Application passes those fields into Helm via `workloadLabels.*` parameters so future controller syncs would not silently drift from the chart defaults.
- Managed namespace metadata carries the same low-cardinality tenant/cost labels with `CreateNamespace=true`, preserving tenant namespace isolation as a static GitOps intent.
- Honest non-claim: this is a source-controlled ApplicationSet intent update only. No ArgoCD controller reconciliation, namespace creation, sync health, cosign verification, OpenCost report, Kubernetes admission, or production tenant workload deployment is claimed.

Verification:
- YAML parse and ApplicationSet semantic checks for four generator elements, label values, chart path existence, Helm workload label parameters, template labels, and namespace labels.
- Backbone governance smoke command parses the new evidence and asserts the ApplicationSet tenant metadata.
- dependency-seam, honest-claims, audit-chain replay, diff hygiene, full `./bin/oya verify --ci-required`, and Oya VCS verify/done/promote.

### PR closeout — isolated worktree branch to `dev`

- Branch: `agent/backbone-microservices-20260523T081210Z`.
- Commit: `1e0a4ef1 feat: add backbone microservice foundations`.
- Pull request: https://github.com/jason931225/oyatie/pull/179.
- Honest non-claim: remote CI/branch-protection results are not claimed here; the PR exists for the platform to run those live checks.

## Remaining non-claims

Still pending before the full objective can honestly be called complete:
- Broader contract-only REST/OpenAPI endpoint implementation plus live vendor broker, live production gateway/TLS rollout evidence, deployed outbox pollers/publishers, stateful community vote/moderation HTTP binding, and acknowledgements beyond framework-free implemented write-route dispatch, shared REST Hyper loopback/runtime catalog binding, stateless JSON write binding over local Hyper, static Oyatie Cloud dogfood tenant workload labels plus rendered tenant-cost-label snapshots and static ArgoCD FD-001 tenant metadata, static messenger/community edge WAF/ECH/PQC manifests, static disabled-by-default Gateway API HTTPRoute chart templates, transport planning metadata, local tonic loopback server/client seams, local HTTP broker publish/executor seams, transactional outbox command/drain seams, and recording acknowledgement contracts.
- Live Postgres/Citus RLS integration runs, backup/restore drills, and Citus rebalance evidence beyond generated write batches, execution contracts, the compile-checked SQLx executor adapter seam, and the env-gated live RLS/Citus harness.
- Live Cedar PDP deployment and service-gateway enforcement evidence beyond the in-process evaluator/conformance pack.
- Live OpenTelemetry collector deployment, production SLO burn alert firing evidence, and production backpressure/circuit-breaker drills beyond the in-process runtime metrics exercise, Prometheus adapter tests, and env-gated OTLP/HTTP exporter harness.
- Live ArgoCD sync/health evidence, branch-protection live evidence, and full CI runtime-run evidence beyond static matrix definitions and static ApplicationSet manifests.
- Oya VCS verify/done/promote lifecycle closeout recorded for expanded ChangeBundles promoted through `cb-backbone-microservices-edge-tls-hardening-20260524` plus PR closeout evidence.
- Pull request against `dev` opened from the isolated worktree branch: https://github.com/jason931225/oyatie/pull/179.
