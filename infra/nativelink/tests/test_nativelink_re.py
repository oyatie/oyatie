import json
import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[3]
RE = (ROOT / "infra/nativelink/nativelink-re.k8s.yaml").read_text()
EDGE = (ROOT / "infra/nativelink/nativelink-edge.k8s.yaml").read_text()
CAS = (ROOT / "infra/nativelink/nativelink-cas.k8s.yaml").read_text()


def embedded_json(document: str, key: str) -> dict:
    match = re.search(rf"^  {re.escape(key)}: \|\n(?P<body>(?:    .*\n)+)", document, re.M)
    if not match:
        raise AssertionError(f"missing embedded {key}")
    return json.loads("\n".join(line[4:] for line in match.group("body").splitlines()))


class NativeLinkRemoteExecutionDesiredState(unittest.TestCase):
    def test_v162_config_shape_and_single_slot_sandbox(self):
        scheduler = embedded_json(RE, "scheduler.json")
        worker = embedded_json(RE, "worker.json")
        self.assertEqual(scheduler["schedulers"][0]["name"], "MAIN_SCHEDULER")
        local = worker["workers"][0]["local"]
        self.assertEqual(local["max_inflight_tasks"], 1)
        self.assertIs(local["use_namespaces"], True)
        self.assertIs(local["use_mount_namespace"], True)
        self.assertEqual(local["work_directory"], "/var/lib/nativelink/action-root")
        self.assertIn("cache_metrics", worker["stores"][2])
        self.assertEqual(local["platform_properties"]["cpu_arch"]["values"], ["arm64"])

    def test_workloads_are_pinned_and_do_not_expose_host_credentials(self):
        digest = "sha256:6750ab337eb1835ebe8452ddb76786641a80e23de71d8a5e630469399219b6ea"
        self.assertEqual(RE.count(digest), 2)
        self.assertGreaterEqual(RE.count("automountServiceAccountToken: false"), 2)
        self.assertNotIn("hostPath:", RE)
        self.assertNotIn("privileged: true", RE)
        self.assertNotRegex(RE, r"(?i)(AWS_SECRET|password:)" )
        self.assertIn("kubernetes.io/arch: arm64", RE)
        self.assertIn("hostUsers: false", RE)
        self.assertIn("NL_OTEL_ENDPOINT", RE)

    def test_cas_integrity_and_no_direct_runner_path(self):
        self.assertIn('"verify_size": true', CAS)
        self.assertIn('"verify_hash": true', CAS)
        cas_policy = CAS[CAS.index("name: nativelink-cas-ingress") :]
        self.assertNotIn("oya.io/nativelink-cas-writer", cas_policy)
        self.assertNotIn("oya.io/nativelink-cas-reader", cas_policy)
        self.assertIn("app: nativelink-edge", cas_policy)
        for overlay in ["warm-cache-ro.buckconfig", "warm-cache-rw.buckconfig"]:
            text = (ROOT / "infra/ci/buckconfig" / overlay).read_text()
            self.assertIn("nativelink-edge.oya-ci.svc.cluster.local", text)
            self.assertNotIn("nativelink-cas-", text)

    def test_envoy_roles_have_exact_sans_ports_and_rpc_paths(self):
        expected = {
            50051: "spiffe://oyatie.cell-build/platform/ci-cache-reader",
            50052: "spiffe://oyatie.cell-build/platform/trusted-dev-writer",
            50053: "spiffe://oyatie.cell-build/platform/nativelink-worker",
            50054: "spiffe://oyatie.cell-build/platform/ci-re-input-client",
        }
        for port, identity in expected.items():
            self.assertIn(f"port_value: {port}", EDGE)
            self.assertIn(f'matcher: {{ exact: "{identity}" }}', EDGE)
        routed = re.findall(r'match: \{ ([^}]+) \}, route:', EDGE)
        self.assertTrue(routed)
        self.assertTrue(all('path: "/' in match and "prefix:" not in match for match in routed))
        self.assertNotIn('safe_regex:', EDGE)
        self.assertIn("failure_mode_allow: false", EDGE)
        self.assertIn("status_on_error: { code: Forbidden }", EDGE)
        self.assertIn("request_headers_to_remove: [x-oya-delegated-spiffe-principal", EDGE)

    def test_network_policies_are_role_and_port_exact(self):
        for role, port in [
            ("reader", 50051),
            ("trusted-dev-writer", 50052),
            ("worker-writer", 50053),
            ("re-input-client", 50054),
        ]:
            self.assertIsNotNone(
                re.search(
                    rf"oya.io/nativelink-role: {role}.*?port: {port}",
                    EDGE,
                    re.S,
                ),
                f"missing exact NetworkPolicy leg for {role}",
            )
        self.assertIn("app: nativelink-worker", RE)
        self.assertIn("port: 50061", RE)
        self.assertIn("port: 51052", RE)


if __name__ == "__main__":
    unittest.main()
