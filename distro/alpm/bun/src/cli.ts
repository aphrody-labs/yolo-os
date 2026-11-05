#!/usr/bin/env bun
// yolo-search: one query across pacman sync repos (via yolo-pm) and the
// package backends that are actually present. Missing backends are reported
// as unavailable; no results are fabricated.

import { allBackends, resolveYoloPm, unifiedSearch, type BackendReport } from "./backends.ts";

interface Args {
  query: string;
  limit: number;
  json: boolean;
  cache?: string;
}

export function parseArgs(argv: string[]): Args {
  const args: Args = { query: "", limit: 10, json: false };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--json") args.json = true;
    else if (a === "--limit") args.limit = Number(argv[++i]);
    else if (a === "--cache") args.cache = argv[++i];
    else if (a === "-h" || a === "--help") throw new Error("help");
    else args.query = a;
  }
  if (!args.query) throw new Error("missing query");
  if (!Number.isInteger(args.limit) || args.limit < 1) throw new Error("--limit must be a positive integer");
  return args;
}

function render(reports: BackendReport[]): string {
  const lines: string[] = [];
  for (const r of reports) {
    const tag = r.status === "ok" ? `${r.hits.length} hit(s)` : `${r.status}: ${r.reason ?? ""}`;
    lines.push(`[${r.backend}] ${tag}`);
    for (const h of r.hits) {
      lines.push(`  ${h.backend}  ${h.name} ${h.version}`);
      if (h.description) lines.push(`      ${h.description}`);
    }
  }
  return lines.join("\n");
}

async function main(): Promise<number> {
  let args: Args;
  try {
    args = parseArgs(Bun.argv.slice(2));
  } catch (err) {
    const msg = (err as Error).message;
    console.error(msg === "help" ? "usage: yolo-search [--json] [--limit N] [--cache DIR] <query>" : `yolo-search: ${msg}`);
    return msg === "help" ? 0 : 2;
  }
  const reports = await unifiedSearch(allBackends(resolveYoloPm(), args.cache), args.query, args.limit);
  console.log(args.json ? JSON.stringify(reports, null, 2) : render(reports));
  return 0;
}

if (import.meta.main) {
  process.exitCode = await main();
}
