use super::*;

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_daily_execution_proof_board() -> &'static str {
    r#"<section class="daily-proof-board" aria-label="FD-001 daily execution and Oyatie Cloud tenant proof"><div class="daily-proof-grid"><article class="daily-proof-card selected" data-daily-proof-card="daily-fd001"><p class="screen-anchor">FD-001 DAILY WORKLOAD</p><h5>Today’s work proves product delivery</h5><p>Tasks, approvals, schedule holds, Workflow routes, Mail, Messenger, Community, and evidence receipts are FD-001 tenant workload operations, not detached widgets.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="stage-packet">Stage work packet</button><button type="button" data-daily-proof-action="route-workflow">Workflow run</button></div></article><article class="daily-proof-card" data-daily-proof-card="daily-cloud"><p class="screen-anchor">OYATIE CLOUD SUBSTRATE</p><h5>The queue dogfoods tenant hosting posture</h5><p>Oyatie Cloud proves the substrate can host real production tenants by tying daily FD-001 work to cell health, policy envelopes, FinOps, and audit freshness.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="route-cloud">Cloud cells</button><button type="button" data-daily-proof-action="route-policy">Policy envelope</button></div></article><article class="daily-proof-card" data-daily-proof-card="daily-local"><p class="screen-anchor">LOCAL-ONLY COMMAND RAIL</p><h5>Visually functional without side effects</h5><p>Operators can filter, stage, route, brief, and inspect work while backend writes, auth changes, workflow execution, mail sends, payroll, billing, and cloud mutations remain disconnected.</p><div class="daily-proof-actions"><button type="button" data-daily-proof-action="route-audit">Audit ledger</button><button type="button" data-daily-proof-action="route-mail">Mail brief</button></div></article></div><div class="daily-proof-footer"><span data-daily-proof-status="true">Daily execution ready · FD-001 work queue dogfoods Oyatie Cloud locally.</span><div class="daily-proof-routes" aria-label="Daily execution connected routes"><button type="button" data-daily-proof-action="route-inbox">Action Inbox</button><button type="button" data-daily-proof-action="route-schedule">Schedule</button><button type="button" data-daily-proof-action="route-community">Community</button><button type="button" data-daily-proof-action="route-evidence">Evidence</button></div></div></section>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_daily_queue_footer() -> &'static str {
    r#"<div class="daily-subroute-proof"><span data-daily-proof-status="true">Execution queue ready · FD-001 work items dogfood Oyatie Cloud with no backend mutation.</span><div class="daily-proof-routes" aria-label="Execution queue connected routes"><button type="button" data-daily-proof-action="route-inbox">Action Inbox</button><button type="button" data-daily-proof-action="route-workflow">Workflow</button><button type="button" data-daily-proof-action="route-evidence">Evidence</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_daily_schedule_footer() -> &'static str {
    r#"<div class="daily-subroute-proof"><span data-daily-proof-status="true">Schedule pressure ready · FD-001 calendar risk stays tenant-scoped on Oyatie Cloud with no calendar, workflow, mail, policy, or cloud mutation.</span><div class="daily-proof-routes" aria-label="Schedule connected routes"><button type="button" data-daily-proof-action="route-policy">Policy</button><button type="button" data-daily-proof-action="route-cloud">Cloud cells</button><button type="button" data-daily-proof-action="route-mail">Reviewer Mail</button></div></div>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_daily_execution_console(envelope: &TenantRenderEnvelope) -> String {
    let rows = daily_execution_rows(envelope);
    let blocking_count = rows.iter().filter(|row| row.state == "blocking").count();
    let task_count = rows.iter().filter(|row| row.kind == "task").count();
    let approval_count = rows.iter().filter(|row| row.kind == "approval").count();
    let schedule_count = rows.iter().filter(|row| row.kind == "schedule").count();
    let evidence_count = rows.iter().filter(|row| row.kind == "evidence").count();
    let row_markup = rows.iter().map(static_daily_row).collect::<String>();
    let schedule_markup = envelope
        .schedule
        .iter()
        .map(|item| {
            format!(
                r#"<li><time>{time}</time><strong>{title}</strong><p>{detail}</p></li>"#,
                time = escape(&item.time),
                title = escape(&item.title),
                detail = escape(&item.detail)
            )
        })
        .collect::<String>();

    format!(
        r##"<section id="daily-execution" class="daily-execution-console panel" aria-labelledby="daily-execution-title"><div class="daily-execution-head"><div><p class="screen-anchor">DAILY WORK · PERSONAL OPERATIONS</p><h3 id="daily-execution-title">Tasks, approvals, schedule, and evidence for today</h3><p>A single operator queue connects calendar pressure, approval risk, workflow routes, Mail/Messenger drafts, and receipt evidence.</p></div><span class="status-chip success">local command surface</span></div><div class="daily-execution-kpis" aria-label="Daily execution summary"><span><strong>{task_count}</strong><small>tasks</small></span><span><strong>{approval_count}</strong><small>approvals</small></span><span><strong>{schedule_count}</strong><small>calendar holds</small></span><span><strong>{blocking_count}</strong><small>blocking</small></span><span><strong>{evidence_count}</strong><small>evidence links</small></span></div><div class="daily-execution-toolbar" aria-label="Daily execution filters"><label><span aria-hidden="true">⌕</span><input data-daily-search="true" aria-label="Search daily work" placeholder="Search tasks, approvals, owners, receipts..." /></label><div class="daily-filter-pills" role="toolbar" aria-label="Daily work filters"><button type="button" class="active" data-daily-filter="all">All</button><button type="button" data-daily-filter="blocking">Blocking</button><button type="button" data-daily-filter="task">Tasks</button><button type="button" data-daily-filter="approval">Approvals</button><button type="button" data-daily-filter="schedule">Schedule</button><button type="button" data-daily-filter="evidence">Evidence</button></div><span data-daily-status="true">{row_count} visible · all work · local only</span></div>{daily_execution_proof_board}<div class="daily-execution-layout"><article id="tasks-title" class="daily-execution-list" aria-labelledby="daily-list-title"><div class="daily-column-head"><p class="screen-anchor">EXECUTION QUEUE</p><h4 id="daily-list-title">One list for personal operations</h4></div><div role="list" aria-label="Daily work rows">{row_markup}</div>{daily_queue_footer}</article><aside id="schedule-title" class="daily-calendar-rail" aria-label="Today schedule and capacity"><div class="daily-column-head"><p class="screen-anchor">CALENDAR</p><h4>Today’s schedule pressure</h4></div><ol class="daily-timeline">{schedule_markup}</ol><div class="daily-capacity" aria-label="Daily capacity"><span role="progressbar" aria-valuenow="73" aria-valuemin="0" aria-valuemax="100" aria-label="Close work: 73%" style="--bar: 73%"><em aria-hidden="true">Close work · 73%</em></span><span role="progressbar" aria-valuenow="64" aria-valuemin="0" aria-valuemax="100" aria-label="Approvals: 64%" style="--bar: 64%"><em aria-hidden="true">Approvals · 64%</em></span><span role="progressbar" aria-valuenow="41" aria-valuemin="0" aria-valuemax="100" aria-label="Context switching: 41%" style="--bar: 41%"><em aria-hidden="true">Context switching · 41%</em></span></div><div class="daily-route-matrix" aria-label="Daily route matrix"><button type="button" data-daily-action="workflow" data-daily-target="#workflow-studio">Workflow</button><button type="button" data-daily-action="mail">Mail brief</button><button type="button" data-daily-action="messenger">Messenger</button><button type="button" data-daily-action="evidence">Audit evidence</button></div>{daily_schedule_footer}</aside></div></section>"##,
        task_count = task_count,
        approval_count = approval_count,
        schedule_count = schedule_count,
        blocking_count = blocking_count,
        evidence_count = evidence_count,
        row_count = rows.len(),
        row_markup = row_markup,
        schedule_markup = schedule_markup,
        daily_execution_proof_board = static_daily_execution_proof_board(),
        daily_queue_footer = static_daily_queue_footer(),
        daily_schedule_footer = static_daily_schedule_footer()
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_daily_row(row: &ExecutionRow) -> String {
    format!(
        r##"<article class="daily-row" data-daily-row="true" data-daily-kind="{kind}" data-daily-state="{state}" role="listitem"><button type="button" class="daily-row-main" data-sidepeek-trigger="daily-work" data-sidepeek-title="{title}" data-sidepeek-id="{id}" data-sidepeek-desc="{body}" data-sidepeek-owner="{owner}" data-sidepeek-risk="{state}" data-sidepeek-sla="{due}"><span class="{chip}">{state}</span><strong>{title}</strong><p>{body}</p></button><dl><div><dt>Kind</dt><dd>{kind}</dd></div><div><dt>Owner</dt><dd>{owner}</dd></div><div><dt>Due</dt><dd>{due}</dd></div></dl><div class="daily-row-actions"><button type="button" data-daily-action="workflow" data-daily-target="{route}">Flow</button><button type="button" data-daily-action="mail">Mail</button><button type="button" data-daily-action="evidence">Evidence</button><button type="button" data-daily-action="stage">Stage</button></div></article>"##,
        kind = escape(row.kind),
        state = escape(row.state),
        title = escape(&row.title),
        id = escape(&row.id),
        body = escape(&row.body),
        owner = escape(&row.owner),
        due = escape(&row.due),
        chip = daily_status_class(row.state),
        route = escape(row.route)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_comms_receipt_bridge() -> &'static str {
    r#"<section class="comms-receipt-bridge" data-comms-receipt-bridge="true" aria-label="Messenger Mail Community receipt bridge"><div class="comms-bridge-head"><div><p class="screen-anchor">COMMS RECEIPT BRIDGE</p><h4>Messenger, Mail, and Community return to one proof packet</h4><span data-comms-bridge-status="true">Ops room, approval brief, council post, and audit receipt are staged as one local FD-001 workload packet.</span></div><button type="button" data-comms-bridge-action="seal">Seal handoff</button></div><div class="comms-bridge-routes" aria-label="Communication proof routes"><button type="button" class="selected" data-comms-bridge-route="messenger" data-comms-bridge-title="Messenger ops room" data-comms-bridge-receipt="REC-COMMS-MSG-021" data-comms-bridge-target="Ops room → Mail brief → Community note"><span>01 · Messenger</span><strong>Ops room thread</strong><em>REC-COMMS-MSG-021</em></button><button type="button" data-comms-bridge-route="mail" data-comms-bridge-title="Mail approval brief" data-comms-bridge-receipt="REC-COMMS-MAIL-022" data-comms-bridge-target="Formal approval → Evidence packet"><span>02 · Mail</span><strong>Approval brief</strong><em>REC-COMMS-MAIL-022</em></button><button type="button" data-comms-bridge-route="community" data-comms-bridge-title="Community council note" data-comms-bridge-receipt="REC-COMMS-COMM-023" data-comms-bridge-target="Council post → Role-visible vote"><span>03 · Community</span><strong>Governance note</strong><em>REC-COMMS-COMM-023</em></button><button type="button" data-comms-bridge-route="receipt" data-comms-bridge-title="Universal receipt packet" data-comms-bridge-receipt="REC-COMMS-HANDOFF-006" data-comms-bridge-target="Audit ledger → Receipt stitching console"><span>04 · Receipt</span><strong>Audit stitch</strong><em>REC-COMMS-HANDOFF-006</em></button></div><aside class="comms-bridge-detail" aria-label="Selected communication receipt detail"><dl><div><dt>Selected</dt><dd data-comms-bridge-detail-title="true">Messenger ops room</dd></div><div><dt>Receipt</dt><dd data-comms-bridge-detail-receipt="true">REC-COMMS-MSG-021</dd></div><div><dt>Route</dt><dd data-comms-bridge-detail-target="true">Ops room → Mail brief → Community note</dd></div></dl><div class="comms-bridge-actions" aria-label="Communication receipt bridge actions"><button type="button" data-comms-bridge-action="workflow">Workflow</button><button type="button" data-comms-bridge-action="cloud">Cloud</button><button type="button" data-comms-bridge-action="audit">Audit receipt</button><button type="button" data-comms-bridge-action="draft">Draft all</button></div></aside></section>"#
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_comms_product_board(surface: ProductSurface) -> String {
    match surface {
        ProductSurface::Messenger => r#"<section class="comms-product-board messenger-board" data-comms-product-board="true" data-comms-board-surface="Messenger" aria-label="Messenger command workspace"><div class="comms-board-head"><div><p class="screen-anchor">MESSENGER COMMAND</p><h4>Ops room thread with FD-001 workload evidence</h4><span>Fast operational chat for dogfooding FD-001 microservices on Oyatie Cloud, with evidence links and action extraction.</span></div><div class="comms-board-actions"><span class="status-chip warning">2 unread</span><button type="button" data-comms-action="thread-escalate">Escalate</button><button type="button" data-comms-action="thread-to-mail">Promote to Mail</button><button type="button" data-comms-action="thread-receipt">Attach receipt</button></div></div><div class="comms-board-grid"><article class="thread-transcript-card"><p class="screen-anchor">LIVE THREAD</p><ol class="comms-transcript"><li><strong>Ops bot</strong><span>Kubernetes runtime tier drift detected in cell-us-east-2.</span><em>09:18 · unread</em></li><li class="mine"><strong>Tenant admin</strong><span>Link rollback runbook and notify Finance before close.</span><em>09:22 · local draft</em></li><li><strong>Security reviewer</strong><span>Need audit-chain evidence before promotion.</span><em>09:24 · evidence</em></li></ol></article><article><p class="screen-anchor">ACTION EXTRACTION</p><div class="comms-action-list"><button type="button" data-comms-action="create-task"><strong>Create task</strong><span>Rollback evidence owner · due 2.1h</span></button><button type="button" data-comms-action="link-workflow"><strong>Link workflow</strong><span>PROC-PAYROLL-CLOSE critical path</span></button><button type="button" data-comms-action="thread-to-mail"><strong>Draft formal mail</strong><span>CFO + SRE approval brief</span></button></div></article><article><p class="screen-anchor">PARTICIPANTS</p><div class="comms-presence-grid"><span><em>OP</em><strong>Ops bot</strong><small>online</small></span><span><em>SR</em><strong>Security</strong><small>watching</small></span><span><em>FL</em><strong>Finance</strong><small>mail owner</small></span><span><em>GV</em><strong>Governance</strong><small>council</small></span></div></article></div></section>"#.to_string(),
        ProductSurface::Mail => r#"<section class="comms-product-board mail-board" data-comms-product-board="true" data-comms-board-surface="Mail" aria-label="Mail command workspace"><div class="comms-board-head"><div><p class="screen-anchor">MAIL COMMAND</p><h4>Formal approval brief composer</h4><span>Structured mail draft with recipients, subject, FD-001 workload evidence attachments, Oyatie Cloud cell context, approvals, and send preview.</span></div><div class="comms-board-actions"><span class="status-chip ai">draft</span><button type="button" data-comms-action="mail-preview">Preview</button><button type="button" data-comms-action="mail-attach">Attach packet</button><button type="button" data-comms-action="send-preview">Send preview</button></div></div><div class="comms-mail-compose-grid"><article class="mail-envelope-card"><p class="screen-anchor">ENVELOPE</p><dl><div><dt>From</dt><dd>Finance lead · Oyatie</dd></div><div><dt>To</dt><dd>CFO, SRE reviewer</dd></div><div><dt>CC</dt><dd>Governance council, Audit</dd></div><div><dt>Subject</dt><dd>Approval needed: payroll close + cloud rollback evidence</dd></div></dl></article><article class="mail-body-card"><p class="screen-anchor">DRAFT BODY</p><div class="mail-paper"><strong>Please review the April close packet before 18:00.</strong><p>Payroll delta, HomeTax readiness, vendor exception, and Oyatie Cloud rollback evidence are attached as read-only receipts for the FD-001 tenant workload. No external send is enabled before live integration.</p><ol><li>REC-PAY-2026-04-PARK</li><li>REC-CLOUD-MESH-4182</li><li>REC-WF-7741</li></ol></div></article><article><p class="screen-anchor">APPROVAL CHECKS</p><div class="mail-checks"><span class="done">Human reviewer required</span><span class="done">PIPA-safe body</span><span class="review">CFO signoff pending</span><span>External delivery disabled</span></div></article></div></section>"#.to_string(),
        ProductSurface::Community => r#"<section class="comms-product-board community-board" data-comms-product-board="true" data-comms-board-surface="Community" aria-label="Community command workspace"><div class="comms-board-head"><div><p class="screen-anchor">COMMUNITY COMMAND</p><h4>Governance council publication</h4><span>Role-aware community post, voting, pinned Oyatie Cloud cell context, and moderation state for FD-001 tenant-workload coordination.</span></div><div class="comms-board-actions"><span class="status-chip success">role-aware</span><button type="button" data-comms-action="community-pin">Pin</button><button type="button" data-comms-action="community-poll">Open poll</button><button type="button" data-comms-action="publish-note">Publish local</button></div></div><div class="community-feed-grid"><article class="community-post-card"><p class="screen-anchor">PINNED POST</p><div class="community-post-preview"><span>Governance council</span><h5>April close governance digest</h5><p>Payroll blocker, withholding filing readiness, Oyatie Cloud rollback evidence, and reviewer assignments are summarized for role-visible FD-001 review.</p><div><button type="button" data-comms-action="community-upvote">▲ 24</button><button type="button" data-comms-action="community-comment">8 comments</button><button type="button" data-comms-action="community-save">Save</button></div></div></article><article><p class="screen-anchor">AUDIENCE</p><div class="community-audience-grid"><span><strong>Finance</strong><em>required</em></span><span><strong>SRE</strong><em>review</em></span><span><strong>People Ops</strong><em>visible</em></span><span><strong>Vendors</strong><em>blocked</em></span></div></article><article><p class="screen-anchor">MODERATION</p><dl class="community-moderation"><div><dt>Policy</dt><dd>PIPA-safe</dd></div><div><dt>Evidence</dt><dd>3 receipts</dd></div><div><dt>Publish</dt><dd>local only</dd></div></dl></article></div></section>"#.to_string(),
        ProductSurface::Workflow => r#"<section class="comms-product-board" data-comms-product-board="true"><p>Workflow route selected.</p></section>"#.to_string(),
    }
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_communication_hub(
    messages: &[MessageItem],
    communities: &[CommunityItem],
) -> String {
    let messenger_items = hub_items(messages, communities, &[], ProductSurface::Messenger);
    let mail_items = hub_items(messages, communities, &[], ProductSurface::Mail);
    let community_items = hub_items(messages, communities, &[], ProductSurface::Community);
    let messenger = messenger_items
        .iter()
        .enumerate()
        .map(|(index, item)| static_hub_button(item, index))
        .collect::<String>();
    let mail = mail_items
        .iter()
        .enumerate()
        .map(|(index, item)| static_hub_button(item, index))
        .collect::<String>();
    let community = community_items
        .iter()
        .enumerate()
        .map(|(index, item)| static_hub_button(item, index))
        .collect::<String>();
    let selected = messenger_items.first().cloned().unwrap_or(HubItem {
        surface: ProductSurface::Messenger,
        source: "Messenger".to_string(),
        title: "Local work hub".to_string(),
        body: "Use the local island to switch channels, inspect items, and queue drafts."
            .to_string(),
        meta: "Visual-only; no backend send".to_string(),
    });
    let selected_kind = hub_item_kind(&selected, 0);
    let selected_chip = hub_item_chip_class(selected_kind);
    let messenger_board = static_comms_product_board(ProductSurface::Messenger);
    let mail_board = static_comms_product_board(ProductSurface::Mail);
    let community_board = static_comms_product_board(ProductSurface::Community);
    let comms_receipt_bridge = static_comms_receipt_bridge();

    format!(
        r#"<div class="communications-hub interactive-hub"><div class="hub-tabs" role="tablist" aria-label="Work hub channels"><button type="button" role="tab" aria-selected="true" class="hub-tab active">Messenger</button><button type="button" role="tab" aria-selected="false" class="hub-tab">Mail</button><button type="button" role="tab" aria-selected="false" class="hub-tab">Community</button></div><div class="comms-kpi-strip" aria-label="Built-in communications summary"><span><strong>18</strong><small>threads · drafts</small></span><span><strong>6</strong><small>workflow routes</small></span><span><strong>4</strong><small>evidence links</small></span><span><strong>0</strong><small>external sends</small></span></div><div class="hub-route-board" aria-label="Workflow output routes"><div><p class="screen-anchor">OUTPUT ROUTES</p><strong>FD-001 tenant-workload drafts fan out to Messenger, Mail, and Community with evidence return paths</strong><span data-comms-route-status="true">FD-001 workload dogfood · REC-WF-7741 · no backend send</span></div><button type="button" data-hub-route="Messenger">Messenger post</button><button type="button" data-hub-route="Mail">Mail draft</button><button type="button" data-hub-route="Community">Community note</button></div>{messenger_board}<section class="comms-substrate-strip" aria-label="Oyatie Cloud tenant-workload proof"><div><p class="screen-anchor">SUBSTRATE PROOF</p><strong>Messenger, Mail, and Community are dogfood tenant workloads on Oyatie Cloud</strong><span data-comms-substrate-status="true">Messenger route pinned to FD-001 workload · cell-us-east-2 · local visual proof</span></div><button type="button" data-comms-action="prove-substrate"><span>Cloud cell</span><strong>cell-us-east-2</strong></button><button type="button" data-comms-action="route-cloud"><span>Tenant workload</span><strong>FD-001 microservices</strong></button><button type="button" data-comms-action="seal-proof"><span>Evidence</span><strong>REC-WF-7741</strong></button></section>{comms_receipt_bridge}<div class="comms-service-toolbar" aria-label="Communications workspace controls"><label><span aria-hidden="true">⌕</span><input data-comms-search="true" aria-label="Search communications" placeholder="Search threads, mail, spaces..." /></label><div class="comms-filter-pills" role="toolbar" aria-label="Communication filters"><button type="button" class="active" data-comms-filter="all">All</button><button type="button" data-comms-filter="unread">Unread</button><button type="button" data-comms-filter="draft">Drafts</button><button type="button" data-comms-filter="evidence">Evidence</button></div><button type="button" data-comms-action="new-thread">New thread</button><button type="button" data-comms-action="attach-evidence">Attach evidence</button><button type="button" data-comms-action="directory">Directory</button><span data-comms-status="true">Local service workspace ready · no external send</span></div><div class="hub-workspace comms-service-shell"><aside class="comms-sidebar" aria-label="Communications folders and spaces"><p class="screen-anchor">WORKSPACES</p><button type="button" class="active" data-hub-route="Messenger"><strong>Ops room</strong><span>Messenger · 5 items · 2 unread</span></button><button type="button" data-hub-route="Mail"><strong>Finance close</strong><span>Mail · 4 drafts · 2 evidence</span></button><button type="button" data-hub-route="Community"><strong>Governance council</strong><span>Community · 5 spaces · 1 publish</span></button><button type="button" data-comms-action="notification-filter"><strong>Notifications</strong><span>6 local alerts · no external send</span></button></aside><div class="hub-list" role="list" aria-label="Channel items">{messenger}</div><div class="hub-detail"><div class="comms-message-toolbar" aria-label="Selected communication actions"><span class="status-chip success">role-visible</span><button type="button" data-comms-action="mark-reviewed">Mark reviewed</button><button type="button" data-comms-action="create-task">Create task</button><button type="button" data-comms-action="link-workflow">Link workflow</button><button type="button" data-comms-action="send-preview">Send preview</button><button type="button" data-comms-action="publish-note">Publish local</button></div><article class="comms-detail-card"><div class="comms-detail-head"><div><p class="eyebrow">{surface}</p><h4>{title}</h4></div><span class="{selected_chip}">{selected_kind}</span></div><p>{body}</p><span class="hub-meta">{meta}</span><dl class="comms-detail-grid"><div><dt>Route</dt><dd>{surface}</dd></div><div><dt>Workflow</dt><dd>Tenant change approval</dd></div><div><dt>Receipt</dt><dd>REC-WF-7741</dd></div><div><dt>Persistence</dt><dd>Local browser state only</dd></div></dl></article><div class="hub-composer"><label for="static-hub-composer">Draft a local response</label><textarea id="static-hub-composer" rows="3" placeholder="Hydration enables local queueing."></textarea><div class="composer-actions"><button type="button">Queue draft</button><button type="button" class="secondary">Clear</button></div><p>Mail previews: {mail_count}. Community spaces: {community_count}. Visual-only; no external send.</p></div></div><aside class="comms-context-rail" aria-label="People, provenance, and notification context"><section><p class="screen-anchor">PEOPLE</p><div class="presence-stack"><span><em>OP</em><strong>Ops bot</strong><small>online</small></span><span><em>SR</em><strong>Security reviewer</strong><small>watching</small></span><span><em>FL</em><strong>Finance lead</strong><small>mail owner</small></span></div></section><section><p class="screen-anchor">PROVENANCE</p><dl class="comms-kv"><div><dt>Envelope</dt><dd>tenant-admin</dd></div><div><dt>Workflow</dt><dd>Tenant change approval</dd></div><div><dt>Receipt</dt><dd>REC-WF-7741</dd></div></dl></section><section class="comms-handoff-card" data-comms-handoff="true" aria-label="Local draft handoff state"><p class="screen-anchor">DRAFT HANDOFF BUS</p><strong data-comms-handoff-title="true">Messenger → Mail approval brief</strong><span data-comms-handoff-status="true">Select Promote to Mail or Publish local to carry context across surfaces.</span><dl class="comms-kv compact"><div><dt>Source</dt><dd data-comms-handoff-source="true">Messenger</dd></div><div><dt>Destination</dt><dd data-comms-handoff-destination="true">Mail</dd></div><div><dt>Audience</dt><dd data-comms-handoff-audience="true">CFO · SRE · Governance</dd></div><div><dt>Persistence</dt><dd>Browser local state only</dd></div></dl><div class="comms-handoff-actions"><button type="button" data-comms-action="thread-to-mail">Promote to Mail</button><button type="button" data-comms-action="publish-note">Publish local</button></div></section><section><p class="screen-anchor">DELIVERY MATRIX</p><div class="comms-delivery-matrix" aria-label="Local delivery readiness"><span class="ready"><strong>Messenger</strong><em>ops room draft</em></span><span class="ready"><strong>Mail</strong><em>approval brief</em></span><span class="review"><strong>Community</strong><em>council review</em></span><span><strong>Audit</strong><em>receipt attached</em></span></div></section><section><p class="screen-anchor">LOCAL NOTIFICATIONS</p><ol class="notification-stack"><li>Draft queued locally</li><li>Evidence link available</li><li>No external send enabled</li><li>Workflow route preview ready</li></ol></section></aside></div><template data-mail-preview="{mail}"></template><template data-community-preview="{community}"></template><template data-comms-board-template="Mail">{mail_board}</template><template data-comms-board-template="Community">{community_board}</template><template data-comms-board-template="Messenger">{messenger_board}</template></div>"#,
        messenger = messenger,
        messenger_board = messenger_board,
        mail_board = mail_board,
        community_board = community_board,
        comms_receipt_bridge = comms_receipt_bridge,
        mail = escape(&mail),
        community = escape(&community),
        surface = selected.surface.label(),
        title = escape(&selected.title),
        body = escape(&selected.body),
        meta = escape(&selected.meta),
        selected_chip = selected_chip,
        selected_kind = selected_kind,
        mail_count = mail_items.len(),
        community_count = community_items.len(),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_hub_button(item: &HubItem, index: usize) -> String {
    let kind = hub_item_kind(item, index);
    let chip = hub_item_chip_class(kind);
    let active = if index == 0 { " active" } else { "" };
    format!(
        r#"<button type="button" class="hub-item{active}" data-comms-item="true" data-comms-kind="{kind}"><span class="{chip}">{source}</span><strong>{title}</strong><p>{body}</p><small><em>{kind}</em><b>{meta}</b></small></button>"#,
        active = active,
        kind = kind,
        chip = chip,
        source = escape(&item.source),
        title = escape(&item.title),
        body = escape(&item.body),
        meta = escape(&item.meta),
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_task(task: &WorkItem) -> String {
    format!(
        "<li><span class=\"priority\">{}</span><strong>{}</strong><p>{}</p></li>",
        escape(&task.priority),
        escape(&task.title),
        escape(&task.detail)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_approval(approval: &ApprovalItem) -> String {
    format!(
        "<li><strong>{}</strong><p>{}</p><span>{}</span></li>",
        escape(&approval.title),
        escape(&approval.requester),
        escape(&approval.risk_note)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_schedule(item: &ScheduleItem) -> String {
    format!(
        "<li><time>{}</time><div><strong>{}</strong><p>{}</p></div></li>",
        escape(&item.time),
        escape(&item.title),
        escape(&item.detail)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_message(message: &MessageItem) -> String {
    format!(
        "<button type=\"button\" class=\"hub-item\"><span>{}</span><strong>{}</strong><p>{}</p></button>",
        escape(&message.channel),
        escape(&message.from),
        escape(&message.preview)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_community(item: &CommunityItem) -> String {
    format!(
        "<button type=\"button\" class=\"hub-item\"><span>{}</span><strong>{}</strong><p>{}</p></button>",
        escape(&item.space),
        escape(&item.topic),
        escape(&item.activity)
    )
}
