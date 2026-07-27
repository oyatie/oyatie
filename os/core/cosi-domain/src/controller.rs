//! The [`Controller`] trait and the [`ReconcileContext`] adapter that mediates
//! a controller's access to the [`State`] store. Mirrors COSI
//! `controller.Controller` plus the `controller.Runtime`/`controller.Reader`/
//! `controller.Writer` surfaces handed to a controller during reconciliation.
//!
//! A controller is a pure-ish function over its inputs that writes outputs. It
//! declares its [`Spec`] (inputs + outputs) once, and on each `reconcile` call
//! reads the current state of its inputs and modifies its outputs. All access
//! goes through [`ReconcileContext`], which enforces the controller's declared
//! output set and ownership rules (mirroring COSI's `ControllerAdapter`).

use crate::inputs::Spec;
use crate::metadata::Phase;
use crate::resource::{AnyResource, ResourceKind};
use crate::store::{State, StoreError, StoreResult};
use core::fmt;
use std::collections::BTreeSet;

/// Errors surfaced from reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerError {
    /// A store operation failed.
    Store(StoreError),
    /// The controller attempted to write a kind it did not declare as an output.
    UndeclaredOutput(String),
    /// The controller asked to be requeued; not a hard failure. Carries an
    /// optional human-readable reason.
    Requeue(String),
    /// A generic controller-defined failure.
    Failed(String),
}

impl From<StoreError> for ControllerError {
    fn from(e: StoreError) -> Self {
        ControllerError::Store(e)
    }
}

impl fmt::Display for ControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControllerError::Store(e) => write!(f, "store error: {e}"),
            ControllerError::UndeclaredOutput(k) => {
                write!(f, "controller wrote undeclared output kind {k}")
            }
            ControllerError::Requeue(r) => write!(f, "requeue requested: {r}"),
            ControllerError::Failed(m) => write!(f, "controller failed: {m}"),
        }
    }
}

impl std::error::Error for ControllerError {}

/// Result of a single reconcile pass.
pub type ReconcileResult = Result<(), ControllerError>;

/// A COSI controller: declares its inputs/outputs and reconciles state.
pub trait Controller: fmt::Debug {
    /// A unique, stable controller name. Used as the owner of exclusive outputs
    /// and the finalizer name on strong inputs.
    fn name(&self) -> &str;

    /// The input/output declaration. Called once at registration; must be
    /// stable for the controller's lifetime.
    fn spec(&self) -> Spec;

    /// Reconcile: read inputs and converge outputs toward desired state. Called
    /// by the runtime whenever a watched input changes (and once at startup).
    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult;
}

/// Mediated, access-controlled view of the store handed to a controller during
/// reconciliation. Enforces the controller's declared outputs and stamps the
/// controller as the owner of exclusive writes.
pub struct ReconcileContext<'a> {
    state: &'a mut State,
    controller: String,
    spec: Spec,
    /// Number of writes performed this pass (for observability/tests).
    writes: usize,
    requeue: Option<String>,
    tracked_outputs: Option<BTreeSet<String>>,
}

impl<'a> ReconcileContext<'a> {
    /// Build a context for `controller` against `state` with declared `spec`.
    pub fn new(state: &'a mut State, controller: impl Into<String>, spec: Spec) -> Self {
        ReconcileContext {
            state,
            controller: controller.into(),
            spec,
            writes: 0,
            requeue: None,
            tracked_outputs: None,
        }
    }

    /// The controller name this context acts as.
    pub fn controller(&self) -> &str {
        &self.controller
    }

    /// Number of mutating writes performed so far this pass.
    pub fn writes(&self) -> usize {
        self.writes
    }

    /// Get a clone of an input/any resource by key.
    pub fn get(&self, key: &str) -> Option<AnyResource> {
        self.state.get(key)
    }

    /// Whether a resource exists.
    pub fn contains(&self, key: &str) -> bool {
        self.state.contains(key)
    }

    /// List resources of an input kind, optionally filtered by labels.
    pub fn list(
        &self,
        kind: &ResourceKind,
        selector: Option<&crate::metadata::Labels>,
    ) -> Vec<AnyResource> {
        self.state.list(kind, selector)
    }

    fn check_output(&self, kind: &ResourceKind) -> Result<(), ControllerError> {
        if !self.spec.can_write(kind) {
            return Err(ControllerError::UndeclaredOutput(kind.to_string()));
        }
        Ok(())
    }

    fn track_output_write(&mut self, key: &str) {
        if let Some(tracked) = &mut self.tracked_outputs {
            tracked.remove(key);
        }
    }

    /// Start tracking existing exclusive outputs owned by this controller.
    ///
    /// This mirrors COSI `StartTrackingOutputs`: a reconcile pass snapshots the
    /// controller-owned outputs that existed before the pass, ordinary writes
    /// mark outputs as touched, and [`cleanup_outputs`](Self::cleanup_outputs)
    /// removes the stale snapshot leftovers.
    pub fn start_tracking_outputs(&mut self) {
        let mut tracked = BTreeSet::new();
        for output in self.spec.outputs() {
            if output.is_shared() {
                continue;
            }
            for resource in self.state.list(output.kind(), None) {
                if resource.metadata().owner() == self.controller {
                    tracked.insert(resource.metadata().key());
                }
            }
        }
        self.tracked_outputs = Some(tracked);
    }

    /// Tear down and destroy tracked exclusive outputs that were not touched
    /// since [`start_tracking_outputs`](Self::start_tracking_outputs).
    ///
    /// Returns the number of stale outputs cleanup acted on. Resources that
    /// disappeared, changed kind, are no longer declared outputs, or are owned
    /// by a different controller are skipped for safety.
    pub fn cleanup_outputs(&mut self) -> StoreResult<usize> {
        let Some(tracked) = self.tracked_outputs.take() else {
            return Ok(0);
        };

        let mut cleaned = 0;
        for key in tracked {
            let Some(current) = self.state.get(&key) else {
                continue;
            };
            let kind = current.resource_kind();
            if let Err(e) = self.check_output(&kind) {
                return Err(undeclared_to_store(e));
            }
            if !self.spec.is_exclusive_output(&kind)
                || current.metadata().owner() != self.controller
            {
                continue;
            }

            cleaned += 1;
            let teardown_version = current.metadata().version();
            let post_teardown_version = self.state.teardown(&key, teardown_version)?;
            self.writes += 1;

            let Some(after_teardown) = self.state.get(&key) else {
                continue;
            };
            if after_teardown.metadata().can_destroy() {
                self.state.destroy(&key, post_teardown_version)?;
                self.writes += 1;
            }
        }

        Ok(cleaned)
    }

    /// Create a new output resource. Fails if the kind was not declared as an
    /// output. Exclusive outputs are stamped with this controller as owner.
    pub fn create(&mut self, mut resource: AnyResource) -> StoreResult<u64> {
        let kind = resource.resource_kind();
        if let Err(e) = self.check_output(&kind) {
            return Err(undeclared_to_store(e));
        }
        if self.spec.is_exclusive_output(&kind) && resource.metadata().owner().is_empty() {
            resource.metadata_mut().set_owner(self.controller.clone());
        }
        let key = resource.metadata().key();
        let v = self.state.create(resource)?;
        self.writes += 1;
        self.track_output_write(&key);
        Ok(v)
    }

    /// Update an existing output resource with optimistic concurrency.
    pub fn update(&mut self, mut resource: AnyResource, expected_version: u64) -> StoreResult<u64> {
        let kind = resource.resource_kind();
        if let Err(e) = self.check_output(&kind) {
            return Err(undeclared_to_store(e));
        }
        if self.spec.is_exclusive_output(&kind) && resource.metadata().owner().is_empty() {
            resource.metadata_mut().set_owner(self.controller.clone());
        }
        let key = resource.metadata().key();
        let v = self.state.update(resource, expected_version)?;
        // Only count a real mutation: a no-op update returns the same version.
        if v != expected_version {
            self.writes += 1;
        }
        self.track_output_write(&key);
        Ok(v)
    }

    /// Read-modify-write helper: fetch `key`, apply `f`, and update with the
    /// current version. A no-op `f` produces a no-op update. Returns the new
    /// version, or `NotFound` if the resource is gone.
    pub fn modify(&mut self, key: &str, f: impl FnOnce(&mut AnyResource)) -> StoreResult<u64> {
        let mut current = self
            .state
            .get(key)
            .ok_or_else(|| StoreError::NotFound(key.to_string()))?;
        let version = current.metadata().version();
        let kind = current.resource_kind();
        if let Err(e) = self.check_output(&kind) {
            return Err(undeclared_to_store(e));
        }
        f(&mut current);
        if self.spec.is_exclusive_output(&kind) && current.metadata().owner().is_empty() {
            current.metadata_mut().set_owner(self.controller.clone());
        }
        let v = self.state.update(current, version)?;
        // Only count a real mutation: a no-op update returns the same version.
        if v != version {
            self.writes += 1;
        }
        self.track_output_write(key);
        Ok(v)
    }

    /// Add this controller's finalizer to a strong input resource. Idempotent.
    /// Only counts as a write when the finalizer was newly added.
    pub fn add_finalizer(&mut self, key: &str) -> StoreResult<()> {
        let name = self.controller.clone();
        let before = self.state.get(key).map(|r| r.metadata().version());
        self.state.add_finalizer(key, &name)?;
        let after = self.state.get(key).map(|r| r.metadata().version());
        if before != after {
            self.writes += 1;
        }
        Ok(())
    }

    /// Remove this controller's finalizer (done cleaning up). Idempotent.
    /// Only counts as a write when the finalizer was actually present.
    pub fn remove_finalizer(&mut self, key: &str) -> StoreResult<()> {
        let name = self.controller.clone();
        let before = self.state.get(key).map(|r| r.metadata().version());
        self.state.remove_finalizer(key, &name)?;
        let after = self.state.get(key).map(|r| r.metadata().version());
        if before != after {
            self.writes += 1;
        }
        Ok(())
    }

    /// Request teardown of an owned output resource.
    pub fn teardown(&mut self, key: &str) -> StoreResult<u64> {
        let current = self
            .state
            .get(key)
            .ok_or_else(|| StoreError::NotFound(key.to_string()))?;
        let kind = current.resource_kind();
        if let Err(e) = self.check_output(&kind) {
            return Err(undeclared_to_store(e));
        }
        let version = current.metadata().version();
        let v = self.state.teardown(key, version)?;
        self.writes += 1;
        self.track_output_write(key);
        Ok(v)
    }

    /// Destroy an output resource that is tearing down with no finalizers.
    pub fn destroy(&mut self, key: &str) -> StoreResult<()> {
        let current = self
            .state
            .get(key)
            .ok_or_else(|| StoreError::NotFound(key.to_string()))?;
        let kind = current.resource_kind();
        if let Err(e) = self.check_output(&kind) {
            return Err(undeclared_to_store(e));
        }
        let version = current.metadata().version();
        let r = self.state.destroy(key, version);
        if r.is_ok() {
            self.writes += 1;
            self.track_output_write(key);
        }
        r
    }

    /// Whether `key` (a resource the controller observes) is tearing down.
    pub fn is_tearing_down(&self, key: &str) -> bool {
        self.state
            .get(key)
            .is_some_and(|r| r.metadata().phase() == Phase::TearingDown)
    }

    /// Ask the runtime to reconcile this controller again later.
    pub fn requeue(&mut self, reason: impl Into<String>) {
        self.requeue = Some(reason.into());
    }

    /// Take the requeue request, if any, consuming it.
    pub fn take_requeue(&mut self) -> Option<String> {
        self.requeue.take()
    }
}

fn undeclared_to_store(e: ControllerError) -> StoreError {
    match e {
        ControllerError::UndeclaredOutput(k) => StoreError::OwnerConflict {
            key: k,
            owner: "<undeclared-output>".to_string(),
        },
        ControllerError::Store(s) => s,
        other => StoreError::NotFound(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inputs::{Input, Output};
    use crate::metadata::Metadata;
    use crate::resource::Resource;
    use os_kernel::ResourceId;

    #[derive(Debug, Clone)]
    struct Status {
        meta: Metadata,
        ready: bool,
    }
    impl Status {
        fn new(id: &str, ready: bool) -> Self {
            Status {
                meta: Metadata::new("default", "Status", ResourceId::new(id).unwrap()),
                ready,
            }
        }
        fn boxed(self) -> AnyResource {
            Box::new(self)
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
            format!("ready={}", self.ready)
        }
        fn clone_box(&self) -> AnyResource {
            Box::new(self.clone())
        }
    }

    fn status_kind() -> ResourceKind {
        ResourceKind::new("default", "Status")
    }
    fn config_kind() -> ResourceKind {
        ResourceKind::new("default", "Config")
    }

    fn spec() -> Spec {
        Spec::new()
            .with_input(Input::strong(config_kind()))
            .with_output(Output::exclusive(status_kind()))
    }

    #[test]
    fn create_stamps_owner_on_exclusive_output() {
        let mut state = State::new();
        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        ctx.create(Status::new("a", false).boxed()).unwrap();
        assert_eq!(ctx.writes(), 1);
        let r = state.get("default/Status/a").unwrap();
        assert_eq!(r.metadata().owner(), "status-ctrl");
    }

    #[test]
    fn writing_undeclared_output_is_rejected() {
        let mut state = State::new();
        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        // Config is an input, not an output -> cannot create it.
        let cfg = Status {
            meta: Metadata::new("default", "Config", ResourceId::new("x").unwrap()),
            ready: false,
        };
        let err = ctx.create(Box::new(cfg)).unwrap_err();
        assert!(matches!(err, StoreError::OwnerConflict { .. }));
    }

    #[test]
    fn modify_read_write_bumps_version() {
        let mut state = State::new();
        {
            let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
            ctx.create(Status::new("a", false).boxed()).unwrap();
        }
        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        let v = ctx
            .modify("default/Status/a", |r| {
                // flip ready via a fresh value with same identity
                let cur = r.metadata().clone();
                *r = Box::new(Status {
                    meta: cur,
                    ready: true,
                });
            })
            .unwrap();
        assert_eq!(v, 2);
    }

    #[test]
    fn finalizer_helpers_use_controller_name() {
        let mut state = State::new();
        // create a Config (an input) directly in the store
        let cfg = Status {
            meta: Metadata::new("default", "Config", ResourceId::new("c").unwrap()),
            ready: false,
        };
        state.create(Box::new(cfg)).unwrap();
        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        ctx.add_finalizer("default/Config/c").unwrap();
        let r = state.get("default/Config/c").unwrap();
        assert!(r.metadata().finalizers().contains("status-ctrl"));
    }

    #[test]
    fn teardown_then_destroy_owned_output() {
        let mut state = State::new();
        {
            let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
            ctx.create(Status::new("a", true).boxed()).unwrap();
        }
        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        assert!(!ctx.is_tearing_down("default/Status/a"));
        ctx.teardown("default/Status/a").unwrap();
        assert!(ctx.is_tearing_down("default/Status/a"));
        ctx.destroy("default/Status/a").unwrap();
        assert!(!state.contains("default/Status/a"));
    }

    #[test]
    fn cleanup_outputs_destroys_tracked_owned_outputs_not_touched() {
        let mut state = State::new();
        {
            let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
            ctx.create(Status::new("stale", true).boxed()).unwrap();
        }

        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        ctx.start_tracking_outputs();
        let cleaned = ctx.cleanup_outputs().unwrap();
        assert_eq!(cleaned, 1);
        assert_eq!(ctx.writes(), 2);
        assert!(!ctx.contains("default/Status/stale"));
    }

    #[test]
    fn cleanup_outputs_keeps_touched_outputs() {
        let mut state = State::new();
        {
            let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
            ctx.create(Status::new("current", false).boxed()).unwrap();
        }

        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        ctx.start_tracking_outputs();
        ctx.modify("default/Status/current", |r| {
            let meta = r.metadata().clone();
            *r = Box::new(Status { meta, ready: true });
        })
        .unwrap();

        let cleaned = ctx.cleanup_outputs().unwrap();
        assert_eq!(cleaned, 0);
        assert!(ctx.contains("default/Status/current"));
    }

    #[test]
    fn cleanup_outputs_ignores_outputs_owned_by_other_controller() {
        let mut state = State::new();
        let mut foreign = Status::new("foreign", true);
        foreign.meta.set_owner("other-ctrl");
        state.create(foreign.boxed()).unwrap();

        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        ctx.start_tracking_outputs();
        let cleaned = ctx.cleanup_outputs().unwrap();

        assert_eq!(cleaned, 0);
        assert!(ctx.contains("default/Status/foreign"));
    }

    #[test]
    fn update_claims_empty_owner_exclusive_output() {
        let mut state = State::new();
        state.create(Status::new("a", false).boxed()).unwrap();

        let mut ctx = ReconcileContext::new(&mut state, "status-ctrl", spec());
        let version = ctx
            .update(Status::new("a", true).boxed(), 1)
            .expect("context update should claim empty-owner exclusive output");

        assert_eq!(version, 2);
        let stored = ctx.get("default/Status/a").unwrap();
        assert_eq!(stored.metadata().owner(), "status-ctrl");
        assert_eq!(ctx.writes(), 1);
    }

    #[test]
    fn requeue_is_recorded_and_taken() {
        let mut state = State::new();
        let mut ctx = ReconcileContext::new(&mut state, "c", spec());
        assert!(ctx.take_requeue().is_none());
        ctx.requeue("waiting for config");
        assert_eq!(ctx.take_requeue().as_deref(), Some("waiting for config"));
        assert!(ctx.take_requeue().is_none());
    }
}
