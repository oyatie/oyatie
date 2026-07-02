#!/usr/bin/env python3
"""Fixture-2 revoke-then-check harness for the owned Policy IR benchmark (G4 evidence).

Produces the sub-60s revocation-evidence artifact for Fixture 2 (>=3-hop
cross-company delegation ReBAC): per-engine consistency-semantics analysis
(zookie / snapshot / read-after-write / bundle-activation / offline-revocation
models) plus topology-documented wall-clock revoke-then-check measurements,
recording per-engine time-to-consistent-deny under the 60s bound.

Honesty contract (N2/N3, no fabrication):
- Measurements run against SINGLE-HOST IN-PROCESS REFERENCE ADAPTERS that
  execute each engine's vendor-documented consistency/propagation semantics
  over the real Fixture-2 relation graph loaded from the frozen FixtureSuite.
  The revoke-then-check loop is a genuine wall-clock poller
  (time.monotonic()); nothing is back-computed or fabricated.
- Every record documents the measurement topology (node counts, cache layers,
  distribution path, parameters, environment class) alongside the numbers and
  carries an N3 emulation disclosure naming the in-process reference shape.
  Production-topology extrapolation lives in the consistency-semantics
  analysis, never in the numbers.
- Every record pins the frozen Fixture-2 content-hash
  (fixture_2_workload_contract.content_hash.value) and the frozen rubric
  content-hash, per the rubric grade-pinning rule.

Scope bindings (fixture_2_workload_contract.revocation_scenario.scope_bindings):
- measured: cedar, spicedb, openfga, opa_rego (fixture_scope=full) + biscuit
  (delegation_only subset IS Fixture 2, so its revocation surface is measured).
- analysis-only: cel (statutory_only subset exercises no Fixture-2 propagation
  surface; rubric G4 bounded_scope_rule).

Stdlib only. Writes evidence/policy-ir-benchmark/fixture-2-revocation-evidence.json.
"""
from __future__ import annotations

import copy
import datetime as _dt
import hashlib
import json
import math
import platform
import sys
import time
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SUITE_PATH = REPO_ROOT / "specs" / "policy-ir-benchmark-fixture-suite.json"
RUBRIC_PATH = REPO_ROOT / "specs" / "policy-ir-benchmark-rubric.json"
EVIDENCE_PATH = (
    REPO_ROOT / "evidence" / "policy-ir-benchmark" / "fixture-2-revocation-evidence.json"
)

POLL_INTERVAL_S = 0.02
CONSECUTIVE_DENIES_REQUIRED = 3
MEASUREMENT_TIMEOUT_S = 70.0


def fail(message: str) -> NoReturn:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot load {path}: {exc}")


def canonical_bytes(document: object) -> bytes:
    return json.dumps(
        document, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def fixture_2_hash(suite: dict) -> str:
    subtree = copy.deepcopy(suite["fixture_2_workload_contract"])
    subtree["content_hash"]["value"] = ""
    return "fixture2-v1:sha256:" + hashlib.sha256(canonical_bytes(subtree)).hexdigest()


def whole_doc_hash(document: dict, prefix: str) -> str:
    candidate = copy.deepcopy(document)
    candidate["content_hash"]["value"] = ""
    return f"{prefix}sha256:" + hashlib.sha256(canonical_bytes(candidate)).hexdigest()


# --- Fixture-2 relation graph -------------------------------------------------


class Fixture2Graph:
    """Reachability evaluation over the frozen Fixture-2 relation tuples.

    A principal holds the fixture action on a resource iff a directed path of
    live tuples (subject -> object) connects principal to resource, per the
    fixture's permission_model evaluation rule.
    """

    def __init__(self, tuples: list[dict]):
        self._tuples = {t["tuple_id"]: t for t in tuples}
        self._revoked: set[str] = set()

    @staticmethod
    def _key(entity: dict) -> tuple[str, str]:
        return (entity["entity_type"], entity["entity_id"])

    def live_tuples(self) -> list[dict]:
        return [t for tid, t in self._tuples.items() if tid not in self._revoked]

    def revoke(self, tuple_id: str) -> None:
        if tuple_id not in self._tuples:
            fail(f"revoked_tuple_id {tuple_id!r} not in fixture tuple set")
        self._revoked.add(tuple_id)

    def check(self, principal: dict, resource: dict, revoked_view: set[str]) -> bool:
        edges: dict[tuple[str, str], list[tuple[str, str]]] = {}
        for tuple_id, t in self._tuples.items():
            if tuple_id in revoked_view:
                continue
            edges.setdefault(self._key(t["subject"]), []).append(self._key(t["object"]))
        start, goal = self._key(principal), self._key(resource)
        frontier, seen = [start], {start}
        while frontier:
            node = frontier.pop()
            if node == goal:
                return True
            for nxt in edges.get(node, ()):
                if nxt not in seen:
                    seen.add(nxt)
                    frontier.append(nxt)
        return False


# --- per-engine reference consistency models ---------------------------------
#
# Each adapter implements the engine's vendor-documented propagation semantics
# as a pure function of real wall-clock time: writes record monotonic
# timestamps, and check() decides which state is visible at call time exactly
# the way the documented mechanism does (revision quantization, cache TTL,
# poll-boundary bundle activation, revocation-list push). The measurement loop
# then genuinely polls until a consistent deny is observed.


class EngineAdapter:
    engine: str
    adapter_shape: str
    consistency_mode: str

    def __init__(self, graph: Fixture2Graph, params: dict[str, int]):
        self.graph = graph
        self.params = params
        self.start = time.monotonic()

    def token(self) -> str | None:
        raise NotImplementedError

    def policy_version(self) -> str:
        raise NotImplementedError

    def revoke(self, tuple_id: str) -> None:
        raise NotImplementedError

    def check(self, principal: dict, resource: dict) -> bool:
        raise NotImplementedError

    def topology(self) -> dict:
        raise NotImplementedError


class SpiceDbAdapter(EngineAdapter):
    """MVCC revision store with ZedTokens and quantized-revision reads.

    Documented model: writes create a new revision and return a ZedToken;
    reads at `minimize_latency` evaluate at a quantized revision (staleness
    bounded by the quantization window); `at_least_as_fresh(zedtoken)` gives
    causal read-after-write. The revocation SLO path measured here is the
    cache-friendly minimize_latency mode — the worst documented staleness.
    """

    engine = "spicedb"
    adapter_shape = "in-process-reference-consistency-model+revoke-then-check-poller"
    consistency_mode = "minimize_latency (quantized revision; worst documented staleness)"

    def __init__(self, graph: Fixture2Graph, params: dict[str, int]):
        super().__init__(graph, params)
        self._revision = 1
        self._writes: list[tuple[float, int, set[str]]] = [(self.start, 1, set())]

    def token(self) -> str:
        return f"zedtoken-rev-{self._revision}"

    def policy_version(self) -> str:
        return "1.0.0"

    def revoke(self, tuple_id: str) -> None:
        self.graph.revoke(tuple_id)
        self._revision += 1
        revoked = {t for t in self.graph._revoked}
        self._writes.append((time.monotonic(), self._revision, revoked))

    def _quantized_view(self) -> set[str]:
        window = self.params["quantization_window_ms"] / 1000.0
        now = time.monotonic()
        boundary = self.start + math.floor((now - self.start) / window) * window
        visible: set[str] = set()
        for written_at, _rev, revoked in self._writes:
            if written_at <= boundary:
                visible = revoked
        return visible

    def check(self, principal: dict, resource: dict) -> bool:
        return self.graph.check(principal, resource, self._quantized_view())

    def topology(self) -> dict:
        return {
            "environment": "single-host in-process reference topology",
            "node_counts": {"spicedb_nodes": 1, "datastore_nodes": 1, "dispatch_cache_nodes": 1},
            "cache_layers": ["quantized-revision read window (dispatch/datastore snapshot selection)"],
            "distribution_path": "WriteRelationships(delete t3) -> new revision + ZedToken -> quantized-revision read window elapses -> CheckPermission(minimize_latency) evaluates at post-revocation revision",
            "parameters": {"quantization_window_ms": self.params["quantization_window_ms"]},
            "reference_model_note": "Implements the documented ZedToken/quantized-revision semantics in-process; production staleness scales with the deployed quantization window (default 5s) and replication, both orders of magnitude inside the 60s bound.",
        }


class OpenFgaAdapter(EngineAdapter):
    """Synchronous tuple writes with a check-query cache TTL.

    Documented model: tuple writes are committed synchronously; Check may be
    served from the check-query cache until its TTL expires unless the request
    asks for HIGHER_CONSISTENCY. The measured SLO path is the cached default.
    """

    engine = "openfga"
    adapter_shape = "in-process-reference-consistency-model+revoke-then-check-poller"
    consistency_mode = "default consistency with check-query cache (worst documented staleness)"

    def __init__(self, graph: Fixture2Graph, params: dict[str, int]):
        super().__init__(graph, params)
        self._cache: dict[str, tuple[float, bool]] = {}
        self._model_id = "01JZK9V0REFMODELFIXTURE2AA"

    def token(self) -> str:
        return f"authorization_model_id:{self._model_id}"

    def policy_version(self) -> str:
        return "1.0.0"

    def revoke(self, tuple_id: str) -> None:
        self.graph.revoke(tuple_id)  # synchronous write; cache entries keep serving until TTL

    def check(self, principal: dict, resource: dict) -> bool:
        ttl = self.params["check_query_cache_ttl_ms"] / 1000.0
        key = json.dumps([principal, resource], sort_keys=True)
        now = time.monotonic()
        cached = self._cache.get(key)
        if cached is not None and now - cached[0] < ttl:
            return cached[1]
        result = self.graph.check(principal, resource, set(self.graph._revoked))
        self._cache[key] = (now, result)
        return result

    def topology(self) -> dict:
        return {
            "environment": "single-host in-process reference topology",
            "node_counts": {"openfga_nodes": 1, "datastore_nodes": 1},
            "cache_layers": ["check-query cache (TTL-bounded)"],
            "distribution_path": "Write(delete t3) committed synchronously -> check-query cache entry ages out (TTL) -> Check(default consistency) recomputes against post-revocation tuples",
            "parameters": {"check_query_cache_ttl_ms": self.params["check_query_cache_ttl_ms"]},
            "reference_model_note": "Implements the documented synchronous-write + check-query-cache TTL semantics in-process; HIGHER_CONSISTENCY bypasses the cache entirely. Production default TTL (10s when enabled) is well inside the 60s bound.",
        }


class BundleActivationAdapter(EngineAdapter):
    """Shared model for compile-time bundle distribution (cedar, opa_rego).

    Documented model: a revocation ships as a new policy/entity bundle version
    through the author->validate->sign->semver->distribute->compose->audit
    pipeline; PDP nodes poll the distribution point and atomically activate
    the new bundle. Staleness = publish pipeline latency + poll interval.
    """

    def __init__(self, graph: Fixture2Graph, params: dict[str, int]):
        super().__init__(graph, params)
        self._published: list[tuple[float, str, set[str]]] = [(self.start, "1.0.0", set())]
        self._pending_version = "1.0.0"

    def _publish_key(self) -> str:
        raise NotImplementedError

    def _poll_key(self) -> str:
        raise NotImplementedError

    def policy_version(self) -> str:
        return self._active()[1]

    def revoke(self, tuple_id: str) -> None:
        self.graph.revoke(tuple_id)
        publish_at = time.monotonic() + self.params[self._publish_key()] / 1000.0
        self._pending_version = "1.1.0"
        self._published.append((publish_at, "1.1.0", set(self.graph._revoked)))

    def _active(self) -> tuple[float, str, set[str]]:
        interval = self.params[self._poll_key()] / 1000.0
        now = time.monotonic()
        last_poll = self.start + math.floor((now - self.start) / interval) * interval
        active = self._published[0]
        for published_at, version, revoked in self._published:
            if published_at <= last_poll:
                active = (published_at, version, revoked)
        return active

    def check(self, principal: dict, resource: dict) -> bool:
        return self.graph.check(principal, resource, self._active()[2])


class CedarAdapter(BundleActivationAdapter):
    engine = "cedar"
    adapter_shape = "in-process-reference-consistency-model+revoke-then-check-poller"
    consistency_mode = "signed PolicyBundle semver activation via PDP poll loop"

    def _publish_key(self) -> str:
        return "publish_pipeline_latency_ms"

    def _poll_key(self) -> str:
        return "pdp_poll_interval_ms"

    def token(self) -> str | None:
        return None  # no per-decision token; bundle semver is the version surface

    def topology(self) -> dict:
        return {
            "environment": "single-host in-process reference topology",
            "node_counts": {"pdp_nodes": 1, "bundle_distribution_nodes": 1},
            "cache_layers": ["PDP-resident active bundle (atomic swap on poll)"],
            "distribution_path": "author revocation -> validate -> sign -> semver bump (1.0.0 -> 1.1.0) -> distribute -> PDP poll boundary -> compose/activate -> is_authorized evaluates post-revocation entities",
            "parameters": {
                "publish_pipeline_latency_ms": self.params["publish_pipeline_latency_ms"],
                "pdp_poll_interval_ms": self.params["pdp_poll_interval_ms"],
            },
            "reference_model_note": "Implements the owned PolicyBundle mechanics (author->validate->sign->semver->distribute->compose->audit) in-process; staleness is publish latency + one poll interval, a deployment parameter held well inside the 60s bound by construction.",
        }


class OpaAdapter(BundleActivationAdapter):
    engine = "opa_rego"
    adapter_shape = "in-process-reference-consistency-model+revoke-then-check-poller"
    consistency_mode = "signed bundle revision activation via bundle poll loop"

    def _publish_key(self) -> str:
        return "bundle_publish_latency_ms"

    def _poll_key(self) -> str:
        return "bundle_poll_interval_ms"

    def token(self) -> str:
        return f"bundle-revision:{self._active()[1]}"

    def topology(self) -> dict:
        return {
            "environment": "single-host in-process reference topology",
            "node_counts": {"opa_nodes": 1, "bundle_server_nodes": 1},
            "cache_layers": ["OPA-resident activated bundle (atomic swap on successful poll)"],
            "distribution_path": "publish signed bundle revision (data change removing t3) -> OPA bundle poll boundary -> atomic activation -> decision rule evaluates post-revocation data",
            "parameters": {
                "bundle_publish_latency_ms": self.params["bundle_publish_latency_ms"],
                "bundle_poll_interval_ms": self.params["bundle_poll_interval_ms"],
            },
            "reference_model_note": "Implements OPA's documented bundle polling/activation semantics in-process with a fixed poll interval (production uses jittered min/max delay, default 60s/120s — the SLO therefore requires configuring polling <= ~30s or bundle push; see consistency analysis).",
        }


class BiscuitAdapter(EngineAdapter):
    """Offline-attenuation tokens with revocation-id list distribution.

    Documented model: tokens are verified offline; revocation is by
    distributing revoked block ids to authorizers, which reject any token
    carrying a listed revocation id. Staleness = revocation-list push latency.
    """

    engine = "biscuit"
    adapter_shape = "in-process-reference-consistency-model+revoke-then-check-poller"
    consistency_mode = "authorizer-side revocation-id list, push-distributed"

    def __init__(self, graph: Fixture2Graph, params: dict[str, int]):
        super().__init__(graph, params)
        self._revocation_pushes: list[tuple[float, str]] = []
        self._token_block_ids = {"t3-delegated-viewer": "rev-id-block-t3"}

    def token(self) -> str:
        delivered = self._delivered_ids()
        digest = hashlib.sha256(canonical_bytes(sorted(delivered))).hexdigest()[:16]
        return f"revocation-id-set:sha256:{digest}"

    def policy_version(self) -> str:
        return "1.0.0"

    def revoke(self, tuple_id: str) -> None:
        self.graph.revoke(tuple_id)
        deliver_at = time.monotonic() + self.params["revocation_push_latency_ms"] / 1000.0
        self._revocation_pushes.append((deliver_at, self._token_block_ids[tuple_id]))

    def _delivered_ids(self) -> set[str]:
        now = time.monotonic()
        return {rid for deliver_at, rid in self._revocation_pushes if deliver_at <= now}

    def check(self, principal: dict, resource: dict) -> bool:
        delivered = self._delivered_ids()
        revoked_view = {
            tuple_id
            for tuple_id, block_id in self._token_block_ids.items()
            if block_id in delivered
        }
        return self.graph.check(principal, resource, revoked_view)

    def topology(self) -> dict:
        return {
            "environment": "single-host in-process reference topology",
            "node_counts": {"authorizer_nodes": 1, "revocation_list_distribution_nodes": 1},
            "cache_layers": ["authorizer-resident revocation-id set"],
            "distribution_path": "revoke delegation token block -> append block revocation id to revocation list -> push to authorizer -> authorize() rejects token carrying listed revocation id -> deny",
            "parameters": {"revocation_push_latency_ms": self.params["revocation_push_latency_ms"]},
            "reference_model_note": "Implements Biscuit's documented revocation-id semantics in-process: offline tokens cannot be recalled, so time-to-consistent-deny is exactly revocation-list distribution latency to every authorizer.",
        }


ADAPTERS: list[tuple[type[EngineAdapter], dict[str, int]]] = [
    (CedarAdapter, {"publish_pipeline_latency_ms": 200, "pdp_poll_interval_ms": 1000}),
    (SpiceDbAdapter, {"quantization_window_ms": 500}),
    (OpenFgaAdapter, {"check_query_cache_ttl_ms": 500}),
    (OpaAdapter, {"bundle_publish_latency_ms": 200, "bundle_poll_interval_ms": 1000}),
    (BiscuitAdapter, {"revocation_push_latency_ms": 300}),
]


# --- consistency-semantics analysis (all six engines) -------------------------

CONSISTENCY_ANALYSIS: dict[str, dict] = {
    "cedar": {
        "model_kind": "bundle-activation",
        "read_after_write": "Within one PDP process, activation is an atomic bundle swap: every is_authorized call after activation evaluates the new bundle (read-after-write at activation granularity). There is no per-decision consistency token; the bundle semver is the version surface, mirrored by the owned decision model's policy_version.",
        "staleness_model": "Bounded staleness = authoring/validate/sign/publish pipeline latency + PDP poll interval. Both are deployment parameters of the owned signed-PolicyBundle mechanics (author->validate->sign->semver->distribute->compose->audit); neither grows with policy-set size at enforcement time (compile-time-over-runtime propagation).",
        "sub_60s_assessment": "MEETS BOUND BY CONFIGURATION: with publish latency in seconds and poll interval <= 30s, worst-case revocation-to-enforcement is well under 60s deterministically; the mechanism is a hard bound (poll boundary), not a probabilistic cache decay.",
        "citations": [
            "docs/ideas/policy-pack-substrate.md (signed PolicyBundle lifecycle: author->validate->sign->semver->distribute->compose->audit)",
            "iam/core/policy-cedar-domain (owned decision model: policy_version zookies, decision_id provenance)",
            "https://docs.cedarpolicy.com/ (Cedar evaluates policies against provided entities per request; policy distribution/activation is host-application responsibility)",
        ],
    },
    "spicedb": {
        "model_kind": "zookie-snapshot-mvcc",
        "read_after_write": "CheckPermission with at_least_as_fresh(ZedToken) or fully_consistent gives causal read-after-write against the revocation write's revision. ZedTokens map directly onto the owned model's policy_version zookies (New-Enemy prevention per Zanzibar).",
        "staleness_model": "minimize_latency evaluates at a quantized revision: staleness is bounded by the revision quantization window (default 5s) plus datastore replication lag. Cache entries are revision-keyed, so quantization is the documented worst case.",
        "sub_60s_assessment": "MEETS BOUND NATIVELY: worst documented staleness (quantization window, default 5s) is an order of magnitude inside 60s; zookie-carrying checks are immediately consistent.",
        "citations": [
            "https://authzed.com/docs/spicedb/concepts/consistency (ZedTokens, minimize_latency / at_least_as_fresh / at_exact_snapshot / fully_consistent)",
            "https://authzed.com/docs/spicedb/concepts/datastores (quantization window / revision selection)",
            "Zanzibar: Google's Consistent, Global Authorization System (USENIX ATC 2019) — zookies, New-Enemy problem",
        ],
    },
    "openfga": {
        "model_kind": "read-after-write-with-cache-ttl",
        "read_after_write": "Tuple writes commit synchronously to the store; Check with HIGHER_CONSISTENCY bypasses caches and observes the write immediately. There is no per-decision zookie; the authorization-model id pins the schema version, not the tuple snapshot.",
        "staleness_model": "With the check-query cache enabled, default-consistency Checks may serve cached results until TTL expiry (documented default 10s when enabled; disabled by default). Staleness is therefore TTL-bounded, not replication-bounded, on a single cluster.",
        "sub_60s_assessment": "MEETS BOUND NATIVELY: worst documented staleness (check-query cache TTL) defaults an order of magnitude inside 60s, and HIGHER_CONSISTENCY provides an immediate path.",
        "citations": [
            "https://openfga.dev/docs/interacting/consistency (consistency preferences, HIGHER_CONSISTENCY, cache interaction)",
            "https://openfga.dev/docs/getting-started/setup-openfga/configuration (check query cache + TTL)",
        ],
    },
    "opa_rego": {
        "model_kind": "bundle-activation",
        "read_after_write": "Bundle activation is atomic per OPA instance: once a polled bundle activates, all queries evaluate the new revision. The bundle revision string is the consistency surface; no per-decision token exists.",
        "staleness_model": "Staleness = bundle publish latency + polling interval (documented default min 60s / max 120s, configurable to seconds) per instance; status API reports activated revisions for fleet-wide convergence tracking. Discovery/bundle signing preserves the signed-distribution chain.",
        "sub_60s_assessment": "MEETS BOUND BY CONFIGURATION: default polling (60-120s) EXCEEDS the bound; meeting sub-60s requires configuring polling <= ~30s or push-mode distribution. Deterministic once configured; recorded as a configuration obligation, not a capability gap.",
        "citations": [
            "https://www.openpolicyagent.org/docs/latest/management-bundles/ (bundle polling min/max delay, revision, signing, activation)",
            "https://www.openpolicyagent.org/docs/latest/status/ (activated-revision reporting)",
        ],
    },
    "cel": {
        "model_kind": "compile-time-expression-set-distribution",
        "read_after_write": "CEL has no runtime policy store: an expression set is compiled and shipped as part of the embedding application's deployment. Within a process, swapping the compiled expression set is atomic; read-after-write holds at swap granularity.",
        "staleness_model": "Propagation is exactly the host's artifact rollout (redeploy/config push). CEL itself contributes zero staleness machinery — the bound is inherited entirely from the embedding platform's distribution path.",
        "sub_60s_assessment": "ANALYSIS-ONLY FOR FIXTURE 2 (rubric G4 bounded_scope_rule): CEL's declared subset is Fixture 1 (statutory_only) and exercises no Fixture-2 propagation surface; no wall-clock measurement is owed and none is recorded. Doc-based assessment: compatible with sub-60s iff the embedding rollout is; CEL imposes no floor of its own.",
        "citations": [
            "https://github.com/google/cel-spec (evaluation model: compiled, side-effect-free expressions)",
            "https://github.com/google/cel-rust (Rust evaluation path; compile-once evaluate-many)",
            "specs/policy-ir-benchmark-rubric.json#gates[G4].evidence_method.bounded_scope_rule",
        ],
    },
    "biscuit": {
        "model_kind": "offline-revocation-id-list",
        "read_after_write": "Tokens verify offline against the authorizer's local state; there is no central decision point. A revocation becomes effective at an authorizer exactly when that authorizer's revocation-id list includes the revoked block id (read-after-write per authorizer, not global).",
        "staleness_model": "Offline attenuation means issued tokens cannot be recalled; staleness is the revocation-list distribution latency to the slowest authorizer. The vendor documents revocation-id checks as the canonical mitigation and leaves list distribution to the deployer.",
        "sub_60s_assessment": "MEETS BOUND BY CONFIGURATION: sub-60s holds iff revocation-list distribution to every authorizer completes under 60s (push or sub-minute poll). The engine provides the rejection primitive natively; the propagation channel is deployer-owned.",
        "citations": [
            "https://www.biscuitsec.org/docs/guides/revocation/ (revocation identifiers, authorizer-side rejection)",
            "https://docs.rs/biscuit-auth (Rust implementation; authorizer revocation_ids surface)",
        ],
    },
}


# --- measurement loop ---------------------------------------------------------


def measure_engine(
    adapter_cls: type[EngineAdapter], params: dict[str, int], fixture2: dict
) -> dict:
    scenario = fixture2["revocation_scenario"]
    watched = scenario["watched_check"]
    principal, resource = watched["principal"], watched["resource"]
    graph = Fixture2Graph(copy.deepcopy(fixture2["relation_tuples"]))
    adapter = adapter_cls(graph, params)

    pre = adapter.check(principal, resource)
    if not pre:
        fail(f"{adapter.engine}: pre-revocation check must be allow (chain broken)")
    token_before = adapter.token()
    version_before = adapter.policy_version()

    t0 = time.monotonic()
    adapter.revoke(scenario["revoked_tuple_id"])

    first_deny_at: float | None = None
    consecutive = 0
    observations = 0
    while True:
        now = time.monotonic()
        if now - t0 > MEASUREMENT_TIMEOUT_S:
            fail(f"{adapter.engine}: no consistent deny within {MEASUREMENT_TIMEOUT_S}s")
        allowed = adapter.check(principal, resource)
        observations += 1
        if allowed:
            first_deny_at, consecutive = None, 0
        else:
            if first_deny_at is None:
                first_deny_at = now
            consecutive += 1
            if consecutive >= CONSECUTIVE_DENIES_REQUIRED:
                break
        time.sleep(POLL_INTERVAL_S)

    time_to_deny_ms = (first_deny_at - t0) * 1000.0
    token_after = adapter.token()
    version_after = adapter.policy_version()
    bound_ms = scenario["sub_60s_bound_ms"]

    record = {
        "engine": adapter.engine,
        "measured": True,
        "consistency_mode": adapter.consistency_mode,
        "topology": adapter.topology(),
        "revoked_tuple_id": scenario["revoked_tuple_id"],
        "watched_check": {
            "request_id": watched["request_id"],
            "principal": principal,
            "action": watched["action"],
            "resource": resource,
        },
        "pre_revocation_decision": "allow",
        "post_revocation_decision": "deny",
        "poll_interval_ms": int(POLL_INTERVAL_S * 1000),
        "check_observations": observations,
        "consecutive_consistent_denies": consecutive,
        "time_to_consistent_deny_ms": round(time_to_deny_ms, 3),
        "sub_60s_bound_ms": bound_ms,
        "sub_60s": time_to_deny_ms < bound_ms,
        "policy_version_before": version_before,
        "policy_version_after": version_after,
        "emulation": {"emulated": True, "adapter_shape": adapter.adapter_shape},
        "measured_at": _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds"),
    }
    if token_before is not None:
        record["consistency_token_before"] = token_before
    if token_after is not None:
        record["consistency_token_after"] = token_after
    return record


def main() -> None:
    suite = load_json(SUITE_PATH)
    rubric = load_json(RUBRIC_PATH)

    if "fixture_2_workload_contract" not in suite:
        fail("FixtureSuite carries no fixture_2_workload_contract — Fixture 2 not landed")
    fixture2 = suite["fixture_2_workload_contract"]

    pinned_f2 = fixture2["content_hash"]["value"]
    recomputed_f2 = fixture_2_hash(suite)
    if pinned_f2 != recomputed_f2:
        fail(f"frozen Fixture-2 content-hash does not recompute: {pinned_f2} != {recomputed_f2}")
    pinned_suite = suite["content_hash"]["value"]
    if pinned_suite != whole_doc_hash(suite, "fixture-suite-v1:"):
        fail("FixtureSuite content-hash does not recompute")
    pinned_rubric = rubric["content_hash"]["value"]
    if pinned_rubric != whole_doc_hash(rubric, "rubric-v1:"):
        fail("rubric content-hash does not recompute")

    scope = fixture2["revocation_scenario"]["scope_bindings"]
    measured_records = []
    for adapter_cls, params in ADAPTERS:
        record = measure_engine(adapter_cls, params, fixture2)
        record["fixture_2_content_hash"] = pinned_f2
        record["fixture_suite_content_hash"] = pinned_suite
        record["rubric_content_hash"] = pinned_rubric
        measured_records.append(record)
        print(
            f"  {record['engine']:>9}: time_to_consistent_deny="
            f"{record['time_to_consistent_deny_ms']:.1f}ms sub_60s={record['sub_60s']}"
        )
    measured_engines = [r["engine"] for r in measured_records]
    if sorted(measured_engines) != sorted(scope["measured_engines"]):
        fail(f"measured engines {measured_engines} diverge from fixture scope bindings")

    evidence = {
        "artifact_kind": "revocation-evidence",
        "title": "PolicyIrBenchmarkFixture2RevocationEvidence",
        "description": "Sub-60s revocation evidence for Fixture 2 (G4 triple parts 2+3): per-engine consistency-semantics analysis plus topology-documented wall-clock revoke-then-check measurements of time-to-consistent-deny against the frozen Fixture-2 cross-company delegation topology. Produced by benchmarks/policy-ir/revoke_then_check_harness.py; validated by scripts/tests/policy_ir_revocation_evidence_check.py.",
        "_meta": {
            "doc_class": "Machine-Readable-Evidence",
            "spec_id": "POL-IR-BENCH-F2-REVOCATION-EVIDENCE",
            "version": "1.0.0",
            "gate_ref": "specs/policy-ir-benchmark-rubric.json#gates[G4]",
            "fixture_ref": "specs/policy-ir-benchmark-fixture-suite.json#fixture_2_workload_contract",
            "produced_by": "benchmarks/policy-ir/revoke_then_check_harness.py",
            "produced_at": _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds"),
            "environment": {
                "host_class": "single developer host, in-process reference adapters",
                "platform": platform.platform(),
                "python": platform.python_version(),
            },
            "honesty_disclosure": "Measurements are genuine wall-clock revoke-then-check runs against single-host IN-PROCESS REFERENCE ADAPTERS implementing each engine's vendor-documented consistency semantics over the frozen Fixture-2 relation graph — not measurements of production engine deployments. Each record documents its topology and parameters alongside the numbers (rubric G4 requirement); production extrapolation is argued in consistency_semantics_analysis with citations. The verifier lane, not this harness, maps this evidence onto NATIVE/EMULATED/UNSUPPORTED grades.",
        },
        "pins": {
            "rubric_content_hash": pinned_rubric,
            "fixture_suite_content_hash": pinned_suite,
            "fixture_2_content_hash": pinned_f2,
        },
        "sub_60s_bound_ms": fixture2["revocation_scenario"]["sub_60s_bound_ms"],
        "consistent_deny_rule": fixture2["revocation_scenario"]["consistent_deny_rule"],
        "consistency_semantics_analysis": CONSISTENCY_ANALYSIS,
        "measurements": measured_records,
        "exclusions": [
            {
                "engine": "cel",
                "measured": False,
                "reason": "fixture_scope=statutory_only: CEL's declared subset is Fixture 1 and exercises no Fixture-2 propagation surface; per the rubric G4 bounded_scope_rule it owes consistency-semantics analysis from vendor documentation only — no wall-clock measurement, no emulation of out-of-scope fixtures.",
                "rubric_citation": "specs/policy-ir-benchmark-rubric.json#gates[G4].evidence_method.bounded_scope_rule",
            }
        ],
    }

    EVIDENCE_PATH.parent.mkdir(parents=True, exist_ok=True)
    EVIDENCE_PATH.write_text(
        json.dumps(evidence, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print(
        f"fixture-2 revocation evidence written: {EVIDENCE_PATH.relative_to(REPO_ROOT)} "
        f"(measured={len(measured_records)}, analysis_engines={len(CONSISTENCY_ANALYSIS)}, "
        f"fixture_2_content_hash={pinned_f2[:32]}...)"
    )


if __name__ == "__main__":
    main()
