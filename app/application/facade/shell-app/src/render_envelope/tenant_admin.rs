use super::builders::{
    approval, community, fact, message, metric, module, node, s, schedule, suggestion, work,
    workflow,
};
use super::product_activity::product_activity_spine;
use super::types::{AccreditationState, ModuleCard, OperatorContext, TenantRenderEnvelope};
use application_ontology_card::OntologyCardSource;

pub(super) fn tenant_admin_envelope(ontology: &dyn OntologyCardSource) -> TenantRenderEnvelope {
    TenantRenderEnvelope {
        context: OperatorContext::TenantAdmin,
        tenant_name: s("Northwind Industrial Group"),
        role_name: s(OperatorContext::TenantAdmin.role()),
        tenant_class: s("Enterprise tenant · US/EU/KR packs enabled"),
        accreditation: AccreditationState {
            label: s("Healthcare not accredited for this tenant"),
            healthcare_enabled: false,
            explanation: s(
                "Healthcare-regulated surfaces are absent from this render envelope because the tenant lacks accredited healthcare state.",
            ),
        },
        server_derivation_note: s(
            "Server-derived envelope: admin can see tenant posture, cloud controls, approvals, service catalog, and workflow governance only.",
        ),
        product_activity: product_activity_spine(OperatorContext::TenantAdmin),
        metrics: vec![
            metric("Close progress", "73%", "+12 vs Mar · payroll run"),
            metric("Cycle time", "5.4d", "+1.4d vs target"),
            metric("Cost of delay", "₩2,180,000", "+₩410k"),
            metric("Open approvals", "8", "2 overdue"),
            metric("Compliance", "4/6", "2 review"),
        ],
        modules: crate::shell_capability_registry::permitted_module_cards(
            OperatorContext::TenantAdmin,
        )
        .into_iter()
        .map(|card| with_ontology_status(card, ontology))
        .collect(),
        daily_tasks: vec![
            work(
                "2026-04 급여 마감 — 박서준 직원 4대보험 변동 확인 필요",
                "건강보험료 등급 상승(변동액 +₩47,200/월). 마감 전 확인 후 승인.",
                "High",
            ),
            work(
                "연차 사용 승인 — 김지영 (5/13 – 5/17, 5일)",
                "$60 잔여 13.5일 → 8.5일. 팀 백업 확인됨.",
                "Medium",
            ),
            work(
                "원천징수이행상황신고서 — 2026-04 제출 준비 완료",
                "118명 직원 검증 완료. 홈택스 전송 대기.",
                "Medium",
            ),
        ],
        schedule: vec![
            schedule("09:30", "Cloud spend review", "FinOps + accounting owners"),
            schedule("11:00", "Access recertification", "Quarterly admin control"),
            schedule(
                "15:30",
                "Workflow change board",
                "Review no-code automation drafts",
            ),
        ],
        messages: vec![
            message(
                "Ops bot",
                "Messenger",
                "Kubernetes runtime tier drift detected in cell-us-east-2.",
            ),
            message(
                "Finance lead",
                "Mail",
                "Please approve department budget tags before close.",
            ),
            message(
                "Security reviewer",
                "Messenger",
                "Network split requires audit-chain evidence before promotion.",
            ),
        ],
        community: vec![
            community(
                "Governance council",
                "Network split RFC",
                "3 reviewers discussing rollback evidence",
            ),
            community(
                "FinOps circle",
                "Budget tag playbook",
                "New guidance pinned for department owners",
            ),
            community(
                "Workflow builders",
                "Payroll template request",
                "Corporate HR is asking for a reusable no-code flow",
            ),
        ],
        approvals: vec![
            approval(
                "구매 승인 경로 단축 가능 — Stripe 청구서 (₩4,820,000)",
                "Procurement",
                "정책상 < ₩5M는 1단계 승인 가능. 현재 3단계로 라우팅 중.",
            ),
            approval(
                "Payroll workflow template request",
                "Corporate HR",
                "No production execution; draft routes to Mail and Community.",
            ),
            approval(
                "Increase compute quota",
                "Factory systems",
                "Budget owner review required before runtime tier change.",
            ),
        ],
        workflow: workflow(
            "Tenant change approval",
            "No-code approval path for risky cloud and tenant configuration changes.",
            vec![
                node(
                    "intake",
                    "Request intake",
                    "Form",
                    55,
                    82,
                    "Captures a tenant change request and maps it to a ChangeRequest ontology object.",
                ),
                node(
                    "policy",
                    "Policy check",
                    "Guardrail",
                    250,
                    82,
                    "Evaluates residency, role, and accreditation gates before any reviewer sees the change.",
                ),
                node(
                    "approval",
                    "Admin approval",
                    "Human",
                    445,
                    82,
                    "Routes high-risk requests to the tenant admin; execution stays disabled until live integration.",
                ),
                node(
                    "evidence",
                    "Evidence note",
                    "Audit",
                    640,
                    82,
                    "Drafts audit-chain evidence text for review, not production emission.",
                ),
            ],
        ),
        ontology: vec![
            fact(
                "Tenant",
                "owns enabled module set",
                "Admin role can inspect module posture.",
            ),
            fact(
                "ChangeRequest",
                "requires approval",
                "High-risk changes route to authorized reviewers.",
            ),
            fact(
                "Budget",
                "constrains cloud action",
                "FinOps module is visible to tenant admins.",
            ),
        ],
        intelligence: vec![
            suggestion(
                "병목: 공계 승인 단계 (2.1d)",
                "과거 12회 사이클 평균 0.6d. 최유나 매니저에 위임 권한 자동 부여 시 1.4d 단축.",
                "Read-only suggestion; never auto-executes.",
            ),
            suggestion(
                "원천세 신고 자동 제출 가능",
                "118명 검증 완료. 사업자등록번호 1건 확인 후 자동 전송.",
                "User must review before HomeTax send.",
            ),
            suggestion(
                "$53 위험: 윤태민",
                "이번 주 49.5h 예상. 백업 가능 인력 2명 추천.",
                "Uses permitted roster data only.",
            ),
        ],
        omitted_capability_note: s(
            "Healthcare and patient-care modules are not present in this tenant admin envelope; they are not hidden client-side.",
        ),
    }
}

/// The Ontology card carries the wedge product's status after its registry
/// copy; a refusal is shown as such, never as stale or invented numbers.
fn with_ontology_status(mut card: ModuleCard, ontology: &dyn OntologyCardSource) -> ModuleCard {
    if card.name != "Ontology" {
        return card;
    }
    let status = match ontology.ontology_card_facts() {
        Ok(facts) => format!(
            "policy {} · serving {} tenants · projection lag {} · poisoned {}",
            facts.policy_version,
            facts.served_tenants,
            facts.projection_lag,
            facts.poisoned_entries
        ),
        Err(error) => error.to_string(),
    };
    card.description = format!("{} · {status}", card.description);
    card
}
