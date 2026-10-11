use super::*;

pub(super) fn business_logic_os_panel() -> impl IntoView {
    view! {
        <section class="business-logic-os" aria-label="Business logic operating system">
            <div class="logic-os-kpis" aria-label="Business logic summary">
                <article class="logic-os-kpi"><span>"Active logics"</span><strong>"17"</strong><small>"7 visible in this envelope"</small></article>
                <article class="logic-os-kpi"><span>"P0 critical"</span><strong>"4"</strong><small>"cannot fail silently"</small></article>
                <article class="logic-os-kpi warn"><span>"Need attention"</span><strong>"3"</strong><small>"blocked or at-risk"</small></article>
                <article class="logic-os-kpi accent"><span>"Real cost / month"</span><strong>"₩9.2M"</strong><small>"hard + soft + delay"</small></article>
                <article class="logic-os-kpi"><span>"Annualized"</span><strong>"₩110M"</strong><small>"if cadence holds"</small></article>
            </div>

            <div class="logic-os-toolbar" aria-label="Business logic filters">
                <label class="logic-os-search">
                    <span aria-hidden="true">"⌕"</span>
                    <input data-logic-search="true" type="search" aria-label="Search business logics" placeholder="Search logic, owner, route, evidence..." />
                </label>
                <div class="logic-os-segments" role="toolbar" aria-label="Logic category filters">
                    <button type="button" class="active" data-logic-filter="all">"All"</button>
                    <button type="button" data-logic-filter="workforce">"Workforce"</button>
                    <button type="button" data-logic-filter="finance">"Finance"</button>
                    <button type="button" data-logic-filter="compliance">"Compliance"</button>
                    <button type="button" data-logic-filter="trust">"Trust"</button>
                    <button type="button" data-logic-filter="cloud">"Cloud"</button>
                    <button type="button" data-logic-filter="attention">"Attention"</button>
                </div>
                <span class="logic-os-status" data-logic-status="true">
                    <strong data-logic-visible-count="true">{BUSINESS_LOGIC_ROWS.len()}</strong>
                    " visible · all categories · local only"
                </span>
            </div>

            <div class="logic-os-layout">
                <div class="logic-table-shell" role="region" aria-label="Business logic catalog">
                    <table class="logic-os-table">
                        <thead>
                            <tr>
                                <th>"Health"</th>
                                <th>"Logic"</th>
                                <th>"Category"</th>
                                <th>"Owner"</th>
                                <th>"Cadence"</th>
                                <th>"Crit."</th>
                                <th>"Cost/run"</th>
                                <th>"SLA"</th>
                                <th>"Tasks"</th>
                                <th>"Action"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {BUSINESS_LOGIC_ROWS.into_iter().map(business_logic_row_view).collect_view()}
                        </tbody>
                    </table>
                </div>

                <aside class="logic-os-rail" aria-label="Business logic dependency and evidence rail">
                    <div class="logic-rail-card">
                        <p class="screen-anchor">"DEPENDENCY MAP"</p>
                        <strong>"Payroll anomaly → Workflow → Messenger/Mail → Audit"</strong>
                        <div class="logic-dependency-map" aria-hidden="true">
                            <span>"HR"</span><i></i><span>"Payroll"</span><i></i><span>"Workflow"</span><i></i><span>"Mail"</span><i></i><span>"Audit"</span>
                        </div>
                        <div class="logic-rail-actions">
                            <button type="button" data-logic-graph-action="workflow">"Workflow"</button>
                            <button type="button" data-logic-graph-action="mail">"Mail brief"</button>
                            <button type="button" data-logic-graph-action="catalog">"Catalog"</button>
                            <button type="button" data-logic-graph-action="audit">"Evidence"</button>
                        </div>
                    </div>
                    <div class="logic-rail-card matrix">
                        <p class="screen-anchor">"COST × HEALTH"</p>
                        <div class="logic-matrix" aria-label="Cost by health preview">
                            <span class="dot danger" style="--x: 78%; --y: 20%" title="Tenant network split"></span>
                            <span class="dot warn" style="--x: 54%; --y: 38%" title="Payroll close"></span>
                            <span class="dot warn" style="--x: 42%; --y: 52%" title="Vendor renewal"></span>
                            <span class="dot ok" style="--x: 24%; --y: 70%" title="New hire onboarding"></span>
                            <span class="dot done" style="--x: 12%; --y: 84%" title="Governance council note"></span>
                        </div>
                    </div>
                </aside>
            </div>
        </section>
    }
}

pub(super) fn business_logic_row_view(row: BusinessLogicRow) -> impl IntoView {
    let state_class = format!("logic-health-dot {}", row.state);
    let criticality_class = format!("crit crit-{}", row.criticality);
    let task_class = if row.tasks == "0" {
        "tasks-cell"
    } else {
        "tasks-cell has-open"
    };
    view! {
        <tr
            class="logic-os-row"
            data-logic-row="true"
            data-logic-category=row.category
            data-logic-state=row.state
        >
            <td><span class=state_class aria-label=row.state_label></span></td>
            <td>
                <button
                    type="button"
                    class="logic-name-button"
                    data-sidepeek-trigger="business-logic"
                    data-sidepeek-title=row.name
                    data-sidepeek-id=row.id
                    data-sidepeek-desc=row.description
                    data-sidepeek-owner=row.owner
                    data-sidepeek-risk=row.state_label
                    data-sidepeek-sla=row.sla
                >
                    {row.name}
                </button>
                <div class="logic-code">{row.id}" · "{row.english_name}</div>
            </td>
            <td><span class="cat-tag">{row.category}</span></td>
            <td><span class="owner-cell"><span class="avatar-xs" aria-hidden="true">{logic_owner_initials(row.owner)}</span>{row.owner}</span></td>
            <td class="mono-cell">{row.cadence}</td>
            <td><span class=criticality_class>{row.criticality}</span></td>
            <td class="cost-cell">{row.cost}</td>
            <td><span class={logic_sla_class(row.state)}>{row.sla}</span></td>
            <td><span class=task_class>{row.tasks}</span></td>
            <td>
                <span class="logic-row-actions">
                    <button type="button" data-logic-action="open" data-logic-target=row.route>"Open"</button>
                    <button type="button" data-logic-action="run">"Run preview"</button>
                </span>
            </td>
        </tr>
    }
}

pub(super) fn logic_owner_initials(owner: &str) -> &'static str {
    match owner {
        "Finance + HR" => "FH",
        "Tax operations" => "TO",
        "CFO office" => "CF",
        "Security reviewer" => "SR",
        "People ops" => "PO",
        "Infrastructure ops" => "PL",
        "Governance" => "GV",
        _ => "LO",
    }
}

pub(super) fn logic_sla_class(state: &str) -> &'static str {
    match state {
        "blocked" | "at-risk" => "sla-cell over",
        "attention" | "review" => "sla-cell near",
        _ => "sla-cell ok",
    }
}
