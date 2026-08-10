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
/// | input the substrate cannot represent | [`Error::Invalid`] / [`Error::Parse`] |
///
/// This exists because PID 1 classifies these outcomes. It previously did so by
/// scanning the Linux adapter's `"errno N"` display text, which silently
/// returns "unexpected" for any substrate that does not spell its errors the
/// Linux way — a false seam. Implementations may still carry an `errno N`
/// suffix in the message for diagnostics; no *correctness* decision may depend
/// on it, but a consumer may use it to sharpen a diagnostic label when it is
/// present.
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

/// Collapse default-route destination spellings the way the Linux FIB does:
/// `None` and `Some([0,0,0,0])` at `prefix_len = 0` are one key.
fn normalize_ipv4_route_destination(
    destination: Option<[u8; 4]>,
    prefix_len: u8,
) -> Option<[u8; 4]> {
    if prefix_len == 0 {
        return None;
    }
    destination
}

/// The FIB identity Linux uses for duplicate detection under
/// `NLM_F_CREATE | NLM_F_EXCL` — origin is provenance, not part of the key.
fn ipv4_route_fib_key(route: &FakeRoute) -> (&str, Option<[u8; 4]>, u8, Option<[u8; 4]>, u32) {
    (
        route.iface.as_str(),
        normalize_ipv4_route_destination(route.destination, route.prefix_len),
        route.prefix_len,
        route.gateway,
        route.metric,
    )
}

/// Format an IPv4 address plus prefix the way the kernel read-back path does,
/// rejecting first what `LinuxNet::add_ipv4` rejects before it reaches netlink.
///
/// The parsed value — not the input string — is formatted, so equivalent
/// spellings canonicalize to one entry rather than defeating the duplicate
/// rejection below by respelling.
fn v4_cidr(addr: &str, prefix_len: u8) -> Result<String> {
    if prefix_len > 32 {
        return Err(Error::invalid(alloc::format!(
            "ipv4 prefix length {prefix_len} out of range 0..=32"
        )));
    }
    let ip = addr
        .parse::<core::net::Ipv4Addr>()
        .map_err(|_| Error::parse(alloc::format!("invalid IPv4 address '{addr}'")))?;
    Ok(alloc::format!("{ip}/{prefix_len}"))
}

/// The IPv6 half of [`v4_cidr`]. Canonicalization matters more here: Linux
/// resolves `fd00::5` and `fd00:0:0:0:0:0:0:5` to one address and answers the
/// second add with `EEXIST`.
fn v6_cidr(addr: &str, prefix_len: u8) -> Result<String> {
    if prefix_len > 128 {
        return Err(Error::invalid(alloc::format!(
            "ipv6 prefix length {prefix_len} out of range 0..=128"
        )));
    }
    let ip = addr
        .parse::<core::net::Ipv6Addr>()
        .map_err(|_| Error::parse(alloc::format!("invalid IPv6 address '{addr}'")))?;
    Ok(alloc::format!("{ip}/{prefix_len}"))
}

/// Address identity of a canonical `addr/prefix` CIDR string — prefix is metadata.
///
/// Forever shape for FakeNet: both IPv4 and IPv6 duplicate installs key on
/// interface+address under `NLM_F_CREATE | NLM_F_EXCL`. A prior IPv4-only
/// CIDR-string key was OVERRULED as dual-truth against the IPv6 contract.
fn cidr_address_key(entry: &str) -> &str {
    entry.split_once('/').map(|(a, _)| a).unwrap_or(entry)
}

/// The shape an IPv4 route must have before any substrate is asked to install
/// it.
///
/// Shared rather than restated: the first two rules were written out by hand in
/// both [`InMemoryKernelNet::add_ipv4_route`] and `LinuxNet::add_ipv4_route`, so
/// a precondition the kernel enforces and the author did not think of was
/// missing from *both* copies at once. That is how the third rule went missing.
///
/// The third rule is Linux's, from `rtm_to_fib_config()` in
/// `net/ipv4/fib_frontend.c`:
///
/// ```c
/// if (cfg->fc_dst_len < 32 && (ntohl(cfg->fc_dst) << cfg->fc_dst_len)) {
///         NL_SET_ERR_MSG(extack, "Invalid prefix for given prefix length");
///         err = -EINVAL;
/// ```
///
/// It runs on the `RTM_NEWROUTE` path *before* `fib_table_insert`, so a
/// destination carrying bits below its own prefix — `10.0.0.5/24` — is rejected
/// outright. It is deliberately **not** masked to `10.0.0.0/24`: masking here
/// would make this fake accept, and silently rewrite, input a real kernel
/// refuses, which is precisely the false green the duplicate-rejection test
/// below exists to prevent. `/32` is exempt from the shift, in the kernel and
/// here.
pub fn check_ipv4_route_shape(destination: Option<[u8; 4]>, prefix_len: u8) -> Result<()> {
    if prefix_len > 32 {
        return Err(Error::invalid(alloc::format!(
            "ipv4 route prefix length {prefix_len} out of range 0..=32"
        )));
    }
    if prefix_len > 0 && destination.is_none() {
        return Err(Error::invalid("non-default IPv4 route needs destination"));
    }
    if let Some(dst) = destination
        && prefix_len < 32
        && u32::from_be_bytes(dst).wrapping_shl(prefix_len as u32) != 0
    {
        return Err(Error::invalid(alloc::format!(
            "invalid prefix for given prefix length: {}.{}.{}.{}/{prefix_len}",
            dst[0],
            dst[1],
            dst[2],
            dst[3]
        )));
    }
    Ok(())
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
        // Validate before touching state, as the adapter validates before
        // touching netlink. For an unknown interface *and* a malformed address
        // the fake reports `parse` where `LinuxNet` reports `not_found` first;
        // that is the safe direction — a stricter fake cannot let a bad test
        // pass — so the adapter's exact check order is deliberately not copied.
        let entry = v4_cidr(addr, prefix_len)?;
        // Forever shape: IPv4 keys like IPv6 — interface+address, not full CIDR.
        // Replaying `10.0.0.5/24` then `10.0.0.5/32` is EEXIST, not a second row.
        let address_key = cidr_address_key(&entry).to_string();
        self.mutate(iface, |l| {
            if l.ipv4
                .iter()
                .any(|existing| cidr_address_key(existing) == address_key)
            {
                return Err(already_exists("address", iface, &entry));
            }
            l.ipv4.push(entry);
            Ok(())
        })?
    }

    fn add_ipv6_address(&self, iface: &str, addr: &str, prefix_len: u8) -> Result<()> {
        let entry = v6_cidr(addr, prefix_len)?;
        // Linux keys IPv6 installs by interface+address, not by CIDR string:
        // replaying `fd00::5/64` then `fd00::5/128` is EEXIST, not a second row.
        let address_key = cidr_address_key(&entry).to_string();
        self.mutate(iface, |l| {
            if l.ipv6
                .iter()
                .any(|existing| cidr_address_key(existing) == address_key)
            {
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
        // The same shape checks `LinuxNet::add_ipv4_route` makes before netlink
        // — the one function, not a second copy of the rules.
        check_ipv4_route_shape(destination, prefix_len)?;
        // A route needs its outgoing link to exist, exactly as rtnetlink does.
        self.mutate(iface, |_| ())?;
        // Linux FIB identity collapses default-route spellings (`destination =
        // None` vs `Some([0,0,0,0])` at prefix 0) and does not key on origin —
        // `NLM_F_CREATE | NLM_F_EXCL` returns EEXIST for either. Compare the
        // normalized key, not the whole `FakeRoute`.
        let destination = normalize_ipv4_route_destination(destination, prefix_len);
        let route = FakeRoute {
            iface: iface.to_string(),
            destination,
            prefix_len,
            gateway,
            metric,
            origin,
        };
        let mut routes = self.routes.borrow_mut();
        if routes.iter().any(|existing| ipv4_route_fib_key(existing) == ipv4_route_fib_key(&route))
        {
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
        // Arrange
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv6_address("eth0", "fd00::5", 64).unwrap();

        // Act — exact replay
        let err = net.add_ipv6_address("eth0", "fd00::5", 64).unwrap_err();

        // Assert
        assert_eq!(err.kind(), "invalid_state");
        assert_eq!(net.ipv6_addresses("eth0").len(), 1);
    }

    #[test]
    fn ipv6_duplicate_key_is_address_not_cidr() {
        // Arrange — Linux ifaddrmsg keys IPv6 by interface+address; prefix is
        // not part of the identity under NLM_F_CREATE|NLM_F_EXCL.
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv6_address("eth0", "fd00::5", 64).unwrap();

        // Act
        let err = net.add_ipv6_address("eth0", "fd00::5", 128).unwrap_err();

        // Assert — EEXIST, not a second row
        assert_eq!(err.kind(), "invalid_state");
        assert_eq!(
            net.ipv6_addresses("eth0"),
            alloc::vec!["fd00::5/64".to_string()]
        );
    }

    #[test]
    fn ipv6_address_key_survives_respelling_and_prefix_change() {
        // Arrange — the two seams that independently used to defeat EEXIST
        // must fail together too: uncompressed spelling + different prefix.
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv6_address("eth0", "fd00::5", 64).unwrap();

        // Act
        let err = net
            .add_ipv6_address("eth0", "fd00:0:0:0:0:0:0:5", 128)
            .unwrap_err();

        // Assert
        assert_eq!(err.kind(), "invalid_state");
        assert_eq!(net.ipv6_addresses("eth0").len(), 1);
    }

    #[test]
    fn distinct_ipv6_addresses_on_one_iface_are_both_kept() {
        // Arrange — address-keying must not over-collapse unrelated rows.
        let net = InMemoryKernelNet::new().with_link("eth0");

        // Act
        net.add_ipv6_address("eth0", "fd00::5", 64).unwrap();
        net.add_ipv6_address("eth0", "fd00::6", 64).unwrap();

        // Assert
        assert_eq!(
            net.ipv6_addresses("eth0"),
            alloc::vec!["fd00::5/64".to_string(), "fd00::6/64".to_string()]
        );
    }

    #[test]
    fn same_ipv6_address_on_different_ifaces_is_not_a_duplicate() {
        // Arrange — key is interface+address, not address alone.
        let net = InMemoryKernelNet::new().with_link("eth0").with_link("eth1");

        // Act
        net.add_ipv6_address("eth0", "fd00::5", 64).unwrap();
        net.add_ipv6_address("eth1", "fd00::5", 64).unwrap();

        // Assert
        assert_eq!(
            net.ipv6_addresses("eth0"),
            alloc::vec!["fd00::5/64".to_string()]
        );
        assert_eq!(
            net.ipv6_addresses("eth1"),
            alloc::vec!["fd00::5/64".to_string()]
        );
    }

    #[test]
    fn ipv4_duplicate_key_is_address_not_cidr() {
        // Arrange — forever shape OVERRULES the prior IPv4 CIDR-asymmetry dual-truth:
        // IPv4 keys like IPv6 (interface+address); prefix is metadata only.
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv4_address("eth0", "10.0.0.5", 24).unwrap();

        // Act
        let err = net.add_ipv4_address("eth0", "10.0.0.5", 32).unwrap_err();

        // Assert — EEXIST, not a second row
        assert_eq!(err.kind(), "invalid_state");
        assert_eq!(
            net.ipv4_addresses("eth0").unwrap(),
            alloc::vec!["10.0.0.5/24".to_string()]
        );
    }

    #[test]
    fn distinct_ipv4_addresses_on_one_iface_are_both_kept() {
        // Arrange — address-keying must not over-collapse unrelated rows.
        let net = InMemoryKernelNet::new().with_link("eth0");

        // Act
        net.add_ipv4_address("eth0", "10.0.0.5", 24).unwrap();
        net.add_ipv4_address("eth0", "10.0.0.6", 24).unwrap();

        // Assert
        assert_eq!(
            net.ipv4_addresses("eth0").unwrap(),
            alloc::vec!["10.0.0.5/24".to_string(), "10.0.0.6/24".to_string()]
        );
    }

    /// The inputs `LinuxNet` rejects before it reaches netlink. Written against
    /// `impl KernelNet` for the same reason `boot_sequence` is: the body is the
    /// contract, and it would run unchanged against the real adapter.
    fn rejects_what_the_kernel_would(net: &impl KernelNet) {
        assert_eq!(
            net.add_ipv4_address("eth0", "10.0.0.5", 33)
                .unwrap_err()
                .kind(),
            "invalid"
        );
        assert_eq!(
            net.add_ipv6_address("eth0", "fd00::5", 129)
                .unwrap_err()
                .kind(),
            "invalid"
        );
        assert_eq!(
            net.add_ipv4_address("eth0", "10.0.0.999", 24)
                .unwrap_err()
                .kind(),
            "parse"
        );
        assert_eq!(
            net.add_ipv6_address("eth0", "not-an-address", 64)
                .unwrap_err()
                .kind(),
            "parse"
        );
        assert_eq!(
            net.add_ipv4_route("eth0", None, 24, None, 1, RouteOrigin::Boot)
                .unwrap_err()
                .kind(),
            "invalid"
        );
    }

    #[test]
    fn the_fake_rejects_inputs_the_linux_adapter_rejects() {
        // A fake that records junk lets a test pass over code that fails on a
        // real kernel — the one property this fake exists to prove.
        let net = InMemoryKernelNet::new().with_link("eth0");
        rejects_what_the_kernel_would(&net);
        assert!(net.ipv4_addresses("eth0").unwrap().is_empty());
        assert!(net.routes().is_empty());
    }

    #[test]
    fn a_destination_with_host_bits_is_rejected_not_masked() {
        // `rtm_to_fib_config()` rejects a destination carrying bits below its
        // prefix before `fib_table_insert` sees the key, so `10.0.0.5/24` and
        // `10.0.0.6/24` are BOTH EINVAL on Linux — they are not two entries
        // here, and they are not one entry plus an EEXIST either.
        let net = InMemoryKernelNet::new().with_link("eth0");
        for host in [5u8, 6] {
            assert_eq!(
                net.add_ipv4_route(
                    "eth0",
                    Some([10, 0, 0, host]),
                    24,
                    None,
                    1,
                    RouteOrigin::Boot
                )
                .unwrap_err()
                .kind(),
                "invalid"
            );
        }
        // Rejected, not normalized: masking to `10.0.0.0/24` would make the
        // fake looser than the kernel, which is the failure this whole port
        // exists to prevent.
        assert!(net.routes().is_empty());

        // The properly-aligned prefix is accepted, and `/32` is exempt from the
        // rule exactly as the kernel exempts it.
        net.add_ipv4_route("eth0", Some([10, 0, 0, 0]), 24, None, 1, RouteOrigin::Boot)
            .unwrap();
        net.add_ipv4_route(
            "eth0",
            Some([169, 254, 0, 1]),
            32,
            None,
            1,
            RouteOrigin::Boot,
        )
        .unwrap();
        assert_eq!(net.routes().len(), 2);
    }

    #[test]
    fn equivalent_default_route_spellings_are_one_fib_entry() {
        // Linux resolves `destination = None, prefix_len = 0` and
        // `destination = Some([0,0,0,0]), prefix_len = 0` to the same default
        // route; a second install (even with a different origin) returns EEXIST.
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv4_route(
            "eth0",
            None,
            0,
            Some([10, 0, 0, 1]),
            1024,
            RouteOrigin::Dhcp,
        )
        .unwrap();
        assert_eq!(
            net.add_ipv4_route(
                "eth0",
                Some([0, 0, 0, 0]),
                0,
                Some([10, 0, 0, 1]),
                1024,
                RouteOrigin::Boot,
            )
            .unwrap_err()
            .kind(),
            "invalid_state"
        );
        assert_eq!(net.routes().len(), 1);
        assert_eq!(net.routes()[0].destination, None);
    }

    #[test]
    fn equivalent_ipv6_spellings_are_one_address() {
        // Compared as raw strings these were two entries here and one EEXIST on
        // Linux, so a respelling defeated the duplicate rejection entirely.
        let net = InMemoryKernelNet::new().with_link("eth0");
        net.add_ipv6_address("eth0", "fd00::5", 64).unwrap();
        assert_eq!(
            net.add_ipv6_address("eth0", "fd00:0:0:0:0:0:0:5", 64)
                .unwrap_err()
                .kind(),
            "invalid_state"
        );
        assert_eq!(
            net.ipv6_addresses("eth0"),
            alloc::vec!["fd00::5/64".to_string()]
        );
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
