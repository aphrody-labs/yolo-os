// SPDX-License-Identifier: Apache-2.0
// Locates, builds and loads the native providers of @yolo/windows.
//   win32 / registry / services / shell  bun:ffi, nothing to build
//   winrt                                Node-API addon on windows-rs (crates/windows-native-napi)
//   winui                                .NET host process (native/winui-host), see winui.ts
import { existsSync, mkdirSync, copyFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { assertWindows, WindowsWinrtError } from "./errors";

export interface NapiAddon {
  winrtUriParse(url: string): {
    scheme: string;
    host: string;
    port: number;
    path: string;
    query: string;
    fragment: string;
    absolute: string;
  };
  winrtFolderInfo(path: string): {
    name: string;
    displayName: string;
    path: string;
    createdMs: number;
  };
  winrtDeviceFamily(): string;
  providerVersion(): string;
}

const crateDir = resolve(
  import.meta.dir,
  "../../../crates/windows-native-napi",
);
export const defaultAddonDir = (): string =>
  process.env.YOLO_WINDOWS_NAPI_DIR ??
  process.env.APHRODY_WINDOWS_NAPI_DIR ??
  join(tmpdir(), "yolo-windows-napi");
export const addonPath = (dir: string = defaultAddonDir()): string =>
  join(dir, "yolo_windows_napi.node");

export const isAddonBuilt = (dir?: string): boolean =>
  existsSync(addonPath(dir));

/** cargo build --release of crates/windows-native-napi, then copies the cdylib to a .node file. */
export async function buildAddon(
  outDir: string = defaultAddonDir(),
): Promise<string> {
  assertWindows();
  const target =
    process.env.CARGO_TARGET_DIR ?? join(tmpdir(), "yolo-windows-napi-target");
  const proc = Bun.spawn(
    [
      "cargo",
      "build",
      "--release",
      "-p",
      "yolo-windows-napi",
      "--manifest-path",
      join(crateDir, "Cargo.toml"),
    ],
    {
      env: { ...process.env, CARGO_TARGET_DIR: target },
      stdout: "inherit",
      stderr: "inherit",
    },
  );
  if ((await proc.exited) !== 0)
    throw new WindowsWinrtError(
      "ERR_WINDOWS_PROVIDER_MISSING",
      "cargo build of yolo-windows-napi failed",
    );
  mkdirSync(outDir, { recursive: true });
  copyFileSync(
    join(target, "release", "yolo_windows_napi.dll"),
    addonPath(outDir),
  );
  return addonPath(outDir);
}

let cached: NapiAddon | undefined;

/** Loads the addon with Bun's Node-API support; throws ERR_WINDOWS_PROVIDER_MISSING when it was not built. */
export function loadAddon(
  path: string = process.env.YOLO_WINDOWS_NAPI ??
    process.env.APHRODY_WINDOWS_NAPI ??
    addonPath(),
): NapiAddon {
  assertWindows();
  if (cached) return cached;
  if (!existsSync(path)) {
    throw new WindowsWinrtError(
      "ERR_WINDOWS_PROVIDER_MISSING",
      `WinRT provider not built (${path}); run \`bun run build:napi\` in packages/windows`,
    );
  }
  cached = createRequire(import.meta.url)(path) as NapiAddon;
  return cached;
}

if (import.meta.main && process.argv[2] === "build") {
  console.log(await buildAddon());
}
