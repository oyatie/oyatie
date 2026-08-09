//! The kernel *network* operations `os/` issues.
//!
//! This is the surface `os-init-app` drives at boot and `os-network-domain`
//! implements over Linux rtnetlink/`ioctl`/`/sys`. Every method is an
//! operation ("bring this link up"), never a Linux encoding: the `RTPROT_*`
//! numbers, netlink message layout and `/sys` paths live in the adapter.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;
use os_kernel::error::{Error, Result};

/// Who installed a route.
///
/// The operation-level notion. Linux encodes these as `RTPROT_*` byte values;
/// that mapping belongs to the adapter, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteOrigin {
    /// Configured by the operator/admin.
    Static,
    /// Installed by the boot process.
    Boot,
    /// Installed by the kernel itself (e.g. on-link routes).
    Kernel,
    /// Installed by a DHCP client.
    Dhcp,
}

/// Network operations `os/` needs from a kernel.
///
/// # Failure contract
///
/// The port classifies failures by [`Error`] **variant**, so a caller can tell
/// a benign outcome from a real one without parsing adapter text. Every
/// implementation MUST map its native failures onto these:
///
/// | Condition | Variant |
/// |---|---|
/// | interface does not exist | [`Error::NotFound`] |
/// | the address/route is already installed | [`Error::InvalidState`] |
/// | the caller may not perform the operation | [`Error::PermissionDenied`] |
/// | the substrate does not implement the operation | [`Error::Unsupported`] |
///
/// This exists because PID 1 classifies these outcomes. It previously did so by
/// scanning the Linux adapter's `"errno N"` display text, which silently
/// returns "unexpected" for any substrate that does not spell its errors the
/// Linux way — a false seam. Implementations may still carry an `errno N`
/// suffix in the message for diagnostics; nothing may *depend* on it.
///
/// Adds are deliberately **not** idempotent: the Linux adapter issues
/// `NLM_F_CREATE | NLM_F_EXCL` and the kernel answers a duplicate with
/// `EEXIST`, so a fake that silently deduplicates would let a test pass over
/// code that fails in production. Re-adding is still the normal re-run path —
/// the caller tolerates [`Error::InvalidState`] rather than aborting.
///
/// Methods take `&self` because the kernel — not the handle — holds the state;
/// adapters over a real kernel are stateless, and the in-memory fake uses
/// interior mutability.
pub trait KernelNet {
    /// Bring an interface administratively up.
    fn set_link_up(&self, iface: &str) -> Result<()>;

    /// Assign an IPv4 address (dotted-quad) with `prefix_len` to an interface.
    fn add_ipv4_address(&self, iface: &str, addr: &str, prefix_len: u8) -> Result<()>;

    /// Assign an IPv6 address with `prefix_len` to an interface.
    fn add_ipv6_address(&self, iface: &str, addr: &str, prefix_len: u8) -> Result<()>;

    /// Install an IPv4 route out of `iface`.
    ///
    /// `destination`/`gateway` are `None` for a default route and an
    /// unspecified gateway respectively. `metric` orders competing routes.
    fn add_ipv4_route(
        &self,
        iface: &str,
        destination: Option<[u8; 4]>,
        prefix_len: u8,
        gateway: Option<[u8; 4]>,
        metric: u32,
        origin: RouteOrigin,
    ) -> Result<()>;

    /// Read back the IPv4 addresses the kernel currently has on `iface`, as
    /// CIDR strings (`"10.0.0.5/24"`). This is the verification path: it must
    /// reflect the kernel's view, not a cached copy of what was requested.
    fn ipv4_addresses(&self, iface: &str) -> Result<Vec<String>>;

    /// The interface's operational state as the kernel reports it — `"up"`,
    /// `"down"`, `"unknown"`, or any other value the kernel uses.
    ///
    /// Deliberately a string: this is a cross-confirmation reading from a
    /// *different* kernel surface than the one that set the link up, and
    /// collapsing unrecognised values into an enum would hide exactly the
    /// disagreement it exists to expose.
    fn link_oper_state(&self, iface: &str) -> Result<String>;
}

// ---------------------------------------------------------------------------
// In-memory fake
// ---------------------------------------------------------------------------

/// One interface in the [`InMemoryKernelNet`].
///
/// Every field carries a `data_class` per the kernel data-boundary rule: this
/// crate is cataloged `role: kernel`, and a crate-level `data_classes_owned`
/// does not discharge the per-field obligation.
#[derive(Debug, Clone, Default)]
struct FakeLink {
    // data_class: INTERNAL_ONLY — admin link flag, machine-local.
    up: bool,
    // data_class: INTERNAL_ONLY — seeded operational state, machine-local.
    oper_state: Option<String>,
    // data_class: INTERNAL_ONLY — node IPv4 addresses, machine-local.
    ipv4: Vec<String>,
    // data_class: INTERNAL_ONLY — node IPv6 addresses, machine-local.
    ipv6: Vec<String>,
}

impl FakeLink {
    /// The operational state a link reports: an explicitly seeded value wins,
    /// otherwise it follows the admin flag the way a kernel link does.
    fn oper_state(&self) -> String {
        match &self.oper_state {
            Some(s) => s.clone(),
            None if self.up => "up".to_string(),
            None => "down".to_string(),
        }
    }
}

/// A route recorded by [`InMemoryKernelNet`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeRoute {
    /// The interface the route leaves by.
    // data_class: INTERNAL_ONLY — interface name, machine-local.
    pub iface: String,
    /// Destination network, `None` for the default route.
    // data_class: INTERNAL_ONLY — node routing topology, machine-local.
    pub destination: Option<[u8; 4]>,
    /// Destination prefix length.
    // data_class: INTERNAL_ONLY — node routing topology, machine-local.
    pub prefix_len: u8,
    /// Next hop, if any.
    // data_class: INTERNAL_ONLY — node routing topology, machine-local.
    pub gateway: Option<[u8; 4]>,
    /// Route metric.
    // data_class: INTERNAL_ONLY — route preference, machine-local.
    pub metric: u32,
    /// Who installed it.
    // data_class: INTERNAL_ONLY — route provenance, machine-local.
    pub origin: RouteOrigin,
}

/// A deterministic in-memory [`KernelNet`] for tests.
///
/// A port with exactly one implementation has not been shown to be a port, so
/// this fake is part of the contract, not a convenience: it is the second
/// substrate the surface is proven against.
#[derive(Debug, Default)]
pub struct InMemoryKernelNet {
    // data_class: INTERNAL_ONLY — simulated interface table, test-process-local.
    links: RefCell<alloc::collections::BTreeMap<String, FakeLink>>,
    // data_class: INTERNAL_ONLY — simulated routing table, test-process-local.
    routes: RefCell<Vec<FakeRoute>>,
}

impl InMemoryKernelNet {
    /// A kernel with no interfaces.
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed an existing (administratively down) interface.
    pub fn with_link(self, iface: &str) -> Self {
        self.links
            .borrow_mut()
            .insert(iface.to_string(), FakeLink::default());
        self
    }

    /// Seed an interface whose operational state is pinned to `state`,
    /// regardless of the admin flag — the real disagreement case.
    pub fn with_oper_state(self, iface: &str, state: &str) -> Self {
        self.links.borrow_mut().insert(
            iface.to_string(),
            FakeLink {
                oper_state: Some(state.to_string()),
                ..FakeLink::default()
            },
        );
        self
    }

    /// Whether `iface` is administratively up.
    pub fn is_up(&self, iface: &str) -> bool {
        self.links.borrow().get(iface).is_some_and(|l| l.up)
    }

    /// The IPv6 addresses assigned to `iface`, as CIDR strings.
    pub fn ipv6_addresses(&self, iface: &str) -> Vec<String> {
        self.links
            .borrow()
            .get(iface)
            .map(|l| l.ipv6.clone())
            .unwrap_or_default()
    }

    /// Every route installed, in installation order.
    pub fn routes(&self) -> Vec<FakeRoute> {
        self.routes.borrow().clone()
    }

    fn mutate<T>(&self, iface: &str, f: impl FnOnce(&mut FakeLink) -> T) -> Result<T> {
        let mut links = self.links.borrow_mut();
        let link = links
            .get_mut(iface)
            .ok_or_else(|| Error::not_found(alloc::format!("interface '{iface}' not found")))?;
        Ok(f(link))
    }
}

/// Format an address plus prefix the way the kernel read-back path does.
fn cidr(addr: &str, prefix_len: u8) -> String {
    alloc::format!("{addr}/{prefix_len}")
}

/// The duplicate-install failure, in the variant the port's failure contract
/// requires — the same one the Linux adapter produces from `EEXIST`.
fn already_exists(what: &str, iface: &str, detail: &str) -> Error {
    Error::invalid_state(alloc::format!(
        "{what} on '{iface}' already exists: {detail}"
    ))
}

impl KernelNet for InMemoryKernelNet {
    fn set_link_up(&self, iface: &str) -> Result<()> {
        self.mutate(iface, |l| l.up = true)
    }

    fn add_ipv4_address(&self, iface: &str, addr: &str, prefix_len: u8) -> Result<()> {
        let entry = cidr(addr, prefix_len);
        self.mutate(iface, |l| {
            if l.ipv4.contains(&entry) {
                return Err(already_exists("address", iface, &entry));
            }
            l.ipv4.push(entry);
            Ok(())
        })?
    }

    fn add_ipv6_address(&self, iface: &str, addr: &str, prefix_len: u8) -> Result<()> {
        let entry = cidr(addr, prefix_len);
        self.mutate(iface, |l| {
            if l.ipv6.contains(&entry) {
                return Err(already_exists("address", iface, &entry));
            }
            l.ipv6.push(entry);
            Ok(())
        })?
    }

    fn add_ipv4_route(
        &self,
        iface: &str,
        destination: Option<[u8; 4]>,
        prefix_len: u8,
        gateway: Option<[u8; 4]>,
        metric: u32,
        origin: RouteOrigin,
    ) -> Result<()> {
        // A route needs its outgoing link to exist, exactly as rtnetlink does.
        self.mutate(iface, |_| ())?;
        let route = FakeRoute {
            iface: iface.to_string(),
            destination,
            prefix_len,
            gateway,
            metric,
            origin,
        };
        let mut routes = self.routes.borrow_mut();
        if routes.contains(&route) {
            return Err(already_exists("route", iface, "already installed"));
        }
        routes.push(route);
        Ok(())
    }

    fn ipv4_addresses(&self, iface: &str) -> Result<Vec<String>> {
        self.mutate(iface, |l| l.ipv4.clone())
    }

    fn link_oper_state(&self, iface: &str) -> Result<String> {
        self.mutate(iface, |l| l.oper_state())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive an arbitrary substrate through the boot sequence `os-init-app`
    /// performs: link up, assign, read back. Written against `impl KernelNet`
    /// so the same body would run against a real adapter.
    fn boot_sequence(net: &impl KernelNet) -> Result<Vec<String>> {
        net.set_link_up("eth0")?;
        net.add_ipv4_address("eth0", "10.0.0.5", 24)?;
        net.add_ipv4_route(
            "eth0",
            None,
            0,
            Some([10, 0, 0, 1]),
            1024,
            RouteOrigin::Dhcp,
        )?;
        net.ipv4_addresses("eth0")
    }

    #[test]
    fn fake_drives_the_boot_sequence() {
        let net = InMemoryKernelNet::new().with_link("eth0");
        assert!(!net.is_up("eth0"));

        let addrs = boot_sequence(&net).unwrap();

        assert!(net.is_up("eth0"));
        assert_eq!(addrs, alloc::vec!["10.0.0.5/24".to_string()]);
        assert_eq!(
            net.routes(),
            alloc::vec![FakeRoute {
                iface: "eth0".to_string(),
                destination: None,
                prefix_len: 0,
                gateway: Some([10, 0, 0, 1]),
                metric: 1024,
                origin: RouteOrigin::Dhcp,
            }]
        );
    }

    #[test]
    fn re_adding_reports_already_exists_and_changes_nothing() {
        // The Linux adapter sends NLM_F_CREATE | NLM_F_EXCL, so the kernel
        // rejects a duplicate rather than absorbing it. A fake that silently
        // deduplicated would let a caller's re-run path pass here and fail on a
        // real kernel — the failure the port's contract exists to prevent.
        let net = InMemoryKernelNet::new().with_link("eth0");
        boot_sequence(&net).unwrap();

        let err = net.add_ipv4_address("eth0", "10.0.0.5", 24).unwrap_err();
        assert_eq!(err.kind(), "invalid_state");

        let route_err = net
            .add_ipv4_route(
                "eth0",
                None,
                0,
                Some([10, 0, 0, 1]),
                1024,
                RouteOrigin::Dhcp,
            )
            .unwrap_err();
        assert_eq!(route_err.kind(), "invalid_state");

        // Rejected, not appended twice.
        assert_eq!(net.ipv4_addresses("eth0").unwrap().len(), 1);
        assert_eq!(net.routes().len(), 1);
    }

    #[test]
    fn a_duplicate_ipv6_address_is_rejected_the_same_way() {
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv6_address("eth0", "fd00::5", 64).unwrap();
        let err = net.add_ipv6_address("eth0", "fd00::5", 64).unwrap_err();
        assert_eq!(err.kind(), "invalid_state");
        assert_eq!(net.ipv6_addresses("eth0").len(), 1);
    }

    #[test]
    fn an_unknown_interface_is_not_found_not_invalid_state() {
        // The two failure classes the contract table separates must not
        // collapse into each other: the caller tolerates one and reports the
        // other.
        let net = InMemoryKernelNet::new();
        assert_eq!(net.set_link_up("eth0").unwrap_err().kind(), "not_found");
    }

    #[test]
    fn operations_on_an_unknown_interface_fail() {
        let net = InMemoryKernelNet::new();
        assert!(net.set_link_up("eth0").is_err());
        assert!(net.add_ipv4_address("eth0", "10.0.0.5", 24).is_err());
        assert!(net.add_ipv6_address("eth0", "fd00::5", 64).is_err());
        assert!(net.ipv4_addresses("eth0").is_err());
        assert!(net.link_oper_state("eth0").is_err());
        assert!(
            net.add_ipv4_route("eth0", None, 0, None, 1, RouteOrigin::Boot)
                .is_err()
        );
        assert!(net.routes().is_empty());
    }

    #[test]
    fn oper_state_follows_the_admin_flag_unless_pinned() {
        let net = InMemoryKernelNet::new().with_link("eth0");
        assert_eq!(net.link_oper_state("eth0").unwrap(), "down");
        net.set_link_up("eth0").unwrap();
        assert_eq!(net.link_oper_state("eth0").unwrap(), "up");
    }

    #[test]
    fn oper_state_preserves_values_no_enum_would_model() {
        // The cross-confirmation surface must be able to disagree with the
        // admin flag, including with states outside up/down/unknown.
        let net = InMemoryKernelNet::new().with_oper_state("eth0", "lowerlayerdown");
        net.set_link_up("eth0").unwrap();
        assert!(net.is_up("eth0"));
        assert_eq!(net.link_oper_state("eth0").unwrap(), "lowerlayerdown");
    }

    #[test]
    fn ipv6_is_tracked_separately_from_ipv4() {
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv6_address("eth0", "fd00::5", 64).unwrap();
        assert!(net.ipv4_addresses("eth0").unwrap().is_empty());
        assert_eq!(
            net.ipv6_addresses("eth0"),
            alloc::vec!["fd00::5/64".to_string()]
        );
    }

    #[test]
    fn routes_differing_only_by_metric_are_distinct() {
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv4_route("eth0", None, 0, None, 100, RouteOrigin::Static)
            .unwrap();
        net.add_ipv4_route("eth0", None, 0, None, 200, RouteOrigin::Static)
            .unwrap();
        assert_eq!(net.routes().len(), 2);
    }
}
