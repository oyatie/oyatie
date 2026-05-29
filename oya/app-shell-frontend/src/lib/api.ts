// Typed API client for the Leptos prototype dev server render-envelope endpoint.
// This module will be superseded by the OpenAPI-generated client (ADR-0372 D2)
// once the Rust backend publishes an OpenAPI 3.2.0 contract for this path.
// The generated client from `npm run codegen` already covers ops-workspace-shell-v1
// and hr-api; this file bridges the prototype-only envelope API.

import type { DemoContext, TenantRenderEnvelope } from "./render-envelope";

// During development the Leptos prototype server runs on port 3000;
// the SolidJS shell runs on port 3001. In production this will point at
// the real Rust backend URL injected via VITE_API_BASE_URL.
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
 * Fetch the server-derived render envelope for a given demo context.
 * Mirrors the Leptos prototype endpoint: GET /api/render-envelope/:context
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
