#!/usr/bin/env bun
import { crawl, inspectGitRemote, listGitHubRepositories, synthesizeMarkdown, validatePublicHttpUrl, type CrawlResult } from "@yolo-web/web-crawler";
import { basename, dirname, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const args = process.argv.slice(2);
const command = args.shift() ?? "help";
const value = (flag: string, fallback?: string) => {
  const index = args.indexOf(flag);
  return index < 0 ? fallback : args[index + 1];
};
const has = (flag: string) => args.includes(flag);
const valueOptions = new Set(["--depth", "--max-pages", "--delay-ms", "--output", "--markdown", "--sources", "--book", "--script", "--step-id", "--journal", "--dir"]);
const positionalArgs = () => {
  const values: string[] = [];
  for (let index = 0; index < args.length; index++) {
    const arg = args[index]!;
    if (valueOptions.has(arg)) { index++; continue; }
    if (!arg.startsWith("-")) values.push(arg);
  }
  return values;
};
const fail = (message: string): never => { console.error(message); process.exit(2); };
async function fetchPublicUrl(rawUrl: string, timeoutMs: number) {
  const initial = validatePublicHttpUrl(rawUrl);
  let current = initial;
  for (let redirects = 0; redirects <= 5; redirects++) {
    const response = await fetch(current, { redirect: "manual", signal: AbortSignal.timeout(timeoutMs) });
    const location = response.headers.get("location");
    if (![301, 302, 303, 307, 308].includes(response.status) || !location) return { response, url: current };
    await response.body?.cancel();
    if (redirects === 5) throw new Error(`Redirect limit exceeded for ${rawUrl}`);
    const next = validatePublicHttpUrl(location, current.toString());
    if (initial.protocol === "https:" && next.protocol !== "https:") throw new Error(`Refusing HTTPS downgrade while fetching ${rawUrl}`);
    current = next;
  }
  throw new Error(`Redirect limit exceeded for ${rawUrl}`);
}
const help = `lfs-bun — Bun orchestration for LFS research and builds

Commands:
  sources                         Print the curated research seeds
  crawl [--depth N --max-pages N] Fetch seeds with YOLO-Web, save JSON and Markdown
  repos [--max-pages N]           Collect GitHub org/user repository rankings via API
  git-heads [--output FILE]       Capture default branches and HEAD commits for Git seeds
  synthesize <crawl.json>         Build deterministic cited research notes
  book sync [--track stable-systemd] [--dir <path>]
                                  Retrieve the selected official LFS package list and checksums
  alfs setup --dir <path> [--execute]
                                  Clone official jhalfs into a new directory
  alfs generate --dir <jhalfs checkout> [--execute]
                                  Generate scripts/Makefile only from RUNMAKE=n config
  alfs check --dir <jhalfs checkout>
                                  Check Linux host tools required by jhalfs
  plan --book <path>              Inspect an LFS book download manifest without running commands
  fetch --book <path> [--sources <dir>] [--execute]
                                  Download manifest files to a cache; requires --execute
  run --script <path> [--step-id <id>] [--journal <path>] [--execute]
                                  Run/resume one ALFS-generated step with a journal

Artifacts default to ./artifacts. Full LFS builds require a supported Linux host and
the official book's host requirements, errata, advisories, and ALFS-generated steps.
`;

if (command === "help" || command === "--help" || command === "-h") { console.log(help); process.exit(0); }
if (command === "sources") {
  console.log(await Bun.file(`${root}/sources.json`).text()); process.exit(0);
}
if (command === "repos") {
  const manifest = await Bun.file(`${root}/sources.json`).json() as { sources: Array<{ name: string; url: string; kind: string }> };
  const targets = manifest.sources.filter((source) => source.kind === "github-org" || source.kind === "github-user");
  const maxPages = Number(value("--max-pages", "5"));
  const listings = [];
  for (const source of targets) {
    const path = new URL(source.url).pathname.split("/").filter(Boolean);
    const ownerType = source.kind === "github-org" ? "org" : "user";
    const owner = ownerType === "org" ? path[1] : path[0];
    if (!owner) throw new Error(`Cannot determine GitHub owner from ${source.url}`);
    listings.push({ label: source.name, ...(await listGitHubRepositories(owner, { ownerType, maxPages, token: process.env.GITHUB_TOKEN })) });
  }
  const snapshot = { schemaVersion: 1, generatedAt: new Date().toISOString(), listings };
  const output = value("--output", `${root}/artifacts/github-repositories-${new Date().toISOString().slice(0, 10)}.json`)!;
  const outputPath = resolve(output);
  await Bun.$`mkdir -p ${dirname(outputPath)}`;
  await Bun.write(outputPath, `${JSON.stringify(snapshot, null, 2)}\n`);
  const markdown = ["# GitHub repository rankings", "", `Collected: ${snapshot.generatedAt}`, "", ...listings.flatMap((listing) => [
    `## ${listing.label} (${listing.owner}, ${listing.repositories.length} repositories${listing.truncated ? ", partial listing" : ""})`,
    "", "| Rank | Repository | Stars | Language | Archived |", "| ---: | --- | ---: | --- | :---: |",
    ...listing.repositories.map((repo, index) => `| ${index + 1} | [${repo.fullName}](${repo.htmlUrl}) | ${repo.stars} | ${repo.language ?? "—"} | ${repo.archived ? "yes" : "no"} |`), "",
  ])].join("\n");
  await Bun.write(outputPath.replace(/\.json$/i, ".md"), markdown);
  console.log(JSON.stringify({ output: outputPath, listings: listings.map(({ label, owner, repositories, truncated }) => ({ label, owner, repositories: repositories.length, top: repositories[0]?.fullName, truncated })) }, null, 2));
  process.exit(0);
}
if (command === "git-heads") {
  const manifest = await Bun.file(`${root}/sources.json`).json() as { sources: Array<{ name: string; url: string; kind: string }> };
  const remotes = [];
  for (const source of manifest.sources.filter((item) => item.kind === "git")) {
    remotes.push({ label: source.name, ...(await inspectGitRemote(source.url)) });
  }
  const snapshot = { schemaVersion: 1, generatedAt: new Date().toISOString(), remotes };
  const output = resolve(value("--output", `${root}/artifacts/git-remotes-${new Date().toISOString().slice(0, 10)}.json`)!);
  await Bun.$`mkdir -p ${dirname(output)}`;
  await Bun.write(output, `${JSON.stringify(snapshot, null, 2)}\n`);
  await Bun.write(output.replace(/\.json$/i, ".md"), ["# Git remote heads", "", `Collected: ${snapshot.generatedAt}`, "", ...remotes.map((remote) => `- [${remote.label}](${remote.remote}): \`${remote.symbolicHead ?? "detached"}\` at \`${remote.commit}\` (SHA-256 \`${remote.sha256}\`, fetched ${remote.fetchedAt})`), ""].join("\n"));
  console.log(JSON.stringify({ output, remotes: remotes.map(({ label, symbolicHead, commit }) => ({ label, symbolicHead, commit })) }, null, 2));
  process.exit(0);
}
if (command === "crawl") {
  const manifest = await Bun.file(`${root}/sources.json`).json() as { sources: Array<{ name: string; url: string; kind: string }> };
  const seeds = positionalArgs();
  const webSeeds = seeds.length ? seeds : manifest.sources.filter((source) => source.kind !== "git").map((source) => source.url);
  const result = await crawl({
    seeds: webSeeds,
    maxDepth: Number(value("--depth", "1")), maxPages: Number(value("--max-pages", "60")),
    delayMs: Number(value("--delay-ms", "900")), respectRobots: !has("--ignore-robots"), includeBody: has("--include-body"),
  });
  const out = value("--output", `${root}/artifacts/research-${new Date().toISOString().slice(0, 10)}.json`)!;
  const md = value("--markdown", out.replace(/\.json$/i, ".md"))!;
  await Bun.$`mkdir -p ${root}/artifacts`;
  await Bun.write(out, `${JSON.stringify(result, null, 2)}\n`);
  await Bun.write(md, synthesizeMarkdown(result, "YOLO OS / LFS web reconnaissance"));
  console.log(JSON.stringify({ pages: result.pages.length, output: out, markdown: md, origins: result.policy.origins }, null, 2));
  process.exit(0);
}
if (command === "synthesize") {
  const input = positionalArgs()[0];
  if (!input) fail("synthesize requires a crawl JSON path");
  const inputPath = input!;
  const result = await Bun.file(inputPath).json() as CrawlResult;
  const output = value("--output", inputPath.replace(/\.json$/i, ".md"))!;
  await Bun.write(output, synthesizeMarkdown(result, "YOLO OS / LFS research synthesis"));
  console.log(output); process.exit(0);
}
if (command === "book") {
  const action = positionalArgs()[0];
  if (action !== "sync") fail("Usage: lfs-bun book sync [--track stable-systemd|development|stable|stable-sysv] [--dir <path>]");
  const track = value("--track", "stable-systemd")!;
  const bases: Record<string, string> = {
    "stable-systemd": "https://www.linuxfromscratch.org/lfs/downloads/stable-systemd/",
    development: "https://www.linuxfromscratch.org/lfs/downloads/development/",
    stable: "https://www.linuxfromscratch.org/lfs/downloads/stable/",
    "stable-sysv": "https://www.linuxfromscratch.org/lfs/downloads/stable/",
  };
  const base = bases[track];
  if (!base) fail(`Unsupported LFS track: ${track}`);
  const directory = resolve(value("--dir", `${root}/artifacts/lfs-book/${track}`)!);
  await Bun.$`mkdir -p ${directory}`;
  const files: Record<string, { url: string; bytes: number; sha256: string }> = {};
  for (const name of ["wget-list", "md5sums"]) {
    const url = `${base}${name}`;
    const { response, url: finalUrl } = await fetchPublicUrl(url, 30000);
    if (!response.ok || finalUrl.hostname !== "www.linuxfromscratch.org") throw new Error(`Could not retrieve official LFS ${name} from ${url} (HTTP ${response.status})`);
    const bytes = new Uint8Array(await response.arrayBuffer());
    if (bytes.byteLength > 1_000_000) throw new Error(`Unexpectedly large LFS manifest: ${name}`);
    const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
    files[name] = { url, bytes: bytes.byteLength, sha256: [...digest].map((byte) => byte.toString(16).padStart(2, "0")).join("") };
    await Bun.write(`${directory}/${name}`, bytes);
  }
  const metadata = { schemaVersion: 1, project: "Linux From Scratch", track, retrievedAt: new Date().toISOString(), files };
  await Bun.write(`${directory}/book-source.json`, `${JSON.stringify(metadata, null, 2)}\n`);
  console.log(JSON.stringify({ directory, ...metadata }, null, 2));
  process.exit(0);
}
if (command === "alfs") {
  const action = positionalArgs()[0];
  const directory = value("--dir");
  if (!directory) fail(`alfs ${action ?? "<setup|check|generate>"} requires --dir <path>`);
  const target = resolve(directory!);
  const upstream = "https://git.linuxfromscratch.org/jhalfs.git";
  if (action === "setup") {
    if (!has("--execute")) { console.log(JSON.stringify({ planned: true, command: ["git", "clone", "--depth", "1", upstream, target] }, null, 2)); process.exit(0); }
    if (await Bun.file(`${target}/.git/HEAD`).exists()) {
      const origin = await Bun.$`git -C ${target} remote get-url origin`.text();
      if (origin.trim() !== upstream) fail(`Refusing to reuse ${target}: origin is ${origin.trim()}`);
      console.log(JSON.stringify({ existing: true, directory: target, origin: origin.trim() }, null, 2)); process.exit(0);
    }
    if (await Bun.file(target).exists()) fail(`Refusing to clone over existing directory: ${target}`);
    await Bun.$`mkdir -p ${dirname(target)}`;
    await Bun.$`git clone --depth 1 ${upstream} ${target}`;
    const commit = (await Bun.$`git -C ${target} rev-parse HEAD`.text()).trim();
    await Bun.write(`${target}/yolo-alfs-source.json`, `${JSON.stringify({ upstream, commit, clonedAt: new Date().toISOString() }, null, 2)}\n`);
    console.log(JSON.stringify({ directory: target, upstream, commit }, null, 2)); process.exit(0);
  }
  if (action === "check") {
    if (!(await Bun.file(`${target}/jhalfs`).exists()) || !(await Bun.file(`${target}/configuration`).exists()) || !(await Bun.file(`${target}/common/libs/func_check_version.sh`).exists())) fail(`Expected a configured jhalfs checkout at ${target}`);
    const tools = process.platform === "linux" ? ["bash", "git", "make", "python3", "sudo", "wget", "xmllint", "xsltproc"] : [];
    const results = [];
    for (const tool of tools) {
      const proc = Bun.spawn(["bash", "-lc", `command -v ${tool}`], { stdin: "ignore", stdout: "pipe", stderr: "ignore" });
      const path = (await new Response(proc.stdout).text()).trim();
      results.push({ tool, found: (await proc.exited) === 0, path: path || null });
    }
    const linuxHost = process.platform === "linux";
    console.log(JSON.stringify({ directory: target, linuxHost, tools: results, note: "Also verify libxml2/libxslt versions, DocBook XML DTD 4.5, DocBook XSL, the selected LFS host requirements, an empty dedicated target mount, and sudo access." }, null, 2));
    process.exit(linuxHost && results.every((result) => result.found) ? 0 : 1);
  }
  if (action === "generate") {
    const configPath = `${target}/configuration`;
    if (!(await Bun.file(`${target}/jhalfs`).exists()) || !(await Bun.file(configPath).exists())) fail(`Expected jhalfs and configuration at ${target}`);
    const config = await Bun.file(configPath).text();
    for (const [index, line] of config.split(/\r?\n/).entries()) {
      if (!line.trim() || line.trimStart().startsWith("#")) continue;
      if (!/^[A-Za-z_][A-Za-z0-9_]*=(?:[yn]|-?\d+|"(?:[^"$`\\\r\n]|\$[A-Za-z_][A-Za-z0-9_]*)*"|'[^'\r\n]*')$/.test(line)) fail(`Refusing non-Kconfig shell syntax in ${configPath}:${index + 1}. Regenerate it with jhalfs menuconfig and use literal values.`);
    }
    for (const option of ["RUNMAKE", "GETPKG", "CLEAN"]) {
      const settings = [...config.matchAll(new RegExp(`^(?:export\\s+)?${option}=(.*)$`, "gm"))].map((match) => match[1]!.trim());
      const unexpected = settings.find((setting) => !["n", `"n"`, `'n'`].includes(setting));
      if (unexpected !== undefined) fail(`Refusing jhalfs generation: ${option} must be unset or explicitly disabled in ${configPath}; found ${unexpected}.`);
    }
    if (!has("--execute")) { console.log(JSON.stringify({ planned: true, cwd: target, command: ["./jhalfs", "run"], runMake: "n (jhalfs default; enabled setting rejected)", note: "This generates ALFS scripts and a Makefile; it does not run the package build." }, null, 2)); process.exit(0); }
    if (process.platform !== "linux") fail("jhalfs generation requires a supported Linux host.");
    const child = Bun.spawn(["bash", "./jhalfs", "run"], { cwd: target, stdin: "pipe", stdout: "inherit", stderr: "inherit", env: { ...process.env, RUNMAKE: "n" } });
    child.stdin.write("yes\nyes\n");
    child.stdin.end();
    process.exit(await child.exited);
  }
  fail(`Unknown alfs action: ${action ?? "<missing>"}`);
}
if (command === "plan" || command === "fetch") {
  const book = value("--book");
  if (!book) fail(`${command} requires --book <official rendered book directory>`);
  const bookRoot = book!;
  const sources = value("--sources", `${root}/artifacts/lfs-sources`)!;
  const listPath = resolve(bookRoot, "wget-list");
  const listFile = Bun.file(listPath);
  if (!(await listFile.exists())) fail(`No wget-list at ${listPath}. Use the official LFS download page for this exact book revision.`);
  const urls = (await listFile.text()).split(/\r?\n/).map((line) => line.trim()).filter((line) => line && !line.startsWith("#"));
  const sourceNames = new Map<string, string>();
  for (const rawUrl of urls) {
    const parsed = validatePublicHttpUrl(rawUrl);
    const name = parsed.pathname.split("/").at(-1);
    if (!name || name === "." || name === "..") fail(`Invalid package URL path: ${rawUrl}`);
    const safeName = name!;
    const previous = sourceNames.get(safeName);
    if (previous && previous !== rawUrl) fail(`Package filename collision: ${safeName} is listed by both ${previous} and ${rawUrl}`);
    sourceNames.set(safeName, rawUrl);
  }
  const md5Path = resolve(bookRoot, "md5sums");
  const md5 = await Bun.file(md5Path).exists() ? (await Bun.file(md5Path).text()).split(/\r?\n/).filter(Boolean).length : null;
  const plan = { bookRoot, sourceCache: sources, packageUrls: urls.length, checksumsListed: md5, downloadCommands: urls.map((url) => `curl --fail --location --continue-at - --output ${JSON.stringify(`${sources}/${url.split("/").at(-1)}`)} ${JSON.stringify(url)}`), verifyCommand: md5 === null ? null : `md5sum --check ${JSON.stringify(md5Path)}`, executionEnabled: command === "fetch" && has("--execute") };
  if (command === "plan" || !has("--execute")) { console.log(JSON.stringify(plan, null, 2)); process.exit(0); }
  if (process.platform === "win32") fail("LFS source preparation and build execution require a Linux host.");
  await Bun.$`mkdir -p ${resolve(sources)}`;
  for (const url of urls) {
    const parsed = validatePublicHttpUrl(url);
    const name = parsed.pathname.split("/").at(-1)!;
    if (!name || name === "." || name === "..") throw new Error(`Invalid package URL path: ${url}`);
    const destination = resolve(sources, name);
    if (await Bun.file(destination).exists()) continue;
    const { response } = await fetchPublicUrl(url, 120000);
    if (!response.ok) throw new Error(`${response.status} fetching ${url}`);
    await Bun.write(destination, response);
    console.log(`Fetched ${name}`);
  }
  if (md5 !== null) {
    const check = Bun.spawn(["md5sum", "--check", md5Path], { cwd: resolve(sources), stdin: "ignore", stdout: "inherit", stderr: "inherit" });
    const code = await check.exited;
    if (code !== 0) throw new Error(`Checksum validation failed (exit ${code})`);
  }
  process.exit(0);
}
if (command === "run") {
  const script = value("--script");
  if (!script) fail("run requires --script <ALFS-generated script>");
  const scriptPath = script!;
  const absoluteScript = resolve(scriptPath);
  const scriptFile = Bun.file(absoluteScript);
  if (!(await scriptFile.exists())) fail(`Script not found: ${absoluteScript}`);
  const scriptBytes = new Uint8Array(await scriptFile.arrayBuffer());
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", scriptBytes));
  const scriptSha256 = [...digest].map((byte) => byte.toString(16).padStart(2, "0")).join("");
  const stepId = value("--step-id", basename(scriptPath).replace(/\.[^.]+$/, ""))!;
  const journalPath = resolve(value("--journal", `${root}/artifacts/lfs-build-state.json`)!);
  if (!has("--execute")) { console.log(JSON.stringify({ planned: true, stepId, script: absoluteScript, scriptSha256, command: ["bash", absoluteScript], journal: journalPath, note: "Pass --execute to run this generated instruction script." }, null, 2)); process.exit(0); }
  if (process.platform === "win32") fail("LFS build scripts require a supported Linux host.");
  type Journal = { schemaVersion: 1; events: Array<{ type: "step-start" | "step-finish"; stepId: string; script: string; sha256: string; at: string; exitCode?: number }> };
  const journalFile = Bun.file(journalPath);
  const journal: Journal = await journalFile.exists() ? await journalFile.json() as Journal : { schemaVersion: 1, events: [] };
  if (journal.schemaVersion !== 1 || !Array.isArray(journal.events)) fail(`Unsupported or invalid build journal: ${journalPath}`);
  const completed = [...journal.events].reverse().find((event) => event.type === "step-finish" && event.stepId === stepId && event.sha256 === scriptSha256 && event.exitCode === 0);
  if (completed) { console.log(JSON.stringify({ skipped: true, stepId, scriptSha256, reason: "The same script already completed successfully in this journal." }, null, 2)); process.exit(0); }
  await Bun.$`mkdir -p ${dirname(journalPath)}`;
  const persist = async () => {
    const temporary = `${journalPath}.tmp`;
    await Bun.write(temporary, `${JSON.stringify(journal, null, 2)}\n`);
    const move = Bun.spawn(["mv", "-f", temporary, journalPath], { stdin: "ignore", stdout: "ignore", stderr: "inherit" });
    if (await move.exited !== 0) throw new Error(`Could not atomically update build journal: ${journalPath}`);
  };
  const event = { stepId, script: absoluteScript, sha256: scriptSha256 };
  journal.events.push({ type: "step-start", ...event, at: new Date().toISOString() });
  await persist();
  const child = Bun.spawn(["bash", "--noprofile", "--norc", absoluteScript], { stdin: "inherit", stdout: "inherit", stderr: "inherit" });
  const exitCode = await child.exited;
  journal.events.push({ type: "step-finish", ...event, at: new Date().toISOString(), exitCode });
  await persist();
  process.exit(exitCode);
}
fail(`Unknown command: ${command}\n\n${help}`);
