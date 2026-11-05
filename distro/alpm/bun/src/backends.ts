// Package backends for the unified search.
//
// A backend is reported "available" only when its executable is on PATH
// (pacman's yolo-pm is also checked at its build location). Missing backends
// are reported as "unavailable" with the reason; no results are invented.

import { existsSync } from "node:fs";

export type Status = "ok" | "unavailable" | "error";

export interface Hit {
  backend: string;
  name: string;
  version: string;
  description: string;
}

export interface BackendReport {
  backend: string;
  status: Status;
  reason?: string;
  hits: Hit[];
}

export interface Backend {
  id: string;
  /** Executable (name or absolute path) that must exist for this backend to run. */
  binary: string;
  /** Absent when no search adapter is implemented in this build. */
  search?: (query: string, limit: number) => Promise<Hit[]>;
}

/** Resolves yolo-pm: $YOLO_PM, then PATH, then the cargo release build. */
export function resolveYoloPm(): string | undefined {
  const fromEnv = process.env.YOLO_PM;
  if (fromEnv) return fromEnv;
  const onPath = Bun.which("yolo-pm");
  if (onPath) return onPath;
  const built = new URL("../../target/release/yolo-pm", import.meta.url).pathname;
  return Bun.file(built).size > 0 ? built : undefined;
}

async function run(cmd: string[]): Promise<{ code: number; stdout: string; stderr: string }> {
  const proc = Bun.spawn(cmd, { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  return { code, stdout, stderr };
}

export function pacmanBackend(binary: string, cacheDir?: string): Backend {
  return {
    id: "pacman",
    binary,
    async search(query, limit) {
      const args = [binary, ...(cacheDir ? ["--cache", cacheDir] : []), "--json", "search", query];
      const res = await run(args);
      if (res.code !== 0) throw new Error(res.stderr.trim() || `exit ${res.code}`);
      const rows = JSON.parse(res.stdout) as Array<{ repo: string; name: string; version: string; desc: string }>;
      return rows.slice(0, limit).map((r) => ({
        backend: `pacman/${r.repo}`,
        name: r.name,
        version: r.version,
        description: r.desc,
      }));
    },
  };
}

// `cargo search` prints lines like: name = "1.2.3"    # description
const CARGO_LINE = /^(\S+) = "([^"]+)"\s+# (.*)$/;

export function parseCargoSearch(text: string): Hit[] {
  const hits: Hit[] = [];
  for (const line of text.split("\n")) {
    const m = CARGO_LINE.exec(line.trim());
    if (m) hits.push({ backend: "cargo", name: m[1], version: m[2], description: m[3] });
  }
  return hits;
}

export function cargoBackend(): Backend {
  return {
    id: "cargo",
    binary: "cargo",
    async search(query, limit) {
      const res = await run(["cargo", "search", query, "--limit", String(limit)]);
      if (res.code !== 0) throw new Error(res.stderr.trim() || `exit ${res.code}`);
      return parseCargoSearch(res.stdout);
    },
  };
}

/** Backends with no search adapter in this build; always reported unavailable. */
export function adapterlessBackend(id: string, binary: string): Backend {
  return { id, binary };
}

export function allBackends(pacmanBinary: string | undefined, cacheDir?: string): Backend[] {
  const pacman = pacmanBinary
    ? pacmanBackend(pacmanBinary, cacheDir)
    : { id: "pacman", binary: "yolo-pm" };
  return [
    pacman,
    cargoBackend(),
    adapterlessBackend("flatpak", "flatpak"),
    adapterlessBackend("uv", "uv"),
    adapterlessBackend("winget", "winget"),
  ];
}

/** True when a bare name is on PATH, or an absolute path exists. */
export function binaryExists(binary: string): boolean {
  if (binary.startsWith("/")) return existsSync(binary);
  return Boolean(Bun.which(binary));
}

/**
 * Runs each backend that has a search adapter and whose binary exists. Every
 * other backend is reported "unavailable" with the reason; nothing is invented.
 */
export async function unifiedSearch(
  backends: Backend[],
  query: string,
  limit: number,
  has: (binary: string) => boolean = binaryExists,
): Promise<BackendReport[]> {
  return Promise.all(
    backends.map(async (b): Promise<BackendReport> => {
      if (!has(b.binary)) {
        return { backend: b.id, status: "unavailable", reason: `${b.binary} not found`, hits: [] };
      }
      if (!b.search) {
        return { backend: b.id, status: "unavailable", reason: "no search adapter in this build", hits: [] };
      }
      try {
        return { backend: b.id, status: "ok", hits: await b.search(query, limit) };
      } catch (err) {
        return { backend: b.id, status: "error", reason: (err as Error).message, hits: [] };
      }
    }),
  );
}
