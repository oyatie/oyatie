//! Kernel-parameter reconciliation controller.
//!
//! Mirrors Talos `KernelParamConfigController` /
//! `KernelParamDefaultsController`: it takes a desired set of
//! [`KernelParamSpec`]s, reads the current kernel value through a
//! [`KernelParamSink`], writes the desired value where it differs, records the
//! prior value as the default (for teardown), and tolerates failures for specs
//! marked `ignore_failure` or for keys the kernel does not expose.

use std::collections::{BTreeMap, BTreeSet};

use crate::kernel_param::{KernelParamError, KernelParamSink, KernelParamSpec, KernelParamStatus};

/// Outcome of reconciling one spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamOutcome {
    /// The value was changed from `from` to `to`.
    Changed { from: String, to: String },
    /// The value already matched the desired value; nothing written.
    Unchanged,
    /// The key was not exposed by the kernel and the spec tolerated it.
    Skipped,
    /// A write failed but the spec tolerated the failure.
    Failed(String),
}

/// The result of a full reconcile pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReconcileReport {
    /// Per-key status records (the COSI `KernelParamStatus` outputs).
    pub statuses: Vec<KernelParamStatus>,
    /// Per-key outcomes, keyed by parameter key.
    pub outcomes: BTreeMap<String, ParamOutcome>,
}

impl ReconcileReport {
    /// Number of keys whose value was actually changed.
    pub fn changed_count(&self) -> usize {
        self.outcomes
            .values()
            .filter(|o| matches!(o, ParamOutcome::Changed { .. }))
            .count()
    }

    /// Number of keys skipped because the kernel did not expose them.
    pub fn skipped_count(&self) -> usize {
        self.outcomes
            .values()
            .filter(|o| matches!(o, ParamOutcome::Skipped))
            .count()
    }

    /// Number of tolerated failures.
    pub fn failed_count(&self) -> usize {
        self.outcomes
            .values()
            .filter(|o| matches!(o, ParamOutcome::Failed(_)))
            .count()
    }

    /// The status for a given key, if recorded.
    pub fn status(&self, key: &str) -> Option<&KernelParamStatus> {
        self.statuses.iter().find(|s| s.key == key)
    }
}

/// Reconciles desired kernel-param specs against a sink.
///
/// Holds the recorded defaults so it can restore them on teardown, exactly like
/// the Talos controller restores `KernelParamDefaultSpec` values when a config
/// param is removed.
#[derive(Debug, Default)]
pub struct KernelParamController {
    /// Recorded prior values, keyed by param key, captured on first change.
    defaults: BTreeMap<String, String>,
}

impl KernelParamController {
    /// A fresh controller with no recorded defaults.
    pub fn new() -> Self {
        KernelParamController::default()
    }

    /// Reconcile a batch of specs against `sink`.
    ///
    /// For each spec: validate it, read the current value, write the desired
    /// value if different (recording the prior value as the default), and build
    /// a [`KernelParamStatus`]. A duplicate key in the batch is a hard
    /// [`KernelParamError::Conflict`]. Write failures abort the pass unless the
    /// spec set `ignore_failure`.
    pub fn reconcile(
        &mut self,
        specs: &[KernelParamSpec],
        sink: &mut impl KernelParamSink,
    ) -> Result<ReconcileReport, KernelParamError> {
        let mut report = ReconcileReport::default();
        let mut seen: BTreeSet<&str> = BTreeSet::new();

        for spec in specs {
            spec.validate()?;
            if !seen.insert(spec.key.as_str()) {
                return Err(KernelParamError::Conflict(spec.key.clone()));
            }

            // Key not exposed by the kernel.
            if !sink.exists(&spec.key) {
                if spec.ignore_failure {
                    report
                        .outcomes
                        .insert(spec.key.clone(), ParamOutcome::Skipped);
                    report
                        .statuses
                        .push(KernelParamStatus::unsupported(&spec.key));
                    continue;
                }
                return Err(KernelParamError::NotFound(spec.key.clone()));
            }

            let current = sink.read(&spec.key)?;
            if current == spec.value {
                report
                    .outcomes
                    .insert(spec.key.clone(), ParamOutcome::Unchanged);
                let default = self.defaults.get(&spec.key).cloned();
                report
                    .statuses
                    .push(KernelParamStatus::applied(&spec.key, &spec.value, default));
                continue;
            }

            match sink.write(&spec.key, &spec.value) {
                Ok(()) => {
                    // Record the original value as the default the first time we
                    // change this key.
                    self.defaults
                        .entry(spec.key.clone())
                        .or_insert_with(|| current.clone());
                    report.outcomes.insert(
                        spec.key.clone(),
                        ParamOutcome::Changed {
                            from: current.clone(),
                            to: spec.value.clone(),
                        },
                    );
                    let default = self.defaults.get(&spec.key).cloned();
                    report.statuses.push(KernelParamStatus::applied(
                        &spec.key,
                        &spec.value,
                        default,
                    ));
                }
                Err(e) => {
                    if spec.ignore_failure {
                        report
                            .outcomes
                            .insert(spec.key.clone(), ParamOutcome::Failed(e.to_string()));
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Ok(report)
    }

    /// Restore every recorded default to the sink (teardown). Keys with no
    /// recorded default are left untouched. Returns the number restored.
    pub fn teardown(&mut self, sink: &mut impl KernelParamSink) -> Result<usize, KernelParamError> {
        let mut restored = 0;
        for (key, value) in &self.defaults {
            if sink.exists(key) {
                sink.write(key, value)?;
                restored += 1;
            }
        }
        self.defaults.clear();
        Ok(restored)
    }

    /// The recorded default for a key, if the controller has changed it.
    pub fn recorded_default(&self, key: &str) -> Option<&str> {
        self.defaults.get(key).map(String::as_str)
    }

    /// How many keys this controller is currently managing (has changed).
    pub fn managed_len(&self) -> usize {
        self.defaults.len()
    }
}

/// A dry-run plan: what a reconcile *would* do, without mutating the sink.
///
/// Mirrors the `--dry-run` diff Talos can produce for kernel params: each spec
/// is classified as a change (with the from/to values), already-satisfied, or
/// unsupported, so an operator can preview the hardening before applying.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReconcilePlan {
    /// Keys that would change, as `(key, from, to)`.
    pub changes: Vec<(String, String, String)>,
    /// Keys already at the desired value.
    pub unchanged: Vec<String>,
    /// Keys the sink does not expose.
    pub unsupported: Vec<String>,
}

impl ReconcilePlan {
    /// Compute the plan for `specs` against `sink` without writing anything.
    pub fn compute(
        specs: &[KernelParamSpec],
        sink: &impl KernelParamSink,
    ) -> Result<Self, KernelParamError> {
        let mut plan = ReconcilePlan::default();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for spec in specs {
            spec.validate()?;
            if !seen.insert(spec.key.as_str()) {
                return Err(KernelParamError::Conflict(spec.key.clone()));
            }
            match sink.read(&spec.key) {
                Ok(current) if current == spec.value => plan.unchanged.push(spec.key.clone()),
                Ok(current) => {
                    plan.changes
                        .push((spec.key.clone(), current, spec.value.clone()));
                }
                Err(_) => plan.unsupported.push(spec.key.clone()),
            }
        }
        Ok(plan)
    }

    /// Whether applying this plan would change nothing.
    pub fn is_noop(&self) -> bool {
        self.changes.is_empty()
    }

    /// Total number of keys that would change.
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_param::MemoryParamSink;

    fn specs() -> Vec<KernelParamSpec> {
        vec![
            KernelParamSpec::sysctl("net.ipv4.ip_forward", "1").unwrap(),
            KernelParamSpec::sysctl("kernel.kptr_restrict", "1").unwrap(),
        ]
    }

    #[test]
    fn reconcile_changes_and_records_defaults() {
        let mut sink = MemoryParamSink::new()
            .with("net.ipv4.ip_forward", "0")
            .with("kernel.kptr_restrict", "0");
        let mut ctrl = KernelParamController::new();
        let report = ctrl.reconcile(&specs(), &mut sink).unwrap();

        assert_eq!(report.changed_count(), 2);
        assert_eq!(sink.read("net.ipv4.ip_forward").unwrap(), "1");
        assert_eq!(ctrl.recorded_default("net.ipv4.ip_forward"), Some("0"));
        let st = report.status("net.ipv4.ip_forward").unwrap();
        assert!(st.was_changed());
    }

    #[test]
    fn reconcile_unchanged_when_already_set() {
        let mut sink = MemoryParamSink::new()
            .with("net.ipv4.ip_forward", "1")
            .with("kernel.kptr_restrict", "1");
        let mut ctrl = KernelParamController::new();
        let report = ctrl.reconcile(&specs(), &mut sink).unwrap();
        assert_eq!(report.changed_count(), 0);
        assert!(matches!(
            report.outcomes.get("net.ipv4.ip_forward"),
            Some(ParamOutcome::Unchanged)
        ));
        assert_eq!(ctrl.managed_len(), 0);
    }

    #[test]
    fn missing_key_aborts_unless_ignored() {
        let mut sink = MemoryParamSink::new().with("net.ipv4.ip_forward", "0");
        let mut ctrl = KernelParamController::new();
        // kernel.kptr_restrict not present -> NotFound.
        assert!(matches!(
            ctrl.reconcile(&specs(), &mut sink),
            Err(KernelParamError::NotFound(_))
        ));

        // With ignore_failure the missing key is skipped.
        let tolerant = vec![
            KernelParamSpec::sysctl("net.ipv4.ip_forward", "1").unwrap(),
            KernelParamSpec::sysctl("kernel.kptr_restrict", "1")
                .unwrap()
                .ignoring_failure(),
        ];
        let report = ctrl.reconcile(&tolerant, &mut sink).unwrap();
        assert_eq!(report.skipped_count(), 1);
        assert!(report.status("kernel.kptr_restrict").unwrap().unsupported);
    }

    #[test]
    fn read_only_write_failure_tolerated_when_ignored() {
        let mut sink = MemoryParamSink::new()
            .with("kernel.kexec_load_disabled", "0")
            .read_only("kernel.kexec_load_disabled");
        let mut ctrl = KernelParamController::new();
        let tolerant = vec![
            KernelParamSpec::sysctl("kernel.kexec_load_disabled", "1")
                .unwrap()
                .ignoring_failure(),
        ];
        let report = ctrl.reconcile(&tolerant, &mut sink).unwrap();
        assert_eq!(report.failed_count(), 1);

        // Without ignore the same write aborts.
        let strict = vec![KernelParamSpec::sysctl("kernel.kexec_load_disabled", "1").unwrap()];
        assert!(matches!(
            ctrl.reconcile(&strict, &mut sink),
            Err(KernelParamError::WriteRejected(_))
        ));
    }

    #[test]
    fn duplicate_key_is_conflict() {
        let mut sink = MemoryParamSink::new().with("net.ipv4.ip_forward", "0");
        let mut ctrl = KernelParamController::new();
        let dup = vec![
            KernelParamSpec::sysctl("net.ipv4.ip_forward", "1").unwrap(),
            KernelParamSpec::sysctl("net.ipv4.ip_forward", "0").unwrap(),
        ];
        assert!(matches!(
            ctrl.reconcile(&dup, &mut sink),
            Err(KernelParamError::Conflict(_))
        ));
    }

    #[test]
    fn teardown_restores_defaults() {
        let mut sink = MemoryParamSink::new()
            .with("net.ipv4.ip_forward", "0")
            .with("kernel.kptr_restrict", "0");
        let mut ctrl = KernelParamController::new();
        ctrl.reconcile(&specs(), &mut sink).unwrap();
        assert_eq!(sink.read("net.ipv4.ip_forward").unwrap(), "1");

        let restored = ctrl.teardown(&mut sink).unwrap();
        assert_eq!(restored, 2);
        assert_eq!(sink.read("net.ipv4.ip_forward").unwrap(), "0");
        assert_eq!(ctrl.managed_len(), 0);
    }

    #[test]
    fn plan_classifies_changes_unchanged_unsupported() {
        let sink = MemoryParamSink::new()
            .with("net.ipv4.ip_forward", "0")
            .with("kernel.kptr_restrict", "1");
        let specs = vec![
            KernelParamSpec::sysctl("net.ipv4.ip_forward", "1").unwrap(),
            KernelParamSpec::sysctl("kernel.kptr_restrict", "1").unwrap(),
            KernelParamSpec::sysctl("kernel.does_not_exist", "1").unwrap(),
        ];
        let plan = ReconcilePlan::compute(&specs, &sink).unwrap();
        assert_eq!(plan.change_count(), 1);
        assert_eq!(
            plan.changes[0],
            (
                "net.ipv4.ip_forward".to_string(),
                "0".to_string(),
                "1".to_string()
            )
        );
        assert_eq!(plan.unchanged, vec!["kernel.kptr_restrict".to_string()]);
        assert_eq!(plan.unsupported, vec!["kernel.does_not_exist".to_string()]);
        assert!(!plan.is_noop());
    }

    #[test]
    fn plan_does_not_mutate_sink() {
        let sink = MemoryParamSink::new().with("net.ipv4.ip_forward", "0");
        let specs = vec![KernelParamSpec::sysctl("net.ipv4.ip_forward", "1").unwrap()];
        let _ = ReconcilePlan::compute(&specs, &sink).unwrap();
        // Sink unchanged.
        assert_eq!(sink.read("net.ipv4.ip_forward").unwrap(), "0");
    }

    #[test]
    fn plan_noop_when_all_satisfied() {
        let sink = MemoryParamSink::new().with("net.ipv4.ip_forward", "1");
        let specs = vec![KernelParamSpec::sysctl("net.ipv4.ip_forward", "1").unwrap()];
        let plan = ReconcilePlan::compute(&specs, &sink).unwrap();
        assert!(plan.is_noop());
    }

    #[test]
    fn plan_duplicate_is_conflict() {
        let sink = MemoryParamSink::new().with("a.b", "0");
        let specs = vec![
            KernelParamSpec::sysctl("a.b", "1").unwrap(),
            KernelParamSpec::sysctl("a.b", "2").unwrap(),
        ];
        assert!(matches!(
            ReconcilePlan::compute(&specs, &sink),
            Err(KernelParamError::Conflict(_))
        ));
    }
}
