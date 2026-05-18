# Cluster API RBAC managed via Terraform per microservices/cell/policy/cell-boundary.md FM-CB-02 + FM-CB-05.
# UI/kubectl-edit forbidden; lifecycle-manager-worker is the sole authorised writer of cell CRDs.

terraform {
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
  }
}

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = "cell-mgmt-${var.pack}"
}

# Per-pack lifecycle-manager ClusterRole — full authority over Cluster API CRDs
resource "kubernetes_cluster_role" "lifecycle_manager" {
  metadata {
    name = "oya-cell-lifecycle-manager-${var.pack}"
    labels = {
      "oyatie.dev/microservice" = "cell"
      "oyatie.dev/pack"          = var.pack
    }
  }

  rule {
    api_groups = ["cluster.x-k8s.io"]
    resources  = ["clusters", "machinedeployments", "machines", "machinesets"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }

  rule {
    api_groups = [""]
    resources  = ["namespaces"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }

  rule {
    api_groups = ["networking.k8s.io"]
    resources  = ["networkpolicies"]
    verbs      = ["create", "update", "patch", "delete"]
  }
}

# Per-pack scheduler ClusterRole — read-only on cluster state for placement decisions
resource "kubernetes_cluster_role" "scheduler" {
  metadata {
    name = "oya-cell-scheduler-${var.pack}"
  }

  rule {
    api_groups = ["cluster.x-k8s.io"]
    resources  = ["clusters", "machinedeployments"]
    verbs      = ["get", "list", "watch"]
  }

  rule {
    api_groups = [""]
    resources  = ["nodes", "namespaces"]
    verbs      = ["get", "list", "watch"]
  }
}

# Host-pool worker — drain + provision authority
resource "kubernetes_cluster_role" "host_pool" {
  metadata {
    name = "oya-cell-host-pool-${var.pack}"
  }

  rule {
    api_groups = ["cluster.x-k8s.io"]
    resources  = ["machinedeployments", "machines"]
    verbs      = ["get", "list", "watch", "patch"]
  }

  rule {
    api_groups = [""]
    resources  = ["nodes"]
    verbs      = ["get", "list", "watch", "patch"]
  }

  rule {
    api_groups = [""]
    resources  = ["pods"]
    verbs      = ["get", "list", "watch", "delete"]  # eviction-by-delete during drain
  }

  rule {
    api_groups = ["policy"]
    resources  = ["poddisruptionbudgets"]
    verbs      = ["get", "list", "watch"]
  }
}

# Decommission requires 2-person rule — handled at OpenBao JIT layer, not RBAC.
# This RBAC grants lifecycle-manager-worker the authority; OpenBao enforces quorum.

# RoleBindings bind ClusterRoles to per-component ServiceAccounts.
resource "kubernetes_cluster_role_binding" "lifecycle_manager" {
  metadata {
    name = "oya-cell-lifecycle-manager-${var.pack}"
  }
  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.lifecycle_manager.metadata[0].name
  }
  subject {
    kind      = "ServiceAccount"
    name      = "lifecycle-manager-worker"
    namespace = "cell-control-plane"
  }
}

resource "kubernetes_cluster_role_binding" "scheduler" {
  metadata {
    name = "oya-cell-scheduler-${var.pack}"
  }
  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.scheduler.metadata[0].name
  }
  subject {
    kind      = "ServiceAccount"
    name      = "scheduler-worker"
    namespace = "cell-control-plane"
  }
}

resource "kubernetes_cluster_role_binding" "host_pool" {
  metadata {
    name = "oya-cell-host-pool-${var.pack}"
  }
  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.host_pool.metadata[0].name
  }
  subject {
    kind      = "ServiceAccount"
    name      = "host-pool-worker"
    namespace = "cell-control-plane"
  }
}

variable "pack" {
  type        = string
  description = "Pack identifier (pack-kr, pack-eu, …)"
}
