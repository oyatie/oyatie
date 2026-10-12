use leptos::prelude::*;

#[cfg(any(feature = "ssr", test))]
use crate::render_envelope::server_derived_envelope;
use crate::render_envelope::{
    ApprovalItem, CommunityItem, IntelligenceSuggestion, MessageItem, MetricCard, ModuleCard,
    OntologyFact, OperatorContext, ProductActivitySpine, ProductActivityStep, ScheduleItem,
    TenantRenderEnvelope, WorkItem, WorkflowNode,
};

mod models;
use models::*;

/// Reads UI state while retaining client-side dependency tracking.
///
/// Native SSR renders a single immutable response, so it must not register reactive dependencies
/// outside a browser reactive context. Hydrated browser builds retain ordinary tracked reads.
#[cfg(not(target_arch = "wasm32"))]
fn render_signal<T: Clone + Send + Sync + 'static>(signal: ReadSignal<T>) -> T {
    signal.get_untracked()
}

#[cfg(target_arch = "wasm32")]
fn render_signal<T: Clone + Send + Sync + 'static>(signal: ReadSignal<T>) -> T {
    signal.get()
}

#[cfg(any(feature = "ssr", test))]
mod static_render;
#[cfg(any(feature = "ssr", test))]
pub use static_render::{render_envelope_json, static_dashboard_html};

#[cfg(any(target_arch = "wasm32", feature = "ssr", test))]
mod keyboard_navigation;
#[cfg(any(feature = "ssr", test))]
pub(crate) use keyboard_navigation::ssr_tab_navigation_script;
#[cfg(target_arch = "wasm32")]
use keyboard_navigation::*;
#[cfg(test)]
mod keyboard_navigation_test;

#[cfg(any(target_arch = "wasm32", test))]
mod fragment_navigation;
#[cfg(target_arch = "wasm32")]
pub(crate) use fragment_navigation::wire_fragment_navigation;
#[cfg(target_arch = "wasm32")]
use fragment_navigation::*;

#[cfg(target_arch = "wasm32")]
mod island_listeners;
#[cfg(target_arch = "wasm32")]
use island_listeners::*;

mod sign_in;
use sign_in::*;

mod chrome;
use chrome::*;

mod notifications;
pub(crate) use notifications::notification_source;
use notifications::*;

mod utility_panels;
use utility_panels::*;

mod dashboard;
use dashboard::*;

mod envelope_request;
use envelope_request::*;

mod dashboard_view;
use dashboard_view::*;

mod product_activity;
use product_activity::*;

mod substrate;
use substrate::*;

mod action_inbox;
use action_inbox::*;

mod intelligence_sample;
use intelligence_sample::*;

mod command_center;
use command_center::*;

mod governance;
#[cfg(test)]
pub(crate) mod governance_owner_tests;
#[cfg(test)]
mod governance_tests;
use governance::*;

mod payroll_preview;
use payroll_preview::*;

mod governance_command;
use governance_command::*;

mod business_logic;
use business_logic::*;

mod account_sample;
use account_sample::*;
#[cfg(test)]
mod account_sample_test;

mod identity_service;
use identity_service::*;

mod employee_directory;

mod identity_anchors;
use identity_anchors::*;

mod identity_command;
use identity_command::*;

mod finance_preview;
use finance_preview::*;
#[cfg(test)]
mod finance_tests;

mod leave_time;
use leave_time::*;

mod finance_service;
use finance_service::*;

#[cfg(all(not(target_arch = "wasm32"), any(feature = "ssr", test)))]
pub(crate) fn finance_body_projection(
    context: OperatorContext,
) -> crate::render_envelope::FinanceBodyProjection {
    crate::render_envelope::FinanceBodyProjection {
        context,
        body: finance_service_markup(context).into_view().to_html(),
    }
}

mod finance_anchors;
use finance_anchors::*;

mod audit_evidence_preview;
use audit_evidence_preview::*;

mod evidence;
use evidence::*;

mod evidence_anchors;
use evidence_anchors::*;

mod policy_preview;
use policy_preview::*;

mod cloud_operations;
use cloud_operations::*;

mod cloud_command;
use cloud_command::*;

mod resource_audit;
use resource_audit::*;

mod deployment_gates;
use deployment_gates::*;

mod resource_rows;
use resource_rows::*;

mod daily_execution;
use daily_execution::*;

mod daily_rows;
use daily_rows::*;

mod communications;
use communications::*;

mod communications_proof;
use communications_proof::*;

mod hub_items;
use hub_items::*;

mod hub_sample;
use hub_sample::*;

mod catalog;
use catalog::*;

mod catalog_metadata;
use catalog_metadata::*;

mod workflow_studio;
use workflow_studio::*;

mod workflow_canvas;
use workflow_canvas::*;

mod workflow_output;
use workflow_output::*;

mod ontology;
use ontology::*;

mod workflow_nodes;
use workflow_nodes::*;

mod workflow_validation;
use workflow_validation::*;

mod evidence_ledger;
use evidence_ledger::*;

mod communications_composer;
use communications_composer::*;

pub use chrome::{App, AppProps, AppPropsBuilder, shell_landmark_label, shell_scope_notice_text};
pub use dashboard::{DashboardIsland, DashboardIslandProps, DashboardIslandPropsBuilder};
