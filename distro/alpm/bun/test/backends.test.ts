import { describe, expect, test } from "bun:test";
import { mkdtempSync, writeFileSync, chmodSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { adapterlessBackend, allBackends, parseCargoSearch, pacmanBackend, unifiedSearch } from "../src/backends.ts";
import { parseArgs } from "../src/cli.ts";

const CARGO_SAMPLE = `serde = "1.0.219"    # A generic serialization/deserialization framework
serde_json = "1.0.140"    # A JSON serialization file format
`;

describe("cargo search parsing", () => {
  test("parses name, version and description", () => {
    const hits = parseCargoSearch(CARGO_SAMPLE);
    expect(hits).toEqual([
      { backend: "cargo", name: "serde", version: "1.0.219", description: "A generic serialization/deserialization framework" },
      { backend: "cargo", name: "serde_json", version: "1.0.140", description: "A JSON serialization file format" },
    ]);
  });

  test("ignores non-result lines", () => {
    expect(parseCargoSearch("... and 1234 crates more\n")).toEqual([]);
  });
});

describe("availability", () => {
  test("missing binaries are unavailable, not errors and not invented hits", async () => {
    const reports = await unifiedSearch(
      [adapterlessBackend("flatpak", "flatpak"), { id: "x", binary: "definitely-absent-binary", search: async () => [] }],
      "q",
      5,
      () => false,
    );
    expect(reports.map((r) => r.status)).toEqual(["unavailable", "unavailable"]);
    expect(reports[1].reason).toContain("not found");
    expect(reports.every((r) => r.hits.length === 0)).toBe(true);
  });

  test("a present backend without an adapter stays unavailable", async () => {
    const [report] = await unifiedSearch([adapterlessBackend("uv", "uv")], "q", 5, () => true);
    expect(report.status).toBe("unavailable");
    expect(report.reason).toBe("no search adapter in this build");
    const [absent] = await unifiedSearch([adapterlessBackend("uv", "uv")], "q", 5, () => false);
    expect(absent.reason).toBe("uv not found");
    expect(report.status).toBe("unavailable");
    expect(report.reason).toBe("no search adapter in this build");
  });

  test("adapter errors are reported as error status", async () => {
    const [report] = await unifiedSearch(
      [{ id: "bad", binary: "bad", search: async () => { throw new Error("boom"); } }],
      "q",
      5,
      () => true,
    );
    expect(report).toEqual({ backend: "bad", status: "error", reason: "boom", hits: [] });
  });

  test("allBackends without yolo-pm reports pacman unavailable", async () => {
    const reports = await unifiedSearch(allBackends(undefined), "q", 5, () => false);
    expect(reports.find((r) => r.backend === "pacman")?.status).toBe("unavailable");
  });
});

describe("pacman adapter", () => {
  test("maps yolo-pm --json rows and respects the limit", async () => {
    const dir = mkdtempSync(join(tmpdir(), "yolo-search-"));
    const fake = join(dir, "yolo-pm");
    const rows = [1, 2, 3].map((n) => ({ repo: "core", name: `p${n}`, version: "1-1", desc: "d" }));
    writeFileSync(fake, `#!/bin/sh\ncat <<'JSON'\n${JSON.stringify(rows)}\nJSON\n`);
    chmodSync(fake, 0o755);
    const hits = await pacmanBackend(fake).search("p", 2);
    expect(hits.map((h) => h.name)).toEqual(["p1", "p2"]);
    expect(hits[0].backend).toBe("pacman/core");
  });
});

describe("cli args", () => {
  test("parses flags and query", () => {
    expect(parseArgs(["--json", "--limit", "3", "--cache", "/c", "hypr"])).toEqual({
      query: "hypr",
      limit: 3,
      json: true,
      cache: "/c",
    });
  });

  test("rejects a missing query and a bad limit", () => {
    expect(() => parseArgs([])).toThrow("missing query");
    expect(() => parseArgs(["--limit", "0", "x"])).toThrow("positive integer");
  });
});
