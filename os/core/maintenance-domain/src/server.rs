//! The maintenance server: wires the state machine, validator, persistence
//! sink, TLS bootstrap, installer and rebooter boundaries together and
//! dispatches RPCs. Also models the HTTP/dashboard config-acquisition endpoint.
//!
//! This corresponds to `internal/app/maintenance.Run` + the small HTTP server
//! Talos brings up so an operator can `POST` a config (or the dashboard can
//! display the maintenance URL). The server:
//!
//! 1. bootstraps a self-signed cert for the node's SANs,
//! 2. transitions Booting -> Bootstrapping -> Serving,
//! 3. accepts `ApplyConfiguration` over gRPC or HTTP, validates and persists,
//! 4. triggers an install (if the config requests one) and a reboot, leaving
//!    maintenance mode.

use os_kernel::Clock;

use crate::config_apply::{
    ApplyConfigInput, ApplyConfigOutcome, ConfigSink, ConfigValidator, DefaultConfigValidator,
    InMemoryConfigSink, apply_configuration,
};
use crate::service::{
    GenerateRequest, GenerateResponse, MaintenanceError, MaintenanceMethod, MaintenancePhase,
    MaintenanceService, MaintenanceState, ResetRequest, UpgradeRequest, UpgradeResponse,
};
use crate::tls::{
    CertBootstrap, DEFAULT_VALIDITY_SECS, InMemoryCertBootstrap, SubjectAltNames, TlsConfig,
};

/// A request to install Talos to disk, derived from an applied config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallRequest {
    /// The target install disk (from `machine.install.disk`).
    pub disk: Option<String>,
    /// The config that triggered the install.
    pub config: Vec<u8>,
}

/// The install boundary: triggers an install-from-maintenance.
///
/// Real Talos hands off to the installer image; here it is a trait so the
/// install-from-maintenance flow is testable.
pub trait Installer {
    /// Trigger an install. Returns an error string on failure.
    fn install(&mut self, req: &InstallRequest) -> Result<(), String>;
}

/// An [`Installer`] that does nothing (config will be picked up on reboot
/// without a disk install — e.g. a container/metal node already installed).
#[derive(Debug, Default, Clone)]
pub struct NoopInstaller;

impl Installer for NoopInstaller {
    fn install(&mut self, _req: &InstallRequest) -> Result<(), String> {
        Ok(())
    }
}

/// An [`Installer`] that records the install requests it received.
#[derive(Debug, Default, Clone)]
pub struct RecordingInstaller {
    /// Every install request handled, in order.
    pub requests: Vec<InstallRequest>,
    fail: bool,
}

impl RecordingInstaller {
    /// A new recorder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Make the next install fail.
    pub fn failing() -> Self {
        RecordingInstaller {
            requests: Vec::new(),
            fail: true,
        }
    }
}

impl Installer for RecordingInstaller {
    fn install(&mut self, req: &InstallRequest) -> Result<(), String> {
        if self.fail {
            return Err("install failed (injected)".to_string());
        }
        self.requests.push(req.clone());
        Ok(())
    }
}

/// The reboot/power boundary.
pub trait Rebooter {
    /// Reboot the node. Returns an error string on failure.
    fn reboot(&mut self) -> Result<(), String>;
}

/// A [`Rebooter`] that records how many times reboot was requested.
#[derive(Debug, Default, Clone)]
pub struct RecordingRebooter {
    /// Number of reboot requests.
    pub reboots: usize,
}

impl Rebooter for RecordingRebooter {
    fn reboot(&mut self) -> Result<(), String> {
        self.reboots += 1;
        Ok(())
    }
}

/// Where a config came from when applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// Applied via the gRPC `ApplyConfiguration` RPC.
    Grpc,
    /// Applied via the HTTP config endpoint (`POST /config`).
    Http,
    /// Acquired from platform metadata while in maintenance.
    PlatformMetadata,
}

/// The maintenance server, generic over its OS boundaries.
///
/// `C` is the cert bootstrap, `V` the config validator, `S` the persistence
/// sink, `I` the installer, `R` the rebooter, and `K` the clock.
pub struct MaintenanceServer<C, V, S, I, R, K>
where
    C: CertBootstrap,
    V: ConfigValidator,
    S: ConfigSink,
    I: Installer,
    R: Rebooter,
    K: Clock,
{
    state: MaintenanceState,
    cert_bootstrap: C,
    validator: V,
    sink: S,
    installer: I,
    rebooter: R,
    clock: K,
    tls: Option<TlsConfig>,
    last_outcome: Option<ApplyConfigOutcome>,
}

impl<C, V, S, I, R, K> MaintenanceServer<C, V, S, I, R, K>
where
    C: CertBootstrap,
    V: ConfigValidator,
    S: ConfigSink,
    I: Installer,
    R: Rebooter,
    K: Clock,
{
    /// Build a server from explicit boundaries.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: MaintenanceState,
        cert_bootstrap: C,
        validator: V,
        sink: S,
        installer: I,
        rebooter: R,
        clock: K,
    ) -> Self {
        MaintenanceServer {
            state,
            cert_bootstrap,
            validator,
            sink,
            installer,
            rebooter,
            clock,
            tls: None,
            last_outcome: None,
        }
    }

    /// The current maintenance phase.
    pub fn phase(&self) -> MaintenancePhase {
        self.state.phase()
    }

    /// Read-only access to the underlying state.
    pub fn state(&self) -> &MaintenanceState {
        &self.state
    }

    /// The active TLS config, if bootstrapped.
    pub fn tls(&self) -> Option<&TlsConfig> {
        self.tls.as_ref()
    }

    /// The persistence sink (e.g. to assert what was written).
    pub fn sink(&self) -> &S {
        &self.sink
    }

    /// The installer boundary.
    pub fn installer(&self) -> &I {
        &self.installer
    }

    /// The rebooter boundary.
    pub fn rebooter(&self) -> &R {
        &self.rebooter
    }

    /// The outcome of the last successful apply, if any.
    pub fn last_outcome(&self) -> Option<&ApplyConfigOutcome> {
        self.last_outcome.as_ref()
    }

    /// Build the SAN set for this node: loopback + hostname + addresses.
    fn build_sans(&self) -> SubjectAltNames {
        let mut sans = SubjectAltNames::with_loopback();
        sans.add_dns(self.state.hostname());
        for addr in self.state.addresses() {
            sans.add_ip(addr.clone());
        }
        sans
    }

    /// Bootstrap the self-signed certificate and bind the listener, taking the
    /// node from Booting through Bootstrapping to Serving.
    pub fn bootstrap(&mut self) -> Result<&TlsConfig, MaintenanceError> {
        self.state.transition_to(MaintenancePhase::Bootstrapping)?;
        let sans = self.build_sans();
        let cert = self.cert_bootstrap.generate_self_signed(
            sans,
            self.clock.now_unix_secs(),
            DEFAULT_VALIDITY_SECS,
        );
        self.tls = Some(TlsConfig::maintenance(cert));
        self.state.transition_to(MaintenancePhase::Serving)?;
        Ok(self.tls.as_ref().expect("tls set"))
    }

    /// Guard: a method may only be dispatched if the maintenance service is
    /// serving.
    fn require_serving(&self, method: &'static str) -> Result<(), MaintenanceError> {
        if self.state.is_serving() {
            Ok(())
        } else {
            Err(MaintenanceError::WrongPhase {
                method,
                phase: self.state.phase(),
            })
        }
    }

    /// Dispatch by parsed gRPC method name; returns whether the method is even
    /// available in maintenance mode. Used to model the gRPC gate.
    pub fn is_method_available(&self, grpc_name: &str) -> bool {
        MaintenanceMethod::parse(grpc_name).is_some()
    }

    /// The full apply-from-maintenance flow: validate + persist + (install) +
    /// (reboot), advancing the state machine out of maintenance.
    ///
    /// This is the heart of "install from maintenance": when the applied config
    /// requests a disk install, the installer boundary is invoked before the
    /// reboot.
    pub fn apply_and_leave(
        &mut self,
        input: &ApplyConfigInput,
        source: ConfigSource,
    ) -> Result<ApplyConfigOutcome, MaintenanceError> {
        self.require_serving("ApplyConfiguration")?;
        let _ = source; // source is informational; recorded by caller if needed

        let outcome = apply_configuration(&self.validator, &mut self.sink, input)
            .map_err(|e| MaintenanceError::Apply(e.to_string()))?;

        if outcome.dry_run {
            // A dry run does not change phase or trigger side effects.
            self.last_outcome = Some(outcome.clone());
            return Ok(outcome);
        }

        // Config persisted: move to ConfigApplied.
        self.state.transition_to(MaintenancePhase::ConfigApplied)?;

        if outcome.install {
            let req = InstallRequest {
                disk: outcome.stored.install_disk.clone(),
                config: outcome.stored.data.clone(),
            };
            self.installer
                .install(&req)
                .map_err(|e| MaintenanceError::Apply(format!("install: {e}")))?;
        }

        if outcome.reboot {
            self.rebooter
                .reboot()
                .map_err(|e| MaintenanceError::Apply(format!("reboot: {e}")))?;
            self.state
                .transition_to(MaintenancePhase::LeavingMaintenance)?;
        }

        self.last_outcome = Some(outcome.clone());
        Ok(outcome)
    }
}

impl<C, V, S, I, R, K> MaintenanceService for MaintenanceServer<C, V, S, I, R, K>
where
    C: CertBootstrap,
    V: ConfigValidator,
    S: ConfigSink,
    I: Installer,
    R: Rebooter,
    K: Clock,
{
    fn version(&self) -> Result<String, MaintenanceError> {
        Ok(self.state.version().to_string())
    }

    fn hostname(&self) -> Result<String, MaintenanceError> {
        Ok(self.state.hostname().to_string())
    }

    fn generate_configuration(
        &self,
        req: &GenerateRequest,
    ) -> Result<GenerateResponse, MaintenanceError> {
        self.require_serving("GenerateConfiguration")?;
        req.validate()?;

        // Model config generation: emit a minimal but well-formed machine
        // config and a matching talosconfig. Real Talos calls the
        // `pkg/machinery/config/generate` bundle here.
        let machine_config = format!(
            "version: v1alpha1\nmachine:\n  type: {}\ncluster:\n  clusterName: {}\n  controlPlane:\n    endpoint: {}\n",
            req.machine_type, req.cluster_name, req.control_plane_endpoint
        )
        .into_bytes();

        let talosconfig = format!(
            "context: {}\ncontexts:\n  {}:\n    endpoints:\n      - {}\n",
            req.cluster_name,
            req.cluster_name,
            self.state.hostname()
        )
        .into_bytes();

        Ok(GenerateResponse {
            machine_config,
            talosconfig,
        })
    }

    fn apply_configuration(
        &mut self,
        input: &ApplyConfigInput,
    ) -> Result<ApplyConfigOutcome, MaintenanceError> {
        self.apply_and_leave(input, ConfigSource::Grpc)
    }

    fn upgrade(&mut self, req: &UpgradeRequest) -> Result<UpgradeResponse, MaintenanceError> {
        self.require_serving("Upgrade")?;
        req.validate()?;
        // Upgrading the maintenance image always reboots into the new image
        // unless staged.
        let reboot = !req.stage;
        if reboot {
            self.rebooter
                .reboot()
                .map_err(|e| MaintenanceError::Apply(format!("reboot: {e}")))?;
            self.state
                .transition_to(MaintenancePhase::LeavingMaintenance)?;
        }
        Ok(UpgradeResponse {
            image: req.image.clone(),
            reboot,
        })
    }

    fn reset(&mut self, req: &ResetRequest) -> Result<(), MaintenanceError> {
        self.require_serving("Reset")?;
        self.state.transition_to(MaintenancePhase::Reset)?;
        if req.reboot {
            self.rebooter
                .reboot()
                .map_err(|e| MaintenanceError::Apply(format!("reboot: {e}")))?;
        }
        Ok(())
    }
}

/// A convenience alias for the fully in-memory server used by tests/offline
/// builds.
pub type DefaultMaintenanceServer = MaintenanceServer<
    InMemoryCertBootstrap,
    DefaultConfigValidator,
    InMemoryConfigSink,
    RecordingInstaller,
    RecordingRebooter,
    os_kernel::ManualClock,
>;

// --------------------------------------------------------------------------
// HTTP config endpoint (dashboard / config acquisition)
// --------------------------------------------------------------------------

/// HTTP methods the maintenance config endpoint understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// `GET` — read the maintenance status / config URL.
    Get,
    /// `POST` — push a machine config.
    Post,
}

/// A minimal HTTP request to the maintenance endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    /// The HTTP method.
    pub method: HttpMethod,
    /// The request path (e.g. `/config`).
    pub path: String,
    /// The request body (config bytes for a POST).
    pub body: Vec<u8>,
}

/// A minimal HTTP response from the maintenance endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// The HTTP status code.
    pub status: u16,
    /// The response body.
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Build a textual response with the given status.
    pub fn text(status: u16, body: impl Into<Vec<u8>>) -> Self {
        HttpResponse {
            status,
            body: body.into(),
        }
    }

    /// Whether the response is 2xx.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// The HTTP config-acquisition endpoint served alongside the gRPC API.
///
/// Talos serves a small HTTP server in maintenance mode so an operator (or the
/// dashboard) can `GET /` to discover the node is in maintenance and `POST
/// /config` to push a machine config. This struct adapts those HTTP calls onto
/// the [`MaintenanceServer`].
pub struct HttpConfigEndpoint;

impl HttpConfigEndpoint {
    /// Handle an HTTP request against a maintenance server.
    pub fn handle<C, V, S, I, R, K>(
        server: &mut MaintenanceServer<C, V, S, I, R, K>,
        req: &HttpRequest,
    ) -> HttpResponse
    where
        C: CertBootstrap,
        V: ConfigValidator,
        S: ConfigSink,
        I: Installer,
        R: Rebooter,
        K: Clock,
    {
        match (req.method, req.path.as_str()) {
            (HttpMethod::Get, "/") | (HttpMethod::Get, "/healthz") => HttpResponse::text(
                200,
                format!(
                    "talos {} in maintenance mode on {}",
                    server.state().version(),
                    server.state().hostname()
                ),
            ),
            (HttpMethod::Post, "/config") => {
                if req.body.is_empty() {
                    return HttpResponse::text(400, "empty config body");
                }
                let input = ApplyConfigInput::reboot(req.body.clone());
                match server.apply_and_leave(&input, ConfigSource::Http) {
                    Ok(outcome) => {
                        let msg = if outcome.install {
                            "config accepted; installing and rebooting"
                        } else {
                            "config accepted; rebooting"
                        };
                        HttpResponse::text(200, msg)
                    }
                    Err(e) => HttpResponse::text(422, format!("config rejected: {e}")),
                }
            }
            (HttpMethod::Post, _) => HttpResponse::text(404, "not found"),
            (HttpMethod::Get, _) => HttpResponse::text(404, "not found"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::ManualClock;

    const GOOD_CONFIG: &str = "version: v1alpha1\nmachine:\n  type: controlplane\n";
    const INSTALL_CONFIG: &str =
        "version: v1alpha1\nmachine:\n  type: worker\n  install:\n    disk: /dev/sda\n";

    fn server() -> DefaultMaintenanceServer {
        let mut state = MaintenanceState::new("node-1", "v1.7.0");
        state.add_address("10.0.0.5");
        MaintenanceServer::new(
            state,
            InMemoryCertBootstrap::new(),
            DefaultConfigValidator::new(),
            InMemoryConfigSink::new(),
            RecordingInstaller::new(),
            RecordingRebooter::default(),
            ManualClock::new(1_000 * 1_000_000_000),
        )
    }

    #[test]
    fn bootstrap_generates_cert_covering_node_and_serves() {
        let mut s = server();
        let tls = s.bootstrap().unwrap().clone();
        assert!(tls.certificate.self_signed);
        assert!(tls.certificate.sans.covers("node-1"));
        assert!(tls.certificate.sans.covers("10.0.0.5"));
        assert!(tls.certificate.sans.covers("localhost"));
        assert!(!tls.require_client_cert);
        assert_eq!(tls.certificate.not_before, 1_000);
        assert_eq!(s.phase(), MaintenancePhase::Serving);
    }

    #[test]
    fn methods_blocked_before_serving() {
        let mut s = server();
        // Not yet bootstrapped -> not serving.
        let err = s
            .apply_configuration(&ApplyConfigInput::reboot(GOOD_CONFIG.as_bytes()))
            .unwrap_err();
        assert!(matches!(err, MaintenanceError::WrongPhase { .. }));
    }

    #[test]
    fn apply_config_persists_and_reboots_leaving_maintenance() {
        let mut s = server();
        s.bootstrap().unwrap();
        let out = s
            .apply_configuration(&ApplyConfigInput::reboot(GOOD_CONFIG.as_bytes()))
            .unwrap();
        assert!(out.reboot);
        assert!(!out.install);
        assert_eq!(s.sink().persist_count(), 1);
        assert_eq!(s.rebooter().reboots, 1);
        assert!(s.installer().requests.is_empty());
        assert_eq!(s.phase(), MaintenancePhase::LeavingMaintenance);
    }

    #[test]
    fn install_from_maintenance_triggers_installer_then_reboot() {
        let mut s = server();
        s.bootstrap().unwrap();
        let out = s
            .apply_configuration(&ApplyConfigInput::reboot(INSTALL_CONFIG.as_bytes()))
            .unwrap();
        assert!(out.install);
        assert_eq!(s.installer().requests.len(), 1);
        assert_eq!(s.installer().requests[0].disk.as_deref(), Some("/dev/sda"));
        assert_eq!(s.rebooter().reboots, 1);
        assert_eq!(s.phase(), MaintenancePhase::LeavingMaintenance);
    }

    #[test]
    fn install_failure_aborts_before_reboot() {
        let mut state = MaintenanceState::new("node-1", "v1.7.0");
        state.add_address("10.0.0.5");
        let mut s: MaintenanceServer<_, _, _, _, _, _> = MaintenanceServer::new(
            state,
            InMemoryCertBootstrap::new(),
            DefaultConfigValidator::new(),
            InMemoryConfigSink::new(),
            RecordingInstaller::failing(),
            RecordingRebooter::default(),
            ManualClock::new(0),
        );
        s.bootstrap().unwrap();
        let err = s
            .apply_configuration(&ApplyConfigInput::reboot(INSTALL_CONFIG.as_bytes()))
            .unwrap_err();
        assert!(matches!(err, MaintenanceError::Apply(_)));
        // Persisted, moved to ConfigApplied, but did NOT reboot.
        assert_eq!(s.rebooter().reboots, 0);
        assert_eq!(s.phase(), MaintenancePhase::ConfigApplied);
    }

    #[test]
    fn invalid_config_rejected_and_stays_serving() {
        let mut s = server();
        s.bootstrap().unwrap();
        let err = s
            .apply_configuration(&ApplyConfigInput::reboot(b"garbage: true\n".to_vec()))
            .unwrap_err();
        assert!(matches!(err, MaintenanceError::Apply(_)));
        assert_eq!(s.phase(), MaintenancePhase::Serving);
        assert_eq!(s.sink().persist_count(), 0);
    }

    #[test]
    fn generate_configuration_emits_config_and_talosconfig() {
        let mut s = server();
        s.bootstrap().unwrap();
        let req = GenerateRequest {
            cluster_name: "demo".into(),
            control_plane_endpoint: "https://10.0.0.1:6443".into(),
            machine_type: "controlplane".into(),
            kubernetes_version: Some("v1.30.0".into()),
        };
        let resp = s.generate_configuration(&req).unwrap();
        let cfg = String::from_utf8(resp.machine_config).unwrap();
        assert!(cfg.contains("version: v1alpha1"));
        assert!(cfg.contains("type: controlplane"));
        assert!(cfg.contains("clusterName: demo"));
        let tc = String::from_utf8(resp.talosconfig).unwrap();
        assert!(tc.contains("context: demo"));
    }

    #[test]
    fn upgrade_reboots_unless_staged() {
        let mut s = server();
        s.bootstrap().unwrap();
        let resp = s
            .upgrade(&UpgradeRequest {
                image: "ghcr.io/siderolabs/installer:v1.7.1".into(),
                preserve: true,
                stage: false,
            })
            .unwrap();
        assert!(resp.reboot);
        assert_eq!(s.rebooter().reboots, 1);
        assert_eq!(s.phase(), MaintenancePhase::LeavingMaintenance);
    }

    #[test]
    fn reset_wipes_and_transitions() {
        let mut s = server();
        s.bootstrap().unwrap();
        s.reset(&ResetRequest {
            wipe: true,
            reboot: true,
        })
        .unwrap();
        assert_eq!(s.phase(), MaintenancePhase::Reset);
        assert_eq!(s.rebooter().reboots, 1);
    }

    #[test]
    fn http_get_reports_maintenance_status() {
        let mut s = server();
        s.bootstrap().unwrap();
        let resp = HttpConfigEndpoint::handle(
            &mut s,
            &HttpRequest {
                method: HttpMethod::Get,
                path: "/".into(),
                body: Vec::new(),
            },
        );
        assert!(resp.is_success());
        let body = String::from_utf8(resp.body).unwrap();
        assert!(body.contains("maintenance mode"));
        assert!(body.contains("node-1"));
    }

    #[test]
    fn http_post_config_applies_and_leaves_maintenance() {
        let mut s = server();
        s.bootstrap().unwrap();
        let resp = HttpConfigEndpoint::handle(
            &mut s,
            &HttpRequest {
                method: HttpMethod::Post,
                path: "/config".into(),
                body: GOOD_CONFIG.as_bytes().to_vec(),
            },
        );
        assert!(resp.is_success());
        assert_eq!(s.phase(), MaintenancePhase::LeavingMaintenance);
        assert_eq!(s.sink().persist_count(), 1);
    }

    #[test]
    fn http_post_empty_config_is_rejected() {
        let mut s = server();
        s.bootstrap().unwrap();
        let resp = HttpConfigEndpoint::handle(
            &mut s,
            &HttpRequest {
                method: HttpMethod::Post,
                path: "/config".into(),
                body: Vec::new(),
            },
        );
        assert_eq!(resp.status, 400);
        assert_eq!(s.phase(), MaintenancePhase::Serving);
    }

    #[test]
    fn http_invalid_config_returns_422() {
        let mut s = server();
        s.bootstrap().unwrap();
        let resp = HttpConfigEndpoint::handle(
            &mut s,
            &HttpRequest {
                method: HttpMethod::Post,
                path: "/config".into(),
                body: b"nope".to_vec(),
            },
        );
        assert_eq!(resp.status, 422);
    }

    #[test]
    fn http_unknown_path_404() {
        let mut s = server();
        s.bootstrap().unwrap();
        let resp = HttpConfigEndpoint::handle(
            &mut s,
            &HttpRequest {
                method: HttpMethod::Get,
                path: "/nope".into(),
                body: Vec::new(),
            },
        );
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn method_availability_gate() {
        let s = server();
        assert!(s.is_method_available("/machine.MachineService/ApplyConfiguration"));
        assert!(!s.is_method_available("/machine.MachineService/Bootstrap"));
    }

    #[test]
    fn version_and_hostname_rpcs() {
        let s = server();
        assert_eq!(s.version().unwrap(), "v1.7.0");
        assert_eq!(s.hostname().unwrap(), "node-1");
    }
}
