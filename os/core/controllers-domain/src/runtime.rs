//! The controller runtime: owns the shared [`State`] store and a set of
//! registered [`Controller`]s, builds the input->controller routing table, and
//! drives reconcile passes.
//!
//! Mirrors `cosi-project/runtime`'s `runtime.Runtime`: each controller is
//! registered with its declared inputs/outputs; a change to any input kind
//! schedules the dependent controllers, and the runtime loops reconciling until
//! the system quiesces (no controller requests a requeue / no output changes).

use crate::reconcile::{Controller, ReconcileContext, ReconcileResult};
use std::collections::{BTreeMap, BTreeSet};
use os_cosi_domain::State;
use os_cosi_domain::resource::ResourceKind;

/// A registered controller plus its declared shape, kept boxed in the runtime.
struct Registered {
    controller: Box<dyn Controller>,
    outputs: Vec<ResourceKind>,
}

/// Summary of a single runtime tick.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TickReport {
    /// Names of controllers that reconciled this tick, in run order.
    pub ran: Vec<String>,
    /// Names of controllers that requested a requeue.
    pub requeued: Vec<String>,
}

impl TickReport {
    /// Whether any controller asked to be invoked again.
    pub fn needs_another_tick(&self) -> bool {
        !self.requeued.is_empty()
    }
}

/// The controller runtime.
#[derive(Default)]
pub struct ControllerRuntime {
    state: State,
    controllers: Vec<Registered>,
    /// Map from input kind -> indices of controllers depending on it.
    routes: BTreeMap<ResourceKind, BTreeSet<usize>>,
}

impl ControllerRuntime {
    /// Build an empty runtime with a fresh store.
    pub fn new() -> Self {
        ControllerRuntime {
            state: State::new(),
            controllers: Vec::new(),
            routes: BTreeMap::new(),
        }
    }

    /// Build a runtime around an existing populated store.
    pub fn with_state(state: State) -> Self {
        ControllerRuntime {
            state,
            controllers: Vec::new(),
            routes: BTreeMap::new(),
        }
    }

    /// Read access to the shared store.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Mutable access to the shared store (e.g. to seed inputs in tests).
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// Number of registered controllers.
    pub fn controller_count(&self) -> usize {
        self.controllers.len()
    }

    /// Register a controller, wiring its input kinds into the routing table.
    pub fn register(&mut self, controller: Box<dyn Controller>) {
        let idx = self.controllers.len();
        for input in controller.inputs() {
            self.routes.entry(input.kind).or_default().insert(idx);
        }
        let outputs = controller.outputs().into_iter().map(|o| o.kind).collect();
        self.controllers.push(Registered {
            controller,
            outputs,
        });
    }

    /// The controllers (by name) registered to react to a given input kind.
    pub fn dependents_of(&self, kind: &ResourceKind) -> Vec<&str> {
        match self.routes.get(kind) {
            Some(set) => set
                .iter()
                .map(|&i| self.controllers[i].controller.name())
                .collect(),
            None => Vec::new(),
        }
    }

    /// Run a single reconcile pass over every registered controller, in
    /// registration order. Returns a [`TickReport`].
    pub fn tick(&mut self) -> ReconcileResult<TickReport> {
        let mut report = TickReport::default();
        for i in 0..self.controllers.len() {
            let outputs = self.controllers[i].outputs.clone();
            let name = self.controllers[i].controller.name().to_string();
            let requeue = {
                let mut ctx = ReconcileContext::new(&mut self.state, name.clone(), outputs);
                self.controllers[i].controller.reconcile(&mut ctx)?;
                ctx.requeue_requested()
            };
            report.ran.push(name.clone());
            if requeue {
                report.requeued.push(name);
            }
        }
        Ok(report)
    }

    /// Drive ticks until no controller requests a requeue or `max_ticks` is
    /// reached. Returns the number of ticks executed.
    pub fn run_until_stable(&mut self, max_ticks: usize) -> ReconcileResult<usize> {
        let mut ticks = 0;
        while ticks < max_ticks {
            ticks += 1;
            let report = self.tick()?;
            if !report.needs_another_tick() {
                break;
            }
        }
        Ok(ticks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconcile::{Input, Output};
    use os_kernel::ResourceId;
    use os_cosi_domain::{Metadata, Resource};

    #[derive(Debug, Clone)]
    struct Src {
        meta: Metadata,
        n: u32,
    }
    impl Src {
        fn new(id: &str, n: u32) -> Self {
            Src {
                meta: Metadata::new("runtime", "Src", ResourceId::new(id).unwrap()),
                n,
            }
        }
    }
    impl Resource for Src {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("n={}", self.n)
        }
        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct Dst {
        meta: Metadata,
        doubled: u32,
    }
    impl Dst {
        fn new(id: &str, doubled: u32) -> Self {
            Dst {
                meta: Metadata::new("runtime", "Dst", ResourceId::new(id).unwrap()),
                doubled,
            }
        }
    }
    impl Resource for Dst {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("doubled={}", self.doubled)
        }
        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    /// Doubles every Src into a Dst.
    struct Doubler {
        requeue_once: std::cell::Cell<bool>,
    }
    impl Controller for Doubler {
        fn name(&self) -> &str {
            "Doubler"
        }
        fn inputs(&self) -> Vec<Input> {
            vec![Input::weak(ResourceKind::new("runtime", "Src"))]
        }
        fn outputs(&self) -> Vec<Output> {
            vec![Output::new(ResourceKind::new("runtime", "Dst"))]
        }
        fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
            for r in ctx.list(&ResourceKind::new("runtime", "Src")) {
                let id = r.metadata().id().as_str();
                // Re-fetch as Src via fingerprint trick: parse n from fingerprint.
                let n: u32 = r
                    .spec_fingerprint()
                    .trim_start_matches("n=")
                    .parse()
                    .unwrap_or(0);
                ctx.write(Box::new(Dst::new(id, n * 2)))?;
            }
            if self.requeue_once.get() {
                self.requeue_once.set(false);
                ctx.requeue("warming up");
            }
            Ok(())
        }
    }

    #[test]
    fn register_builds_routes() {
        let mut rt = ControllerRuntime::new();
        rt.register(Box::new(Doubler {
            requeue_once: std::cell::Cell::new(false),
        }));
        assert_eq!(rt.controller_count(), 1);
        let deps = rt.dependents_of(&ResourceKind::new("runtime", "Src"));
        assert_eq!(deps, vec!["Doubler"]);
        assert!(
            rt.dependents_of(&ResourceKind::new("runtime", "Other"))
                .is_empty()
        );
    }

    #[test]
    fn tick_reconciles_outputs() {
        let mut rt = ControllerRuntime::new();
        rt.state_mut().create(Box::new(Src::new("a", 3))).unwrap();
        rt.state_mut().create(Box::new(Src::new("b", 5))).unwrap();
        rt.register(Box::new(Doubler {
            requeue_once: std::cell::Cell::new(false),
        }));

        let report = rt.tick().unwrap();
        assert_eq!(report.ran, vec!["Doubler"]);
        assert!(!report.needs_another_tick());

        let dst = rt.state().get("runtime/Dst/a").unwrap();
        assert_eq!(dst.spec_fingerprint(), "doubled=6");
        let dst = rt.state().get("runtime/Dst/b").unwrap();
        assert_eq!(dst.spec_fingerprint(), "doubled=10");
    }

    #[test]
    fn run_until_stable_loops_on_requeue() {
        let mut rt = ControllerRuntime::new();
        rt.state_mut().create(Box::new(Src::new("a", 1))).unwrap();
        rt.register(Box::new(Doubler {
            requeue_once: std::cell::Cell::new(true),
        }));
        let ticks = rt.run_until_stable(10).unwrap();
        // first tick requeues, second tick is stable
        assert_eq!(ticks, 2);
    }
}
