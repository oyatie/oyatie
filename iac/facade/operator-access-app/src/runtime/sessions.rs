use super::*;

pub(super) struct Sessions<'a> {
    pub(super) oci: Oci<'a>,
    pub(super) ids: Vec<String>,
    pub(super) tunnels: Vec<OwnedProcess>,
    pub(super) attempts: Vec<creation::Attempt>,
}

pub(super) fn valid_session_id(id: &str, region: &str) -> bool {
    id.strip_prefix(&format!("ocid1.bastionsession.oc1.{region}."))
        .is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

pub(super) fn retry_bastion_auth(message: &str, elapsed: Duration) -> bool {
    elapsed < Duration::from_secs(90)
        && message.contains("permission denied (publickey)")
        && !message.contains("host key verification failed")
}

impl Sessions<'_> {
    pub(super) fn connect(&mut self, port: u16) -> Result<u16, AccessError> {
        let p = self.oci.0;
        let port_string = port.to_string();
        let key = format!("{}.pub", p.ssh_identity_file);
        let attempt = creation::Attempt::new(port)?;
        let name = attempt.name.clone();
        self.attempts.push(attempt);
        let created = self.oci.json(
            &[
                "bastion",
                "session",
                "create-port-forwarding",
                "--bastion-id",
                &p.bastion,
                "--display-name",
                &name,
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
        let resolved = self
            .attempts
            .last()
            .ok_or(AccessError::TargetMismatch)?
            .resolve(p, &json!({"data": [created["data"]]}))?;
        if resolved.len() != 1 {
            return Err(AccessError::TargetMismatch);
        }
        let id = resolved[0].clone();
        self.ids.push(id.clone());
        self.attempts.retain(|attempt| attempt.name != name);
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
            self.tunnels.push(OwnedProcess::new(tunnel));
            let deadline = Instant::now() + Duration::from_secs(15);
            let tunnel = self
                .tunnels
                .last_mut()
                .ok_or(AccessError::DependencyFailed)?;
            loop {
                if CANCELLED.load(Ordering::Relaxed) {
                    return Err(AccessError::Cancelled);
                }
                if tunnel.exited()? {
                    if let Some(stderr) = tunnel.child()?.stderr.take() {
                        pipe_io::nonblocking(&stderr)?;
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

    pub(super) fn close(&mut self) -> Result<(), AccessError> {
        for tunnel in &mut self.tunnels {
            kill_group(tunnel);
        }
        self.tunnels.clear();
        let reconciled = creation::reconcile(&self.oci, &mut self.attempts, &mut self.ids);
        let deleted = cleanup_ids(&mut self.ids, |id| {
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
        });
        reconciled.and(deleted)
    }
}

impl Drop for Sessions<'_> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
