# Bun package workflow for yolo-os

Status: architecture contract, 2026-11-04. This records the user's direction
that Bun should provide the package workflow for yolo-os while preserving the
native responsibilities of the package ecosystems yolo-os needs to integrate.

## Ownership

- **`yolo-bun`** provides the `yolo pkg` CLI and manages Bun/JavaScript
  workspace dependencies with `package.json`, workspaces, and its committed
  `bun.lock`.
  Use the upstream Bun package-manager behavior for install, update, audit,
  dedupe, pruning, and offline/cache operations.
- **`yolo-os pkg`** forwards to the `yolo` executable shipped by `yolo-bun`.
  The bridge contains no package resolver or installer of its own. `yolo-bun`
  produces plans, invokes selected backends, and records verification receipts.
- **Native backends** retain their package identities, resolver, package
  database, signature policy, and transaction behavior. The orchestrator
  records the requested package and the backend's exact result; it does not
  reinterpret backend metadata as a universal version scheme.
- **`cargo-lfs`** remains the source-build, image-assembly, and receipt
  orchestrator. It is not an interactive package manager. It may consume a
  package plan and record the build/image result.

## Backend map

| Package domain                                              | Backend                                               | Boundary                                                                                                                                                                         |
| ----------------------------------------------------------- | ----------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Bun/TypeScript dependencies in the yolo-os source workspace | Bun                                                   | `bun.lock` owns the JS dependency graph. Do not use it as the system package database.                                                                                           |
| Rust crates and Rust tools                                  | Cargo                                                 | Cargo manifests and `Cargo.lock` own the Rust dependency graph and build.                                                                                                        |
| Python tools used during development or builds              | uv                                                    | `pyproject.toml` and `uv.lock` own the Python environment; export lock data when an SBOM or another tool needs it.                                                               |
| C/C++ libraries for a project/toolchain                     | vcpkg manifest mode                                   | `vcpkg.json`, baseline, registries, and port versions own the native library graph.                                                                                              |
| Base OS packages on Arch/CachyOS/Omarchy images             | pacman and the image's configured repositories        | pacman owns package state, repositories, signature checks, and transactions in image builds. Host lifecycle rules can add a distribution-specific command layer.                   |
| Desktop application/runtime distribution                    | Flatpak; Flathub as one remote                        | Flatpak manifests, runtimes, remotes, repository refs, and sandbox permissions remain authoritative.                                                                             |
| Android application repositories                            | `fdroidcl` with configured repositories and ADB       | Native client and repository trust policy remain authoritative; the adapter adds no signature verification. This backend is for Android apps, not the Android SDK/NDK toolchain. |
| Windows application distribution                            | Windows Package Manager (`winget`)                    | Winget package identifiers, configured sources, manifests, import/export, and installer behavior remain authoritative. It does not replace MSVC or Windows SDK management.       |
| Cross-compilation toolchains and SDKs                       | `yolo-cross` tooling and its native package providers | Resolve and qualify compiler/toolchain inputs before they are used in OS package builds.                                                                                         |

The `yolo pkg install` adapter currently supports install planning and explicit
execution for Bun, Cargo, uv, vcpkg, pacman, Flatpak/Flathub, WinGet, and F-Droid
via `fdroidcl`. F-Droid additionally maps search, show, index update, upgrade,
remove, devices, and repo listing. Only Bun and vcpkg expose dependency dry-run
resolution. `yolo pkg manage` adds native search and installed-list commands
for supported managers, plus upgrade/remove plans. Search works through the
native CLIs for Flatpak, WinGet, Cargo, pacman, and vcpkg; Bun and uv search is
rejected because those CLIs do not expose package search. Installed listings
are available for Bun, uv, Cargo, Flatpak, WinGet, pacman, and vcpkg. Read-only
search/list operations run directly. Upgrade/remove print plans and only run
with `--execute`. Bun/uv operations target global tools; Flatpak is per-user;
WinGet scope follows each installer; pacman upgrade is system-wide; and vcpkg
upgrade rebuilds all outdated Classic-mode packages without rollback. `backends`
only detects executable/version availability; it does not probe repositories,
profile compatibility, signatures, devices, or installability. Cross-backend
solving, staging-root install, and recovery orchestration are not implemented
by this CLI. Only enable a backend on an image after its source trust, version
pinning, license metadata, offline behavior, and recovery path have been
qualified.

CachyOS publishes ISA-specific repositories (including x86-64-v3, x86-64-v4,
and Zen-specific tiers) through pacman configuration. The package profile must
not select an optimized repository until the target CPU's supported ISA is
verified; keep the generic x86-64 repository as a fallback. Omarchy packages
use pacman repositories, but Omarchy documents `omarchy update` as the blessed
host update flow; its ALPM guard rejects direct `pacman -Syu` unless explicitly
bypassed. Use native Omarchy package helpers for host package changes. A yolo-os
image build may use pacman inside its controlled target root; it must not
present that image operation as an Omarchy host update. These remain
distribution repository policies, not Bun dependency-resolution rules.

## Package profile and receipts

The `yolo-bun` command `yolo pkg verify-image-plan` accepts its generated image plan and a JSON mapping from artifact IDs to local paths. It checks the plan digest and local SHA-256 hashes before image assembly. The receipt proves byte matching only; `cargo-lfs` integration, backend signatures and image qualification remain separate work.

A `yolo-os` image profile uses `yolo-bun` image-plan schema v2 and records:

- a stable profile key, target OS, architecture, and channel;
- a target distribution such as Arch, CachyOS, or Omarchy, with release and
  descriptive CPU tier where applicable;
- source records with a stable ID, owning backend, source kind, and native
  source reference;
- for every artifact, the native backend and package/app/port identifier, a
  source ID that must belong to that same backend, pinned version, HTTPS URL,
  and SHA-256;
- license and trust-policy references.

`yolo-bun` validates source ownership, produces canonical plans, artifact
digests, and file-hash receipts. Dependency/conflict metadata and repository
signature verification are not implemented in the plan validator. The OS
profile consumes native package identities; it does not create another
package name/version namespace or configure repositories.

The aggregate yolo-os image manifest may list results from several ecosystems,
but each ecosystem's native lock, repository metadata, and signing remains the
authority for its own package graph. The aggregate receipt joins those
identities to the image digest and build record; it does not silently replace
them.

The bridge accepts either an installed native `yolo` executable on `PATH` or
`YOLO_BUN_CLI` pointing to one. During development, `YOLO_BUN_CLI` may point
directly to the `yolo-bun` `scripts/yolo.ts` entrypoint; the bridge runs that
file with Bun and passes each argument as a separate process argument. It
inherits stdin/stdout/stderr and returns the child exit code. It never invokes a
shell. If the CLI is unavailable, it reports the missing executable and exits
127; invalid bridge syntax exits 2.

Example from a checkout where both repositories are present:

```powershell
$env:YOLO_BUN_CLI = 'C:\src\yolo-bun\scripts\yolo.ts'
bun run yolo-os pkg backends
```

## Command contract

The `yolo-bun` CLI exposes these commands; `yolo-os pkg` forwards to it:

```text
yolo pkg backends
yolo pkg install <backend> <package> [--execute]
yolo pkg manage <backend> <search|list|installed|upgrade|remove> [query-or-package] [--execute]
yolo pkg fdroid <search|show|update|install|upgrade|remove|devices|repos> [arguments...] [--execute]
yolo pkg resolve <backend> <package>
yolo pkg image-plan <resolved-profile.json>
yolo pkg verify-image-plan <plan.json> <files.json>
```

`backends`, `image-plan`, and `verify-image-plan` are read-only. The schema-v2
profile supplies a structured target, source records, and artifacts with
native package IDs; an artifact source must match its backend. `install`
prints a native invocation by default and executes only with `--execute`.
`manage` runs supported read-only searches/lists directly. Upgrade and remove
print a plan and run only with `--execute`; pacman upgrade is explicitly a
system-wide `-Syu`, while vcpkg upgrade is all outdated classic ports.
`resolve` delegates only to the native Bun and vcpkg dry-runs currently
implemented. `yolo pkg fdroid` maps search, show, index update, install,
upgrade, removal, device listing and repository listing to `fdroidcl`; all
operations are planned by default and need `--execute` to run. `fdroidcl` and
ADB must already be installed and configured. The tool manages connected
Android devices, not host packages. Its upstream documentation says index
verification uses HTTPS rather than JAR signatures; this adapter preserves
that behavior and does not add or claim signature verification. General
dependency solving, image-stage transactions, signature receipts, and rollback
remain roadmap work.

The `yolo-bun` CLI now probes native backends, plans or invokes explicit package
installs, creates canonical image plans from already resolved artifacts, and
streams local artifact hashes through `Bun.CryptoHasher`. It does not resolve a
full dependency graph or validate repository signatures. Add backend
resolution only when it can return structured results and retain native locks.
Image assembly should consume the same plan in a staged root.

## Bun 1.4 build and quality policy

Use Bun 1.4's native `bun check`, `bun lint`, `bun fmt`, `bun test`, and Bun
package-manager commands in the Bun-owned CLI/workspaces. For bundled tools,
use `bun build --compile --bytecode`; record startup and memory benchmarks
before changing runtime flags. Profile-guided bytecode ordering can optimize
the observed startup path. `bun compile` / `--aot` is experimental and builds
for the host platform, so use it only in a matching-platform lane after
`bun check --aot` and runtime parity checks. It is not a Windows/Linux/Android
cross-compiler. Rust and C/C++ LTO remains a native toolchain setting in
`yolo-cross`; do not imply that Bun's JS AOT flag enables linker LTO.

Keep upstream Bun API/Node compatibility tests and yolo-bun's package install,
update, offline, conflict, signature, failure, and rollback tests separate.
Performance goals use reproducible baselines for startup,
steady-state CPU/RSS, install resolution, and package-plan execution. Do not
accept a fork optimization based only on a microbenchmark or a successful
compile.

## Research sources

Checked 2026-11-04. These links describe upstream contracts; no upstream source
code was copied into this document.

- Bun 1.4 package manager, test, check, lint, fmt and AOT/bytecode interfaces:
  [Bun 1.4](https://bun.sh/blog/bun-v1.4),
  [Bun v1.4.4](https://bun.sh/blog/bun-v1.4.4),
  [type checking](https://bun.sh/docs/runtime/check),
  [lint](https://bun.sh/docs/lint),
  [compiler](https://bun.sh/docs/compiler),
  [standalone executables](https://bun.sh/docs/compiler/executables),
  [AOT troubleshooting](https://bun.sh/docs/compiler/troubleshooting).
- Flatpak app/runtime repositories, build manifests, and sandboxing:
  [under the hood](https://docs.flatpak.org/en/latest/under-the-hood.html),
  [first build](https://docs.flatpak.org/en/latest/first-build.html),
  [Flathub documentation](https://docs.flathub.org/).
- F-Droid app repository tooling and reproducibility:
  [F-Droid documentation](https://f-droid.org/en/docs/),
  [reproducible builds](https://f-droid.org/docs/Reproducible_Builds/).
- Windows app package identity and restoring an app set:
  [winget export](https://learn.microsoft.com/en-us/windows/package-manager/winget/export).
- Language/native dependency managers:
  [Cargo packages](https://doc.rust-lang.org/cargo/reference/packages.html),
  [uv locking and syncing](https://docs.astral.sh/uv/concepts/projects/sync/),
  [vcpkg manifests](https://learn.microsoft.com/en-us/vcpkg/users/manifests),
  [Arch package signing](https://wiki.archlinux.org/title/Pacman/Package_signing).
- Distribution-specific sources to qualify before adoption:
  [CachyOS optimized repositories and CPU tiers](https://github.com/CachyOS/linux-cachyos),
  [Omarchy package/update model](https://github.com/omacom/omarchy/blob/quattro/manual/30-updates.md),
  [Omarchy additional package sources](https://github.com/omacom/omarchy/blob/quattro/manual/29-other-packages.md).
