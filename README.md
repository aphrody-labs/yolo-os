# YOLO OS / LFS

Rust research and orchestration tools for Linux From Scratch (LFS). The book
remains the authority for package versions, build commands, host requirements,
errata and security notices. This repository records sources and state so each
step stays inspectable and reproducible.

## Build and run

Install Rust 1.85 or newer, then:

```sh
cargo build --locked
cargo run -p cargo-lfs -- --help
```

Install the Cargo subcommand with `cargo install --path crates/cargo-lfs --locked`.
It is then available as `cargo lfs --help`.

`yolo-bun` owns the `yolo pkg` package workflow and native backend adapters.
The `yolo-os` command forwards package operations to that CLI; install the
`yolo` executable on `PATH` or set `YOLO_BUN_CLI` to its path:

```sh
bun install --frozen-lockfile
bun run yolo-os pkg backends
bun run yolo-os pkg manage pacman upgrade
bun run yolo-os pkg fdroid search adaway
bun run yolo-os pkg fdroid install org.adaway --execute
bun run yolo-os pkg image-plan path/to/resolved-profile.json
bun run yolo-os pkg verify-image-plan plan.json artifact-files.json
```

The first-party Bun PID 1 prototype and test initramfs builder live here under
`packages/bunos-init` and `factory/linux`. On a Linux build host with the
listed `cpio`, `gzip`, `mknod` and Bun tools installed, use:

```sh
bun run pid1:check
bun run pid1:build-initramfs -- ./dist/bunos-initramfs.cpio.gz
```

The initramfs is an experiment. It does not switch to an LFS root, launch the
full service set, or carry a boot qualification. AOT is selected only when the
chosen Bun exposes the AOT flags; the AOT checker currently requires the
upstream canary lane.

Capture current research sources:

```sh
cargo lfs crawl --depth 1 --max-pages 60
```

The crawler respects `robots.txt`, stays within seed origins, limits response
size and page count, and writes JSON evidence plus linked Markdown under
`artifacts/`. Pass URLs after `crawl` to override the seed list.

F-Droid commands are forwarded to `yolo-bun` and then to an already installed
`fdroidcl`/ADB setup. They manage apps on connected Android devices. Plans are
printed by default; add `--execute` to run the native operation. The adapter
does not install the F-Droid CLI or add signature verification; review
`fdroidcl`'s repository trust model before executing.

## LFS workflow

Downloading, planning, generation and execution are separate steps. The official
book supplies package sources and checksums; official ALFS/jhalfs extracts the
book's structured instructions. Rust handles research, provenance and
orchestration. Package build recipes remain the commands specified by upstream
LFS and each package.

```sh
cargo lfs book sync --track stable-systemd
cargo lfs plan --book ./artifacts/lfs-book/stable-systemd
cargo lfs fetch --book ./artifacts/lfs-book/stable-systemd --execute
cargo lfs alfs setup --dir ./work/jhalfs --execute
# Configure the selected book and leave RUNMAKE, GETPKG, CLEAN disabled.
cargo lfs alfs generate --dir ./work/jhalfs --execute
cargo lfs run --script path/to/reviewed-generated-script.sh
```

`run` prints a plan unless `--execute` is given and journals script digests,
timestamps and exit status for step-level continuation. Use a disposable Linux
VM and a dedicated virtual disk: LFS instructions can format partitions and
install a bootable system. This tool does not select or format a disk.

## LFS principles in this project

- Follow one selected book revision from preparation through the final system.
- Preserve upstream commands and expose their inputs, outputs and provenance.
- Make state changes visible, resumable and reviewable one step at a time.
- Keep the resulting system minimal; LFS is a starting point for learning and
  building a system, not a preconfigured general-purpose distribution.
- Use Rust for the tools around the build, without disguising package recipes
  behind a second, unverified build engine.

## Project map

- [`crates/cargo-lfs/src/main.rs`](crates/cargo-lfs/src/main.rs): Rust CLI,
  crawling, source retrieval, manifest checks and execution journal.
- [`src/yolo.ts`](src/yolo.ts): Bun `yolo-os pkg` command bridge to the
  `yolo-bun` package CLI; it does not implement a second resolver or installer.
- [`docs/bun-package-orchestration.md`](docs/bun-package-orchestration.md):
  backend ownership and package-plan contract.
- [`docs/`](docs/README.md): source research, architecture, CLI and LFS method.
- [`docs/awesome-rust-lfs.md`](docs/awesome-rust-lfs.md): selected Rust
  references and the `aphrody-dev/awesome-rust` LFS curation branch.
- [`docs/bunos-lfs-integration-plan.md`](docs/bunos-lfs-integration-plan.md):
  source-built LFS path into Bun OS with Bun PID 1, uutils and sudo-rs.
- [`sources.json`](sources.json): research seeds and companion Git remotes.
- `artifacts/`: generated, ignored evidence and package cache.

## Platform and scope

Research and planning can run on Windows. Package acquisition with the selected
checksum verifier and build execution are intended for Linux. The CLI does not
replace the LFS book, validate every host requirement, or claim that a build
succeeded. Run LFS within Linux or a VM with its own dedicated target disk.
