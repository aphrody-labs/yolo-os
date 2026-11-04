#!/usr/bin/env bun
// SPDX-License-Identifier: MIT
// Yolo OS forwards package operations to yolo-bun, the single package CLI.

const args = Bun.argv.slice(2);
export {};
if (args[0] !== "pkg" || args.length < 2) {
  console.error(`Usage:
  yolo-os pkg backends
  yolo-os pkg install <backend> <package> [--execute]
  yolo-os pkg resolve <backend> <package>
  yolo-os pkg image-plan <resolved-profile.json>
  yolo-os pkg verify-image-plan <plan.json> <files.json>

Package resolution and native backend policy are owned by yolo-bun.`);
  process.exit(2);
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
