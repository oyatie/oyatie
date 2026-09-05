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

mod credentials;
mod process;
mod sessions;
#[cfg(test)]
mod tests;

use credentials::{Credentials, credentials};
pub use process::install_signal_handlers;
use process::{CANCELLED, Oci, kill_group, read_bounded, run, strings};
use sessions::Sessions;

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

pub(super) fn cleanup_ids(
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
