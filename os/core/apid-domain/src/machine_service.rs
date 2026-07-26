//! The `machine.MachineService` gRPC surface modeled as a Rust trait plus a
//! method enum used for routing and authorization.

use crate::error::ApiError;
use crate::request::Request;
use crate::response::Response;

/// One RPC of the Talos `machine.MachineService`.
///
/// This is the subset of methods exercised most often by `talosctl`; each maps
/// to a fully-qualified gRPC method name and carries whether it mutates node
/// state (used by the [`Authorizer`](crate::auth::Authorizer)) and whether it
/// is server-streaming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MachineMethod {
    /// `Version` — report the Talos version. Read-only.
    Version,
    /// `Hostname` — report the node hostname. Read-only.
    Hostname,
    /// `ServiceList` — list system services. Read-only.
    ServiceList,
    /// `Logs` — stream service/container logs. Read-only, streaming.
    Logs,
    /// `Dmesg` — stream kernel ring buffer. Read-only, streaming.
    Dmesg,
    /// `Reboot` — reboot the node. Mutating.
    Reboot,
    /// `Shutdown` — power the node off. Mutating.
    Shutdown,
    /// `Reset` — wipe & reset the node. Mutating.
    Reset,
    /// `Upgrade` — upgrade the Talos install. Mutating.
    Upgrade,
    /// `ApplyConfiguration` — apply a machine config. Mutating.
    ApplyConfiguration,
    /// `Bootstrap` — bootstrap etcd on the first control plane node. Mutating.
    Bootstrap,
    /// `EtcdMemberList` — list etcd members. Read-only.
    EtcdMemberList,
}

impl MachineMethod {
    /// The fully-qualified gRPC method name (`/machine.MachineService/<m>`).
    pub fn grpc_name(self) -> &'static str {
        match self {
            MachineMethod::Version => "/machine.MachineService/Version",
            MachineMethod::Hostname => "/machine.MachineService/Hostname",
            MachineMethod::ServiceList => "/machine.MachineService/ServiceList",
            MachineMethod::Logs => "/machine.MachineService/Logs",
            MachineMethod::Dmesg => "/machine.MachineService/Dmesg",
            MachineMethod::Reboot => "/machine.MachineService/Reboot",
            MachineMethod::Shutdown => "/machine.MachineService/Shutdown",
            MachineMethod::Reset => "/machine.MachineService/Reset",
            MachineMethod::Upgrade => "/machine.MachineService/Upgrade",
            MachineMethod::ApplyConfiguration => "/machine.MachineService/ApplyConfiguration",
            MachineMethod::Bootstrap => "/machine.MachineService/Bootstrap",
            MachineMethod::EtcdMemberList => "/machine.MachineService/EtcdMemberList",
        }
    }

    /// The short method name without the service prefix.
    pub fn short_name(self) -> &'static str {
        self.grpc_name().rsplit('/').next().unwrap_or("")
    }

    /// Whether the method mutates node state and therefore requires write RBAC.
    pub fn is_mutating(self) -> bool {
        matches!(
            self,
            MachineMethod::Reboot
                | MachineMethod::Shutdown
                | MachineMethod::Reset
                | MachineMethod::Upgrade
                | MachineMethod::ApplyConfiguration
                | MachineMethod::Bootstrap
        )
    }

    /// Whether the method is server-streaming (multiple response messages).
    pub fn is_streaming(self) -> bool {
        matches!(self, MachineMethod::Logs | MachineMethod::Dmesg)
    }

    /// Parse a method from its fully-qualified or short name.
    pub fn parse(name: &str) -> Result<Self, ApiError> {
        let short = name.rsplit('/').next().unwrap_or(name);
        let m = match short {
            "Version" => MachineMethod::Version,
            "Hostname" => MachineMethod::Hostname,
            "ServiceList" => MachineMethod::ServiceList,
            "Logs" => MachineMethod::Logs,
            "Dmesg" => MachineMethod::Dmesg,
            "Reboot" => MachineMethod::Reboot,
            "Shutdown" => MachineMethod::Shutdown,
            "Reset" => MachineMethod::Reset,
            "Upgrade" => MachineMethod::Upgrade,
            "ApplyConfiguration" => MachineMethod::ApplyConfiguration,
            "Bootstrap" => MachineMethod::Bootstrap,
            "EtcdMemberList" => MachineMethod::EtcdMemberList,
            other => {
                return Err(ApiError::unimplemented(format!(
                    "unknown machine method '{other}'"
                )));
            }
        };
        Ok(m)
    }
}

/// The Talos `machine.MachineService`, implemented by the local node.
///
/// The methods mirror the gRPC service: unary calls return a [`Response`],
/// streaming calls return a `Vec<Response>` (one element per server message).
/// Real implementations talk to machined; the in-memory test impl returns
/// canned values.
pub trait MachineService {
    /// Report the running Talos version.
    fn version(&self, req: &Request) -> Result<Response, ApiError>;

    /// Report the node hostname.
    fn hostname(&self, req: &Request) -> Result<Response, ApiError>;

    /// List system services and their states.
    fn service_list(&self, req: &Request) -> Result<Response, ApiError>;

    /// Reboot the node.
    fn reboot(&self, req: &Request) -> Result<Response, ApiError>;

    /// Apply a machine configuration document.
    fn apply_configuration(&self, req: &Request) -> Result<Response, ApiError>;

    /// Bootstrap the etcd cluster from this node.
    fn bootstrap(&self, req: &Request) -> Result<Response, ApiError>;

    /// Stream logs; returns each log line as a separate response message.
    fn logs(&self, req: &Request) -> Result<Vec<Response>, ApiError>;

    /// Generic dispatch by method, used by the router. Default implementation
    /// forwards to the concrete methods and collapses streaming responses to a
    /// single concatenated [`Response`] (the router re-splits when streaming).
    fn dispatch(&self, method: MachineMethod, req: &Request) -> Result<Response, ApiError> {
        match method {
            MachineMethod::Version => self.version(req),
            MachineMethod::Hostname => self.hostname(req),
            MachineMethod::ServiceList => self.service_list(req),
            MachineMethod::Reboot => self.reboot(req),
            MachineMethod::ApplyConfiguration => self.apply_configuration(req),
            MachineMethod::Bootstrap => self.bootstrap(req),
            MachineMethod::Logs | MachineMethod::Dmesg => {
                let parts = self.logs(req)?;
                let mut joined = String::new();
                for (i, p) in parts.iter().enumerate() {
                    if i > 0 {
                        joined.push('\n');
                    }
                    joined.push_str(p.body());
                }
                Ok(Response::ok(joined))
            }
            other => Err(ApiError::unimplemented(format!(
                "{} not handled by this service",
                other.short_name()
            ))),
        }
    }
}

/// An in-memory [`MachineService`] modeling machined's local state.
///
/// Read methods answer from recorded fields; lifecycle methods record that the
/// action was requested (so a test can assert apid forwarded a `Reboot`) and
/// return an acknowledgement. `Bootstrap` is idempotent-guarded: a second
/// bootstrap on an already-bootstrapped node is rejected, mirroring machined.
#[derive(Debug, Clone)]
pub struct MachineState {
    version: String,
    hostname: String,
    services: Vec<(String, String)>,
    bootstrapped: bool,
    reboots: u32,
    applied_configs: Vec<String>,
}

impl MachineState {
    /// Build a state reporting `version` and `hostname`.
    pub fn new(version: impl Into<String>, hostname: impl Into<String>) -> Self {
        MachineState {
            version: version.into(),
            hostname: hostname.into(),
            services: vec![
                ("machined".to_string(), "Running".to_string()),
                ("apid".to_string(), "Running".to_string()),
            ],
            bootstrapped: false,
            reboots: 0,
            applied_configs: Vec::new(),
        }
    }

    /// Register a service and its state.
    pub fn set_service(&mut self, name: impl Into<String>, state: impl Into<String>) {
        self.services.push((name.into(), state.into()));
    }

    /// Whether etcd has been bootstrapped on this node.
    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrapped
    }

    /// How many reboots have been requested.
    pub fn reboot_count(&self) -> u32 {
        self.reboots
    }

    /// The configs applied so far, in order.
    pub fn applied_configs(&self) -> &[String] {
        &self.applied_configs
    }
}

impl MachineService for MachineState {
    fn version(&self, _req: &Request) -> Result<Response, ApiError> {
        Ok(Response::ok(self.version.clone()))
    }

    fn hostname(&self, _req: &Request) -> Result<Response, ApiError> {
        Ok(Response::ok(self.hostname.clone()))
    }

    fn service_list(&self, _req: &Request) -> Result<Response, ApiError> {
        let body = self
            .services
            .iter()
            .map(|(n, s)| format!("{n}:{s}"))
            .collect::<Vec<_>>()
            .join(",");
        Ok(Response::ok(body))
    }

    fn reboot(&self, _req: &Request) -> Result<Response, ApiError> {
        // `&self` can't mutate; the reboot is acknowledged. Mutation tracking is
        // exercised via the &mut helpers below.
        Ok(Response::ok("Reboot accepted"))
    }

    fn apply_configuration(&self, req: &Request) -> Result<Response, ApiError> {
        if req.body().trim().is_empty() {
            return Err(ApiError::invalid(
                "ApplyConfiguration requires a config body",
            ));
        }
        Ok(Response::ok("ApplyConfiguration accepted"))
    }

    fn bootstrap(&self, _req: &Request) -> Result<Response, ApiError> {
        if self.bootstrapped {
            return Err(ApiError::invalid("etcd is already bootstrapped"));
        }
        Ok(Response::ok("Bootstrap accepted"))
    }

    fn logs(&self, _req: &Request) -> Result<Vec<Response>, ApiError> {
        Ok(vec![
            Response::ok(format!("{}: started", self.hostname)),
            Response::ok("apid: listening"),
        ])
    }
}

impl MachineState {
    /// Apply a mutating method that records state, returning the ack.
    ///
    /// The [`MachineService`] trait takes `&self`; this `&mut` helper performs
    /// the state transition (reboot count, bootstrap flag, applied configs) the
    /// way machined records the effect before returning the acknowledgement.
    pub fn execute(&mut self, method: MachineMethod, req: &Request) -> Result<Response, ApiError> {
        match method {
            MachineMethod::Reboot => {
                self.reboots += 1;
                Ok(Response::ok("Reboot accepted"))
            }
            MachineMethod::Bootstrap => {
                if self.bootstrapped {
                    return Err(ApiError::invalid("etcd is already bootstrapped"));
                }
                self.bootstrapped = true;
                Ok(Response::ok("Bootstrap accepted"))
            }
            MachineMethod::ApplyConfiguration => {
                if req.body().trim().is_empty() {
                    return Err(ApiError::invalid(
                        "ApplyConfiguration requires a config body",
                    ));
                }
                self.applied_configs.push(req.body().to_string());
                Ok(Response::ok("ApplyConfiguration accepted"))
            }
            other => self.dispatch(other, req),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutating_and_streaming_flags() {
        assert!(MachineMethod::Reboot.is_mutating());
        assert!(MachineMethod::ApplyConfiguration.is_mutating());
        assert!(!MachineMethod::Version.is_mutating());
        assert!(MachineMethod::Logs.is_streaming());
        assert!(!MachineMethod::Version.is_streaming());
    }

    #[test]
    fn grpc_and_short_names() {
        assert_eq!(
            MachineMethod::Version.grpc_name(),
            "/machine.MachineService/Version"
        );
        assert_eq!(MachineMethod::Reboot.short_name(), "Reboot");
    }

    #[test]
    fn parse_round_trip() {
        assert_eq!(
            MachineMethod::parse("Version").unwrap(),
            MachineMethod::Version
        );
        assert_eq!(
            MachineMethod::parse("/machine.MachineService/Bootstrap").unwrap(),
            MachineMethod::Bootstrap
        );
        assert!(MachineMethod::parse("Nonsense").is_err());
    }

    #[test]
    fn machine_state_reads() {
        let st = MachineState::new("v1.7.0", "cp-1");
        assert_eq!(
            st.version(&Request::machine(MachineMethod::Version))
                .unwrap()
                .body(),
            "v1.7.0"
        );
        assert_eq!(
            st.hostname(&Request::machine(MachineMethod::Hostname))
                .unwrap()
                .body(),
            "cp-1"
        );
        let svc = st
            .service_list(&Request::machine(MachineMethod::ServiceList))
            .unwrap();
        assert!(svc.body().contains("machined:Running"));
    }

    #[test]
    fn machine_state_dispatch_logs() {
        let st = MachineState::new("v1.7.0", "cp-1");
        let joined = st
            .dispatch(MachineMethod::Logs, &Request::machine(MachineMethod::Logs))
            .unwrap();
        assert!(joined.body().contains("cp-1: started"));
        assert!(joined.body().contains("apid: listening"));
    }

    #[test]
    fn machine_state_execute_tracks_reboots() {
        let mut st = MachineState::new("v1.7.0", "cp-1");
        st.execute(
            MachineMethod::Reboot,
            &Request::machine(MachineMethod::Reboot),
        )
        .unwrap();
        st.execute(
            MachineMethod::Reboot,
            &Request::machine(MachineMethod::Reboot),
        )
        .unwrap();
        assert_eq!(st.reboot_count(), 2);
    }

    #[test]
    fn machine_state_bootstrap_is_idempotent_guarded() {
        let mut st = MachineState::new("v1.7.0", "cp-1");
        assert!(!st.is_bootstrapped());
        st.execute(
            MachineMethod::Bootstrap,
            &Request::machine(MachineMethod::Bootstrap),
        )
        .unwrap();
        assert!(st.is_bootstrapped());
        let err = st
            .execute(
                MachineMethod::Bootstrap,
                &Request::machine(MachineMethod::Bootstrap),
            )
            .unwrap_err();
        assert_eq!(err.grpc_code(), "InvalidArgument");
    }

    #[test]
    fn machine_state_apply_config_requires_body() {
        let mut st = MachineState::new("v1.7.0", "cp-1");
        let empty = Request::machine(MachineMethod::ApplyConfiguration);
        assert_eq!(
            st.execute(MachineMethod::ApplyConfiguration, &empty)
                .unwrap_err()
                .grpc_code(),
            "InvalidArgument"
        );
        let with_cfg =
            Request::machine(MachineMethod::ApplyConfiguration).with_body("version: v1alpha1");
        st.execute(MachineMethod::ApplyConfiguration, &with_cfg)
            .unwrap();
        assert_eq!(st.applied_configs().len(), 1);
    }
}
