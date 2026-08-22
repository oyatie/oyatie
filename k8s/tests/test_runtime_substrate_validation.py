from pathlib import Path
import re
import unittest


REPO_ROOT = Path(__file__).resolve().parents[3]
K8S_ROOT = REPO_ROOT / "cloud" / "cloud-k8s"
MANAGED_K8S_ROOTS = [
    REPO_ROOT / "cloud" / "managed-k8s-cluster-lifecycle",
    REPO_ROOT / "cloud" / "managed-k8s-control-plane-host",
    REPO_ROOT / "cloud" / "managed-k8s-sla-observability",
    REPO_ROOT / "cloud" / "managed-k8s-tenant-quota",
]
BASE = K8S_ROOT / "iac" / "kustomize" / "base"
HELM = K8S_ROOT / "iac" / "helm"

BASE_RESOURCES = [
    "namespace.yaml",
    "openbao-secret-references.yaml",
    "kyverno-cluster-policies.yaml",
    "cilium-network-policies-default-deny.yaml",
    "istio-peerauth-strict.yaml",
]
CSI_CHARTS = {
    "csi-block-volume": {
        "chart_name": "cloud-k8s-csi-block-volume",
        "driver": "block.csi.oyatie.com",
        "backend": "block-volume",
        "secret": "cloud-k8s-csi-block-volume-credentials",
        "repository": "ghcr.io/oyatie/cloud-k8s-csi-block-volume",
        "attach": "true",
        "modes": ["Persistent"],
    },
    "csi-object": {
        "chart_name": "cloud-k8s-csi-object",
        "driver": "object.csi.oyatie.com",
        "backend": "seaweedfs-object",
        "secret": "cloud-k8s-csi-object-credentials",
        "repository": "ghcr.io/oyatie/cloud-k8s-csi-object",
        "attach": "false",
        "modes": ["Persistent", "Ephemeral"],
    },
    "csi-file": {
        "chart_name": "cloud-k8s-csi-file",
        "driver": "file.csi.oyatie.com",
        "backend": "file-storage",
        "secret": "cloud-k8s-csi-file-credentials",
        "repository": "ghcr.io/oyatie/cloud-k8s-csi-file",
        "attach": "false",
        "modes": ["Persistent"],
    },
}
REQUIRED_TEMPLATE_KINDS = {
    "templates/csidriver.yaml": "kind: CSIDriver",
    "templates/deployment.yaml": "kind: Deployment",
    "templates/networkpolicy.yaml": "kind: NetworkPolicy",
    "templates/rbac.yaml": "kind: ClusterRole",
    "templates/serviceaccount.yaml": "kind: ServiceAccount",
}


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def non_comment_documents(text: str):
    docs = []
    current = []
    for line in text.splitlines():
        if line.strip() == "---":
            if current:
                docs.append("\n".join(current))
                current = []
            continue
        if line.strip() and not line.lstrip().startswith("#"):
            current.append(line)
    if current:
        docs.append("\n".join(current))
    return docs


def assembled_bad_terms():
    return [
        "Min" + "IO",
        "Rust" + "FS",
        "Found" + "ry",
        "Jenk" + "ins",
        "oya" + "-vcs",
        "oya" + "-dev-cli",
        "oya " + "gate",
        "oya " + "verify",
        "oya " + "check",
        "M" + "VP",
        "de" + "mo",
        "place" + "holder",
        "st" + "ub",
        "kubectl" + " apply",
        "helm" + " install",
        "helm" + " upgrade",
    ]


class RuntimeSubstrateValidationTest(unittest.TestCase):
    def test_kustomize_base_references_existing_artifacts(self):
        text = read(BASE / "kustomization.yaml")
        self.assertIn("helmGlobals:", text)
        self.assertIn("chartHome: ../../helm", text)
        for resource in BASE_RESOURCES:
            self.assertIn(f"  - {resource}", text)
            self.assertTrue((BASE / resource).is_file(), resource)
        self.assertIn("  - name-suffix.yaml", text)
        self.assertTrue((BASE / "name-suffix.yaml").is_file())
        self.assertIn("components:", text)
        component = BASE.parent / "components" / "storage-classes" / "kustomization.yaml"
        self.assertTrue(component.is_file())
        component_text = read(component)
        for resource in re.findall(r"^  - (storage-class-[^\n]+\.yaml)$", component_text, re.MULTILINE):
            self.assertTrue((component.parent / resource).is_file(), resource)
        for chart in ["cni-cilium", "istio-base", "istiod", "envoy-gateway", *CSI_CHARTS]:
            self.assertIn(f"  - name: {chart}", text)
            self.assertIn(f"valuesFile: ../../helm/{chart}/values.yaml", text)
            self.assertTrue((HELM / chart / "Chart.yaml").is_file(), chart)
            self.assertTrue((HELM / chart / "values.yaml").is_file(), chart)

    def test_base_yaml_documents_have_kubernetes_shape(self):
        for path in [BASE / name for name in BASE_RESOURCES]:
            text = read(path)
            self.assertNotIn("\t", text, path.name)
            self.assertTrue(non_comment_documents(text), path.name)
            for doc in non_comment_documents(text):
                self.assertRegex(doc, r"(?m)^apiVersion:")
                self.assertRegex(doc, r"(?m)^kind:")
                self.assertRegex(doc, r"(?m)^metadata:")
                self.assertRegex(doc, r"(?m)^  name:")
        config_text = read(BASE / "name-suffix.yaml")
        self.assertIn("nameReference:", config_text)
        self.assertIn("kind: ExternalSecret", config_text)

    def test_csi_charts_have_required_values_and_resources(self):
        for chart, expected in CSI_CHARTS.items():
            chart_dir = HELM / chart
            chart_text = read(chart_dir / "Chart.yaml")
            values_text = read(chart_dir / "values.yaml")
            self.assertIn(f"name: {expected['chart_name']}", chart_text)
            self.assertIn(f"name: {expected['driver']}", values_text)
            self.assertIn(f"kind: {expected['backend']}", values_text)
            self.assertIn(f"name: {expected['secret']}", values_text)
            self.assertIn(f"repository: {expected['repository']}", values_text)
            self.assertIn(f"attachRequired: {expected['attach']}", values_text)
            self.assertIn("storageCapacity: true", values_text)
            self.assertIn("enabled: true", values_text)
            for mode in expected["modes"]:
                self.assertIn(mode, values_text)
            for template, marker in REQUIRED_TEMPLATE_KINDS.items():
                text = read(chart_dir / template)
                self.assertEqual(text.count("{{"), text.count("}}"), template)
                self.assertIn(marker, text, template)
                self.assertIn("app.kubernetes.io/part-of: cloud-k8s", read(chart_dir / "templates" / "_helpers.tpl"))
            deployment = read(chart_dir / "templates" / "deployment.yaml")
            self.assertIn("runAsNonRoot: true", deployment)
            self.assertIn("seccompProfile: {type: RuntimeDefault}", deployment)
            self.assertIn("allowPrivilegeEscalation: false", deployment)
            self.assertIn("readOnlyRootFilesystem: true", deployment)
            self.assertIn('capabilities: {drop: ["ALL"]}', deployment)

    def test_runtime_files_do_not_adopt_rejected_or_local_authority_terms(self):
        files = [
            K8S_ROOT / "IP-011-csi-storage-driver-per-backend.md",
            REPO_ROOT / "cloud" / "cloud-storage" / "faqs" / "storage-engineer-faq.md",
            REPO_ROOT / "infra" / "seaweedfs" / "seaweedfs.k8s.yaml",
            *BASE.glob("*.yaml"),
            *(HELM / "csi-block-volume").rglob("*"),
            *(HELM / "csi-object").rglob("*"),
            *(HELM / "csi-file").rglob("*"),
        ]
        terms = [term.lower() for term in assembled_bad_terms()]
        for path in files:
            if not path.is_file():
                continue
            text = read(path).lower()
            for term in terms:
                self.assertNotIn(term, text, f"{path} contains {term}")

    def test_openbao_secret_references_do_not_embed_secret_values(self):
        text = read(BASE / "openbao-secret-references.yaml")
        self.assertEqual(text.count("kind: ExternalSecret"), 3)
        self.assertEqual(text.count("remoteRef:"), 9)
        self.assertIn("kind: ClusterSecretStore", text)
        self.assertNotIn("kind: Secret\n", text)
        self.assertNotIn("stringData:", text)
        for block in re.findall(r"- secretKey:.*?(?=\n    - secretKey:|\n---|\Z)", text, re.DOTALL):
            self.assertIn("remoteRef:", block)
            self.assertIn("property:", block)

    def test_managed_k8s_manifests_keep_provider_runtime_nonclaims_explicit(self):
        for service_root in MANAGED_K8S_ROOTS:
            manifest = read(service_root / "manifest.json")
            self.assertIn('"explicit_non_claims"', manifest, service_root.name)
            self.assertIn("production readiness", manifest.lower(), service_root.name)
            self.assertRegex(
                manifest.lower(),
                r"(provider-live|live .*integration|runtime .*integration|provider .*actuation)",
                service_root.name,
            )


if __name__ == "__main__":
    unittest.main()
