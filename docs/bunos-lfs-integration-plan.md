# Bun OS integration plan, built with LFS

Status: design plan, 2026-11-04. The legacy Bun OS source archive is inspected
read-only at `C:\src\bun-os-archive`; this plan does not modify the archive.
`yolo-os` owns the current OS distribution, Linux boot code, Rust research and
build-orchestration CLI.

## Goal

Build one minimal, source-built LFS base and extend it into the Bun OS image
with a pinned Rust-based Bun runtime, Rust userland packages, a first-party
PID 1 and the Bun OS kernel package. Preserve the LFS book's package order,
commands, checksums, errata and host requirements. Rust changes how Bun OS
packages and records the build; it does not silently replace an upstream LFS
recipe.

The system is successful only when the declared image builds from pinned
sources, boots the patched kernel and reaches the intended PID 1. Source
presence, package compilation, initramfs boot, and full rootfs boot are separate
evidence levels.

## Bun OS starting point found in the checkout

These are repository records, not independently repeated build results:

| Area               | Existing Bun OS integration                                                                                                                                                                                                              | Plan interpretation                                                                                                                                                                                                                                                                                                                  |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Bun                | Runtime pin `1.4.4-canary.20261028.1` and Arch `bun` package are recorded in `docs/UNIFIED-IMAGE.md` and `factory/rustbase/pins.json`. Bun 1.4 is Rust-based; upstream has a large Cargo workspace with internal crates.                 | Build the pinned upstream runtime from source and record its Rust toolchain, submodules/vendor inputs and non-Rust native components. Do not equate Bun's Rust implementation with a crates.io dependency.                                                                                                                           |
| Userland utilities | `crates/bunos-utils` consumes uutils coreutils/findutils/diffutils and optional util-linux/procps crates from pinned nested workspaces. `factory/rustbase` creates a multicall package and a smoke script.                               | Make this a first-party Bun OS package assembled from pinned upstream Rust projects. Keep GNU implementations available for commands still under compatibility/security review.                                                                                                                                                      |
| Privilege          | `bunos-sudo-rs` packages sudo-rs 0.2.15, PAM policy files, `sudo` and `su-rs` setuid binaries, and `visudo`.                                                                                                                             | Keep sudo-rs as a first-party, root-owned OS package with explicit PAM and sudoers inputs; qualify privilege transitions inside an isolated image.                                                                                                                                                                                   |
| PID 1              | `packages/bunos-init/src/init.ts` is a first-party Bun program compiled by `factory/linux/build-initramfs.sh` into `/init` in a test initramfs. It mounts proc/sys/dev/run/tmp, reaps children, handles shutdown and can probe binderfs. | Keep Bun as the actual userland PID 1 runtime. Build Bun from its Rust upstream workspace, then use `bun run pid1:build-initramfs -- <output.cpio.gz>`; the script enables AOT/trimming when the selected Bun supports it. Preserve the current boot modes and qualify the final standalone artifact and its native-library closure. |
| Kernel             | `factory/kernel/linux-bun/PKGBUILD` builds an Arch-derived, Rust-enabled kernel and merges Bun OS and Android config fragments. The product checklist records a 7.4-rc1 package and QEMU PID 1/Android receipts dated 2026-11-03.        | Reconcile the newer product record with the older `factory/kernel/linux-bun/README.md` before calling the kernel qualified. Pin a stable kernel for the release lane; keep release candidates in a separate qualification lane. Keep kernel code patches separate from Kconfig/package changes.                                      |

The kernel roadmap says Bun remains a userspace runtime. Do not embed
JavaScriptCore, Bun, or a general scripting engine in ring 0. A kernel patch is
accepted only for a demonstrated kernel need and stays independently buildable
without the userland.

## Target image layers

```text
LFS book + checksummed sources
       │
       ├── pinned toolchain and base libraries
       ├── stable linux-bun kernel + reviewed Bun OS config/patch series
       └── Rust first-party packages
             ├── Bun runtime + first-party bunos-init (PID 1 and ordinary userland)
             ├── bunos-utils (uutils multicall and selected tools)
             └── bunos-sudo-rs (PAM, sudoers, sudo/su-rs)
       │
signed image manifest + initramfs + rootfs + boot receipt
```

The first root filesystem stays minimal. Desktop, Android, browser, app and
server compositions are later profiles with declared dependencies; they do not
enter the LFS bootstrap closure by default.

## Phased plan and gates

### 0. Freeze ownership, pins and evidence

- Treat the selected LFS book edition as a single immutable input set. Store
  book revision, `wget-list`, checksum manifest, errata/advisory snapshot and
  retrieval digests together.
- Reconcile Bun OS records for Bun, uutils, sudo-rs, `linux-bun`, PID 1 and
  QEMU receipts. Resolve the 7.2.11/no-boot kernel README against the later
  7.4-rc1/product record; preserve both records with their dates until the
  owning Bun OS team updates the source of truth.
- Pin source tag, exact commit, license, submodules/vendor closure, toolchain,
  patches and generated package recipe for each component. No floating branch
  or unverified binary enters the image.

**Gate:** an auditable source lock and one named owner for every package and
image profile.

### 1. Build the LFS foundation

- Use the official LFS book and ALFS/jhalfs output for the base toolchain,
  libraries, utilities and kernel prerequisites. Preserve upstream build
  instructions; store the selected book and ALFS commits next to the logs.
- Use the matching book checksums for source downloads and separate package
  build logs from image assembly logs. The Rust `cargo-lfs` CLI plans and
  records these operations; it does not translate package recipes into Rust.
- Keep a plain LFS boot target before adding Bun OS packages. This provides a
  known-good recovery image and makes regressions attributable.

**Gate:** reproducible LFS base rootfs boots in QEMU, with source and artifact
hashes recorded.

### 2. Qualify the kernel and initramfs independently

- Rebuild the current `linux-bun` package from a stable upstream kernel release
  plus a pinned Arch packaging revision, Bun OS configuration fragments and
  only justified patches. Keep the RC kernel as an optional early lane.
- Run Kconfig merge checks and `make rustavailable`; build the selected
  architecture/configuration. Record kernel, config, patch and initramfs
  digests. Test secure boot/signing policy and rollback before promotion.
- Build the pinned Bun Rust runtime from source, then compile the first-party
  `packages/bunos-init/src/init.ts` entry into `/init` using the upstream Bun
  compiler through `factory/linux/build-initramfs.sh`. Keep the existing PID 1 responsibilities: mount required
  pseudo-filesystems, set hostname, adopt and reap orphaned children, handle
  PID 1 signals and perform orderly shutdown/reboot. Add root discovery,
  validation and switch-root for the complete LFS image. A failure path reports
  to the console and halts safely; PID 1 never exits into an undefined state.
- Boot with `rdinit=/init` (or the selected kernel's equivalent) under QEMU.
  First qualify the basic path, then binderfs/Android as a separate profile.
  The kernel patch is not a precondition for Bun or uutils unless a measured
  feature specifically requires it.

**Gate:** repeatable QEMU receipts for initramfs PID 1, root switch and shutdown;
separate receipts for every enabled kernel feature. No acceptance based solely
on a package build or a stale roadmap line.

### 3. Promote Bun and Rust userland packages

- Build Bun 1.4 from its upstream Rust workspace and pinned native dependencies.
  Preserve upstream build system and license notices; do not import Bun's
  internal path crates into `cargo-lfs` or the LFS tool workspace. Use that
  build to compile the first-party Bun PID 1 entry as a standalone artifact.
- Build `bunos-utils` from pinned uutils sources. Install prefixed tools first
  (for example, `uu-ls`) and collect GNU compatibility results by utility.
  Promote bare command names only after image-level smoke and package-conflict
  checks. Keep GNU `cp`, `mv`, `rm`, `install` and `stty` as fallbacks until
  race, symlink, metadata and LFS/Arch packaging compatibility is demonstrated.
- Build sudo-rs against the pinned PAM stack. Audit ownership and mode of
  `/usr/bin/sudo`, `su-rs`, sudoers and PAM files; run positive and negative
  privilege tests in a disposable VM. Keep a recovery path that can restore
  root access if policy or PAM configuration fails.
- Keep applets, system services and optional shells as separately declared
  packages. Do not promote experimental uutils sibling projects merely because
  they compile.

**Gate:** each package has a source lock, license record, isolated build log,
functional compatibility report and install/rollback evidence in the target
rootfs.

### 4. Assemble and qualify the full image

- Compose the LFS base, stable `linux-bun`, Bun PID 1, selected
  `bunos-utils` commands and sudo-rs into a signed package/image manifest.
- Boot the final image in QEMU/OVMF, then target hardware. Capture kernel and
  initramfs hashes, kernel command line, PID 1 identity, mount table, service
  state, privilege checks, utility compatibility report, console log and
  shutdown/reboot result.
- Verify upgrade and rollback from one pinned image to the next. Keep the
  fallback LFS boot entry until the Bun OS image passes the full matrix.

**Gate:** a second clean build from the same source lock produces equivalent
package identities, and the final image passes boot/runtime qualification.

### 5. Submit only narrowly justified kernel patches

- Separate downstream distribution patches from upstream proposals. Each
  kernel patch solves one documented problem, has a reproducer and tests, builds
  across relevant config values and has maintainer routing.
- Do not make `bun_accel` a release dependency: the inspected upstream Bun
  runtime has no consumer for it. Keep it as an optional research branch until
  a real userspace client and measurable benefit exist.
- Follow kernel subsystem review, `checkpatch.pl`, build/selftest coverage,
  licensing and disclosure rules. A patch remains downstream unless accepted
  upstream; an RFC or a local fork is not an upstream merge.

**Gate:** userland still boots without optional kernel experiments, and every
shipped patch is pinned, documented, tested and independently revertible.

## Rust CLI work in `yolo-os`

`cargo-lfs` is the Rust source-build and image orchestration/control plane.
The Bun-owned `yolo pkg` user interface and its native package-manager adapter
boundary are defined in [`bun-package-orchestration.md`](bun-package-orchestration.md).
This keeps Bun as the user-facing package workflow while `cargo-lfs` records
source builds, image assembly, and receipts; neither replaces the native
backend package databases or signature policies. Extend `cargo-lfs` only after
the plan and package contracts are stable:

- Add a versioned source lock for BunOS packages and kernel/boot artifacts.
- Emit a read-only build plan with dependency closure, license, patch stack,
  checksums, host requirements and expected output paths.
- Add explicit commands for source fetch, isolated build, rootfs assembly and
  receipt collection; each side-effecting command requires an explicit
  execution flag and writes a digest-linked journal.
- Keep LFS book acquisition, ALFS generation, BunOS package builds and kernel
  image qualification as separate stages with separate receipts.
- Do not turn `cargo-lfs` into a second package manager or rewrite Bun OS's
  image factory. Consume package plans and image interfaces through the
  declared contracts after their owning repositories define them.

## Open decisions

1. Which LFS edition and architecture is the canonical first image?
2. Which native libraries does the standalone Bun PID 1 need in the initramfs
   for each supported boot mode (`selftest`, `serve`, `android`)?
3. Which current `linux-bun` source and QEMU receipts supersede the dated README
   status, and which kernel patches are required for the first image?
4. Which uutils applets may own bare command names in the first release, and
   which remain prefixed beside GNU implementations?
5. What signing and update/rollback mechanism protects kernel, initramfs and
   rootfs as one bootable release?

Until those inputs are reconciled, the current source records establish a
credible integration direction, not a qualified LFS-derived Bun OS release.
