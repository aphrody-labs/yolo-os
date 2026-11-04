# YOLO OS / LFS

Bun-first research and orchestration tools for Linux From Scratch (LFS), with a
small Cargo subcommand launcher. The repository keeps upstream facts linked to
their sources and stores crawler output with timestamps and SHA-256 digests.

## Quick start

Install Bun and Rust, then install the workspace packages:

```sh
bun install
bun run lfs -- help
cargo run -p cargo-lfs-bun -- --help
```

To install the Cargo subcommand into Cargo's configured binary directory, run
`cargo install --path crates/cargo-lfs-bun --locked`, then invoke
`cargo lfs-bun --help`. Keep this checkout available to the launcher, or set
`YOLO_OS_BUN` when Bun is not on `PATH`.

Capture the current source set:

```sh
bun run lfs -- crawl --depth 1 --max-pages 60
```

The crawler respects `robots.txt`, stays within seed origins, limits response
size and page count, and writes JSON evidence plus a linked Markdown extraction
under `artifacts/`. Pass one or more URLs after `crawl` to override the seed list.

## LFS workflow

The current LFS book, its matching package versions, errata, advisories and host
requirements are authoritative. Downloading and building are separate steps.
`book sync` retrieves the official `wget-list` and `md5sums`; `plan` is
read-only. `fetch --execute` downloads the package sources and checks their
official MD5 list. LFS construction runs on Linux. For full book
automation, clone official ALFS/jhalfs with `alfs setup`, configure the exact
book revision with `RUNMAKE`, `GETPKG` and `CLEAN` disabled, then generate
scripts with `alfs generate`. Review target, mount and privilege assumptions,
then run reviewed steps with `lfs-bun run`.

```sh
bun run lfs -- book sync --track stable-systemd
bun run lfs -- plan --book ./artifacts/lfs-book/stable-systemd
bun run lfs -- fetch --book ./artifacts/lfs-book/stable-systemd --execute
bun run lfs -- alfs setup --dir ./work/jhalfs --execute
# Configure the selected book and leave RUNMAKE, GETPKG, CLEAN disabled in make menuconfig.
bun run lfs -- alfs generate --dir ./work/jhalfs --execute
bun run lfs -- run --script path/to/reviewed-generated-script.sh
```

`run` prints a plan unless `--execute` is given and journals script digests,
timestamps and exit status for step-level continuation. LFS instructions can
format partitions and install a bootable operating system; use a disposable Linux VM
and a dedicated virtual disk. The tool does not select or format a disk.

## Project map

- [`docs/`](docs/README.md): research, source map, architecture, CLI operations,
  automation boundaries and decisions.
- [`src/cli.ts`](src/cli.ts): Bun CLI, source retrieval, manifest checks and
  explicit execution gate.
- [`crates/cargo-lfs-bun/`](crates/cargo-lfs-bun/Cargo.toml): Cargo-compatible
  launcher (`cargo lfs-bun`).
- [`sources.json`](sources.json): requested research seeds and companion Git remotes.
- [`packages/web-crawler/`](packages/web-crawler/): workspace crawler adapter
  used by the CLI, with raw evidence capture and bounded crawling.

## Scope and platform

The repository automates acquisition and orchestration; it does not replace the
LFS book, verify that a host meets every LFS requirement, or claim that a build
succeeded. Current LFS builds require Linux and the exact instructions for the
selected book revision. Windows can run the research CLI but cannot execute LFS
build scripts.
