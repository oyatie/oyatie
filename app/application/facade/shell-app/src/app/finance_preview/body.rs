use super::*;

pub(super) fn finance_render_body_view(
    envelope: &TenantRenderEnvelope,
    native: bool,
) -> impl IntoView + use<> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        view! {
            <div class="finance-preview-body" inner_html=finance_render_body_html(envelope, native)></div>
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let scope = finance_sample_scope(envelope.context);
        view! {
            <div class="finance-preview-body">
                {finance_service_markup(envelope.context)}
                {["commercial", "finops", "filing"].into_iter().map(|group| view! {
                    <template data-finance-rail-template=group inner_html=finance_rail_html(scope, group, native)></template>
                }).collect_view()}
            </div>
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn finance_render_body_html(envelope: &TenantRenderEnvelope, native: bool) -> String {
    let body = finance_body_html(envelope, native)
        + &["commercial", "finops", "filing"]
            .into_iter()
            .map(|group| {
                format!(
                    r#"<template data-finance-rail-template="{group}">{}</template>"#,
                    finance_rail_html(finance_sample_scope(envelope.context), group, native)
                )
            })
            .collect::<String>();
    #[cfg(not(target_arch = "wasm32"))]
    if finance_sample_scope(envelope.context).commercial_examples {
        return body
            + &[
                HubChannel::Mail,
                HubChannel::Community,
                HubChannel::Messenger,
            ]
            .into_iter()
            .map(|channel| {
                format!(
                    r#"<template data-finance-hub-template="{}">{}</template>"#,
                    channel.label(),
                    communication_hub_html(envelope, channel)
                )
            })
            .collect::<String>();
    }
    body
}
