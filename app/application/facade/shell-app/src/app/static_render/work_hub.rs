use super::*;

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_daily_execution_console(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"<section id="daily-execution" tabindex="-1" class="daily-execution-console panel" aria-labelledby="daily-execution-title" data-daily-source="{}" data-daily-context="{}" data-daily-label="{}" data-daily-mode="ssr" data-daily-available="true">{}</section>"#,
        daily_escape(&daily_sample_source(envelope)),
        envelope.context.id(),
        daily_escape(&daily_sample_label(envelope)),
        daily_execution_body_html(envelope)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_communication_hub(envelope: &TenantRenderEnvelope) -> String {
    communication_hub_html(envelope, HubChannel::Messenger)
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
