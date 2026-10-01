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

#[cfg(target_arch = "wasm32")]
mod keyboard_navigation;
#[cfg(target_arch = "wasm32")]
use keyboard_navigation::*;

#[cfg(target_arch = "wasm32")]
mod fragment_navigation;
#[cfg(target_arch = "wasm32")]
use fragment_navigation::*;

#[cfg(target_arch = "wasm32")]
mod island_listeners;
#[cfg(target_arch = "wasm32")]
use island_listeners::*;

mod chrome;
use chrome::*;

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

mod command_center;
use command_center::*;

mod governance;
use governance::*;

mod governance_command;
use governance_command::*;

mod business_logic;
use business_logic::*;

mod identity_service;
use identity_service::*;

mod identity_anchors;
use identity_anchors::*;

mod identity_command;
use identity_command::*;

mod finance_service;
use finance_service::*;

mod finance_anchors;
use finance_anchors::*;

mod evidence;
use evidence::*;

mod evidence_anchors;
use evidence_anchors::*;

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

mod evidence_ledger;
use evidence_ledger::*;

mod communications_composer;
use communications_composer::*;

pub use chrome::{App, AppProps, AppPropsBuilder, shell_landmark_label, shell_scope_notice_text};
pub use dashboard::{DashboardIsland, DashboardIslandProps, DashboardIslandPropsBuilder};
