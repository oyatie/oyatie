//! The COSI controller wiring for machined.
//!
//! Mirrors `siderolabs/talos`'s use of the COSI runtime: machined registers a
//! set of controllers, each declaring the resources it consumes (inputs) and
//! produces (outputs). The [`ControllerRuntime`] resolves a startup order that
//! respects those dependencies (a topological sort), refusing to start if a
//! required input is produced by no registered controller or if there is a
//! dependency cycle.

use crate::error::{MachinedError, Result};
use os_cosi_domain::Controller;
use std::collections::{HashMap, HashSet};

/// A stable identifier for a controller.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControllerId(String);

impl ControllerId {
    /// Build a controller id.
    pub fn new(s: impl Into<String>) -> Self {
        ControllerId(s.into())
    }

    /// The id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for ControllerId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ControllerId {
    fn from(s: &str) -> Self {
        ControllerId(s.to_string())
    }
}

/// A machined controller declaration.
///
/// A controller reconciles a set of output resource kinds from a set of input
/// resource kinds. Resource kinds are modeled as plain strings (mirroring COSI
/// `resource.Type`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachinedController {
    id: ControllerId,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl MachinedController {
    /// Declare a controller with its id, inputs and outputs.
    pub fn new(id: impl Into<ControllerId>, inputs: Vec<&str>, outputs: Vec<&str>) -> Self {
        MachinedController {
            id: id.into(),
            inputs: inputs
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
            outputs: outputs
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }

    /// Declare a machined startup controller from a ported COSI controller.
    ///
    /// Source Talos registers concrete `controller.Controller` instances during
    /// v1alpha2 startup. Only strong COSI inputs participate in the controller
    /// dependency graph; weak and destroy-ready inputs are watches/cleanup
    /// surfaces and must not block startup ordering.
    pub fn from_cosi_controller(controller: &impl Controller) -> Self {
        let spec = controller.spec();
        MachinedController {
            id: ControllerId::new(controller.name()),
            inputs: spec
                .strong_input_kinds()
                .into_iter()
                .map(|kind| kind.to_string())
                .collect(),
            outputs: spec
                .outputs()
                .iter()
                .map(|output| output.kind().to_string())
                .collect(),
        }
    }

    /// The controller id.
    pub fn id(&self) -> &ControllerId {
        &self.id
    }

    /// The resource kinds this controller consumes.
    pub fn inputs(&self) -> &[String] {
        &self.inputs
    }

    /// The resource kinds this controller produces.
    pub fn outputs(&self) -> &[String] {
        &self.outputs
    }
}

impl From<String> for ControllerId {
    fn from(s: String) -> Self {
        ControllerId(s)
    }
}

/// The registry that wires controllers together and computes a valid startup
/// order.
#[derive(Debug, Default)]
pub struct ControllerRuntime {
    controllers: Vec<MachinedController>,
}

impl ControllerRuntime {
    /// Create an empty controller runtime.
    pub fn new() -> Self {
        ControllerRuntime {
            controllers: Vec::new(),
        }
    }

    /// Register a controller. Duplicate ids are rejected.
    pub fn register(&mut self, controller: MachinedController) -> Result<()> {
        if self.controllers.iter().any(|c| c.id() == controller.id()) {
            return Err(MachinedError::DependencyUnmet(format!(
                "controller '{}' already registered",
                controller.id()
            )));
        }
        self.controllers.push(controller);
        Ok(())
    }

    /// Number of registered controllers.
    pub fn len(&self) -> usize {
        self.controllers.len()
    }

    /// Whether no controllers are registered.
    pub fn is_empty(&self) -> bool {
        self.controllers.is_empty()
    }

    /// Look up a controller by id.
    pub fn get(&self, id: &ControllerId) -> Option<&MachinedController> {
        self.controllers.iter().find(|c| c.id() == id)
    }

    /// Validate that every input kind is produced by some registered
    /// controller. Returns the list of unsatisfied input kinds (empty = ok).
    pub fn unsatisfied_inputs(&self) -> Vec<String> {
        let produced: HashSet<&str> = self
            .controllers
            .iter()
            .flat_map(|c| c.outputs().iter().map(std::string::String::as_str))
            .collect();
        let mut missing = Vec::new();
        for c in &self.controllers {
            for input in c.inputs() {
                if !produced.contains(input.as_str()) && !missing.contains(input) {
                    missing.push(input.clone());
                }
            }
        }
        missing
    }

    /// Compute a startup order that respects input/output dependencies: a
    /// controller appears after every controller producing one of its inputs.
    ///
    /// Returns [`MachinedError::DependencyUnmet`] if an input is produced by no
    /// controller, and [`MachinedError::IllegalTransition`] if there is a cycle.
    pub fn startup_order(&self) -> Result<Vec<ControllerId>> {
        let missing = self.unsatisfied_inputs();
        if !missing.is_empty() {
            return Err(MachinedError::DependencyUnmet(format!(
                "no controller produces inputs: {}",
                missing.join(", ")
            )));
        }

        // Map each output kind to the controllers that produce it.
        let mut producers: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, c) in self.controllers.iter().enumerate() {
            for out in c.outputs() {
                producers.entry(out.as_str()).or_default().push(i);
            }
        }

        // Build edges: producer -> consumer.
        let n = self.controllers.len();
        let mut indegree = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (consumer, c) in self.controllers.iter().enumerate() {
            for input in c.inputs() {
                if let Some(prods) = producers.get(input.as_str()) {
                    for &producer in prods {
                        if producer == consumer {
                            continue; // a controller may consume its own output
                        }
                        adj[producer].push(consumer);
                        indegree[consumer] += 1;
                    }
                }
            }
        }

        // Kahn's algorithm, breaking ties by registration order for determinism.
        let mut queue: Vec<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
        queue.sort_unstable();
        let mut order = Vec::with_capacity(n);
        let mut head = 0;
        while head < queue.len() {
            let node = queue[head];
            head += 1;
            order.push(ControllerId(self.controllers[node].id().0.clone()));
            let mut newly_ready = Vec::new();
            for &next in &adj[node] {
                indegree[next] -= 1;
                if indegree[next] == 0 {
                    newly_ready.push(next);
                }
            }
            newly_ready.sort_unstable();
            for nr in newly_ready {
                queue.push(nr);
            }
        }

        if order.len() != n {
            return Err(MachinedError::illegal_transition(
                "controller-graph",
                "acyclic-order",
            ));
        }
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctrl(id: &str, inputs: Vec<&str>, outputs: Vec<&str>) -> MachinedController {
        MachinedController::new(id, inputs, outputs)
    }

    #[test]
    fn registers_and_rejects_duplicates() {
        let mut rt = ControllerRuntime::new();
        rt.register(ctrl("A", vec![], vec!["x"])).unwrap();
        assert_eq!(rt.len(), 1);
        let err = rt.register(ctrl("A", vec![], vec!["y"])).unwrap_err();
        assert_eq!(err.kind(), "dependency_unmet");
    }

    #[test]
    fn topological_order_respects_dependencies() {
        let mut rt = ControllerRuntime::new();
        // config -> network -> kubelet
        rt.register(ctrl(
            "kubelet",
            vec!["NetworkStatus"],
            vec!["KubeletStatus"],
        ))
        .unwrap();
        rt.register(ctrl(
            "network",
            vec!["MachineConfig"],
            vec!["NetworkStatus"],
        ))
        .unwrap();
        rt.register(ctrl("config", vec![], vec!["MachineConfig"]))
            .unwrap();
        let order = rt.startup_order().unwrap();
        let pos = |id: &str| order.iter().position(|c| c.as_str() == id).unwrap();
        assert!(pos("config") < pos("network"));
        assert!(pos("network") < pos("kubelet"));
    }

    #[test]
    fn missing_input_is_unsatisfied() {
        let mut rt = ControllerRuntime::new();
        rt.register(ctrl(
            "kubelet",
            vec!["NetworkStatus"],
            vec!["KubeletStatus"],
        ))
        .unwrap();
        let missing = rt.unsatisfied_inputs();
        assert_eq!(missing, vec!["NetworkStatus".to_string()]);
        let err = rt.startup_order().unwrap_err();
        assert_eq!(err.kind(), "dependency_unmet");
    }

    #[test]
    fn cycle_is_detected() {
        let mut rt = ControllerRuntime::new();
        rt.register(ctrl("A", vec!["y"], vec!["x"])).unwrap();
        rt.register(ctrl("B", vec!["x"], vec!["y"])).unwrap();
        let err = rt.startup_order().unwrap_err();
        assert_eq!(err.kind(), "illegal_transition");
    }

    #[test]
    fn self_referential_output_is_allowed() {
        let mut rt = ControllerRuntime::new();
        // A consumes and produces the same kind (e.g. a status reconciler).
        rt.register(ctrl("A", vec!["x"], vec!["x"])).unwrap();
        let order = rt.startup_order().unwrap();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0].as_str(), "A");
    }

    #[test]
    fn volume_config_startup_declaration_uses_only_strong_inputs_for_dependencies() {
        let controller = os_block_domain::VolumeConfigController::new_in_container();
        let declaration = MachinedController::from_cosi_controller(&controller);
        assert_eq!(
            declaration.id().as_str(),
            os_block_domain::VOLUME_CONFIG_CONTROLLER_NAME
        );
        assert!(
            declaration.inputs().is_empty(),
            "VolumeConfigController source inputs are weak/destroy-ready and should not block startup"
        );
        assert_eq!(
            declaration.outputs(),
            &[
                "runtime/VolumeConfigs.block.talos.dev".to_string(),
                "runtime/VolumeMountRequests.block.talos.dev".to_string()
            ]
        );

        let mut rt = ControllerRuntime::new();
        rt.register(declaration).unwrap();
        assert!(rt.unsatisfied_inputs().is_empty());
        assert_eq!(
            rt.startup_order().unwrap()[0].as_str(),
            os_block_domain::VOLUME_CONFIG_CONTROLLER_NAME
        );
    }

    #[test]
    fn cri_registries_config_startup_declaration_uses_only_strong_inputs_for_dependencies() {
        let controller = os_runtime_cri_domain::RegistriesConfigController::new();
        let declaration = MachinedController::from_cosi_controller(&controller);
        assert_eq!(
            declaration.id().as_str(),
            os_runtime_cri_domain::REGISTRIES_CONFIG_CONTROLLER_NAME
        );
        assert!(
            declaration.inputs().is_empty(),
            "RegistriesConfigController source inputs are weak and should not block startup"
        );
        assert_eq!(
            declaration.outputs(),
            &[os_runtime_cri_domain::RegistriesConfigResource::kind().to_string()]
        );
    }
}
