#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ResourceState {
    Pending,
    Running,
    Stopped,
    Terminated,
    Error,
}

impl ResourceState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Terminated)
    }

    /// Returns the canonical lowercase string label for this state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Terminated => "terminated",
            Self::Error => "error",
        }
    }

    /// Parses a canonical string label back to a `ResourceState`.
    /// Returns `None` for any unrecognised input (fail-closed).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "stopped" => Some(Self::Stopped),
            "terminated" => Some(Self::Terminated),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// Returns `true` iff the resource is actively consuming compute (`Running`).
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns `true` iff the resource is idle but not destroyed (`Stopped`).
    pub const fn is_quiescent(self) -> bool {
        matches!(self, Self::Stopped)
    }

    /// Returns the ordered slice of legal successor states reachable from `self`
    /// in a single transition, including the self-loop.
    ///
    /// This exposes the transition graph defined by the crate-private
    /// `state_transition_allowed` predicate so callers can introspect reachability
    /// without holding a [`crate::Resource`] reference.
    pub const fn allowed_next(self) -> &'static [Self] {
        match self {
            Self::Pending => &[Self::Pending, Self::Running, Self::Error, Self::Terminated],
            Self::Running => &[Self::Running, Self::Stopped, Self::Error, Self::Terminated],
            Self::Stopped => &[Self::Stopped, Self::Running, Self::Error, Self::Terminated],
            Self::Error => &[Self::Error, Self::Terminated],
            Self::Terminated => &[Self::Terminated],
        }
    }

    /// Pre-checks whether a transition from `self` to `next` is legal without
    /// mutating a [`crate::Resource`]. Delegates to the existing transition predicate.
    pub fn can_transition_to(self, next: Self) -> bool {
        state_transition_allowed(self, next)
    }
}

pub(crate) fn state_transition_allowed(current: ResourceState, next: ResourceState) -> bool {
    current == next
        || matches!(
            (current, next),
            (ResourceState::Pending, ResourceState::Running)
                | (ResourceState::Pending, ResourceState::Error)
                | (ResourceState::Pending, ResourceState::Terminated)
                | (ResourceState::Running, ResourceState::Stopped)
                | (ResourceState::Running, ResourceState::Error)
                | (ResourceState::Running, ResourceState::Terminated)
                | (ResourceState::Stopped, ResourceState::Running)
                | (ResourceState::Stopped, ResourceState::Error)
                | (ResourceState::Stopped, ResourceState::Terminated)
                | (ResourceState::Error, ResourceState::Terminated)
        )
}
