//! The controller [`Runtime`] engine. Mirrors `cosi-project/runtime`
//! `controller.Runtime` (and its queued variant, the `QRuntime`): controllers
//! are registered, their declared inputs/outputs are validated and turned into
//! a dependency graph, and a reconcile loop drives them to a fixed point.
//!
//! Unlike real COSI — which runs each controller in its own goroutine, woken by
//! watch channels — this engine is a single-threaded, deterministic scheduler.
//! It keeps a work queue of controllers that need reconciling, runs them in
//! dependency-respecting order, and re-enqueues any controller whose inputs
//! changed (because another controller wrote one of its inputs, or because the
//! controller asked to be requeued). The loop terminates when the queue drains
//! (a *settled* / converged runtime) or a fuel limit is hit.

use crate::controller::{Controller, ReconcileContext, ReconcileResult};
use crate::inputs::{InputKind, Spec};
use crate::metadata::Phase;
use crate::resource::ResourceKind;
use crate::store::State;
use crate::watch::EventKind;
use core::fmt;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Errors from runtime setup and execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// Two controllers share the same name.
    DuplicateController(String),
    /// Two controllers declared the same exclusive output kind (only one owner
    /// is allowed per kind in COSI).
    ConflictingOutput {
        /// The contested output kind.
        kind: String,
        /// The first controller that claimed it.
        first: String,
        /// The second controller that also claimed it.
        second: String,
    },
    /// The dependency graph among controllers contains a cycle, which the
    /// settle loop cannot order. Carries the controllers involved.
    DependencyCycle(Vec<String>),
    /// The reconcile loop did not converge within the fuel budget.
    NotConverged {
        /// Reconcile passes executed before giving up.
        passes: usize,
    },
    /// A controller reconcile returned an error while running through the
    /// source-shaped event-pass API.
    ControllerFailed {
        /// The controller whose pass failed.
        controller: String,
        /// Human-readable controller error.
        error: String,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::DuplicateController(n) => write!(f, "duplicate controller {n}"),
            RuntimeError::ConflictingOutput {
                kind,
                first,
                second,
            } => write!(f, "output {kind} claimed by both {first} and {second}"),
            RuntimeError::DependencyCycle(c) => {
                write!(f, "dependency cycle among controllers: {}", c.join(" -> "))
            }
            RuntimeError::NotConverged { passes } => {
                write!(f, "runtime did not converge after {passes} passes")
            }
            RuntimeError::ControllerFailed { controller, error } => {
                write!(f, "controller {controller} failed: {error}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// One registered controller plus its cached spec and watch bookkeeping.
struct Registration {
    controller: Box<dyn Controller>,
    spec: Spec,
    /// Per-input-kind watch handle index returned by [`State::watch_kind`].
    watches: BTreeMap<ResourceKind, usize>,
}

impl fmt::Debug for Registration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registration")
            .field("name", &self.controller.name())
            .field("inputs", &self.spec.inputs().len())
            .field("outputs", &self.spec.outputs().len())
            .finish()
    }
}

/// A record of one reconcile invocation, for observability/tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileRecord {
    /// The controller that ran.
    pub controller: String,
    /// Whether the reconcile returned Ok.
    pub ok: bool,
    /// Number of store writes the controller performed.
    pub writes: usize,
    /// Whether the controller requested a requeue.
    pub requeued: bool,
}

/// The controller runtime: owns the [`State`] store and a set of controllers,
/// and drives them to convergence.
pub struct Runtime {
    state: State,
    controllers: Vec<Registration>,
    /// Map from output kind -> owning controller index, used to wire the
    /// dependency graph and to detect conflicting exclusive outputs.
    producers: BTreeMap<ResourceKind, usize>,
    /// History of reconcile invocations from the last [`Runtime::run`].
    history: Vec<ReconcileRecord>,
    /// Per-run fuel: maximum number of reconcile invocations.
    max_passes: usize,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::new()
    }
}

impl Runtime {
    /// Build a runtime over a fresh empty store.
    pub fn new() -> Self {
        Runtime::with_state(State::new())
    }

    /// Build a runtime over an existing (possibly pre-populated) store.
    pub fn with_state(state: State) -> Self {
        Runtime {
            state,
            controllers: Vec::new(),
            producers: BTreeMap::new(),
            history: Vec::new(),
            max_passes: 1024,
        }
    }

    /// Set the maximum number of reconcile invocations per [`run`](Self::run).
    pub fn set_max_passes(&mut self, n: usize) {
        self.max_passes = n.max(1);
    }

    /// Borrow the underlying store (read-only).
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Mutably borrow the underlying store (e.g. for the user to seed inputs).
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// The reconcile history from the last run.
    pub fn history(&self) -> &[ReconcileRecord] {
        &self.history
    }

    /// Number of registered controllers.
    pub fn controller_count(&self) -> usize {
        self.controllers.len()
    }

    /// Register a controller. Validates name uniqueness and exclusive-output
    /// ownership, then wires watches for each input kind.
    pub fn register(&mut self, controller: Box<dyn Controller>) -> Result<(), RuntimeError> {
        let name = controller.name().to_string();
        if self.controllers.iter().any(|r| r.controller.name() == name) {
            return Err(RuntimeError::DuplicateController(name));
        }
        let spec = controller.spec();

        // Exclusive-output ownership: each exclusive kind has exactly one owner.
        let idx = self.controllers.len();
        for out in spec.outputs() {
            if out.is_exclusive()
                && let Some(&owner) = self.producers.get(out.kind())
            {
                return Err(RuntimeError::ConflictingOutput {
                    kind: out.kind().to_string(),
                    first: self.controllers[owner].controller.name().to_string(),
                    second: name,
                });
            }
            // Even shared outputs become producers for graph edges; first wins
            // as the representative producer.
            self.producers.entry(out.kind().clone()).or_insert(idx);
        }

        // Register a watch per input kind so input changes can wake us.
        let mut watches = BTreeMap::new();
        for kind in spec.input_kinds() {
            let h = self.state.watch_kind(kind.clone(), 4096);
            watches.insert(kind, h);
        }

        self.controllers.push(Registration {
            controller,
            spec,
            watches,
        });
        Ok(())
    }

    /// Compute a topological order of controllers based on input/output edges.
    /// Edge `producer -> consumer` when the consumer takes a kind the producer
    /// outputs as a (strong or weak) input. Returns indices in run order, or a
    /// cycle error.
    pub fn topo_order(&self) -> Result<Vec<usize>, RuntimeError> {
        let n = self.controllers.len();
        let mut adj: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
        let mut indeg = vec![0usize; n];

        for (ci, reg) in self.controllers.iter().enumerate() {
            for input in reg.spec.inputs() {
                if let Some(&producer) = self.producers.get(input.kind())
                    && producer != ci
                    && adj[producer].insert(ci)
                {
                    indeg[ci] += 1;
                }
            }
        }

        // Kahn's algorithm; ties broken by index for determinism.
        let mut queue: VecDeque<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
        let mut order = Vec::with_capacity(n);
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for &v in &adj[u] {
                indeg[v] -= 1;
                if indeg[v] == 0 {
                    queue.push_back(v);
                }
            }
        }

        if order.len() != n {
            // Remaining nodes form a cycle.
            let cycle: Vec<String> = (0..n)
                .filter(|i| !order.contains(i))
                .map(|i| self.controllers[i].controller.name().to_string())
                .collect();
            return Err(RuntimeError::DependencyCycle(cycle));
        }
        Ok(order)
    }

    /// Drain any pending watch events for a controller's inputs and report
    /// whether it has work to do. Used to decide re-enqueue.
    fn drain_inputs_with_bootstrap(&mut self, ctrl_idx: usize, bootstrap_is_work: bool) -> bool {
        let kinds: Vec<(ResourceKind, usize)> = self.controllers[ctrl_idx]
            .watches
            .iter()
            .map(|(k, h)| (k.clone(), *h))
            .collect();
        let mut changed = false;
        for (kind, handle) in kinds {
            if let Some(ch) = self.state.watch_mut(&kind, handle) {
                for ev in ch.drain() {
                    // Bootstrapped alone is not "real" work, but the snapshot
                    // Created events that precede it are. Source-shaped event
                    // passes opt into treating bootstrap as work because the
                    // controller's EventCh yielded and the controller Run loop
                    // does not inspect event type before reconciling.
                    if ev.kind() == EventKind::Bootstrapped {
                        if bootstrap_is_work {
                            changed = true;
                        }
                        continue;
                    }

                    let Some(resource) = ev.resource() else {
                        continue;
                    };
                    let Some(input) = self.controllers[ctrl_idx]
                        .spec
                        .matching_input(&kind, resource.metadata().id().as_str())
                    else {
                        continue;
                    };

                    let is_ready_destroy = || {
                        let metadata = resource.metadata();
                        metadata.owner() == self.controllers[ctrl_idx].controller.name()
                            && metadata.phase() == Phase::TearingDown
                            && metadata.finalizers().is_empty()
                    };
                    if input.strength() != InputKind::DestroyReady || is_ready_destroy() {
                        changed = true;
                    }
                }
            }
        }
        changed
    }

    /// Drain input events for the fixed-point convergence loop. Bootstrap-only
    /// snapshots are ignored because [`run`](Self::run) seeds all controllers
    /// explicitly before converging.
    fn drain_inputs(&mut self, ctrl_idx: usize) -> bool {
        self.drain_inputs_with_bootstrap(ctrl_idx, false)
    }

    /// Run a single reconcile of controller `idx`, recording the outcome.
    fn reconcile_one(&mut self, idx: usize) -> (ReconcileResult, usize, bool) {
        // Split the borrow: take the controller out temporarily.
        let spec = self.controllers[idx].spec.clone();
        let name = self.controllers[idx].controller.name().to_string();
        let mut ctx = ReconcileContext::new(&mut self.state, name.clone(), spec);
        let result = self.controllers[idx].controller.reconcile(&mut ctx);
        let writes = ctx.writes();
        let requeued = ctx.take_requeue().is_some();
        self.history.push(ReconcileRecord {
            controller: name,
            ok: result.is_ok(),
            writes,
            requeued,
        });
        (result, writes, requeued)
    }

    fn drain_queue(
        &mut self,
        mut queue: VecDeque<usize>,
        mut queued: BTreeSet<usize>,
        abort_on_error: bool,
    ) -> Result<usize, RuntimeError> {
        let mut passes = 0usize;

        while let Some(idx) = queue.pop_front() {
            queued.remove(&idx);
            if passes >= self.max_passes {
                return Err(RuntimeError::NotConverged { passes });
            }
            passes += 1;

            let (result, writes, requeued) = self.reconcile_one(idx);
            if abort_on_error && let Err(error) = result {
                return Err(RuntimeError::ControllerFailed {
                    controller: self.controllers[idx].controller.name().to_string(),
                    error: error.to_string(),
                });
            }

            // If this controller wrote anything, those writes produced watch
            // events; any controller (including itself) consuming the changed
            // kinds should be re-enqueued.
            if writes > 0 {
                for other in 0..self.controllers.len() {
                    let has_work = self.drain_inputs(other);
                    if has_work && !queued.contains(&other) {
                        queued.insert(other);
                        queue.push_back(other);
                    }
                }
            }

            if requeued && !queued.contains(&idx) {
                queued.insert(idx);
                queue.push_back(idx);
            }
        }

        Ok(passes)
    }

    /// Drive all controllers to convergence. Returns the number of reconcile
    /// passes executed. Controllers are seeded in topological order, then
    /// re-enqueued whenever their inputs change (because they themselves wrote
    /// something, another controller wrote an input, or they requested a
    /// requeue). Converges when the queue empties.
    pub fn run(&mut self) -> Result<usize, RuntimeError> {
        self.history.clear();
        let order = self.topo_order()?;

        // Drain the initial bootstrap snapshots before the first pass so the
        // first reconcile sees clean watch state.
        for idx in 0..self.controllers.len() {
            self.drain_inputs(idx);
        }

        let queue: VecDeque<usize> = order.into_iter().collect();
        let queued: BTreeSet<usize> = queue.iter().copied().collect();

        self.drain_queue(queue, queued, false)
    }

    /// Process pending controller input events, if any, without unconditionally
    /// seeding every controller.
    ///
    /// This is the source-shaped companion to [`run`](Self::run): Talos
    /// controllers block on `Runtime.EventCh()` and run one reconcile pass when
    /// the runtime yields an event. Unlike [`run`](Self::run), bootstrap-only
    /// watch events are therefore treated as work because the controller loop
    /// woke up even when the watched input is currently absent.
    ///
    /// Returns the number of reconcile passes executed. A zero return means no
    /// controller input event was pending.
    pub fn run_event_pass(&mut self) -> Result<usize, RuntimeError> {
        self.history.clear();
        let order = self.topo_order()?;

        let mut queue = VecDeque::new();
        let mut queued = BTreeSet::new();
        for idx in order {
            if self.drain_inputs_with_bootstrap(idx, true) && queued.insert(idx) {
                queue.push_back(idx);
            }
        }

        self.drain_queue(queue, queued, true)
    }
}

impl fmt::Debug for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Runtime")
            .field("controllers", &self.controllers.len())
            .field("producers", &self.producers.len())
            .field("state", &self.state)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::{ControllerError, ReconcileContext};
    use crate::inputs::{Input, Output};
    use crate::metadata::{Metadata, Phase};
    use crate::resource::{AnyResource, Resource};
    use os_kernel::ResourceId;

    // ---- Test resources -------------------------------------------------

    #[derive(Debug, Clone)]
    struct Config {
        meta: Metadata,
        replicas: u32,
    }
    impl Config {
        fn new(id: &str, replicas: u32) -> Self {
            Config {
                meta: Metadata::new("default", "Config", ResourceId::new(id).unwrap()),
                replicas,
            }
        }
        fn boxed(self) -> AnyResource {
            Box::new(self)
        }
    }
    impl Resource for Config {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("replicas={}", self.replicas)
        }
        fn clone_box(&self) -> AnyResource {
            Box::new(self.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct Status {
        meta: Metadata,
        replicas: u32,
    }
    impl Status {
        fn new(id: &str, replicas: u32) -> Self {
            Status {
                meta: Metadata::new("default", "Status", ResourceId::new(id).unwrap()),
                replicas,
            }
        }
    }
    impl Resource for Status {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("replicas={}", self.replicas)
        }
        fn clone_box(&self) -> AnyResource {
            Box::new(self.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct Summary {
        meta: Metadata,
        total: u32,
    }
    impl Summary {
        fn new(id: &str, total: u32) -> Self {
            Summary {
                meta: Metadata::new("default", "Summary", ResourceId::new(id).unwrap()),
                total,
            }
        }
    }
    impl Resource for Summary {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("total={}", self.total)
        }
        fn clone_box(&self) -> AnyResource {
            Box::new(self.clone())
        }
    }

    fn config_kind() -> ResourceKind {
        ResourceKind::new("default", "Config")
    }
    fn status_kind() -> ResourceKind {
        ResourceKind::new("default", "Status")
    }
    fn summary_kind() -> ResourceKind {
        ResourceKind::new("default", "Summary")
    }

    // ---- Controllers ----------------------------------------------------

    /// Mirrors each Config to a Status of the same id.
    #[derive(Debug)]
    struct StatusController;
    impl Controller for StatusController {
        fn name(&self) -> &str {
            "status-controller"
        }
        fn spec(&self) -> Spec {
            Spec::new()
                .with_input(Input::strong(config_kind()))
                .with_output(Output::exclusive(status_kind()))
        }
        fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
            for cfg in ctx.list(&config_kind(), None) {
                let id = cfg.metadata().id().as_str().to_string();
                let replicas: u32 = cfg
                    .spec_fingerprint()
                    .strip_prefix("replicas=")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let key = format!("default/Status/{id}");
                if ctx.contains(&key) {
                    let _ = ctx.modify(&key, |r| {
                        let meta = r.metadata().clone();
                        *r = Box::new(Status { meta, replicas });
                    });
                } else {
                    ctx.create(Box::new(Status::new(&id, replicas)))?;
                }
            }
            Ok(())
        }
    }

    /// Sums all Status replicas into a single Summary/all resource.
    #[derive(Debug)]
    struct SummaryController;
    impl Controller for SummaryController {
        fn name(&self) -> &str {
            "summary-controller"
        }
        fn spec(&self) -> Spec {
            Spec::new()
                .with_input(Input::weak(status_kind()))
                .with_output(Output::exclusive(summary_kind()))
        }
        fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
            let total: u32 = ctx
                .list(&status_kind(), None)
                .iter()
                .map(|s| {
                    s.spec_fingerprint()
                        .strip_prefix("replicas=")
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0)
                })
                .sum();
            let key = "default/Summary/all";
            if ctx.contains(key) {
                let _ = ctx.modify(key, |r| {
                    let meta = r.metadata().clone();
                    *r = Box::new(Summary { meta, total });
                });
            } else {
                ctx.create(Box::new(Summary::new("all", total)))?;
            }
            Ok(())
        }
    }

    // ---- Tests ----------------------------------------------------------

    #[test]
    fn register_rejects_duplicate_names() {
        let mut rt = Runtime::new();
        rt.register(Box::new(StatusController)).unwrap();
        let err = rt.register(Box::new(StatusController)).unwrap_err();
        assert!(matches!(err, RuntimeError::DuplicateController(_)));
    }

    #[test]
    fn register_rejects_conflicting_exclusive_output() {
        #[derive(Debug)]
        struct Other;
        impl Controller for Other {
            fn name(&self) -> &str {
                "other"
            }
            fn spec(&self) -> Spec {
                Spec::new().with_output(Output::exclusive(status_kind()))
            }
            fn reconcile(&mut self, _ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
                Ok(())
            }
        }
        let mut rt = Runtime::new();
        rt.register(Box::new(StatusController)).unwrap();
        let err = rt.register(Box::new(Other)).unwrap_err();
        assert!(matches!(err, RuntimeError::ConflictingOutput { .. }));
    }

    #[test]
    fn topo_order_is_producer_before_consumer() {
        let mut rt = Runtime::new();
        // Register consumer first to prove ordering is by edges, not insertion.
        rt.register(Box::new(SummaryController)).unwrap();
        rt.register(Box::new(StatusController)).unwrap();
        let order = rt.topo_order().unwrap();
        // status-controller produces Status which summary-controller consumes,
        // so status must come before summary.
        let pos_status = order
            .iter()
            .position(|&i| rt.controllers[i].controller.name() == "status-controller")
            .unwrap();
        let pos_summary = order
            .iter()
            .position(|&i| rt.controllers[i].controller.name() == "summary-controller")
            .unwrap();
        assert!(pos_status < pos_summary);
    }

    #[test]
    fn run_converges_and_propagates_through_chain() {
        let mut rt = Runtime::new();
        rt.register(Box::new(StatusController)).unwrap();
        rt.register(Box::new(SummaryController)).unwrap();
        // Seed two configs.
        rt.state_mut().create(Config::new("a", 3).boxed()).unwrap();
        rt.state_mut().create(Config::new("b", 5).boxed()).unwrap();

        let passes = rt.run().unwrap();
        assert!(passes >= 2);

        // Status mirrored.
        assert!(rt.state().contains("default/Status/a"));
        assert!(rt.state().contains("default/Status/b"));
        // Summary computed = 3 + 5 = 8.
        let summary = rt.state().get("default/Summary/all").unwrap();
        assert_eq!(summary.spec_fingerprint(), "total=8");
    }

    #[test]
    fn run_is_idempotent_second_run_no_writes() {
        let mut rt = Runtime::new();
        rt.register(Box::new(StatusController)).unwrap();
        rt.register(Box::new(SummaryController)).unwrap();
        rt.state_mut().create(Config::new("a", 3).boxed()).unwrap();
        rt.run().unwrap();
        // Second run: everything already converged, so no controller writes.
        rt.run().unwrap();
        let total_writes: usize = rt.history().iter().map(|r| r.writes).sum();
        assert_eq!(total_writes, 0, "converged runtime must not rewrite");
    }

    #[test]
    fn run_reacts_to_input_change() {
        let mut rt = Runtime::new();
        rt.register(Box::new(StatusController)).unwrap();
        rt.register(Box::new(SummaryController)).unwrap();
        rt.state_mut().create(Config::new("a", 3).boxed()).unwrap();
        rt.run().unwrap();
        assert_eq!(
            rt.state()
                .get("default/Summary/all")
                .unwrap()
                .spec_fingerprint(),
            "total=3"
        );
        // Change the config; the chain must recompute.
        let cur = rt.state().get("default/Config/a").unwrap();
        let v = cur.metadata().version();
        rt.state_mut()
            .update(Config::new("a", 10).boxed(), v)
            .unwrap();
        rt.run().unwrap();
        assert_eq!(
            rt.state()
                .get("default/Summary/all")
                .unwrap()
                .spec_fingerprint(),
            "total=10"
        );
    }

    #[test]
    fn dependency_cycle_is_detected() {
        // Two controllers each consuming the other's exclusive output.
        #[derive(Debug)]
        struct A;
        impl Controller for A {
            fn name(&self) -> &str {
                "a"
            }
            fn spec(&self) -> Spec {
                Spec::new()
                    .with_input(Input::weak(status_kind()))
                    .with_output(Output::exclusive(config_kind()))
            }
            fn reconcile(&mut self, _: &mut ReconcileContext<'_>) -> ReconcileResult {
                Ok(())
            }
        }
        #[derive(Debug)]
        struct B;
        impl Controller for B {
            fn name(&self) -> &str {
                "b"
            }
            fn spec(&self) -> Spec {
                Spec::new()
                    .with_input(Input::weak(config_kind()))
                    .with_output(Output::exclusive(status_kind()))
            }
            fn reconcile(&mut self, _: &mut ReconcileContext<'_>) -> ReconcileResult {
                Ok(())
            }
        }
        let mut rt = Runtime::new();
        rt.register(Box::new(A)).unwrap();
        rt.register(Box::new(B)).unwrap();
        let err = rt.topo_order().unwrap_err();
        assert!(matches!(err, RuntimeError::DependencyCycle(_)));
        assert!(matches!(
            rt.run().unwrap_err(),
            RuntimeError::DependencyCycle(_)
        ));
    }

    #[test]
    fn requeue_loop_is_fuel_limited() {
        #[derive(Debug)]
        struct Spinner;
        impl Controller for Spinner {
            fn name(&self) -> &str {
                "spinner"
            }
            fn spec(&self) -> Spec {
                Spec::new().with_output(Output::exclusive(summary_kind()))
            }
            fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
                ctx.requeue("never satisfied");
                Ok(())
            }
        }
        let mut rt = Runtime::new();
        rt.set_max_passes(10);
        rt.register(Box::new(Spinner)).unwrap();
        let err = rt.run().unwrap_err();
        assert!(matches!(err, RuntimeError::NotConverged { passes: 10 }));
    }

    #[test]
    fn history_records_each_reconcile() {
        let mut rt = Runtime::new();
        rt.register(Box::new(StatusController)).unwrap();
        rt.state_mut().create(Config::new("a", 1).boxed()).unwrap();
        rt.run().unwrap();
        assert!(!rt.history().is_empty());
        assert!(
            rt.history()
                .iter()
                .all(|r| r.controller == "status-controller")
        );
        assert!(rt.history().iter().any(|r| r.writes > 0));
    }

    #[test]
    fn controller_error_surfaces_in_history_but_does_not_abort() {
        #[derive(Debug)]
        struct Failing;
        impl Controller for Failing {
            fn name(&self) -> &str {
                "failing"
            }
            fn spec(&self) -> Spec {
                Spec::new().with_output(Output::exclusive(summary_kind()))
            }
            fn reconcile(&mut self, _: &mut ReconcileContext<'_>) -> ReconcileResult {
                Err(ControllerError::Failed("boom".into()))
            }
        }
        let mut rt = Runtime::new();
        rt.register(Box::new(Failing)).unwrap();
        let passes = rt.run().unwrap();
        assert_eq!(passes, 1);
        assert!(!rt.history()[0].ok);
    }

    #[test]
    fn event_pass_returns_controller_error_after_recording_history() {
        #[derive(Debug)]
        struct FailingOnConfig;
        impl Controller for FailingOnConfig {
            fn name(&self) -> &str {
                "failing-on-config"
            }
            fn spec(&self) -> Spec {
                Spec::new().with_input(Input::weak(config_kind()))
            }
            fn reconcile(&mut self, _: &mut ReconcileContext<'_>) -> ReconcileResult {
                Err(ControllerError::Failed("source pass failed".into()))
            }
        }

        let mut state = State::new();
        state.create(Config::new("a", 1).boxed()).unwrap();

        let mut rt = Runtime::with_state(state);
        rt.register(Box::new(FailingOnConfig)).unwrap();

        let err = rt.run_event_pass().unwrap_err();

        assert!(matches!(
            err,
            RuntimeError::ControllerFailed {
                controller,
                error
            } if controller == "failing-on-config" && error.contains("source pass failed")
        ));
        assert_eq!(rt.history().len(), 1);
        assert_eq!(rt.history()[0].controller, "failing-on-config");
        assert!(!rt.history()[0].ok);
    }

    #[test]
    fn event_pass_treats_bootstrap_event_as_controller_wake() {
        let mut rt = Runtime::new();
        rt.register(Box::new(SummaryController)).unwrap();

        let passes = rt.run_event_pass().unwrap();

        assert_eq!(passes, 1);
        assert_eq!(rt.history()[0].controller, "summary-controller");
        assert_eq!(
            rt.state()
                .get("default/Summary/all")
                .unwrap()
                .spec_fingerprint(),
            "total=0"
        );

        let passes = rt.run_event_pass().unwrap();

        assert_eq!(passes, 0);
        assert!(rt.history().is_empty());
    }

    #[test]
    fn event_pass_ignores_non_matching_specific_id_events_after_bootstrap() {
        #[derive(Debug)]
        struct ActiveConfigOnly;
        impl Controller for ActiveConfigOnly {
            fn name(&self) -> &str {
                "active-config-only"
            }

            fn spec(&self) -> Spec {
                Spec::new()
                    .with_input(Input::weak(config_kind()).with_id("active"))
                    .with_output(Output::exclusive(summary_kind()))
            }

            fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
                let total = ctx
                    .get("default/Config/active")
                    .and_then(|cfg| {
                        cfg.spec_fingerprint()
                            .strip_prefix("replicas=")
                            .and_then(|v| v.parse::<u32>().ok())
                    })
                    .unwrap_or(0);
                let key = "default/Summary/all";
                if ctx.contains(key) {
                    let _ = ctx.modify(key, |r| {
                        let meta = r.metadata().clone();
                        *r = Box::new(Summary { meta, total });
                    });
                } else {
                    ctx.create(Box::new(Summary::new("all", total)))?;
                }
                Ok(())
            }
        }

        let mut rt = Runtime::new();
        rt.register(Box::new(ActiveConfigOnly)).unwrap();

        let passes = rt.run_event_pass().unwrap();
        assert_eq!(passes, 1);

        rt.state_mut()
            .create(Config::new("inactive", 99).boxed())
            .unwrap();
        let passes = rt.run_event_pass().unwrap();
        assert_eq!(passes, 0);
        assert!(rt.history().is_empty());
        assert_eq!(
            rt.state()
                .get("default/Summary/all")
                .unwrap()
                .spec_fingerprint(),
            "total=0"
        );

        rt.state_mut()
            .create(Config::new("active", 7).boxed())
            .unwrap();
        let passes = rt.run_event_pass().unwrap();
        assert_eq!(passes, 1);
        assert_eq!(
            rt.state()
                .get("default/Summary/all")
                .unwrap()
                .spec_fingerprint(),
            "total=7"
        );
    }

    #[test]
    fn event_pass_filters_destroy_ready_inputs_to_owned_teardown_without_finalizers() {
        #[derive(Debug)]
        struct DestroyReadyObserver;
        impl Controller for DestroyReadyObserver {
            fn name(&self) -> &str {
                "destroy-ready-observer"
            }

            fn spec(&self) -> Spec {
                Spec::new().with_input(Input::destroy_ready(status_kind()))
            }

            fn reconcile(&mut self, _: &mut ReconcileContext<'_>) -> ReconcileResult {
                Ok(())
            }
        }

        fn status_with_metadata(
            id: &str,
            owner: &str,
            phase: Phase,
            finalizers: &[&str],
        ) -> Status {
            let mut status = Status::new(id, 0);
            status.metadata_mut().set_owner(owner);
            status.metadata_mut().set_phase(phase);
            for finalizer in finalizers {
                status.metadata_mut().finalizers_mut().add(*finalizer);
            }
            status
        }

        let mut rt = Runtime::new();
        rt.register(Box::new(DestroyReadyObserver)).unwrap();

        assert_eq!(rt.run_event_pass().unwrap(), 1);

        rt.state_mut()
            .create(Box::new(status_with_metadata(
                "running",
                "destroy-ready-observer",
                Phase::Running,
                &[],
            )))
            .unwrap();
        assert_eq!(rt.run_event_pass().unwrap(), 0);
        assert!(rt.history().is_empty());

        rt.state_mut()
            .create(Box::new(status_with_metadata(
                "still-finalized",
                "destroy-ready-observer",
                Phase::TearingDown,
                &["other-controller"],
            )))
            .unwrap();
        assert_eq!(rt.run_event_pass().unwrap(), 0);
        assert!(rt.history().is_empty());

        rt.state_mut()
            .create(Box::new(status_with_metadata(
                "foreign-owned",
                "other-controller",
                Phase::TearingDown,
                &[],
            )))
            .unwrap();
        assert_eq!(rt.run_event_pass().unwrap(), 0);
        assert!(rt.history().is_empty());

        rt.state_mut()
            .create(Box::new(status_with_metadata(
                "ready",
                "destroy-ready-observer",
                Phase::TearingDown,
                &[],
            )))
            .unwrap();
        assert_eq!(rt.run_event_pass().unwrap(), 1);
        assert_eq!(rt.history()[0].controller, "destroy-ready-observer");
    }

    #[test]
    fn event_pass_propagates_bootstrap_input_chain_in_topological_order() {
        let mut state = State::new();
        state.create(Config::new("a", 3).boxed()).unwrap();

        let mut rt = Runtime::with_state(state);
        rt.register(Box::new(SummaryController)).unwrap();
        rt.register(Box::new(StatusController)).unwrap();

        let passes = rt.run_event_pass().unwrap();

        assert_eq!(passes, 2);
        assert_eq!(rt.history()[0].controller, "status-controller");
        assert_eq!(rt.history()[1].controller, "summary-controller");
        assert_eq!(
            rt.state()
                .get("default/Summary/all")
                .unwrap()
                .spec_fingerprint(),
            "total=3"
        );
    }
}
