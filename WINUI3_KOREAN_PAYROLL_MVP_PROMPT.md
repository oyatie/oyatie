# Greenfield WinUI 3 + TypeScript Korean Payroll MVP Prompt

```text
You are a senior Windows desktop engineer, TypeScript systems engineer, and Korean HR/payroll SaaS architect.

Build a greenfield, fully functional, standalone Korean corporate payroll automation MVP.

There are no existing files.
Start from an empty folder and create the full project.

The application must be convincing as a native Windows desktop proof-of-concept, not a toy demo.

Important stack decision:
- Use WinUI 3 / Windows App SDK for the native Windows desktop shell.
- Use TypeScript / Node.js for the payroll engine.
- C# is allowed only for the WinUI 3 Windows shell.
- All payroll, import, classification, validation, matching, calculation, audit, comparison, and export logic must live in the TypeScript engine.
- The WinUI 3 shell must contain no payroll formulas.
- The WinUI 3 shell must call the local TypeScript engine.
- No web app.
- No React.
- No Vite.
- No Electron.
- No browser/webview UI.
- No HTTP server.
- No backend/frontend service split.
- No cloud.
- No external database server.
- Local PC only.
- Windows native desktop only.

Target outcome:
A Windows native desktop app where a user selects a local Excel file, ZIP file, or folder. The app reads local files, classifies Korean HR/payroll documents, previews detected data, identifies blockers, calculates payroll under versioned Korean payroll rules, allows administrator corrections, logs every correction, compares against existing payroll ledgers, and exports payroll outputs.

Do not use the word “upload” anywhere in product copy, README, CLI messages, UI labels, or tests. This is local file import/read, not upload.

Monorepo structure:
Create this structure:

- `apps/windows-shell/`
  - WinUI 3 app.
  - Native Windows UI.
  - File/folder picker.
  - Wizard.
  - Review grids.
  - Blocker resolution.
  - Payroll preview.
  - Override workflow.
  - Comparison matrix screen.
  - Export screen.
  - Calls local TypeScript engine.

- `packages/payroll-engine/`
  - TypeScript package.
  - Node.js 20+.
  - Pure payroll/domain logic.
  - Excel/ZIP import.
  - File/sheet classification.
  - Normalization.
  - Employee matching.
  - Attendance parsing.
  - Ruleset selection.
  - Payroll calculation.
  - Validation/blockers.
  - Audit logs.
  - Comparison matrix.
  - Export generation.
  - CLI entry for tests and automation.

- `fixtures/`
  - Generated sample Korean payroll folders/files.
  - Must be created by the project itself.
  - Do not assume user-provided files exist.

- `docs/`
  - Requirements.
  - Implementation plan.
  - Test criteria.
  - ADRs.
  - Legal/rate source notes.

- `out/`
  - Generated outputs.

Greenfield bootstrapping:
Create everything needed:
- solution/project files
- package files
- TypeScript config
- test config
- WinUI 3 project
- local engine CLI
- sample fixture generator
- README
- ADRs
- test fixtures
- E2E tests

Required commands:
From repo root:

- `npm install`
- `npm test`
- `npm run build`
- `npm run e2e`
- `npm run generate:fixtures`
- `npm run engine -- --input "./fixtures/26년 인사프로그램 (2)" --period 2026-01 --out "./out/e2e"`
- `dotnet build apps/windows-shell`
- command or README instruction to launch the WinUI 3 app

If a unified root script is feasible, also provide:
- `npm run build:all`
- `npm run test:all`

Engine requirements:
The TypeScript engine must support:
- local Excel file input
- local ZIP file input
- local folder input
- recursive folder scan
- `.xlsx`
- `.xls`
- `.pdf` as reference/attachment
- Korean filenames
- Unicode NFC/NFD Korean paths
- spaces
- parentheses
- Windows paths
- mixed slash styles
- shell-escaped paths
- mojibake/corrupted filenames where possible

Path examples that must work:
- `C:\Users\user\Downloads\26년 인사프로그램 (2)`
- `"C:\Users\user\Downloads\26년 인사프로그램 (2)"`
- `/Users/example/Downloads/26년\ 인사프로그램\ \(2\)`
- `./fixtures/26년 인사프로그램 (2)`

Fixture generator:
Because no user files are provided, create realistic generated fixtures.

`npm run generate:fixtures` must create:

`fixtures/26년 인사프로그램 (2)/`
- `5일자 급여/`
- `10일자 급여/`
- `15일자 급여/`
- `17일자 급여/`

Inside those folders generate nested Korean payroll files:
- attendance workbook
- payroll ledger workbook
- payslip workbook
- annual leave workbook
- HR master workbook
- bonus/allowance workbook
- invoice/reference workbook
- ambiguous workbook requiring review
- misleading filename with classifiable content
- mojibake-like filename case
- dummy PDF/reference files

Generated Excel files must include realistic messy patterns:
- Korean sheet names
- multiple sheets
- title rows before headers
- merged-cell-like layout where possible
- multi-row payroll headers
- attendance day columns `1..31`
- weekday rows
- Korean employee names
- duplicate names
- missing employee numbers
- employees across multiple files
- workplace/job/pay-date differences
- annual salary workers
- hourly workers
- irregular allowances
- low-confidence attendance codes
- existing payroll ledger values intentionally different from generated results for comparison matrix

Local runtime flow:
1. User selects or provides local path.
2. App/engine normalizes the path.
3. If ZIP, extract to local workspace.
4. If folder, scan recursively.
5. If Excel, process directly.
6. Register PDFs/reference files.
7. Read workbook and sheet metadata.
8. Classify files and sheets.
9. Detect workplace, pay date, payroll month, job group, pay type.
10. Normalize rows.
11. Match employees.
12. Infer attendance codes.
13. Select rulesets.
14. Create blockers/warnings/review items.
15. Calculate payroll.
16. Allow correction/override.
17. Recalculate immediately after override.
18. Audit every override.
19. Compare against existing payroll ledger.
20. Export local output files.

File/sheet classification:
Do not classify by filename alone.

Use weighted evidence from:
- folder path
- filename
- extension
- sheet names
- workbook dimensions
- title rows
- header rows
- multi-row headers
- payroll amount columns
- attendance day columns
- employee identifier columns
- month/pay-date patterns
- row density
- numeric patterns
- Korean/English aliases
- formula/reference sheets

Categories:
- 근태현황표
- 급여대장
- 급여명세서
- 연차대장
- 직원/인사 기본자료
- 청구서/도급비/일용비
- 상여금/성과금/격려금/귀향비
- 퇴직금/소득세정산
- 퇴직연금
- PDF/첨부자료
- 기타/검토필요

Every classification must include:
- category
- confidence score 0-100
- evidence
- missing evidence
- status

Statuses:
- 완료
- 경고
- 오류
- 검토필요
- 승인차단

Employee matching:
Never match employees by name only.

Support:
- duplicate names
- missing employee numbers
- inconsistent identifiers
- transfers
- terminated employees
- same employee appearing in HR master, attendance, payroll ledger, annual leave, and payslip files

Matching signals:
- 사번
- name
- workplace
- department
- job group
- hire date
- termination date
- birth year/month if present
- payroll period
- source file
- row fingerprint
- sheet name
- pay-date group

If employee number is missing:
- generate stable internal employee code
- persist mapping in local workspace
- show generated code in review UI
- low-confidence matches must become 승인차단

Attendance:
Assume attendance codes differ by workplace.

Support codes like:
- `●`
- `반`
- `휴`
- `연`
- `특`
- `야`
- numbers
- blanks
- mixed text

Attendance parser must:
- detect day columns
- detect weekday rows
- detect employee/name columns
- infer code meaning with confidence
- batch similar uncertain codes
- block unresolved critical codes
- save confirmed mappings as local rulesets

Ruleset design:
No payroll formulas in UI.
No company-name hardcoding.
All payroll logic must be ruleset-driven.

Rules selected by:
- workplace
- legal employer
- job group
- pay date
- pay type
- attendance basis
- effective date
- user-confirmed mapping

Same workplace may have multiple payroll standards.
Same company may have multiple pay dates.

Pay types:
- 연봉직
- 월급직
- 시급직
- 일용직/일급직 where present
- 관리자/현장직 where present

Korean payroll calculation support:
- 기본급
- 시급
- 근무시간
- 통상시급
- 연장근로수당
- 야간근로수당
- 휴일근로수당
- 특근수당
- 주휴수당
- 연차수당
- 고정연장수당
- 직책수당
- 직무수당
- 복지수당
- 만근수당
- 상여금
- 성과금
- 격려금
- 귀향비
- 과세수당
- 비과세수당
- 식대
- 소득세
- 지방소득세
- 국민연금
- 건강보험
- 장기요양보험
- 고용보험
- 기타공제
- 공제합계
- 차인지급액
- 회사부담분
- 총 인건비 estimate

Legal/rate data:
All legal, tax, insurance, minimum wage, and payroll assumptions must be versioned data.

Each ruleset/table must include:
- id
- name
- jurisdiction
- effective_from
- effective_to
- source_url
- source_reviewed_at
- assumptions
- confidence
- requires_review
- notes

Seed 2026 Korea defaults as editable/versioned data:
- National Pension workplace contribution: 9.5% total, employee 4.75%, employer 4.75%.
- National Pension monthly standard income floor/ceiling for July 2025 to June 2026: KRW 400,000 / KRW 6,370,000.
- 2026 Health Insurance total rate: 7.19%, split 3.595% / 3.595%.
- 2026 Long-Term Care income-rate total: 0.9448%, health premium ratio 13.14%.
- Employment Insurance employee unemployment-benefit rate: 0.9%.
- 2026 minimum wage: KRW 10,320/hour.
- 2026 monthly minimum wage equivalent: KRW 2,156,880.

Withholding tax:
- Support official NTS simplified withholding table import.
- If no official table exists, use `DEMO_ESTIMATE`.
- Every payroll result must show tax method:
  - `OFFICIAL_TABLE`
  - `IMPORTED_TABLE`
  - `DEMO_ESTIMATE`
- `DEMO_ESTIMATE` must create 검토필요.
- Do not present estimated withholding as legally final.

Official source anchors to document:
- National Pension Service:
  https://www.nps.or.kr/eng/ntnlpnsplan/cntb/getOHAI0013M0.do
- NTS withholding calculator:
  https://www.nts.go.kr/english/ad/help/myWthtxCalclPage.do?mi=11212
- Employment Insurance rates:
  https://edrm.ei.go.kr/ei/eim/eg/ei/eiEminsr/retrieveEi0301Info.do
- 2026 health insurance rate:
  https://m.korea.kr/briefing/pressReleaseView.do?gubun=pressRelease&newsId=156715187
- 2026 long-term care rate:
  https://www.mohw.go.kr/board.es?act=view&bid=0027&list_no=1487817&mid=a10503010200
- 2026 minimum wage notice:
  https://www.moel.go.kr/info/lawinfo/instruction/view.do?bbs_seq=20250800121

WinUI 3 UX:
Build a native Fluent-style wizard.

Screens:
1. 시작
   - local-only privacy notice
   - select Excel / ZIP / folder

2. 파일 읽기
   - progress indicator
   - discovered file counts
   - Excel count
   - PDF/reference count
   - ZIP extraction status

3. 자동 분류
   - grouped by pay date / workplace / category / month
   - confidence badges
   - review-needed count

4. 기준 정보 확인
   - workplace
   - legal employer
   - pay date
   - payroll month
   - job group
   - pay type

5. 직원 매칭
   - matched employees
   - generated internal employee codes
   - duplicate-name warnings
   - blocker list

6. 근태코드 확인
   - inferred attendance code meanings
   - confidence scores
   - batch approve/correct mappings

7. 계산 기준 확인
   - ruleset selection
   - effective dates
   - assumptions
   - source/review metadata

8. 급여 계산 미리보기
   - payroll grid
   - base pay
   - overtime
   - night work
   - holiday work
   - annual leave
   - taxable/non-taxable
   - insurance
   - tax
   - net pay

9. 검토필요/차단 항목
   - blockers
   - warnings
   - review-needed items
   - final approval blocked until blockers resolved

10. 관리자 수정
   - editable cells clearly marked
   - calculated cells locked
   - override reason required
   - live recalculation
   - before/after diff

11. 기존 급여대장 비교
   - employee/component matrix
   - differences highlighted
   - variance threshold warnings

12. 내보내기
   - output summary
   - clickable local paths
   - export success/failure details

UX principles:
- Use clear Korean business language.
- Minimize questions.
- Ask only when payroll correctness depends on the answer.
- Batch similar review items.
- Show why the app made each inference.
- Let user approve high-confidence mappings in bulk.
- Force resolution of blockers.
- Save confirmed mappings for future runs.
- Make progress feel fluid.
- Long operations must not freeze the UI.
- Support cancel/retry.
- Errors must have plain-language explanation and technical details.

Privacy:
- Mask resident registration numbers.
- Mask phone numbers.
- Mask bank accounts.
- Do not print sensitive raw identifiers in logs.
- Original files, normalized data, calculation results, overrides, approval history, exports, and audit logs must be stored separately.
- Audit log is append-only.

Retention:
- Document Korean HR/payroll retention assumptions.
- Make retention policy configurable.
- Do not silently delete files.
- Add retention config.

TypeScript module structure:
Create:

- `src/path`
- `src/import`
- `src/classification`
- `src/domain`
- `src/normalization`
- `src/matching`
- `src/rules`
- `src/calculation`
- `src/validation`
- `src/audit`
- `src/compare`
- `src/export`
- `src/storage`
- `src/cli`
- `src/fixtures`

Testing:
Use TDD.
Write failing tests before implementation where practical.
Do not delete tests to pass.
Do not reduce scope silently.

Required TypeScript unit tests:
- path normalization
- NFC/NFD Korean path handling
- shell-escaped path handling
- recursive discovery
- ZIP extraction
- `.xlsx` parsing
- `.xls` parsing
- PDF/reference registration
- workbook metadata extraction
- file classification
- sheet classification
- content-based classification
- ambiguous file becomes 기타/검토필요
- multi-row payroll header normalization
- attendance day/weekday parsing
- employee code generation
- duplicate-name handling
- low-confidence employee match blocker
- attendance code inference
- low-confidence attendance blocker
- ruleset selection by workplace/job/pay-date/effective date
- national pension floor/ceiling
- health insurance
- long-term care
- employment insurance
- overtime pay
- night work pay
- holiday work pay
- weekly holiday allowance
- annual leave allowance
- irregular allowance handling
- withholding DEMO_ESTIMATE warning
- net pay calculation
- admin override audit log
- comparison matrix
- export generation

Required E2E test:
`npm run e2e`

Must prove:
- fixture generation
- local folder path flow
- recursive scan
- classification
- preview JSON generation
- blocker detection
- deterministic blocker resolution in test mode
- payroll calculation
- correction/override
- audit log
- comparison matrix
- export generation

E2E output must include:
- payroll ledger Excel
- payslips
- review-needed report
- comparison matrix
- audit log
- normalized JSON package
- readable transcript

WinUI 3 tests/checks:
- app builds
- app launches
- engine process can be invoked
- wizard can load generated fixture path
- blocker prevents export
- correction updates totals
- export screen shows output paths
- engine errors display clearly

Documentation:
Create:
- `README.md`
- `REQUIREMENTS.md`
- `IMPLEMENTATION_PLAN.md`
- `TEST_CRITERIA.md`
- `docs/adr/0001-winui3-native-windows-shell.md`
- `docs/adr/0002-typescript-payroll-engine-boundary.md`
- `docs/adr/0003-ruleset-driven-korean-payroll.md`
- `docs/adr/0004-confidence-scoring-and-blockers.md`
- `docs/adr/0005-fluid-wizard-ux.md`
- `docs/adr/0006-greenfield-fixture-driven-e2e-proof.md`

README must include:
- prerequisites
- Windows setup
- Node.js setup
- .NET / Windows App SDK setup
- exact commands
- how to generate fixtures
- how to run engine CLI
- how to run E2E
- how to build WinUI 3 app
- how to launch WinUI 3 app
- output file list
- legal disclaimer
- legal/rate source URLs
- reviewed date
- privacy/masking behavior
- retention policy
- local file reading explanation

Implementation discipline:
1. Scaffold repo.
2. Add fixture generator.
3. Add path tests.
4. Implement path normalization.
5. Add discovery tests.
6. Implement file discovery.
7. Add Excel/ZIP tests.
8. Implement import layer.
9. Add classification tests.
10. Implement classification.
11. Add matching tests.
12. Implement employee matching.
13. Add attendance tests.
14. Implement attendance parser.
15. Add ruleset/calculation tests.
16. Implement payroll calculation.
17. Add validation/blocker tests.
18. Implement validation.
19. Add audit tests.
20. Implement audit.
21. Add compare/export tests.
22. Implement compare/export.
23. Add E2E test.
24. Make E2E pass.
25. Build WinUI 3 shell.
26. Wire shell to engine.
27. Add docs and ADRs.
28. Verify all commands.

Acceptance criteria:
- Project works from an empty folder.
- Fixtures are generated automatically.
- `npm install` succeeds.
- `npm test` passes.
- `npm run build` passes.
- `npm run e2e` passes.
- TypeScript engine processes generated Korean payroll fixture folder.
- Engine produces all required exports.
- WinUI 3 app builds.
- WinUI 3 app launches.
- User can select Excel, ZIP, or folder.
- App shows classification preview.
- App shows blockers.
- App prevents export while blockers remain.
- App calculates payroll after blockers are resolved.
- App allows admin override with reason.
- App recalculates totals after override.
- App writes audit log.
- App compares against existing payroll ledger.
- App exports required output files.

Definition of done:
The MVP is not complete until:
- TypeScript unit tests pass.
- E2E tests pass.
- WinUI 3 app builds and launches.
- Generated messy Korean fixture folder is processed successfully.
- Output files are generated.
- README commands are accurate.
- ADRs exist.
- Legal/rate sources are documented.
- Uncertain legal/tax logic is marked 검토필요.
```
