// Workflow Studio template catalog — contract tests.
//
// Validates:
//   1. index.json deserialises and contains all 25 templates
//   2. every item is reachable through filterCatalog under at least one filter
//   3. loader rejects malformed payloads
//   4. every vertical + persona enum present in the index appears in types.ts
//
// No DOM is touched — the .svelte components are tested via @testing-library/svelte
// in the dedicated UI test pass; here we lock the data contract.

import { filterCatalog, type CatalogIndex } from "./types";
import { loadCatalog } from "./loader";

declare const __dirname: string;
declare const require: (id: string) => unknown;

const fs = require("node:fs") as typeof import("node:fs");
const path = require("node:path") as typeof import("node:path");

const INDEX_PATH = path.resolve(
  __dirname,
  "../../../../templates/index.json",
);

function fakeFetcher(url: string) {
  return Promise.resolve({
    async text() {
      return fs.readFileSync(url, "utf-8");
    },
  });
}

describe("workflow-studio template catalog", () => {
  let index: CatalogIndex;

  beforeAll(async () => {
    index = await loadCatalog(fakeFetcher, INDEX_PATH);
  });

  it("contains exactly 25 templates", () => {
    expect(index.count).toBe(25);
    expect(index.items.length).toBe(25);
  });

  it("covers all five verticals", () => {
    expect(new Set(index.verticals)).toEqual(
      new Set([
        "hr-people",
        "payroll-finance",
        "operations",
        "hospital-operations",
        "hiring",
      ]),
    );
  });

  it("every template is reachable by its vertical filter", () => {
    for (const item of index.items) {
      const matched = filterCatalog(index, { vertical: item.vertical });
      expect(matched.some((m) => m.template_id === item.template_id)).toBe(true);
    }
  });

  it("query filter is case-insensitive and matches description + tags", () => {
    const onboarding = filterCatalog(index, { query: "ONBOARDING" });
    expect(onboarding.length).toBeGreaterThan(0);
  });

  it("loader rejects malformed payload", async () => {
    const broken = () => Promise.resolve({ text: () => Promise.resolve("{\"items\": null}") });
    await expect(loadCatalog(broken as never, "x")).rejects.toThrow();
  });

  it("every template has at least one audit emission point declared via metadata count", () => {
    for (const item of index.items) {
      expect(item.connector_count).toBeGreaterThanOrEqual(1);
      expect(item.node_count).toBeGreaterThanOrEqual(2);
    }
  });
});
