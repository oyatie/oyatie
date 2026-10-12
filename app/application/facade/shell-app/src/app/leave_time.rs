use super::*;

pub(super) fn leave_time_panel(context: OperatorContext) -> impl IntoView {
    let active = !finance_sample_scope(context).commercial_examples;
    let source = format!("sample-may-2026:{}", context.id());
    view! {
        <article id="finance-panel-leave" class=if active { "finance-panel active" } else { "finance-panel" } data-finance-panel="leave" role="tabpanel" aria-labelledby="finance-tab-leave"
            data-leave-context=context.id() data-leave-source=source
            data-leave-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }>
            <div class="finance-panel-copy">
                <p class="screen-anchor">"HCM · SAMPLE MAY 2026"</p>
                <h4 id="leave-panel-title" tabindex="-1">"Sample leave & time preview"</h4>
                <p>"Example requests, hours, coverage, utilization, and liability only. No live leave, schedule, timesheet, or payroll data is connected."</p>
                <p>"HR permission, installed rulepack and effective date, employee readiness, and decision evidence are unavailable. Foundry access does not grant HR approval permission."</p>
            </div>
            <label for="leave-sample-search">"Search sample leave and time"</label>
            <input id="leave-sample-search" type="search" placeholder="Request, coverage, cutoff" aria-describedby="leave-sample-count" />
            <p id="leave-sample-count" data-leave-projected="count" role="status">"3 sample items"</p>
            <p data-leave-status="true" data-leave-projected="status" role="status">"Local sample preview; no approval, assignment, lock, receipt, or payroll effect."</p>
            <div class="leave-layout">
                <ol class="leave-queue" aria-label="Sample May 2026 leave and time items">
                    <li data-leave-item="request"><time>"Sample 2026-05-13 through 2026-05-17"</time><strong>"김지영 sample leave request"</strong><span>"5 requested sample units; approved duration and backup eligibility are not established"</span><button type="button" data-finance-action="approve-leave">"Review sample request"</button></li>
                    <li data-leave-item="coverage"><time>"Sample period 2026-05-11 through 2026-05-17"</time><strong>"윤태민 sample coverage"</strong><span>"Example 49.5h; no actual hours, overtime determination, or legal entitlement"</span><button type="button" data-finance-action="reassign-time">"Preview sample coverage"</button></li>
                    <li data-leave-item="cutoff"><time>"Sample 2026-05-22"</time><strong>"Sample payroll cutoff"</strong><span>"Example configuration: 3 days before payout; no real schedule or lock"</span><button type="button" data-finance-action="lock-timesheets">"Preview sample cutoff"</button></li>
                </ol>
                <div class="time-heatmap" aria-label="Sample 2026-05-11 through 2026-05-15 utilization">
                    <span role="progressbar" aria-valuenow="58" aria-valuemin="0" aria-valuemax="100" aria-label="Sample Monday utilization: 58%" style="--bar: 58%"><em aria-hidden="true">"Sample Mon"</em></span>
                    <span role="progressbar" aria-valuenow="72" aria-valuemin="0" aria-valuemax="100" aria-label="Sample Tuesday utilization: 72%" style="--bar: 72%"><em aria-hidden="true">"Sample Tue"</em></span>
                    <span role="progressbar" aria-valuenow="91" aria-valuemin="0" aria-valuemax="100" aria-label="Sample Wednesday utilization: 91%" style="--bar: 91%"><em aria-hidden="true">"Sample Wed"</em></span>
                    <span role="progressbar" aria-valuenow="84" aria-valuemin="0" aria-valuemax="100" aria-label="Sample Thursday utilization: 84%" style="--bar: 84%"><em aria-hidden="true">"Sample Thu"</em></span>
                    <span role="progressbar" aria-valuenow="49" aria-valuemin="0" aria-valuemax="100" aria-label="Sample Friday utilization: 49%" style="--bar: 49%"><em aria-hidden="true">"Sample Fri"</em></span>
                </div>
            </div>
            <details id="leave-sample-request" data-leave-detail="request">
                <summary id="leave-sample-request-summary">"김지영 sample request · 2026-05-13 through 2026-05-17"</summary>
                <p>"Requested: 5 sample units. This is not an approved working-day duration. Balance, eligibility, rulepack, and effective date are unavailable."</p>
                <button type="button" disabled aria-describedby="leave-approval-unavailable">"Live leave approval unavailable"</button>
                <p id="leave-approval-unavailable">"Approval requires a live request source, durable state, verified HR approver permission, installed rulepack and effective date, and decision evidence. No approval or payroll effect is issued."</p>
                <button type="button" data-leave-close="request">"Close sample request"</button>
            </details>
            <details id="leave-sample-coverage" data-leave-detail="coverage">
                <summary id="leave-sample-coverage-summary">"Sample coverage · 2026-05-11 through 2026-05-17"</summary>
                <label for="leave-sample-backup">"Sample backup preview"</label>
                <select id="leave-sample-backup" aria-describedby="leave-coverage-preview leave-coverage-unavailable">
                    <option value="none">"Choose a sample backup"</option>
                    <option value="jiyoung">"김지영 · sample backup"</option>
                    <option value="seojun">"박서준 · sample backup"</option>
                </select>
                <p id="leave-coverage-preview" data-leave-projected="coverage" role="status">"No sample backup selected; no coverage assignment made."</p>
                <button type="button" disabled aria-describedby="leave-coverage-unavailable">"Live coverage assignment unavailable"</button>
                <p id="leave-coverage-unavailable">"Assignment requires a live schedule, durable state, verified HR permission, and eligibility evidence. Selecting a sample does not assign anyone or change a schedule."</p>
                <button type="button" data-leave-close="coverage">"Close sample coverage"</button>
            </details>
            <details id="leave-sample-cutoff" data-leave-detail="cutoff">
                <summary id="leave-sample-cutoff-summary">"Sample cutoff · 2026-05-22"</summary>
                <p>"Example configuration: timesheet cutoff 3 days before payout. This sample does not calculate a payout date or a real lock schedule."</p>
                <button type="button" disabled aria-describedby="leave-lock-unavailable">"Live timesheet locking unavailable"</button>
                <p id="leave-lock-unavailable">"Locking requires a live timesheet source, durable state, verified HR permission, and payroll schedule evidence. No timesheet is locked or payroll run changed."</p>
                <button type="button" data-leave-close="cutoff">"Close sample cutoff"</button>
            </details>
            {finance_leave_anchor_board()}
        </article>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leave_preview_current_projection_is_truthful_and_context_bound() {
        let owner = Owner::new();
        owner.with(|| {
            for context in OperatorContext::ALL {
                let envelope = server_derived_envelope(context);
                let body = finance_body_html(&envelope, true);
                assert!(body.contains("Sample leave &amp; time preview"));
                assert!(body.contains(&format!("data-leave-context=\"{}\"", context.id())));
                for marker in [
                    "leave-sample-search",
                    "leave-sample-backup",
                    "leave-approval-unavailable",
                    "leave-coverage-unavailable",
                    "leave-lock-unavailable",
                    "Foundry access does not grant HR",
                    "live HR source and approval readiness unavailable",
                ] {
                    assert!(body.contains(marker), "{context:?}: {marker}");
                }
                assert!(!body.contains("Approve locally") && !body.contains("backup confirmed"));
                assert!(!body.contains("Leave/time ready"));
                let compatibility = finance_body_html(&envelope, false);
                assert!(compatibility.contains("id=\"leave-time\""));
                assert!(compatibility.contains("aria-controls=\"leave-time\""));
                assert_eq!(
                    body.split_once("id=\"finance-panel-leave\"")
                        .unwrap()
                        .1
                        .split_once('>')
                        .unwrap()
                        .0
                        .contains("class=\"finance-panel active\""),
                    context == OperatorContext::HealthcareClinician
                );
            }
        });
        owner.cleanup();
    }
}
