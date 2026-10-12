use super::*;

pub(super) fn business_logic_os_html(native: bool) -> String {
    let rows = BUSINESS_LOGIC_ROWS
        .iter()
        .map(|row| business_logic_row_html(row, native))
        .collect::<String>();
    let filters = [("all", "All"), ("workforce", "Workforce"), ("finance", "Finance"), ("compliance", "Compliance"), ("trust", "Trust"), ("cloud", "Cloud"), ("attention", "Attention")]
        .into_iter().map(|(value, label)| format!(r#"<button type="button" class="{}" data-logic-filter="{value}" data-governance-focus="filter:{value}" aria-pressed="{}">{label}</button>"#, if value == "all" { "active" } else { "" }, value == "all")).collect::<String>();
    format!(
        r##"<section class="business-logic-os" aria-label="Sample business logic catalog">
<div class="logic-os-kpis" aria-label="Derived sample logic summary">
<article class="logic-os-kpi"><span>Sample logics</span><strong>{count}</strong><small>native example rows only</small></article>
<article class="logic-os-kpi"><span>Sample P0</span><strong>{critical}</strong><small>example criticality</small></article>
<article class="logic-os-kpi warn"><span>Sample attention</span><strong>{attention}</strong><small>blocked or at-risk examples</small></article>
<article class="logic-os-kpi accent"><span>Monthly cost</span><strong>Unavailable</strong><small>Cost/run and cadence do not establish a monthly total</small></article>
<article class="logic-os-kpi"><span>Annual cost</span><strong>Unavailable</strong><small>No observed cost or annual total</small></article></div>
<div class="logic-os-toolbar" aria-label="Local sample logic filters"><label class="logic-os-search"><span aria-hidden="true">⌕</span><input data-logic-search="true" data-governance-focus="search" type="search" aria-label="Search sample business logics" placeholder="Search sample logic, owner, category, route…" /></label>
<div class="logic-os-segments" role="group" aria-label="Sample logic category filters">{filters}</div><span class="logic-os-status" data-logic-status="true" role="status">{count} of {count} sample logics shown · all</span></div>
<p>Attention means blocked or at-risk in these examples. The period is May 2026–April 2027; owners, state, cost/run, SLA and tasks are sample values.</p><p data-logic-empty="true" role="status" hidden>No sample logics match this search and filter.</p>
<div class="logic-os-layout"><div class="logic-table-shell" role="region" aria-label="Scrollable sample business logic catalog" tabindex="0" data-governance-scroll="logic" data-governance-focus="scroll:logic"><table class="logic-os-table"><caption class="sr-only">Seven business logic examples; no live execution or verified evidence</caption><thead><tr><th scope="col">Sample health</th><th scope="col">Logic</th><th scope="col">Category</th><th scope="col">Sample owner</th><th scope="col">Cadence</th><th scope="col">Crit.</th><th scope="col">Sample cost/run</th><th scope="col">Sample SLA</th><th scope="col">Sample tasks</th><th scope="col">Sample view</th></tr></thead><tbody>{rows}</tbody></table></div>
<aside class="logic-os-rail" aria-label="Sample dependencies"><div class="logic-rail-card"><p class="screen-anchor">EXAMPLE DEPENDENCY MAP</p><strong>Payroll → Workflow → Mail → Audit sample</strong><div class="logic-dependency-map" aria-hidden="true"><span>HR</span><i></i><span>Payroll</span><i></i><span>Workflow</span><i></i><span>Mail</span><i></i><span>Audit</span></div><nav class="logic-rail-actions" aria-label="Existing sample dependency views">{routes}</nav></div>
<div class="logic-rail-card matrix"><p class="screen-anchor">ILLUSTRATIVE COST × HEALTH</p><p>Live costs and health are unavailable. Sample values are not a measured financial forecast.</p></div></aside></div></section>"##,
        count = BUSINESS_LOGIC_ROWS.len(),
        critical = BUSINESS_LOGIC_ROWS
            .iter()
            .filter(|row| row.criticality == "P0")
            .count(),
        attention = BUSINESS_LOGIC_ROWS
            .iter()
            .filter(|row| logic_needs_attention(row))
            .count(),
        routes = [
            ("workflow", "Workflow"),
            ("mail", "Mail"),
            ("catalog", "Catalog"),
            ("audit", "Audit")
        ]
        .into_iter()
        .map(|(route, label)| governance_route_html(route, label, native))
        .collect::<String>()
    )
}

pub(super) fn logic_needs_attention(row: &BusinessLogicRow) -> bool {
    matches!(row.state, "blocked" | "at-risk")
}

fn business_logic_row_html(row: &BusinessLogicRow, native: bool) -> String {
    let route = if native {
        row.route
    } else {
        match row.id {
            "BL-003" => "#vendors-spend",
            "BL-004" => "#cockpit-panel-policy",
            "BL-005" => "#identity-onboarding",
            _ => row.route,
        }
    };
    let channel = if row.id == "BL-007" {
        " data-governance-channel=\"Community\""
    } else {
        ""
    };
    let record = serde_json::to_string(row).expect("native sample logic is serializable");
    format!(
        r#"<tr class="logic-os-row" data-logic-row="true" data-logic-id="{id}" data-logic-record="{record}" data-logic-category="{category}" data-logic-state="{state}" data-logic-search-text="{search}">
<td><span class="logic-health-dot {state}" aria-label="Sample {state_label}"></span></td><td><details data-logic-detail="{id}"><summary class="logic-name-button" data-governance-focus="detail:{id}">{name}</summary><p>Sample description: {description}</p><dl><div><dt>Sample ID</dt><dd>{id}</dd></div><div><dt>Sample owner</dt><dd>{owner}</dd></div><div><dt>Sample health</dt><dd>{state_label}</dd></div><div><dt>Example cost/run</dt><dd>{cost}</dd></div><div><dt>Example SLA</dt><dd>{sla}</dd></div><div><dt>Live status, execution and verified evidence</dt><dd>Unavailable</dd></div></dl><button type="button" data-logic-close="true" data-governance-focus="close:{id}">Close sample details</button></details><div class="logic-code">{id} · {english}</div></td>
<td><span class="cat-tag">{category}</span></td><td><span class="owner-cell">{owner}</span></td><td class="mono-cell">{cadence}</td><td><span class="crit crit-{criticality}">{criticality}</span></td><td class="cost-cell">{cost}</td><td><span class="{sla_class}">{sla}</span></td><td><span class="tasks-cell">{tasks}</span></td><td><span class="logic-row-actions"><a href="{route}" data-governance-route="logic" data-logic-action="open" data-logic-target="{route}"{channel}>Open sample view</a><button type="button" data-logic-action="run" data-governance-unavailable="true" disabled aria-describedby="governance-live-unavailable">Run unavailable</button></span></td></tr>"#,
        id = daily_escape(row.id),
        record = daily_escape(&record),
        category = daily_escape(row.category),
        state = daily_escape(row.state),
        search = daily_escape(&format!(
            "{} {} {} {} {} {} {} {}",
            row.id,
            row.name,
            row.english_name,
            row.category,
            row.owner,
            row.description,
            row.route,
            row.state_label
        )),
        state_label = daily_escape(row.state_label),
        name = daily_escape(row.name),
        description = daily_escape(row.description),
        owner = daily_escape(row.owner),
        english = daily_escape(row.english_name),
        cadence = daily_escape(row.cadence),
        criticality = daily_escape(row.criticality),
        cost = daily_escape(row.cost),
        sla = daily_escape(row.sla),
        tasks = daily_escape(row.tasks),
        sla_class = logic_sla_class(row.state)
    )
}

pub(super) fn logic_sla_class(state: &str) -> &'static str {
    match state {
        "blocked" | "at-risk" => "sla-cell over",
        "attention" | "review" => "sla-cell near",
        _ => "sla-cell ok",
    }
}
