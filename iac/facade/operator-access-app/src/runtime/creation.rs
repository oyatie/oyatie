use super::*;

pub(super) struct Attempt {
    pub(super) name: String,
    pub(super) port: u16,
}

impl Attempt {
    pub(super) fn new(port: u16) -> Result<Self, AccessError> {
        let mut random = [0u8; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut random))
            .map_err(|_| AccessError::DependencyFailed)?;
        let suffix: String = random.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Self {
            name: format!("operator-access-{suffix}"),
            port,
        })
    }

    pub(super) fn resolve(&self, p: &Profile, listing: &Value) -> Result<Vec<String>, AccessError> {
        let rows = listing["data"]
            .as_array()
            .ok_or(AccessError::CleanupFailed)?;
        let mut ids = Vec::new();
        for row in rows.iter().filter(|row| row["display-name"] == self.name) {
            let target = &row["target-resource-details"];
            let id = row["id"].as_str().ok_or(AccessError::CleanupFailed)?;
            if row["bastion-id"] != p.bastion
                || target["target-resource-id"] != p.instance
                || target["target-resource-private-ip-address"] != p.private_ip
                || target["target-resource-port"].as_u64() != Some(u64::from(self.port))
                || !sessions::valid_session_id(id, &p.region)
            {
                return Err(AccessError::CleanupFailed);
            }
            ids.push(id.to_string());
        }
        Ok(ids)
    }
}

pub(super) fn reconcile(
    oci: &Oci<'_>,
    attempts: &mut Vec<Attempt>,
    ids: &mut Vec<String>,
) -> Result<(), AccessError> {
    reconcile_with(oci.0, attempts, ids, Duration::from_secs(30), || {
        oci.json(
            &[
                "bastion",
                "session",
                "list",
                "--bastion-id",
                &oci.0.bastion,
                "--all",
            ],
            true,
        )
    })
}

fn reconcile_with(
    p: &Profile,
    attempts: &mut Vec<Attempt>,
    ids: &mut Vec<String>,
    timeout: Duration,
    mut list: impl FnMut() -> Result<Value, AccessError>,
) -> Result<(), AccessError> {
    let deadline = Instant::now() + timeout;
    while !attempts.is_empty() {
        let listing = list()?;
        let mut unresolved = Vec::new();
        for attempt in attempts.drain(..) {
            match attempt.resolve(p, &listing) {
                Ok(found) if !found.is_empty() => ids.extend(found),
                _ => unresolved.push(attempt),
            }
        }
        *attempts = unresolved;
        if attempts.is_empty() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(AccessError::CleanupFailed);
        }
        thread::sleep(Duration::from_millis(250));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_create_with_lost_response_is_resolved_without_other_operators() {
        let p = crate::tests::profile();
        let attempt = Attempt::new(50000).unwrap();
        let id = "ocid1.bastionsession.oc1.ap-chuncheon-1.abc";
        let row = json!({"id": id, "display-name": attempt.name, "bastion-id": p.bastion,
            "target-resource-details": {"target-resource-id": p.instance,
                "target-resource-private-ip-address": p.private_ip, "target-resource-port": 50000}});
        let mut other = row.clone();
        other["display-name"] = json!("other-operator");
        assert_eq!(
            attempt.resolve(&p, &json!({"data": [other, row]})).unwrap(),
            [id]
        );
        assert!(
            attempt
                .resolve(&p, &json!({"data": []}))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn matching_name_with_wrong_target_is_never_compensated() {
        let p = crate::tests::profile();
        let attempt = Attempt::new(50000).unwrap();
        assert_eq!(
            attempt.resolve(
                &p,
                &json!({"data": [{"display-name": attempt.name,
            "id": "ocid1.bastionsession.oc1.ap-chuncheon-1.abc", "bastion-id": "other"}]})
            ),
            Err(AccessError::CleanupFailed)
        );
    }

    #[test]
    fn cancelled_create_reconciles_delayed_visibility_and_compensates_exact_session() {
        let p = crate::tests::profile();
        let attempt = Attempt::new(50000).unwrap();
        let row = json!({"id": "ocid1.bastionsession.oc1.ap-chuncheon-1.abc",
            "display-name": attempt.name, "bastion-id": p.bastion,
            "target-resource-details": {"target-resource-id": p.instance,
                "target-resource-private-ip-address": p.private_ip, "target-resource-port": 50000}});
        let mut attempts = vec![attempt];
        let mut ids = Vec::new();
        let mut calls = 0;
        let result = reconcile_with(&p, &mut attempts, &mut ids, Duration::from_secs(2), || {
            calls += 1;
            Ok(if calls == 1 {
                json!({"data": []})
            } else {
                json!({"data": [row]})
            })
        });
        assert_eq!(result, Ok(()));
        assert!(attempts.is_empty());
        let mut deleted = Vec::new();
        assert_eq!(
            cleanup_ids(&mut ids, |id| {
                deleted.push(id.to_string());
                Ok(())
            }),
            Ok(())
        );
        assert_eq!(deleted, ["ocid1.bastionsession.oc1.ap-chuncheon-1.abc"]);
    }

    #[test]
    fn unknown_creation_remains_owned_and_never_reports_cleanup_success() {
        let p = crate::tests::profile();
        let mut attempts = vec![Attempt::new(50000).unwrap()];
        let mut ids = Vec::new();
        assert_eq!(
            reconcile_with(&p, &mut attempts, &mut ids, Duration::ZERO, || Ok(
                json!({"data": []})
            )),
            Err(AccessError::CleanupFailed)
        );
        assert_eq!(attempts.len(), 1);
        assert!(ids.is_empty());
    }
}
