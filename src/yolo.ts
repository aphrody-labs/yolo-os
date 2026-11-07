#!/usr/bin/env bun
// SPDX-License-Identifier: MIT
// Aphrody OS / YOLO OS CLI:
// - Manages 2026 GoogleBook Laptop (Aluminium OS + Chromium OS + Android 17 x64) profiles & crosh diagnostics
// - Forwards package operations (`yolo-os pkg ...`) to yolo-bun, the single package CLI.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const args = Bun.argv.slice(2);
const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const profilesDir = join(repoRoot, "profiles");

function printUsage(): never {
  console.error(`Usage:
  yolo-os profile list
  yolo-os profile show [aphrody-googlebook-2026|rust-userland] [--json]
  yolo-os googlebook [--json]
  yolo-os crosh [status|vmc|arcvm|apps]
  yolo-os pkg backends
  yolo-os pkg install <backend> <package> [--execute]
  yolo-os pkg resolve <backend> <package>
  yolo-os pkg image-plan <resolved-profile.json>
  yolo-os pkg verify-image-plan <plan.json> <files.json>

Aphrody OS (2026 GoogleBook Laptop) combines Android 17 x64 + Chromium OS + Aluminium OS + Linux 7.4-rc + Upstream Bun v1.4.4+.
Package resolution and native backend policy are owned by yolo-bun.`);
  process.exit(2);
}

function loadGoogleBookProfile(): Record<string, unknown> {
  const profilePath = join(profilesDir, "aphrody-googlebook-2026", "profile.json");
  return JSON.parse(readFileSync(profilePath, "utf8")) as Record<string, unknown>;
}

if (args.length === 0 || args[0] === "--help" || args[0] === "-h") {
  printUsage();
}

if (args[0] === "profile" || args[0] === "googlebook") {
  const sub = args[0] === "googlebook" ? "show" : (args[1] ?? "list");
  if (sub === "list") {
    const entries = existsSync(profilesDir)
      ? readdirSync(profilesDir, { withFileTypes: true })
          .filter((d) => d.isDirectory())
          .map((d) => d.name)
      : [];
    console.log(JSON.stringify({ profiles: entries, default: "aphrody-googlebook-2026" }, null, 2));
    process.exit(0);
  }
  if (sub === "show") {
    const target =
      args[0] === "googlebook"
        ? "aphrody-googlebook-2026"
        : args[2] && !args[2].startsWith("--")
          ? args[2]
          : "aphrody-googlebook-2026";
    const jsonFlag = args.includes("--json");
    const candidateProfile = join(profilesDir, target, "profile.json");
    const candidateMatrix = join(profilesDir, target, "matrix.json");
    const filePath = existsSync(candidateProfile)
      ? candidateProfile
      : existsSync(candidateMatrix)
        ? candidateMatrix
        : null;
    if (!filePath) {
      console.error(`Unknown profile: ${target}`);
      process.exit(1);
    }
    const data = JSON.parse(readFileSync(filePath, "utf8")) as Record<string, any>;
    if (jsonFlag || !data.androidSubsystem) {
      console.log(JSON.stringify(data, null, 2));
      process.exit(0);
    }
    console.log(`${data.name} (${data.version})`);
    console.log(`  Organization : ${data.organization} (${data.domain}) | Core Library: ${data.coreLibraryScope}`);
    console.log(`  Kernel       : ${data.kernel.version} [${data.kernel.arch.join(", ")}]`);
    console.log(`  Init/Runtime : ${data.initAndRuntime.pid1}`);
    console.log(
      `  Android 17   : ${data.androidSubsystem.version} (API ${data.androidSubsystem.apiLevel} ${data.androidSubsystem.codename}) ${data.androidSubsystem.abi} — ${data.androidSubsystem.preinstalledApps.length} preinstalled Google/Aphrody apps`,
    );
    console.log(`  Chromium OS  : ${data.chromiumOsServices.cli} + ${data.chromiumOsServices.daemons.map((d: any) => d.name).join(", ")}`);
    console.log(`  Aluminium OS : ${data.aluminiumOsShell.architecture}`);
    process.exit(0);
  }
  printUsage();
}

if (args[0] === "crosh") {
  const topic = args[1] ?? "status";
  const profile = loadGoogleBookProfile() as Record<string, any>;
  if (topic === "status" || topic === "vmc" || topic === "arcvm") {
    console.log(
      JSON.stringify(
        {
          shell: "crosh (Aphrody OS / 2026 GoogleBook Laptop)",
          topic,
          kernel: profile.kernel.version,
          chromiumOsServices: profile.chromiumOsServices,
          androidSubsystem: {
            version: profile.androidSubsystem.version,
            apiLevel: profile.androidSubsystem.apiLevel,
            abi: profile.androidSubsystem.abi,
            imageFlavor: profile.androidSubsystem.imageFlavor,
            isolationRuntime: profile.androidSubsystem.isolationRuntime,
          },
          aluminiumOsShell: profile.aluminiumOsShell.codename,
        },
        null,
        2,
      ),
    );
    process.exit(0);
  }
  if (topic === "apps") {
    console.log(JSON.stringify(profile.androidSubsystem.preinstalledApps, null, 2));
    process.exit(0);
  }
  printUsage();
}

if (args[0] !== "pkg" || args.length < 2) {
  printUsage();
}

const executable = Bun.env.YOLO_BUN_CLI || Bun.which("yolo");
if (!executable) {
  console.error(
    "yolo-bun CLI not found; install its yolo executable or set YOLO_BUN_CLI.",
  );
  process.exit(127);
}

// A development checkout can point YOLO_BUN_CLI at yolo-bun's TypeScript
// entrypoint. Run it with Bun directly; never interpolate a command through a
// shell. Installed native `yolo` binaries continue to run as-is.
const sourceEntrypoint = /\.(?:[cm]?js|tsx?)$/i.test(executable);
const bun = sourceEntrypoint ? Bun.which("bun") : undefined;
if (sourceEntrypoint && !bun) {
  console.error("Bun is required to run the yolo-bun source CLI.");
  process.exit(127);
}

const child = Bun.spawn(
  sourceEntrypoint ? [bun!, "run", executable, ...args] : [executable, ...args],
  {
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  },
);
process.exitCode = await child.exited;
