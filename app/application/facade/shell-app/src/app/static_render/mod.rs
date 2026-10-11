use super::*;

mod shell;
use shell::*;
mod work_hub;
use work_hub::*;
mod workflow;
use workflow::*;
mod tenant_services;
use tenant_services::*;
mod operations;
use operations::*;
mod governance;
use governance::*;

#[cfg(any(feature = "ssr", test))]
pub fn render_envelope_json(context_id: &str) -> Option<String> {
    OperatorContext::from_id(context_id)
        .and_then(|context| serde_json::to_string(&server_derived_envelope(context)).ok())
}

#[cfg(any(feature = "ssr", test))]
pub fn static_dashboard_html() -> String {
    let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
    format!(
        r##"<div class="console-app">
  <a class="skip-link" href="#console-shell">Skip to dashboard</a>
  {rail}
  {header}
  <main id="console-shell" class="control-center" aria-labelledby="console-title" aria-describedby="console-notice">
    {hero}
    <div id="dashboard-island-root" class="dashboard-island" data-island="render-envelope-dashboard">
      {dashboard}
    </div>
  </main>
  {utility_panels}
  {side_peek}
  {command_palette}
</div>"##,
        rail = static_rail_html(),
        header = static_header_html(),
        hero = static_hero_html(),
        dashboard = static_dashboard_content(&envelope),
        utility_panels = static_utility_panels_html(),
        side_peek = static_side_peek_html(),
        command_palette = static_command_palette_html(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
