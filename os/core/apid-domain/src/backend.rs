//! The backend abstraction apid dispatches to once a call is authorized.
//!
//! apid never executes machine logic itself: a local call is forwarded over a
//! unix socket to `machined`, and a fanned-out call is forwarded over mTLS to a
//! peer node's apid. Both are modeled here as the [`Backend`] trait. The
//! in-memory [`LocalBackend`] answers from a canned state table, and
//! [`RemoteBackend`] simulates a peer (including connectivity failures) so the
//! router's fan-out/error-aggregation logic can be tested without a network.

use crate::error::ApiError;
use crate::machine_service::MachineMethod;
use crate::request::{Method, Request};
use crate::resource_service::ResourceMethod;
use crate::response::Response;
use std::collections::BTreeMap;

/// A target apid can dispatch an authorized request to.
///
/// Implementations either serve the request (local machined, or a reachable
/// peer) or report a transport failure ([`ApiError::Unavailable`]).
pub trait Backend {
    /// The endpoint this backend represents (`"local"` or a peer host/IP).
    fn endpoint(&self) -> &str;

    /// Serve one unary request, producing a body or a transport/RPC error.
    fn serve(&self, req: &Request) -> Result<Response, ApiError>;

    /// Serve a streaming request, producing the ordered message bodies.
    ///
    /// The default collapses to a single message by delegating to [`serve`]; a
    /// real streaming backend overrides this.
    ///
    /// [`serve`]: Backend::serve
    fn serve_stream(&self, req: &Request) -> Result<Vec<Response>, ApiError> {
        Ok(vec![self.serve(req)?])
    }
}

/// An in-memory local backend modeling the on-node `machined` + COSI state.
///
/// Read methods are answered from a small key/value table; mutating lifecycle
/// methods (`Reboot`, `Reset`, ...) flip the corresponding flag and return an
/// acknowledgement, mirroring how machined records that an action was queued.
#[derive(Debug, Clone)]
pub struct LocalBackend {
    endpoint: String,
    /// Canned read values, keyed by method short name (e.g. `Version`).
    values: BTreeMap<String, String>,
    /// The Talos version string this node reports.
    version: String,
    /// The node hostname.
    hostname: String,
}

impl LocalBackend {
    /// Build a local backend reporting `version` and `hostname`.
    pub fn new(
        endpoint: impl Into<String>,
        version: impl Into<String>,
        hostname: impl Into<String>,
    ) -> Self {
        LocalBackend {
            endpoint: endpoint.into(),
            values: BTreeMap::new(),
            version: version.into(),
            hostname: hostname.into(),
        }
    }

    /// Seed a canned value for a COSI resource or arbitrary method body.
    pub fn set_value(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    fn serve_machine(&self, m: MachineMethod, _req: &Request) -> Response {
        match m {
            MachineMethod::Version => Response::ok(self.version.clone()),
            MachineMethod::Hostname | MachineMethod::EtcdMemberList => {
                Response::ok(self.hostname.clone())
            }
            MachineMethod::ServiceList => Response::ok("apid:Running,machined:Running"),
            // Mutating lifecycle calls are acknowledged.
            MachineMethod::Reboot
            | MachineMethod::Shutdown
            | MachineMethod::Reset
            | MachineMethod::Upgrade
            | MachineMethod::ApplyConfiguration
            | MachineMethod::Bootstrap => Response::ok(format!("{} accepted", m.short_name())),
            MachineMethod::Logs | MachineMethod::Dmesg => Self::serve_stream_machine(m).join(),
        }
    }

    fn serve_stream_machine(m: MachineMethod) -> crate::response::StreamResponse {
        use crate::response::StreamResponse;
        let mut s = StreamResponse::new();
        match m {
            MachineMethod::Logs => {
                s.push(Response::ok("machined: started"));
                s.push(Response::ok("apid: listening"));
            }
            MachineMethod::Dmesg => {
                s.push(Response::ok("[0.000000] Linux version"));
                s.push(Response::ok("[0.123456] kvm: enabled"));
            }
            _ => {}
        }
        s
    }

    fn serve_resource(&self, m: ResourceMethod, req: &Request) -> Result<Response, ApiError> {
        // Look up a body keyed by the request body (used as the resource id) or
        // fall through to a generic representation.
        let key = if req.body().is_empty() {
            m.short_name()
        } else {
            req.body()
        };
        match self.values.get(key) {
            Some(v) => Ok(Response::ok(v.clone())),
            None if m == ResourceMethod::Get => {
                Err(ApiError::NotFound(format!("resource '{key}' not found")))
            }
            None => Ok(Response::empty()),
        }
    }
}

impl Backend for LocalBackend {
    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn serve(&self, req: &Request) -> Result<Response, ApiError> {
        match req.method() {
            Method::Machine(m) => Ok(self.serve_machine(m, req)),
            Method::Resource(m) => self.serve_resource(m, req),
        }
    }

    fn serve_stream(&self, req: &Request) -> Result<Vec<Response>, ApiError> {
        match req.method() {
            Method::Machine(m @ (MachineMethod::Logs | MachineMethod::Dmesg)) => {
                Ok(Self::serve_stream_machine(m).messages().to_vec())
            }
            Method::Resource(ResourceMethod::List | ResourceMethod::Watch) => {
                // Emit each seeded value as a list item.
                let items: Vec<Response> = self
                    .values
                    .values()
                    .map(|v| Response::ok(v.clone()))
                    .collect();
                Ok(items)
            }
            _ => Ok(vec![self.serve(req)?]),
        }
    }
}

/// A simulated peer node's apid, reachable over the network.
///
/// `reachable == false` models a partitioned or down peer: every call returns
/// [`ApiError::Unavailable`], which the router records as a per-node error
/// without failing the whole fan-out.
#[derive(Debug, Clone)]
pub struct RemoteBackend {
    endpoint: String,
    reachable: bool,
    /// The version this peer reports (so fan-out responses differ per node).
    version: String,
}

impl RemoteBackend {
    /// A reachable peer reporting `version`.
    pub fn new(endpoint: impl Into<String>, version: impl Into<String>) -> Self {
        RemoteBackend {
            endpoint: endpoint.into(),
            reachable: true,
            version: version.into(),
        }
    }

    /// An unreachable peer (every call fails with `Unavailable`).
    pub fn unreachable(endpoint: impl Into<String>) -> Self {
        RemoteBackend {
            endpoint: endpoint.into(),
            reachable: false,
            version: String::new(),
        }
    }

    /// Mark the peer reachable or not.
    pub fn set_reachable(&mut self, reachable: bool) {
        self.reachable = reachable;
    }
}

impl Backend for RemoteBackend {
    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn serve(&self, req: &Request) -> Result<Response, ApiError> {
        if !self.reachable {
            return Err(ApiError::unavailable(format!(
                "peer {} is unreachable",
                self.endpoint
            )));
        }
        match req.method() {
            Method::Machine(MachineMethod::Version) => Ok(Response::ok(self.version.clone())),
            Method::Machine(m) => Ok(Response::ok(format!(
                "{}@{}",
                m.short_name(),
                self.endpoint
            ))),
            Method::Resource(m) => Ok(Response::ok(format!(
                "{}@{}",
                m.short_name(),
                self.endpoint
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_serves_read_methods() {
        let be = LocalBackend::new("local", "v1.7.0", "cp-1");
        let req = Request::machine(MachineMethod::Version);
        assert_eq!(be.serve(&req).unwrap().body(), "v1.7.0");
        let req = Request::machine(MachineMethod::Hostname);
        assert_eq!(be.serve(&req).unwrap().body(), "cp-1");
    }

    #[test]
    fn local_acknowledges_mutations() {
        let be = LocalBackend::new("local", "v1.7.0", "cp-1");
        let req = Request::machine(MachineMethod::Reboot);
        assert_eq!(be.serve(&req).unwrap().body(), "Reboot accepted");
    }

    #[test]
    fn local_streams_logs() {
        let be = LocalBackend::new("local", "v1.7.0", "cp-1");
        let req = Request::machine(MachineMethod::Logs);
        let msgs = be.serve_stream(&req).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].body(), "machined: started");
    }

    #[test]
    fn local_resource_get_missing_is_not_found() {
        let mut be = LocalBackend::new("local", "v1.7.0", "cp-1");
        be.set_value("MachineStatus", "running");
        let hit = Request::resource(ResourceMethod::Get).with_body("MachineStatus");
        assert_eq!(be.serve(&hit).unwrap().body(), "running");
        let miss = Request::resource(ResourceMethod::Get).with_body("Absent");
        assert_eq!(be.serve(&miss).unwrap_err().grpc_code(), "NotFound");
    }

    #[test]
    fn remote_unreachable_fails() {
        let be = RemoteBackend::unreachable("10.0.0.9");
        let req = Request::machine(MachineMethod::Version);
        assert_eq!(be.serve(&req).unwrap_err().grpc_code(), "Unavailable");
    }

    #[test]
    fn remote_reports_its_version() {
        let be = RemoteBackend::new("10.0.0.2", "v1.6.0");
        let req = Request::machine(MachineMethod::Version);
        assert_eq!(be.serve(&req).unwrap().body(), "v1.6.0");
    }
}
