# Oyatie — Product PRD: Vertical Education

> **Status:** preview (skeleton)
> **Owning team:** [`teams/vertical-education/CHARTER.md`](../../teams/vertical-education/CHARTER.md)
> **Owning axis:** vertical-education (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-education-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Education is a learning management and student information platform for K-12 schools, higher education institutions, and corporate training organizations. It covers LMS (course delivery, content management, learner progress), SIS (student enrollment, grades, attendance, transcripts), and AI-assisted grading and personalized learning pathways (Foundry-powered). It exists within Oyatie's ecosystem because the coupling of a single identity across student, instructor, and administrative roles, the privacy program enforcing `CHILDREN_UNDER_14` hard-deny for advertising, the audit chain providing accreditation-grade records, and the Corporate vertical's HR layer for staff management creates the integrated institutional operations platform that no standalone LMS or SIS can replicate.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Student / Learner | Course content, assignments, grades, certificates, personalized learning path (Foundry) | Included in institution subscription |
| Instructor / Educator | Course authoring, assignment grading (Foundry-assisted), attendance, analytics | Per-seat (instructor tier) |
| School Administrator | Enrollment, transcript management, accreditation reports, compliance dashboard | Per-seat (admin tier) |
| Corporate L&D Manager | Training curriculum management, completion tracking, skills gap analysis | Per-seat (L&D tier) |
| Education IT / Tenant Builder | LMS configuration, SIS integration, Foundry learning-workflow authoring | Builder seat |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Course creation and delivery (SCORM/xAPI), learner enrollment, assignment submission, basic grading, KR 학사관리 (university grade registry) | REST API v1, Web UI (LMS), Mobile learner app |
| Vertical-Stable | SIS (student enrollment lifecycle, attendance, transcripts, credit transfer), Foundry-assisted grading (rubric matching, feedback generation — instructor approves), personalized learning paths (Foundry adaptive sequencing), parent portal (K-12), accreditation evidence export, FERPA/KR-PIPA compliant data handling | REST API stable, LTI 1.3 integration, Webhook console |
| Public-GA | Skills-based credentials and blockchain-anchored certificates, AI-driven curriculum gap analysis (Foundry), alumni network, corporate training marketplace | Public OpenAPI, Analytics dashboard |

### 3.2 Out-of-scope (anti-scope)

- Advertising targeting using student data or learning behavior — `CHILDREN_UNDER_14` is HARD DENY; FERPA/KR-청소년보호법 prevent student data monetization
- Direct tutoring / live classroom video infrastructure (integration seam to Zoom/Teams/Webex; not in-house)
- Testing / standardized assessment proctoring at depth (declared as seam; anti-cheating proctoring is a specialized ISV)

---

## 4. Architecture Overview

### 4.1 Bounded Context

Flat-crates target prefix: `crates/oya-vertical-education-*`.

```
crates/oya-vertical-education-kernel-lms/      — Course, Module, Lesson, Assignment, Submission, Grade entities
crates/oya-vertical-education-kernel-sis/      — Student, Enrollment, Attendance, Transcript, CreditRecord entities
crates/oya-vertical-education-kernel-learning/ — LearningPath, SkillMap, LearnerProgress entities
crates/oya-vertical-education-domain-*/        — Use cases per sub-domain
crates/oya-vertical-education-app-*/           — Sagas + Foundry grading/path delegation
crates/oya-vertical-education-adapter-*/       — DB, xAPI/SCORM, LTI adapters
crates/oya-vertical-education-api-rest/        — REST API
crates/oya-vertical-education-runtime/         — Composition root
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Education REST API | `contracts/education-core.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| LTI 1.3 provider | `contracts/education-lti.yaml` | Data | 99.5% / p95 < 500ms |
| xAPI (Tin Can) statement endpoint | `contracts/education-xapi.yaml` | Data | 99.5% / p95 < 200ms |

### 4.4 Internal Seams

| Seam | Trait | Consumer products |
|---|---|---|
| `LearnerProgressSearchIndexable` | `SearchIndexable` (tenant-private) | Search axis (course discovery) |
| `StaffEnrollmentSync` | `IdentitySync` | Platform identity (SCIM — staff accounts) |

> TODO v0.2 — vertical owner to expand cross-axis dependency table.

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-education-kernel-lms
/// data_class: BEHAVIORAL_TENANT_PRODUCT (course content); PII_IDENTIFYING (student name/ID)
/// CHILDREN_UNDER_14 flag forces ad_targetable_blocked for K-12 tenants
/// plane: data
pub struct Course {
    pub id: CourseId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub title: String,                        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub description: String,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: CourseStatus,
    pub instructor_ids: Vec<UserId>,
    pub modules: Vec<ModuleRef>,
    pub credit_hours: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Grade {
    pub id: GradeId,
    pub tenant_id: TenantId,
    pub student_id: StudentId,                // data_class: PII_IDENTIFYING
    pub assignment_id: AssignmentId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub raw_score: Option<Decimal>,           // data_class: PII_IDENTIFYING (FERPA protected)
    pub letter_grade: Option<LetterGrade>,    // data_class: PII_IDENTIFYING
    pub feedback: Option<String>,             // data_class: PII_IDENTIFYING
    pub foundry_grading_run_id: Option<FoundryRunId>, // instructor must approve before persisting
    pub graded_by: UserId,                    // human instructor ID
    pub graded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// crates/oya-vertical-education-kernel-sis
pub struct Student {
    pub id: StudentId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub legal_name: PersonName,               // data_class: PII_IDENTIFYING
    pub date_of_birth: Option<NaiveDate>,     // data_class: PII_IDENTIFYING
    pub student_number: String,               // data_class: PII_IDENTIFYING
    pub enrollment_status: EnrollmentStatus,
    pub is_minor: bool,                       // forces CHILDREN_UNDER_14 class if true + < 14
    pub guardian_consent_ref: Option<ConsentId>, // required if is_minor = true
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

> TODO v0.2 — vertical owner to add `Enrollment`, `Transcript`, `Attendance`, `LearningPath`, `SkillMap` entities with full field enumeration and data_class annotations.

### 5.2–5.7

> TODO v0.2 — vertical owner to expand aggregate boundaries, persistence layout, event schemas, index touchpoints, audit-chain contract, migration policy.

Key audit events: `GradePosted`, `StudentEnrolled`, `TranscriptIssued`, `AttendanceRecorded`.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `tenant_id` → cell (institution-level isolation) |
| Sharding strategy | Per-tenant shard; xAPI statement ingestion in TimescaleDB hypertable per tenant |
| Caching tier | Redis for active course session state; in-memory for course catalog |
| Bulk endpoint contract | `POST /grades/bulk`; `POST /enrollments/bulk`; `POST /xapi/statements/bulk` |
| Agent-driven optimization | Foundry `AdaptiveLearningPath` (personalized content sequencing, T1 autonomy); Foundry `GradingAssist` (rubric-based feedback draft, instructor approves) |

> TODO v0.2 — vertical owner to expand.

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with |
|---|---|---|---|
| Academic calendar / grading scale | `LocaleFormatter` | Yes | `oya-pack-kr` (학기제, A+/A0 grade scale, NEIS integration), `oya-pack-us` (semester, GPA 4.0 scale, FERPA) |
| Student identity verification | `IdentityProvider` | Yes | `oya-pack-kr` (교육부 학생 본인확인), `oya-pack-us` (OSIS / state edu ID) |
| Accreditation evidence format | `RegulatoryPack` | Yes | `oya-pack-kr` (교육부, 한국대학교육협의회), `oya-pack-us` (SACSCOC, HLC, Middle States) |

### Regulatory Pack Declaration

```yaml
regulatory_packs:
  - oya-pack-kr   # 교육부, 개인정보보호법, 청소년보호법, NEIS
  - oya-pack-us   # FERPA, COPPA (K-12), IDEA (special ed), FAFSA
  - oya-pack-eu   # GDPR (minor data), Erasmus+ reporting
tenant_class_overrides:
  ad_targetable_blocked: true   # K-12 tenants: CHILDREN_UNDER_14 HARD DENY
```

---

## 8. In-House vs External Dependency Posture

> TODO v0.2 — vertical owner to expand. Key: `tokio`/`axum`/`sqlx`/`serde`/`rustls` (kernel-grade); xAPI/SCORM parsing in-house; LTI 1.3 library evaluation pending.

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Learners enrolled | ≥ 1,000 | ≥ 100,000 | ≥ 1,000,000 |
| Course completion rate | baseline | ≥ 70% | ≥ 80% |
| Foundry grading assist adoption | ≥ 20% of assignments | ≥ 60% | ≥ 80% |
| Transcript issuance P99 | < 2s | < 500ms | < 200ms |
| CHILDREN_UNDER_14 data ad-block enforcement | 100% | 100% | 100% |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Student minor data (CHILDREN_UNDER_14) in ads pipeline | Catastrophic | Structural: `CHILDREN_UNDER_14` class is HARD DENY at eventing backbone; no override path | Privacy + Architecture |
| FERPA violation (US student record disclosure) | Critical | All grade/transcript data is `PII_IDENTIFYING`; tenant-private only; no cross-tenant exposure; guardian consent required for minors | Privacy + US pack |
| Foundry grading hallucination (wrong grade applied) | High | Foundry grading is T1 (draft only); instructor must review + explicitly submit grade; Foundry draft never auto-posted | Foundry + Education domain |

> TODO v0.2 — vertical owner to expand risk register.

---

## 11. Open Questions

- KR NEIS (나이스) integration: direct API or batch file transfer? Affects SIS architecture.
- LTI 1.3 provider: build in-house or use `ims-lti` library?
- Special education accommodation tracking (US IDEA / KR 특수교육법) — in-scope for Stable?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| Foundry grading at T1 (draft only, instructor approves) | 2026-05-09 | Academic integrity; instructor is grade authority | ADR-0050 |
| `CHILDREN_UNDER_14` forced ad block for K-12 tenants | 2026-05-09 | COPPA / KR-청소년보호법 / GDPR-K mandate | PRIVACY-PROGRAM §2.2.3 |
| Flat-crates: `crates/oya-vertical-education-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.3 (`CHILDREN_UNDER_14` hard deny)

---

## Doc-Catalog Row

```
| `vertical-education` | `vertical-2` | LMS/SIS/grading; FERPA/COPPA/청소년보호법 | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3 |
```
