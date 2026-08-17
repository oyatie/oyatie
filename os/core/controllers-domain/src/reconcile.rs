//! The [`Controller`] trait, its input/output declarations, and the shared
//! reconcile glue used by every controller domain.
//!
//! Mirrors `cosi-project/runtime`'s `controller.Controller` /
//! `controller.Runtime` split: a controller declares the kinds it reads
//! (inputs) and the kinds it writes (outputs), then `reconcile` is invoked
//! whenever an input changes. The controller is given a [`ReconcileContext`]
//! through which it reads inputs and modifies outputs in the shared
//! [`State`](os_cosi_domain::State) store.

use os_cosi_domain::resource::{AnyResource, ResourceKind};
use os_cosi_domain::{Metadata, State};
use std::collections::BTreeMap;

/// How a controller depends on an input kind.
///
/// In COSI a `Weak` input only triggers reconciliation, while a `Strong` input
/// additionally guarantees the resource will not be destroyed out from under
/// the controller (the runtime keeps a finalizer). `DestroyReady` is used when
/// a controller must observe teardown to release its own finalizers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// Changes trigger reconcile; no destroy protection.
    Weak,
    /// Changes trigger reconcile and the input is destroy-protected.
    Strong,
    /// The controller wants to observe teardown to clean up finalizers.
    DestroyReady,
}

/// A declared input: the resource kind plus how it is depended upon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// The resource kind being read.
    pub kind: ResourceKind,
    /// The dependency strength.
    pub dependency: InputKind,
}

impl Input {
    /// A weak input on a kind.
    pub fn weak(kind: ResourceKind) -> Self {
        Input {
            kind,
            dependency: InputKind::Weak,
        }
    }

    /// A strong input on a kind.
    pub fn strong(kind: ResourceKind) -> Self {
        Input {
            kind,
            dependency: InputKind::Strong,
        }
    }

    /// A destroy-ready input on a kind.
    pub fn destroy_ready(kind: ResourceKind) -> Self {
        Input {
            kind,
            dependency: InputKind::DestroyReady,
        }
    }
}

/// A declared output: a kind the controller is allowed to create/update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The resource kind being written.
    pub kind: ResourceKind,
}

impl Output {
    /// An output on a kind.
    pub fn new(kind: ResourceKind) -> Self {
        Output { kind }
    }
}

/// Errors surfaced from a reconcile pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileError {
    /// The controller attempted to write a kind it did not declare as output.
    UndeclaredOutput(String),
    /// A required input resource was missing.
    MissingInput(String),
    /// The underlying store rejected an operation.
    Store(String),
    /// The controller produced an invalid spec.
    Invalid(String),
    /// Reconciliation could not complete and should be retried.
    Requeue(String),
}

impl std::fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconcileError::UndeclaredOutput(k) => write!(f, "write to undeclared output kind {k}"),
            ReconcileError::MissingInput(k) => write!(f, "missing required input {k}"),
            ReconcileError::Store(m) => write!(f, "store error: {m}"),
            ReconcileError::Invalid(m) => write!(f, "invalid: {m}"),
            ReconcileError::Requeue(m) => write!(f, "requeue: {m}"),
        }
    }
}

impl std::error::Error for ReconcileError {}

/// Result alias for reconcile passes.
pub type ReconcileResult<T> = Result<T, ReconcileError>;

/// The context handed to a controller during a reconcile pass.
///
/// It wraps the shared [`State`] store and the set of output kinds the calling
/// controller declared, enforcing that the controller only writes kinds it
/// owns. Writes are tagged with the controller's name as owner.
pub struct ReconcileContext<'a> {
    state: &'a mut State,
    owner: String,
    outputs: Vec<ResourceKind>,
    /// Set to `true` by [`ReconcileContext::requeue`] to signal the runtime
    /// that the controller wants to be invoked again.
    requeue: bool,
}

impl<'a> ReconcileContext<'a> {
    /// Build a context for `owner` over `state`, permitting writes to
    /// `outputs`.
    pub fn new(state: &'a mut State, owner: impl Into<String>, outputs: Vec<ResourceKind>) -> Self {
        ReconcileContext {
            state,
            owner: owner.into(),
            outputs,
            requeue: false,
        }
    }

    /// The owning controller name.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Read access to the underlying store.
    pub fn state(&self) -> &State {
        self.state
    }

    /// Fetch a single input resource by key (`namespace/kind/id`).
    pub fn get(&self, key: &str) -> Option<AnyResource> {
        self.state.get(key)
    }

    /// List every resource of an input kind.
    pub fn list(&self, kind: &ResourceKind) -> Vec<AnyResource> {
        self.state.list(kind, None)
    }

    /// Whether `kind` is a declared output of this controller.
    fn is_output(&self, kind: &ResourceKind) -> bool {
        self.outputs.iter().any(|k| k == kind)
    }

    /// Create or update an output resource. The resource's metadata is stamped
    /// with this controller as owner. Fails if the kind was not declared as an
    /// output.
    pub fn write(&mut self, mut resource: AnyResource) -> ReconcileResult<u64> {
        let kind = resource.resource_kind();
        if !self.is_output(&kind) {
            return Err(ReconcileError::UndeclaredOutput(kind.to_string()));
        }
        resource.metadata_mut().set_owner(self.owner.clone());
        let key = resource.metadata().key();
        match self.state.get(&key) {
            None => self
                .state
                .create(resource)
                .map_err(|e| ReconcileError::Store(e.to_string())),
            Some(existing) => {
                let version = existing.metadata().version();
                self.state
                    .update(resource, version)
                    .map_err(|e| ReconcileError::Store(e.to_string()))
            }
        }
    }

    /// Tear down then destroy an owned output by key. Used when the desired
    /// state no longer contains a resource the controller previously produced.
    pub fn destroy(&mut self, key: &str) -> ReconcileResult<()> {
        let existing = match self.state.get(key) {
            Some(r) => r,
            None => return Ok(()),
        };
        let kind = existing.resource_kind();
        if !self.is_output(&kind) {
            return Err(ReconcileError::UndeclaredOutput(kind.to_string()));
        }
        let version = existing.metadata().version();
        let teardown_version = self
            .state
            .teardown(key, version)
            .map_err(|e| ReconcileError::Store(e.to_string()))?;
        self.state
            .destroy(key, teardown_version)
            .map_err(|e| ReconcileError::Store(e.to_string()))
    }

    /// Request that the runtime invoke this controller again.
    pub fn requeue(&mut self, reason: impl Into<String>) {
        let _ = reason;
        self.requeue = true;
    }

    /// Whether a requeue was requested during this pass.
    pub fn requeue_requested(&self) -> bool {
        self.requeue
    }
}

/// A COSI controller: declares inputs and outputs and reconciles desired from
/// observed state.
pub trait Controller {
    /// Stable controller name, also used as the resource owner string.
    fn name(&self) -> &str;

    /// The input kinds (with dependency strength) this controller reads.
    fn inputs(&self) -> Vec<Input>;

    /// The output kinds this controller may write.
    fn outputs(&self) -> Vec<Output>;

    /// Reconcile a single pass. Implementations read inputs via `ctx` and
    /// create/update/destroy outputs.
    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()>;
}

/// Convenience: a metadata builder used by output specs across the crate.
pub fn output_metadata(namespace: &str, kind: &str, id: os_kernel::ResourceId) -> Metadata {
    Metadata::new(namespace, kind, id)
}

/// A read-only index of input keys to their fingerprints, used by controllers
/// (and the runtime) to detect whether a reconcile pass is needed.
#[derive(Debug, Default, Clone)]
pub struct InputDigest {
    fingerprints: BTreeMap<String, String>,
}

impl InputDigest {
    /// An empty digest.
    pub fn new() -> Self {
        InputDigest {
            fingerprints: BTreeMap::new(),
        }
    }

    /// Record `key`'s fingerprint, returning `true` if it changed from the
    /// previously recorded value (or is new).
    pub fn record(&mut self, key: &str, fingerprint: &str) -> bool {
        match self.fingerprints.get(key) {
            Some(prev) if prev == fingerprint => false,
            _ => {
                self.fingerprints
                    .insert(key.to_string(), fingerprint.to_string());
                true
            }
        }
    }

    /// Number of tracked keys.
    pub fn len(&self) -> usize {
        self.fingerprints.len()
    }

    /// Whether nothing is tracked.
    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::ResourceId;

    #[derive(Debug, Clone)]
    struct Out {
        meta: Metadata,
        value: u32,
    }

    impl Out {
        fn new(id: &str, value: u32) -> Self {
            Out {
                meta: Metadata::new("runtime", "Out", ResourceId::new(id).unwrap()),
                value,
            }
        }
    }

    impl os_cosi_domain::Resource for Out {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }
        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }
        fn spec_fingerprint(&self) -> String {
            format!("value={}", self.value)
        }
        fn clone_box(&self) -> Box<dyn os_cosi_domain::Resource> {
            Box::new(self.clone())
        }
    }

    fn out_kind() -> ResourceKind {
        ResourceKind::new("runtime", "Out")
    }

    #[test]
    fn write_rejects_undeclared_output() {
        let mut state = State::new();
        let mut ctx = ReconcileContext::new(&mut state, "ctrl", vec![]);
        let err = ctx.write(Box::new(Out::new("a", 1))).unwrap_err();
        assert!(matches!(err, ReconcileError::UndeclaredOutput(_)));
    }

    #[test]
    fn write_creates_then_updates_and_stamps_owner() {
        let mut state = State::new();
        {
            let mut ctx = ReconcileContext::new(&mut state, "ctrl", vec![out_kind()]);
            // Fresh resources start at version 1 in this COSI store.
            assert_eq!(ctx.write(Box::new(Out::new("a", 1))).unwrap(), 1);
        }
        let stored = state.get("runtime/Out/a").unwrap();
        assert_eq!(stored.metadata().owner(), "ctrl");
        {
            let mut ctx = ReconcileContext::new(&mut state, "ctrl", vec![out_kind()]);
            let v = ctx.write(Box::new(Out::new("a", 2))).unwrap();
            assert_eq!(v, 2);
        }
    }

    #[test]
    fn destroy_tears_down_and_removes() {
        let mut state = State::new();
        {
            let mut ctx = ReconcileContext::new(&mut state, "ctrl", vec![out_kind()]);
            ctx.write(Box::new(Out::new("a", 1))).unwrap();
        }
        assert!(state.contains("runtime/Out/a"));
        {
            let mut ctx = ReconcileContext::new(&mut state, "ctrl", vec![out_kind()]);
            ctx.destroy("runtime/Out/a").unwrap();
        }
        assert!(!state.contains("runtime/Out/a"));
    }

    #[test]
    fn destroy_missing_is_noop() {
        let mut state = State::new();
        let mut ctx = ReconcileContext::new(&mut state, "ctrl", vec![out_kind()]);
        assert!(ctx.destroy("runtime/Out/missing").is_ok());
    }

    #[test]
    fn input_digest_detects_changes() {
        let mut d = InputDigest::new();
        assert!(d.record("k", "v1"));
        assert!(!d.record("k", "v1"));
        assert!(d.record("k", "v2"));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn requeue_flag_propagates() {
        let mut state = State::new();
        let mut ctx = ReconcileContext::new(&mut state, "ctrl", vec![]);
        assert!(!ctx.requeue_requested());
        ctx.requeue("not ready");
        assert!(ctx.requeue_requested());
    }
}
