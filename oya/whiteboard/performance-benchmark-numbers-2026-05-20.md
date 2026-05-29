# Whiteboard Performance Benchmark Numbers - 2026-05-20
µservice: `whiteboard`
Counterpart bar: Miro / FigJam / Lucidchart
Benchmark shape: single industry-leader target set with deployment-context and tenant_class overlays
Tier-delta status: retired; this report does not segment targets by retired capability tiers

## Header Anchor Block
1. Canonical deployment contexts: `specs/master-plan-sequencing.json:704-746` defines `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
2. OpenTofu and forbidden engines: `specs/master-plan-sequencing.json:747-775` requires OpenTofu, context modules, pinned providers, signed modules, and no Terraform/Pulumi/CloudFormation/ARM.
3. OS/language/OCI profile: `specs/master-plan-sequencing.json:777-868` requires supported OS matrix, Rust backend policy, and OCI Always Free profile.
4. Performance-doc amendment: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139-142` says this batch drops the tier-delta deliverable and uses performance targets without old tier segments.
5. Substance bar: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-12` requires deliverable quality beyond line count.
6. Local SLO evidence: `microservices/whiteboard/slos/local-cursor-latency.openslo.yaml:12-15`, `local-stroke-persistence-latency.openslo.yaml:12-15`, `local-board-load-time.openslo.yaml:12-15`, `local-crdt-merge-success.openslo.yaml:12-15`, and `local-export-render-latency.openslo.yaml:12-15` name core whiteboard performance signals.
7. Local SLO gap: `microservices/whiteboard/slos/local-*.openslo.yaml:26-31` generally group by `tenant_id, cell_tier`, not by `deployment_context` or `tenant_class`.

## Methodology Disclosure
1. Public whiteboard vendors rarely publish server-side p50/p95/p99 latencies for cursor fanout, stroke persistence, board load, CRDT merge, or export render.
2. This report therefore separates public counterpart numbers from Oyatie target numbers.
3. Public counterpart numbers are only listed when a public source provides them.
4. Where a public source provides a usage limit rather than latency, the row labels it as product-visible limit, not measured latency.
5. Where this report estimates from a documented limit, it states the estimate basis.
6. Oyatie target numbers are proposed canonical targets for future benchmark harnesses, not current measured whiteboard performance.
7. Existing local artifacts do not contain a `benchmarks/` directory, so no local historical benchmark run is cited.
8. Existing local artifacts do contain SLO names and target ratios, but they do not contain latency thresholds in milliseconds.
9. Existing local code has contract stubs in `src/adapter/http.rs:65-67`, so runtime latency cannot be honestly measured from the current implementation.
10. Existing tests mark core contract/policy/repository fixture tests as ignored at `tests/integration.rs:57-75`.
11. All targets below assume a Rust backend, event-sourced/CRDT board state, HTTP/3 where available, fallback HTTP/2, and explicit backpressure.
12. All targets below require future benchmark fixtures for both write path and read/model projection path.
13. Deployment-context overlays constrain throughput and scale, not product quality.
14. Tenant_class overlays constrain usage caps, cost envelope, and contractual SLO, not feature quality.
15. `demo_trial` means free OCI Always Free profile and time/usage caps.
16. `paid` means per-seat plus usage-based billing and contractual SLO.
17. `revenue_share` means at-cost or zero-margin substrate with Oyatie revenue share from the customer's gross revenue.
18. The quality bar is uniform across all tenant classes.
19. Every benchmark must report OS, architecture, deployment_context, tenant_class, tenant_id hash, board size, object count, active editors, total participants, operation mix, and artifact digest.
20. Benchmark harnesses must run on at least x86_64 and aarch64 because canonical OS support includes both.
21. Benchmark harnesses must report browser/app client profile separately from backend numbers.
22. Benchmarks must include cold load, warm load, active collaboration, degraded/static view, export, import, and recovery workloads.
23. Benchmarks must include failure cases: reconnect storm, burst sticky import, large export, CRDT conflict, and rate-limit response.
24. Benchmarks must not use the retired capability-tier vocabulary.

## §1 Methodology
1. Dimension: cursor fanout latency p50/p95/p99 from one editor to visible peer update.
2. Dimension: stroke/canvas-op persistence acknowledgement p50/p95/p99.
3. Dimension: board open latency p50/p95/p99 at 1k, 10k, 50k, and 100k objects.
4. Dimension: CRDT merge latency and merge success ratio.
5. Dimension: presence freshness lag and stale-presence eviction time.
6. Dimension: export render latency for PNG, PDF, CSV, and SVG when supported.
7. Dimension: template install latency and marketplace settlement wait split.
8. Dimension: bulk sticky import throughput from CSV/spreadsheet payloads.
9. Dimension: API rate-limit behavior, including 429 shape and Retry-After.
10. Dimension: concurrent active editors before graceful degradation.
11. Dimension: total participants before static snapshot fallback.
12. Dimension: board object ceiling before UI degradation.
13. Dimension: metadata ceiling per board item and per app/extension.
14. Dimension: recovery time after board-session leader failover.
15. Dimension: replay freshness after region failover.
16. Workload A: small workshop board, 1k objects, 25 editors, 75 viewers, 60-minute session.
17. Workload B: standard product discovery board, 10k objects, 100 editors, 250 viewers, 120-minute session.
18. Workload C: large enterprise program board, 50k objects, 250 editors, 750 viewers, 4-hour session.
19. Workload D: diagram-heavy board, 25k objects with 15k connectors and 2k data-backed shapes.
20. Workload E: import/export board, 20k sticky notes imported in batches and exported to CSV/PDF.
21. Workload F: reconnect storm, 250 editors reconnecting over 90 seconds after region failover.
22. Workload G: OCI Always Free profile, 5k objects, 25 editors, 75 viewers, enforced demo_trial caps.
23. OS disclosure: benchmark rows must state the OS from the canonical supported OS matrix.
24. Architecture disclosure: benchmark rows must state x86_64, aarch64, ppc64le, or s390x where applicable.
25. Deployment-context disclosure: every row must state one of the six canonical deployment contexts.
26. Tenant_class disclosure: every row must state `demo_trial`, `paid`, or `revenue_share`.
27. Data-class disclosure: board object, canvas operation, presence cursor, export snapshot, template marketplace projection.
28. Success rule: measured p95 must meet the canonical target in the relevant context overlay.
29. Failure rule: any benchmark that omits deployment_context or tenant_class is invalid.
30. Publication rule: benchmark outputs should be committed as machine-readable JSON plus human summary.

## §2 Counterpart Numbers

### §2.1 Miro Public Numbers
Miro-001: REST API global allowance: 100,000 credits per minute, source `https://developers.miro.com/reference/rate-limiting`, fetched lines 52-58.
Miro-002: REST Level 1 request shape: 50 credits per call and 2,000 requests per minute, same source lines 52-53.
Miro-003: REST Level 2 request shape: 100 credits per call and 1,000 requests per minute, same source lines 52-54.
Miro-004: REST Level 3 request shape: 500 credits per call and 200 requests per minute, same source lines 52-55.
Miro-005: REST Level 4 request shape: 2,000 credits per call and 50 requests per minute, same source lines 52-56.
Miro-006: REST 429 behavior includes `status: 429`, code `tooManyRequests`, and rate-limit headers, same source lines 59-78.
Miro-007: Web SDK global allowance: 100,000 credits per minute, source `https://developers.miro.com/docs/websdk-reference-rate-limiting`, search result captured 2026-05-21.
Miro-008: Web SDK bulk item creation estimate: 2,000 board items per minute from 100,000 credits/minute at 50 credits per item, same source.
Miro-009: Web SDK image creation estimate: 200 image items per minute from 100,000 credits/minute at 500 credits per image, same source.
Miro-010: Web SDK hourly allowance: 1,000,000 credits per hour, equivalent to 20,000 board items/hour or 2,000 image items/hour, same source.
Miro-011: Recommended board UI object ceiling: do not exceed 10,000 items per board to avoid UI degradation, source `https://developers.miro.com/docs/websdk-reference-board`, fetched lines 1537-1542 and 1660-1665.
Miro-012: Board app metadata storage limit: 30 KB per app, same source fetched lines 1743-1746.
Miro-013: Board item metadata storage limit: 6 KB per item, same source fetched lines 1978-1980.
Miro-014: Common simultaneous collaboration scenario: around 200 editors/commenters on one board, source `https://help.miro.com/hc/en-us/articles/360017730813-Sharing-boards-and-inviting-collaborators`, fetched lines 223-224.
Miro-015: Built-in Miro video chat supports up to 25 participants, source `https://help.miro.com/hc/en-us/articles/360012753200-Miro-for-workshops-meetings`, fetched lines 144-146.
Miro-016: Free plan has three active editable boards, source `https://help.miro.com/hc/en-us/articles/360012753200-Miro-for-workshops-meetings`, fetched lines 137-143.
Miro-017: Sticky note text limit is about 3,000 symbols depending on text size, source `https://help.miro.com/hc/en-us/articles/360017572054-Sticky-notes`, fetched lines 43-48.
Miro-018: Spreadsheet paste into sticky notes supports up to 5,000 cells with 50 rows and 100 columns and 6,000 maximum characters, same source fetched lines 80-88.

### §2.2 FigJam Public Numbers
FigJam-001: Multiplayer cursor display limit: 200 cursors, source `https://help.figma.com/hc/en-us/articles/1500006775761-How-many-people-can-be-in-a-file-at-once`, fetched lines 99-107.
FigJam-002: Concurrent editor limit: 200 people editing, same source fetched lines 99-103 and 112-117.
FigJam-003: Total participant limit: 500 total participants, same source fetched lines 99-103 and 126-130.
FigJam-004: Post-editor-limit behavior: additional editors are converted to view-only for the current multiplayer session, same source fetched lines 112-117.
FigJam-005: Post-participant-limit behavior: late joiners see a static version without multiplayer actions, same source fetched lines 126-130.
FigJam-006: Timer maximum: 99 minutes and 59 seconds, source `https://help.figma.com/hc/en-us/articles/4402269549591-Stay-on-track-with-the-timer-in-FigJam`, fetched lines 92-95.
FigJam-007: Active timer count: one active timer per file, same source fetched lines 92-95.
FigJam-008: Timer controls can be started/paused/extended/stopped by anyone in the file, same source fetched lines 92-101 and 118-127.
FigJam-009: Voting session hides participant choices and reveals final tally at session end, same source fetched lines 142-146.
FigJam-010: Export availability: FigJam board export is available on any team or plan for editors, source `https://help.figma.com/hc/en-us/articles/4407699832855-Export-your-FigJam-board`, fetched lines 82-88.
FigJam-011: CSV export supports FigJam tables and sticky notes, same source fetched lines 92-101.
FigJam-012: Entire board export supports image output, same source fetched lines 104-111.
FigJam-013: Direct SVG export from FigJam is not supported, same source fetched line 91.
FigJam-014: REST API rate limits are affected by seat type, endpoint bucket, and resource plan/location, source `https://developers.figma.com/docs/rest-api/rate-limits/`, fetched lines 67-77.
FigJam-015: REST API uses a leaky bucket and returns 429 when the bucket is full, same source fetched lines 140-142.
FigJam-016: 429 responses include Retry-After and Figma rate-limit headers, same source fetched lines 146-177.
FigJam-017: View and Collab seats can have very low monthly request allowances under load, same source fetched line 139.
FigJam-018: FigJam external contribution can be free for 24 hours without login, source `https://www.figma.com/figjam/`, fetched lines 361-363.

### §2.3 Lucidchart Public Numbers
Lucid-001: Public numeric API quota: not published in the fetched official reference; source `https://developer.lucid.co/reference/reference-rest`, fetched lines 122-129.
Lucid-002: API rate-limit behavior: HTTP 429 Too Many Requests when threshold exceeded, same source fetched lines 122-127.
Lucid-003: Retry-After behavior: 429 includes a Retry-After header with seconds until requests are accepted again, same source fetched lines 126-127.
Lucid-004: Rate-limit retry guidance: wait at least until Retry-After passes; additional requests may extend the limit window, same source fetched lines 126-127.
Lucid-005: Page object required fields: id and title, source `https://developer.lucid.co/docs/pages-si`, fetched lines 419-423.
Lucid-006: Page object optional arrays: shapes, data-backed shapes, lines, groups, layers, and custom data, same source fetched lines 426-453.
Lucid-007: Lucidspark default page background fill color: `#f2f3f5`; Lucidchart default page background fill color: white, same source fetched lines 460-462.
Lucid-008: Infinite canvas field: `infiniteCanvas` boolean, same source fetched lines 463-464.
Lucid-009: Auto-tiling field: `autoTiling` boolean, same source fetched lines 465-466.
Lucid-010: Page sizing field: `size` controls standard page sizes when infiniteCanvas is false, same source fetched lines 467-468.
Lucid-011: Page-count hard limit: not set publicly by the community answer, but large files are discouraged for performance, source `https://community.lucid.co/product-questions-3/max-number-of-pages-in-a-lucidchart-document-1834`, fetched lines 43-68.
Lucid-012: Collaboration feature set includes comments, templates, follow cursor, revision history, visual activities, AI, embedded video, and assigned tasks, source `https://www.lucidchart.com/blog/getting-started-sharing-and-collaboration`, fetched lines 253-289.
Lucid-013: Join ID exists for large-group sharing, same source fetched lines 246-248.
Lucid-014: Publish URL/password/export/copy/embed workflow exists, same source fetched lines 249-252.
Lucid-015: Public collaboration latency and throughput numbers are not published in the fetched official sources; treat Oyatie targets below as internal engineering targets.

## §3 Oyatie Target Numbers - Single Industry-Leader Set
Target-001: Cursor fanout latency canonical target: p50 <= 35 ms, p95 <= 90 ms, p99 <= 180 ms for active editors in the same region.
Target-002: Cursor fanout remote-region overlay: p95 <= 150 ms when editors are cross-region but pinned to the same tenant board session.
Target-003: Cursor fanout OCI Always Free overlay: p95 <= 160 ms for up to 25 editors and 75 viewers on 5k-object boards.
Target-004: Cursor fanout paid/revenue_share overlay: p95 <= 90 ms at 250 editors and 750 total participants in elastic contexts.
Target-005: Presence freshness canonical target: p95 <= 2 seconds for stale detection and p99 <= 5 seconds for eviction after disconnect.
Target-006: Presence static fallback target: switch late joiners to static snapshot in <= 1 second after participant cap is reached.
Target-007: Active editor canonical target: 250 simultaneous editors, intentionally above FigJam's 200 editor public limit and Miro's around-200 common scenario.
Target-008: Total participant canonical target: 750 participants, intentionally above FigJam's 500 public participant limit.
Target-009: Demo_trial participant overlay: cap at 25 editors and 75 total participants on OCI Always Free profile.
Target-010: Paid participant overlay: scale by purchased seats and usage budgets, default benchmark at 250 editors and 750 total participants.
Target-011: Revenue_share participant overlay: scale by at-cost substrate approval, default benchmark at 250 editors and 750 total participants.
Target-012: Stroke/canvas-op durable ack target: p50 <= 45 ms, p95 <= 120 ms, p99 <= 250 ms for 10k-object board workload.
Target-013: Stroke persistence OCI Always Free overlay: p95 <= 220 ms for Workload G.
Target-014: Canvas operation ordering target: 99.999 percent operations applied in causal order without manual repair.
Target-015: CRDT merge success target: >= 99.99 percent conflict-free durable merges over 30 days.
Target-016: CRDT conflict resolution latency target: p95 <= 250 ms for normal conflicts and p99 <= 1 second for reconnect storms.
Target-017: Board open 1k-object target: p50 <= 400 ms, p95 <= 900 ms, p99 <= 1.8 seconds.
Target-018: Board open 10k-object target: p50 <= 1.2 seconds, p95 <= 2.5 seconds, p99 <= 4 seconds.
Target-019: Board open 50k-object target: p50 <= 2.5 seconds, p95 <= 5.5 seconds, p99 <= 9 seconds in elastic contexts.
Target-020: Board open 100k-object target: p50 <= 5 seconds, p95 <= 12 seconds, p99 <= 20 seconds with progressive hydration.
Target-021: OCI Always Free board object overlay: demo_trial profile supports 5k objects by default and hard-caps at 10k objects unless tenant upgrades.
Target-022: Paid/revenue_share board object overlay: default live board target 50k objects, stretch target 100k objects with progressive hydration.
Target-023: Object ceiling target: 50k live objects without UI degradation in elastic contexts, intentionally above Miro's 10k UI recommendation.
Target-024: Archival object ceiling target: 250k historical objects per board with lazy loading and snapshot compaction.
Target-025: Board app metadata target: 128 KB per app/extension per board, ahead of Miro's 30 KB app metadata limit.
Target-026: Board item metadata target: 16 KB per object, ahead of Miro's 6 KB item metadata limit.
Target-027: Bulk sticky import target: 10,000 cells per import batch, ahead of Miro's 5,000-cell paste limit.
Target-028: Bulk sticky import latency target: p95 <= 8 seconds for 10,000 cells in elastic contexts.
Target-029: Bulk sticky import OCI Always Free overlay: cap import batch at 2,500 cells and p95 <= 10 seconds.
Target-030: CSV export target: p95 <= 1.5 seconds for 10k sticky/table rows and p99 <= 4 seconds.
Target-031: PNG export target: p95 <= 4 seconds for 10k objects and p99 <= 10 seconds.
Target-032: PDF export target: p95 <= 6 seconds for 10k objects and p99 <= 15 seconds.
Target-033: SVG export target: p95 <= 5 seconds for diagram-compatible selections; this intentionally improves on FigJam's direct SVG export limitation.
Target-034: Export OCI Always Free overlay: p95 <= 12 seconds for 5k-object PNG/PDF export and queue larger exports.
Target-035: Template install target: p50 <= 300 ms, p95 <= 1.2 seconds excluding external settlement wait.
Target-036: Marketplace settlement split target: record external DealSet wait separately with p95 budget <= 2 seconds when dependency is healthy.
Target-037: Timer state propagation target: p95 <= 100 ms to all active editors.
Target-038: Voting ballot submit target: p95 <= 150 ms and final tally reveal p95 <= 300 ms after close.
Target-039: Follow/spotlight target: p95 <= 120 ms to move participants to facilitator focus region.
Target-040: REST/API control plane target: support at least 2,000 low-cost API calls per minute per tenant/app/principal bucket, matching Miro's Level 1 public rate-limit number.
Target-041: API heavy operation target: support at least 200 heavy operations per minute per tenant/app/principal bucket, matching Miro's Level 3 public rate-limit number.
Target-042: API 429 target: include Retry-After, limit, remaining, reset, operation family, tenant_class, deployment_context, and trace id.
Target-043: Rate-limit isolation target: bucket by tenant_id, principal_id, app_id, operation family, tenant_class, and deployment_context.
Target-044: Reconnect storm target: 250 editors rejoin within 90 seconds, p95 cursor recovery <= 3 seconds, no data loss.
Target-045: Region failover RTO target: board session read-only within 15 seconds and full write recovery within 90 seconds in public cloud, AWS, OCI paid, and Oyatie-as-provider contexts.
Target-046: On-prem/colo failover overlay: RTO target depends on facility topology but must publish measured RTO with the benchmark artifact.
Target-047: Replay freshness target: p95 <= 10 seconds after regional recovery for 10k-object boards.
Target-048: Availability target: >= 99.9 percent baseline because current SLO files target 0.999 at `slos/availability.openslo.yaml:26-30`.
Target-049: Paid contractual SLO target: >= 99.95 percent service availability where deployment_context supports multi-region operation.
Target-050: Revenue_share contractual SLO target: same as paid when substrate is approved, otherwise documented at-cost context overlay.
Target-051: Demo_trial SLO target: best-effort, but the benchmark quality thresholds remain identical until usage caps are hit.
Target-052: Public cloud overlay: elastic autoscaling supports canonical throughput targets and 100k-object stretch boards.
Target-053: Guest-on-AWS overlay: supports canonical targets when customer account grants required managed services and network budget.
Target-054: Guest-on-OCI overlay: supports canonical targets for paid/revenue_share; demo_trial uses OCI Always Free profile caps.
Target-055: On-prem overlay: supports canonical latency only when facility has local Valkey/event-log and low-latency websocket edge.
Target-056: Colo overlay: supports canonical latency when redundant edge and storage replicas are present.
Target-057: Oyatie-as-cloud-provider overlay: supports canonical public-cloud targets and should be the reference implementation.
Target-058: Benchmark publication target: every benchmark emits machine-readable results under a future `benchmarks/` path with source commit, OS, arch, context, tenant_class, and dataset hash.
Target-059: Regression gate target: no merge may regress p95 by more than 5 percent for canonical workloads without an accepted performance decision.
Target-060: Safety gate target: no benchmark may omit policy/audit overhead; performance without Cedar/audit is invalid.

## §4 Comparison Narrative
1. Cursor scale: Oyatie target of 250 active editors is ahead of FigJam's 200 editor limit and Miro's around-200 common board scenario.
2. Participant scale: Oyatie target of 750 total participants is ahead of FigJam's 500 public participant limit.
3. Demo_trial cap: 25 editors and 75 viewers is below the canonical target by design because it maps to OCI Always Free profile usage caps, not lower product quality.
4. Board object scale: Oyatie target of 50k live objects is ahead of Miro's 10k UI recommendation, but it requires progressive hydration and real benchmark proof.
5. Board object stretch: 100k-object open target is aspirational and must be measured before product claims.
6. Metadata: Oyatie target of 128 KB app metadata and 16 KB object metadata is ahead of Miro's documented 30 KB app and 6 KB item limits.
7. Bulk import: Oyatie target of 10k cells per batch is ahead of Miro's 5k-cell spreadsheet paste limit.
8. Export: Oyatie SVG export target is ahead of FigJam's direct SVG limitation, but only if diagram-compatible selection export is implemented.
9. API low-cost throughput: Oyatie target of 2,000 calls/min matches Miro's Level 1 public number.
10. API heavy operation throughput: Oyatie target of 200 heavy ops/min matches Miro's Level 3 public number.
11. API 429 semantics: Oyatie should match Miro, Figma, and Lucid by including Retry-After and clear headers, then exceed them with tenant_class and deployment_context attribution.
12. Rate-limit isolation: Oyatie should match Figma's per-app isolation principle and make tenant/app/principal dimensions explicit.
13. Timer: Oyatie should match FigJam's 99:59 timer capability and one-active-timer invariant unless product design chooses a stricter facilitator-governed model.
14. Voting: Oyatie should match FigJam hidden-choice/reveal-tally behavior and Lucid visual activity result persistence.
15. Static fallback: Oyatie should match FigJam's static-view fallback when session limits are reached, but with explicit snapshot latency targets.
16. Revision/replay: Oyatie must catch up to Lucidchart revision history and Miro board-history expectations by adding restore and revision browse APIs.
17. Diagram semantics: Oyatie is behind Lucidchart until it models pages, shapes, data-backed shapes, lines, groups, layers, and custom data.
18. Current implementation: no measured latency claim is allowed while `src/adapter/http.rs:65-67` returns a contract stub.
19. Current tests: no benchmark-complete claim is allowed while `tests/integration.rs:57-75` leaves core fixtures ignored.
20. Current SLOs: existing OpenSLO files are useful because they name the right signals, but they need millisecond thresholds and canonical labels.
21. Deployment contexts: no context overlay can be proven until OpenTofu context modules exist.
22. OCI Always Free: no demo_trial infrastructure claim can be proven until `iac/oci-guest/always-free/` exists.
23. OS/arch: no OS matrix claim can be proven until `supported-oses.json` exists.
24. Revenue_share: no at-cost substrate claim can be proven until capacity and cost budgets model revenue_share tenant_class explicitly.
25. Paid: no contractual SLO claim can be proven until paid tenant_class semantics exist in manifest, IaC, SLOs, dashboards, and runbooks.
26. Public cloud: target numbers should first be proven in Oyatie public cloud or Oyatie-as-cloud-provider because those contexts can control the full substrate.
27. Guest cloud: AWS/OCI guest contexts must include account-limit detection in benchmark output.
28. On-prem/colo: benchmark output must include facility network, storage, and hardware notes because context constraints vary.
29. Benchmark safety: all benchmark numbers must include policy, audit, encryption, and observability overhead.
30. Final benchmark verdict: target set is credible and industry-leader-grade, but current whiteboard artifacts do not yet implement or measure it.

## §5 Evidence-Locked Benchmark Adoption Plan
Bench-001: The first benchmark artifact should be `benchmarks/cursor-fanout/` because cursor latency is the service's most visible collaboration signal.
Bench-002: Cursor fanout input must include 25, 100, 250, 500, and 750 participant profiles so demo_trial caps and paid/revenue_share scale can be compared without changing feature quality.
Bench-003: Cursor fanout output must report p50, p95, p99, max latency, dropped updates, reconnect count, and static-view transitions.
Bench-004: Cursor fanout output must label OS, architecture, deployment_context, tenant_class, board object count, participant count, and source commit.
Bench-005: The second benchmark artifact should be `benchmarks/operation-log/` because CRDT merge success is named in local SLO files.
Bench-006: Operation-log input must include sticky edit, stroke append, connector move, object delete, comment add, vote submit, and export-start operations.
Bench-007: Operation-log output must report accepted operations per second, rejected operations per second, duplicate suppression, causal-order success, merge retries, and durable commit latency.
Bench-008: Operation-log output must include policy and audit overhead because `src/usecase/mod.rs:81-95` sequences policy, idempotency, audit, event, and persistence.
Bench-009: The third benchmark artifact should be `benchmarks/board-open/` because large-board load is the main scale gap against Miro/FigJam/Lucidchart.
Bench-010: Board-open datasets must include 1k, 10k, 50k, 100k, and 250k historical object boards.
Bench-011: Board-open output must report first byte, first interactive cursor, first visible object, all visible objects, full hydration, and memory peak.
Bench-012: Board-open output must identify whether progressive hydration is active because the 100k-object stretch target depends on it.
Bench-013: The fourth benchmark artifact should be `benchmarks/export-render/` because export is a named capability in PRD, manifest, and Rust domain files.
Bench-014: Export-render workloads must include PNG, PDF, CSV, SVG, and provenance packet outputs.
Bench-015: Export-render output must report queue wait, render time, upload/store time, policy check time, and final byte size.
Bench-016: Export-render output must include selected range and whole-board modes because customers use both workshop snapshots and archival exports.
Bench-017: The fifth benchmark artifact should be `benchmarks/template-install/` because template installation and marketplace-paid template economics appear in implementation plans.
Bench-018: Template-install output must separate local install latency from external DealSet or billing wait latency.
Bench-019: Template-install output must include rollback success rate because failed marketplace installs are customer-facing failures.
Bench-020: The sixth benchmark artifact should be `benchmarks/facilitation/` because FigJam and Lucid both expose facilitation-style surfaces.
Bench-021: Facilitation workloads must include timer start/pause/resume, vote open/submit/reveal, spotlight/follow, reaction burst, and participant lock/unlock.
Bench-022: Facilitation output must report fanout latency, tally latency, dropped reactions, facilitator command rejection, and audit event completeness.
Bench-023: The seventh benchmark artifact should be `benchmarks/diagram-semantics/` because Lucidchart parity depends on pages, shapes, layers, lines, groups, and custom data.
Bench-024: Diagram workloads must include page creation, layer toggle, grouped-shape move, connector reroute, data-backed shape refresh, and custom-data query.
Bench-025: Diagram output must report validation latency, render latency, connector reroute latency, and data refresh latency.
Bench-026: The eighth benchmark artifact should be `benchmarks/migration-import/` because migration playbooks are absent but counterpart parity requires data portability.
Bench-027: Migration-import workloads must include Miro board archive, FigJam export, Lucidchart diagram export, CSV/spreadsheet sticky import, and unsupported item handling.
Bench-028: Migration-import output must report import time, object preservation rate, unsupported item count, asset rewrite failures, and customer acceptance hash.
Bench-029: Every benchmark must reject runs that omit tenant_class because the replacement commercial model is now canonical.
Bench-030: Every benchmark must reject runs that omit deployment_context because context overlays are central to the master-plan sequence.
Bench-031: Every benchmark must reject runs that omit OS and architecture because `supported-oses.json` is a required per-service control surface.
Bench-032: Every benchmark must reject runs against non-Rust backend substitutes because backend language policy is Rust-strict.
Bench-033: Every benchmark must distinguish source-measured numbers from targets and estimates.
Bench-034: Every benchmark must cite the counterpart source used for target calibration when it compares against Miro, FigJam, or Lucidchart.
Bench-035: Every benchmark must publish raw samples, aggregate numbers, and harness configuration.
Bench-036: Every benchmark must include warmup duration, test duration, seed, data generator version, and failure injection mode.
Bench-037: Every benchmark must measure with authorization enabled, because policy-free performance is not product performance.
Bench-038: Every benchmark must measure with audit enabled, because collaboration and export flows are compliance-sensitive.
Bench-039: Every benchmark must measure with observability enabled, because production telemetry overhead is part of the service budget.
Bench-040: Every benchmark must state whether websockets, HTTP streaming, or another transport was used.
Bench-041: Every benchmark must state whether the board event log was memory-only, local disk, replicated storage, or managed database backed.
Bench-042: Every benchmark must state whether clients were local loopback, same-region, cross-region, on-prem LAN, or internet-distributed.
Bench-043: Public-cloud overlay runs must include autoscaling state, region, pod/node count, and storage class.
Bench-044: Guest-on-AWS overlay runs must include customer account service quotas and any throttling observed.
Bench-045: Guest-on-OCI overlay runs must separate paid/revenue_share OCI guest resources from the OCI Always Free profile.
Bench-046: OCI Always Free profile runs must publish OCPU, memory, storage, egress, and always-free resource assumptions.
Bench-047: On-prem runs must publish facility network latency, storage class, CPU model, memory, and operator-controlled constraints.
Bench-048: Colo runs must publish rack/network/storage redundancy and cross-connect latency.
Bench-049: Oyatie-as-cloud-provider runs must become the reference target because Oyatie controls the full substrate there.
Bench-050: Demo_trial runs must enforce hard usage caps but keep the same correctness assertions as paid and revenue_share runs.
Bench-051: Paid runs must show contractual SLO headroom under expected per-seat plus usage scaling.
Bench-052: Revenue_share runs must show at-cost substrate assumptions and the same quality bar as paid when infrastructure allows.
Bench-053: The first release gate should require cursor p95, operation p95, board-open p95, export p95, and merge success to have measured values.
Bench-054: The second release gate should require API rate-limit behavior, 429 headers, reconnect storm recovery, and static-view fallback to have measured values.
Bench-055: The third release gate should require migration import and diagram semantic benchmarks before Lucidchart-class parity claims.
Bench-056: The first regression rule should fail any p95 regression above 5 percent without an accepted performance decision.
Bench-057: The second regression rule should fail any correctness regression regardless of latency improvement.
Bench-058: The third regression rule should fail any benchmark that loses tenant_class or deployment_context labels.
Bench-059: The fourth regression rule should fail any benchmark that silently disables policy, audit, persistence, or telemetry.
Bench-060: The fifth regression rule should fail any benchmark that uses synthetic success without durable board-state verification.
Bench-061: Target calibration must treat Miro's public API limits as lower-bound public control-plane references, not full product internals.
Bench-062: Target calibration must treat FigJam's participant numbers as public session-behavior anchors, not a complete latency contract.
Bench-063: Target calibration must treat Lucid's page schema as a feature-shape anchor, not a published throughput benchmark.
Bench-064: Where counterpart public numbers are absent, Oyatie targets must be marked as internal canonical targets derived from product requirements and measured before external claims.
Bench-065: Cursor p95 <= 80 ms in elastic contexts is a target that must be measured, not a current claim.
Bench-066: 250 active editors is a target that must be measured, not a current claim.
Bench-067: 750 total participants is a target that must be measured with graceful degradation, not a current claim.
Bench-068: 50k live board objects is a target that must be measured with progressive hydration, not a current claim.
Bench-069: 100k live board objects is a stretch target that must be measured before appearing in sales material.
Bench-070: 250k historical objects is an archival target that must rely on lazy loading and snapshot compaction.
Bench-071: CSV/PDF/PNG/SVG export targets must be measured separately because each format has a different rendering and policy profile.
Bench-072: Migration import targets must include unsupported-item preservation reports because counterpart data models differ.
Bench-073: API throughput targets must be measured per operation family because generic request-per-minute numbers hide heavy export/import work.
Bench-074: Availability targets must distinguish service availability from individual board session health.
Bench-075: Failover targets must distinguish read-only recovery from full write recovery.
Bench-076: Local current-state evidence remains insufficient because the adapter returns a contract stub and integration tests are ignored.
Bench-077: Benchmark work should begin only after the contract model is expanded enough to exercise real board, object, session, export, template, and diagram operations.
Bench-078: Benchmark reports should be machine-readable first and human-readable second so future gates can consume them directly.
Bench-079: The performance doc should be retired or replaced once measured benchmark artifacts exist, because this file is a target-setting audit deliverable.
Bench-080: Final benchmark adoption criterion: measured data must prove the same product-quality bar across tenant classes, with only usage caps and infrastructure ceilings changing by overlay.
