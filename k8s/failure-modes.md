---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cloud
deciders: ops-sre-reliability, axis-cloud, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0121, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-k8s/threat-model.md
  - microservices/cloud-k8s/dpia.md
  - microservices/cloud-k8s/policy/cluster-isolation.md
  - microservices/cloud-k8s/incident-response.md
  - microservices/cloud-k8s/runbooks/
review_cadence: quarterly + after every Sev-1/Sev-2 incident affecting cloud-k8s
doc_status: published
---

# Failure-Mode Catalog (cloud-k8s µservice)

## Purpose

Enumerate failure scenarios on-call must handle, detection signal, immediate mitigation, RCA path, RTO, and recovery runbook. Cross-referenced from `incident-response.md` for severity classification.

## Failure-Mode Index

Each carries: FM-ID; Trigger; Detection; Tenant impact; Severity; Immediate mitigation; RTO; Recovery runbook; Postmortem owner.

## FM-01: kube-apiserver pod outage (single replica)

| Field | Value |
|---|---|
| Trigger | OOM kill; node kernel panic; pod eviction |
| Detection | `kube_apiserver_request_duration_seconds{quantile="0.99"} > 1s` for ≥ 2min OR replica-count drops below quorum |
| Tenant impact | API call latency spike; new pod scheduling pauses |
| Severity | Sev-1 (M01 single-node CP outage = total cluster mutation freeze); Sev-2 after M04 HA |
| Immediate mitigation | Verify HPA on api-proxy fronting kube-apiserver; if single-node M01: restart kube-apiserver via kubeadm; if HA: leader election re-runs |
| RTO | ≤ 5 min (HA) / ≤ 30 min (M01 single) |
| Recovery runbook | `runbooks/control-plane-restore.md` |
| Postmortem owner | axis-cloud + ops-sre-reliability |

## FM-02: etcd quorum loss (2/3 etcd nodes down subsequent-to-M04-completion HA)

| Field | Value |
|---|---|
| Trigger | Cross-AZ network partition; simultaneous 2-node failure |
| Detection | `etcd_server_has_leader == 0` for ≥ 1min |
| Tenant impact | Cluster mutations frozen; reads degraded; data-plane (pods) continues serving |
| Severity | Sev-1 |
| Immediate mitigation | Verify network partition vs hardware failure; if recoverable network: wait for re-join; if permanent: initiate etcd restore from snapshot |
| RTO | ≤ 30 min (snapshot restore) |
| Recovery runbook | `runbooks/etcd-quorum-recovery.md` |
| Postmortem owner | axis-cloud + ops-security |

## FM-03: etcd at-rest encryption key rotation failure

| Field | Value |
|---|---|
| Trigger | KMS unreachable during scheduled key rotation; partial-rotation state |
| Detection | `kube_apiserver_storage_encryption_test_failure_total > 0` |
| Tenant impact | New writes encrypted with new key but reads of recent old-key data fail until KMS recovers |
| Severity | Sev-1 (data-integrity adjacent) |
| Immediate mitigation | Pause rotation; restore KMS; resume rotation; verify both keys still present |
| RTO | ≤ 1h |
| Recovery runbook | `runbooks/control-plane-restore.md` §"Encryption key rotation rollback" |
| Postmortem owner | ops-security + cloud-secrets |

## FM-04: Control-plane node-network partition

| Field | Value |
|---|---|
| Trigger | AZ-level network split; control-plane node isolated |
| Detection | `kubelet_node_ready{node=<cp-node>} == 0` for ≥ 5min |
| Tenant impact | Pods on isolated AZ continue serving; new scheduling decisions degraded |
| Severity | Sev-1 |
| Immediate mitigation | Verify AZ status; cordon isolated AZ; allow workload re-balancing to surviving AZs |
| RTO | ≤ 30 min (cordon + reschedule) |
| Recovery runbook | `runbooks/node-cordon-and-drain.md` |
| Postmortem owner | ops-sre-reliability + axis-cloud |

## FM-05: Worker node failure (kernel panic, hardware, kubelet crashloop)

| Field | Value |
|---|---|
| Trigger | kernel panic; hardware failure; kubelet OOM |
| Detection | Node `Ready=false` for ≥ 5min; node-lifecycle worker emits `NodeFailed` event |
| Tenant impact | Pods on failed node rescheduled (PDB-aware) |
| Severity | Sev-2 |
| Immediate mitigation | node-lifecycle worker auto-cordons + drains via taint-based eviction; HPA replaces pods elsewhere |
| RTO | ≤ 15 min (pods rescheduled) |
| Recovery runbook | `runbooks/node-cordon-and-drain.md` |
| Postmortem owner | ops-sre-reliability |

## FM-06: CNI (Cilium) agent failure on a node

| Field | Value |
|---|---|
| Trigger | Cilium pod crashloop; eBPF program load failure (kernel ABI break post-upgrade) |
| Detection | `cilium_agent_endpoints_total == 0` on node for ≥ 2min |
| Tenant impact | New pod-to-pod connectivity unavailable on affected node; existing connections kept in containerd; no new pods scheduled to node |
| Severity | Sev-2 |
| Immediate mitigation | Restart Cilium DaemonSet on node; if persistent: cordon node; investigate eBPF |
| RTO | ≤ 15 min (Cilium restart) |
| Recovery runbook | `runbooks/cni-rebuild.md` (extension; per cni-cilium chart) |
| Postmortem owner | axis-cloud |

## FM-07: Istio control-plane (istiod) outage

| Field | Value |
|---|---|
| Trigger | istiod pod crashloop; xDS error; upstream Istio CVE |
| Detection | `istio_pilot_xds_pushes_total` rate drops; `pilot_xds_push_time_seconds{quantile="0.99"} > 30s` |
| Tenant impact | New mesh config doesn't propagate; data-plane sidecars retain last-known-good config and continue serving |
| Severity | Sev-2 (data-plane survives) |
| Immediate mitigation | Verify istiod HA replicas; restart failed replica; if persistent: rollback to prior Istio revision (canary upgrade pattern) |
| RTO | ≤ 30 min |
| Recovery runbook | `runbooks/istio-mtls-rotation.md` (extension for control-plane recovery) |
| Postmortem owner | axis-cloud |

## FM-08: Envoy ingress gateway outage (DDoS or pod crash)

| Field | Value |
|---|---|
| Trigger | DDoS attack saturates gateway; pod OOM |
| Detection | `envoy_cluster_upstream_rq_pending_overflow_total > 0` OR ingress `requests_5xx` rate > threshold |
| Tenant impact | External tenant traffic refused or degraded |
| Severity | Sev-1 (external availability) |
| Immediate mitigation | Engage Cloudflare DDoS mitigation; scale ingress replicas via HPA; activate rate-limit; engage provider-edge shield |
| RTO | ≤ 15 min |
| Recovery runbook | `runbooks/ingress-ddos-throttle.md` |
| Postmortem owner | ops-sre-reliability + axis-cloud |

## FM-09: Envoy SNI sniffing / TLS termination misconfig

| Field | Value |
|---|---|
| Trigger | Cert-manager fails to renew tenant cert; Envoy sees cert mismatch with SNI |
| Detection | `envoy_listener_https_downstream_cx_ssl_handshake_errors_total > 0` |
| Tenant impact | Tenant-facing TLS handshakes fail |
| Severity | Sev-2 |
| Immediate mitigation | Force cert-manager reconcile; validate cert-manager + OpenBao reachability; fall back to staging cert if production cert unavailable |
| RTO | ≤ 1h |
| Recovery runbook | `runbooks/envoy-sni-debug.md` |
| Postmortem owner | axis-cloud |

## FM-10: CSI driver failure (block-volume / object / file)

| Field | Value |
|---|---|
| Trigger | CSI controller pod crashloop; backend API outage |
| Detection | `csi_controller_publish_volume_errors_total > 0` for ≥ 5min |
| Tenant impact | New PVC binding paused; existing mounts continue |
| Severity | Sev-2 |
| Immediate mitigation | Verify CSI controller HA; restart failed replica; verify backend (OCI block / object / file) reachability |
| RTO | ≤ 30 min |
| Recovery runbook | `runbooks/csi-rebuild.md` |
| Postmortem owner | axis-cloud + cloud-iac |

## FM-11: kubeadm minor-version upgrade rollback

| Field | Value |
|---|---|
| Trigger | Post-upgrade: pods stuck Pending; control-plane component CrashLoopBackOff; API breaking change discovered |
| Detection | Post-upgrade SLO breach (cluster_health_score declines; scheduling latency spikes) |
| Tenant impact | Cluster operations degraded |
| Severity | Sev-1 |
| Immediate mitigation | Initiate `kubeadm upgrade rollback` to prior version on control-plane; restore etcd from pre-upgrade snapshot; worker nodes downgrade via DaemonSet rolling restart |
| RTO | ≤ 90 min (full cluster downgrade) |
| Recovery runbook | `runbooks/kubeadm-upgrade.md` §"Rollback" |
| Postmortem owner | axis-cloud + ops-sre-reliability |

## FM-12: NetworkPolicy / AuthorizationPolicy regression (cross-tenant leak)

| Field | Value |
|---|---|
| Trigger | LEAN check missed a cross-namespace policy gap; live drift |
| Detection | `cross_namespace_cleartext_attempt_total > 0` OR continuous-state-validator alarms |
| Tenant impact | Potential confidentiality breach (DPIA R-01) |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Apply emergency default-deny override; engage ops-security; freeze affected namespaces; forensic trace |
| RTO | ≤ 5 min freeze; investigation may take 72h+ (GDPR Art. 33) |
| Recovery runbook | `runbooks/security-incident.md` (cross-references `incident-response.md` Sev-1) |
| Postmortem owner | ops-security + axis-cloud |

## FM-13: Cosign signature verification bypass

| Field | Value |
|---|---|
| Trigger | Kyverno admission failure (operator error or supply-chain compromise) |
| Detection | `admission_unsigned_image_admitted_total > 0` |
| Tenant impact | Untrusted image potentially running |
| Severity | Sev-1 (supply chain) |
| Immediate mitigation | Force Kyverno reconcile; identify admitted unsigned pod; terminate; investigate supply chain |
| RTO | ≤ 30 min |
| Recovery runbook | `runbooks/security-incident.md` §"Supply chain bypass" |
| Postmortem owner | ops-security + axis-foundry |

## FM-14: kubernetes-api-proxy outage

| Field | Value |
|---|---|
| Trigger | api-proxy pod crashloop; Cedar evaluator stuck; OpenBao token unreachable |
| Detection | `kubernetes_api_proxy_request_duration_seconds{quantile="0.99"} > 1s` for ≥ 2min |
| Tenant impact | All kubectl + Foundry capability + CI access frozen |
| Severity | Sev-1 |
| Immediate mitigation | Verify api-proxy HA replicas; restart; verify Cedar fragment integrity; verify OpenBao reachability |
| RTO | ≤ 15 min |
| Recovery runbook | `runbooks/control-plane-restore.md` §"API proxy recovery" |
| Postmortem owner | axis-cloud + ops-security |

## FM-15: Persistent Volume backend outage (per pack)

| Field | Value |
|---|---|
| Trigger | OCI Block / Object / File service outage in pack region |
| Detection | `csi_controller_publish_volume_errors_total > 0` + `csi_controller_unpublish_volume_errors_total > 0` |
| Tenant impact | Stateful workloads cannot bind new PVs; existing mounts may degrade |
| Severity | Sev-1 |
| Immediate mitigation | Engage cloud provider; if pack has DR pair: initiate failover per `multi-region.md` |
| RTO | varies by provider; ≤ 1h DR failover (DR pair packs); best-effort otherwise |
| Recovery runbook | `runbooks/csi-rebuild.md` §"Backend outage" |
| Postmortem owner | ops-sre-reliability + cloud-iac |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| kube-apiserver outage (HA) | 5 min | 0 |
| kube-apiserver outage (M01 single) | 30 min | 5 min (last snapshot) |
| etcd quorum loss | 30 min | 5 min |
| etcd encryption key rotation failure | 1h | 0 |
| Control-plane node partition | 30 min | 0 |
| Worker node failure | 15 min | 0 (PDB-aware reschedule) |
| Cilium agent failure | 15 min | 0 |
| Istio control-plane outage | 30 min | 0 |
| Envoy ingress DDoS | 15 min | N/A |
| Envoy TLS misconfig | 1h | N/A |
| CSI driver failure | 30 min | 0 |
| Kubeadm upgrade rollback | 90 min | 5 min (pre-upgrade snapshot) |
| NetworkPolicy regression | 5 min freeze | N/A (breach occurred) |
| Cosign bypass | 30 min | N/A |
| API-proxy outage | 15 min | 0 |
| PV backend outage | 1h (DR pair) / varies | varies |

## SLO on Failure-Detection Pipeline

Meta-SLO: no failure remains undetected longer than its detection window.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency p99 | ≤ 60s | 14.4× burn over 1h |
| Detection coverage (synthetic injection caught within window) | ≥ 99.5% | 6× burn over 6h |
| Two-channel corroboration completion | ≥ 99% within 90s | ticket burn 3d |
| False-positive page rate | ≤ 1/week/on-call | informational |

## References

- `microservices/cloud-k8s/threat-model.md` (each FM maps to one or more STRIDE / LINDDUN threat IDs).
- `microservices/cloud-k8s/dpia.md` (FM-12, FM-13 map to R-01, R-02).
- `microservices/cloud-k8s/incident-response.md` §"Severity Definitions".
- `microservices/cloud-k8s/runbooks/*` (recovery procedures).
- `microservices/cloud-k8s/capacity-model.md`.
- `microservices/cloud-k8s/multi-region.md`.
- Kubernetes failure modes — `kubernetes.io/docs/tasks/administer-cluster/`.
- Istio operations — `istio.io/latest/docs/ops/`.
- Cilium ops — `docs.cilium.io/en/stable/operations/`.
- Google SRE Workbook ch. 12 (Postmortem culture).
