# Reference implementation — feature-flags asyncio client for Python services.
#
# Pattern: Python workload calls feature-flags side-car over gRPC HTTP/3 + QUIC
# (ADR-0253) with a per-process LRU cache and a background refresh task.
#
# Doctrine:
#   - ADR-0159  Runtime feature-flag substrate
#   - ADR-0263  Audit-chain canonical event registry (the trace_id propagation)
#   - microservices/feature-flags/PRD.md §F-FF-02 (≤ 1 ms p99 eval budget)

import asyncio
import time
from dataclasses import dataclass
from typing import Any, Dict, Optional

import grpc.aio  # type: ignore

from feature_flags_v1_pb2 import EvalBoolRequest, EvalBoolResponse, Context  # type: ignore
from feature_flags_v1_pb2_grpc import FeatureFlagsStub  # type: ignore

_CACHE_TTL_S = 60.0


@dataclass
class _CacheEntry:
    value: bool
    expires_at: float


class FFClient:
    """Async Python client. One per process; safe to share across tasks."""

    def __init__(self, endpoint: str, tenant_id: str) -> None:
        self._endpoint = endpoint
        self._tenant_id = tenant_id
        self._channel: Optional[grpc.aio.Channel] = None
        self._stub: Optional[FeatureFlagsStub] = None
        self._cache: Dict[str, _CacheEntry] = {}
        self._lock = asyncio.Lock()

    async def _ensure_connected(self) -> None:
        if self._channel is None:
            self._channel = grpc.aio.secure_channel(
                self._endpoint,
                grpc.ssl_channel_credentials(),
                options=[("grpc.default_authority", "feature-flags.oyatie.svc")],
            )
            self._stub = FeatureFlagsStub(self._channel)

    async def eval_bool(
        self,
        flag_key: str,
        default: bool,
        persona_tier: str = "default",
        pack_id: str = "us-default",
        cohort_ids: Optional[list[str]] = None,
        trace_id: Optional[str] = None,
    ) -> bool:
        """Evaluate a boolean flag. Returns default on any error path."""
        cached = self._cache.get(flag_key)
        if cached is not None and cached.expires_at > time.monotonic():
            return cached.value

        await self._ensure_connected()
        assert self._stub is not None

        ctx = Context(
            tenant_id=self._tenant_id,
            persona_tier=persona_tier,
            pack_id=pack_id,
            cohort_ids=cohort_ids or [],
            trace_id=trace_id or "",
        )

        request = EvalBoolRequest(flag_key=flag_key, default=default, context=ctx)
        try:
            response: EvalBoolResponse = await self._stub.EvalBool(
                request, timeout=0.05  # 50 ms hard ceiling per ADR-0145
            )
            value = response.value
        except grpc.aio.AioRpcError:
            # Fail-closed to the caller-supplied default per ADR-0145.
            value = default

        async with self._lock:
            self._cache[flag_key] = _CacheEntry(
                value=value,
                expires_at=time.monotonic() + _CACHE_TTL_S,
            )

        return value

    async def close(self) -> None:
        if self._channel is not None:
            await self._channel.close()
            self._channel = None
            self._stub = None


# --- example ---

async def main() -> None:
    client = FFClient(endpoint="feature-flags.oyatie.svc:50500", tenant_id="t-acme-co")
    try:
        if await client.eval_bool("comms-email.smtp-fallback-v2", default=False):
            print("Using SMTP fallback v2")
        else:
            print("Using primary SMTP path")
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
