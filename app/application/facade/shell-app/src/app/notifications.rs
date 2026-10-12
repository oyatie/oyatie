use super::*;

type SampleNotification = (
    &'static str,
    bool,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

const RECORDS: [SampleNotification; 4] = [
    (
        "insurance-delta",
        false,
        "blocking",
        "09:18",
        "Sample insurance delta needs review",
        "An example insurance delta would need review before a payroll close.",
    ),
    (
        "withholding-brief",
        false,
        "review",
        "09:42",
        "Sample withholding brief",
        "An illustrative tax brief awaits review; no filing or transport is staged.",
    ),
    (
        "vendor-renewal",
        false,
        "blocking",
        "10:05",
        "Sample vendor renewal needs owner",
        "An example renewal exceeds the sample approval threshold; no attestation is requested.",
    ),
    (
        "audit-receipt",
        true,
        "info",
        "10:21",
        "Illustrative audit receipt",
        "REC-FIN-2026-05 is an example receipt, not a sealed or persisted audit record.",
    ),
];

pub(super) fn notification_unread_count() -> usize {
    RECORDS.iter().filter(|record| !record.1).count()
}

pub(crate) fn notification_source() -> String {
    serde_json::to_string(&RECORDS).expect("static notification records serialize")
}

pub(super) fn notification_body_html() -> String {
    let rows = RECORDS.iter().map(|(id, read, severity, time, title, detail)| {
        let audit = if *id == "audit-receipt" {
            r#"<button type="button" data-notification-route="audit" disabled>Open sample audit</button>"#
        } else { "" };
        format!(r#"<li data-activity-item="{id}" data-activity-state="{state}" data-activity-severity="{severity}">
<time data-notification-value="time">{time}</time><span class="{chip}">{severity}</span>
<strong data-notification-value="title">{title}</strong><button type="button" data-activity-action="mark-read" disabled>{read_label}</button>
<p data-notification-value="detail">{detail}</p>{audit}</li>"#,
            state = if *read { "read" } else { "unread" }, chip = daily_status_class(severity),
            time = daily_escape(time), title = daily_escape(title), detail = daily_escape(detail),
            read_label = if *read { "Read locally" } else { "Mark read locally" })
    }).collect::<String>();
    format!(
        r#"<div class="utility-panel-head"><div><p class="screen-anchor">SAMPLE ACTIVITY CENTER</p><h2 id="notification-title">Sample notifications</h2></div>
<button type="button" data-utility-close="true" aria-label="Close sample notifications" commandfor="notification-center" command="close" autofocus>×</button></div>
<section class="utility-proof-strip" aria-label="Notification sample scope"><article>
<strong>Anonymous examples only</strong><span id="notification-notice">Example events, times and receipts only. Mark read and clear read change this local preview. No approval, filing, delivery or audit mutation occurs.</span>
<span data-notification-context="true">Displayed sample context unavailable</span></article>
<div class="utility-route-grid" aria-label="Existing sample destinations">
<button type="button" data-notification-route="work-hub" disabled><span>Comms</span><strong>Sample Work Hub</strong></button>
<button type="button" data-notification-route="evidence" disabled><span>Receipt</span><strong>Sample Evidence</strong></button>
<button type="button" data-notification-route="cloud" disabled><span>Substrate</span><strong>Sample Cloud</strong></button></div></section>
<div class="utility-summary"><span><strong data-activity-count="unread">{unread}</strong><small>unread samples</small></span>
<span><strong data-activity-count="total">{total}</strong><small>sample records</small></span><span><strong data-activity-count="blocking">{blocking}</strong><small>blocking examples</small></span></div>
<div class="utility-filter-row" role="group" aria-label="Sample notification filters">
<button type="button" class="active" data-activity-filter="all" aria-pressed="true" disabled>All</button>
<button type="button" data-activity-filter="unread" aria-pressed="false" disabled>Unread</button>
<button type="button" data-activity-filter="blocking" aria-pressed="false" disabled>Blocking</button>
<button type="button" data-activity-action="clear-read" disabled>Clear read locally</button></div>
<p id="notification-status" class="notification-status" role="status" data-activity-status="true">Sample context unavailable; local actions are disabled.</p>
<p class="notification-empty" data-activity-empty="true" hidden></p>
<ol class="activity-list" data-activity-list="true">{rows}</ol>"#,
        unread = notification_unread_count(),
        total = RECORDS.len(),
        blocking = RECORDS
            .iter()
            .filter(|record| record.2 == "blocking")
            .count()
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn notification_dialog_html() -> String {
    format!(
        r#"<dialog id="notification-center" class="utility-panel activity-center" data-utility-panel="notifications" data-notification-source="{}" aria-labelledby="notification-title" aria-describedby="notification-notice notification-status" closedby="any">{}</dialog>"#,
        daily_escape(&notification_source()),
        notification_body_html()
    )
}
