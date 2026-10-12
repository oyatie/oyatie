use super::*;

pub(super) fn governance_route_target(route: &str, native: bool) -> &'static str {
    match route {
        "workflow" => "#workflow-studio",
        "mail" | "community" => "#work-hub",
        "finance" => {
            if native {
                "#finance-panel-ledger"
            } else {
                "#ledger-preview"
            }
        }
        "identity" => {
            if native {
                "#identity-panel-auth"
            } else {
                "#identity-workforce-service"
            }
        }
        "cloud" => "#cloud-ops-cockpit",
        "evidence" => "#evidence-spine",
        "audit" => {
            if native {
                "#resource-panel-audit"
            } else {
                "#audit-ledger"
            }
        }
        "catalog" => "#service-catalog",
        "inbox" => "#command-center-workbench",
        _ => "#governance-analytics",
    }
}

pub(super) fn governance_route_html(route: &str, label: &str, native: bool) -> String {
    let channel = if matches!(route, "mail" | "community") {
        format!(
            " data-governance-channel=\"{}\"",
            if route == "mail" { "Mail" } else { "Community" }
        )
    } else {
        String::new()
    };
    format!(
        r#"<a href="{}" data-governance-route="{route}"{channel}>Open {} sample</a>"#,
        governance_route_target(route, native),
        daily_escape(label)
    )
}

pub(super) fn governance_command_html(workflow_name: &str, native: bool) -> String {
    format!(
        r#"<section id="governance-analytics" class="governance-ops-command" aria-labelledby="governance-ops-title"><div class="governance-command-head"><div><p class="screen-anchor">ANONYMOUS GOVERNANCE SAMPLE</p><h4 id="governance-ops-title">Sample policy, evidence, risk and logic</h4><p>Illustrative executive board for the Tenant admin and Corporate office sample contexts. Example period: May 2026–April 2027. No tenant or role is verified; live metrics, approvals, attestations and durable receipts are unavailable.</p></div><p id="governance-live-unavailable">Live review, execution, attestation, sealing and receipt creation are unavailable. Sample selection grants no permission.</p><div class="governance-command-actions"><span class="status-chip warning" data-governance-status="true">5 risk examples · live posture unavailable</span><button type="button" data-governance-action="run-review" data-governance-unavailable="true" disabled aria-describedby="governance-live-unavailable">Run review unavailable</button><button type="button" data-governance-action="seal-brief" data-governance-unavailable="true" disabled aria-describedby="governance-live-unavailable">Seal brief unavailable</button><a href="{route_inbox}" data-governance-action="route-inbox" data-governance-route="inbox">Open Inbox sample</a></div></div><div class="governance-posture-strip" aria-label="Illustrative sample posture; no live measurements">
<article class="gov-posture-score"><span>Illustrative posture</span><strong>84</strong><em>Example score only · no observed comparison or target</em></article>

<article class="gov-posture-trend"><div><span>Illustrative trend</span><strong>78 → 84</strong></div><svg viewBox="0 0 160 42" aria-hidden="true" class="gov-sparkline"><polyline points="0,34 14,32 28,32 42,29 56,29 70,27 84,29 98,26 112,23 126,23 140,18 154,16" /></svg></article>
<div class="gov-pillar-grid" aria-label="Sample pillar illustrations and destinations"><a href="{route_finance}" data-governance-action="pillar-compliance" data-governance-route="finance"><span>Compliance</span><strong>87</strong><em>example controls only</em><small>Open sample view</small></a><a href="{route_finance}" data-governance-action="pillar-financial" data-governance-route="finance"><span>Financial controls</span><strong>91</strong><em>illustrative score only</em><small>Open sample view</small></a><a href="{route_identity}" data-governance-action="pillar-workforce" data-governance-route="identity"><span>Workforce risk</span><strong>71</strong><em>sample §53 scenario</em><small>Open sample view</small></a><a href="{route_community}" data-governance-action="pillar-disclosure" data-governance-route="community" data-governance-channel="Community"><span>Board + disclosure</span><strong>88</strong><em>example May 12 board date</em><small>Open sample view</small></a></div></div><div class="governance-command-grid">
<article class="governance-command-card policy-gate-card"><div class="governance-card-head"><div><p class="screen-anchor">EXAMPLE POLICY GATES</p><h5>Illustrative review checkpoints</h5></div><span class="status-chip danger">No live gate verified</span></div><div class="policy-gate-list" aria-label="Governance policy gates"><a href="{route_workflow}" data-governance-action="select-payroll" data-governance-route="workflow"><span>P0</span><strong>Payroll close</strong><em>example two-person review</em><small>Open sample view</small></a><a href="{route_finance}" data-governance-action="select-hometax" data-governance-route="finance"><span>P0</span><strong>HomeTax filing</strong><em>example registration review</em><small>Open sample view</small></a><a href="{route_cloud}" data-governance-action="select-cloud" data-governance-route="cloud"><span>P0</span><strong>Network split</strong><em>example rollback review</em><small>Open sample view</small></a><a href="{route_identity}" data-governance-action="select-pipa" data-governance-route="identity"><span>P1</span><strong>PIPA boundary</strong><em>example PIPA boundary</em><small>Open sample view</small></a></div></article>

<article class="governance-command-card decision-queue-card"><div class="governance-card-head"><div><p class="screen-anchor">SAMPLE DECISION QUEUE</p><h5>Illustrative owners, SLA and route</h5></div><a href="{route_inbox}" data-governance-action="open-inbox" data-governance-route="inbox">Open Inbox sample</a></div><table class="governance-decision-table"><thead><tr><th scope="col">Sample decision</th><th scope="col">Sample owner</th><th scope="col">Example SLA</th><th scope="col">Sample state</th></tr></thead><tbody><tr><td><strong>Park 4대보험 delta</strong><small>REC-PAY-2026-04-PARK</small></td><td>CFO</td><td>4.0h</td><td><span class="status-chip danger">blocking</span></td></tr><tr><td><strong>Stripe approval compression</strong><small>policy ≤ ₩5M route</small></td><td>AP</td><td>1d</td><td><span class="status-chip warning">review</span></td></tr><tr><td><strong>Governance council note</strong><small>Community + Mail packet</small></td><td>Gov</td><td>today</td><td><span class="status-chip success">ready</span></td></tr><tr><td><strong>Board option pool</strong><small>resolution pack May 12</small></td><td>CEO</td><td>6d</td><td><span class="status-chip">scheduled</span></td></tr></tbody></table></article>

<article id="governance-risk-matrix" class="governance-command-card risk-matrix-card"><div class="governance-card-head"><div><p class="screen-anchor">SAMPLE RISK MATRIX</p><h5>Five illustrative risks</h5></div><span class="status-chip ai">interactive</span></div>{risk_workspace}</article>

<article class="governance-command-card compliance-calendar-card"><div class="governance-card-head"><div><p class="screen-anchor">EXAMPLE COMPLIANCE CALENDAR</p><h5>May 2026–April 2027 examples</h5></div><a href="{route_finance}" data-governance-action="calendar-review" data-governance-route="finance">Open Finance sample</a></div>{calendar}</article>

<article class="governance-command-card evidence-readiness-card"><div class="governance-card-head"><div><p class="screen-anchor">EXAMPLE EVIDENCE LANES</p><h5>Live readiness and receipts unavailable</h5></div><span class="status-chip success">unverified examples</span></div><div class="evidence-readiness-lanes" aria-label="Evidence readiness lanes"><span role="progressbar" aria-valuenow="92" aria-valuemin="0" aria-valuemax="100" aria-label="Illustrative workflow receipts: 92%; live receipts unavailable" style="--bar: 92%"><strong aria-hidden="true">Workflow receipts</strong><em aria-hidden="true">example 11 / 12</em></span><span role="progressbar" aria-valuenow="78" aria-valuemin="0" aria-valuemax="100" aria-label="Illustrative Mail approvals: 78%; live approvals unavailable" style="--bar: 78%"><strong aria-hidden="true">Mail approvals</strong><em aria-hidden="true">example 7 linked</em></span><span role="progressbar" aria-valuenow="64" aria-valuemin="0" aria-valuemax="100" aria-label="Illustrative cloud runbooks: 64%; live readiness unavailable" style="--bar: 64%"><strong aria-hidden="true">Cloud runbooks</strong><em aria-hidden="true">example 2 waiting</em></span><span role="progressbar" aria-valuenow="86" aria-valuemin="0" aria-valuemax="100" aria-label="Illustrative PIPA audit: 86%; live audit unavailable" style="--bar: 86%"><strong aria-hidden="true">PIPA audit</strong><em aria-hidden="true">example vendor gate</em></span></div></article>

<article class="governance-command-card graph-route-card"><div class="governance-card-head"><div><p class="screen-anchor">ROUTE MATRIX</p><h5>Open existing sample views</h5></div></div><div class="governance-route-grid" aria-label="Governance route matrix"><a href="{route_workflow}" data-governance-action="route-workflow" data-governance-route="workflow">Open Workflow sample</a><a href="{route_mail}" data-governance-action="route-mail" data-governance-route="mail" data-governance-channel="Mail">Open Mail sample</a><a href="{route_community}" data-governance-action="route-community" data-governance-route="community" data-governance-channel="Community">Open Community sample</a><a href="{route_finance}" data-governance-action="route-finance" data-governance-route="finance">Open Finance sample</a><a href="{route_cloud}" data-governance-action="route-cloud" data-governance-route="cloud">Open Cloud Ops sample</a><a href="{route_identity}" data-governance-action="route-identity" data-governance-route="identity">Open Identity sample</a><a href="{route_evidence}" data-governance-action="route-evidence" data-governance-route="evidence">Open Evidence sample</a><a href="{route_catalog}" data-governance-action="route-catalog" data-governance-route="catalog">Open Catalog sample</a></div><dl class="governance-kv"><div><dt>Sample workflow name</dt><dd>{workflow_name}</dd></div><div><dt>Autonomy ceiling</dt><dd>Live authorization and approval unavailable</dd></div></dl></article>
</div>
<div class="governance-lower-deck">
<article class="governance-command-card control-attestation-card"><div class="governance-card-head"><div><p class="screen-anchor">EXAMPLE CONTROL ATTESTATION</p><h5>Live attestation unavailable</h5></div><span class="status-chip warning">3 control examples</span></div><div class="gov-attestation-list"><button type="button" data-governance-action="attest-payroll" data-governance-unavailable="true" disabled aria-describedby="governance-live-unavailable"><strong>CTRL-PAY-09 · payroll 4-eyes</strong><span style="--bar: 82%"></span><em>example 82% · not attested</em><small>Attest unavailable</small></button><button type="button" data-governance-action="attest-pipa" data-governance-unavailable="true" disabled aria-describedby="governance-live-unavailable"><strong>CTRL-PIPA-03 · retention boundary</strong><span style="--bar: 68%"></span><em>example 68% · not attested</em><small>Attest unavailable</small></button><button type="button" data-governance-action="attest-cloud" data-governance-unavailable="true" disabled aria-describedby="governance-live-unavailable"><strong>CTRL-CLOUD-12 · rollback runbook</strong><span style="--bar: 74%"></span><em>example 74% · freshness unavailable</em><small>Attest unavailable</small></button></div></article>

<article class="governance-command-card audit-chain-card"><div class="governance-card-head"><div><p class="screen-anchor">ILLUSTRATIVE AUDIT CHAIN</p><h5>No verified or durable receipt</h5></div><a href="{route_evidence}" data-governance-action="route-evidence" data-governance-route="evidence">Open Evidence sample</a></div><ol class="gov-audit-chain"><li><span>01</span><strong>Workflow run</strong><em>example reference only</em></li><li><span>02</span><strong>Mail approval</strong><em>example reference only</em></li><li><span>03</span><strong>Community note</strong><em>example reference only</em></li><li><span>04</span><strong>Board packet</strong><em>draft</em></li></ol></article>

<article class="governance-command-card board-cycle-card"><div class="governance-card-head"><div><p class="screen-anchor">EXAMPLE BOARD CYCLE</p><h5>Illustrative dates; no acknowledgement verified</h5></div><span class="status-chip success">Acknowledgements unavailable</span></div><div class="gov-board-timeline" aria-label="Illustrative May 2026 board timeline"><span class="done"><strong>Draft</strong><em>May 04</em></span><span class="active"><strong>Review</strong><em>May 07</em></span><span><strong>Send</strong><em>May 10</em></span><span><strong>Vote</strong><em>May 12</em></span></div></article>
</div></section>"#,
        workflow_name = daily_escape(workflow_name),
        risk_workspace = governance_risk_html(),
        calendar = governance_calendar_html(),
        route_catalog = governance_route_target("catalog", native),
        route_cloud = governance_route_target("cloud", native),
        route_community = governance_route_target("community", native),
        route_evidence = governance_route_target("evidence", native),
        route_finance = governance_route_target("finance", native),
        route_identity = governance_route_target("identity", native),
        route_inbox = governance_route_target("inbox", native),
        route_mail = governance_route_target("mail", native),
        route_workflow = governance_route_target("workflow", native),
    )
}

const RISK_SAMPLES: [(&str, &str, &str, &str, &str, &str, &str); 5] = [
    (
        "RISK-042",
        "AI",
        "--x: 78%; --y: 42%",
        "AI agent governance",
        "Auto-delegation rollout needs change-management, rollback evidence, and human signoff.",
        "EMP-104",
        "4×3 High",
    ),
    (
        "RISK-014",
        "53",
        "--x: 58%; --y: 30%",
        "LSA §53 weekly-hour breach",
        "Yoon Tae-min projected 49.5h; automatic reassignment lowers residual risk.",
        "EMP-211",
        "3×4 Moderate",
    ),
    (
        "RISK-009",
        "PI",
        "--x: 40%; --y: 34%",
        "PIPA retention overrun",
        "43 medical certificates expire in 14 days; consent renewal or purge decision required.",
        "EMP-274",
        "2×4 Moderate",
    ),
    (
        "RISK-031",
        "JE",
        "--x: 57%; --y: 54%",
        "JE four-eyes gap",
        "7 manual journal entries posted with a single approver; enforcement pending.",
        "EMP-188",
        "3×3 Moderate",
    ),
    (
        "RISK-018",
        "BD",
        "--x: 21%; --y: 20%",
        "Missed board resolution",
        "Option pool expansion requires May 12 board resolution and quorum confirmation.",
        "EMP-274",
        "1×5 Low",
    ),
];

fn governance_risk_html() -> String {
    let pins=RISK_SAMPLES.into_iter().enumerate().map(|(index,(id,label,position,title,detail,owner,score))| format!(r#"<button type="button" class="gov-risk-pin {}" style="{position}" data-governance-risk="{id}" data-risk-title="{title}" data-risk-detail="{detail}" data-risk-owner="{owner}" data-risk-score="{score}" data-governance-focus="risk:{id}" aria-pressed="{}" aria-label="Inspect sample {title}, {score}, sample owner {owner}">{label}</button>"#, if index==0 { "selected" } else { "" }, index==0)).collect::<String>();
    let (id, _, _, title, detail, owner, score) = RISK_SAMPLES[0];
    format!(
        r#"<div class="gov-risk-workspace"><div class="gov-risk-heatmap" role="group" aria-label="Five selectable sample risks"><span class="tone-minimal"></span><span class="tone-minimal"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-minimal"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-extreme"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-extreme"></span><span class="tone-extreme"></span>{pins}</div><aside class="gov-risk-peek" aria-live="polite"><div><span data-risk-peek-id="true">{id}</span><strong data-risk-peek-score="true">{score}</strong></div><h6 data-risk-peek-title="true">{title}</h6><p data-risk-peek-detail="true">{detail}</p><dl><div><dt>Sample owner</dt><dd data-risk-peek-owner="true">{owner}</dd></div><div><dt>Live score, ownership and review date</dt><dd>Unavailable</dd></div></dl><p data-governance-risk-status="true">Sample risk selected; no live score or owner verified.</p></aside></div>"#
    )
}

const CALENDAR_MONTHS: [&str; 12] = [
    "May 2026",
    "June 2026",
    "July 2026",
    "August 2026",
    "September 2026",
    "October 2026",
    "November 2026",
    "December 2026",
    "January 2027",
    "February 2027",
    "March 2027",
    "April 2027",
];
const CALENDAR_SAMPLES: [(&str, [(&str, &str); 12]); 3] = [
    (
        "Withholding",
        [
            ("10 ₩38.2M", "ready"),
            ("10 ₩39.1M", "pending"),
            ("10 ₩40.0M", "pending"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ),
    (
        "Social insurance",
        [
            ("10 ₩57.0M", "review"),
            ("10 ₩58.2M", "pending"),
            ("10 ₩59.1M", "pending"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ),
    (
        "VAT",
        [
            ("", ""),
            ("", ""),
            ("25 Q2 prelim", "pending"),
            ("", ""),
            ("", ""),
            ("25 Q3 final", "pending"),
            ("", ""),
            ("", ""),
            ("25 Q4 prelim", "pending"),
            ("", ""),
            ("", ""),
            ("25 Q1 final", "pending"),
        ],
    ),
];

fn governance_calendar_html() -> String {
    let months = CALENDAR_MONTHS
        .into_iter()
        .enumerate()
        .map(|(index, month)| format!(r#"<th scope="col" id="gov-month-{index}">{month}</th>"#))
        .collect::<String>();
    let rows=CALENDAR_SAMPLES.into_iter().enumerate().map(|(family_index,(family,cells))| {
        let body=cells.into_iter().enumerate().map(|(index,(label,state))| {
            let headers=format!("gov-family-{family_index} gov-month-{index}");
            if label.is_empty() { format!(r#"<td class="empty" headers="{headers}"><span aria-label="No sample obligation">—</span></td>"#) }
            else { let month=CALENDAR_MONTHS[index];let id=format!("{family_index}:{index}");format!(r#"<td headers="{headers}"><button type="button" class="{state}" data-gov-calendar-cell="{id}" data-gov-calendar-description="Sample {family}, {month}: {label}, {state} example. No filing or readiness verified." data-governance-focus="calendar:{id}" aria-pressed="false" aria-label="Inspect sample {family}, {month}, {label}, {state} example">{label}</button></td>"#) }
        }).collect::<String>();
        format!(r#"<tr><th scope="row" id="gov-family-{family_index}">{family}</th>{body}</tr>"#)
    }).collect::<String>();
    format!(
        r#"<div class="gov-calendar" role="region" aria-label="Scrollable example compliance calendar, May 2026 to April 2027" tabindex="0" data-governance-scroll="calendar" data-governance-focus="scroll:calendar"><table class="gov-calendar-table"><caption>May 2026–April 2027 examples; readiness, dates and amounts are unverified</caption><thead><tr><th scope="col">Family</th>{months}</tr></thead><tbody>{rows}</tbody></table></div><p data-governance-calendar-status="true" role="status">Select a populated sample obligation to inspect it. Empty cells have no sample obligation.</p>"#
    )
}
