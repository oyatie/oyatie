use super::*;

#[derive(Clone, Copy, PartialEq)]
enum SampleStatus {
    Active,
    Probation,
    OnLeave,
}

impl SampleStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Active => "활성 · Active",
            Self::Probation => "수습 · Probation",
            Self::OnLeave => "휴직 · On leave",
        }
    }

    fn class(self) -> &'static str {
        match self {
            Self::Active => "status-chip success",
            Self::Probation => "status-chip warning",
            Self::OnLeave => "status-chip",
        }
    }
}

struct SampleEmployee {
    id: &'static str,
    name: &'static str,
    english_name: &'static str,
    title: &'static str,
    team: &'static str,
    team_label: &'static str,
    manager: &'static str,
    joined: &'static str,
    status: SampleStatus,
}

const EMPLOYEES: [SampleEmployee; 8] = [
    SampleEmployee {
        id: "emp_0000",
        name: "이재현",
        english_name: "Jaehyun Lee",
        title: "Chief Executive Officer",
        team: "office",
        team_label: "Office of CEO",
        manager: "—",
        joined: "2021-03-14",
        status: SampleStatus::Active,
    },
    SampleEmployee {
        id: "emp_0011",
        name: "최유나",
        english_name: "Yuna Choi",
        title: "Chief Financial Officer",
        team: "finance",
        team_label: "Finance",
        manager: "이재현",
        joined: "2022-04-01",
        status: SampleStatus::Active,
    },
    SampleEmployee {
        id: "emp_0001",
        name: "박서준",
        english_name: "Seojun Park",
        title: "VP of Engineering",
        team: "infrastructure",
        team_label: "플랫폼팀",
        manager: "이재현",
        joined: "2021-06-01",
        status: SampleStatus::Active,
    },
    SampleEmployee {
        id: "emp_0002",
        name: "김지영",
        english_name: "Jiyoung Kim",
        title: "Engineering Manager",
        team: "infrastructure",
        team_label: "플랫폼팀",
        manager: "박서준",
        joined: "2022-09-12",
        status: SampleStatus::Active,
    },
    SampleEmployee {
        id: "emp_0003",
        name: "윤태민",
        english_name: "Taemin Yoon",
        title: "Senior Software Engineer",
        team: "infrastructure",
        team_label: "플랫폼팀",
        manager: "김지영",
        joined: "2026-05-12",
        status: SampleStatus::Probation,
    },
    SampleEmployee {
        id: "emp_0004",
        name: "임도윤",
        english_name: "Doyun Lim",
        title: "Engineering Manager · Product",
        team: "product",
        team_label: "프로덕트팀",
        manager: "박서준",
        joined: "2022-11-04",
        status: SampleStatus::Active,
    },
    SampleEmployee {
        id: "emp_0005",
        name: "강수아",
        english_name: "Sua Kang",
        title: "Software Engineer",
        team: "product",
        team_label: "프로덕트팀",
        manager: "임도윤",
        joined: "2024-02-19",
        status: SampleStatus::OnLeave,
    },
    SampleEmployee {
        id: "emp_0006",
        name: "정우진",
        english_name: "Woojin Jung",
        title: "Software Engineer · Frontend",
        team: "data",
        team_label: "데이터팀",
        manager: "임도윤",
        joined: "2025-08-07",
        status: SampleStatus::Active,
    },
];

pub(super) fn sample_employee_directory() -> impl IntoView {
    let count = |status| {
        EMPLOYEES
            .iter()
            .filter(|employee| employee.status == status)
            .count()
    };
    let team_count = |team| {
        EMPLOYEES
            .iter()
            .filter(|employee| employee.team == team)
            .count()
    };
    view! {
        <div data-employee-directory="sample" data-employee-source="eight-row-sample-v1">
            <p>"Sample directory · eight example records. No live HR source or verified employee access is connected."</p>
            <div class="employee-directory-stats" aria-label="Sample employee totals">
                <span><small>"전체 · Sample employees"</small><strong>{EMPLOYEES.len()}</strong></span>
                <span><small>"활성 · Sample active"</small><strong>{count(SampleStatus::Active)}</strong></span>
                <span><small>"수습 · Sample probation"</small><strong>{count(SampleStatus::Probation)}</strong></span>
                <span><small>"휴직 · Sample on leave"</small><strong>{count(SampleStatus::OnLeave)}</strong></span>
            </div>
            <p>"Employee readiness unavailable · no checklist or supporting evidence has been evaluated."</p>
            <div class="employee-directory-tools">
                <label><span>"Search samples"</span><input type="search" data-employee-search="true" aria-label="Search sample employees" placeholder="이름, 직책, 팀, ID 검색..." /></label>
                <div class="employee-filter-pills" role="group" aria-label="Sample employee team filters">
                    <button type="button" class="active" data-employee-filter="all" aria-pressed="true">{format!("전체 · All {}", EMPLOYEES.len())}</button>
                    <button type="button" data-employee-filter="infrastructure" aria-pressed="false">{format!("플랫폼팀 · Infrastructure {}", team_count("infrastructure"))}</button>
                    <button type="button" data-employee-filter="finance" aria-pressed="false">{format!("Finance {}", team_count("finance"))}</button>
                </div>
                <button type="button" data-employee-clear="true">"Clear search and filters"</button>
                <button type="button" data-identity-action="add-employee" disabled aria-describedby="employee-add-unavailable">"+ 직원 추가 · Add employee unavailable"</button>
            </div>
            <p id="employee-add-unavailable">"Adding employees is unavailable in this sample preview. A live HR source and verified HR access are required; no employee or invitation is created here."</p>
            <p data-employee-count="true" role="status" aria-live="polite" aria-atomic="true">{format!("{} matching sample employees · {} of {} shown", EMPLOYEES.len(), EMPLOYEES.len(), EMPLOYEES.len())}</p>
            <p data-employee-reset="true" role="status" hidden>"Sample context changed. Search, filters and open profiles reset."</p>
            <p data-employee-empty="true" hidden>"No matching sample employees. Clear search and filters to show all samples."</p>
            <p id="employee-table-scroll-help">"Scroll horizontally to see all sample columns."</p>
            <div class="employee-directory-scroll" data-employee-scroll="true" role="region" aria-label="Sample employee table" aria-describedby="employee-table-scroll-help" tabindex="0">
                <table class="employee-directory-table">
                    <caption>"Sample employee records · names, dates and status are examples"</caption>
                    <thead><tr><th scope="col">"이름"</th><th scope="col">"직책"</th><th scope="col">"부서 · 팀"</th><th scope="col">"매니저"</th><th scope="col">"Sample 입사일"</th><th scope="col">"Sample 상태"</th><th scope="col">"Inspect sample"</th></tr></thead>
                    <tbody>{EMPLOYEES.iter().map(sample_employee_row).collect_view()}</tbody>
                </table>
            </div>
        </div>
    }
}

fn sample_employee_row(employee: &'static SampleEmployee) -> impl IntoView {
    let search = format!(
        "{} {} {} {} {} {}",
        employee.name,
        employee.english_name,
        employee.id,
        employee.title,
        employee.team,
        employee.team_label
    );
    view! {
        <tr data-employee-row="true" data-employee-id=employee.id data-employee-team=employee.team data-employee-search-text=search>
            <td><strong>{employee.name}</strong><small>{format!("{} · {}", employee.english_name, employee.id)}</small></td>
            <td>{employee.title}</td><td>{employee.team_label}</td><td>{employee.manager}</td><td>{employee.joined}</td>
            <td><span class=employee.status.class()>{employee.status.label()}</span></td>
            <td>
                <details data-employee-inspection=employee.id>
                    <summary data-employee-action="inspect" aria-label=format!("Inspect sample employee {} · {}", employee.name, employee.id)>"Inspect"</summary>
                    <div>
                        <p class="screen-anchor">"SAMPLE PROFILE"</p>
                        <dl>
                            <dt>"Sample name"</dt><dd>{format!("{} · {}", employee.name, employee.english_name)}</dd>
                            <dt>"Sample ID"</dt><dd>{employee.id}</dd>
                            <dt>"Sample title"</dt><dd>{employee.title}</dd>
                            <dt>"Sample team"</dt><dd>{employee.team_label}</dd>
                            <dt>"Sample status"</dt><dd>{employee.status.label()}</dd>
                            <dt>"Employee readiness"</dt><dd>"Unavailable · no checklist or evidence evaluated"</dd>
                        </dl>
                        <button type="button" data-employee-close="true">"Close sample profile"</button>
                    </div>
                </details>
            </td>
        </tr>
    }
}

#[cfg(test)]
#[path = "employee_directory_test.rs"]
mod tests;
