//! talos-conditions
//!
//! Models Talos's `conditions.Condition` abstraction from `pkg/conditions`:
//! composable predicates the boot sequencer and service dependency graph block
//! on (a file exists, a service is running/healthy, the network is ready, etcd
//! has quorum), with combinators (all-of, any-of), a deterministic polling
//! driver, and human-readable status strings.
//!
//! ## The model
//!
//! - [`Condition`] is the central trait, mirroring Talos's
//!   `Condition interface { fmt.Stringer; Wait(ctx) error }`. It exposes a
//!   non-blocking [`Condition::poll`] returning [`Poll::Ready`] /
//!   [`Poll::Pending`] / [`Poll::Failed`], a [`Condition::describe`] status
//!   string, and a default blocking [`Condition::wait`].
//! - [`Poller`] is the wait budget/cadence (max attempts, sleep interval,
//!   optional deadline), driven by a [`WaitClock`] so tests are deterministic.
//! - OS boundaries are traits with in-memory implementations:
//!   [`FileProbe`]/[`InMemoryFiles`], [`ServiceProbe`]/[`ServiceRegistry`],
//!   [`NetworkProbe`]/[`InMemoryNetwork`], [`EtcdProbe`]/[`InMemoryEtcd`].
//! - Concrete conditions: [`WaitForFileToExist`], [`WaitForFilesToExist`],
//!   [`WaitForServiceState`], [`WaitForServiceHealthy`],
//!   [`WaitForNetworkReady`], [`WaitForEtcdReady`].
//! - Combinators: [`All`], [`Any`], plus the trivial [`None`].

pub mod compose;
pub mod condition;
pub mod file;
pub mod network;
pub mod service;

pub use compose::{All, Any};
pub use condition::{Condition, None, Poll, Poller, SimClock, WaitClock, WaitReport};
pub use file::{FileProbe, InMemoryFiles, WaitForFileToExist, WaitForFilesToExist};
pub use network::{
    EtcdProbe, EtcdStatus, InMemoryEtcd, InMemoryNetwork, NetworkProbe, NetworkStatus,
    WaitForEtcdReady, WaitForNetworkReady,
};
pub use service::{
    Health, ServiceProbe, ServiceRegistry, ServiceState, ServiceStatus, WaitForServiceHealthy,
    WaitForServiceState,
};

#[cfg(test)]
mod tests {
    use super::*;
    use condition::SimClock;
    use os_kernel::os::Clock;

    /// End-to-end: combine heterogeneous conditions (file + service + network)
    /// into an all-of and drive it to ready, mirroring a boot-task dependency.
    #[test]
    fn boot_task_dependency_all_of() {
        let mut fs = InMemoryFiles::new();
        fs.create("/var/run/containerd/containerd.sock");

        let mut reg = ServiceRegistry::new();
        reg.set(ServiceStatus::new(
            "cri",
            ServiceState::Running,
            Health::Healthy,
        ));

        let mut net = InMemoryNetwork::new();
        net.set(NetworkStatus::all_ready());

        let file_cond = WaitForFileToExist::new(&fs, "/var/run/containerd/containerd.sock");
        let svc_cond = WaitForServiceHealthy::new(&reg, "cri");
        let net_cond = WaitForNetworkReady::new(&net);

        let all = All::new(vec![&file_cond, &svc_cond, &net_cond]);
        let clock = SimClock::new(0);
        let report = all.wait(&clock, Poller::new(5, 100)).unwrap();
        assert_eq!(report.attempts, 1);
    }

    /// If a dependency is permanently failed, the all-of aborts immediately
    /// rather than waiting out the whole poll budget.
    #[test]
    fn failed_dependency_aborts_all_of() {
        let mut reg = ServiceRegistry::new();
        reg.set(ServiceStatus::new(
            "etcd",
            ServiceState::Failed,
            Health::Unhealthy,
        ));
        let net = InMemoryNetwork::new(); // not ready, but failure should win

        let svc = WaitForServiceState::running(&reg, "etcd");
        let net_cond = WaitForNetworkReady::new(&net);
        let all = All::new(vec![&svc, &net_cond]);

        let clock = SimClock::new(0);
        let err = all.wait(&clock, Poller::new(10, 50)).unwrap_err();
        assert_eq!(err.kind(), "invalid_state");
        // No polling time consumed.
        assert_eq!(clock.now_unix_nanos(), 0);
    }

    #[test]
    fn any_of_either_file() {
        let mut fs = InMemoryFiles::new();
        fs.create("/b");
        let a = WaitForFileToExist::new(&fs, "/a");
        let b = WaitForFileToExist::new(&fs, "/b");
        let any = Any::new(vec![&a, &b]);
        assert_eq!(any.poll(), Poll::Ready);
    }
}
