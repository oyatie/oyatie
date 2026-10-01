use super::*;

pub(super) fn evidence_ledger_panel(evidence_events: [EvidenceEvent; 6]) -> impl IntoView {
    view! {
        <article id="evidence-ledger" class="evidence-ledger-panel" aria-labelledby="evidence-ledger-title">
            <div class="evidence-panel-head">
                <div>
                    <p class="screen-anchor">"LEDGER"</p>
                    <h4 id="evidence-ledger-title">"Receipt timeline"</h4>
                </div>
                <button type="button" data-evidence-action="attach">"Attach to inbox"</button>
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
                        <li class="evidence-event" data-evidence-event="true" data-evidence-state=state>
                            <button
                                type="button"
                                class="evidence-event-main"
                                data-evidence-action="open"
                                data-sidepeek-trigger="evidence"
                                data-sidepeek-title=title
                                data-sidepeek-id=receipt
                                data-sidepeek-desc=body
                                data-sidepeek-owner=owner
                                data-sidepeek-risk=state
                                data-sidepeek-sla=sla
                            >
                                <span class=chip_class>{state}</span>
                                <strong>{title}</strong>
                                <p>{body}</p>
                            </button>
                            <dl>
                                <div><dt>"Source"</dt><dd>{source}</dd></div>
                                <div><dt>"Owner"</dt><dd>{owner}</dd></div>
                                <div><dt>"SLA"</dt><dd>{sla}</dd></div>
                            </dl>
                        </li>
                    }
                }).collect_view()}
            </ol>
            {evidence_ledger_anchor_board()}
        </article>

    }
}
