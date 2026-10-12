use super::*;

pub(super) fn evidence_ledger_panel(
    evidence_events: [EvidenceEvent; 6],
    context: OperatorContext,
) -> impl IntoView {
    view! {
        <article id="evidence-ledger" class="evidence-ledger-panel" aria-labelledby="evidence-ledger-title">
            {payroll_preview_detail(PayrollPreview::Evidence)}
            <div class="evidence-panel-head">
                <div>
                    <p class="screen-anchor">"LEDGER"</p>
                    <h4 id="evidence-ledger-title">"Receipt timeline"</h4>
                </div>
                {ae_action("evidence", "attach", "Attach to inbox")}
            </div>
            <ol class="evidence-event-list">
                {evidence_events.into_iter().map(|(state, receipt, title, body, source, owner, sla)| {
                    let chip_class = match state {
                        "blocking" => "status-chip danger",
                        "review" => "status-chip warning",
                        "sealed" => "status-chip success",
                        _ => "status-chip",
                    };
                    view! {
                        <li class="evidence-event" data-evidence-event="true" data-evidence-state=state data-ae-record-row="true">
                            <button
                                type="button"
                                class="evidence-event-main"
                                data-evidence-action="open"
                                data-ae-select=format!("event:{receipt}")
                                data-ae-record=ae_record(&format!("event:{receipt}"))
                                data-ae-focus=format!("event:{receipt}")
                                aria-pressed="false"
                                data-sidepeek-title=title
                                data-sidepeek-id=receipt
                                data-sidepeek-desc=body
                                data-sidepeek-owner=owner
                                data-sidepeek-risk=state
                                data-sidepeek-sla=sla
                            >
                                <span class=chip_class><span data-ae-value="State">{state}</span>" example"</span>
                                <strong data-ae-value="Title">{title}</strong>
                                <p data-ae-value="Body">{body}</p>
                                <code data-ae-value="Receipt">{receipt}</code>
                            </button>
                            <dl>
                                <div><dt>"Source"</dt><dd data-ae-value="Source">{source}</dd></div>
                                <div><dt>"Owner"</dt><dd data-ae-value="Owner">{owner}</dd></div>
                                <div><dt>"SLA"</dt><dd data-ae-value="SLA">{sla}</dd></div>
                            </dl>
                        </li>
                    }
                }).collect_view()}
            </ol>
            {evidence_ledger_anchor_board(context)}
        </article>

    }
}
