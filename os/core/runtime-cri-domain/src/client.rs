//! An in-memory model of the containerd client used by Talos.
//!
//! In the real system, machined and the CRI plugin talk to containerd over its
//! gRPC API to pull images, create containers, and run tasks. We model that API
//! surface as the [`ContainerdClient`] trait and provide [`InMemoryContainerd`],
//! an in-memory implementation that enforces the same ordering invariants the
//! real daemon does:
//!
//! * images must be pulled and unpacked before a container can be created;
//! * a container id is unique within its namespace;
//! * a task can only be created from an existing container, started once, and
//!   the container is removed only after its task is stopped or never started.

use crate::container::{Container, ContainerStatus};
use crate::image::{Image, ImageRef, ImageState};
use crate::namespace::Namespace;
use crate::oci_spec::OciSpec;
use crate::task::{Signal, Task};
use std::collections::HashMap;
use os_kernel::error::{Error, Result};

/// The outcome of running a container to completion via [`ContainerdClient::run`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    /// The container id that was run.
    pub container_id: String,
    /// The pid the task was assigned.
    pub pid: u32,
    /// The exit code recorded when the task stopped.
    pub exit_code: i32,
}

/// The subset of the containerd client API Talos relies on.
///
/// This is the trait boundary that, in production, would be backed by the
/// containerd gRPC services (images, containers, tasks).
pub trait ContainerdClient {
    /// Pull and unpack an image into the content store, returning the resolved
    /// [`Image`]. Idempotent: pulling an already-unpacked image is a no-op.
    fn pull_image(
        &mut self,
        ns: &Namespace,
        reference: &ImageRef,
        layers: Vec<String>,
        size: u64,
    ) -> Result<Image>;

    /// Create a container record from a previously unpacked image.
    fn create_container(&mut self, ns: &Namespace, container: Container) -> Result<()>;

    /// Create and start a task for a container, returning its pid.
    fn start_task(&mut self, ns: &Namespace, container_id: &str, pid: u32) -> Result<()>;

    /// Signal a container's task; Term/Kill stop it.
    fn kill_task(&mut self, ns: &Namespace, container_id: &str, signal: Signal) -> Result<()>;

    /// Delete a container (and its task), which must not be running.
    fn delete_container(&mut self, ns: &Namespace, container_id: &str) -> Result<()>;

    /// Fetch a snapshot of a container record.
    fn get_container(&self, ns: &Namespace, container_id: &str) -> Option<&Container>;

    /// List container ids within a namespace.
    fn list_containers(&self, ns: &Namespace) -> Vec<&str>;
}

/// Per-namespace state.
#[derive(Debug, Default)]
struct NamespaceState {
    images: HashMap<String, Image>,
    containers: HashMap<String, Container>,
    tasks: HashMap<String, Task>,
}

/// An in-memory [`ContainerdClient`] implementation for tests and modeling.
#[derive(Debug, Default)]
pub struct InMemoryContainerd {
    namespaces: HashMap<String, NamespaceState>,
}

impl InMemoryContainerd {
    /// Create an empty client.
    pub fn new() -> Self {
        InMemoryContainerd {
            namespaces: HashMap::new(),
        }
    }

    fn ns_mut(&mut self, ns: &Namespace) -> &mut NamespaceState {
        self.namespaces.entry(ns.as_str().to_string()).or_default()
    }

    fn ns(&self, ns: &Namespace) -> Option<&NamespaceState> {
        self.namespaces.get(ns.as_str())
    }

    /// Convenience: pull, unpack, create and run a container to completion in a
    /// single call, mirroring the high-level flow machined uses to start a
    /// system service. The container is killed with `signal` after start.
    // Each parameter maps to a distinct containerd call argument (namespace,
    // spec, image ref, layers, id, pid, signal); bundling them into a struct
    // would only move the noise without clarifying the call site.
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &mut self,
        ns: &Namespace,
        spec: OciSpec,
        reference: ImageRef,
        layers: Vec<String>,
        container_id: impl Into<String>,
        pid: u32,
        signal: Signal,
    ) -> Result<RunResult> {
        let id = container_id.into();
        self.pull_image(ns, &reference, layers, 4096)?;
        let container = Container::new(id.clone(), reference, spec)?;
        self.create_container(ns, container)?;
        self.start_task(ns, &id, pid)?;
        self.kill_task(ns, &id, signal)?;
        let state = self.ns(ns).ok_or_else(|| Error::not_found("namespace"))?;
        let task = state
            .tasks
            .get(&id)
            .ok_or_else(|| Error::not_found("task"))?;
        let exit_code = task
            .exit_code
            .ok_or_else(|| Error::invalid_state("task did not record exit code"))?;
        Ok(RunResult {
            container_id: id,
            pid,
            exit_code,
        })
    }
}

impl ContainerdClient for InMemoryContainerd {
    fn pull_image(
        &mut self,
        ns: &Namespace,
        reference: &ImageRef,
        layers: Vec<String>,
        size: u64,
    ) -> Result<Image> {
        let key = reference.canonical();
        let state = self.ns_mut(ns);
        if let Some(existing) = state.images.get(&key)
            && existing.state == ImageState::Unpacked
        {
            return Ok(existing.clone());
        }
        let mut image = Image::new(reference.clone());
        image.mark_pulled(layers, size)?;
        image.mark_unpacked()?;
        state.images.insert(key.clone(), image.clone());
        Ok(image)
    }

    fn create_container(&mut self, ns: &Namespace, container: Container) -> Result<()> {
        let image_key = container.image.canonical();
        let state = self.ns_mut(ns);
        match state.images.get(&image_key) {
            Some(img) if img.is_runnable() => {}
            Some(_) => return Err(Error::invalid_state("image is not unpacked")),
            None => return Err(Error::not_found("image not pulled")),
        }
        if state.containers.contains_key(&container.id) {
            return Err(Error::invalid_state("container id already exists"));
        }
        state.containers.insert(container.id.clone(), container);
        Ok(())
    }

    fn start_task(&mut self, ns: &Namespace, container_id: &str, pid: u32) -> Result<()> {
        let state = self.ns_mut(ns);
        if !state.containers.contains_key(container_id) {
            return Err(Error::not_found("container"));
        }
        if state.tasks.contains_key(container_id) {
            return Err(Error::invalid_state("task already exists for container"));
        }
        let mut task = Task::new(container_id.to_string());
        task.start(pid)?;
        state.tasks.insert(container_id.to_string(), task);
        if let Some(c) = state.containers.get_mut(container_id) {
            c.status = ContainerStatus::Running;
        }
        Ok(())
    }

    fn kill_task(&mut self, ns: &Namespace, container_id: &str, signal: Signal) -> Result<()> {
        let state = self.ns_mut(ns);
        let task = state
            .tasks
            .get_mut(container_id)
            .ok_or_else(|| Error::not_found("task"))?;
        task.kill(signal)?;
        let stopped = task.is_stopped();
        if stopped && let Some(c) = state.containers.get_mut(container_id) {
            c.status = ContainerStatus::Stopped;
        }
        Ok(())
    }

    fn delete_container(&mut self, ns: &Namespace, container_id: &str) -> Result<()> {
        let state = self.ns_mut(ns);
        if let Some(task) = state.tasks.get(container_id)
            && !task.is_stopped()
        {
            return Err(Error::invalid_state(
                "cannot delete container with running task",
            ));
        }
        if state.containers.remove(container_id).is_none() {
            return Err(Error::not_found("container"));
        }
        state.tasks.remove(container_id);
        Ok(())
    }

    fn get_container(&self, ns: &Namespace, container_id: &str) -> Option<&Container> {
        self.ns(ns)?.containers.get(container_id)
    }

    fn list_containers(&self, ns: &Namespace) -> Vec<&str> {
        match self.ns(ns) {
            Some(state) => {
                let mut ids: Vec<&str> = state.containers.keys().map(String::as_str).collect();
                ids.sort_unstable();
                ids
            }
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oci_spec::OciSpec;

    fn spec() -> OciSpec {
        OciSpec::new(vec!["/usr/local/bin/etcd".to_string()]).unwrap()
    }

    fn etcd_ref() -> ImageRef {
        ImageRef::parse("registry.k8s.io/etcd:3.5").unwrap()
    }

    #[test]
    fn create_requires_pulled_image() {
        let mut cd = InMemoryContainerd::new();
        let ns = Namespace::system();
        let container = Container::new("etcd", etcd_ref(), spec()).unwrap();
        // No image pulled yet.
        let err = cd.create_container(&ns, container.clone()).unwrap_err();
        assert_eq!(err.kind(), "not_found");

        cd.pull_image(&ns, &etcd_ref(), vec!["sha256:l1".to_string()], 1024)
            .unwrap();
        cd.create_container(&ns, container).unwrap();
        assert!(cd.get_container(&ns, "etcd").is_some());
    }

    #[test]
    fn duplicate_container_rejected() {
        let mut cd = InMemoryContainerd::new();
        let ns = Namespace::system();
        cd.pull_image(&ns, &etcd_ref(), vec!["l1".to_string()], 1)
            .unwrap();
        cd.create_container(&ns, Container::new("etcd", etcd_ref(), spec()).unwrap())
            .unwrap();
        let again = Container::new("etcd", etcd_ref(), spec()).unwrap();
        assert_eq!(
            cd.create_container(&ns, again).unwrap_err().kind(),
            "invalid_state"
        );
    }

    #[test]
    fn task_lifecycle_and_status() {
        let mut cd = InMemoryContainerd::new();
        let ns = Namespace::system();
        cd.pull_image(&ns, &etcd_ref(), vec!["l1".to_string()], 1)
            .unwrap();
        cd.create_container(&ns, Container::new("etcd", etcd_ref(), spec()).unwrap())
            .unwrap();

        cd.start_task(&ns, "etcd", 4242).unwrap();
        assert!(cd.get_container(&ns, "etcd").unwrap().is_running());
        // Starting twice fails.
        assert!(cd.start_task(&ns, "etcd", 1).is_err());

        // Cannot delete a running container.
        assert_eq!(
            cd.delete_container(&ns, "etcd").unwrap_err().kind(),
            "invalid_state"
        );

        cd.kill_task(&ns, "etcd", Signal::Term).unwrap();
        assert_eq!(
            cd.get_container(&ns, "etcd").unwrap().status,
            ContainerStatus::Stopped
        );
        cd.delete_container(&ns, "etcd").unwrap();
        assert!(cd.get_container(&ns, "etcd").is_none());
    }

    #[test]
    fn start_task_unknown_container() {
        let mut cd = InMemoryContainerd::new();
        let ns = Namespace::k8s();
        assert_eq!(
            cd.start_task(&ns, "nope", 1).unwrap_err().kind(),
            "not_found"
        );
    }

    #[test]
    fn namespaces_are_isolated() {
        let mut cd = InMemoryContainerd::new();
        let sys = Namespace::system();
        let k8s = Namespace::k8s();
        cd.pull_image(&sys, &etcd_ref(), vec!["l1".to_string()], 1)
            .unwrap();
        cd.create_container(&sys, Container::new("etcd", etcd_ref(), spec()).unwrap())
            .unwrap();
        // Same id, different namespace: not visible, and image not pulled there.
        assert!(cd.get_container(&k8s, "etcd").is_none());
        assert!(cd.list_containers(&k8s).is_empty());
        assert_eq!(cd.list_containers(&sys), vec!["etcd"]);
    }

    #[test]
    fn run_flow_completes() {
        let mut cd = InMemoryContainerd::new();
        let ns = Namespace::system();
        let result = cd
            .run(
                &ns,
                spec(),
                etcd_ref(),
                vec!["sha256:l1".to_string()],
                "etcd",
                7,
                Signal::Term,
            )
            .unwrap();
        assert_eq!(result.container_id, "etcd");
        assert_eq!(result.pid, 7);
        assert_eq!(result.exit_code, 128 + 15);
        assert_eq!(
            cd.get_container(&ns, "etcd").unwrap().status,
            ContainerStatus::Stopped
        );
    }

    #[test]
    fn pull_is_idempotent() {
        let mut cd = InMemoryContainerd::new();
        let ns = Namespace::system();
        let a = cd
            .pull_image(&ns, &etcd_ref(), vec!["l1".to_string()], 10)
            .unwrap();
        assert_eq!(a.state, ImageState::Unpacked);
        // Second pull returns the already-unpacked image.
        let b = cd
            .pull_image(&ns, &etcd_ref(), vec!["ignored".to_string()], 999)
            .unwrap();
        assert_eq!(b.layers, vec!["l1".to_string()]);
        assert_eq!(b.size, 10);
    }
}
