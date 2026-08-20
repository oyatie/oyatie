from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "infra" / "seaweedfs" / "seaweedfs.k8s.yaml"
STORAGE_FAQ = ROOT / "cloud" / "cloud-storage" / "faqs" / "storage-engineer-faq.md"


class SeaweedFsManifestPolicyTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = MANIFEST.read_text(encoding="utf-8")
        cls.storage_faq = STORAGE_FAQ.read_text(encoding="utf-8")

    def test_manifest_has_no_stale_mvp_or_anonymous_hardening_claims(self) -> None:
        banned = [
            "MVP",
            "anonymous-access",
            "anonymous access",
            "hardening follow-on",
            "MinIO",
            "RustFS",
        ]
        combined = self.manifest + "\n" + self.storage_faq
        for term in banned:
            with self.subTest(term=term):
                self.assertNotIn(term, combined)

    def test_bucket_api_requires_secret_backed_non_anonymous_credentials(self) -> None:
        self.assertIn("-s3.config=/tmp/s3.json", self.manifest)
        self.assertIn("SEAWEEDFS_S3_CI_CACHE_ACCESS_KEY", self.manifest)
        self.assertIn("SEAWEEDFS_S3_CI_CACHE_SECRET_KEY", self.manifest)
        self.assertIn("SEAWEEDFS_S3_ADMIN_ACCESS_KEY", self.manifest)
        self.assertIn("SEAWEEDFS_S3_ADMIN_SECRET_KEY", self.manifest)
        self.assertGreaterEqual(self.manifest.count("secretKeyRef:"), 4)
        self.assertNotRegex(self.manifest, r'"name"\s*:\s*"anonymous"')
        self.assertNotRegex(self.manifest, r'"accessKey"\s*:\s*"(?!\$\{SEAWEEDFS_)')
        self.assertNotRegex(self.manifest, r'"secretKey"\s*:\s*"(?!\$\{SEAWEEDFS_)')

    def test_pod_security_is_restricted_and_service_accountless(self) -> None:
        required_lines = [
            "automountServiceAccountToken: false",
            "runAsNonRoot: true",
            "runAsUser: 65532",
            "runAsGroup: 65532",
            "fsGroup: 65532",
            "allowPrivilegeEscalation: false",
            "readOnlyRootFilesystem: true",
            "drop: [ALL]",
            "type: RuntimeDefault",
        ]
        for line in required_lines:
            with self.subTest(line=line):
                self.assertIn(line, self.manifest)

    def test_bucket_api_has_kubernetes_network_policy_boundary(self) -> None:
        self.assertIn("kind: NetworkPolicy", self.manifest)
        self.assertIn("name: seaweedfs-bucket-api-ingress", self.manifest)
        self.assertIn("policyTypes: [Ingress]", self.manifest)
        self.assertIn("oya.io/seaweedfs-client: \"true\"", self.manifest)
        self.assertIn("podSelector:", self.manifest)
        self.assertIn("app: seaweedfs", self.manifest)
        self.assertIn("port: 8333", self.manifest)


if __name__ == "__main__":
    unittest.main()
