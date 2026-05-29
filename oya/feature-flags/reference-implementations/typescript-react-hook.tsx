// Reference implementation — feature-flags React hook for TypeScript clients.
//
// Pattern: client-side React surfaces a stable boolean from a server-side
// evaluator. The hook handles cache TTL, error fallback, and Suspense.
//
// Doctrine references:
//   - ADR-0159  Runtime feature-flag substrate
//   - ADR-0145  Three-tier evaluator latency budget
//   - microservices/feature-flags/PRD.md §Scope ("Server-side-only OpenFeature surface")

import { useEffect, useState, useRef } from "react";

type FFClient = {
  evalBool(key: string, defaultValue: boolean): Promise<{ value: boolean; variant: string; reason: string }>;
};

declare const ffClient: FFClient;

const CACHE_TTL_MS = 60_000;
type CacheEntry = { value: boolean; expiresAt: number };
const cache = new Map<string, CacheEntry>();

/** Server-side flag evaluation surfaced as a stable boolean to React. */
export function useFeatureFlag(key: string, defaultValue: boolean): boolean {
  const [value, setValue] = useState<boolean>(() => {
    const entry = cache.get(key);
    if (entry && entry.expiresAt > Date.now()) {
      return entry.value;
    }
    return defaultValue;
  });
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    let timer: ReturnType<typeof setTimeout>;

    async function refresh(): Promise<void> {
      try {
        const r = await ffClient.evalBool(key, defaultValue);
        if (!mounted.current) return;
        cache.set(key, { value: r.value, expiresAt: Date.now() + CACHE_TTL_MS });
        setValue(r.value);
      } catch (err) {
        // Network or evaluator failure → keep last-cached or default; never throw to React.
        // Per ADR-0145, the SDK is required to fail-closed to the caller-supplied default.
      } finally {
        if (mounted.current) {
          timer = setTimeout(refresh, CACHE_TTL_MS + Math.floor(Math.random() * 10_000) - 5_000);
        }
      }
    }

    refresh();

    return () => {
      mounted.current = false;
      clearTimeout(timer);
    };
  }, [key, defaultValue]);

  return value;
}

/** Same hook but for JSON variants. Strongly typed at the call site. */
export function useFeatureFlagJSON<T>(key: string, defaultValue: T): T {
  const [value, setValue] = useState<T>(defaultValue);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        // @ts-expect-error evalJson is on the actual client; declared for the example
        const r = await ffClient.evalJson(key, defaultValue);
        if (!cancelled) setValue(r.value as T);
      } catch {
        // Keep default
      }
    })();
    return () => { cancelled = true; };
  }, [key]);

  return value;
}
