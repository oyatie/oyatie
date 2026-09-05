use crate::{AccessError, Profile, verified_digest};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use x509_parser::prelude::*;
use zeroize::{Zeroize, Zeroizing};

static CANCELLED: AtomicBool = AtomicBool::new(false);
const LIMIT: usize = 8 * 1024 * 1024;

extern "C" fn cancel(_: libc::c_int) {
    CANCELLED.store(true, Ordering::Relaxed);
}

pub fn install_signal_handlers() -> Result<(), AccessError> {
    // SAFETY: the handler only stores to a lock-free atomic. No allocation, I/O,
    // or Rust unwinding occurs in the signal handler; it lives for the process.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = cancel as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
            if libc::sigaction(signal, &action, std::ptr::null_mut()) != 0 {
                return Err(AccessError::DependencyFailed);
            }
        }
    }
    Ok(())
}

fn kill_group(child: &mut Child) {
    if !matches!(child.try_wait(), Ok(None)) {
        return;
    }
    // SAFETY: every child here is spawned into its own process group. This PID
    // remains owned and unreaped until wait(), so it cannot target a reused PID.
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.wait();
}

fn read_bounded(mut stream: impl Read) -> Result<Zeroizing<Vec<u8>>, AccessError> {
    let mut bytes = Zeroizing::new(Vec::new());
    stream
        .by_ref()
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| AccessError::DependencyFailed)?;
    if bytes.len() > LIMIT {
        return Err(AccessError::OutputLimit);
    }
    Ok(bytes)
}

fn run(
    program: &str,
    args: &[String],
    input: &[u8],
    cleanup: bool,
) -> Result<Zeroizing<Vec<u8>>, AccessError> {
    run_with_timeout(
        program,
        args,
        input,
        cleanup,
        Duration::from_secs(30),
        &CANCELLED,
    )
}

fn run_with_timeout(
    program: &str,
    args: &[String],
    input: &[u8],
    cleanup: bool,
    timeout: Duration,
    cancelled: &AtomicBool,
) -> Result<Zeroizing<Vec<u8>>, AccessError> {
    let mut child = Command::new(program)
        .args(args)
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| AccessError::DependencyFailed)?;
    let stdin = child.stdin.take().ok_or(AccessError::DependencyFailed)?;
    let stdout = child.stdout.take().ok_or(AccessError::DependencyFailed)?;
    let stderr = child.stderr.take().ok_or(AccessError::DependencyFailed)?;
    let input = Zeroizing::new(input.to_vec());
    let writer = thread::spawn(move || {
        let mut stdin = stdin;
        stdin.write_all(&input)
    });
    let reader = thread::spawn(move || read_bounded(stdout));
    let errors = thread::spawn(move || read_bounded(stderr));
    let deadline = Instant::now() + timeout;
    let result = loop {
        if !cleanup && cancelled.load(Ordering::Relaxed) {
            kill_group(&mut child);
            break Err(AccessError::Cancelled);
        }
        if Instant::now() >= deadline {
            kill_group(&mut child);
            break Err(AccessError::Timeout);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err(AccessError::DependencyFailed)
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(_) => {
                kill_group(&mut child);
                break Err(AccessError::DependencyFailed);
            }
        }
    };
    let written = writer.join().map_err(|_| AccessError::DependencyFailed)?;
    let output = reader.join().map_err(|_| AccessError::DependencyFailed)?;
    let error_bytes = errors.join().map_err(|_| AccessError::DependencyFailed)??;
    if result.is_err() {
        let message = Zeroizing::new(String::from_utf8_lossy(&error_bytes).to_lowercase());
        let conditions: Vec<_> = [
            "illegal seek",
            "permission denied",
            "unknown authority",
            "connection refused",
            "no such file",
            "cannot unmarshal",
            "read /dev/stdin",
            "expired",
            "certificate",
            "config",
            "timeout",
        ]
        .into_iter()
        .filter(|condition| message.contains(condition))
        .collect();
        eprintln!("operator_access_dependency_failure program={program} conditions={conditions:?}");
    }
    result?;
    written.map_err(|_| AccessError::DependencyFailed)?;
    output
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).to_string()).collect()
}

struct Oci<'a>(&'a Profile);
impl Oci<'_> {
    fn run(&self, args: &[&str], cleanup: bool) -> Result<Zeroizing<Vec<u8>>, AccessError> {
        let mut argv = strings(args);
        argv.extend(strings(&[
            "--config-file",
            &self.0.oci_config_file,
            "--profile",
            &self.0.oci_profile,
            "--auth",
            "security_token",
            "--region",
            &self.0.region,
            "--max-retries",
            "0",
            "--connection-timeout",
            "10",
            "--read-timeout",
            "15",
        ]));
        run("oci", &argv, &[], cleanup)
    }
    fn json(&self, args: &[&str], cleanup: bool) -> Result<Value, AccessError> {
        serde_json::from_slice(&self.run(args, cleanup)?).map_err(|_| AccessError::DependencyFailed)
    }
}

struct SecretYaml(serde_yaml::Value);
impl Drop for SecretYaml {
    fn drop(&mut self) {
        fn wipe(v: &mut serde_yaml::Value) {
            match v {
                serde_yaml::Value::String(s) => s.zeroize(),
                serde_yaml::Value::Sequence(items) => items.iter_mut().for_each(wipe),
                serde_yaml::Value::Mapping(items) => items.values_mut().for_each(wipe),
                _ => (),
            }
        }
        wipe(&mut self.0);
    }
}

fn decode(value: &serde_yaml::Value) -> Result<Zeroizing<Vec<u8>>, AccessError> {
    STANDARD
        .decode(value.as_str().ok_or(AccessError::InvalidCredentials)?)
        .map(Zeroizing::new)
        .map_err(|_| AccessError::InvalidCredentials)
}

fn check_certificate(ca_pem: &[u8], crt_pem: &[u8], expected_ca: &str) -> Result<(), AccessError> {
    if !verified_digest(ca_pem, expected_ca) {
        return Err(AccessError::InvalidCredentials);
    }
    let (_, ca) = parse_x509_pem(ca_pem).map_err(|_| AccessError::InvalidCredentials)?;
    let (_, crt) = parse_x509_pem(crt_pem).map_err(|_| AccessError::InvalidCredentials)?;
    let ca = ca
        .parse_x509()
        .map_err(|_| AccessError::InvalidCredentials)?;
    let crt = crt
        .parse_x509()
        .map_err(|_| AccessError::InvalidCredentials)?;
    if !ca.validity().is_valid() || !crt.validity().is_valid() {
        return Err(AccessError::CertificateExpired);
    }
    crt.verify_signature(Some(ca.public_key()))
        .map_err(|_| AccessError::InvalidCredentials)
}

struct Credentials {
    talos: Zeroizing<Vec<u8>>,
    kube: Zeroizing<Vec<u8>>,
}

fn credentials(profile: &Profile, archive: &[u8]) -> Result<Credentials, AccessError> {
    let talos = run(
        "tar",
        &strings(&["-xOf", "-", &profile.talos_member]),
        archive,
        false,
    )?;
    let kube = run(
        "tar",
        &strings(&["-xOf", "-", &profile.kube_member]),
        archive,
        false,
    )?;
    let t =
        SecretYaml(serde_yaml::from_slice(&talos).map_err(|_| AccessError::InvalidCredentials)?);
    if t.0["context"].as_str() != Some(&profile.talos_context) {
        return Err(AccessError::InvalidCredentials);
    }
    let context = &t.0["contexts"][profile.talos_context.as_str()];
    if context.as_mapping().is_none_or(|m| {
        m.keys().any(|k| {
            !matches!(
                k.as_str(),
                Some("ca" | "crt" | "key" | "endpoints" | "nodes")
            )
        })
    }) {
        return Err(AccessError::InvalidCredentials);
    }
    check_certificate(
        &decode(&context["ca"])?,
        &decode(&context["crt"])?,
        &profile.talos_ca_sha256,
    )?;
    if context["key"].as_str().is_none_or(str::is_empty) {
        return Err(AccessError::InvalidCredentials);
    }
    let k = SecretYaml(serde_yaml::from_slice(&kube).map_err(|_| AccessError::InvalidCredentials)?);
    let current = k.0["current-context"]
        .as_str()
        .ok_or(AccessError::InvalidCredentials)?;
    let contexts = k.0["contexts"]
        .as_sequence()
        .ok_or(AccessError::InvalidCredentials)?;
    let matches: Vec<_> = contexts
        .iter()
        .filter(|v| v["name"].as_str() == Some(current))
        .collect();
    if matches.len() != 1 {
        return Err(AccessError::InvalidCredentials);
    }
    let cluster_name = matches[0]["context"]["cluster"]
        .as_str()
        .ok_or(AccessError::InvalidCredentials)?;
    let user_name = matches[0]["context"]["user"]
        .as_str()
        .ok_or(AccessError::InvalidCredentials)?;
    let clusters = k.0["clusters"]
        .as_sequence()
        .ok_or(AccessError::InvalidCredentials)?;
    let users = k.0["users"]
        .as_sequence()
        .ok_or(AccessError::InvalidCredentials)?;
    if clusters.len() != 1
        || users.len() != 1
        || clusters[0]["name"].as_str() != Some(cluster_name)
        || users[0]["name"].as_str() != Some(user_name)
    {
        return Err(AccessError::InvalidCredentials);
    }
    let cluster = &clusters[0]["cluster"];
    let user = &users[0]["user"];
    if cluster["server"].as_str() != Some(&format!("https://{}:6443", profile.private_ip))
        || cluster["insecure-skip-tls-verify"].as_bool() == Some(true)
        || !cluster["proxy-url"].is_null()
        || user.as_mapping().is_none_or(|m| m.len() != 2)
        || user["client-key-data"].as_str().is_none_or(str::is_empty)
    {
        return Err(AccessError::InvalidCredentials);
    }
    check_certificate(
        &decode(&cluster["certificate-authority-data"])?,
        &decode(&user["client-certificate-data"])?,
        &profile.kube_ca_sha256,
    )?;
    Ok(Credentials { talos, kube })
}

struct Sessions<'a> {
    oci: Oci<'a>,
    ids: Vec<String>,
    tunnels: Vec<Child>,
}

fn valid_session_id(id: &str, region: &str) -> bool {
    id.strip_prefix(&format!("ocid1.bastionsession.oc1.{region}."))
        .is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

fn retry_bastion_auth(message: &str, elapsed: Duration) -> bool {
    elapsed < Duration::from_secs(90)
        && message.contains("permission denied (publickey)")
        && !message.contains("host key verification failed")
}

impl Sessions<'_> {
    fn connect(&mut self, port: u16) -> Result<u16, AccessError> {
        let p = self.oci.0;
        let port_string = port.to_string();
        let key = format!("{}.pub", p.ssh_identity_file);
        let created = self.oci.json(
            &[
                "bastion",
                "session",
                "create-port-forwarding",
                "--bastion-id",
                &p.bastion,
                "--display-name",
                "seed-operator-access",
                "--ssh-public-key-file",
                &key,
                "--key-type",
                "PUB",
                "--session-ttl",
                "1800",
                "--target-resource-id",
                &p.instance,
                "--target-private-ip",
                &p.private_ip,
                "--target-port",
                &port_string,
            ],
            false,
        )?;
        let id = created["data"]["id"]
            .as_str()
            .filter(|id| valid_session_id(id, &p.region))
            .ok_or(AccessError::TargetMismatch)?
            .to_string();
        self.ids.push(id.clone());
        let deadline = Instant::now() + Duration::from_secs(90);
        loop {
            let session = self
                .oci
                .json(&["bastion", "session", "get", "--session-id", &id], false)?;
            let d = &session["data"];
            let target = &d["target-resource-details"];
            if d["id"] != id
                || d["bastion-id"] != p.bastion
                || target["target-resource-id"] != p.instance
                || target["target-resource-private-ip-address"] != p.private_ip
                || target["target-resource-port"].as_u64() != Some(u64::from(port))
            {
                return Err(AccessError::TargetMismatch);
            }
            if d["lifecycle-state"] == "ACTIVE" {
                break;
            }
            if d["lifecycle-state"] != "CREATING" {
                return Err(AccessError::DependencyFailed);
            }
            if Instant::now() >= deadline {
                return Err(AccessError::Timeout);
            }
            thread::sleep(Duration::from_millis(250));
        }
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|_| AccessError::DependencyFailed)?;
        let local_port = listener
            .local_addr()
            .map_err(|_| AccessError::DependencyFailed)?
            .port();
        drop(listener);
        let forward = format!("127.0.0.1:{local_port}:{}:{port}", p.private_ip);
        let host = format!("{id}@host.bastion.{}.oci.oraclecloud.com", p.region);
        let known_hosts = format!("UserKnownHostsFile={}", p.ssh_known_hosts_file);
        let started = Instant::now();
        'authenticate: loop {
            let tunnel = Command::new("ssh")
                .args([
                    "-F",
                    "/dev/null",
                    "-N",
                    "-L",
                    &forward,
                    "-i",
                    &p.ssh_identity_file,
                    "-o",
                    "BatchMode=yes",
                    "-o",
                    "IdentitiesOnly=yes",
                    "-o",
                    "StrictHostKeyChecking=yes",
                    "-o",
                    &known_hosts,
                    "-o",
                    "ExitOnForwardFailure=yes",
                    "-o",
                    "ConnectTimeout=10",
                    "-o",
                    "ServerAliveInterval=15",
                    "-o",
                    "ServerAliveCountMax=2",
                    &host,
                ])
                .process_group(0)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|_| AccessError::DependencyFailed)?;
            self.tunnels.push(tunnel);
            let deadline = Instant::now() + Duration::from_secs(15);
            let tunnel = self
                .tunnels
                .last_mut()
                .ok_or(AccessError::DependencyFailed)?;
            loop {
                if CANCELLED.load(Ordering::Relaxed) {
                    return Err(AccessError::Cancelled);
                }
                if tunnel
                    .try_wait()
                    .map_err(|_| AccessError::DependencyFailed)?
                    .is_some()
                {
                    if let Some(stderr) = tunnel.stderr.take() {
                        let bytes = read_bounded(stderr)?;
                        let message =
                            Zeroizing::new(String::from_utf8_lossy(&bytes).to_lowercase());
                        let conditions: Vec<_> = [
                            "permission denied",
                            "host key verification failed",
                            "connection closed",
                            "connection reset",
                            "publickey",
                            "bad permissions",
                            "could not resolve",
                            "address already in use",
                            "bad configuration option",
                            "no such file",
                            "timed out",
                            "unsupported",
                        ]
                        .into_iter()
                        .filter(|condition| message.contains(condition))
                        .collect();
                        eprintln!("operator_access_tunnel_failed conditions={conditions:?}");
                        if retry_bastion_auth(&message, started.elapsed()) {
                            self.tunnels.pop();
                            thread::sleep(Duration::from_secs(1));
                            continue 'authenticate;
                        }
                    }
                    return Err(AccessError::DependencyFailed);
                }
                if TcpStream::connect_timeout(
                    &std::net::SocketAddr::from(([127, 0, 0, 1], local_port)),
                    Duration::from_millis(100),
                )
                .is_ok()
                {
                    return Ok(local_port);
                }
                if Instant::now() >= deadline {
                    return Err(AccessError::Timeout);
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    fn close(&mut self) -> Result<(), AccessError> {
        for tunnel in &mut self.tunnels {
            kill_group(tunnel);
        }
        self.tunnels.clear();
        cleanup_ids(&mut self.ids, |id| {
            self.oci.run(
                &[
                    "bastion",
                    "session",
                    "delete",
                    "--session-id",
                    id,
                    "--force",
                ],
                true,
            )?;
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                let observed = self
                    .oci
                    .json(&["bastion", "session", "get", "--session-id", id], true)?;
                if observed["data"]["lifecycle-state"] == "DELETED" {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    return Err(AccessError::CleanupFailed);
                }
                thread::sleep(Duration::from_millis(250));
            }
        })
    }
}

impl Drop for Sessions<'_> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub fn status(p: &Profile) -> Result<Value, AccessError> {
    p.validate()?;
    let oci = Oci(p);
    oci.run(&["session", "validate", "--local"], false)
        .map_err(|_| AccessError::OciAuthenticationRequired)?;
    p.verify_instance(&oci.json(
        &["compute", "instance", "get", "--instance-id", &p.instance],
        false,
    )?)?;
    p.verify_bastion(&oci.json(
        &["bastion", "bastion", "get", "--bastion-id", &p.bastion],
        false,
    )?)?;
    let cipher = oci.run(
        &[
            "os",
            "object",
            "get",
            "--namespace-name",
            &p.namespace,
            "--bucket-name",
            &p.bucket,
            "--name",
            &p.object,
            "--version-id",
            &p.object_version,
            "--file",
            "-",
        ],
        false,
    )?;
    if !verified_digest(&cipher, &p.archive_sha256) {
        return Err(AccessError::ArchiveMismatch);
    }
    let archive = run(
        "age",
        &strings(&["--decrypt", "--identity", &p.identity_file]),
        &cipher,
        false,
    )?;
    let Credentials { talos, kube } = credentials(p, &archive)?;
    let mut sessions = Sessions {
        oci,
        ids: Vec::new(),
        tunnels: Vec::new(),
    };
    let result = (|| {
        let talos_port = sessions.connect(50000)?;
        let endpoint = format!("127.0.0.1:{talos_port}");
        run(
            "talosctl",
            &strings(&[
                "--talosconfig",
                "/dev/stdin",
                "--endpoints",
                &endpoint,
                "--nodes",
                &p.private_ip,
                "version",
            ]),
            &talos,
            false,
        )?;
        let kube_port = sessions.connect(6443)?;
        let server = format!("https://127.0.0.1:{kube_port}");
        let output = run(
            "kubectl",
            &strings(&[
                "--kubeconfig",
                "/dev/stdin",
                "--server",
                &server,
                "--tls-server-name",
                &p.private_ip,
                "--request-timeout=15s",
                "get",
                "node",
                &p.node_name,
                "-o",
                "json",
            ]),
            &kube,
            false,
        )?;
        let node: Value =
            serde_json::from_slice(&output).map_err(|_| AccessError::DependencyFailed)?;
        if node["metadata"]["name"] != p.node_name
            || !node["status"]["addresses"].as_array().is_some_and(|a| {
                a.iter()
                    .any(|v| v["type"] == "InternalIP" && v["address"] == p.private_ip)
            })
        {
            return Err(AccessError::TargetMismatch);
        }
        let ready = node["status"]["conditions"].as_array().is_some_and(|a| {
            a.iter()
                .any(|v| v["type"] == "Ready" && v["status"] == "True")
        });
        Ok(
            json!({"schema": 1, "node": p.node_name, "talos_authenticated": true,
            "kubernetes_authenticated": true, "node_ready": ready,
            "os_image": node["status"]["nodeInfo"]["osImage"],
            "kubernetes_version": node["status"]["nodeInfo"]["kubeletVersion"]}),
        )
    })();
    sessions.close()?;
    result
}

fn cleanup_ids(
    ids: &mut Vec<String>,
    mut delete: impl FnMut(&str) -> Result<(), AccessError>,
) -> Result<(), AccessError> {
    ids.retain(|id| delete(id).is_err());
    if ids.is_empty() {
        Ok(())
    } else {
        Err(AccessError::CleanupFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn bastion_auth_readiness_retry_is_bounded_and_never_relaxes_host_trust() {
        assert!(retry_bastion_auth(
            "permission denied (publickey)",
            Duration::from_secs(2)
        ));
        assert!(!retry_bastion_auth(
            "permission denied (publickey)",
            Duration::from_secs(90)
        ));
        assert!(!retry_bastion_auth(
            "host key verification failed; permission denied (publickey)",
            Duration::ZERO
        ));
        assert!(!retry_bastion_auth("connection refused", Duration::ZERO));
    }

    #[test]
    fn session_ids_admit_oci_region_hyphens_but_not_other_regions_or_arguments() {
        assert!(valid_session_id(
            "ocid1.bastionsession.oc1.ap-chuncheon-1.abc",
            "ap-chuncheon-1"
        ));
        assert!(!valid_session_id(
            "ocid1.bastionsession.oc1.other.abc",
            "ap-chuncheon-1"
        ));
        assert!(!valid_session_id("--endpoint=other", "ap-chuncheon-1"));
    }

    #[test]
    fn bounds_dependency_output() {
        assert_eq!(
            read_bounded(&vec![0; LIMIT + 1][..]).unwrap_err(),
            AccessError::OutputLimit
        );
        assert_eq!(read_bounded(&b"small"[..]).unwrap().as_slice(), b"small");
    }

    #[test]
    fn dependency_failures_never_render_stderr_or_credentials() {
        let error = run(
            "/bin/sh",
            &strings(&["-c", "echo secret-fixture >&2; exit 7"]),
            &[],
            false,
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "operator_access_dependency_failed");
    }

    #[test]
    fn process_timeout_is_bounded_and_reaps_process_group() {
        let start = Instant::now();
        assert_eq!(
            run_with_timeout(
                "/bin/sh",
                &strings(&["-c", "sleep 60"]),
                &[],
                false,
                Duration::from_millis(50),
                &AtomicBool::new(false)
            )
            .unwrap_err(),
            AccessError::Timeout
        );
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn cancellation_reaps_the_command_but_does_not_prevent_cleanup() {
        let cancelled = AtomicBool::new(true);
        assert_eq!(
            run_with_timeout(
                "/bin/sh",
                &strings(&["-c", "sleep 60"]),
                &[],
                false,
                Duration::from_secs(2),
                &cancelled
            )
            .unwrap_err(),
            AccessError::Cancelled
        );
        assert!(
            run_with_timeout(
                "/bin/sh",
                &strings(&["-c", "exit 0"]),
                &[],
                true,
                Duration::from_secs(2),
                &cancelled
            )
            .is_ok()
        );
    }

    #[test]
    fn cleanup_attempts_every_session_and_retains_failures_for_retry() {
        let mut ids = vec!["first".to_string(), "second".to_string()];
        let mut attempted = Vec::new();
        let result = cleanup_ids(&mut ids, |id| {
            attempted.push(id.to_string());
            if id == "first" {
                Err(AccessError::DependencyFailed)
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err(AccessError::CleanupFailed));
        assert_eq!(attempted, ["first", "second"]);
        assert_eq!(ids, ["first"]);
        assert_eq!(cleanup_ids(&mut ids, |_| Ok(())), Ok(()));
        assert!(ids.is_empty());
    }

    #[test]
    fn certificates_reject_wrong_trust_and_expired_identity() {
        let key = rcgen::KeyPair::generate().unwrap();
        let mut params = rcgen::CertificateParams::default();
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let ca = params.self_signed(&key).unwrap();
        let digest = format!("{:x}", Sha256::digest(ca.pem().as_bytes()));
        assert_eq!(
            check_certificate(ca.pem().as_bytes(), ca.pem().as_bytes(), &digest),
            Ok(())
        );
        assert_eq!(
            check_certificate(ca.pem().as_bytes(), ca.pem().as_bytes(), &"0".repeat(64)),
            Err(AccessError::InvalidCredentials)
        );
        params.not_before = rcgen::date_time_ymd(2000, 1, 1);
        params.not_after = rcgen::date_time_ymd(2001, 1, 1);
        let expired = params.self_signed(&key).unwrap();
        let digest = format!("{:x}", Sha256::digest(expired.pem().as_bytes()));
        assert_eq!(
            check_certificate(expired.pem().as_bytes(), expired.pem().as_bytes(), &digest),
            Err(AccessError::CertificateExpired)
        );
    }
}
