//! The full Talos resource type catalog.
//!
//! Mirrors the `Type` constants and `ResourceDefinition()` methods across all
//! of `pkg/machinery/resources/*`: config, network, k8s, cluster, secrets,
//! runtime, hardware, perf, time, etcd, block, siderolink and files. Each entry
//! is a [`ResourceDefinition`] registered into the [`Registry`] at startup, the
//! same way Talos calls `meta.RegisterResourceDefinition` for every kind.
//!
//! [`Registry`]: crate::registry::Registry

use crate::definition::ResourceDefinition;
use crate::namespaces;

/// Build a definition or panic — the catalog is static and must be valid.
fn def(rd: Result<ResourceDefinition, os_kernel::error::Error>) -> ResourceDefinition {
    rd.expect("built-in resource definition must be valid")
}

/// All configuration-domain resource definitions (`config` namespace).
pub fn config_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(ResourceDefinition::builder(
            "MachineConfigs.config.talos.dev",
            namespaces::CONFIG.name(),
        )
        .aliases(["machineconfig", "mc"])
        .sensitive()
        .build()),
        def(ResourceDefinition::builder(
            "MachineTypes.config.talos.dev",
            namespaces::CONFIG.name(),
        )
        .aliases(["machinetype"])
        .print_column("TYPE", "{.type}")
        .build()),
        def(
            ResourceDefinition::builder("V1Alpha1s.config.talos.dev", namespaces::CONFIG.name())
                .alias("v1alpha1")
                .sensitive()
                .build(),
        ),
    ]
}

/// All network-domain resource definitions (`network` namespace).
pub fn network_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(
            ResourceDefinition::builder("NodeAddresses.net.talos.dev", namespaces::NETWORK.name())
                .aliases(["nodeaddress", "addresses"])
                .print_column("ADDRESSES", "{.addresses}")
                .build(),
        ),
        def(
            ResourceDefinition::builder("OperatorSpecs.net.talos.dev", namespaces::NETWORK.name())
                .alias("operatorspec")
                .print_column("OPERATOR", "{.operator}")
                .print_column("LINK", "{.linkName}")
                .build(),
        ),
        def(
            ResourceDefinition::builder("AddressSpecs.net.talos.dev", namespaces::NETWORK.name())
                .alias("addressspec")
                .print_column("ADDRESS", "{.address}")
                .print_column("LINK", "{.linkName}")
                .build(),
        ),
        def(ResourceDefinition::builder(
            "AddressStatuses.net.talos.dev",
            namespaces::NETWORK.name(),
        )
        .aliases(["addressstatus", "address"])
        .print_column("ADDRESS", "{.address}")
        .print_column("LINK", "{.linkName}")
        .build()),
        def(
            ResourceDefinition::builder("RouteSpecs.net.talos.dev", namespaces::NETWORK.name())
                .alias("routespec")
                .print_column("DESTINATION", "{.destination}")
                .print_column("GATEWAY", "{.gateway}")
                .build(),
        ),
        def(
            ResourceDefinition::builder("RouteStatuses.net.talos.dev", namespaces::NETWORK.name())
                .aliases(["routestatus", "route"])
                .print_column("DESTINATION", "{.destination}")
                .print_column("GATEWAY", "{.gateway}")
                .build(),
        ),
        def(
            ResourceDefinition::builder("LinkStatuses.net.talos.dev", namespaces::NETWORK.name())
                .aliases(["linkstatus", "link"])
                .print_column("TYPE", "{.type}")
                .print_column("OPER STATE", "{.operationalState}")
                .build(),
        ),
        def(ResourceDefinition::builder(
            "HostnameStatuses.net.talos.dev",
            namespaces::NETWORK.name(),
        )
        .aliases(["hostnamestatus", "hostname"])
        .print_column("HOSTNAME", "{.hostname}")
        .build()),
        def(
            ResourceDefinition::builder("HostnameSpecs.net.talos.dev", namespaces::NETWORK.name())
                .alias("hostnamespec")
                .print_column("HOSTNAME", "{.hostname}")
                .build(),
        ),
        def(
            ResourceDefinition::builder("ResolverSpecs.net.talos.dev", namespaces::NETWORK.name())
                .alias("resolverspec")
                .print_column("RESOLVERS", "{.dnsServers}")
                .build(),
        ),
        def(ResourceDefinition::builder(
            "ResolverStatuses.net.talos.dev",
            namespaces::NETWORK.name(),
        )
        .aliases(["resolverstatus", "resolvers"])
        .print_column("RESOLVERS", "{.dnsServers}")
        .build()),
    ]
}

/// All Kubernetes-domain resource definitions (`k8s` namespace).
pub fn k8s_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(ResourceDefinition::builder(
            "KubeletConfigs.kubernetes.talos.dev",
            namespaces::K8S.name(),
        )
        .alias("kubeletconfig")
        .build()),
        def(
            ResourceDefinition::builder("StaticPods.kubernetes.talos.dev", namespaces::K8S.name())
                .aliases(["staticpod"])
                .build(),
        ),
        def(ResourceDefinition::builder(
            "StaticPodStatuses.kubernetes.talos.dev",
            namespaces::K8S.name(),
        )
        .aliases(["staticpodstatus"])
        .print_column("READY", "{.ready}")
        .build()),
        def(
            ResourceDefinition::builder("Nodenames.kubernetes.talos.dev", namespaces::K8S.name())
                .alias("nodename")
                .print_column("NODENAME", "{.nodename}")
                .build(),
        ),
        def(
            ResourceDefinition::builder("NodeIPs.kubernetes.talos.dev", namespaces::K8S.name())
                .alias("nodeip")
                .print_column("ADDRESSES", "{.addresses}")
                .build(),
        ),
    ]
}

/// All cluster-domain resource definitions (`cluster` namespace).
pub fn cluster_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(
            ResourceDefinition::builder("Members.cluster.talos.dev", namespaces::CLUSTER.name())
                .alias("member")
                .print_column("MACHINE TYPE", "{.machineType}")
                .print_column("HOSTNAME", "{.hostname}")
                .build(),
        ),
        def(ResourceDefinition::builder(
            "Affiliates.cluster.talos.dev",
            namespaces::CLUSTER.name(),
        )
        .alias("affiliate")
        .print_column("HOSTNAME", "{.hostname}")
        .build()),
        def(ResourceDefinition::builder(
            "Identities.cluster.talos.dev",
            namespaces::CLUSTER.name(),
        )
        .alias("identity")
        .sensitive()
        .build()),
    ]
}

/// All secrets-domain resource definitions (`secrets` namespace, all sensitive).
pub fn secrets_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(ResourceDefinition::builder(
            "EtcdRootSecrets.secrets.talos.dev",
            namespaces::SECRETS.name(),
        )
        .alias("etcdrootsecret")
        .sensitive()
        .build()),
        def(ResourceDefinition::builder(
            "KubernetesRootSecrets.secrets.talos.dev",
            namespaces::SECRETS.name(),
        )
        .alias("k8srootsecret")
        .sensitive()
        .build()),
        def(ResourceDefinition::builder(
            "OSRootSecrets.secrets.talos.dev",
            namespaces::SECRETS.name(),
        )
        .alias("osrootsecret")
        .sensitive()
        .build()),
        def(ResourceDefinition::builder(
            "TrustdCertificates.secrets.talos.dev",
            namespaces::SECRETS.name(),
        )
        .alias("trustdcert")
        .sensitive()
        .build()),
    ]
}

/// All runtime-domain resource definitions (`runtime` namespace).
pub fn runtime_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(
            ResourceDefinition::builder("MetaKeys.meta.cosi.dev", namespaces::RUNTIME.name())
                .alias("metakey")
                .build(),
        ),
        def(ResourceDefinition::builder(
            "MountStatuses.runtime.talos.dev",
            namespaces::RUNTIME.name(),
        )
        .alias("mounts")
        .print_column("SOURCE", "{.source}")
        .print_column("TARGET", "{.target}")
        .print_column("FILESYSTEM TYPE", "{.filesystemType}")
        .build()),
        def(
            ResourceDefinition::builder("Services.v1alpha1.talos.dev", namespaces::RUNTIME.name())
                .aliases(["service", "svc"])
                .print_column("RUNNING", "{.running}")
                .print_column("HEALTHY", "{.healthy}")
                .build(),
        ),
        def(ResourceDefinition::builder(
            "MachineStatuses.runtime.talos.dev",
            namespaces::RUNTIME.name(),
        )
        .alias("machinestatus")
        .print_column("STAGE", "{.stage}")
        .print_column("READY", "{.status.ready}")
        .build()),
    ]
}

/// All hardware-domain resource definitions (`hardware` namespace).
pub fn hardware_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(ResourceDefinition::builder(
            "Processors.hardware.talos.dev",
            namespaces::HARDWARE.name(),
        )
        .alias("processor")
        .print_column("CORES", "{.coreCount}")
        .build()),
        def(ResourceDefinition::builder(
            "MemoryModules.hardware.talos.dev",
            namespaces::HARDWARE.name(),
        )
        .alias("memorymodule")
        .print_column("SIZE MB", "{.sizeMiB}")
        .build()),
        def(ResourceDefinition::builder(
            "SystemInformations.hardware.talos.dev",
            namespaces::HARDWARE.name(),
        )
        .alias("systeminformation")
        .build()),
    ]
}

/// All perf-domain resource definitions (`perf` namespace).
pub fn perf_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(
            ResourceDefinition::builder("CPUStats.perf.talos.dev", namespaces::PERF.name())
                .alias("cpu")
                .build(),
        ),
        def(
            ResourceDefinition::builder("MemoryStats.perf.talos.dev", namespaces::PERF.name())
                .alias("memory")
                .build(),
        ),
    ]
}

/// All time-domain resource definitions (in the `runtime` namespace).
pub fn time_definitions() -> Vec<ResourceDefinition> {
    vec![def(ResourceDefinition::builder(
        "TimeStatuses.v1alpha1.talos.dev",
        namespaces::RUNTIME.name(),
    )
    .aliases(["timestatus", "time"])
    .print_column("SYNCED", "{.synced}")
    .build())]
}

/// All etcd-domain resource definitions (`etcd` namespace).
pub fn etcd_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(
            ResourceDefinition::builder("EtcdMembers.etcd.talos.dev", namespaces::ETCD.name())
                .alias("etcdmember")
                .print_column("MEMBERS", "{.memberId}")
                .build(),
        ),
        def(
            ResourceDefinition::builder("EtcdConfigs.etcd.talos.dev", namespaces::ETCD.name())
                .alias("etcdconfig")
                .build(),
        ),
    ]
}

/// All block-domain resource definitions (`runtime` namespace).
pub fn block_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(ResourceDefinition::builder(
            "MountRequests.block.talos.dev",
            namespaces::RUNTIME.name(),
        )
        .print_column("Volume", "{.volumeID}")
        .print_column("Parent", "{.parentID}")
        .print_column("Requesters", "{.requesters}")
        .build()),
        def(ResourceDefinition::builder(
            "VolumeMountRequests.block.talos.dev",
            namespaces::RUNTIME.name(),
        )
        .print_column("Volume ID", "{.volumeID}")
        .print_column("Requester", "{.requester}")
        .build()),
        def(ResourceDefinition::builder(
            "MountStatuses.block.talos.dev",
            namespaces::RUNTIME.name(),
        )
        .print_column("Source", "{.source}")
        .print_column("Target", "{.target}")
        .print_column("Filesystem", "{.filesystem}")
        .print_column("Volume", "{.spec.volumeID}")
        .build()),
        def(ResourceDefinition::builder(
            "VolumeMountStatuses.block.talos.dev",
            namespaces::RUNTIME.name(),
        )
        .print_column("Volume ID", "{.volumeID}")
        .print_column("Requester", "{.requester}")
        .print_column("Target", "{.target}")
        .build()),
        def(
            ResourceDefinition::builder("Disks.block.talos.dev", namespaces::RUNTIME.name())
                .alias("disk")
                .print_column("SIZE", "{.size}")
                .print_column("MODEL", "{.model}")
                .build(),
        ),
        def(ResourceDefinition::builder(
            "DiscoveredVolumes.block.talos.dev",
            namespaces::RUNTIME.name(),
        )
        .aliases(["discoveredvolume", "volumes"])
        .print_column("TYPE", "{.type}")
        .print_column("LABEL", "{.partitionLabel}")
        .build()),
        def(ResourceDefinition::builder(
            "VolumeStatuses.block.talos.dev",
            namespaces::RUNTIME.name(),
        )
        .aliases(["volumestatus"])
        .print_column("PHASE", "{.phase}")
        .build()),
    ]
}

/// All SideroLink-domain resource definitions (`siderolink` namespace).
pub fn siderolink_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(ResourceDefinition::builder(
            "Tunnels.siderolink.talos.dev",
            namespaces::SIDEROLINK.name(),
        )
        .alias("tunnel")
        .build()),
        def(ResourceDefinition::builder(
            "Configs.siderolink.talos.dev",
            namespaces::SIDEROLINK.name(),
        )
        .alias("siderolinkconfig")
        .sensitive()
        .build()),
    ]
}

/// All files-domain resource definitions (`files` namespace).
pub fn files_definitions() -> Vec<ResourceDefinition> {
    vec![
        def(ResourceDefinition::builder(
            "EtcFileStatuses.files.talos.dev",
            namespaces::FILES.name(),
        )
        .aliases(["etcfilestatus"])
        .build()),
        def(ResourceDefinition::builder(
            "CRIRegistryConfigs.cri.talos.dev",
            namespaces::FILES.name(),
        )
        .alias("criregistryconfig")
        .sensitive()
        .build()),
    ]
}

/// Every built-in resource definition across all domains, in a stable order.
pub fn all_definitions() -> Vec<ResourceDefinition> {
    let mut out = Vec::new();
    out.extend(config_definitions());
    out.extend(runtime_definitions());
    out.extend(network_definitions());
    out.extend(k8s_definitions());
    out.extend(cluster_definitions());
    out.extend(secrets_definitions());
    out.extend(hardware_definitions());
    out.extend(perf_definitions());
    out.extend(time_definitions());
    out.extend(etcd_definitions());
    out.extend(block_definitions());
    out.extend(siderolink_definitions());
    out.extend(files_definitions());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_definition_is_valid_and_unique() {
        let defs = all_definitions();
        assert!(
            defs.len() >= 30,
            "expected a substantial catalog, got {}",
            defs.len()
        );
        let mut types = BTreeSet::new();
        for d in &defs {
            assert!(
                types.insert(d.type_name().to_string()),
                "duplicate type {}",
                d.type_name()
            );
        }
    }

    #[test]
    fn aliases_do_not_collide_across_catalog() {
        let defs = all_definitions();
        let mut aliases = BTreeSet::new();
        for d in &defs {
            for a in d.aliases() {
                assert!(aliases.insert(a.clone()), "duplicate alias {a}");
            }
        }
    }

    #[test]
    fn all_default_namespaces_are_known() {
        for d in all_definitions() {
            assert!(
                namespaces::is_known(d.default_namespace()),
                "unknown namespace {} for {}",
                d.default_namespace(),
                d.type_name()
            );
        }
    }

    #[test]
    fn secrets_are_all_sensitive() {
        for d in secrets_definitions() {
            assert!(
                d.sensitivity().is_sensitive(),
                "{} should be sensitive",
                d.type_name()
            );
        }
    }

    #[test]
    fn machine_config_is_sensitive_with_aliases() {
        let mc = config_definitions()
            .into_iter()
            .find(|d| d.kind() == "MachineConfigs")
            .unwrap();
        assert!(mc.sensitivity().is_sensitive());
        assert!(mc.aliases().contains(&"mc".to_string()));
    }

    #[test]
    fn network_operator_derived_specs_are_registered() {
        let types: BTreeSet<_> = network_definitions()
            .into_iter()
            .map(|d| d.type_name().to_string())
            .collect();

        for type_name in [
            "OperatorSpecs.net.talos.dev",
            "AddressSpecs.net.talos.dev",
            "RouteSpecs.net.talos.dev",
            "HostnameSpecs.net.talos.dev",
            "ResolverSpecs.net.talos.dev",
        ] {
            assert!(
                types.contains(type_name),
                "{type_name} should be registered"
            );
        }
    }

    #[test]
    fn block_mount_status_resource_definitions_match_talos_v113() {
        let reg = crate::Registry::with_defaults();

        let runtime_mount = reg
            .get("MountStatuses.runtime.talos.dev")
            .expect("runtime mount status remains registered");
        assert_eq!(runtime_mount.default_namespace(), "runtime");
        assert_eq!(runtime_mount.aliases(), &["mounts".to_string()]);

        let block_mount = reg
            .get("MountStatuses.block.talos.dev")
            .expect("block mount status should be registered");
        assert_eq!(block_mount.default_namespace(), "runtime");
        assert!(block_mount.aliases().is_empty());
        assert_eq!(
            block_mount
                .print_columns()
                .iter()
                .map(super::super::printcolumns::PrintColumn::name)
                .collect::<Vec<_>>(),
            vec!["Source", "Target", "Filesystem", "Volume"]
        );

        let volume_mount = reg
            .get("VolumeMountStatuses.block.talos.dev")
            .expect("volume mount status should be registered");
        assert_eq!(volume_mount.default_namespace(), "runtime");
        assert!(volume_mount.aliases().is_empty());
        assert_eq!(
            volume_mount
                .print_columns()
                .iter()
                .map(super::super::printcolumns::PrintColumn::name)
                .collect::<Vec<_>>(),
            vec!["Volume ID", "Requester", "Target"]
        );

        assert_eq!(
            reg.resolve("mounts").unwrap().type_name(),
            "MountStatuses.runtime.talos.dev"
        );
        assert_eq!(
            reg.resolve("MountStatuses.block.talos.dev")
                .unwrap()
                .type_name(),
            "MountStatuses.block.talos.dev"
        );
    }
}
