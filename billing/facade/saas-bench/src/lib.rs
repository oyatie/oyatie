//! SaaS bench application — end-to-end harness driving the M03-P04 preview.
//!
//! Composes [`workflow_saas_app::WorkflowEngine`] with
//! [`marketplace_plugin_kernel::MarketplaceRegistry`] +
//! [`oya_saas_plugin_app::PluginRuntime`] to exercise the full
//! `definition.publish` -> `run.start` -> `plugin.invocation` -> `run.complete`
//! sequence. Used by the M03-P04 acceptance lane to record SLO counters per
//! tenant and to verify cross-crate contracts compose without any external
//! dependencies.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use marketplace_plugin_kernel::{
    MarketplaceListingPublish, MarketplaceRegistry, PluginManifestRegister, TrustTier, Vertical,
};
use oya_saas_plugin_app::{
    PluginContext, PluginInvocation, PluginInvocationOutcome, PluginInvoke, PluginRuntime,
    PluginRuntimeError,
};
use workflow_saas_app::{
    PublishDefinitionInput, PublishStepInput, StartRunInput, WorkflowAppError, WorkflowEngine,
    WorkflowSloCounters,
};
use workflow_saas_kernel::{
    WorkflowEventKind, WorkflowRunId, WorkflowRunState, WorkflowStepId, WorkflowStepKind,
};

/// Errors returned by the bench harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BenchError {
    Workflow(WorkflowAppError),
    PluginRuntime(PluginRuntimeError),
    MarketplaceFailed,
}

impl From<WorkflowAppError> for BenchError {
    fn from(value: WorkflowAppError) -> Self {
        Self::Workflow(value)
    }
}

impl From<PluginRuntimeError> for BenchError {
    fn from(value: PluginRuntimeError) -> Self {
        Self::PluginRuntime(value)
    }
}

/// Bench scenario shape — one definition, one run, one plugin invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchScenario {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub regional_pack: String,         // data_class: INTERNAL_ONLY
    pub definition_id: String,         // data_class: INTERNAL_ONLY
    pub run_id: String,                // data_class: INTERNAL_ONLY
    pub manifest_id: String,           // data_class: INTERNAL_ONLY
    pub listing_id: String,            // data_class: INTERNAL_ONLY
    pub invocation_id: String,         // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

/// Aggregated outcome of a scenario run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchOutcome {
    pub counters: WorkflowSloCounters,     // data_class: INTERNAL_ONLY
    pub invocation: PluginInvocation,      // data_class: INTERNAL_ONLY
    pub final_run_state: WorkflowRunState, // data_class: INTERNAL_ONLY
    pub event_count: usize,                // data_class: INTERNAL_ONLY
}

/// Composite harness owning all three substrate components.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SaasBench {
    pub engine: WorkflowEngine,        // data_class: INTERNAL_ONLY
    pub registry: MarketplaceRegistry, // data_class: INTERNAL_ONLY
    pub runtime: PluginRuntime,        // data_class: INTERNAL_ONLY
}

impl SaasBench {
    /// Execute one bench scenario end-to-end.
    pub fn run_scenario(&mut self, scenario: BenchScenario) -> Result<BenchOutcome, BenchError> {
        // 1. Marketplace: register a Cosign-signed plugin manifest + listing.
        self.registry
            .register_manifest(PluginManifestRegister {
                id: scenario.manifest_id.clone(),
                publisher_id: "pub_bench".to_string(),
                name: "Bench Plugin".to_string(),
                semver: "0.1.0".to_string(),
                cosign_signature: "cosign:sha256:bench".to_string(),
                entrypoint: "wasm/bench.wasm".to_string(),
                registered_at_epoch_seconds: scenario.started_at_epoch_seconds,
            })
            .map_err(|_| BenchError::MarketplaceFailed)?;
        self.registry
            .publish_listing(MarketplaceListingPublish {
                id: scenario.listing_id.clone(),
                manifest_id: scenario.manifest_id.clone(),
                trust_tier: TrustTier::Reviewed,
                verticals: vec![Vertical::Development],
                regional_packs: vec![scenario.regional_pack.clone()],
                headline: "Bench harness plugin".to_string(),
                published_at_epoch_seconds: scenario.started_at_epoch_seconds,
            })
            .map_err(|_| BenchError::MarketplaceFailed)?;

        // 2. Workflow: publish a definition + start a run.
        self.engine.publish(PublishDefinitionInput {
            definition_id: scenario.definition_id.clone(),
            tenant_id: scenario.tenant_id.clone(),
            regional_pack: scenario.regional_pack.clone(),
            steps: vec![PublishStepInput {
                step_id: "wfs_bench".to_string(),
                kind: WorkflowStepKind::Plugin,
                order: 1,
                plugin_manifest: scenario.manifest_id.clone(),
            }],
            published_at_epoch_seconds: scenario.started_at_epoch_seconds,
        })?;
        let run_id = WorkflowRunId::new(scenario.run_id.clone())
            .map_err(|_| BenchError::Workflow(WorkflowAppError::InvalidId))?;
        self.engine.start_run(StartRunInput {
            run_id: scenario.run_id.clone(),
            definition_id: scenario.definition_id.clone(),
            started_at_epoch_seconds: scenario.started_at_epoch_seconds + 10,
        })?;
        self.engine.record_step(
            &run_id,
            &WorkflowStepId::new("wfs_bench")
                .map_err(|_| BenchError::Workflow(WorkflowAppError::InvalidId))?,
            WorkflowEventKind::StepStarted,
            scenario.started_at_epoch_seconds + 20,
        )?;

        // 3. Plugin runtime: invoke the manifest under the run context.
        let context = PluginContext::new(
            scenario.tenant_id.clone(),
            scenario.regional_pack.clone(),
            Some(scenario.run_id.clone()),
            scenario.started_at_epoch_seconds + 999,
            TrustTier::Reviewed,
        )?;
        let invocation = self.runtime.invoke(
            &self.registry,
            PluginInvoke {
                id: scenario.invocation_id.clone(),
                manifest_id: scenario.manifest_id.clone(),
                context,
                payload_bytes: b"{\"op\":\"bench\"}".to_vec(),
                started_at_epoch_seconds: scenario.started_at_epoch_seconds + 30,
                finished_at_epoch_seconds: scenario.started_at_epoch_seconds + 40,
                outcome: PluginInvocationOutcome::Succeeded,
            },
        )?;

        // 4. Workflow: complete the run and snapshot.
        self.engine.record_step(
            &run_id,
            &WorkflowStepId::new("wfs_bench")
                .map_err(|_| BenchError::Workflow(WorkflowAppError::InvalidId))?,
            WorkflowEventKind::StepCompleted,
            scenario.started_at_epoch_seconds + 50,
        )?;
        self.engine.complete_run(
            &run_id,
            WorkflowRunState::Succeeded,
            scenario.started_at_epoch_seconds + 60,
        )?;
        let snap =
            self.engine
                .snapshot(&run_id)
                .ok_or(BenchError::Workflow(WorkflowAppError::Domain(
                    workflow_saas_domain::WorkflowDomainError::UnknownRun,
                )))?;
        Ok(BenchOutcome {
            counters: self.engine.counters(&scenario.tenant_id),
            invocation,
            final_run_state: snap.run.state,
            event_count: snap.events.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario() -> BenchScenario {
        BenchScenario {
            tenant_id: "ten_bench".to_string(),
            regional_pack: "oya-pack-alpha".to_string(),
            definition_id: "wfd_bench_v1".to_string(),
            run_id: "wfr_bench_001".to_string(),
            manifest_id: "plg_bench_v1".to_string(),
            listing_id: "lst_bench_v1".to_string(),
            invocation_id: "inv_bench_001".to_string(),
            started_at_epoch_seconds: 1_700_000_000,
        }
    }

    #[test]
    fn bench_runs_end_to_end_scenario_with_slo_counters() {
        let mut bench = SaasBench::default();
        let outcome = bench.run_scenario(scenario()).expect("scenario succeeds");
        assert_eq!(outcome.final_run_state, WorkflowRunState::Succeeded);
        assert_eq!(outcome.counters.definitions_published, 1);
        assert_eq!(outcome.counters.runs_started, 1);
        assert_eq!(outcome.counters.runs_succeeded, 1);
        assert_eq!(
            outcome.invocation.outcome,
            PluginInvocationOutcome::Succeeded
        );
        // RunStarted + StepStarted + StepCompleted + RunCompleted = 4.
        assert_eq!(outcome.event_count, 4);
    }

    #[test]
    fn bench_rejects_invalid_tenant_via_workflow_app() {
        let mut bench = SaasBench::default();
        let bad = bench
            .run_scenario(BenchScenario {
                tenant_id: "bench".to_string(),
                ..scenario()
            })
            .expect_err("bad tenant rejected");
        match bad {
            BenchError::Workflow(_) => {}
            other => panic!("expected workflow error, got {other:?}"),
        }
    }

    #[test]
    fn bench_rejects_duplicate_invocation_on_replay() {
        let mut bench = SaasBench::default();
        bench.run_scenario(scenario()).unwrap();
        // Replaying the same scenario must hit duplicate-listing path before
        // the duplicate invocation; either way the bench refuses to silently
        // double-count.
        let again = bench.run_scenario(scenario());
        assert!(again.is_err(), "duplicate scenario rejected");
    }

    #[test]
    fn bench_runs_multiple_tenants_independently() {
        let mut bench = SaasBench::default();
        bench.run_scenario(scenario()).unwrap();
        let mut second = scenario();
        second.tenant_id = "ten_other".to_string();
        second.definition_id = "wfd_bench_v2".to_string();
        second.run_id = "wfr_bench_002".to_string();
        second.manifest_id = "plg_bench_v2".to_string();
        second.listing_id = "lst_bench_v2".to_string();
        second.invocation_id = "inv_bench_002".to_string();
        let outcome = bench.run_scenario(second).expect("second tenant scenario");
        assert_eq!(outcome.counters.runs_succeeded, 1);
        assert_eq!(bench.engine.counters("ten_bench").runs_succeeded, 1);
        assert_eq!(bench.engine.counters("ten_other").runs_succeeded, 1);
    }
}
