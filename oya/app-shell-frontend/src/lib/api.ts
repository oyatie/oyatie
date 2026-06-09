// Typed API client for the canonical Leptos render-envelope endpoint.
// ADR-0393 keeps the Rust render_envelope contract as source of truth; this
// archived TS client remains only until the Leptos shell migration removes it.
// Generated clients from `pnpm codegen` cover ops-workspace-shell-v1 and hr-api.

import type { DemoContext, TenantRenderEnvelope } from "./render-envelope";

// During transition evidence runs the Leptos render-envelope service runs on
// port 3000 and this archived shell runs on port 3001. Deployed environments
// inject the real Rust backend URL via VITE_API_BASE_URL.
const API_BASE =
  typeof import.meta.env !== "undefined" && import.meta.env["VITE_API_BASE_URL"]
    ? String(import.meta.env["VITE_API_BASE_URL"])
    : "http://localhost:3000";

export class RenderEnvelopeNotFoundError extends Error {
  constructor(context: DemoContext) {
    super(`Render envelope not found for context: ${context}`);
    this.name = "RenderEnvelopeNotFoundError";
  }
}

export class RenderEnvelopeApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "RenderEnvelopeApiError";
  }
}

/**
 * Fetch the server-derived render envelope for a given role context.
 * Mirrors the Leptos endpoint: GET /api/render-envelope/:context
 */
export async function fetchRenderEnvelope(
  context: DemoContext,
  init?: RequestInit,
): Promise<TenantRenderEnvelope> {
  const url = `${API_BASE}/api/render-envelope/${context}`;
  const response = await fetch(url, {
    headers: { Accept: "application/json" },
    ...init,
  });

  if (response.status === 404) {
    throw new RenderEnvelopeNotFoundError(context);
  }

  if (!response.ok) {
    throw new RenderEnvelopeApiError(
      response.status,
      `Render envelope request failed: ${response.status} ${response.statusText}`,
    );
  }

  return response.json() as Promise<TenantRenderEnvelope>;
}

// --- Typed client for the ops-workspace-shell OpenAPI contract ---
// (ops-workspace-shell-v1.openapi.yaml, openapi: 3.2.0)
// Full generated types live in generated/ops-workspace-shell.d.ts after
// `npm run codegen`. This thin wrapper provides ergonomic fetch helpers.

export interface ShellSurface {
  id: string;
  canonical_route: string;
  visibility_tier:
    | "public"
    | "tenant-public"
    | "tenant-private"
    | "internal-public"
    | "internal-private"
    | "system-only";
  state: "reserved-coming-soon" | "live" | "retired";
  owning_bc_id: string;
  cedar_fragments?: string[];
  openapi_contract?: string | null;
  retired_redirects_to?: string | null;
}

export interface SurfaceListResponse {
  surfaces: ShellSurface[];
  count: number;
}

export interface HealthResponse {
  status: "healthy" | "degraded" | "unhealthy";
  surface_count: number;
  version: string;
  cell_id?: string;
}

export async function listLiveSurfaces(init?: RequestInit): Promise<SurfaceListResponse> {
  const url = `${API_BASE}/workspace`;
  const response = await fetch(url, {
    headers: { Accept: "application/json" },
    ...init,
  });
  if (!response.ok) {
    throw new RenderEnvelopeApiError(response.status, `listLiveSurfaces: ${response.statusText}`);
  }
  return response.json() as Promise<SurfaceListResponse>;
}

export async function shellHealth(init?: RequestInit): Promise<HealthResponse> {
  const url = `${API_BASE}/workspace/api/v1/health`;
  const response = await fetch(url, {
    headers: { Accept: "application/json" },
    ...init,
  });
  if (!response.ok) {
    throw new RenderEnvelopeApiError(response.status, `shellHealth: ${response.statusText}`);
  }
  return response.json() as Promise<HealthResponse>;
}
