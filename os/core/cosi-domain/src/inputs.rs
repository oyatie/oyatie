//! Controller [`Input`]/[`Output`] declarations. Mirrors COSI
//! `controller.Input` / `controller.Output` and the `Inputs()`/`Outputs()`
//! surface of `controller.Controller`.
//!
//! A controller declares, statically, which resource kinds it reads (its
//! *inputs*) and which kinds it writes (its *outputs*). The runtime uses these
//! declarations to:
//!
//! - build the dependency graph between controllers (an edge `A -> B` exists
//!   when `A` outputs a kind that `B` takes as a strong input);
//! - register the watches needed to wake a controller when one of its inputs
//!   changes;
//! - enforce write access (a controller may only create/update/destroy kinds it
//!   declared as an output, and only with the matching ownership semantics).

use crate::resource::ResourceKind;
use core::fmt;

/// How a controller depends on an input kind. Mirrors COSI's
/// `controller.InputKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputKind {
    /// A strong input: the controller is reconciled whenever the input changes
    /// and the input participates in the dependency graph / teardown ordering.
    /// A resource that is a strong input of a controller cannot be destroyed
    /// until that controller removes its finalizer.
    Strong,
    /// A weak input: the controller is reconciled on change but the input does
    /// *not* hold up teardown and does not add finalizers.
    Weak,
    /// A destroy-ready input: like [`InputKind::Strong`] but used to observe
    /// resources the controller owns and must tear down; the controller may
    /// destroy these once they are ready.
    DestroyReady,
}

impl InputKind {
    /// Stable lowercase name.
    pub fn as_str(&self) -> &'static str {
        match self {
            InputKind::Strong => "strong",
            InputKind::Weak => "weak",
            InputKind::DestroyReady => "destroy-ready",
        }
    }

    /// Whether this input participates in finalizer-based teardown ordering.
    /// Only strong inputs add finalizers and block destruction.
    pub fn adds_finalizer(&self) -> bool {
        matches!(self, InputKind::Strong)
    }
}

impl fmt::Display for InputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single declared input: a resource kind plus optionally a specific id and
/// the [`InputKind`] strength. Mirrors COSI `controller.Input`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    kind: ResourceKind,
    /// Optional specific id; `None` means "all resources of this kind".
    id: Option<String>,
    strength: InputKind,
}

impl Input {
    /// A strong input watching all resources of a kind.
    pub fn strong(kind: ResourceKind) -> Self {
        Input {
            kind,
            id: None,
            strength: InputKind::Strong,
        }
    }

    /// A weak input watching all resources of a kind.
    pub fn weak(kind: ResourceKind) -> Self {
        Input {
            kind,
            id: None,
            strength: InputKind::Weak,
        }
    }

    /// A destroy-ready input watching all resources of a kind.
    pub fn destroy_ready(kind: ResourceKind) -> Self {
        Input {
            kind,
            id: None,
            strength: InputKind::DestroyReady,
        }
    }

    /// Narrow this input to a single resource id.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// The watched kind.
    pub fn kind(&self) -> &ResourceKind {
        &self.kind
    }

    /// The specific id, if narrowed.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The input strength.
    pub fn strength(&self) -> InputKind {
        self.strength
    }

    /// Whether this input matches a concrete `(kind, id)` pair.
    pub fn matches(&self, kind: &ResourceKind, id: &str) -> bool {
        if &self.kind != kind {
            return false;
        }
        match &self.id {
            Some(want) => want == id,
            None => true,
        }
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "{}[{}]/{}", self.kind, id, self.strength),
            None => write!(f, "{}/{}", self.kind, self.strength),
        }
    }
}

/// A declared output: a resource kind the controller is allowed to write.
/// Mirrors COSI `controller.Output`. The `shared` flag distinguishes the two
/// COSI output kinds: an *exclusive* output may be written only by this
/// controller (which becomes the owner), while a *shared* output may be written
/// by several controllers cooperatively.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    kind: ResourceKind,
    shared: bool,
}

impl Output {
    /// An exclusive output: only this controller owns and writes this kind.
    pub fn exclusive(kind: ResourceKind) -> Self {
        Output {
            kind,
            shared: false,
        }
    }

    /// A shared output: multiple controllers may write this kind.
    pub fn shared(kind: ResourceKind) -> Self {
        Output { kind, shared: true }
    }

    /// The output kind.
    pub fn kind(&self) -> &ResourceKind {
        &self.kind
    }

    /// Whether this is a shared (cooperative) output.
    pub fn is_shared(&self) -> bool {
        self.shared
    }

    /// Whether this is an exclusive (owned) output.
    pub fn is_exclusive(&self) -> bool {
        !self.shared
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = if self.shared { "shared" } else { "exclusive" };
        write!(f, "{}/{}", self.kind, mode)
    }
}

/// The full input/output declaration of a controller.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Spec {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

impl Spec {
    /// An empty spec.
    pub fn new() -> Self {
        Spec {
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Add an input (builder).
    pub fn with_input(mut self, input: Input) -> Self {
        self.add_input(input);
        self
    }

    /// Add an output (builder).
    pub fn with_output(mut self, output: Output) -> Self {
        self.add_output(output);
        self
    }

    /// Add an input. Duplicate `(kind,id)` declarations are rejected by keeping
    /// the strongest one; re-declaring upgrades weak -> strong.
    pub fn add_input(&mut self, input: Input) {
        if let Some(existing) = self
            .inputs
            .iter_mut()
            .find(|i| i.kind() == input.kind() && i.id() == input.id())
        {
            // Upgrade strength: Strong > DestroyReady > Weak.
            if rank(input.strength()) > rank(existing.strength()) {
                *existing = input;
            }
            return;
        }
        self.inputs.push(input);
    }

    /// Add an output, deduplicating by kind (exclusive wins over shared).
    pub fn add_output(&mut self, output: Output) {
        if let Some(existing) = self.outputs.iter_mut().find(|o| o.kind() == output.kind()) {
            if output.is_exclusive() {
                *existing = output;
            }
            return;
        }
        self.outputs.push(output);
    }

    /// All declared inputs.
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// All declared outputs.
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Find the matching input for a concrete `(kind, id)` event, if any.
    pub fn matching_input(&self, kind: &ResourceKind, id: &str) -> Option<&Input> {
        self.inputs.iter().find(|i| i.matches(kind, id))
    }

    /// Whether this controller is allowed to write the given output kind.
    pub fn can_write(&self, kind: &ResourceKind) -> bool {
        self.outputs.iter().any(|o| o.kind() == kind)
    }

    /// Whether the given output kind is declared exclusive.
    pub fn is_exclusive_output(&self, kind: &ResourceKind) -> bool {
        self.outputs
            .iter()
            .find(|o| o.kind() == kind)
            .is_some_and(Output::is_exclusive)
    }

    /// All input kinds (deduplicated) the controller watches.
    pub fn input_kinds(&self) -> Vec<ResourceKind> {
        let mut out: Vec<ResourceKind> = Vec::new();
        for i in &self.inputs {
            if !out.contains(i.kind()) {
                out.push(i.kind().clone());
            }
        }
        out
    }

    /// All strong-input kinds: those participating in teardown ordering.
    pub fn strong_input_kinds(&self) -> Vec<ResourceKind> {
        let mut out: Vec<ResourceKind> = Vec::new();
        for i in &self.inputs {
            if i.strength().adds_finalizer() && !out.contains(i.kind()) {
                out.push(i.kind().clone());
            }
        }
        out
    }
}

fn rank(k: InputKind) -> u8 {
    match k {
        InputKind::Weak => 0,
        InputKind::DestroyReady => 1,
        InputKind::Strong => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(name: &str) -> ResourceKind {
        ResourceKind::new("default", name)
    }

    #[test]
    fn input_strength_finalizer_semantics() {
        assert!(InputKind::Strong.adds_finalizer());
        assert!(!InputKind::Weak.adds_finalizer());
        assert!(!InputKind::DestroyReady.adds_finalizer());
        assert_eq!(InputKind::DestroyReady.as_str(), "destroy-ready");
    }

    #[test]
    fn input_matches_kind_and_optional_id() {
        let any = Input::strong(k("Config"));
        assert!(any.matches(&k("Config"), "a"));
        assert!(any.matches(&k("Config"), "b"));
        assert!(!any.matches(&k("Other"), "a"));

        let one = Input::weak(k("Config")).with_id("a");
        assert_eq!(one.id(), Some("a"));
        assert!(one.matches(&k("Config"), "a"));
        assert!(!one.matches(&k("Config"), "b"));
    }

    #[test]
    fn output_modes() {
        let ex = Output::exclusive(k("Status"));
        let sh = Output::shared(k("Status"));
        assert!(ex.is_exclusive());
        assert!(!ex.is_shared());
        assert!(sh.is_shared());
        assert_eq!(ex.to_string(), "default/Status/exclusive");
    }

    #[test]
    fn spec_dedups_and_upgrades_input_strength() {
        let mut s = Spec::new();
        s.add_input(Input::weak(k("Config")));
        s.add_input(Input::strong(k("Config"))); // upgrade
        assert_eq!(s.inputs().len(), 1);
        assert_eq!(s.inputs()[0].strength(), InputKind::Strong);

        // Re-adding weaker does not downgrade.
        s.add_input(Input::weak(k("Config")));
        assert_eq!(s.inputs()[0].strength(), InputKind::Strong);
    }

    #[test]
    fn spec_dedups_outputs_exclusive_wins() {
        let mut s = Spec::new();
        s.add_output(Output::shared(k("Status")));
        s.add_output(Output::exclusive(k("Status")));
        assert_eq!(s.outputs().len(), 1);
        assert!(s.outputs()[0].is_exclusive());
    }

    #[test]
    fn spec_write_access_and_matching() {
        let s = Spec::new()
            .with_input(Input::strong(k("Config")))
            .with_input(Input::weak(k("Secret")).with_id("token"))
            .with_output(Output::exclusive(k("Status")));
        assert!(s.can_write(&k("Status")));
        assert!(!s.can_write(&k("Config")));
        assert!(s.is_exclusive_output(&k("Status")));
        assert!(s.matching_input(&k("Config"), "anything").is_some());
        assert!(s.matching_input(&k("Secret"), "token").is_some());
        assert!(s.matching_input(&k("Secret"), "other").is_none());
    }

    #[test]
    fn spec_input_kind_views() {
        let s = Spec::new()
            .with_input(Input::strong(k("Config")))
            .with_input(Input::weak(k("Secret")))
            .with_input(Input::strong(k("Config")).with_id("x"));
        // Config appears twice (all + id x) but input_kinds dedups by kind.
        assert_eq!(s.input_kinds().len(), 2);
        // strong kinds: Config only.
        let strong = s.strong_input_kinds();
        assert_eq!(strong.len(), 1);
        assert_eq!(strong[0].kind(), "Config");
    }
}
