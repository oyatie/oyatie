// Workflow Studio template catalog — loader.
//
// `loadCatalog` reads the aggregated index.json file produced by
// `microservices/workflow-studio/templates/index.json`. In SSR + vitest
// environments we read from the filesystem (Node fs); in the browser the
// catalog is served as a static asset.
//
// The runtime is intentionally environment-agnostic — the caller passes a
// `fetch`-shaped function.

import type { CatalogIndex } from "./types";

export type FetchLike = (url: string) => Promise<{ text: () => Promise<string> }>;

const CATALOG_URL = "/workflow-studio/templates/index.json";

export async function loadCatalog(fetcher: FetchLike, url: string = CATALOG_URL): Promise<CatalogIndex> {
  const res = await fetcher(url);
  const body = await res.text();
  const parsed = JSON.parse(body) as CatalogIndex;
  if (!parsed || !Array.isArray(parsed.items)) {
    throw new Error("workflow-studio template catalog: malformed index");
  }
  return parsed;
}
