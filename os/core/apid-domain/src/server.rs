//! A runnable Unix-domain-socket server for the machine API.
//!
//! Real `apid` terminates mTLS and speaks gRPC; this module provides the
//! concrete, dependency-free analog: a [`Server`] that binds a
//! `std::os::unix::net::UnixListener`, accepts connections (one OS thread per
//! connection), reads length-prefixed [`Request`](os_proto_api::Request)
//! frames using the [`os_proto_api`] wire codec, dispatches them to a
//! [`Backend`], and writes the framed [`Response`](os_proto_api::Response).
//!
//! The wire format and framing are owned entirely by [`os_proto_api`]; this
//! module only wires the socket lifecycle, threading and request dispatch.
//!
//! # Client flow
//!
//! ```no_run
//! use std::os::unix::net::UnixStream;
//! use os_proto_api::{write_message, read_message, Request, Response};
//!
//! let mut stream = UnixStream::connect("/run/apid.sock").unwrap();
//! write_message(&mut stream, &Request::Version).unwrap();
//! let resp: Response = read_message(&mut stream).unwrap();
//! match resp {
//!     Response::Version(v) => println!("version = {}", v.tag),
//!     _ => {}
//! }
//! ```

use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use os_proto_api::{
    Request, Response, ServiceEntry, VersionReply, WireErrorReply, WireRebootMode,
    read_message_opt, write_message,
};

/// Description of a single service, returned by [`Backend::list_services`].
///
/// A minimal, codec-independent view that the server maps onto the wire
/// [`ServiceEntry`] before sending it back to the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceInfo {
    /// The service id (e.g. `etcd`, `apid`).
    pub id: String,
    /// The lifecycle state (e.g. `Running`, `Stopped`).
    pub state: String,
    /// The health, if known: `Some(true)` healthy, `Some(false)` unhealthy,
    /// `None` if health is not reported.
    pub healthy: Option<bool>,
}

impl ServiceInfo {
    /// Build a [`ServiceInfo`].
    pub fn new(id: impl Into<String>, state: impl Into<String>, healthy: Option<bool>) -> Self {
        ServiceInfo {
            id: id.into(),
            state: state.into(),
            healthy,
        }
    }
}

/// The node-side logic the [`Server`] dispatches authorized requests to.
///
/// Mirrors the subset of `machined`'s machine API exercised by this transport.
/// Implementations are shared across connection threads, so the trait requires
/// `Send + Sync`. A method body is what apid would forward to `machined` over
/// the local unix socket.
pub trait Backend: Send + Sync {
    /// The node's Talos version string (e.g. `v1.7.0`).
    fn version(&self) -> String;

    /// The node's hostname.
    fn hostname(&self) -> String;

    /// The currently-known services.
    fn list_services(&self) -> Vec<ServiceInfo>;

    /// The kernel ring buffer, one entry per line.
    fn dmesg(&self) -> Vec<String>;

    /// Start a named service. Returns a human-readable acknowledgement.
    ///
    /// The default records nothing and simply acknowledges; override to model
    /// real service control.
    fn service_start(&self, name: &str) -> String {
        format!("{name} started")
    }

    /// Queue a reboot in the given mode. Returns a monotonic acknowledgement id.
    fn reboot(&self, mode: WireRebootMode);
}

/// An in-memory [`Backend`] for tests and local runs.
///
/// All state is interior-mutable behind a [`Mutex`] so it can be inspected from
/// the test thread while connection threads serve requests. Records every
/// `service_start` and `reboot` so tests can assert on side effects.
#[derive(Debug, Default)]
pub struct FakeBackend {
    inner: Mutex<FakeState>,
}

#[derive(Debug)]
struct FakeState {
    version: String,
    hostname: String,
    services: Vec<ServiceInfo>,
    dmesg: Vec<String>,
    started: Vec<String>,
    reboots: Vec<WireRebootMode>,
}

impl Default for FakeState {
    fn default() -> Self {
        FakeState {
            version: "v1.7.0".to_string(),
            hostname: "operating-system-node".to_string(),
            services: vec![
                ServiceInfo::new("machined", "Running", Some(true)),
                ServiceInfo::new("apid", "Running", Some(true)),
            ],
            dmesg: vec![
                "[0.000000] Linux version".to_string(),
                "[0.123456] kvm: enabled".to_string(),
            ],
            started: Vec::new(),
            reboots: Vec::new(),
        }
    }
}

impl FakeBackend {
    /// A backend seeded with sensible defaults.
    pub fn new() -> Self {
        FakeBackend::default()
    }

    /// Override the reported version.
    pub fn with_version(self, version: impl Into<String>) -> Self {
        self.inner.lock().unwrap().version = version.into();
        self
    }

    /// Override the reported hostname.
    pub fn with_hostname(self, hostname: impl Into<String>) -> Self {
        self.inner.lock().unwrap().hostname = hostname.into();
        self
    }

    /// Replace the service table.
    pub fn with_services(self, services: Vec<ServiceInfo>) -> Self {
        self.inner.lock().unwrap().services = services;
        self
    }

    /// Replace the dmesg buffer.
    pub fn with_dmesg(self, lines: Vec<String>) -> Self {
        self.inner.lock().unwrap().dmesg = lines;
        self
    }

    /// The services that were started, in call order.
    pub fn started(&self) -> Vec<String> {
        self.inner.lock().unwrap().started.clone()
    }

    /// The reboots that were requested, in call order.
    pub fn reboots(&self) -> Vec<WireRebootMode> {
        self.inner.lock().unwrap().reboots.clone()
    }
}

impl Backend for FakeBackend {
    fn version(&self) -> String {
        self.inner.lock().unwrap().version.clone()
    }

    fn hostname(&self) -> String {
        self.inner.lock().unwrap().hostname.clone()
    }

    fn list_services(&self) -> Vec<ServiceInfo> {
        self.inner.lock().unwrap().services.clone()
    }

    fn dmesg(&self) -> Vec<String> {
        self.inner.lock().unwrap().dmesg.clone()
    }

    fn service_start(&self, name: &str) -> String {
        self.inner.lock().unwrap().started.push(name.to_string());
        format!("{name} started")
    }

    fn reboot(&self, mode: WireRebootMode) {
        self.inner.lock().unwrap().reboots.push(mode);
    }
}

/// Translate a [`Request`] into a [`Response`] using the backend.
///
/// Pure and panic-free: every request maps to a `Response`, so a connection
/// thread never crashes on input. Exposed for direct unit testing of dispatch
/// without a socket.
pub fn dispatch<B: Backend + ?Sized>(backend: &B, req: Request) -> Response {
    match req {
        Request::Version => {
            let tag = backend.version();
            Response::Version(VersionReply {
                tag,
                sha: String::new(),
                arch: std::env::consts::ARCH.to_string(),
            })
        }
        Request::Hostname => Response::Hostname {
            hostname: backend.hostname(),
        },
        Request::ServiceList => {
            let services = backend
                .list_services()
                .into_iter()
                .map(|s| ServiceEntry {
                    id: s.id,
                    state: s.state,
                    healthy: s.healthy,
                })
                .collect();
            Response::ServiceList { services }
        }
        Request::ServiceStart { name } => Response::ServiceStart {
            resp: backend.service_start(&name),
        },
        Request::Dmesg => {
            let data = backend.dmesg().join("\n").into_bytes();
            Response::Dmesg { data }
        }
        Request::Reboot { mode } => {
            backend.reboot(mode);
            Response::Reboot { ack: 1 }
        }
    }
}

/// A handle to a running [`Server`], used to stop it and join its accept loop.
///
/// Dropping the handle does **not** stop the server; call [`shutdown`] (or
/// [`stop`] for a fire-and-forget signal) explicitly.
///
/// [`shutdown`]: ServerHandle::shutdown
/// [`stop`]: ServerHandle::stop
#[derive(Debug)]
pub struct ServerHandle {
    shutdown: Arc<AtomicBool>,
    path: PathBuf,
    accept: Option<JoinHandle<()>>,
}

impl ServerHandle {
    /// The socket path the server is bound to.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Signal the accept loop to stop without waiting for it.
    ///
    /// Wakes the blocked `accept()` by self-connecting once, then returns.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        // Unblock the listener's accept() with a throwaway connection.
        let _ = UnixStream::connect(&self.path);
    }

    /// Signal shutdown, wait for the accept loop to finish, and remove the
    /// socket file. Idempotent.
    pub fn shutdown(mut self) {
        self.stop();
        if let Some(handle) = self.accept.take() {
            let _ = handle.join();
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

/// A machine API server listening on a Unix domain socket.
///
/// Construct with [`Server::new`], then [`serve`](Server::serve) to take over
/// the calling thread or [`spawn`](Server::spawn) to run the accept loop on a
/// background thread and get a [`ServerHandle`] back.
pub struct Server<B: Backend + 'static> {
    backend: Arc<B>,
    path: PathBuf,
}

impl<B: Backend + 'static> Server<B> {
    /// Create a server that will bind `path` and dispatch to `backend`.
    pub fn new(path: impl Into<PathBuf>, backend: B) -> Self {
        Server {
            backend: Arc::new(backend),
            path: path.into(),
        }
    }

    /// Access the shared backend (e.g. to assert on its state after serving).
    pub fn backend(&self) -> Arc<B> {
        Arc::clone(&self.backend)
    }

    /// Bind the listener, removing any stale socket file first.
    fn bind(&self) -> io::Result<UnixListener> {
        // A leftover socket file from a crashed run would make bind() fail with
        // AddrInUse, so clear it. (Safe: we own this path.)
        let _ = std::fs::remove_file(&self.path);
        let listener = UnixListener::bind(&self.path)?;
        Ok(listener)
    }

    /// Run the accept loop on a background thread, returning a handle to stop it.
    ///
    /// Binds synchronously so a bind error surfaces to the caller immediately.
    pub fn spawn(self) -> io::Result<ServerHandle> {
        let listener = self.bind()?;
        // accept() must observe the shutdown flag promptly.
        listener.set_nonblocking(false)?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let path = self.path.clone();
        let backend = self.backend;
        let flag = Arc::clone(&shutdown);

        let accept = thread::Builder::new()
            .name("apid-accept".to_string())
            .spawn(move || accept_loop(listener, backend, flag))?;

        Ok(ServerHandle {
            shutdown,
            path,
            accept: Some(accept),
        })
    }

    /// Run the accept loop on the calling thread until `shutdown` is set.
    ///
    /// Shares the `shutdown` flag with the caller so another thread can stop it.
    pub fn serve(self, shutdown: Arc<AtomicBool>) -> io::Result<()> {
        let listener = self.bind()?;
        listener.set_nonblocking(false)?;
        accept_loop(listener, self.backend, shutdown);
        let _ = std::fs::remove_file(&self.path);
        Ok(())
    }
}

/// Accept connections until `shutdown` is observed, spawning a detached thread
/// per connection.
///
/// Connection threads are intentionally **not** joined here. A well-behaved
/// client keeps its stream open between requests, so a connection thread is
/// legitimately blocked in `read` until the client closes or shutdown is
/// requested; blocking the accept loop on `join` would deadlock shutdown. The
/// shutdown flag is checked on every connection thread's read, and the process
/// reclaims the threads on exit.
// Args are taken by value: this runs as the body of a spawned `move` closure
// (and the direct call consumes `self.backend`), so ownership is required.
#[allow(clippy::needless_pass_by_value)]
fn accept_loop<B: Backend + 'static>(
    listener: UnixListener,
    backend: Arc<B>,
    shutdown: Arc<AtomicBool>,
) {
    // Give blocked connection reads a chance to observe the shutdown flag.
    let _ = listener.set_nonblocking(false);
    for conn in listener.incoming() {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        match conn {
            Ok(stream) => {
                let backend = Arc::clone(&backend);
                let flag = Arc::clone(&shutdown);
                let _ = thread::Builder::new()
                    .name("apid-conn".to_string())
                    .spawn(move || handle_connection(stream, backend, flag));
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
    }
}

/// Serve every framed request on one connection until EOF, error, or shutdown.
///
/// A decode error (garbage/unknown request) is answered with a
/// [`Response::Error`] rather than dropping the connection or panicking. Other
/// I/O errors (a closed peer, etc.) end the connection.
///
/// To keep a server shutdown from being blocked by an idle-but-open client, the
/// read uses a poll timeout: a read that times out re-checks `shutdown` and
/// either loops or exits, so the thread is guaranteed to wind down once
/// shutdown is signalled even if the client never sends another byte.
// Args are taken by value: this runs as the body of a spawned `move` closure,
// so the connection thread must own its stream and shared handles.
#[allow(clippy::needless_pass_by_value)]
fn handle_connection<B: Backend + ?Sized>(
    mut stream: UnixStream,
    backend: Arc<B>,
    shutdown: Arc<AtomicBool>,
) {
    // Poll interval: short enough that shutdown is observed promptly, long
    // enough not to spin.
    let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
    loop {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        match read_message_opt::<_, Request>(&mut stream) {
            Ok(Some(req)) => {
                let resp = dispatch(&*backend, req);
                if write_message(&mut stream, &resp).is_err() {
                    break;
                }
            }
            // Clean EOF on a frame boundary: the client is done.
            Ok(None) => break,
            // Read timed out with no bytes pending: loop to re-check shutdown.
            Err(ref e) if is_timeout(e) => {}
            Err(err) => {
                // Garbage or an unknown tag inside a complete frame: reply with a
                // structured error so the client gets a response, not a hang.
                let reply = Response::Error(WireErrorReply {
                    code: 3, // InvalidArgument
                    message: err.to_string(),
                });
                let _ = write_message(&mut stream, &reply);
                break;
            }
        }
    }
}

/// Whether a wire error is an idle read timeout (WouldBlock/TimedOut), as
/// produced by a `set_read_timeout` poll with no data pending.
fn is_timeout(err: &os_proto_api::WireError) -> bool {
    matches!(
        err,
        os_proto_api::WireError::Io(e)
            if e.kind() == io::ErrorKind::WouldBlock
                || e.kind() == io::ErrorKind::TimedOut
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;
    use std::sync::atomic::AtomicU32;
    use std::time::{SystemTime, UNIX_EPOCH};

    use os_proto_api::{read_message, write_frame, write_message};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// A unique temp socket path for a test (sockets must live under `sun_path`'s
    /// length limit, so keep it short).
    fn temp_socket() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        std::env::temp_dir().join(format!("operating-system-apid-{pid}-{n}-{nanos}.sock"))
    }

    fn start() -> ServerHandle {
        let server = Server::new(temp_socket(), FakeBackend::new());
        server.spawn().expect("server spawns")
    }

    fn connect(handle: &ServerHandle) -> UnixStream {
        UnixStream::connect(handle.path()).expect("client connects")
    }

    #[test]
    fn version_roundtrip() {
        let handle = start();
        let mut c = connect(&handle);
        write_message(&mut c, &Request::Version).unwrap();
        let resp: Response = read_message(&mut c).unwrap();
        match resp {
            Response::Version(v) => assert_eq!(v.tag, "v1.7.0"),
            other => panic!("unexpected: {other:?}"),
        }
        handle.shutdown();
    }

    #[test]
    fn hostname_roundtrip() {
        let server = Server::new(temp_socket(), FakeBackend::new().with_hostname("cp-1"));
        let handle = server.spawn().unwrap();
        let mut c = connect(&handle);
        write_message(&mut c, &Request::Hostname).unwrap();
        match read_message::<_, Response>(&mut c).unwrap() {
            Response::Hostname { hostname } => assert_eq!(hostname, "cp-1"),
            other => panic!("unexpected: {other:?}"),
        }
        handle.shutdown();
    }

    #[test]
    fn service_list_roundtrip() {
        let handle = start();
        let mut c = connect(&handle);
        write_message(&mut c, &Request::ServiceList).unwrap();
        match read_message::<_, Response>(&mut c).unwrap() {
            Response::ServiceList { services } => {
                assert_eq!(services.len(), 2);
                assert_eq!(services[0].id, "machined");
                assert_eq!(services[0].state, "Running");
                assert_eq!(services[0].healthy, Some(true));
            }
            other => panic!("unexpected: {other:?}"),
        }
        handle.shutdown();
    }

    #[test]
    fn service_start_records_side_effect() {
        let server = Server::new(temp_socket(), FakeBackend::new());
        let backend = server.backend();
        let handle = server.spawn().unwrap();
        let mut c = connect(&handle);
        write_message(
            &mut c,
            &Request::ServiceStart {
                name: "etcd".to_string(),
            },
        )
        .unwrap();
        match read_message::<_, Response>(&mut c).unwrap() {
            Response::ServiceStart { resp } => assert_eq!(resp, "etcd started"),
            other => panic!("unexpected: {other:?}"),
        }
        handle.shutdown();
        assert_eq!(backend.started(), vec!["etcd".to_string()]);
    }

    #[test]
    fn dmesg_roundtrip() {
        let server = Server::new(
            temp_socket(),
            FakeBackend::new().with_dmesg(vec!["line a".to_string(), "line b".to_string()]),
        );
        let handle = server.spawn().unwrap();
        let mut c = connect(&handle);
        write_message(&mut c, &Request::Dmesg).unwrap();
        match read_message::<_, Response>(&mut c).unwrap() {
            Response::Dmesg { data } => {
                assert_eq!(String::from_utf8(data).unwrap(), "line a\nline b");
            }
            other => panic!("unexpected: {other:?}"),
        }
        handle.shutdown();
    }

    #[test]
    fn reboot_records_mode_and_acks() {
        let server = Server::new(temp_socket(), FakeBackend::new());
        let backend = server.backend();
        let handle = server.spawn().unwrap();
        let mut c = connect(&handle);
        write_message(
            &mut c,
            &Request::Reboot {
                mode: WireRebootMode::Powercycle,
            },
        )
        .unwrap();
        match read_message::<_, Response>(&mut c).unwrap() {
            Response::Reboot { ack } => assert_eq!(ack, 1),
            other => panic!("unexpected: {other:?}"),
        }
        handle.shutdown();
        assert_eq!(backend.reboots(), vec![WireRebootMode::Powercycle]);
    }

    #[test]
    fn multiple_requests_on_one_connection() {
        let handle = start();
        let mut c = connect(&handle);
        write_message(&mut c, &Request::Version).unwrap();
        let _: Response = read_message(&mut c).unwrap();
        write_message(&mut c, &Request::Hostname).unwrap();
        match read_message::<_, Response>(&mut c).unwrap() {
            Response::Hostname { hostname } => assert_eq!(hostname, "operating-system-node"),
            other => panic!("unexpected: {other:?}"),
        }
        handle.shutdown();
    }

    #[test]
    fn concurrent_clients() {
        let handle = start();
        let path = handle.path().to_path_buf();
        let mut threads = Vec::new();
        for _ in 0..16 {
            let path = path.clone();
            threads.push(thread::spawn(move || {
                let mut c = UnixStream::connect(&path).unwrap();
                write_message(&mut c, &Request::Version).unwrap();
                match read_message::<_, Response>(&mut c).unwrap() {
                    Response::Version(v) => v.tag,
                    other => panic!("unexpected: {other:?}"),
                }
            }));
        }
        for t in threads {
            assert_eq!(t.join().unwrap(), "v1.7.0");
        }
        handle.shutdown();
    }

    #[test]
    fn garbage_request_yields_error_response_not_panic() {
        let handle = start();
        let mut c = connect(&handle);
        // A well-framed payload whose first byte is not a valid Request tag.
        write_frame(&mut c, &[0xFF, 0x00, 0x01]).unwrap();
        match read_message::<_, Response>(&mut c).unwrap() {
            Response::Error(e) => {
                assert_eq!(e.code, 3);
                assert!(!e.message.is_empty());
            }
            other => panic!("expected Error, got: {other:?}"),
        }
        // Server is still alive: a fresh connection still works.
        let mut c2 = connect(&handle);
        write_message(&mut c2, &Request::Version).unwrap();
        assert!(matches!(
            read_message::<_, Response>(&mut c2).unwrap(),
            Response::Version(_)
        ));
        handle.shutdown();
    }

    #[test]
    fn shutdown_removes_socket_file() {
        let handle = start();
        let path = handle.path().to_path_buf();
        assert!(path.exists());
        handle.shutdown();
        assert!(!path.exists());
    }

    #[test]
    fn dispatch_is_pure() {
        let backend = FakeBackend::new();
        assert!(matches!(
            dispatch(&backend, Request::Version),
            Response::Version(_)
        ));
    }
}
