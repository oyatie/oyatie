//! `talos-init` — the Linux PID 1 (init) binary for the operating-system Talos-in-Rust
//! port.
//!
//! This binary is the very first userspace process the kernel starts (`/init`
//! in an initramfs, or `init=/init` on the kernel command line). Unlike the
//! earlier "mount + hostname + exit" demo, this init is a **real supervising
//! init that drives the `talos-machined` boot sequencer**:
//!
//! 1. Wire stdio to the console so `println!` reaches the serial port.
//! 2. Implement the machined [`Runtime`] trait for real with `libc`:
//!    `mount(2)`, `sethostname(2)`, sysctl writes to `/proc/sys/...`, and a
//!    `fork(2)`+`execve(2)` service spawner.
//! 3. Construct a [`BootSequencer`] and call `run_boot(&mut self_as_runtime)`,
//!    printing each phase/task as the sequencer reports it. This proves the
//!    ported machined logic actually drives the boot.
//! 4. After the sequence reaches `Running`, reap the spawned service via
//!    `waitpid(2)`, print the success banner, and power the machine off with
//!    `reboot(2)` so QEMU exits.
//!
//! PID 1 must NEVER return, or the kernel panics with "Attempted to kill
//! init!"; every path ends in [`init::shutdown`] followed by a guard loop.
//!
//! The kernel-facing logic is gated behind `#[cfg(target_os = "linux")]`
//! because the `libc` syscall signatures only exist / match on Linux. On a
//! non-Linux build host the binary refuses to run, while the pure helpers (and
//! their unit tests) still build and run everywhere.

use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::net::{Ipv6Addr, SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

use os_block_domain::{
    IMAGE_CACHE_VOLUME_ID, VolumeConfig, VolumeMountStatusResource, VolumeMountStatusSpec,
    VolumePhase, VolumeStatus, VolumeStatusResource,
};
use os_cosi_domain::State;
use os_machined_domain::error::{MachinedError, Result as MachinedResult};
use os_machined_domain::{ServiceLauncher, Supervisor, SupervisorRegistrydServiceManager};
use os_runtime_cri_domain::{
    IMAGE_CACHE_CONTROLLER_NAME, IMAGE_CACHE_DISK_MOUNT_POINT, IMAGE_CACHE_DISK_VOLUME_ID,
    IMAGE_CACHE_ISO_MOUNT_POINT, IMAGE_CACHE_ISO_VOLUME_ID, ImageCacheConfigController,
    ImageCacheCopyExecutionStatus, ImageCacheCopyGate, ImageCacheCopyReport,
    ImageCacheCopyRuntimeAdapter, ImageCacheCopyRuntimeEnvironment, ImageCacheReconcileInput,
    ImageCacheRuntimePlan, REGISTRYD_HEALTH_PATH, REGISTRYD_LISTEN_ADDRESS, REGISTRYD_SERVICE_ID,
    RegistrydContentResponse, RegistrydHealthProbe, RegistrydRuntimeAdapter,
    RegistrydRuntimeService, RegistrydServiceReport, RegistrydState,
    apply_image_cache_copy_report_to_state, apply_image_cache_plan_to_state,
    image_cache_copy_done_from_state, image_cache_mount_status_id,
};

// ===========================================================================
// Pure, host-testable helpers (no syscalls). Kept out of the Linux module so
// `cargo test` exercises them on any platform.
// ===========================================================================

/// The filesystem path of the bundled service binary the initramfs ships and
/// the `StartServices` phase exec's.
pub const SVC_PATH: &str = "/usr/bin/svc";

/// The service name used in logs and in the [`BootService`] definition.
pub const SVC_NAME: &str = "svc";

/// Pre-config DHCP materializes DNS into the conventional resolver file.
pub const PRE_CONFIG_RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

/// Translate a machined `MountRequest`'s comma-separated flag string into the
/// kernel `MS_*` bitmask. Pure: a string in, a `u64` out, unit-tested on host.
///
/// Returned as `u64` (not `libc::c_ulong`) so this helper compiles and is
/// testable off-Linux; the Linux glue narrows it to `c_ulong` at the call site.
pub fn ms_flags_from_str(flags: &str) -> u64 {
    // KSPP/Talos mount flag names -> MS_* values. These constants are part of
    // the stable Linux ABI, so hard-coding them keeps this helper host-pure.
    const MS_RDONLY: u64 = 1;
    const MS_NOSUID: u64 = 2;
    const MS_NODEV: u64 = 4;
    const MS_NOEXEC: u64 = 8;
    const MS_RELATIME: u64 = 1 << 21;

    let mut out: u64 = 0;
    for tok in flags.split(',') {
        match tok.trim() {
            "" => {}
            "ro" | "rdonly" => out |= MS_RDONLY,
            "nosuid" => out |= MS_NOSUID,
            "nodev" => out |= MS_NODEV,
            "noexec" => out |= MS_NOEXEC,
            "relatime" => out |= MS_RELATIME,
            // Unknown flag names are ignored rather than failing the mount.
            _ => {}
        }
    }
    out
}

/// Convert a sysctl key (dotted, e.g. `kernel.kptr_restrict`) into its
/// `/proc/sys` path (`/proc/sys/kernel/kptr_restrict`). Pure / host-testable.
pub fn sysctl_path(key: &str) -> String {
    format!("/proc/sys/{}", key.replace('.', "/"))
}

/// Format the "service started" marker the task asserts on.
pub fn service_started_line(name: &str, pid: u32) -> String {
    format!("service {name} started pid={pid}")
}

/// Format the "reaped" marker the task asserts on.
pub fn reaped_line(name: &str, pid: u32, status: i32) -> String {
    format!("reaped {name} pid={pid} status={status}")
}

/// Format a `[seq]` progress line for a task outcome, e.g.
/// `[seq] phase=mountPseudoFs task=mountPseudoFilesystems: ok`.
pub fn seq_task_line(phase: &str, task: &str, outcome: &str) -> String {
    format!("[seq] phase={phase} task={task}: {outcome}")
}

/// Log marker emitted when PID1 starts the boot-owned COSI network bridge.
pub fn boot_cosi_bridge_start_line() -> &'static str {
    "boot-cosi-network-bridge: start"
}

/// Log marker emitted after PID1 seeds the active machine config into COSI.
pub fn boot_cosi_bridge_seeded_line() -> &'static str {
    "boot-cosi-network-bridge: seeded MachineConfigDocument"
}

/// Log marker emitted when no active machine config was cached for the bridge.
pub fn boot_cosi_bridge_no_seed_line() -> &'static str {
    "boot-cosi-network-bridge: no MachineConfigDocument seed"
}

/// Log marker emitted after the boot-owned COSI network bridge quiesces.
pub fn boot_cosi_bridge_stable_line(ticks: usize) -> String {
    format!("boot-cosi-network-bridge: stable ticks={ticks}")
}

/// Format the AWS/pre-config network bootstrap evidence line. Kept pure so the
/// boot contract is testable without opening sockets or mutating interfaces.
pub fn pre_config_bootstrap_line(
    iface: &str,
    dhcp4: bool,
    dhcp6: bool,
    route_metric: u32,
) -> String {
    format!(
        "[net] {iface}: pre-config metadata bootstrap (dhcp4={dhcp4} dhcp6={dhcp6} metric={route_metric})"
    )
}

/// Format the DHCPv4 transaction-start evidence line.
pub fn pre_config_dhcp4_start_line(iface: &str) -> String {
    format!("[net] {iface}: starting pre-config DHCPv4 transaction")
}

/// Format the DHCPv6 transaction-start evidence line.
pub fn pre_config_dhcp6_start_line(iface: &str) -> String {
    format!("[net] {iface}: starting pre-config DHCPv6 rapid-solicit transaction")
}

/// Derive the DHCPv6 IAID the upstream `insomniacslk/dhcp` `NewSolicit`
/// helper derives from a hardware address: the last four MAC bytes, big-endian.
pub fn dhcp6_iaid_from_mac(mac: [u8; 6]) -> u32 {
    u32::from_be_bytes([mac[2], mac[3], mac[4], mac[5]])
}

/// Derive a non-zero 24-bit DHCPv6 transaction id from a MAC address and a
/// caller-provided salt. Production passes a time-derived salt; tests pass a
/// fixed value. The upstream Go library uses a random `TransactionID`; this is
/// the PID1-friendly deterministic replacement.
pub fn dhcp6_transaction_id(mac: [u8; 6], salt: u32) -> [u8; 3] {
    let mut mixed = 0x0044_4836u32 ^ salt; // "DH6" in the low 24 bits.
    for byte in mac {
        mixed = mixed.rotate_left(5) ^ u32::from(byte);
    }
    mixed ^= salt.rotate_left(13);
    let mut id = [
        ((mixed >> 16) & 0xff) as u8,
        ((mixed >> 8) & 0xff) as u8,
        (mixed & 0xff) as u8,
    ];
    if id == [0, 0, 0] {
        id = [0x39, 0x03, 0xf3];
    }
    id
}

/// Parse `/proc/net/if_inet6` and return the first ready link-local IPv6
/// address for `iface`, plus whether the kernel marked it DADFAILED.
///
/// Linux formats each row as:
/// `<32-hex-address> <ifindex> <prefixlen> <scope> <flags> <name>`.
/// We mirror Talos' `waitIPv6LinkReady`: a link-local address is ready once it
/// is not tentative; DADFAILED is logged by the caller but does not block.
pub fn ready_ipv6_link_local_from_proc(
    proc_if_inet6: &str,
    iface: &str,
) -> Option<(Ipv6Addr, bool)> {
    const IPV6_ADDR_LINKLOCAL_SCOPE: u32 = 0x20;
    const IFA_F_DADFAILED: u32 = 0x08;
    const IFA_F_TENTATIVE: u32 = 0x40;

    for line in proc_if_inet6.lines() {
        let mut fields = line.split_whitespace();
        let (Some(addr_hex), Some(_index), Some(_prefix_len), Some(scope), Some(flags), Some(name)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            continue;
        };
        if name != iface || addr_hex.len() != 32 {
            continue;
        }
        let Ok(scope) = u32::from_str_radix(scope, 16) else {
            continue;
        };
        if scope != IPV6_ADDR_LINKLOCAL_SCOPE {
            continue;
        }
        let Ok(flags) = u32::from_str_radix(flags, 16) else {
            continue;
        };
        if flags & IFA_F_TENTATIVE != 0 {
            continue;
        }
        let mut octets = [0u8; 16];
        let mut valid = true;
        for idx in 0..16 {
            match u8::from_str_radix(&addr_hex[idx * 2..idx * 2 + 2], 16) {
                Ok(byte) => octets[idx] = byte,
                Err(_) => {
                    valid = false;
                    break;
                }
            }
        }
        if valid {
            return Some((Ipv6Addr::from(octets), flags & IFA_F_DADFAILED != 0));
        }
    }

    None
}

/// Render a DHCP/operator resolver output into the body PID1 writes during
/// pre-config bootstrap. This is intentionally a thin wrapper around
/// `talos-network`'s source-guided resolver renderer so PID1 owns the sink, not
/// resolver semantics.
pub fn pre_config_resolv_conf_body(resolver: &os_network_domain::ResolverSpec) -> String {
    resolver.render_resolv_conf()
}

/// Atomically materialize a pre-config resolver file via a sibling temp file
/// and rename. The production caller passes [`PRE_CONFIG_RESOLV_CONF_PATH`];
/// tests pass a temp path so the write/rename behavior is covered without
/// touching the host's `/etc`.
pub fn write_pre_config_resolv_conf_at(path: impl AsRef<Path>, body: &str) -> std::io::Result<()> {
    if body.is_empty() {
        return Ok(());
    }

    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut tmp_name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_else(|| OsString::from("resolv.conf"));
    tmp_name.push(".talos-tmp");
    let tmp_path = path.with_file_name(tmp_name);

    match fs::write(&tmp_path, body.as_bytes()).and_then(|()| fs::rename(&tmp_path, path)) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            Err(e)
        }
    }
}

/// Extract the errno from a `os_network_domain` error's Display string. Pure /
/// host-testable.
///
/// The `linux_net` module is dep-light and surfaces failures as
/// `Error::Other("<ctx>: errno <N>")` (socket-level) or
/// `"netlink request failed: errno <N>"` (rtnetlink ACK, where the kernel's
/// negative errno has already been negated back to positive). Both end in
/// `errno <N>`, so we pull the trailing integer (taking its absolute value, to
/// be robust to either sign convention). Returns `-1` when no errno is present,
/// which forces the "unexpected" classification (i.e. fail loudly rather than
/// silently tolerate an unparseable error).
pub fn net_errno(msg: &str) -> i32 {
    match msg.rsplit_once("errno ") {
        Some((_, tail)) => {
            let digits: String = tail
                .trim()
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '-')
                .collect();
            digits.parse::<i32>().map(i32::abs).unwrap_or(-1)
        }
        None => -1,
    }
}

/// Project the image-cache runtime plan PID1 should hand to runtime adapters.
///
/// This is pure and host-testable: machine config supplies the
/// `machine.features.imageCache.localEnabled` flag, and explicit observed
/// block/mount status slices supply cache roots. Privileged filesystem copy and
/// service effects remain in the returned plan for the runtime phase to execute
/// deliberately.
pub fn project_image_cache_runtime_plan(
    raw_config: Option<&str>,
    registryd: RegistrydState,
    volume_statuses: &[VolumeStatus],
    volume_mount_statuses: &[VolumeMountStatusResource],
) -> os_kernel::Result<ImageCacheRuntimePlan> {
    project_image_cache_runtime_plan_with_copy_done(
        raw_config,
        registryd,
        volume_statuses,
        volume_mount_statuses,
        false,
    )
}

/// Project an image-cache runtime plan with known copy completion memory.
pub fn project_image_cache_runtime_plan_with_copy_done(
    raw_config: Option<&str>,
    registryd: RegistrydState,
    volume_statuses: &[VolumeStatus],
    volume_mount_statuses: &[VolumeMountStatusResource],
    copy_done: bool,
) -> os_kernel::Result<ImageCacheRuntimePlan> {
    let local_enabled = match raw_config {
        Some(raw) => os_init_app::config::machine_config_image_cache_local_enabled(raw)?,
        None => false,
    };
    let mut controller = ImageCacheConfigController::new();
    if copy_done {
        controller.mark_cache_copy_done();
    }
    Ok(controller.reconcile(ImageCacheReconcileInput {
        local_enabled,
        registryd,
        volume_statuses,
        volume_mount_statuses,
    }))
}

/// Project an image-cache runtime plan from the boot-owned COSI state.
///
/// Wave89 made the PID1 projection pure by accepting explicit volume and mount
/// observations. This adapter is the next host-safe boundary: it decodes block
/// `VolumeStatus` and `VolumeMountStatus` resources already present in COSI
/// and feeds the same pure projection. If the boot-owned state has no block
/// observations yet, the result remains the documented disabled/no-action plan.
pub fn project_image_cache_runtime_plan_from_cosi_state(
    raw_config: Option<&str>,
    registryd: RegistrydState,
    state: &State,
) -> os_kernel::Result<ImageCacheRuntimePlan> {
    let volume_statuses: Vec<VolumeStatus> = state
        .list(&VolumeStatusResource::kind(), None)
        .iter()
        .filter_map(|resource| VolumeStatusResource::from_resource(resource.as_ref()))
        .map(|resource| resource.status)
        .collect();
    let volume_mount_statuses: Vec<VolumeMountStatusResource> = state
        .list(&VolumeMountStatusResource::kind(), None)
        .iter()
        .filter_map(|resource| VolumeMountStatusResource::from_resource(resource.as_ref()))
        .collect();
    let copy_done = image_cache_copy_done_from_state(state);

    project_image_cache_runtime_plan_with_copy_done(
        raw_config,
        registryd,
        &volume_statuses,
        &volume_mount_statuses,
        copy_done,
    )
}

/// Apply a post-runtime registryd observation back into boot-owned COSI state.
///
/// The boot bridge computes block/copy/finalizer intent before service effects.
/// After PID1 starts/probes registryd, only the registryd-dependent config
/// status/action should change; the existing mount/copy/finalizer plan stays
/// grounded in the earlier block observations.
pub fn apply_image_cache_runtime_observation_to_state(
    state: &mut State,
    plan: &ImageCacheRuntimePlan,
    registryd: RegistrydState,
) -> os_cosi_domain::StoreResult<ImageCacheRuntimePlan> {
    let observed_plan = plan.reproject_after_registryd_observation(registryd);
    apply_image_cache_plan_to_state(state, &observed_plan)?;
    Ok(observed_plan)
}

/// Stable log/status label for image-cache copy runtime effects.
pub fn image_cache_copy_execution_status_label(
    status: ImageCacheCopyExecutionStatus,
) -> &'static str {
    match status {
        ImageCacheCopyExecutionStatus::NoPlan => "no-plan",
        ImageCacheCopyExecutionStatus::DisabledByEnvironment => "disabled-by-environment",
        ImageCacheCopyExecutionStatus::DisabledByGate => "disabled-by-gate",
        ImageCacheCopyExecutionStatus::Copied => "copied",
    }
}

/// Stable PID1 log line for boot-owned image-cache copy completion state.
pub fn image_cache_copy_state_line(done: bool) -> String {
    format!("image-cache-runtime: copyState done={done}")
}

/// Build the boot-runtime copy adapter for a known privileged boundary.
///
/// PID1 is allowed to mutate image-cache storage only after the runtime has
/// positively classified itself as a privileged VM/metal boot. Host-safe tests
/// and container/sandbox boots keep the source-shaped copy intent observable
/// while suppressing filesystem mutation.
pub fn image_cache_copy_adapter_for_boot_privilege(
    privileged_vm_boot: bool,
) -> ImageCacheCopyRuntimeAdapter {
    if privileged_vm_boot {
        ImageCacheCopyRuntimeAdapter::new(
            ImageCacheCopyRuntimeEnvironment::VmPrivileged,
            ImageCacheCopyGate::Enabled,
        )
    } else {
        ImageCacheCopyRuntimeAdapter::new(
            ImageCacheCopyRuntimeEnvironment::HostSafe,
            ImageCacheCopyGate::Disabled,
        )
    }
}

/// Result of running boot-owned image-cache runtime adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCacheRuntimeAdapterOutcome {
    /// Source-shaped image-cache copy execution report.
    pub copy_report: ImageCacheCopyReport,
    /// Source-shaped registryd service-manager execution report.
    pub report: RegistrydServiceReport,
    /// Registryd state projected from the caller-owned supervisor.
    pub registryd_state: RegistrydState,
    /// Whether boot-owned COSI records successful image-cache copy completion.
    pub copy_done: bool,
    /// Runtime plan after applying the post-adapter registryd observation.
    pub observed_plan: ImageCacheRuntimePlan,
}

/// Run image-cache runtime adapters against caller-owned supervisor state using
/// the host-safe copy boundary.
pub fn run_image_cache_runtime_adapters_with_supervisor(
    plan: &ImageCacheRuntimePlan,
    state: &mut State,
    supervisor: &mut Supervisor,
    launcher: &mut dyn ServiceLauncher,
) -> MachinedResult<ImageCacheRuntimeAdapterOutcome> {
    run_image_cache_runtime_adapters_with_supervisor_and_copy_adapter(
        plan,
        state,
        supervisor,
        launcher,
        ImageCacheCopyRuntimeAdapter::default(),
    )
}

/// Run image-cache runtime adapters against caller-owned supervisor state.
///
/// Source Talos first executes the privileged image-cache copy effect when both
/// ISO and disk roots are ready, then loads `registryd` into a service manager,
/// starts it, and leaves the loaded service available for HTTP serving. Keeping
/// the supervisor owned by the boot runtime, rather than constructing a
/// throwaway manager here, preserves the loaded
/// [`os_runtime_cri_domain::RegistrydRuntimeService`] payload for host-safe request
/// handling and later socket-backed execution.
pub fn run_image_cache_runtime_adapters_with_supervisor_and_copy_adapter(
    plan: &ImageCacheRuntimePlan,
    state: &mut State,
    supervisor: &mut Supervisor,
    launcher: &mut dyn ServiceLauncher,
    copy_adapter: ImageCacheCopyRuntimeAdapter,
) -> MachinedResult<ImageCacheRuntimeAdapterOutcome> {
    let copy_report = copy_adapter.execute(plan).map_err(|err| {
        MachinedError::task_failed("imageCacheCopyRuntimeAdapter", err.to_string())
    })?;
    apply_image_cache_copy_report_to_state(state, &copy_report)
        .map_err(|err| MachinedError::task_failed("imageCacheCopyState", err.to_string()))?;
    let copy_done = image_cache_copy_done_from_state(state);
    let post_copy_plan = plan.reproject_after_copy_report(&copy_report);

    let (report, registryd_state) = {
        let mut manager = SupervisorRegistrydServiceManager::new(supervisor, launcher);
        let report = RegistrydRuntimeAdapter
            .execute(&post_copy_plan, &mut manager)
            .map_err(|err| {
                MachinedError::task_failed("imageCacheRuntimeAdapters", err.to_string())
            })?;
        (report, manager.registryd_state())
    };
    let observed_plan =
        apply_image_cache_runtime_observation_to_state(state, &post_copy_plan, registryd_state)
            .map_err(|err| {
                MachinedError::task_failed("imageCacheRuntimeObservation", err.to_string())
            })?;

    Ok(ImageCacheRuntimeAdapterOutcome {
        copy_report,
        report,
        registryd_state,
        copy_done,
        observed_plan,
    })
}

/// Seed boot-owned COSI with the declared `IMAGECACHE` block volume status.
///
/// Talos source config declares the image-cache disk as a source-managed
/// `VolumeConfig` named `IMAGECACHE`. PID1 does not perform block provisioning in
/// this bridge; it only hydrates the same initial `Waiting` status the block
/// manager would start with so image-cache projection can observe a declared
/// disk volume without inventing a mount root or starting registryd early.
pub fn hydrate_declared_image_cache_block_state(
    raw_config: Option<&str>,
    state: &mut State,
) -> os_kernel::Result<bool> {
    let Some(raw_config) = raw_config.filter(|raw| !raw.trim().is_empty()) else {
        return Ok(false);
    };
    if !os_init_app::config::machine_config_image_cache_local_enabled(raw_config)? {
        return Ok(false);
    }

    let manager = os_init_app::config::machine_config_volume_manager(raw_config)?;
    let Some(volume) = manager.volume(IMAGE_CACHE_VOLUME_ID) else {
        return Ok(false);
    };
    let resource = VolumeStatusResource::new(VolumeStatus::new(volume.config.clone()))
        .map_err(os_kernel::Error::from)?;
    let key = resource.metadata().key();
    if state.contains(&key) {
        return Ok(false);
    }

    state.create(Box::new(resource)).map_err(|err| {
        os_kernel::Error::invalid_state(format!(
            "boot COSI image-cache VolumeStatus create {key}: {err}"
        ))
    })?;
    Ok(true)
}

/// Seed boot-owned COSI with ready image-cache mount observations for bundled roots.
///
/// The minimal Rust PID1 boot harness does not yet run the full Talos block
/// manager. When image cache is enabled and the canonical disk/ISO roots are
/// already present in the initramfs, this bridge records those roots as the
/// same ready `VolumeStatus` + `VolumeMountStatus` observations the source CRI
/// controller consumes. Missing paths do not synthesize roots: the runtime plan
/// remains disabled/pending until a real observation exists.
pub fn hydrate_bootstrap_image_cache_mount_state(
    raw_config: Option<&str>,
    state: &mut State,
) -> os_kernel::Result<usize> {
    hydrate_bootstrap_image_cache_mount_state_with_probe(raw_config, state, |path| {
        Path::new(path).is_dir()
    })
}

/// Host-testable variant of [`hydrate_bootstrap_image_cache_mount_state`].
pub fn hydrate_bootstrap_image_cache_mount_state_with_probe(
    raw_config: Option<&str>,
    state: &mut State,
    path_exists: impl Fn(&str) -> bool,
) -> os_kernel::Result<usize> {
    let Some(raw_config) = raw_config.filter(|raw| !raw.trim().is_empty()) else {
        return Ok(0);
    };
    if !os_init_app::config::machine_config_image_cache_local_enabled(raw_config)? {
        return Ok(0);
    }

    let mut seeded = 0usize;
    if path_exists(IMAGE_CACHE_DISK_MOUNT_POINT) {
        upsert_image_cache_bootstrap_volume_status(state, IMAGE_CACHE_DISK_VOLUME_ID)?;
        upsert_image_cache_bootstrap_mount_status(
            state,
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_MOUNT_POINT,
            false,
        )?;
        seeded += 2;
    }

    let iso_payload_root = format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache");
    if path_exists(&iso_payload_root) {
        upsert_image_cache_bootstrap_volume_status(state, IMAGE_CACHE_ISO_VOLUME_ID)?;
        upsert_image_cache_bootstrap_mount_status(
            state,
            IMAGE_CACHE_ISO_VOLUME_ID,
            IMAGE_CACHE_ISO_MOUNT_POINT,
            true,
        )?;
        seeded += 2;
    }

    Ok(seeded)
}

fn upsert_image_cache_bootstrap_volume_status(
    state: &mut State,
    volume_id: &str,
) -> os_kernel::Result<()> {
    let mut status = VolumeStatus::new(VolumeConfig::partition(volume_id, volume_id, 0));
    status.phase = VolumePhase::Ready;
    let mut desired = VolumeStatusResource::new(status).map_err(os_kernel::Error::from)?;
    let key = desired.metadata().key();

    if let Some(existing) = state.get(&key) {
        *desired.metadata_mut() = existing.metadata().clone();
        state
            .update(Box::new(desired), existing.metadata().version())
            .map_err(|err| {
                os_kernel::Error::invalid_state(format!(
                    "boot COSI image-cache VolumeStatus update {key}: {err}"
                ))
            })?;
    } else {
        state.create(Box::new(desired)).map_err(|err| {
            os_kernel::Error::invalid_state(format!(
                "boot COSI image-cache VolumeStatus create {key}: {err}"
            ))
        })?;
    }

    Ok(())
}

fn upsert_image_cache_bootstrap_mount_status(
    state: &mut State,
    volume_id: &str,
    target: &str,
    read_only: bool,
) -> os_kernel::Result<()> {
    let mount_id = image_cache_mount_status_id(volume_id);
    let mut desired = VolumeMountStatusResource::new(
        mount_id.clone(),
        VolumeMountStatusSpec::new(volume_id, IMAGE_CACHE_CONTROLLER_NAME, target)
            .with_read_only(read_only),
    )
    .map_err(os_kernel::Error::from)?;
    let key = desired.metadata().key();

    if let Some(existing) = state.get(&key) {
        *desired.metadata_mut() = existing.metadata().clone();
        state
            .update(Box::new(desired), existing.metadata().version())
            .map_err(|err| {
                os_kernel::Error::invalid_state(format!(
                    "boot COSI image-cache VolumeMountStatus update {key}: {err}"
                ))
            })?;
    } else {
        state.create(Box::new(desired)).map_err(|err| {
            os_kernel::Error::invalid_state(format!(
                "boot COSI image-cache VolumeMountStatus create {key}: {err}"
            ))
        })?;
    }

    Ok(())
}

/// Fail-safe registryd health probe timeout for PID1 boot.
const PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT: Duration = Duration::from_secs(15);

/// Source-shaped PID1 registryd launch boundary.
///
/// Talos source (`internal/app/machined/pkg/system/services/registryd.go`)
/// defines a service with ID `registryd`, starts it through an in-process
/// goroutine runner, and only reports health after the registry HTTP endpoint
/// answers `/healthz`.
pub fn pid1_registryd_launch_result(id: &str) -> os_kernel::Result<bool> {
    pid1_registryd_launch_result_at(
        id,
        REGISTRYD_LISTEN_ADDRESS,
        REGISTRYD_HEALTH_PATH,
        PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
    )
}

/// Payload-free PID1 registryd launch boundary with injectable endpoint.
///
/// The generic service-launch path has no loaded image-cache runtime payload,
/// so it only reports healthy when an already-serving loopback registryd
/// endpoint answers `/healthz`. Payload-aware starts use
/// [`ServiceLauncher::launch_registryd_runtime_service`] and the runtime
/// service launcher below.
pub fn pid1_registryd_launch_result_at(
    id: &str,
    address: &str,
    path: &str,
    timeout: Duration,
) -> os_kernel::Result<bool> {
    if id != REGISTRYD_SERVICE_ID {
        return Err(os_kernel::Error::unsupported(format!(
            "unsupported registryd service id {id}"
        )));
    }

    let Some(status_code) = registryd_http_health_status_at(address, path, timeout)? else {
        return Ok(false);
    };
    Ok(RegistrydHealthProbe::source().accepts_status(status_code))
}

/// Serve one host-safe registryd request through PID1's retained supervisor.
///
/// This is the PID1-facing counterpart to the loopback launch health probe:
/// the service id must still be Talos' source `registryd` id, but the request
/// is delegated to the boot-retained [`Supervisor`] instead of opening a
/// socket. Later socket-backed boot serving can use this bridge as the
/// deterministic request boundary.
pub fn pid1_registryd_request_result(
    supervisor: &Supervisor,
    id: &str,
    method: &str,
    target: &str,
) -> os_kernel::Result<Option<RegistrydContentResponse>> {
    if id != REGISTRYD_SERVICE_ID {
        return Err(os_kernel::Error::unsupported(format!(
            "unsupported registryd service id {id}"
        )));
    }

    Ok(supervisor.handle_registryd_request(method, target))
}

/// Clone PID1's boot-retained registryd runtime-service payload.
///
/// The source registryd runner owns a request handler built from image-cache
/// roots. This helper keeps Talos' `registryd` service-id validation at the
/// PID1 boundary while handing later serving code an owned payload snapshot.
pub fn pid1_registryd_runtime_service_snapshot(
    supervisor: &Supervisor,
    id: &str,
) -> os_kernel::Result<Option<RegistrydRuntimeService>> {
    if id != REGISTRYD_SERVICE_ID {
        return Err(os_kernel::Error::unsupported(format!(
            "unsupported registryd service id {id}"
        )));
    }

    Ok(supervisor.registryd_runtime_service().cloned())
}

/// Serve one host-safe registryd request as HTTP response bytes.
///
/// This composes PID1's retained-supervisor request bridge with the
/// source-shaped response serialization model. It is the deterministic seam a
/// later loopback listener can write to a stream.
pub fn pid1_registryd_response_bytes(
    supervisor: &Supervisor,
    id: &str,
    method: &str,
    target: &str,
) -> os_kernel::Result<Option<Vec<u8>>> {
    pid1_registryd_response_bytes_for_request_headers(
        supervisor,
        id,
        method,
        target,
        os_runtime_cri_domain::RegistrydSourceRequestHeaders::default(),
    )
}

/// Serve one host-safe registryd request as HTTP response bytes with a Range header.
pub fn pid1_registryd_response_bytes_for_range(
    supervisor: &Supervisor,
    id: &str,
    method: &str,
    target: &str,
    range_header: Option<&str>,
) -> os_kernel::Result<Option<Vec<u8>>> {
    pid1_registryd_response_bytes_for_request_headers(
        supervisor,
        id,
        method,
        target,
        os_runtime_cri_domain::RegistrydSourceRequestHeaders {
            range: range_header,
            ..os_runtime_cri_domain::RegistrydSourceRequestHeaders::default()
        },
    )
}

/// Serve one host-safe registryd request as HTTP response bytes with modeled headers.
pub fn pid1_registryd_response_bytes_for_request_headers(
    supervisor: &Supervisor,
    id: &str,
    method: &str,
    target: &str,
    headers: os_runtime_cri_domain::RegistrydSourceRequestHeaders<'_>,
) -> os_kernel::Result<Option<Vec<u8>>> {
    Ok(
        pid1_registryd_request_result(supervisor, id, method, target)?
            .map(|response| response.source_http_response_bytes_for_request_headers(headers)),
    )
}

/// Serve one host-safe registryd runtime-service request as HTTP response bytes.
///
/// This is the service-owned counterpart to the retained-supervisor byte
/// bridge. It keeps the same service-id gate while delegating directly to the
/// cloneable runtime payload loaded from the source-shaped image-cache plan.
pub fn pid1_registryd_runtime_service_response_bytes(
    service: &RegistrydRuntimeService,
    id: &str,
    method: &str,
    target: &str,
) -> os_kernel::Result<Option<Vec<u8>>> {
    pid1_registryd_runtime_service_response_bytes_for_request_headers(
        service,
        id,
        method,
        target,
        os_runtime_cri_domain::RegistrydSourceRequestHeaders::default(),
    )
}

/// Serve one host-safe runtime-service request as response bytes with a Range header.
pub fn pid1_registryd_runtime_service_response_bytes_for_range(
    service: &RegistrydRuntimeService,
    id: &str,
    method: &str,
    target: &str,
    range_header: Option<&str>,
) -> os_kernel::Result<Option<Vec<u8>>> {
    pid1_registryd_runtime_service_response_bytes_for_request_headers(
        service,
        id,
        method,
        target,
        os_runtime_cri_domain::RegistrydSourceRequestHeaders {
            range: range_header,
            ..os_runtime_cri_domain::RegistrydSourceRequestHeaders::default()
        },
    )
}

/// Serve one host-safe runtime-service request as response bytes with modeled headers.
pub fn pid1_registryd_runtime_service_response_bytes_for_request_headers(
    service: &RegistrydRuntimeService,
    id: &str,
    method: &str,
    target: &str,
    headers: os_runtime_cri_domain::RegistrydSourceRequestHeaders<'_>,
) -> os_kernel::Result<Option<Vec<u8>>> {
    if id != REGISTRYD_SERVICE_ID {
        return Err(os_kernel::Error::unsupported(format!(
            "unsupported registryd service id {id}"
        )));
    }

    Ok(service
        .handle_request(method, target)
        .map(|response| response.source_http_response_bytes_for_request_headers(headers)))
}

fn registryd_read_http_request<S>(stream: &mut S) -> os_kernel::Result<Vec<u8>>
where
    S: Read,
{
    let mut request = Vec::new();
    let mut buffer = [0_u8; 512];
    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|err| os_kernel::Error::Other(format!("registryd request read: {err}")))?;
        if read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..read]);

        if request.windows(4).any(|window| window == b"\r\n\r\n")
            || request.windows(2).any(|window| window == b"\n\n")
        {
            break;
        }

        if request.len() > 8192 {
            return Err(os_kernel::Error::invalid(
                "registryd HTTP request exceeds 8192 bytes",
            ));
        }
    }

    Ok(request)
}

/// Serve one source-shaped registryd HTTP request from an injectable stream.
///
/// This is the host-safe stream seam below the future loopback listener: it
/// reads one HTTP request line, delegates through PID1's retained-supervisor
/// response-byte bridge, and writes the response bytes when the route is
/// handled.
pub fn pid1_registryd_serve_http_once<S>(
    supervisor: &Supervisor,
    id: &str,
    stream: &mut S,
) -> os_kernel::Result<bool>
where
    S: Read + Write,
{
    let request = registryd_read_http_request(stream)?;
    let (method, target) = registryd_http_request_method_target(&request)?;
    let range_header = registryd_http_request_header(&request, "Range");
    let if_match = registryd_http_request_header(&request, "If-Match");
    let if_unmodified_since = registryd_http_request_header(&request, "If-Unmodified-Since");
    let if_none_match = registryd_http_request_header(&request, "If-None-Match");
    let if_modified_since = registryd_http_request_header(&request, "If-Modified-Since");
    let if_range = registryd_http_request_header(&request, "If-Range");
    let Some(response) = pid1_registryd_response_bytes_for_request_headers(
        supervisor,
        id,
        method,
        target,
        os_runtime_cri_domain::RegistrydSourceRequestHeaders {
            range: range_header.as_deref(),
            if_match: if_match.as_deref(),
            if_unmodified_since: if_unmodified_since.as_deref(),
            if_none_match: if_none_match.as_deref(),
            if_modified_since: if_modified_since.as_deref(),
            if_range: if_range.as_deref(),
        },
    )?
    else {
        return Ok(false);
    };

    stream
        .write_all(&response)
        .map_err(|err| os_kernel::Error::Other(format!("registryd response write: {err}")))?;
    stream
        .flush()
        .map_err(|err| os_kernel::Error::Other(format!("registryd response flush: {err}")))?;
    Ok(true)
}

/// Serve one registryd HTTP request from a cloneable runtime-service payload.
///
/// This lets PID1 HTTP serving use the service data selected at registryd load
/// time without needing a live borrow of the boot supervisor during stream I/O.
pub fn pid1_registryd_runtime_service_serve_http_once<S>(
    service: &RegistrydRuntimeService,
    id: &str,
    stream: &mut S,
) -> os_kernel::Result<bool>
where
    S: Read + Write,
{
    let request = registryd_read_http_request(stream)?;
    let (method, target) = registryd_http_request_method_target(&request)?;
    let range_header = registryd_http_request_header(&request, "Range");
    let if_match = registryd_http_request_header(&request, "If-Match");
    let if_unmodified_since = registryd_http_request_header(&request, "If-Unmodified-Since");
    let if_none_match = registryd_http_request_header(&request, "If-None-Match");
    let if_modified_since = registryd_http_request_header(&request, "If-Modified-Since");
    let if_range = registryd_http_request_header(&request, "If-Range");
    let Some(response) = pid1_registryd_runtime_service_response_bytes_for_request_headers(
        service,
        id,
        method,
        target,
        os_runtime_cri_domain::RegistrydSourceRequestHeaders {
            range: range_header.as_deref(),
            if_match: if_match.as_deref(),
            if_unmodified_since: if_unmodified_since.as_deref(),
            if_none_match: if_none_match.as_deref(),
            if_modified_since: if_modified_since.as_deref(),
            if_range: if_range.as_deref(),
        },
    )?
    else {
        return Ok(false);
    };

    stream
        .write_all(&response)
        .map_err(|err| os_kernel::Error::Other(format!("registryd response write: {err}")))?;
    stream
        .flush()
        .map_err(|err| os_kernel::Error::Other(format!("registryd response flush: {err}")))?;
    Ok(true)
}

/// Accept one loopback registryd connection and serve it from a runtime payload.
///
/// This mirrors [`pid1_registryd_accept_http_once`] while using a
/// service-owned payload that can be cloned when PID1 starts registryd serving.
pub fn pid1_registryd_runtime_service_accept_http_once(
    service: &RegistrydRuntimeService,
    id: &str,
    listener: &std::net::TcpListener,
    timeout: Duration,
) -> os_kernel::Result<bool> {
    pid1_registryd_runtime_service_accept_http_once_with_stop(service, id, listener, timeout, None)
}

fn pid1_registryd_runtime_service_accept_http_once_with_stop(
    service: &RegistrydRuntimeService,
    id: &str,
    listener: &std::net::TcpListener,
    timeout: Duration,
    stop: Option<&std::sync::atomic::AtomicBool>,
) -> os_kernel::Result<bool> {
    listener
        .set_nonblocking(true)
        .map_err(|err| os_kernel::Error::Other(format!("registryd listener configure: {err}")))?;

    let deadline = std::time::Instant::now() + timeout;
    let result = loop {
        if stop.is_some_and(|stop| stop.load(std::sync::atomic::Ordering::Relaxed)) {
            break Ok(false);
        }

        match listener.accept() {
            Ok((mut stream, _peer)) => {
                if stop.is_some_and(|stop| stop.load(std::sync::atomic::Ordering::Relaxed)) {
                    break Ok(false);
                }
                let _ = stream.set_nonblocking(false);
                let _ = stream.set_read_timeout(Some(timeout));
                let _ = stream.set_write_timeout(Some(timeout));
                break pid1_registryd_runtime_service_serve_http_once(service, id, &mut stream);
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline
                    || stop.is_some_and(|stop| stop.load(std::sync::atomic::Ordering::Relaxed))
                {
                    break Ok(false);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(err) => {
                break Err(os_kernel::Error::Other(format!(
                    "registryd listener accept: {err}"
                )));
            }
        }
    };

    let _ = listener.set_nonblocking(false);
    result
}

/// Serve up to `max_connections` from a registryd runtime-service payload.
///
/// This composes the cloned-payload stream seam into the same bounded
/// per-connection loop used by PID1's supervisor-backed serving tests.
pub fn pid1_registryd_runtime_service_serve_http_bounded(
    service: &RegistrydRuntimeService,
    id: &str,
    listener: &std::net::TcpListener,
    max_connections: usize,
    timeout: Duration,
) -> os_kernel::Result<usize> {
    let mut served = 0;
    for _ in 0..max_connections {
        if !pid1_registryd_runtime_service_accept_http_once(service, id, listener, timeout)? {
            break;
        }
        served += 1;
    }
    Ok(served)
}

/// PID1-owned registryd runtime-service server thread.
///
/// Source registryd runs as an in-process goroutine owned by machined. This
/// handle gives tests an explicit stop/join point while allowing PID1 to detach
/// the thread for its process lifetime after a successful health probe.
#[derive(Debug)]
pub struct Pid1RegistrydRuntimeServer {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    wake_address: SocketAddr,
    join: Option<std::thread::JoinHandle<os_kernel::Result<usize>>>,
}

impl Pid1RegistrydRuntimeServer {
    /// Ask the registryd server loop to stop and return the number of served requests.
    pub fn stop(mut self) -> os_kernel::Result<usize> {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = TcpStream::connect_timeout(&self.wake_address, Duration::from_millis(50));
        let Some(join) = self.join.take() else {
            return Ok(0);
        };

        join.join()
            .map_err(|_| os_kernel::Error::Other("registryd server thread panicked".into()))?
    }
}

/// Spawn a PID1-owned registryd runtime-service HTTP loop on a caller-owned listener.
///
/// The loop serves from the loaded [`RegistrydRuntimeService`] payload and
/// repeatedly accepts source-shaped requests until its stop flag is set. A
/// caller-owned listener keeps binding policy explicit for host tests and for
/// PID1's source listen address.
pub fn pid1_registryd_runtime_service_spawn_http_server_at(
    service: RegistrydRuntimeService,
    id: &str,
    listener: std::net::TcpListener,
    timeout: Duration,
) -> os_kernel::Result<Pid1RegistrydRuntimeServer> {
    if id != REGISTRYD_SERVICE_ID {
        return Err(os_kernel::Error::unsupported(format!(
            "unsupported registryd service id {id}"
        )));
    }
    let wake_address = listener.local_addr().map_err(|err| {
        os_kernel::Error::Other(format!("registryd listener local address: {err}"))
    })?;

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_stop = std::sync::Arc::clone(&stop);
    let service_id = id.to_string();
    let join = std::thread::spawn(move || {
        let mut served = 0;
        while !thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
            if pid1_registryd_runtime_service_accept_http_once_with_stop(
                &service,
                &service_id,
                &listener,
                timeout,
                Some(&thread_stop),
            )? {
                served += 1;
            }
        }
        Ok(served)
    });

    Ok(Pid1RegistrydRuntimeServer {
        stop,
        wake_address,
        join: Some(join),
    })
}

/// Start registryd serving from a runtime payload, then probe source health.
///
/// Source registryd becomes healthy only after its HTTP endpoint answers
/// `/healthz`. This helper composes the payload-owned serving loop with the
/// existing loopback health probe and returns both the readiness result and the
/// server handle that keeps the endpoint alive.
pub fn pid1_registryd_runtime_service_launch_result_at(
    service: &RegistrydRuntimeService,
    id: &str,
    listener: std::net::TcpListener,
    health_address: &str,
    timeout: Duration,
) -> os_kernel::Result<(bool, Pid1RegistrydRuntimeServer)> {
    let server = pid1_registryd_runtime_service_spawn_http_server_at(
        service.clone(),
        id,
        listener,
        timeout,
    )?;
    let deadline = std::time::Instant::now() + timeout;
    let mut healthy = false;
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let probe_timeout = remaining;
        if registryd_http_health_status_at(health_address, REGISTRYD_HEALTH_PATH, probe_timeout)?
            .is_some_and(|status| RegistrydHealthProbe::source().accepts_status(status))
        {
            healthy = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok((healthy, server))
}

/// Host-safe registryd launcher that starts a runtime payload on loopback.
///
/// This is the testable PID1 launcher core: it implements machined's
/// [`ServiceLauncher`] hook, consumes a caller-owned listener on first
/// registryd start, keeps the payload-backed server handle alive, and exposes
/// an explicit stop/join method for tests.
#[derive(Debug)]
pub struct Pid1RegistrydRuntimeServiceLauncher {
    listener: Option<std::net::TcpListener>,
    health_address: String,
    timeout: Duration,
    registryd_server: Option<Pid1RegistrydRuntimeServer>,
}

impl Pid1RegistrydRuntimeServiceLauncher {
    /// Build a launcher from an already-bound loopback listener.
    pub fn new(
        listener: std::net::TcpListener,
        health_address: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            listener: Some(listener),
            health_address: health_address.into(),
            timeout,
            registryd_server: None,
        }
    }

    /// Bind a source registryd listen address and use the bound socket for health probes.
    pub fn bind_at(listen_address: &str, timeout: Duration) -> os_kernel::Result<Self> {
        let listener = std::net::TcpListener::bind(listen_address).map_err(|err| {
            os_kernel::Error::Other(format!("registryd listen {listen_address}: {err}"))
        })?;
        let health_address = listener.local_addr().map_err(|err| {
            os_kernel::Error::Other(format!("registryd local address {listen_address}: {err}"))
        })?;

        Ok(Self::new(listener, health_address.to_string(), timeout))
    }

    /// Return the address the launcher probes for registryd health.
    pub fn health_address(&self) -> &str {
        &self.health_address
    }

    /// Stop the registryd server started by this launcher, returning served requests.
    pub fn stop_registryd(mut self) -> os_kernel::Result<usize> {
        match self.registryd_server.take() {
            Some(server) => server.stop(),
            None => Ok(0),
        }
    }
}

impl ServiceLauncher for Pid1RegistrydRuntimeServiceLauncher {
    fn launch(&mut self, id: &str) -> MachinedResult<bool> {
        pid1_registryd_launch_result_at(
            id,
            &self.health_address,
            REGISTRYD_HEALTH_PATH,
            self.timeout,
        )
        .map_err(|err| MachinedError::service_error(id, err.to_string()))
    }

    fn launch_registryd_runtime_service(
        &mut self,
        id: &str,
        service: &RegistrydRuntimeService,
    ) -> MachinedResult<bool> {
        let listener = self
            .listener
            .take()
            .ok_or_else(|| MachinedError::service_error(id, "registryd listener consumed"))?;
        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            service,
            id,
            listener,
            &self.health_address,
            self.timeout,
        )
        .map_err(|err| MachinedError::service_error(id, err.to_string()))?;

        if healthy {
            self.registryd_server = Some(server);
        } else {
            let _ = server.stop();
        }

        Ok(healthy)
    }

    fn stop(&mut self, id: &str) -> MachinedResult<()> {
        if id != REGISTRYD_SERVICE_ID {
            return Err(MachinedError::service_error(
                id,
                format!("unsupported registryd service id {id}"),
            ));
        }

        if let Some(server) = self.registryd_server.take() {
            server
                .stop()
                .map(|_| ())
                .map_err(|err| MachinedError::service_error(id, err.to_string()))?;
        }

        Ok(())
    }
}

/// Accept one loopback registryd connection and serve one HTTP request.
///
/// This models the source `http.Server` boundary in a bounded form suitable
/// for boot orchestration tests. A caller-owned listener keeps binding policy
/// explicit while this helper owns only one accept/read/write exchange.
pub fn pid1_registryd_accept_http_once(
    supervisor: &Supervisor,
    id: &str,
    listener: &std::net::TcpListener,
    timeout: Duration,
) -> os_kernel::Result<bool> {
    listener
        .set_nonblocking(true)
        .map_err(|err| os_kernel::Error::Other(format!("registryd listener configure: {err}")))?;

    let deadline = std::time::Instant::now() + timeout;
    let result = loop {
        match listener.accept() {
            Ok((mut stream, _peer)) => {
                let _ = stream.set_nonblocking(false);
                let _ = stream.set_read_timeout(Some(timeout));
                let _ = stream.set_write_timeout(Some(timeout));
                break pid1_registryd_serve_http_once(supervisor, id, &mut stream);
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline {
                    break Ok(false);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(err) => {
                break Err(os_kernel::Error::Other(format!(
                    "registryd listener accept: {err}"
                )));
            }
        }
    };

    let _ = listener.set_nonblocking(false);
    result
}

/// Serve up to `max_connections` registryd HTTP connections on a listener.
///
/// Source registryd uses a long-lived `http.Server`; this finite helper keeps
/// the same per-connection boundary but makes tests and boot slices explicit
/// about when the loop returns.
pub fn pid1_registryd_serve_http_bounded(
    supervisor: &Supervisor,
    id: &str,
    listener: &std::net::TcpListener,
    max_connections: usize,
    timeout: Duration,
) -> os_kernel::Result<usize> {
    let mut served = 0;
    for _ in 0..max_connections {
        if !pid1_registryd_accept_http_once(supervisor, id, listener, timeout)? {
            break;
        }
        served += 1;
    }
    Ok(served)
}

fn registryd_http_request_method_target(request: &[u8]) -> os_kernel::Result<(&str, &str)> {
    let line_end = request
        .windows(2)
        .position(|window| window == b"\r\n")
        .or_else(|| request.iter().position(|byte| *byte == b'\n'))
        .ok_or_else(|| os_kernel::Error::parse("registryd HTTP request missing request line"))?;
    let line = &request[..line_end];
    let line = std::str::from_utf8(line).map_err(|err| {
        os_kernel::Error::parse(format!("registryd HTTP request line is not UTF-8: {err}"))
    })?;

    let mut parts = line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| os_kernel::Error::parse("registryd HTTP request missing method"))?;
    let target = parts
        .next()
        .ok_or_else(|| os_kernel::Error::parse("registryd HTTP request missing target"))?;
    let version = parts
        .next()
        .ok_or_else(|| os_kernel::Error::parse("registryd HTTP request missing version"))?;
    if parts.next().is_some() || !version.starts_with("HTTP/") {
        return Err(os_kernel::Error::parse(format!(
            "registryd HTTP request line has invalid version {version}"
        )));
    }

    Ok((method, target))
}

fn registryd_http_request_header(request: &[u8], header_name: &str) -> Option<String> {
    let request = std::str::from_utf8(request).ok()?;
    for line in request.lines().skip(1) {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case(header_name) {
            return Some(value.trim().to_string());
        }
    }
    None
}

/// Probe one HTTP endpoint and return its status code when a response is read.
pub fn registryd_http_health_status_at(
    address: &str,
    path: &str,
    timeout: Duration,
) -> os_kernel::Result<Option<u16>> {
    let socket_addr: SocketAddr = address.parse().map_err(|err| {
        os_kernel::Error::parse(format!("registryd health address {address}: {err}"))
    })?;

    let mut stream = match TcpStream::connect_timeout(&socket_addr, timeout) {
        Ok(stream) => stream,
        Err(_err) => return Ok(None),
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));

    let request = format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    if stream.write_all(request.as_bytes()).is_err() || stream.flush().is_err() {
        return Ok(None);
    }

    let mut response = [0_u8; 512];
    let bytes = match stream.read(&mut response) {
        Ok(0) | Err(_) => return Ok(None),
        Ok(bytes) => bytes,
    };
    let response = match std::str::from_utf8(&response[..bytes]) {
        Ok(response) => response,
        Err(_) => return Ok(None),
    };

    Ok(registryd_status_code_from_status_line(
        response.lines().next().unwrap_or_default(),
    ))
}

/// Parse an HTTP status line into its numeric status code.
pub fn registryd_status_code_from_status_line(line: &str) -> Option<u16> {
    let mut parts = line.split_whitespace();
    let version = parts.next()?;
    if !version.starts_with("HTTP/") {
        return None;
    }
    parts.next()?.parse().ok()
}

// ===========================================================================
// Linux PID 1 implementation
// ===========================================================================

#[cfg(target_os = "linux")]
mod init {
    use std::ffi::{CStr, CString};
    use std::fs;
    use std::io::Write;
    use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use os_controllers_domain::network::LinkStatusResource;
    use os_controllers_domain::{Domain, MachineConfigDocument, RegistryBuilder};
    use os_cosi_domain::State;
    use os_init_app::config::{StaticAddress, try_early_config};
    use os_init_app::platform_config::{
        FileConfigStore, pre_config_network_bootstrap, resolve_config,
    };
    use os_init_app::{DEFAULT_HOSTNAME, MACHINE_CONFIG_PATH, PROC_CMDLINE_PATH};

    use os_kernel::MachineType;
    use os_kernel_abi::KernelNet;
    use os_machined_domain::boot::{
        BootPhaseId, BootRuntimeMode, BootSequencer, BootService, MountRequest, ProgressLogger,
        RestartPolicy, Runtime, SysctlOutcome, TaskOutcome, detect_runtime_mode, mount_skip_reason,
        privileged_op_skip_reason, sysctl_skip_reason,
    };
    use os_machined_domain::error::{MachinedError, Result as MResult};
    use os_machined_domain::{ServiceLauncher, Supervisor};
    use os_runtime_cri_domain::{
        ImageCacheRuntimePlan, REGISTRYD_LISTEN_ADDRESS, RegistrydAction, RegistrydRuntimeService,
        RegistrydServiceExecutionStatus, RegistrydState,
    };

    use crate::{
        PRE_CONFIG_RESOLV_CONF_PATH, Pid1RegistrydRuntimeServiceLauncher, SVC_NAME, SVC_PATH,
        boot_cosi_bridge_no_seed_line, boot_cosi_bridge_seeded_line, boot_cosi_bridge_stable_line,
        boot_cosi_bridge_start_line, dhcp6_iaid_from_mac, dhcp6_transaction_id,
        hydrate_bootstrap_image_cache_mount_state, hydrate_declared_image_cache_block_state,
        image_cache_copy_adapter_for_boot_privilege, image_cache_copy_execution_status_label,
        image_cache_copy_state_line, ms_flags_from_str, net_errno, pid1_registryd_launch_result,
        pre_config_bootstrap_line, pre_config_dhcp4_start_line, pre_config_dhcp6_start_line,
        pre_config_resolv_conf_body, project_image_cache_runtime_plan_from_cosi_state,
        ready_ipv6_link_local_from_proc, reaped_line,
        run_image_cache_runtime_adapters_with_supervisor_and_copy_adapter, seq_task_line,
        service_started_line, sysctl_path, write_pre_config_resolv_conf_at,
    };

    /// The kernel-network port binding for this build.
    ///
    /// PID 1 reaches the kernel's network stack only through
    /// [`os_kernel_abi::KernelNet`]; `LinuxNet` is one adapter for it. Swapping
    /// the kernel substrate for another that implements the same operations is
    /// a change to this one function, not to the call sites below.
    fn kernel_net() -> impl KernelNet {
        os_network_domain::linux_net::LinuxNet::new()
    }

    /// Entry point for the real PID 1. Never returns: on any path it powers the
    /// machine off (and if `reboot(2)` somehow returns, the trailing loop keeps
    /// us from unwinding out of `main` and panicking the kernel).
    pub fn main() -> ! {
        setup_console();
        match run() {
            Ok(()) => {}
            Err(e) => {
                eprintln!("talos-rust init: fatal error during boot: {e}");
            }
        }
        shutdown();
        loop {
            // Unreachable on a real kernel: shutdown() does not return.
            std::thread::sleep(Duration::from_secs(3600));
        }
    }

    /// Drive the full machined boot sequence, then reap the service and report.
    fn run() -> std::io::Result<()> {
        println!("=== talos-rust init (PID1) booting ===");

        // Detect the runtime mode ONCE at boot (metal vs sandbox/container) and
        // log it for operator clarity. This is purely informational: the
        // best-effort tolerance of the privileged bring-up tasks is driven by
        // errno classification (privileged_op_skip_reason), not this flag, so a
        // mis-detection cannot change behavior. /proc/version is mounted by the
        // MountPseudoFs phase, but on a sandbox the Sentry provides it from the
        // start; read it best-effort here (None => default to metal).
        let mode = detect_runtime_mode(fs::read_to_string("/proc/version").ok().as_deref());
        println!("[seq] runtime mode: {}", mode.as_str());

        // Build the real libc-backed runtime and the boot sequencer. We replace
        // the standard kubelet/containerd service set with our single bundled
        // `svc` binary, since the initramfs has no real services to exec.
        let mut rt = LibcRuntime::new("metal", mode);
        let services = vec![BootService::new(SVC_NAME, [SVC_PATH], RestartPolicy::Never)];
        let seq = BootSequencer::with_services(services);

        let mut logger = SeqLogger;
        // run_boot drives every phase in order against our runtime; each task
        // is reported through `logger` (which prints the `[seq]` lines), and
        // our runtime performs the real syscalls.
        let report = match seq.run_boot(&mut rt, &mut logger) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[seq] boot FAILED: {e}");
                return Err(std::io::Error::other(e.to_string()));
            }
        };
        println!(
            "[seq] boot complete: {} phase(s), {} task(s) done",
            report.phase_count(),
            report.total_done()
        );

        // The sequence has reached Running. The StartServices phase spawned our
        // service via the runtime; reap it now (blocking) so we observe its
        // clean exit before powering off.
        rt.reap_spawned();

        print_proc_proof();

        println!("=== talos-rust init: OK ===");
        std::io::stdout().flush().ok();
        std::io::stderr().flush().ok();

        // Give the console a moment to drain before we pull the plug.
        std::thread::sleep(Duration::from_millis(500));
        Ok(())
    }

    /// Open `/dev/console` and dup it onto fds 0/1/2 so `println!`/`eprintln!`
    /// reach the serial console (and so spawned children inherit it).
    fn setup_console() {
        // SAFETY: plain libc FFI; we check the return value before using the fd.
        unsafe {
            let path = CString::new("/dev/console").unwrap();
            let fd = libc::open(path.as_ptr(), libc::O_RDWR);
            if fd < 0 {
                return;
            }
            libc::dup2(fd, libc::STDIN_FILENO);
            libc::dup2(fd, libc::STDOUT_FILENO);
            libc::dup2(fd, libc::STDERR_FILENO);
            if fd > libc::STDERR_FILENO {
                libc::close(fd);
            }
        }
    }

    /// Print proof that we are on a real, booted Linux kernel by reading
    /// `/proc` (which only exists if the MountPseudoFs phase succeeded).
    fn print_proc_proof() {
        match fs::read_to_string("/proc/version") {
            Ok(version) => {
                let first = version.lines().next().unwrap_or("").trim();
                println!("/proc/version: {first}");
            }
            Err(e) => println!("/proc/version: unavailable ({e})"),
        }
        match fs::read_to_string("/proc/sys/kernel/hostname") {
            Ok(h) => println!("/proc/sys/kernel/hostname: {}", h.trim()),
            Err(e) => println!("/proc/sys/kernel/hostname: unavailable ({e})"),
        }
    }

    /// Sync filesystems and power off the machine via `reboot(2)`. Does not
    /// return under a real kernel; the caller's loop guards us if it does.
    fn shutdown() {
        // SAFETY: plain libc FFI with no arguments.
        unsafe {
            libc::sync();
        }
        std::io::stdout().flush().ok();
        // SAFETY: RB_POWER_OFF is a documented reboot(2) command constant.
        unsafe {
            libc::reboot(libc::RB_POWER_OFF);
        }
    }

    // -----------------------------------------------------------------------
    // The real, libc-backed Runtime implementation.
    // -----------------------------------------------------------------------

    /// A `libc`-backed [`Runtime`] (a.k.a. `PlatformOps`) that performs the
    /// actual syscalls each boot task requests: `mkdir`, sysctl writes,
    /// `mount(2)`, `sethostname(2)`, link-up (best effort), and
    /// `fork(2)`+`execve(2)` service spawns.
    ///
    /// Spawned child pids are recorded in `spawned` so init can reap them
    /// (via `waitpid(2)`) after the sequence reaches `Running`.
    struct LibcRuntime {
        platform: String,
        /// The runtime mode (metal vs sandbox/container), detected once at boot.
        /// Informational only: tolerance is errno-driven. Carried so skip logs
        /// can note the mode and so the field documents the detected environment.
        mode: BootRuntimeMode,
        /// Hostname declared by the loaded machine config, cached so the
        /// SetHostname phase need not re-read disk.
        config_hostname: Option<String>,
        /// Static IPv4 address declared on the first configured interface,
        /// cached during `apply_config` for the Network phase to assign to
        /// `eth0` via `add_ipv4`.
        config_address: Option<StaticAddress>,
        /// Cached raw machine config: the sequencer asks for it in both the
        /// LoadConfig and SetHostname phases, but the platform source should be
        /// resolved exactly once per boot.
        raw_config: Option<String>,
        /// Host-safe image-cache plan projected after the boot-owned COSI bridge.
        image_cache_runtime_plan: ImageCacheRuntimePlan,
        /// Boot-owned COSI state used to materialize post-runtime image-cache
        /// config updates without handing service effects to reconciliation.
        image_cache_cosi_state: State,
        /// Boot-owned machined supervisor state for registryd runtime effects.
        image_cache_supervisor: Supervisor,
        /// (name, pid) of every successfully spawned service, in order.
        spawned: Vec<(String, u32)>,
    }

    impl LibcRuntime {
        fn new(platform: impl Into<String>, mode: BootRuntimeMode) -> Self {
            LibcRuntime {
                platform: platform.into(),
                mode,
                config_hostname: None,
                config_address: None,
                raw_config: None,
                image_cache_runtime_plan: ImageCacheRuntimePlan::default(),
                image_cache_cosi_state: State::new(),
                image_cache_supervisor: Supervisor::new(MachineType::Worker),
                spawned: Vec::new(),
            }
        }

        /// Assign the configured static IPv4 address to `eth0` and verify it via
        /// the kernel (`query_addrs`) and `/sys` (`get_operstate`). All steps are
        /// best-effort: a missing NIC or an already-assigned address must not
        /// abort the boot, so failures are logged as `[net]` lines and tolerated.
        fn configure_eth0_address(&mut self) {
            let Some(addr) = self.config_address.clone() else {
                println!("[net] eth0: no static address in machine config; skipping add_ipv4");
                return;
            };
            println!(
                "[net] eth0: assigning {}/{} (add_ipv4)",
                addr.addr, addr.prefix
            );
            match kernel_net().add_ipv4_address("eth0", &addr.addr, addr.prefix) {
                Ok(()) => println!("[net] eth0: add_ipv4 ok"),
                Err(e) => {
                    // add_ipv4 (RTM_NEWADDR) is BEST EFFORT under a sandbox: the
                    // host already owns eth0's addressing and forbids changing
                    // it. Classify the netlink errno: the sandbox set
                    // (EPERM/EACCES/ENOSYS/EOPNOTSUPP) is logged-and-skipped in
                    // the Talos `[seq] <task>: skipped (<errno>, sandbox)` form;
                    // EEXIST (re-run) and any other errno are merely noted, since
                    // eth0 addressing never aborts the boot. On a real kernel
                    // add_ipv4 succeeds and this path never runs.
                    let errno = net_errno(&e.to_string());
                    match privileged_op_skip_reason(errno) {
                        Some(reason) => self.skip_log("add_ipv4(eth0)", reason),
                        None => println!("[net] eth0: add_ipv4 error (tolerated): {e}"),
                    }
                }
            }

            // Verification path: read the kernel's view back.
            match kernel_net().ipv4_addresses("eth0") {
                Ok(addrs) => {
                    let joined = if addrs.is_empty() {
                        "<none>".to_string()
                    } else {
                        addrs.join(",")
                    };
                    println!("net: eth0 addr(kernel)={joined}");
                }
                Err(e) => {
                    println!("[net] eth0: query_addrs error: {e}");
                    println!("net: eth0 addr(kernel)=<unknown>");
                }
            }

            // Cross-confirmation from /sys.
            match kernel_net().link_oper_state("eth0") {
                Ok(state) => println!("net: eth0 operstate={state}"),
                Err(e) => {
                    println!("[net] eth0: get_operstate error: {e}");
                    println!("net: eth0 operstate=<unknown>");
                }
            }
        }

        /// Bring up the platform's primary metadata interface before config
        /// acquisition. This mirrors AWS's pre-IMDS bootstrap: bring `eth0` up,
        /// run the bounded DHCPv4 wire-state client when requested, and leave
        /// DHCPv6 to the later operator slice.
        fn bootstrap_config_network(
            &mut self,
            bootstrap: &os_platform_domain::aws::BootstrapNetworkConfig,
        ) {
            let iface = bootstrap.interface;
            println!(
                "{}",
                pre_config_bootstrap_line(
                    iface,
                    bootstrap.dhcp4,
                    bootstrap.dhcp6,
                    bootstrap.route_metric,
                )
            );
            if iface != "lo" {
                wait_for_iface(iface, 50, self.mode);
            }
            println!("[net] {iface}: pre-config link up (set_link_up / rtnetlink RTM_NEWLINK)");
            match kernel_net().set_link_up(iface) {
                Ok(()) => println!("[net] {iface}: pre-config link up"),
                Err(e) => {
                    let errno = net_errno(&e.to_string());
                    match privileged_op_skip_reason(errno) {
                        Some(reason) => {
                            self.skip_log(&format!("pre_config_link_up({iface})"), reason)
                        }
                        None => {
                            println!("[net] {iface}: pre-config link-up error (tolerated): {e}")
                        }
                    }
                }
            }

            if bootstrap.dhcp4 {
                self.bootstrap_config_network_dhcp4(bootstrap);
            }

            if bootstrap.dhcp6 {
                self.bootstrap_config_network_dhcp6(bootstrap);
            }
        }

        /// Run a bounded one-shot DHCPv4 acquisition for pre-config metadata
        /// networking. The pure transaction/packet logic lives in
        /// `talos-network`; PID1 owns only the Linux UDP socket and the immediate
        /// application of the parsed operator output.
        fn bootstrap_config_network_dhcp4(
            &mut self,
            bootstrap: &os_platform_domain::aws::BootstrapNetworkConfig,
        ) {
            let iface = bootstrap.interface;
            println!("{}", pre_config_dhcp4_start_line(iface));
            let mac = match read_iface_mac(iface) {
                Ok(mac) => mac,
                Err(e) => {
                    println!("[net] {iface}: DHCPv4 skipped, unable to read MAC: {e}");
                    return;
                }
            };
            let socket = match open_dhcp4_socket(iface) {
                Ok(socket) => socket,
                Err(e) => {
                    let errno = e.raw_os_error().unwrap_or(-1);
                    match privileged_op_skip_reason(errno) {
                        Some(reason) => self.skip_log(&format!("dhcp4_socket({iface})"), reason),
                        None => println!("[net] {iface}: DHCPv4 socket error: {e}"),
                    }
                    return;
                }
            };

            let xid = dhcp4_xid(mac);
            let config = os_network_domain::Dhcp4ClientConfig::new(xid, mac);
            let mut transaction =
                os_network_domain::Dhcp4WireTransaction::new(config).with_retry_schedule(2, 3);
            let mut outbound = match transaction.start() {
                Ok(outbound) => outbound,
                Err(e) => {
                    println!("[net] {iface}: DHCPv4 start error: {e}");
                    return;
                }
            };
            let mut buf = [0u8; 2048];

            loop {
                if let Err(e) = send_dhcp4_packet(&socket, &outbound) {
                    println!("[net] {iface}: DHCPv4 send error: {e}");
                    return;
                }

                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((n, _peer)) => match transaction.handle_packet(&buf[..n]) {
                            Ok(os_network_domain::Dhcp4WireAction::Send(next)) => {
                                outbound = next;
                                break;
                            }
                            Ok(os_network_domain::Dhcp4WireAction::Bound(lease)) => {
                                println!(
                                    "[net] {iface}: DHCPv4 bound {}/{}",
                                    lease.address, lease.prefix_len
                                );
                                self.apply_pre_config_dhcp4_lease(bootstrap, &lease);
                                return;
                            }
                            Ok(os_network_domain::Dhcp4WireAction::Ignored(reason)) => {
                                println!("[net] {iface}: DHCPv4 ignored packet ({reason})");
                            }
                            Err(e) => {
                                println!("[net] {iface}: DHCPv4 packet error: {e}");
                            }
                        },
                        Err(e)
                            if matches!(
                                e.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            ) =>
                        {
                            match transaction.handle_timeout() {
                                Ok(next) => {
                                    outbound = next;
                                    break;
                                }
                                Err(os_kernel::Error::Timeout) => {
                                    println!(
                                        "[net] {iface}: DHCPv4 timed out after {} attempt(s)",
                                        transaction.attempts()
                                    );
                                    return;
                                }
                                Err(e) => {
                                    println!("[net] {iface}: DHCPv4 retry error: {e}");
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            println!("[net] {iface}: DHCPv4 receive error: {e}");
                            return;
                        }
                    }
                }
            }
        }

        fn apply_pre_config_dhcp4_lease(
            &self,
            bootstrap: &os_platform_domain::aws::BootstrapNetworkConfig,
            lease: &os_network_domain::Dhcp4Lease,
        ) {
            let iface = bootstrap.interface;
            let mut op = os_network_domain::OperatorSpec::dhcp4(iface);
            op.route_metric = bootstrap.route_metric;
            let output = match op.apply_dhcp4_lease(lease, false) {
                Ok(output) => output,
                Err(e) => {
                    println!("[net] {iface}: DHCPv4 lease conversion error: {e}");
                    return;
                }
            };

            let os_network_domain::OperatorOutput {
                addresses,
                routes,
                resolver,
                ..
            } = output;

            for address in addresses {
                println!(
                    "[net] {iface}: DHCPv4 assigning {}/{}",
                    address.address, address.prefix_len
                );
                match kernel_net().add_ipv4_address(
                    iface,
                    &address.address.to_string(),
                    address.prefix_len,
                ) {
                    Ok(()) => println!("[net] {iface}: DHCPv4 address applied"),
                    Err(e) => {
                        let errno = net_errno(&e.to_string());
                        if errno == libc::EEXIST {
                            println!("[net] {iface}: DHCPv4 address already exists");
                        } else {
                            println!("[net] {iface}: DHCPv4 address apply error: {e}");
                        }
                    }
                }
            }

            for route in routes {
                let destination = route
                    .destination
                    .map(|addr| os_network_domain::linux_net::parse_ipv4(&addr.to_string()))
                    .transpose();
                let gateway = route
                    .gateway
                    .map(|addr| os_network_domain::linux_net::parse_ipv4(&addr.to_string()))
                    .transpose();
                let (destination, gateway) = match (destination, gateway) {
                    (Ok(destination), Ok(gateway)) => (destination, gateway),
                    (Err(e), _) | (_, Err(e)) => {
                        println!("[net] {iface}: DHCPv4 route parse error: {e}");
                        continue;
                    }
                };
                println!(
                    "[net] {iface}: DHCPv4 route dest={}/{} gateway={:?} metric={}",
                    route
                        .destination
                        .map(|addr| addr.to_string())
                        .unwrap_or_else(|| "default".to_string()),
                    route.prefix_len,
                    route.gateway.map(|addr| addr.to_string()),
                    route.metric
                );
                match kernel_net().add_ipv4_route(
                    iface,
                    destination,
                    route.prefix_len,
                    gateway,
                    route.metric,
                    route.protocol.into(),
                ) {
                    Ok(()) => println!("[net] {iface}: DHCPv4 route applied"),
                    Err(e) => {
                        let errno = net_errno(&e.to_string());
                        if errno == libc::EEXIST {
                            println!("[net] {iface}: DHCPv4 route already exists");
                        } else {
                            println!("[net] {iface}: DHCPv4 route apply error: {e}");
                        }
                    }
                }
            }

            if let Some(resolver) = resolver {
                self.write_pre_config_resolver(iface, "DHCPv4", &resolver);
            }
        }

        /// Run a bounded one-shot DHCPv6 rapid-solicit acquisition for
        /// pre-config metadata networking. This mirrors Talos'
        /// `operator/dhcp6.go` / `nclient6.RapidSolicit`: wait for a ready
        /// link-local address, send SOLICIT to ff02::1:2%iface:547 from port
        /// 546, accept either a rapid REPLY or ADVERTISE->REQUEST, then apply
        /// the parsed operator output immediately.
        fn bootstrap_config_network_dhcp6(
            &mut self,
            bootstrap: &os_platform_domain::aws::BootstrapNetworkConfig,
        ) {
            let iface = bootstrap.interface;
            println!("{}", pre_config_dhcp6_start_line(iface));

            let mac = match read_iface_mac(iface) {
                Ok(mac) => mac,
                Err(e) => {
                    println!("[net] {iface}: DHCPv6 skipped, unable to read MAC: {e}");
                    return;
                }
            };

            let Some((link_local, dad_failed)) = wait_for_ipv6_link_ready(iface, 300, self.mode)
            else {
                println!("[net] {iface}: DHCPv6 skipped, no ready IPv6 link-local address");
                return;
            };
            if dad_failed {
                println!("[net] {iface}: DHCPv6 continuing despite DADFAILED on {link_local}");
            }

            let ifindex = match iface_index(iface) {
                Ok(ifindex) => ifindex,
                Err(e) => {
                    println!("[net] {iface}: DHCPv6 skipped, unable to resolve ifindex: {e}");
                    return;
                }
            };
            let socket = match open_dhcp6_socket(iface, ifindex, link_local) {
                Ok(socket) => socket,
                Err(e) => {
                    let errno = e.raw_os_error().unwrap_or(-1);
                    match privileged_op_skip_reason(errno) {
                        Some(reason) => self.skip_log(&format!("dhcp6_socket({iface})"), reason),
                        None => println!("[net] {iface}: DHCPv6 socket error: {e}"),
                    }
                    return;
                }
            };

            let salt = time_mix();
            let config = os_network_domain::Dhcp6ClientConfig::new(
                dhcp6_transaction_id(mac, salt),
                dhcp6_iaid_from_mac(mac),
                os_network_domain::Dhcp6ClientIdentifier::DuidLlt {
                    mac,
                    seconds_since_2000: dhcp6_duid_llt_time(),
                },
            )
            .with_request_transaction_id(dhcp6_transaction_id(mac, salt ^ 0x0052_4551));
            let mut transaction = os_network_domain::Dhcp6WireTransaction::new(config);
            let mut outbound = match transaction.start() {
                Ok(outbound) => outbound,
                Err(e) => {
                    println!("[net] {iface}: DHCPv6 start error: {e}");
                    return;
                }
            };
            let mut timeout = Duration::from_secs(5);
            let mut attempts = 1u8;
            let mut buf = [0u8; 2048];

            loop {
                if let Err(e) = send_dhcp6_packet(&socket, ifindex, &outbound) {
                    println!("[net] {iface}: DHCPv6 send error: {e}");
                    return;
                }
                if let Err(e) = socket.set_read_timeout(Some(timeout)) {
                    println!("[net] {iface}: DHCPv6 timeout setup error: {e}");
                    return;
                }

                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((n, _peer)) => match transaction.handle_packet(&buf[..n]) {
                            Ok(os_network_domain::Dhcp6WireAction::Send(next)) => {
                                outbound = next;
                                attempts = 1;
                                timeout = Duration::from_secs(5);
                                break;
                            }
                            Ok(os_network_domain::Dhcp6WireAction::Bound(lease)) => {
                                if let Some(address) = &lease.address {
                                    println!("[net] {iface}: DHCPv6 bound {}/128", address.address);
                                } else {
                                    println!("[net] {iface}: DHCPv6 bound without IA_NA address");
                                }
                                self.apply_pre_config_dhcp6_lease(bootstrap, &lease);
                                return;
                            }
                            Ok(os_network_domain::Dhcp6WireAction::Ignored(reason)) => {
                                println!("[net] {iface}: DHCPv6 ignored packet ({reason})");
                            }
                            Err(e) => {
                                println!("[net] {iface}: DHCPv6 packet error: {e}");
                            }
                        },
                        Err(e)
                            if matches!(
                                e.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            ) =>
                        {
                            if attempts >= 3 {
                                println!(
                                    "[net] {iface}: DHCPv6 timed out after {attempts} attempt(s)"
                                );
                                return;
                            }
                            attempts += 1;
                            timeout =
                                Duration::from_secs(timeout.as_secs().saturating_mul(2).max(1));
                            match transaction.retry(0) {
                                Ok(next) => {
                                    outbound = next;
                                    break;
                                }
                                Err(e) => {
                                    println!("[net] {iface}: DHCPv6 retry error: {e}");
                                    return;
                                }
                            }
                        }
                        Err(e) => {
                            println!("[net] {iface}: DHCPv6 receive error: {e}");
                            return;
                        }
                    }
                }
            }
        }

        fn apply_pre_config_dhcp6_lease(
            &self,
            bootstrap: &os_platform_domain::aws::BootstrapNetworkConfig,
            lease: &os_network_domain::Dhcp6Lease,
        ) {
            let iface = bootstrap.interface;
            let output =
                match os_network_domain::OperatorSpec::dhcp6(iface).apply_dhcp6_lease(lease, false) {
                    Ok(output) => output,
                    Err(e) => {
                        println!("[net] {iface}: DHCPv6 lease conversion error: {e}");
                        return;
                    }
                };

            let os_network_domain::OperatorOutput {
                addresses,
                resolver,
                time_servers,
                ..
            } = output;

            for address in addresses {
                match address.address {
                    os_kernel::address::NodeAddress::V6(_) => {
                        println!(
                            "[net] {iface}: DHCPv6 assigning {}/{}",
                            address.address, address.prefix_len
                        );
                        match kernel_net().add_ipv6_address(
                            iface,
                            &address.address.to_string(),
                            address.prefix_len,
                        ) {
                            Ok(()) => println!("[net] {iface}: DHCPv6 address applied"),
                            Err(e) => {
                                let errno = net_errno(&e.to_string());
                                if errno == libc::EEXIST {
                                    println!("[net] {iface}: DHCPv6 address already exists");
                                } else {
                                    println!("[net] {iface}: DHCPv6 address apply error: {e}");
                                }
                            }
                        }
                    }
                    os_kernel::address::NodeAddress::V4(_) => {
                        println!("[net] {iface}: DHCPv6 produced IPv4 address; ignored")
                    }
                }
            }

            if let Some(resolver) = resolver {
                self.write_pre_config_resolver(iface, "DHCPv6", &resolver);
            }

            if !time_servers.is_empty() {
                println!(
                    "[net] {iface}: DHCPv6 time servers [{}]",
                    time_servers.join(", ")
                );
            }
        }

        fn write_pre_config_resolver(
            &self,
            iface: &str,
            family: &str,
            resolver: &os_network_domain::ResolverSpec,
        ) {
            let body = pre_config_resolv_conf_body(resolver);
            if body.is_empty() {
                return;
            }

            match write_pre_config_resolv_conf_at(PRE_CONFIG_RESOLV_CONF_PATH, &body) {
                Ok(()) => println!(
                    "[net] {iface}: {family} resolver applied path={PRE_CONFIG_RESOLV_CONF_PATH}"
                ),
                Err(e) => println!("[net] {iface}: {family} resolver apply error: {e}"),
            }
        }

        /// Log a best-effort skip of a privileged bring-up op in the
        /// Talos-style `[seq] <task>: skipped (<errno>, sandbox)` form. Called
        /// when [`privileged_op_skip_reason`] classifies an errno as a benign
        /// sandbox case so the boot continues. The detected [`Self::mode`] is
        /// noted for context (it never gates the decision — the errno does).
        fn skip_log(&self, task: &str, reason: &str) {
            println!(
                "[seq] {task}: skipped ({reason}, sandbox; mode={})",
                self.mode.as_str()
            );
        }

        /// Blockingly reap every spawned service via `waitpid(2)`, printing the
        /// reaper markers. Called once the boot has reached `Running`.
        fn reap_spawned(&mut self) {
            // Take ownership so the borrow checker is happy and we don't reap
            // twice.
            let spawned = std::mem::take(&mut self.spawned);
            for (name, pid) in spawned {
                let mut status: libc::c_int = 0;
                // SAFETY: status is a valid writable int; we wait on a specific
                // child pid we spawned. 0 flags => block until it exits.
                let rc = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, 0) };
                if rc < 0 {
                    println!(
                        "reaper: waitpid({pid}) failed: {}",
                        std::io::Error::last_os_error()
                    );
                    continue;
                }
                let code = exit_code(status);
                println!("{}", reaped_line(&name, pid, code));
            }
        }
    }

    /// Decode a `waitpid` status into a conventional exit code (signal-killed
    /// children report `128 + signo`, matching shell convention).
    fn exit_code(status: libc::c_int) -> i32 {
        if libc::WIFEXITED(status) {
            libc::WEXITSTATUS(status)
        } else if libc::WIFSIGNALED(status) {
            128 + libc::WTERMSIG(status)
        } else {
            -1
        }
    }

    impl Runtime for LibcRuntime {
        fn platform(&self) -> &str {
            &self.platform
        }

        fn make_directory(&mut self, path: &str) -> MResult<()> {
            fs::create_dir_all(path)
                .map_err(|e| MachinedError::task_failed("make_directory", format!("{path}: {e}")))
        }

        fn write_sysctl(&mut self, key: &str, value: &str) -> MResult<SysctlOutcome> {
            let path = sysctl_path(key);
            // KSPP sysctl hardening is BEST EFFORT (matching upstream Talos).
            // On a privileged kernel the write succeeds and hardening is
            // applied. Under a sandbox (gVisor/runsc makes some knobs
            // read-only), an unprivileged container, a read-only /proc, or a
            // kernel lacking the knob, the write fails with one of a small set
            // of expected errnos: those are classified as a skip (logged,
            // boot continues). Any other errno is genuinely unexpected and
            // aborts the boot.
            match fs::write(&path, value) {
                Ok(()) => Ok(SysctlOutcome::Applied),
                Err(e) => {
                    let errno = e.raw_os_error().unwrap_or(-1);
                    match sysctl_skip_reason(errno) {
                        Some(reason) => Ok(SysctlOutcome::Skipped(reason)),
                        None => Err(MachinedError::task_failed(
                            "write_sysctl",
                            format!("{path}: {e}"),
                        )),
                    }
                }
            }
        }

        fn mount(&mut self, req: &MountRequest) -> MResult<()> {
            // Ensure the mount point exists first.
            let _ = fs::create_dir_all(&req.target);
            // Pseudo-fs mounts are BEST EFFORT under sandboxes (mirroring the
            // best-effort sysctl path). On a real privileged kernel the mount
            // succeeds. Under gVisor/runsc the Sentry already provides /proc and
            // /sys and forbids mounting over them, a kernel-supplied devtmpfs is
            // already on /dev, and an unprivileged container cannot mount at all:
            // those come back as a small set of expected errnos
            // (EPERM/EACCES/EBUSY/ENODEV) that we log and skip rather than abort.
            // Any other errno is genuinely unexpected and fails the boot.
            match do_mount(req) {
                Ok(()) => Ok(()),
                Err(e) => {
                    let errno = e.raw_os_error().unwrap_or(-1);
                    match mount_skip_reason(errno) {
                        Some(reason) => {
                            println!(
                                "[seq] mount {}: skipped ({reason}, already provided)",
                                req.target
                            );
                            Ok(())
                        }
                        None => Err(MachinedError::task_failed(
                            "mount",
                            format!("{}: {e}", req.target),
                        )),
                    }
                }
            }
        }

        fn load_config(&mut self) -> MResult<String> {
            if let Some(raw) = &self.raw_config {
                return Ok(raw.clone());
            }

            let cmdline_raw = fs::read_to_string(PROC_CMDLINE_PATH).unwrap_or_default();
            let cmdline = os_init_app::cmdline::CmdLine::parse(&cmdline_raw);
            let machine_config_contents = fs::read_to_string(MACHINE_CONFIG_PATH).ok();
            let store = FileConfigStore::new("/");

            let resolved = resolve_config(&cmdline, &store, machine_config_contents.as_deref())
                .map_err(|e| MachinedError::task_failed("load_config", e.to_string()))?;

            println!("[seq] config source: {}", resolved.origin);
            self.platform = resolved.platform.clone();
            self.raw_config = Some(resolved.contents.clone());
            Ok(resolved.contents)
        }

        fn bootstrap_network_for_config(&mut self) -> MResult<bool> {
            let cmdline_raw = fs::read_to_string(PROC_CMDLINE_PATH).unwrap_or_default();
            let cmdline = os_init_app::cmdline::CmdLine::parse(&cmdline_raw);
            let Some(bootstrap) = pre_config_network_bootstrap(&cmdline).map_err(|e| {
                MachinedError::task_failed("bootstrap_network_for_config", e.to_string())
            })?
            else {
                return Ok(false);
            };

            self.bootstrap_config_network(&bootstrap);
            Ok(true)
        }

        fn apply_config(&mut self, raw: &str) -> MResult<Option<String>> {
            // Parse the early config for the hostname (and cache it). The full
            // config is machined's domain; init only needs the hostname here.
            let ec = try_early_config(raw)
                .map_err(|e| MachinedError::task_failed("apply_config", e.to_string()))?;
            self.config_hostname = ec.hostname.clone();
            self.config_address = ec.first_iface_address.clone();
            for op in &ec.dhcp_operators {
                println!(
                    "[net] {}: machine-config {} operator metric={}",
                    op.link_name,
                    op.kind.as_str(),
                    op.route_metric
                );
            }
            Ok(ec.hostname)
        }

        fn set_hostname(&mut self, hostname: &str) -> MResult<()> {
            let bytes = hostname.as_bytes();
            // SAFETY: pointer/len describe a valid byte slice for the duration
            // of the call; sethostname does not retain it.
            let rc = unsafe {
                libc::sethostname(bytes.as_ptr() as *const libc::c_char, bytes.len() as _)
            };
            if rc == 0 {
                if let Some(read_back) = get_hostname() {
                    println!("hostname set to {hostname} (read back: {read_back})");
                } else {
                    println!("hostname set to {hostname}");
                }
                return Ok(());
            }
            // sethostname is BEST EFFORT under a sandbox: gVisor's Sentry / an
            // unprivileged container owns the UTS namespace and denies (or never
            // implements) the call. Classify the errno: the sandbox set
            // (EPERM/EACCES/ENOSYS/EOPNOTSUPP) is logged-and-skipped so the boot
            // continues; any other errno is genuinely unexpected and fails. On a
            // real privileged kernel rc == 0 above and this path never runs.
            let e = std::io::Error::last_os_error();
            let errno = e.raw_os_error().unwrap_or(-1);
            match privileged_op_skip_reason(errno) {
                Some(reason) => {
                    self.skip_log("set_hostname", reason);
                    Ok(())
                }
                None => Err(MachinedError::task_failed(
                    "set_hostname",
                    format!("sethostname({hostname}): {e}"),
                )),
            }
        }

        fn link_up(&mut self, iface: &str) -> MResult<()> {
            // Bring the link up via the REAL rtnetlink path (RTM_NEWLINK/IFF_UP)
            // exposed by talos-network's linux_net module, replacing the legacy
            // in-memory fake. Loopback in particular must come up; failures on
            // other links are non-fatal (best effort), but we surface a real
            // error for `lo` so the boot is honest.
            //
            // Physical NICs (virtio-net on QEMU `-M virt`) are probed
            // *asynchronously* by the kernel after the PCI/MMIO bus scan, so at
            // this very early point in boot `eth0` may not have been created
            // yet. Poll `/sys/class/net` for it (bounded) before giving up, and
            // log the interfaces the kernel currently knows about so the serial
            // log is diagnostic.
            if iface != "lo" {
                wait_for_iface(iface, 50, self.mode);
            }
            println!("[net] {iface}: bringing up (set_link_up / rtnetlink RTM_NEWLINK)");
            match kernel_net().set_link_up(iface) {
                Ok(()) => {
                    println!("[net] {iface}: link up");
                    if iface == "lo" {
                        println!("net: lo up");
                    }
                    if iface == "eth0" {
                        // The desired static address (from machine config) is
                        // assigned and verified here, once eth0 is administratively up.
                        self.configure_eth0_address();
                    }
                    Ok(())
                }
                Err(e) => {
                    if iface == "lo" {
                        // Use the legacy ioctl compatibility path before giving up on
                        // loopback, which must come up for a healthy boot.
                        println!(
                            "[net] lo: set_link_up failed ({e}); trying ioctl compatibility path"
                        );
                        match bring_link_up(iface) {
                            Ok(()) => {
                                println!("[net] lo: link up (ioctl compatibility path)");
                                println!("net: lo up");
                                Ok(())
                            }
                            Err(e2) => {
                                // Link-up is BEST EFFORT under a sandbox: gVisor /
                                // an unprivileged container already provides a
                                // configured `lo` and forbids re-configuring it.
                                // Classify the ioctl errno: the sandbox set
                                // (EPERM/EACCES/ENOSYS/EOPNOTSUPP) is
                                // logged-and-skipped so the boot continues; any
                                // other errno genuinely fails. On a privileged
                                // kernel set_link_up (or the ioctl) succeeds and
                                // this path never runs, so metal stays full.
                                let errno = e2.raw_os_error().unwrap_or(-1);
                                match privileged_op_skip_reason(errno) {
                                    Some(reason) => {
                                        self.skip_log("link_up(lo)", reason);
                                        Ok(())
                                    }
                                    None => Err(MachinedError::task_failed(
                                        "link_up",
                                        format!("{iface}: {e2}"),
                                    )),
                                }
                            }
                        }
                    } else {
                        // e.g. eth0 may be absent in a minimal QEMU; log + skip,
                        // but still attempt address verification so the serial log
                        // is informative.
                        println!("[net] {iface}: not brought up ({e})");
                        if iface == "eth0" {
                            self.configure_eth0_address();
                        }
                        Ok(())
                    }
                }
            }
        }

        fn boot_cosi_network_bridge(&mut self) -> MResult<()> {
            self.log(boot_cosi_bridge_start_line());

            let mut controllers = RegistryBuilder::in_memory()
                .domain(Domain::NetworkLinkStatusSource)
                .domain(Domain::NetworkLinkConfig)
                .domain(Domain::NetworkOperatorConfig)
                .domain(Domain::NetworkResolverConfig)
                .domain(Domain::NetworkOperatorMerge)
                .domain(Domain::NetworkOperatorBridge)
                .domain(Domain::NetworkLinkMerge)
                .domain(Domain::NetworkAddressMerge)
                .domain(Domain::NetworkRouteMerge)
                .domain(Domain::NetworkHostnameMerge)
                .domain(Domain::NetworkResolverMerge)
                .build();

            if let Some(raw) = self.raw_config.clone() {
                controllers
                    .state_mut()
                    .create(Box::new(MachineConfigDocument::new(raw)))
                    .map_err(|e| {
                        MachinedError::task_failed("boot_cosi_network_bridge", e.to_string())
                    })?;
                self.log(boot_cosi_bridge_seeded_line());
            } else {
                self.log(boot_cosi_bridge_no_seed_line());
            }

            let ticks = controllers.run_until_stable(5).map_err(|e| {
                MachinedError::task_failed("boot_cosi_network_bridge", e.to_string())
            })?;
            self.log(&boot_cosi_bridge_stable_line(ticks));

            let eth0_key = "network/LinkStatuses.net.talos.dev/eth0";
            if let Some(resource) = controllers.state().get(eth0_key) {
                println!(
                    "[net] link-status: published eth0 owner={}",
                    resource.metadata().owner()
                );
                println!("{eth0_key}");
            } else {
                let published = controllers
                    .state()
                    .list(&LinkStatusResource::kind(), None)
                    .len();
                println!(
                    "[net] link-status: eth0 not published in boot snapshot (published={published})"
                );
            }

            if hydrate_declared_image_cache_block_state(
                self.raw_config.as_deref(),
                controllers.state_mut(),
            )
            .map_err(|e| MachinedError::task_failed("boot_cosi_network_bridge", e.to_string()))?
            {
                self.log("image-cache-block-state: declared IMAGECACHE VolumeStatus phase=waiting");
            }

            let image_cache_bootstrap_resources = hydrate_bootstrap_image_cache_mount_state(
                self.raw_config.as_deref(),
                controllers.state_mut(),
            )
            .map_err(|e| MachinedError::task_failed("boot_cosi_network_bridge", e.to_string()))?;
            if image_cache_bootstrap_resources > 0 {
                self.log(&format!(
                    "image-cache-block-state: observed bootstrap roots resources={image_cache_bootstrap_resources}"
                ));
            }

            self.image_cache_runtime_plan = project_image_cache_runtime_plan_from_cosi_state(
                self.raw_config.as_deref(),
                RegistrydState::default(),
                controllers.state(),
            )
            .map_err(|e| MachinedError::task_failed("boot_cosi_network_bridge", e.to_string()))?;
            let roots = self.image_cache_runtime_plan.config.roots.join(",");
            self.log(&format!(
                "image-cache-runtime-plan: status={} copyStatus={} roots=[{roots}] registrydAction={}",
                self.image_cache_runtime_plan.config.status.as_str(),
                self.image_cache_runtime_plan.config.copy_status.as_str(),
                registryd_action_label(self.image_cache_runtime_plan.registryd_action),
            ));
            self.image_cache_cosi_state = std::mem::take(controllers.state_mut());

            Ok(())
        }

        fn run_image_cache_runtime_adapters(&mut self) -> MResult<()> {
            // Source-shaped readiness rationale: Talos first crosses the
            // explicit privileged copy boundary, then starts `registryd` as an
            // in-process machined goroutine service and reports readiness
            // through `/healthz`. This PID1 bridge binds the source loopback
            // address through the host-tested runtime-service launcher and
            // reports healthy only after the payload-backed endpoint answers.
            let plan = self.image_cache_runtime_plan.clone();
            let copy_adapter =
                image_cache_copy_adapter_for_boot_privilege(!self.mode.is_container());
            let mut launcher = Pid1RegistrydLauncher::default();
            let outcome = run_image_cache_runtime_adapters_with_supervisor_and_copy_adapter(
                &plan,
                &mut self.image_cache_cosi_state,
                &mut self.image_cache_supervisor,
                &mut launcher,
                copy_adapter,
            )?;
            let copy_report = outcome.copy_report;
            let copy_status = image_cache_copy_execution_status_label(copy_report.status);
            self.log(&format!(
                "image-cache-runtime: copy status={copy_status} source={} target={} filesCopied={} filesSkipped={} dirsCreated={} bytesCopied={}",
                copy_report.source,
                copy_report.target,
                copy_report.files_copied,
                copy_report.files_skipped,
                copy_report.directories_created,
                copy_report.bytes_copied,
            ));
            self.log(&image_cache_copy_state_line(outcome.copy_done));

            let report = outcome.report;
            let registryd_state = outcome.registryd_state;
            let status = registryd_execution_status_label(report.status);
            self.image_cache_runtime_plan = outcome.observed_plan.clone();
            self.log(&format!(
                "image-cache-runtime: registryd status={status} loaded={} started={} running={} healthy={} observedStatus={} observedAction={}",
                report.loaded,
                report.started,
                registryd_state.running,
                registryd_state.healthy,
                outcome.observed_plan.config.status.as_str(),
                registryd_action_label(outcome.observed_plan.registryd_action),
            ));
            Ok(())
        }

        fn spawn_service(&mut self, service: &BootService) -> MResult<u32> {
            let argv = service.command();
            let pid = spawn_child(argv).map_err(|e| {
                MachinedError::service_error(service.name(), format!("spawn failed: {e}"))
            })?;
            self.spawned.push((service.name().to_string(), pid));
            println!("{}", service_started_line(service.name(), pid));
            Ok(pid)
        }

        fn log(&mut self, line: &str) {
            println!("[seq] {line}");
        }
    }

    #[derive(Default)]
    struct Pid1RegistrydLauncher {
        registryd_launcher: Option<Pid1RegistrydRuntimeServiceLauncher>,
    }

    impl ServiceLauncher for Pid1RegistrydLauncher {
        fn launch(&mut self, id: &str) -> MResult<bool> {
            pid1_registryd_launch_result(id)
                .map_err(|err| MachinedError::service_error(id, err.to_string()))
        }

        fn launch_registryd_runtime_service(
            &mut self,
            id: &str,
            service: &RegistrydRuntimeService,
        ) -> MResult<bool> {
            let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
                REGISTRYD_LISTEN_ADDRESS,
                super::PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
            )
            .map_err(|err| MachinedError::service_error(id, err.to_string()))?;
            let healthy = launcher.launch_registryd_runtime_service(id, service)?;

            if healthy {
                self.registryd_launcher = Some(launcher);
            }

            Ok(healthy)
        }
    }

    fn registryd_execution_status_label(status: RegistrydServiceExecutionStatus) -> &'static str {
        match status {
            RegistrydServiceExecutionStatus::NoAction => "no-action",
            RegistrydServiceExecutionStatus::AlreadyRunning => "already-running",
            RegistrydServiceExecutionStatus::Started => "started",
            RegistrydServiceExecutionStatus::LoadedAndStarted => "loaded-and-started",
        }
    }

    fn registryd_action_label(action: RegistrydAction) -> &'static str {
        match action {
            RegistrydAction::None => "none",
            RegistrydAction::Start => "start",
        }
    }

    /// Invoke `mount(2)` for one [`MountRequest`].
    fn do_mount(req: &MountRequest) -> std::io::Result<()> {
        let source = CString::new(req.source.as_str()).unwrap();
        let target = CString::new(req.target.as_str()).unwrap();
        let fstype = CString::new(req.fstype.as_str()).unwrap();
        let flags = ms_flags_from_str(&req.flags) as libc::c_ulong;
        // SAFETY: all pointers are valid C strings living until the call
        // returns; data is NULL (no fs-specific options needed for pseudo-fs).
        let rc = unsafe {
            libc::mount(
                source.as_ptr(),
                target.as_ptr(),
                fstype.as_ptr(),
                flags,
                std::ptr::null(),
            )
        };
        if rc == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    /// Read the current hostname via `gethostname(2)`.
    fn get_hostname() -> Option<String> {
        let mut buf = [0u8; 256];
        // SAFETY: buf is a valid, sufficiently large, writable buffer.
        let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        if rc != 0 {
            return None;
        }
        buf[buf.len() - 1] = 0;
        let cstr = CStr::from_bytes_until_nul(&buf).ok()?;
        Some(cstr.to_string_lossy().into_owned())
    }

    /// Directory in the initramfs where the bundled `.ko` kernel modules live.
    const MODULE_DIR: &str = "/lib/modules";

    /// Kernel modules required to make the virtio NIC (`eth0`) appear, in
    /// dependency order. `virtio_net` pulls in `net_failover`, which pulls in
    /// `failover`; `virtio_net` itself is a module on the Alpine `virt` kernel,
    /// so without loading it no `eth0` netdev is ever created.
    ///
    /// `virtio_mmio` is the virtio *MMIO transport* and must load FIRST. On the
    /// Alpine `virt` kernel `virtio_pci` is built in (so QEMU `-M virt` and
    /// Cloud-Hypervisor, which expose virtio-net over PCI, create `eth0` even
    /// without it), but `CONFIG_VIRTIO_MMIO=m`. Firecracker has NO PCI bus
    /// (`pci=off`) and exposes its devices over virtio-MMIO; without
    /// `virtio_mmio` loaded the kernel never enumerates Firecracker's MMIO net
    /// device, so `virtio_net` has nothing to bind to and `eth0` never appears.
    /// Loading the transport before the driver fixes that and is a harmless
    /// no-op on the PCI VMMs. `finit_module` does no dependency resolution, so
    /// the order here is load-significant.
    const NETWORK_MODULES: &[&str] = &["virtio_mmio", "failover", "net_failover", "virtio_net"];

    /// `finit_module(2)` one `.ko` file by absolute path. Returns the raw errno
    /// on failure so the caller can tolerate `EEXIST` (module already loaded).
    ///
    /// We use `finit_module` (not `init_module`) so the kernel reads the module
    /// image straight from the fd — no need to slurp it into a buffer first.
    fn finit_module(path: &str) -> std::result::Result<(), i32> {
        let cpath = match CString::new(path) {
            Ok(c) => c,
            Err(_) => return Err(libc::EINVAL),
        };
        // SAFETY: cpath is a valid NUL-terminated path string.
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(-1));
        }
        // No module parameters; flags 0.
        let empty = CString::new("").unwrap();
        // SAFETY: fd is an open, readable module image; the params pointer is a
        // valid empty C string; finit_module does not retain either past the call.
        let rc =
            unsafe { libc::syscall(libc::SYS_finit_module, fd, empty.as_ptr(), 0 as libc::c_int) };
        let err = if rc != 0 {
            Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(-1))
        } else {
            Ok(())
        };
        // SAFETY: fd was opened by us and is unused after this point.
        unsafe { libc::close(fd) };
        err
    }

    /// Load every module in [`NETWORK_MODULES`] from [`MODULE_DIR`], in order.
    /// Idempotent: an already-loaded module (`EEXIST`) is treated as success so
    /// reruns are harmless. Missing/failed modules are logged but do not abort
    /// the boot (the network phase will simply find no `eth0`).
    fn load_network_modules(mode: BootRuntimeMode) {
        for name in NETWORK_MODULES {
            let path = format!("{MODULE_DIR}/{name}.ko");
            if !std::path::Path::new(&path).exists() {
                println!("[net] module {name}: not bundled at {path}; skipping");
                continue;
            }
            match finit_module(&path) {
                Ok(()) => println!("[net] module {name}: loaded (finit_module)"),
                // EEXIST (17) / EBUSY (16): already present — fine.
                Err(e) if e == libc::EEXIST || e == libc::EBUSY => {
                    println!("[net] module {name}: already loaded (errno {e})")
                }
                // finit_module is BEST EFFORT under a sandbox: gVisor's Sentry
                // does not implement module loading (ENOSYS) and an unprivileged
                // container is denied (EPERM/EACCES) — the sandbox already
                // provides whatever virtual NIC it offers. Log these in the
                // Talos `[seq] <task>: skipped (<errno>, sandbox)` form and
                // continue. On a real kernel the load succeeds and this never
                // runs, so metal still loads its NIC modules unchanged.
                Err(e) => match privileged_op_skip_reason(e) {
                    Some(reason) => println!(
                        "[seq] finit_module({name}): skipped ({reason}, sandbox; mode={})",
                        mode.as_str()
                    ),
                    None => {
                        println!("[net] module {name}: finit_module failed (errno {e})")
                    }
                },
            }
        }
    }

    /// List the network interfaces the kernel currently knows about by reading
    /// the directory entries of `/sys/class/net`. Best effort: returns an empty
    /// vec if the directory cannot be read.
    fn list_net_ifaces() -> Vec<String> {
        let mut names = Vec::new();
        if let Ok(rd) = fs::read_dir("/sys/class/net") {
            for ent in rd.flatten() {
                if let Some(name) = ent.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        names
    }

    /// Poll `/sys/class/net/<iface>` until the interface appears or `attempts`
    /// (~10ms each) are exhausted. Physical NICs are probed asynchronously, so
    /// the network phase can run before `eth0` has been created; this gives the
    /// kernel a brief window to finish probing. Logs the currently-known
    /// interfaces on entry so the serial log shows what the kernel saw.
    fn wait_for_iface(iface: &str, attempts: u32, mode: BootRuntimeMode) {
        // The virtio NIC driver is a kernel module on this kernel; load it (and
        // its deps) before expecting `eth0` to exist.
        load_network_modules(mode);
        println!("[net] {iface}: waiting for interface to appear...");
        println!("[net] /sys/class/net: [{}]", list_net_ifaces().join(", "));
        let path = format!("/sys/class/net/{iface}");
        for i in 0..attempts {
            if std::path::Path::new(&path).exists() {
                if i > 0 {
                    println!("[net] {iface}: appeared after {} ms", i * 10);
                }
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        println!(
            "[net] {iface}: did not appear after {} ms; known interfaces: [{}]",
            attempts * 10,
            list_net_ifaces().join(", ")
        );
    }

    fn wait_for_ipv6_link_ready(
        iface: &str,
        attempts: u32,
        mode: BootRuntimeMode,
    ) -> Option<(Ipv6Addr, bool)> {
        load_network_modules(mode);
        println!("[net] {iface}: waiting for IPv6 link-local readiness...");
        for i in 0..attempts {
            match fs::read_to_string("/proc/net/if_inet6") {
                Ok(raw) => {
                    if let Some(ready) = ready_ipv6_link_local_from_proc(&raw, iface) {
                        if i > 0 {
                            println!("[net] {iface}: IPv6 link-local ready after {} ms", i * 100);
                        }
                        return Some(ready);
                    }
                }
                Err(e) => {
                    println!("[net] {iface}: unable to read /proc/net/if_inet6: {e}");
                    return None;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        println!(
            "[net] {iface}: IPv6 link-local not ready after {} ms",
            attempts * 100
        );
        None
    }

    fn read_iface_mac(iface: &str) -> std::io::Result<[u8; 6]> {
        let raw = fs::read_to_string(format!("/sys/class/net/{iface}/address"))?;
        let parts: Vec<&str> = raw.trim().split(':').collect();
        if parts.len() != 6 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid MAC address {:?}", raw.trim()),
            ));
        }
        let mut mac = [0u8; 6];
        for (idx, part) in parts.iter().enumerate() {
            mac[idx] = u8::from_str_radix(part, 16).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid MAC address octet {part:?}"),
                )
            })?;
        }
        Ok(mac)
    }

    fn time_mix() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_secs() as u32) ^ d.subsec_nanos())
            .unwrap_or(0)
    }

    fn dhcp4_xid(mac: [u8; 6]) -> u32 {
        let mut xid = 0x4448_4350 ^ time_mix(); // "DHCP"
        for byte in mac {
            xid = xid.rotate_left(5) ^ u32::from(byte);
        }
        if xid == 0 { 0x3903_f326 } else { xid }
    }

    fn dhcp6_duid_llt_time() -> u32 {
        const UNIX_TO_2000_SECS: u64 = 946_684_800;
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs().saturating_sub(UNIX_TO_2000_SECS) as u32)
            .unwrap_or(0)
    }

    fn iface_index(iface: &str) -> std::io::Result<u32> {
        let ciface = CString::new(iface).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "interface name contains NUL",
            )
        })?;
        // SAFETY: ciface is a valid NUL-terminated interface name.
        let index = unsafe { libc::if_nametoindex(ciface.as_ptr()) };
        if index == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(index)
        }
    }

    fn open_dhcp4_socket(iface: &str) -> std::io::Result<UdpSocket> {
        let socket = UdpSocket::bind("0.0.0.0:68")?;
        socket.set_broadcast(true)?;
        socket.set_read_timeout(Some(Duration::from_secs(2)))?;
        bind_socket_to_device(&socket, iface);
        Ok(socket)
    }

    fn open_dhcp6_socket(
        iface: &str,
        ifindex: u32,
        link_local: Ipv6Addr,
    ) -> std::io::Result<UdpSocket> {
        let socket = UdpSocket::bind(SocketAddrV6::new(link_local, 546, 0, ifindex))?;
        bind_socket_to_device(&socket, iface);
        set_ipv6_multicast_if(&socket, ifindex, iface);
        Ok(socket)
    }

    fn bind_socket_to_device(socket: &UdpSocket, iface: &str) {
        use std::os::fd::AsRawFd;

        let Ok(ciface) = CString::new(iface) else {
            println!("[net] {iface}: SO_BINDTODEVICE skipped (invalid iface name)");
            return;
        };
        // SAFETY: fd is owned by UdpSocket and valid for the call; ciface is a
        // NUL-terminated interface name whose bytes live until setsockopt returns.
        let rc = unsafe {
            libc::setsockopt(
                socket.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_BINDTODEVICE,
                ciface.as_ptr().cast(),
                (ciface.as_bytes_with_nul().len()) as libc::socklen_t,
            )
        };
        if rc != 0 {
            println!(
                "[net] {iface}: SO_BINDTODEVICE failed (tolerated): {}",
                std::io::Error::last_os_error()
            );
        }
    }

    fn set_ipv6_multicast_if(socket: &UdpSocket, ifindex: u32, iface: &str) {
        use std::os::fd::AsRawFd;

        let index = ifindex as libc::c_uint;
        // SAFETY: fd is owned by UdpSocket and valid for the call; index points
        // to a live c_uint for the duration of setsockopt.
        let rc = unsafe {
            libc::setsockopt(
                socket.as_raw_fd(),
                libc::IPPROTO_IPV6,
                libc::IPV6_MULTICAST_IF,
                std::ptr::addr_of!(index).cast(),
                std::mem::size_of::<libc::c_uint>() as libc::socklen_t,
            )
        };
        if rc != 0 {
            println!(
                "[net] {iface}: IPV6_MULTICAST_IF failed (tolerated): {}",
                std::io::Error::last_os_error()
            );
        }
    }

    fn send_dhcp4_packet(
        socket: &UdpSocket,
        outbound: &os_network_domain::Dhcp4Outbound,
    ) -> std::io::Result<()> {
        let target: SocketAddr = match outbound.target {
            os_network_domain::Dhcp4SendTarget::Broadcast => "255.255.255.255:67".parse().unwrap(),
            os_network_domain::Dhcp4SendTarget::Unicast(addr) => {
                format!("{addr}:67").parse().map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("invalid DHCPv4 server address {addr}: {e}"),
                    )
                })?
            }
        };
        socket.send_to(&outbound.packet, target)?;
        Ok(())
    }

    fn send_dhcp6_packet(
        socket: &UdpSocket,
        ifindex: u32,
        outbound: &os_network_domain::Dhcp6Outbound,
    ) -> std::io::Result<()> {
        let target = match outbound.target {
            os_network_domain::Dhcp6SendTarget::AllDhcpRelayAgentsAndServers => {
                SocketAddrV6::new(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 1, 2), 547, 0, ifindex)
            }
        };
        socket.send_to(&outbound.packet, target)?;
        Ok(())
    }

    /// Bring a network interface up via an `ioctl(SIOCSIFFLAGS)` setting
    /// `IFF_UP`. Returns an error if the interface does not exist or the ioctl
    /// fails.
    fn bring_link_up(iface: &str) -> std::io::Result<()> {
        // SAFETY: a zeroed ifreq is a valid starting point; we fill the name.
        unsafe {
            let fd = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
            if fd < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut ifr: libc::ifreq = std::mem::zeroed();
            let name = iface.as_bytes();
            let n = name.len().min(ifr.ifr_name.len() - 1);
            for (slot, &b) in ifr.ifr_name.iter_mut().zip(name.iter()).take(n) {
                *slot = b as libc::c_char;
            }
            // Read current flags. `SIOCxIFFLAGS` are `c_ulong` constants but the
            // `ioctl` request arg is `libc::Ioctl` (c_int on musl), so narrow.
            let get_req = libc::SIOCGIFFLAGS as libc::Ioctl;
            let set_req = libc::SIOCSIFFLAGS as libc::Ioctl;
            let mut rc = libc::ioctl(fd, get_req, &mut ifr);
            if rc < 0 {
                let e = std::io::Error::last_os_error();
                libc::close(fd);
                return Err(e);
            }
            ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as libc::c_short;
            rc = libc::ioctl(fd, set_req, &ifr);
            let result = if rc < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            };
            libc::close(fd);
            result
        }
    }

    /// `fork(2)` + `execve(2)` a child process from an argv vector, returning the
    /// child's pid in the parent. The child inherits init's stdio (the console).
    fn spawn_child(argv: &[String]) -> std::io::Result<u32> {
        if argv.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "empty argv",
            ));
        }
        // Build NULL-terminated C argv. Keep the CStrings alive until after
        // execve in the child / fork in the parent.
        let c_args: Vec<CString> = argv
            .iter()
            .map(|a| CString::new(a.as_str()).unwrap())
            .collect();
        let mut c_argv: Vec<*const libc::c_char> = c_args.iter().map(|a| a.as_ptr()).collect();
        c_argv.push(std::ptr::null());
        let empty_env: [*const libc::c_char; 1] = [std::ptr::null()];

        // SAFETY: fork in a single-threaded PID1 is safe; in the child we only
        // call async-signal-safe execve and _exit.
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            return Err(std::io::Error::last_os_error());
        }
        if pid == 0 {
            // Child: replace our image with the service binary.
            unsafe {
                libc::execve(c_argv[0], c_argv.as_ptr(), empty_env.as_ptr());
                // execve only returns on failure; report and bail without
                // running atexit handlers / unwinding.
                let _ = libc::write(
                    libc::STDERR_FILENO,
                    b"talos-init: execve failed\n".as_ptr() as *const libc::c_void,
                    26,
                );
                libc::_exit(127);
            }
        }
        Ok(pid as u32)
    }

    // -----------------------------------------------------------------------
    // The ProgressLogger that prints `[seq]` lines as the sequencer runs.
    // -----------------------------------------------------------------------

    /// Prints one `[seq]` line per task/phase as the sequencer reports it,
    /// proving the ported machined logic drives the boot.
    struct SeqLogger;

    impl ProgressLogger for SeqLogger {
        fn phase_start(&mut self, index: usize, total: usize, phase: BootPhaseId) {
            println!("[seq] >>> phase {}/{} {}", index + 1, total, phase.as_str());
        }
        fn task_done(&mut self, phase: BootPhaseId, task: &str, outcome: TaskOutcome) {
            let label = match outcome {
                TaskOutcome::Done => "ok",
                TaskOutcome::Skipped => "skipped",
            };
            println!("{}", seq_task_line(phase.as_str(), task, label));
        }
        fn task_failed(&mut self, phase: BootPhaseId, task: &str, err: &MachinedError) {
            println!(
                "{}",
                seq_task_line(phase.as_str(), task, &format!("FAILED: {err}"))
            );
        }
        fn boot_done(&mut self) {
            println!("[seq] <<< all phases complete");
        }
    }

    /// Silence "field never read" lints for `DEFAULT_HOSTNAME` re-export and
    /// keep the constant referenced (it documents the compatibility contract).
    #[allow(dead_code)]
    fn _hostname_default() -> &'static str {
        DEFAULT_HOSTNAME
    }
}

// ===========================================================================
// main
// ===========================================================================

#[cfg(target_os = "linux")]
fn main() -> ! {
    init::main()
}

/// On non-Linux build hosts (e.g. macOS) the PID 1 syscalls are unavailable, so
/// the binary refuses to run. This keeps `cargo build`/`cargo test` working on
/// the host while making it impossible to accidentally "run init" off-target.
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("talos-init: this init is Linux-only and must run as PID 1 on Linux");
    std::process::exit(1);
}

// ===========================================================================
// Host-compilable unit tests for the pure helpers.
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn registryd_test_last_modified(path: &std::path::Path) -> String {
        os_runtime_cri_domain::registryd_http_last_modified_value(
            std::fs::metadata(path).unwrap().modified().unwrap(),
        )
        .expect("source-shaped last modified")
    }

    fn registryd_test_set_file_time(path: &std::path::Path, when: std::time::SystemTime) {
        let times = std::fs::FileTimes::new()
            .set_accessed(when)
            .set_modified(when);
        std::fs::File::options()
            .write(true)
            .open(path)
            .expect("open registryd fixture to set times")
            .set_times(times)
            .expect("set registryd test file times");
    }

    fn registryd_loopback_roundtrip(address: SocketAddr, request: &str) -> Vec<u8> {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        response
    }

    fn registryd_response_header_body(response: &[u8]) -> (String, &[u8]) {
        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .unwrap_or_else(|| {
                panic!(
                    "source-shaped HTTP response header terminator in {:?}",
                    String::from_utf8_lossy(response)
                )
            });
        (
            String::from_utf8_lossy(&response[..header_end]).into_owned(),
            &response[header_end..],
        )
    }

    fn registryd_response_header_value(headers: &str, name: &str) -> String {
        let prefix = format!("{name}: ");
        headers
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap_or_else(|| panic!("source-shaped HTTP response header {name}: {headers}"))
            .to_string()
    }

    #[test]
    fn ms_flags_parses_known_tokens() {
        // Matches the canonical Linux MS_* ABI values.
        assert_eq!(ms_flags_from_str("nosuid,nodev,noexec"), 2 | 4 | 8);
        assert_eq!(ms_flags_from_str("nosuid"), 2);
        assert_eq!(ms_flags_from_str("ro"), 1);
        assert_eq!(ms_flags_from_str("relatime"), 1 << 21);
    }

    #[test]
    fn ms_flags_ignores_blanks_and_unknowns() {
        assert_eq!(ms_flags_from_str(""), 0);
        assert_eq!(ms_flags_from_str("  nosuid , , bogus "), 2);
        assert_eq!(ms_flags_from_str("unknown,flags,only"), 0);
    }

    #[test]
    fn sysctl_path_dots_to_slashes() {
        assert_eq!(
            sysctl_path("kernel.kptr_restrict"),
            "/proc/sys/kernel/kptr_restrict"
        );
        assert_eq!(
            sysctl_path("net.ipv4.ip_forward"),
            "/proc/sys/net/ipv4/ip_forward"
        );
        assert_eq!(sysctl_path("vm.swappiness"), "/proc/sys/vm/swappiness");
    }

    #[test]
    fn pid1_registryd_launcher_accepts_only_source_registryd_without_claiming_health() {
        assert!(!pid1_registryd_launch_result(os_runtime_cri_domain::REGISTRYD_SERVICE_ID).unwrap());

        let err = pid1_registryd_launch_result("not-registryd").unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported registryd service id not-registryd"),
            "{err}"
        );
    }

    #[test]
    fn registryd_health_status_line_parser_accepts_only_http_status_lines() {
        assert_eq!(
            registryd_status_code_from_status_line("HTTP/1.1 204 No Content"),
            Some(204)
        );
        assert_eq!(
            registryd_status_code_from_status_line("HTTP/1.0 500 Internal Server Error"),
            Some(500)
        );
        assert_eq!(registryd_status_code_from_status_line("not-http 200"), None);
        assert_eq!(
            registryd_status_code_from_status_line("HTTP/1.1 nope"),
            None
        );
    }

    #[test]
    fn pid1_registryd_launcher_uses_loopback_health_probe_when_endpoint_answers() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;
        use std::time::Duration;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 256];
            let bytes = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..bytes]);
            assert!(request.starts_with("GET /healthz HTTP/1.1"));
            stream
                .write_all(b"HTTP/1.1 204 No Content\\r\\nContent-Length: 0\\r\\n\\r\\n")
                .unwrap();
        });

        assert!(
            pid1_registryd_launch_result_at(
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &address,
                os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
                Duration::from_secs(1),
            )
            .unwrap()
        );
        server.join().unwrap();
    }

    #[test]
    fn marker_lines_have_stable_shape() {
        assert_eq!(
            service_started_line("svc", 42),
            "service svc started pid=42"
        );
        assert_eq!(reaped_line("svc", 42, 0), "reaped svc pid=42 status=0");
        assert_eq!(
            seq_task_line("mountPseudoFs", "mountPseudoFilesystems", "ok"),
            "[seq] phase=mountPseudoFs task=mountPseudoFilesystems: ok"
        );
    }

    #[test]
    fn containerd_kubelet_service_marker_lines_have_stable_shape() {
        assert_eq!(
            service_started_line("containerd", 100),
            "service containerd started pid=100"
        );
        assert_eq!(
            service_started_line("kubelet", 101),
            "service kubelet started pid=101"
        );
        assert_eq!(
            reaped_line("containerd", 100, 0),
            "reaped containerd pid=100 status=0"
        );
        assert_eq!(
            reaped_line("kubelet", 101, 137),
            "reaped kubelet pid=101 status=137"
        );
    }

    #[test]
    fn containerd_kubelet_service_marker_pid1_override_emits_service_start_lines() {
        use os_machined_domain::boot::{
            BootSequencer, BootService, FakeRuntime, NullLogger, RestartPolicy,
        };

        let services = vec![BootService::new(SVC_NAME, [SVC_PATH], RestartPolicy::Never)];
        let seq = BootSequencer::with_services(services);
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        seq.run_boot(&mut rt, &mut log).unwrap();

        assert!(rt.logs.iter().any(|line| {
            line == "service-start: name=svc status=preparing command=/usr/bin/svc restart=never"
        }));
        assert!(
            rt.logs
                .iter()
                .any(|line| line == "service-start: name=svc status=running pid=1000")
        );
    }

    #[test]
    fn boot_cosi_bridge_marker_lines_have_stable_shape() {
        assert_eq!(
            boot_cosi_bridge_start_line(),
            "boot-cosi-network-bridge: start"
        );
        assert_eq!(
            boot_cosi_bridge_seeded_line(),
            "boot-cosi-network-bridge: seeded MachineConfigDocument"
        );
        assert_eq!(
            boot_cosi_bridge_no_seed_line(),
            "boot-cosi-network-bridge: no MachineConfigDocument seed"
        );
        assert_eq!(
            boot_cosi_bridge_stable_line(2),
            "boot-cosi-network-bridge: stable ticks=2"
        );
    }

    #[test]
    fn image_cache_copy_state_marker_line_has_stable_shape() {
        assert_eq!(
            image_cache_copy_state_line(false),
            "image-cache-runtime: copyState done=false"
        );
        assert_eq!(
            image_cache_copy_state_line(true),
            "image-cache-runtime: copyState done=true"
        );
    }

    #[test]
    fn bootstrap_network_log_shape_names_pre_config_dhcp4_and_metric() {
        assert_eq!(
            pre_config_bootstrap_line("eth0", true, true, 1024),
            "[net] eth0: pre-config metadata bootstrap (dhcp4=true dhcp6=true metric=1024)"
        );
        assert_eq!(
            pre_config_dhcp4_start_line("eth0"),
            "[net] eth0: starting pre-config DHCPv4 transaction"
        );
        assert_eq!(
            pre_config_dhcp6_start_line("eth0"),
            "[net] eth0: starting pre-config DHCPv6 rapid-solicit transaction"
        );
    }

    #[test]
    fn dhcp6_identity_helpers_match_source_defaults() {
        let mac = [0x02, 0x00, 0x5e, 0x10, 0x20, 0x30];

        assert_eq!(dhcp6_iaid_from_mac(mac), 0x5e10_2030);
        assert_eq!(dhcp6_transaction_id(mac, 0x0102_0304).len(), 3);
        assert_eq!(
            dhcp6_transaction_id(mac, 0x0102_0304),
            dhcp6_transaction_id(mac, 0x0102_0304)
        );
        assert_ne!(
            dhcp6_transaction_id(mac, 0x0102_0304),
            dhcp6_transaction_id(mac, 0x0102_0305)
        );
    }

    #[test]
    fn proc_if_inet6_parser_requires_ready_link_local_address() {
        let tentative = "\
fe8000000000000002005efffe102030 02 40 20 40 eth0\n\
20010db8000000000000000000000001 02 40 00 00 eth0\n";
        assert_eq!(ready_ipv6_link_local_from_proc(tentative, "eth0"), None);

        let ready = "\
fe8000000000000002005efffe102030 02 40 20 00 eth0\n\
fe8000000000000002005efffe102031 03 40 20 08 eth1\n";
        assert_eq!(
            ready_ipv6_link_local_from_proc(ready, "eth0"),
            Some((
                "fe80::200:5eff:fe10:2030"
                    .parse::<std::net::Ipv6Addr>()
                    .unwrap(),
                false
            ))
        );
        assert_eq!(
            ready_ipv6_link_local_from_proc(ready, "eth1"),
            Some((
                "fe80::200:5eff:fe10:2031"
                    .parse::<std::net::Ipv6Addr>()
                    .unwrap(),
                true
            ))
        );
    }

    #[test]
    fn pre_config_resolver_materializes_resolv_conf_body() {
        let resolver = os_network_domain::ResolverSpec::new_with_search(
            vec![
                os_kernel::address::NodeAddress::parse_v4("8.8.8.8").unwrap(),
                os_kernel::address::NodeAddress::parse_v4("8.8.4.4").unwrap(),
            ],
            vec!["corp.example.com".to_string(), "example.com".to_string()],
            os_network_domain::ConfigLayer::Operator,
        )
        .unwrap();

        assert_eq!(
            pre_config_resolv_conf_body(&resolver),
            "search corp.example.com example.com\nnameserver 8.8.8.8\nnameserver 8.8.4.4\n"
        );
        assert_eq!(PRE_CONFIG_RESOLV_CONF_PATH, "/etc/resolv.conf");
    }

    #[test]
    fn pre_config_resolver_write_materializes_file_via_temp_rename() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "operating-system-pre-config-resolv-conf-{}-{unique}",
            std::process::id()
        ));
        let path = root.join("etc").join("resolv.conf");

        write_pre_config_resolv_conf_at(&path, "nameserver 8.8.8.8\n").unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "nameserver 8.8.8.8\n"
        );
        assert!(!path.with_file_name("resolv.conf.talos-tmp").exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn net_errno_parses_linux_net_error_strings() {
        // rtnetlink ACK failure (kernel errno negated back to positive).
        assert_eq!(net_errno("netlink request failed: errno 1"), 1);
        // socket-level failure context prefix.
        assert_eq!(net_errno("socket(AF_NETLINK): errno 13"), 13);
        assert_eq!(net_errno("open(/sys/class/net/eth0): errno 2"), 2);
        // Absolute value taken, regardless of sign convention.
        assert_eq!(net_errno("netlink request failed: errno -1"), 1);
        // Trailing punctuation after the number is tolerated.
        assert_eq!(net_errno("foo: errno 95 (oops)"), 95);
    }

    #[test]
    fn net_errno_unparseable_yields_minus_one() {
        // No "errno" marker => -1 (forces the unexpected/fail classification).
        assert_eq!(net_errno("address exists"), -1);
        assert_eq!(net_errno(""), -1);
        assert_eq!(net_errno("errno notanumber"), -1);
    }

    #[test]
    fn svc_constants_are_consistent() {
        assert_eq!(SVC_PATH, "/usr/bin/svc");
        assert_eq!(SVC_NAME, "svc");
    }

    #[test]
    fn image_cache_runtime_plan_projection_reads_config_and_observed_disk_root() {
        use os_block_domain::{
            VolumeConfig, VolumeMountStatusResource, VolumeMountStatusSpec, VolumePhase,
            VolumeStatus,
        };
        use os_runtime_cri_domain::{
            IMAGE_CACHE_CONTROLLER_NAME, IMAGE_CACHE_DISK_MOUNT_POINT, IMAGE_CACHE_DISK_VOLUME_ID,
            ImageCacheCopyStatus, ImageCacheStatus, RegistrydAction, RegistrydState,
            image_cache_mount_status_id,
        };

        let config = "\
version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
";
        let mut disk_status = VolumeStatus::new(VolumeConfig::partition(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_VOLUME_ID,
            0,
        ));
        disk_status.phase = VolumePhase::Ready;
        let disk_mount = VolumeMountStatusResource::new(
            image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID),
            VolumeMountStatusSpec::new(
                IMAGE_CACHE_DISK_VOLUME_ID,
                IMAGE_CACHE_CONTROLLER_NAME,
                IMAGE_CACHE_DISK_MOUNT_POINT,
            )
            .with_read_only(true),
        )
        .unwrap();

        let plan = project_image_cache_runtime_plan(
            Some(config),
            RegistrydState::default(),
            &[disk_status],
            &[disk_mount],
        )
        .unwrap();

        assert_eq!(plan.config.status, ImageCacheStatus::Preparing);
        assert_eq!(plan.config.copy_status, ImageCacheCopyStatus::Skipped);
        assert_eq!(plan.config.roots, vec![IMAGE_CACHE_DISK_MOUNT_POINT]);
        assert_eq!(plan.registryd_action, RegistrydAction::Start);
    }

    #[test]
    fn image_cache_runtime_observation_applies_post_adapter_config_to_cosi_state() {
        use os_cosi_domain::State;
        use os_runtime_cri_domain::{
            IMAGE_CACHE_DISK_MOUNT_POINT, ImageCacheConfig, ImageCacheCopyStatus, ImageCacheStatus,
            RegistrydAction, image_cache_config_key,
        };

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: ImageCacheConfig {
                status: ImageCacheStatus::Preparing,
                copy_status: ImageCacheCopyStatus::Skipped,
                roots: vec![IMAGE_CACHE_DISK_MOUNT_POINT.to_string()],
            },
            registryd_action: RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };

        let observed = apply_image_cache_runtime_observation_to_state(
            &mut state,
            &plan,
            RegistrydState {
                running: true,
                healthy: true,
            },
        )
        .unwrap();

        assert_eq!(observed.config.status, ImageCacheStatus::Ready);
        assert_eq!(observed.registryd_action, RegistrydAction::None);

        let stored = state.get(&image_cache_config_key().unwrap()).unwrap();
        assert_eq!(
            stored.spec_fingerprint(),
            format!("status=ready;copy_status=skipped;roots=[{IMAGE_CACHE_DISK_MOUNT_POINT}]")
        );
    }

    #[test]
    fn image_cache_runtime_adapters_retain_loaded_registryd_service_on_boot_supervisor() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-supervisor-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "5".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "5".repeat(64)));
        let blob = b"pid1 retained registryd blob bytes";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;

        let outcome = run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        assert_eq!(
            outcome.report.status,
            os_runtime_cri_domain::RegistrydServiceExecutionStatus::LoadedAndStarted
        );
        assert!(outcome.registryd_state.running);
        assert!(outcome.registryd_state.healthy);
        assert_eq!(
            outcome.observed_plan.registryd_action,
            os_runtime_cri_domain::RegistrydAction::None
        );

        let response = supervisor
            .handle_registryd_request(
                "GET",
                &format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io"),
            )
            .expect("boot-owned supervisor retained loaded registryd service");
        assert_eq!(response.status_code, 200);
        assert_eq!(
            response.last_modified.as_deref(),
            Some(blob_last_modified.as_str())
        );
        assert_eq!(response.body, blob);
        assert_eq!(response.content_path, Some(blob_path));

        std::fs::remove_dir_all(temp).unwrap();
    }

    fn image_cache_copy_test_root(label: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "operating-system-wave176-image-cache-copy-{label}-{}-{unique}",
            std::process::id()
        ))
    }

    fn image_cache_copy_ready_plan(
        source: &std::path::Path,
        target: &std::path::Path,
    ) -> ImageCacheRuntimePlan {
        ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Ready,
                roots: vec![target.display().to_string()],
            },
            copy_plan: Some(os_runtime_cri_domain::ImageCacheCopyPlan {
                source: source.display().to_string(),
                target: target.display().to_string(),
            }),
            registryd_action: os_runtime_cri_domain::RegistrydAction::None,
            ..ImageCacheRuntimePlan::default()
        }
    }

    struct NoopRegistrydLauncher;

    impl os_machined_domain::ServiceLauncher for NoopRegistrydLauncher {
        fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
            panic!("registryd launcher should not be called for action=none: {id}");
        }
    }

    #[test]
    fn image_cache_copy_adapter_for_boot_privilege_maps_host_and_vm_gates() {
        let host = image_cache_copy_adapter_for_boot_privilege(false);
        assert_eq!(
            host.environment(),
            os_runtime_cri_domain::ImageCacheCopyRuntimeEnvironment::HostSafe
        );
        assert_eq!(host.gate(), os_runtime_cri_domain::ImageCacheCopyGate::Disabled);

        let vm = image_cache_copy_adapter_for_boot_privilege(true);
        assert_eq!(
            vm.environment(),
            os_runtime_cri_domain::ImageCacheCopyRuntimeEnvironment::VmPrivileged
        );
        assert_eq!(vm.gate(), os_runtime_cri_domain::ImageCacheCopyGate::Enabled);
    }

    #[test]
    fn image_cache_runtime_adapters_leave_copy_effect_host_safe_by_default() {
        let temp = image_cache_copy_test_root("host-safe");
        let source = temp.join("iso").join("imagecache");
        let target = temp.join("disk");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("layer"), b"cached layer").unwrap();
        let plan = image_cache_copy_ready_plan(&source, &target);
        let mut state = State::new();
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = NoopRegistrydLauncher;

        let outcome = run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        assert_eq!(
            outcome.copy_report.status,
            os_runtime_cri_domain::ImageCacheCopyExecutionStatus::DisabledByEnvironment
        );
        assert_eq!(
            image_cache_copy_execution_status_label(outcome.copy_report.status),
            "disabled-by-environment"
        );
        assert_eq!(outcome.copy_report.files_copied, 0);
        assert!(!target.join("layer").exists());
        assert!(!outcome.copy_done);
        assert_eq!(
            outcome.observed_plan.config.copy_status,
            os_runtime_cri_domain::ImageCacheCopyStatus::Ready
        );
        assert!(outcome.observed_plan.copy_plan.is_some());

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn image_cache_runtime_adapters_execute_copy_effect_for_vm_adapter_before_registryd() {
        let temp = image_cache_copy_test_root("vm-enabled");
        let source = temp.join("iso").join("imagecache");
        let target = temp.join("disk");
        std::fs::create_dir_all(source.join("nested")).unwrap();
        std::fs::write(source.join("manifest.json"), b"{}").unwrap();
        std::fs::write(source.join("nested").join("layer"), b"cached layer").unwrap();
        let plan = image_cache_copy_ready_plan(&source, &target);
        let mut state = State::new();
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = NoopRegistrydLauncher;

        let outcome = run_image_cache_runtime_adapters_with_supervisor_and_copy_adapter(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
            image_cache_copy_adapter_for_boot_privilege(true),
        )
        .unwrap();

        assert_eq!(
            outcome.copy_report.status,
            os_runtime_cri_domain::ImageCacheCopyExecutionStatus::Copied
        );
        assert_eq!(
            image_cache_copy_execution_status_label(outcome.copy_report.status),
            "copied"
        );
        assert_eq!(outcome.copy_report.files_copied, 2);
        assert_eq!(outcome.copy_report.files_skipped, 0);
        assert_eq!(outcome.copy_report.bytes_copied, 14);
        assert_eq!(std::fs::read(target.join("manifest.json")).unwrap(), b"{}");
        assert_eq!(
            std::fs::read(target.join("nested").join("layer")).unwrap(),
            b"cached layer"
        );
        assert!(outcome.copy_done);
        assert_eq!(
            outcome.observed_plan.config.copy_status,
            os_runtime_cri_domain::ImageCacheCopyStatus::Ready
        );
        assert_eq!(outcome.observed_plan.copy_plan, None);

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn image_cache_copy_success_persists_into_next_cosi_projection() {
        let temp = image_cache_copy_test_root("persisted-copy-done");
        let iso_mount = temp.join("iso");
        let source = iso_mount.join("imagecache");
        let target = temp.join("disk");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("layer"), b"cached layer").unwrap();
        std::fs::create_dir_all(&target).unwrap();

        let config = "\
version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
";
        let mut state = State::new();
        let mut disk_status = VolumeStatus::new(VolumeConfig::partition(
            os_runtime_cri_domain::IMAGE_CACHE_DISK_VOLUME_ID,
            os_runtime_cri_domain::IMAGE_CACHE_DISK_VOLUME_ID,
            0,
        ));
        disk_status.phase = VolumePhase::Ready;
        let mut iso_status = VolumeStatus::new(VolumeConfig::partition(
            os_runtime_cri_domain::IMAGE_CACHE_ISO_VOLUME_ID,
            os_runtime_cri_domain::IMAGE_CACHE_ISO_VOLUME_ID,
            0,
        ));
        iso_status.phase = VolumePhase::Ready;
        state
            .create(Box::new(VolumeStatusResource::new(disk_status).unwrap()))
            .unwrap();
        state
            .create(Box::new(VolumeStatusResource::new(iso_status).unwrap()))
            .unwrap();
        state
            .create(Box::new(
                VolumeMountStatusResource::new(
                    image_cache_mount_status_id(os_runtime_cri_domain::IMAGE_CACHE_DISK_VOLUME_ID),
                    VolumeMountStatusSpec::new(
                        os_runtime_cri_domain::IMAGE_CACHE_DISK_VOLUME_ID,
                        os_runtime_cri_domain::IMAGE_CACHE_CONTROLLER_NAME,
                        target.display().to_string(),
                    ),
                )
                .unwrap(),
            ))
            .unwrap();
        state
            .create(Box::new(
                VolumeMountStatusResource::new(
                    image_cache_mount_status_id(os_runtime_cri_domain::IMAGE_CACHE_ISO_VOLUME_ID),
                    VolumeMountStatusSpec::new(
                        os_runtime_cri_domain::IMAGE_CACHE_ISO_VOLUME_ID,
                        os_runtime_cri_domain::IMAGE_CACHE_CONTROLLER_NAME,
                        iso_mount.display().to_string(),
                    )
                    .with_read_only(true),
                )
                .unwrap(),
            ))
            .unwrap();

        let ready_registryd = RegistrydState {
            running: true,
            healthy: true,
        };
        let first_plan =
            project_image_cache_runtime_plan_from_cosi_state(Some(config), ready_registryd, &state)
                .unwrap();
        assert!(first_plan.copy_plan.is_some());

        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = NoopRegistrydLauncher;
        let outcome = run_image_cache_runtime_adapters_with_supervisor_and_copy_adapter(
            &first_plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
            image_cache_copy_adapter_for_boot_privilege(true),
        )
        .unwrap();

        assert_eq!(
            outcome.copy_report.status,
            os_runtime_cri_domain::ImageCacheCopyExecutionStatus::Copied
        );
        assert!(outcome.copy_done);
        let second_plan =
            project_image_cache_runtime_plan_from_cosi_state(Some(config), ready_registryd, &state)
                .unwrap();

        assert_eq!(
            second_plan.config.copy_status,
            os_runtime_cri_domain::ImageCacheCopyStatus::Ready
        );
        assert_eq!(second_plan.copy_plan, None);

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_request_bridge_serves_retained_supervisor_runtime_service() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-pid1-request-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "6".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "6".repeat(64)));
        let blob = b"pid1 request bridge blob bytes";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;
        run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        let health = pid1_registryd_request_result(
            &supervisor,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            "GET",
            os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
        )
        .unwrap()
        .expect("loaded registryd handles health route");
        assert_eq!(health.status_code, 200);

        let response = pid1_registryd_request_result(
            &supervisor,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            "GET",
            &format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io"),
        )
        .unwrap()
        .expect("loaded registryd handles blob route");
        assert_eq!(response.status_code, 200);
        assert_eq!(
            response.last_modified.as_deref(),
            Some(blob_last_modified.as_str())
        );
        assert_eq!(response.body, blob);
        assert_eq!(response.content_path, Some(blob_path));

        let unloaded = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        assert_eq!(
            pid1_registryd_request_result(
                &unloaded,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                "GET",
                os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
            )
            .unwrap(),
            None
        );

        let err = pid1_registryd_request_result(
            &supervisor,
            "not-registryd",
            "GET",
            os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported registryd service id not-registryd"),
            "{err}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_response_byte_bridge_serves_retained_supervisor_runtime_service() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-pid1-response-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "a".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "a".repeat(64)));
        let blob = b"pid1 response byte bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;
        run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        let bytes = pid1_registryd_response_bytes(
            &supervisor,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            "GET",
            &format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io"),
        )
        .unwrap()
        .expect("loaded registryd returns response bytes");
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], blob);

        let unloaded = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        assert_eq!(
            pid1_registryd_response_bytes(
                &unloaded,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                "GET",
                os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
            )
            .unwrap(),
            None
        );

        let err = pid1_registryd_response_bytes(
            &supervisor,
            "not-registryd",
            "GET",
            os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported registryd service id not-registryd"),
            "{err}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_response_bridge_preserves_eot_blob_content_type() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-pid1-eot-response-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "e".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "e".repeat(64)));
        let mut blob = vec![0_u8; 36];
        blob[34] = b'L';
        blob[35] = b'P';
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, &blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;
        run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        let bytes = pid1_registryd_response_bytes(
            &supervisor,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            "GET",
            &format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io"),
        )
        .unwrap()
        .expect("loaded registryd returns EOT response bytes");
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: application/vnd.ms-fontobject\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert_eq!(&bytes[prefix.len()..], blob.as_slice());

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_http_once_stream_preserves_eot_blob_content_type() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        struct MemoryStream {
            read: std::io::Cursor<Vec<u8>>,
            written: Vec<u8>,
        }

        impl MemoryStream {
            fn new(request: Vec<u8>) -> Self {
                Self {
                    read: std::io::Cursor::new(request),
                    written: Vec::new(),
                }
            }
        }

        impl std::io::Read for MemoryStream {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.read.read(buf)
            }
        }

        impl std::io::Write for MemoryStream {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.written.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-pid1-eot-http-once-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "f".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "f".repeat(64)));
        let mut blob = vec![0_u8; 36];
        blob[34] = b'L';
        blob[35] = b'P';
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, &blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;
        run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: application/vnd.ms-fontobject\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(&stream.written[prefix.len()..], blob.as_slice());

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_http_once_stream_serves_retained_supervisor_runtime_service() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        struct MemoryStream {
            read: std::io::Cursor<Vec<u8>>,
            written: Vec<u8>,
        }

        impl MemoryStream {
            fn new(request: Vec<u8>) -> Self {
                Self {
                    read: std::io::Cursor::new(request),
                    written: Vec::new(),
                }
            }
        }

        impl std::io::Read for MemoryStream {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.read.read(buf)
            }
        }

        impl std::io::Write for MemoryStream {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.written.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-pid1-http-once-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "b".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "b".repeat(64)));
        let blob = b"pid1 http once stream blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;
        run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(&stream.written[prefix.len()..], blob);
        let full_prefix = prefix.clone();

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nIf-Match: *\r\nIf-Unmodified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert!(stream.written.starts_with(full_prefix.as_bytes()));
        assert_eq!(&stream.written[full_prefix.len()..], blob);

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nIf-Match: \"sha256:test\"\r\nIf-None-Match: *\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let precondition = format!(
            "HTTP/1.1 412 Precondition Failed\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );
        assert_eq!(stream.written.as_slice(), precondition.as_bytes());

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nIf-None-Match: *\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let not_modified = format!(
            "HTTP/1.1 304 Not Modified\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );
        assert_eq!(stream.written.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nIf-None-Match: \"sha256:test\"\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert!(stream.written.starts_with(full_prefix.as_bytes()));
        assert_eq!(&stream.written[full_prefix.len()..], blob);

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nIf-Unmodified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let precondition = format!(
            "HTTP/1.1 412 Precondition Failed\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );
        assert_eq!(stream.written.as_slice(), precondition.as_bytes());

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nIf-Unmodified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert!(stream.written.starts_with(full_prefix.as_bytes()));
        assert_eq!(&stream.written[full_prefix.len()..], blob);

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let not_modified = format!(
            "HTTP/1.1 304 Not Modified\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );
        assert_eq!(stream.written.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=5-12\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert_eq!(stream.written.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=5-12\r\nIf-Range: {blob_last_modified}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 8\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes 5-12/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(&stream.written[prefix.len()..], &blob[5..=12]);

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=5-12\r\nIf-Range: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert!(stream.written.starts_with(full_prefix.as_bytes()));
        assert_eq!(&stream.written[full_prefix.len()..], blob);

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=5-12\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 8\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes 5-12/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(&stream.written[prefix.len()..], &blob[5..=12]);

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=0-3,10-13\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let response = String::from_utf8(stream.written).expect("registryd response is utf-8");
        let (headers, body) = response
            .split_once("\r\n\r\n")
            .expect("response has header delimiter");
        assert!(headers.starts_with("HTTP/1.1 206 Partial Content\r\n"));
        assert!(headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")));
        assert!(headers.contains(&format!("Last-Modified: {blob_last_modified}\r\n")));
        assert!(headers.lines().any(|line| line == "Accept-Ranges: bytes"));
        assert!(!headers.contains("\r\nContent-Range: "));
        let boundary = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Type: multipart/byteranges; boundary="))
            .expect("multipart boundary header");
        let content_length = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .expect("multipart content length")
            .parse::<usize>()
            .expect("content length is numeric");
        assert_eq!(content_length, body.len());
        assert!(body.contains(&format!(
            "--{boundary}\r\nContent-Range: bytes 0-3/{}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\npid1\r\n",
            blob.len()
        )));
        assert!(body.contains(&format!(
            "--{boundary}\r\nContent-Range: bytes 10-13/{}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nonce\r\n",
            blob.len()
        )));
        assert!(body.ends_with(&format!("--{boundary}--\r\n")));

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=-6\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let suffix_start = blob.len() - 6;
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 6\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes {suffix_start}-{}/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len() - 1,
            blob.len()
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(&stream.written[prefix.len()..], &blob[suffix_start..]);

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=-0\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 0\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes {}-{}/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len(),
            blob.len() - 1,
            blob.len()
        );
        assert_eq!(stream.written, prefix.into_bytes());

        let request = format!(
            "HEAD /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=5-12\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 8\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Range: bytes 5-12/{}\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert_eq!(stream.written, prefix.into_bytes());

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=999-1000\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 416 Requested Range Not Satisfiable\r\nContent-Length: 33\r\nDocker-Content-Digest: {blob_digest}\r\nContent-Type: text/plain; charset=utf-8\r\nX-Content-Type-Options: nosniff\r\nContent-Range: bytes */{}\r\n\r\n",
            blob.len()
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(
            &stream.written[prefix.len()..],
            b"invalid range: failed to overlap\n"
        );

        let request = format!(
            "HEAD /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=999-1000\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert_eq!(stream.written, prefix.into_bytes());

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=5-4\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 416 Requested Range Not Satisfiable\r\nContent-Length: 14\r\nDocker-Content-Digest: {blob_digest}\r\nContent-Type: text/plain; charset=utf-8\r\nX-Content-Type-Options: nosniff\r\n\r\n"
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(&stream.written[prefix.len()..], b"invalid range\n");

        let request = format!(
            "HEAD /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nRange: bytes=5-4\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_serve_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert_eq!(stream.written, prefix.into_bytes());

        let unloaded = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut stream = MemoryStream::new(
            format!(
                "GET {} HTTP/1.1\r\nHost: {}\r\n\r\n",
                os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
                os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
            )
            .into_bytes(),
        );
        assert!(
            !pid1_registryd_serve_http_once(
                &unloaded,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert!(stream.written.is_empty());

        let mut stream = MemoryStream::new(
            format!(
                "GET {} HTTP/1.1\r\nHost: {}\r\n\r\n",
                os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
                os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
            )
            .into_bytes(),
        );
        let err =
            pid1_registryd_serve_http_once(&supervisor, "not-registryd", &mut stream).unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported registryd service id not-registryd"),
            "{err}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_snapshot_serves_boot_retained_payload() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        struct MemoryStream {
            read: std::io::Cursor<Vec<u8>>,
            written: Vec<u8>,
        }

        impl MemoryStream {
            fn new(request: Vec<u8>) -> Self {
                Self {
                    read: std::io::Cursor::new(request),
                    written: Vec::new(),
                }
            }
        }

        impl std::io::Read for MemoryStream {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.read.read(buf)
            }
        }

        impl std::io::Write for MemoryStream {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.written.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-runtime-service-snapshot-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "1".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "1".repeat(64)));
        let blob = b"pid1 runtime service snapshot blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;
        run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        let service = pid1_registryd_runtime_service_snapshot(
            &supervisor,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
        )
        .unwrap()
        .expect("registryd service payload snapshot");
        assert_eq!(service.roots().roots(), std::slice::from_ref(&root));

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_runtime_service_serve_http_once(
                &service,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(&stream.written[prefix.len()..], blob);

        let empty_supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        assert!(
            pid1_registryd_runtime_service_snapshot(
                &empty_supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            )
            .unwrap()
            .is_none()
        );

        let err =
            pid1_registryd_runtime_service_snapshot(&supervisor, "not-registryd").unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported registryd service id not-registryd"),
            "{err}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_http_once_stream_serves_cloned_runtime_payload() {
        struct MemoryStream {
            read: std::io::Cursor<Vec<u8>>,
            written: Vec<u8>,
        }

        impl MemoryStream {
            fn new(request: Vec<u8>) -> Self {
                Self {
                    read: std::io::Cursor::new(request),
                    written: Vec::new(),
                }
            }
        }

        impl std::io::Read for MemoryStream {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.read.read(buf)
            }
        }

        impl std::io::Write for MemoryStream {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.written.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-runtime-service-http-once-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "e".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "e".repeat(64)));
        let blob = b"pid1 runtime-service stream blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let cloned_service = service.clone();

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_runtime_service_serve_http_once(
                &cloned_service,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(stream.written.starts_with(prefix.as_bytes()));
        assert_eq!(&stream.written[prefix.len()..], blob);

        let empty_plan = ImageCacheRuntimePlan::default();
        let empty_service =
            os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&empty_plan);
        let mut stream = MemoryStream::new(
            format!(
                "GET {} HTTP/1.1\r\nHost: {}\r\n\r\n",
                os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
                os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
            )
            .into_bytes(),
        );
        assert!(
            pid1_registryd_runtime_service_serve_http_once(
                &empty_service,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        assert!(stream.written.starts_with(b"HTTP/1.1 200 OK\r\n\r\n"));

        let mut stream = MemoryStream::new(
            format!(
                "GET {} HTTP/1.1\r\nHost: {}\r\n\r\n",
                os_runtime_cri_domain::REGISTRYD_HEALTH_PATH,
                os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
            )
            .into_bytes(),
        );
        let err = pid1_registryd_runtime_service_serve_http_once(
            &cloned_service,
            "not-registryd",
            &mut stream,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported registryd service id not-registryd"),
            "{err}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_http_once_stream_preserves_eot_blob_content_type() {
        struct MemoryStream {
            read: std::io::Cursor<Vec<u8>>,
            written: Vec<u8>,
        }

        impl MemoryStream {
            fn new(request: Vec<u8>) -> Self {
                Self {
                    read: std::io::Cursor::new(request),
                    written: Vec::new(),
                }
            }
        }

        impl std::io::Read for MemoryStream {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.read.read(buf)
            }
        }

        impl std::io::Write for MemoryStream {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.written.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-runtime-service-eot-http-once-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "a".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "a".repeat(64)));
        let mut blob = vec![0_u8; 36];
        blob[34] = b'L';
        blob[35] = b'P';
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, &blob).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let cloned_service = service.clone();

        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            os_runtime_cri_domain::REGISTRYD_LISTEN_ADDRESS
        );
        let mut stream = MemoryStream::new(request.into_bytes());
        assert!(
            pid1_registryd_runtime_service_serve_http_once(
                &cloned_service,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &mut stream,
            )
            .unwrap()
        );
        let header_end = stream
            .written
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("source-shaped HTTP response header terminator");
        let headers = String::from_utf8_lossy(&stream.written[..header_end]);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: application/vnd.ms-fontobject\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(&stream.written[header_end..], blob.as_slice());

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_serve_http_bounded_processes_loopback_requests() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-runtime-service-serve-bounded-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "f".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "f".repeat(64)));
        let blob = b"pid1 runtime-service bounded serve loop blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan).clone();

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let spawn_client = |target: String| {
            std::thread::spawn(move || {
                let mut stream = TcpStream::connect(address).unwrap();
                let request = format!(
                    "GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
                );
                stream.write_all(request.as_bytes()).unwrap();
                stream.shutdown(std::net::Shutdown::Write).unwrap();
                let mut response = Vec::new();
                stream.read_to_end(&mut response).unwrap();
                response
            })
        };
        let first = spawn_client(target.clone());
        let second = spawn_client(target);

        assert_eq!(
            pid1_registryd_runtime_service_serve_http_bounded(
                &service,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &listener,
                2,
                Duration::from_secs(1),
            )
            .unwrap(),
            2
        );

        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        for response in [first.join().unwrap(), second.join().unwrap()] {
            assert!(response.starts_with(prefix.as_bytes()));
            assert_eq!(&response[prefix.len()..], blob);
        }

        let idle_listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        assert_eq!(
            pid1_registryd_runtime_service_serve_http_bounded(
                &service,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &idle_listener,
                2,
                Duration::from_millis(10),
            )
            .unwrap(),
            0
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_serves_health_then_payload() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-runtime-service-launch-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "7".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "7".repeat(64)));
        let blob = b"pid1 runtime-service launcher blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request =
            format!("GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(response.starts_with(prefix.as_bytes()));
        assert_eq!(&response[prefix.len()..], blob);
        assert_eq!(server.stop().unwrap(), 2);

        let bad_listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let bad_address = bad_listener.local_addr().unwrap().to_string();
        let err = pid1_registryd_runtime_service_launch_result_at(
            &service,
            "not-registryd",
            bad_listener,
            &bad_address,
            Duration::from_millis(10),
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("unsupported registryd service id not-registryd"),
            "{err}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_bind_at_serves_bound_payload() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-bind-at-launcher-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "9".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "9".repeat(64)));
        let blob = b"pid1 registryd bind-at launcher blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();
        assert_eq!(address.ip(), std::net::Ipv4Addr::LOCALHOST);
        assert_ne!(address.port(), 0);

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request =
            format!("GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(
            response.starts_with(prefix.as_bytes()),
            "unexpected response: {}",
            String::from_utf8_lossy(&response)
        );
        assert_eq!(&response[prefix.len()..], blob);
        assert_eq!(launcher.stop_registryd().unwrap(), 2);

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_preserves_eot_blob_content_type() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launch-eot-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "b".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "b".repeat(64)));
        let mut blob = vec![0_u8; 36];
        blob[34] = b'L';
        blob[35] = b'P';
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, &blob).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request =
            format!("GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("source-shaped HTTP response header terminator");
        let headers = String::from_utf8_lossy(&response[..header_end]);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: application/vnd.ms-fontobject\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(&response[header_end..], blob.as_slice());
        let served = server.stop().unwrap();
        assert!(
            served >= 2,
            "expected health probe and payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_preserves_eot_blob_content_type() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launcher-eot-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "d".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "d".repeat(64)));
        let mut blob = vec![0_u8; 36];
        blob[34] = b'L';
        blob[35] = b'P';
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, &blob).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request =
            format!("GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("source-shaped HTTP response header terminator");
        let headers = String::from_utf8_lossy(&response[..header_end]);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: application/vnd.ms-fontobject\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(&response[header_end..], blob.as_slice());
        let served = launcher.stop_registryd().unwrap();
        assert!(
            served >= 2,
            "expected health probe and payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_preserves_single_range_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launch-range-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "5".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "5".repeat(64)));
        let blob = b"pid1 launcher range bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("source-shaped HTTP response header terminator");
        let headers = String::from_utf8_lossy(&response[..header_end]);
        assert!(
            headers.starts_with("HTTP/1.1 206 Partial Content\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Content-Length: 8\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Content-Range: bytes 5-12/{}\r\n", blob.len())),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(&response[header_end..], &blob[5..=12]);
        let served = server.stop().unwrap();
        assert!(
            served >= 2,
            "expected health probe and payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_preserves_single_range_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launcher-range-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "7".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "7".repeat(64)));
        let blob = b"pid1 launcher range bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("source-shaped HTTP response header terminator");
        let headers = String::from_utf8_lossy(&response[..header_end]);
        assert!(
            headers.starts_with("HTTP/1.1 206 Partial Content\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Content-Length: 8\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Content-Range: bytes 5-12/{}\r\n", blob.len())),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(&response[header_end..], &blob[5..=12]);
        let served = launcher.stop_registryd().unwrap();
        assert!(
            served >= 2,
            "expected health probe and payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_preserves_head_range_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launch-head-range-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "8".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "8".repeat(64)));
        let blob = b"pid1 launcher head range bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request = format!(
            "HEAD {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("source-shaped HTTP response header terminator");
        let headers = String::from_utf8_lossy(&response[..header_end]);
        assert!(
            headers.starts_with("HTTP/1.1 206 Partial Content\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Content-Length: 8\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Content-Range: bytes 5-12/{}\r\n", blob.len())),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(&response[header_end..], b"");
        let served = server.stop().unwrap();
        assert!(
            served >= 2,
            "expected health probe and payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_preserves_head_range_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launcher-head-range-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "9".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "9".repeat(64)));
        let blob = b"pid1 launcher head range bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request = format!(
            "HEAD {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|idx| idx + 4)
            .expect("source-shaped HTTP response header terminator");
        let headers = String::from_utf8_lossy(&response[..header_end]);
        assert!(
            headers.starts_with("HTTP/1.1 206 Partial Content\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Content-Length: 8\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Content-Range: bytes 5-12/{}\r\n", blob.len())),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(&response[header_end..], b"");
        let served = launcher.stop_registryd().unwrap();
        assert!(
            served >= 2,
            "expected health probe and payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_preserves_if_range_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launch-if-range-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "a".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "a".repeat(64)));
        let blob = b"pid1 launcher if range bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-Range: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(
            headers.starts_with("HTTP/1.1 206 Partial Content\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Content-Length: 8\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Content-Range: bytes 5-12/{}\r\n", blob.len())),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(body, &blob[5..=12]);

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-Range: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert!(!headers.contains("Content-Range: "), "{headers}");
        assert_eq!(body, blob.as_slice());

        let served = server.stop().unwrap();
        assert!(
            served >= 3,
            "expected health probe and If-Range payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_preserves_if_range_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launcher-if-range-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "b".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "b".repeat(64)));
        let blob = b"pid1 launcher if range bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-Range: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(
            headers.starts_with("HTTP/1.1 206 Partial Content\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Content-Length: 8\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Content-Range: bytes 5-12/{}\r\n", blob.len())),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(body, &blob[5..=12]);

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-Range: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert!(headers.contains("Last-Modified: "), "{headers}");
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert!(!headers.contains("Content-Range: "), "{headers}");
        assert_eq!(body, blob.as_slice());

        let served = launcher.stop_registryd().unwrap();
        assert!(
            served >= 3,
            "expected health probe and If-Range payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_preserves_if_modified_since_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launch-if-modified-since-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "c".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "c".repeat(64)));
        let blob = b"pid1 launcher if modified since bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let not_modified = format!(
            "HTTP/1.1 304 Not Modified\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Modified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert!(!headers.contains("Content-Range: "), "{headers}");
        assert_eq!(body, blob.as_slice());

        let served = server.stop().unwrap();
        assert!(
            served >= 4,
            "expected health probe and If-Modified-Since payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_preserves_if_modified_since_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launcher-if-modified-since-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "d".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "d".repeat(64)));
        let blob = b"pid1 launcher if modified since bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let not_modified = format!(
            "HTTP/1.1 304 Not Modified\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Modified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert!(!headers.contains("Content-Range: "), "{headers}");
        assert_eq!(body, blob.as_slice());

        let served = launcher.stop_registryd().unwrap();
        assert!(
            served >= 4,
            "expected health probe and If-Modified-Since payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_preserves_if_none_match_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launch-if-none-match-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "e".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "e".repeat(64)));
        let blob = b"pid1 launcher if none match bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let not_modified = format!(
            "HTTP/1.1 304 Not Modified\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-None-Match: *\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-None-Match: *\r\nIf-Modified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-None-Match: \"sha256:test\"\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert!(!headers.contains("Content-Range: "), "{headers}");
        assert_eq!(body, blob.as_slice());

        let served = server.stop().unwrap();
        assert!(
            served >= 4,
            "expected health probe and If-None-Match payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_preserves_if_none_match_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launcher-if-none-match-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "f".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "f".repeat(64)));
        let blob = b"pid1 launcher if none match bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let not_modified = format!(
            "HTTP/1.1 304 Not Modified\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-None-Match: *\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-None-Match: *\r\nIf-Modified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), not_modified.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-None-Match: \"sha256:test\"\r\nIf-Modified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert!(!headers.contains("Content-Range: "), "{headers}");
        assert_eq!(body, blob.as_slice());

        let served = launcher.stop_registryd().unwrap();
        assert!(
            served >= 4,
            "expected health probe and If-None-Match payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_preserves_if_match_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launch-if-match-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "0".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "0".repeat(64)));
        let blob = b"pid1 launcher if match bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Match: *\r\nIf-Unmodified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(body, blob.as_slice());

        let precondition = format!(
            "HTTP/1.1 412 Precondition Failed\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Match: \"sha256:test\"\r\nIf-Unmodified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), precondition.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Match: \"sha256:test\"\r\nIf-None-Match: *\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), precondition.as_bytes());

        let served = server.stop().unwrap();
        assert!(
            served >= 4,
            "expected health probe and If-Match payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_preserves_if_match_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launcher-if-match-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "1".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "1".repeat(64)));
        let blob = b"pid1 launcher if match bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Match: *\r\nIf-Unmodified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert_eq!(body, blob.as_slice());

        let precondition = format!(
            "HTTP/1.1 412 Precondition Failed\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Match: \"sha256:test\"\r\nIf-Unmodified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), precondition.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Match: \"sha256:test\"\r\nIf-None-Match: *\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), precondition.as_bytes());

        let served = launcher.stop_registryd().unwrap();
        assert!(
            served >= 4,
            "expected health probe and If-Match payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launch_result_preserves_if_unmodified_since_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launch-if-unmodified-since-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "2".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "2".repeat(64)));
        let blob = b"pid1 launcher if unmodified since bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();

        let (healthy, server) = pid1_registryd_runtime_service_launch_result_at(
            &service,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
            listener,
            &address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        assert!(healthy);

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let precondition = format!(
            "HTTP/1.1 412 Precondition Failed\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Unmodified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), precondition.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-Unmodified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), precondition.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Unmodified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert!(!headers.contains("Content-Range: "), "{headers}");
        assert_eq!(body, blob.as_slice());

        let served = server.stop().unwrap();
        assert!(
            served >= 4,
            "expected health probe and If-Unmodified-Since payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_preserves_if_unmodified_since_over_loopback() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-launcher-if-unmodified-since-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "3".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "3".repeat(64)));
        let blob = b"pid1 launcher if unmodified since bridge blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let source_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_704_067_200);
        registryd_test_set_file_time(&blob_path, source_time);
        let blob_last_modified =
            os_runtime_cri_domain::registryd_http_last_modified_value(source_time).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();

        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let precondition = format!(
            "HTTP/1.1 412 Precondition Failed\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\n\r\n"
        );

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Unmodified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), precondition.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nRange: bytes=5-12\r\nIf-Unmodified-Since: Sun, 06 Nov 1994 08:49:37 GMT\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        assert_eq!(response.as_slice(), precondition.as_bytes());

        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {address}\r\nIf-Unmodified-Since: {blob_last_modified}\r\nConnection: close\r\n\r\n"
        );
        let response = registryd_loopback_roundtrip(address, &request);
        let (headers, body) = registryd_response_header_body(&response);
        assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"), "{headers}");
        assert!(
            headers.contains(&format!("Content-Length: {}\r\n", blob.len())),
            "{headers}"
        );
        assert!(
            headers.contains(&format!("Docker-Content-Digest: {blob_digest}\r\n")),
            "{headers}"
        );
        assert_eq!(
            registryd_response_header_value(&headers, "Last-Modified"),
            blob_last_modified
        );
        assert!(
            headers.contains("Content-Type: text/plain; charset=utf-8\r\n"),
            "{headers}"
        );
        assert!(headers.contains("Accept-Ranges: bytes\r\n"), "{headers}");
        assert!(!headers.contains("Content-Range: "), "{headers}");
        assert_eq!(body, blob.as_slice());

        let served = launcher.stop_registryd().unwrap();
        assert!(
            served >= 4,
            "expected health probe and If-Unmodified-Since payload requests, got {served}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_stop_tears_down_bound_server() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-stop-launcher-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_path = root.join(format!("blob/sha256-{}", "6".repeat(64)));
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, b"pid1 registryd stop launcher blob").unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let address: SocketAddr = launcher.health_address().parse().unwrap();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);

        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );
        os_machined_domain::ServiceLauncher::stop(
            &mut launcher,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
        )
        .unwrap();

        let stopped_status = registryd_http_health_status_at(
            &address.to_string(),
            REGISTRYD_HEALTH_PATH,
            Duration::from_millis(50),
        )
        .unwrap();
        let remaining_served = launcher.stop_registryd().unwrap();

        assert_eq!(stopped_status, None);
        assert_eq!(remaining_served, 0);

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_stop_wakes_idle_server_before_accept_deadline() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-stop-wake-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        std::fs::create_dir_all(&root).unwrap();

        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::bind_at(
            "127.0.0.1:0",
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        )
        .unwrap();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let service = os_runtime_cri_domain::RegistrydRuntimeService::from_runtime_plan(&plan);
        assert!(
            launcher
                .launch_registryd_runtime_service(os_runtime_cri_domain::REGISTRYD_SERVICE_ID, &service)
                .unwrap()
        );

        let started = std::time::Instant::now();
        os_machined_domain::ServiceLauncher::stop(
            &mut launcher,
            os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
        )
        .unwrap();
        let elapsed = started.elapsed();

        assert!(
            elapsed < Duration::from_millis(500),
            "registryd stop waited for idle accept deadline: {elapsed:?}"
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_drives_runtime_adapters_to_loopback_payload() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-adapter-loopback-launcher-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "8".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "8".repeat(64)));
        let blob = b"pid1 runtime adapter loopback launcher blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::new(
            listener,
            address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        );

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);

        let outcome = run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        assert_eq!(
            outcome.report.status,
            os_runtime_cri_domain::RegistrydServiceExecutionStatus::LoadedAndStarted
        );
        assert!(outcome.registryd_state.running);
        assert!(outcome.registryd_state.healthy);
        assert_eq!(
            outcome.observed_plan.config.status,
            os_runtime_cri_domain::ImageCacheStatus::Ready
        );
        assert_eq!(
            outcome.observed_plan.registryd_action,
            os_runtime_cri_domain::RegistrydAction::None
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request =
            format!("GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(response.starts_with(prefix.as_bytes()));
        assert_eq!(&response[prefix.len()..], blob);
        assert_eq!(launcher.stop_registryd().unwrap(), 2);

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_runtime_service_launcher_stop_all_tears_down_adapter_server() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-adapter-stop-all-launcher-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "9".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "9".repeat(64)));
        let blob = b"pid1 runtime adapter stop all launcher blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let mut launcher = Pid1RegistrydRuntimeServiceLauncher::new(
            listener,
            address.to_string(),
            PID1_REGISTRYD_HEALTH_PROBE_TIMEOUT,
        );

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);

        let outcome = run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        assert_eq!(
            outcome.report.status,
            os_runtime_cri_domain::RegistrydServiceExecutionStatus::LoadedAndStarted
        );
        assert!(outcome.registryd_state.running);
        assert!(outcome.registryd_state.healthy);
        assert_eq!(
            supervisor.state_of(os_runtime_cri_domain::REGISTRYD_SERVICE_ID),
            Some(os_machined_domain::ServiceState::Healthy)
        );

        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let mut stream = TcpStream::connect(address).unwrap();
        let request =
            format!("GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(response.starts_with(prefix.as_bytes()));
        assert_eq!(&response[prefix.len()..], blob);

        supervisor.stop_all(&mut launcher).unwrap();

        assert_eq!(
            supervisor.state_of(os_runtime_cri_domain::REGISTRYD_SERVICE_ID),
            Some(os_machined_domain::ServiceState::Finished)
        );
        assert_eq!(
            registryd_http_health_status_at(
                &address.to_string(),
                REGISTRYD_HEALTH_PATH,
                Duration::from_millis(50),
            )
            .unwrap(),
            None
        );
        assert_eq!(launcher.stop_registryd().unwrap(), 0);

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_accept_http_once_serves_one_loopback_connection() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-pid1-accept-once-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "c".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "c".repeat(64)));
        let blob = b"pid1 accept once loopback blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;
        run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let request = format!(
            "GET /v2/library/alpine/blobs/{blob_digest}?ns=docker.io HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n",
        );
        let client = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(address).unwrap();
            stream.write_all(request.as_bytes()).unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut response = Vec::new();
            stream.read_to_end(&mut response).unwrap();
            response
        });

        assert!(
            pid1_registryd_accept_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &listener,
                Duration::from_secs(1),
            )
            .unwrap()
        );
        let response = client.join().unwrap();
        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        assert!(response.starts_with(prefix.as_bytes()));
        assert_eq!(&response[prefix.len()..], blob);

        let idle_listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        assert!(
            !pid1_registryd_accept_http_once(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &idle_listener,
                Duration::from_millis(10),
            )
            .unwrap()
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pid1_registryd_serve_http_bounded_processes_multiple_loopback_requests() {
        struct HealthyRegistrydLauncher;

        impl os_machined_domain::ServiceLauncher for HealthyRegistrydLauncher {
            fn launch(&mut self, id: &str) -> os_machined_domain::error::Result<bool> {
                assert_eq!(id, os_runtime_cri_domain::REGISTRYD_SERVICE_ID);
                Ok(true)
            }
        }

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!(
            "operating-system-image-cache-registryd-pid1-serve-bounded-{}-{unique}",
            std::process::id()
        ));
        let root = temp.join("root");
        let blob_digest = format!("sha256:{}", "d".repeat(64));
        let blob_path = root.join(format!("blob/sha256-{}", "d".repeat(64)));
        let blob = b"pid1 bounded serve loop blob";
        std::fs::create_dir_all(blob_path.parent().unwrap()).unwrap();
        std::fs::write(&blob_path, blob).unwrap();
        let blob_last_modified = registryd_test_last_modified(&blob_path);

        let mut state = State::new();
        let plan = ImageCacheRuntimePlan {
            config: os_runtime_cri_domain::ImageCacheConfig {
                status: os_runtime_cri_domain::ImageCacheStatus::Preparing,
                copy_status: os_runtime_cri_domain::ImageCacheCopyStatus::Skipped,
                roots: vec![root.display().to_string()],
            },
            registryd_action: os_runtime_cri_domain::RegistrydAction::Start,
            ..ImageCacheRuntimePlan::default()
        };
        let mut supervisor = os_machined_domain::Supervisor::new(os_kernel::MachineType::Worker);
        let mut launcher = HealthyRegistrydLauncher;
        run_image_cache_runtime_adapters_with_supervisor(
            &plan,
            &mut state,
            &mut supervisor,
            &mut launcher,
        )
        .unwrap();

        let listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let target = format!("/v2/library/alpine/blobs/{blob_digest}?ns=docker.io");
        let spawn_client = |target: String| {
            std::thread::spawn(move || {
                let mut stream = TcpStream::connect(address).unwrap();
                let request = format!(
                    "GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
                );
                stream.write_all(request.as_bytes()).unwrap();
                stream.shutdown(std::net::Shutdown::Write).unwrap();
                let mut response = Vec::new();
                stream.read_to_end(&mut response).unwrap();
                response
            })
        };
        let first = spawn_client(target.clone());
        let second = spawn_client(target);

        assert_eq!(
            pid1_registryd_serve_http_bounded(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &listener,
                2,
                Duration::from_secs(1),
            )
            .unwrap(),
            2
        );

        let prefix = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nDocker-Content-Digest: {blob_digest}\r\nLast-Modified: {blob_last_modified}\r\nContent-Type: text/plain; charset=utf-8\r\nAccept-Ranges: bytes\r\n\r\n",
            blob.len()
        );
        for response in [first.join().unwrap(), second.join().unwrap()] {
            assert!(response.starts_with(prefix.as_bytes()));
            assert_eq!(&response[prefix.len()..], blob);
        }

        let idle_listener = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap();
        assert_eq!(
            pid1_registryd_serve_http_bounded(
                &supervisor,
                os_runtime_cri_domain::REGISTRYD_SERVICE_ID,
                &idle_listener,
                2,
                Duration::from_millis(10),
            )
            .unwrap(),
            0
        );

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn image_cache_runtime_plan_projection_stays_disabled_without_feature_flag() {
        use os_block_domain::{VolumeConfig, VolumePhase, VolumeStatus};
        use os_runtime_cri_domain::{
            IMAGE_CACHE_DISK_VOLUME_ID, ImageCacheCopyStatus, ImageCacheStatus, RegistrydAction,
            RegistrydState,
        };

        let mut disk_status = VolumeStatus::new(VolumeConfig::partition(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_VOLUME_ID,
            0,
        ));
        disk_status.phase = VolumePhase::Ready;

        let plan = project_image_cache_runtime_plan(
            Some("version: v1alpha1\nmachine:\n  type: worker\n"),
            RegistrydState::default(),
            &[disk_status],
            &[],
        )
        .unwrap();

        assert_eq!(plan.config.status, ImageCacheStatus::Disabled);
        assert_eq!(plan.config.copy_status, ImageCacheCopyStatus::Skipped);
        assert_eq!(plan.registryd_action, RegistrydAction::None);
    }

    #[test]
    fn image_cache_runtime_plan_projection_reads_cosi_block_observations() {
        use os_block_domain::{
            VolumeConfig, VolumeMountStatusResource, VolumeMountStatusSpec, VolumePhase,
            VolumeStatus, VolumeStatusResource,
        };
        use os_cosi_domain::State;
        use os_runtime_cri_domain::{
            IMAGE_CACHE_CONTROLLER_NAME, IMAGE_CACHE_DISK_VOLUME_ID, ImageCacheStatus,
            RegistrydAction, RegistrydState, image_cache_mount_status_id,
        };

        let config = "\
version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
";
        let mut state = State::new();
        let mut disk_status = VolumeStatus::new(VolumeConfig::partition(
            IMAGE_CACHE_DISK_VOLUME_ID,
            IMAGE_CACHE_DISK_VOLUME_ID,
            0,
        ));
        disk_status.phase = VolumePhase::Ready;
        let observed_disk_target = "/observed/imagecache/disk";
        state
            .create(Box::new(VolumeStatusResource::new(disk_status).unwrap()))
            .unwrap();
        state
            .create(Box::new(
                VolumeMountStatusResource::new(
                    image_cache_mount_status_id(IMAGE_CACHE_DISK_VOLUME_ID),
                    VolumeMountStatusSpec::new(
                        IMAGE_CACHE_DISK_VOLUME_ID,
                        IMAGE_CACHE_CONTROLLER_NAME,
                        observed_disk_target,
                    )
                    .with_read_only(true),
                )
                .unwrap(),
            ))
            .unwrap();

        let plan = project_image_cache_runtime_plan_from_cosi_state(
            Some(config),
            RegistrydState::default(),
            &state,
        )
        .unwrap();

        assert_eq!(plan.config.status, ImageCacheStatus::Preparing);
        assert_eq!(plan.config.roots, vec![observed_disk_target]);
        assert_eq!(plan.registryd_action, RegistrydAction::Start);
    }

    #[test]
    fn image_cache_declared_volume_status_hydrates_boot_cosi_state_without_starting_registryd() {
        use os_block_domain::{IMAGE_CACHE_VOLUME_ID, VolumePhase, VolumeStatusResource};
        use os_cosi_domain::State;
        use os_runtime_cri_domain::{
            ImageCacheStatus, RegistrydAction, RegistrydState, image_cache_mount_status_id,
        };

        let config = "\
version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
---
apiVersion: v1alpha1
kind: VolumeConfig
name: IMAGECACHE
provisioning:
  maxSize: 10737418240
";
        let mut state = State::new();

        assert!(hydrate_declared_image_cache_block_state(Some(config), &mut state).unwrap());
        assert!(!hydrate_declared_image_cache_block_state(Some(config), &mut state).unwrap());

        let key = format!("runtime/VolumeStatuses.block.talos.dev/{IMAGE_CACHE_VOLUME_ID}");
        let observed = state.get(&key).expect("declared IMAGECACHE status");
        let parsed = VolumeStatusResource::from_resource(observed.as_ref()).unwrap();
        assert_eq!(parsed.status.config.id, IMAGE_CACHE_VOLUME_ID);
        assert_eq!(parsed.status.phase, VolumePhase::Waiting);

        let plan = project_image_cache_runtime_plan_from_cosi_state(
            Some(config),
            RegistrydState::default(),
            &state,
        )
        .unwrap();

        assert_eq!(plan.config.status, ImageCacheStatus::Disabled);
        assert!(plan.config.roots.is_empty());
        assert_eq!(plan.registryd_action, RegistrydAction::None);
        assert_eq!(plan.mount_requests.len(), 1);
        assert_eq!(
            plan.mount_requests[0].id,
            image_cache_mount_status_id(IMAGE_CACHE_VOLUME_ID)
        );
    }

    #[test]
    fn image_cache_bootstrap_mount_state_seeds_ready_roots_when_canonical_paths_exist() {
        use os_block_domain::{IMAGE_CACHE_VOLUME_ID, VolumeMountStatusResource, VolumeStatusResource};
        use os_cosi_domain::State;
        use os_runtime_cri_domain::{
            IMAGE_CACHE_DISK_MOUNT_POINT, IMAGE_CACHE_DISK_VOLUME_ID, IMAGE_CACHE_ISO_MOUNT_POINT,
            IMAGE_CACHE_ISO_VOLUME_ID, ImageCacheCopyPlan, ImageCacheCopyStatus, ImageCacheStatus,
            RegistrydAction, RegistrydState, image_cache_mount_status_id,
        };

        let config = "version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
---
apiVersion: v1alpha1
kind: VolumeConfig
name: IMAGECACHE
provisioning:
  maxSize: 10737418240
";
        let mut state = State::new();
        assert!(hydrate_declared_image_cache_block_state(Some(config), &mut state).unwrap());

        let seeded = hydrate_bootstrap_image_cache_mount_state_with_probe(
            Some(config),
            &mut state,
            |path| {
                path == IMAGE_CACHE_DISK_MOUNT_POINT
                    || path == format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache")
            },
        )
        .unwrap();

        assert_eq!(seeded, 4);
        let disk_key = format!("runtime/VolumeStatuses.block.talos.dev/{IMAGE_CACHE_VOLUME_ID}");
        let disk = VolumeStatusResource::from_resource(state.get(&disk_key).unwrap().as_ref())
            .expect("disk volume status");
        assert_eq!(disk.status.phase, os_block_domain::VolumePhase::Ready);

        for volume_id in [IMAGE_CACHE_DISK_VOLUME_ID, IMAGE_CACHE_ISO_VOLUME_ID] {
            let mount_key = format!(
                "runtime/VolumeMountStatuses.block.talos.dev/{}",
                image_cache_mount_status_id(volume_id)
            );
            assert!(
                VolumeMountStatusResource::from_resource(state.get(&mount_key).unwrap().as_ref())
                    .is_some(),
                "missing {mount_key}"
            );
        }

        let plan = project_image_cache_runtime_plan_from_cosi_state(
            Some(config),
            RegistrydState::default(),
            &state,
        )
        .unwrap();

        assert_eq!(plan.config.status, ImageCacheStatus::Preparing);
        assert_eq!(plan.config.copy_status, ImageCacheCopyStatus::Ready);
        assert_eq!(
            plan.config.roots,
            vec![
                IMAGE_CACHE_DISK_MOUNT_POINT.to_string(),
                format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache"),
            ]
        );
        assert_eq!(plan.registryd_action, RegistrydAction::Start);
        assert_eq!(
            plan.copy_plan,
            Some(ImageCacheCopyPlan {
                source: format!("{IMAGE_CACHE_ISO_MOUNT_POINT}/imagecache"),
                target: IMAGE_CACHE_DISK_MOUNT_POINT.to_string(),
            })
        );
    }

    #[test]
    fn image_cache_bootstrap_mount_state_does_not_synthesize_missing_roots() {
        use os_cosi_domain::State;
        use os_runtime_cri_domain::{ImageCacheStatus, RegistrydAction, RegistrydState};

        let config = "version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
";
        let mut state = State::new();

        let seeded = hydrate_bootstrap_image_cache_mount_state_with_probe(
            Some(config),
            &mut state,
            |_path| false,
        )
        .unwrap();

        assert_eq!(seeded, 0);
        let plan = project_image_cache_runtime_plan_from_cosi_state(
            Some(config),
            RegistrydState::default(),
            &state,
        )
        .unwrap();
        assert_eq!(plan.config.status, ImageCacheStatus::Disabled);
        assert!(plan.config.roots.is_empty());
        assert_eq!(plan.registryd_action, RegistrydAction::None);
    }

    #[test]
    fn boot_sequencer_drives_helpers_via_fake_runtime() {
        // Prove our chosen API (with_services + run_boot) works end-to-end
        // against the machined FakeRuntime, on the host, with our svc service.
        use os_machined_domain::boot::{
            BootSequencer, BootService, FakeRuntime, NullLogger, RestartPolicy,
        };
        let services = vec![BootService::new(SVC_NAME, [SVC_PATH], RestartPolicy::Never)];
        let seq = BootSequencer::with_services(services);
        let mut rt = FakeRuntime::new("metal");
        let mut log = NullLogger;
        let report = seq.run_boot(&mut rt, &mut log).unwrap();
        assert_eq!(report.phase_count(), 7);
        assert_eq!(rt.spawned.len(), 1);
        assert_eq!(rt.spawned[0].0, SVC_NAME);
    }
}
