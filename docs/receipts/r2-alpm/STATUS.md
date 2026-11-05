# r2-alpm status

Updated: 2026-11-04 (UTC, ~00:00). Branch: claude-r2-alpm. Scope: distro/alpm/**, docs/receipts/r2-alpm/**.

## Done
1. yolo-alpm library (`distro/alpm/crates/yolo-alpm`, `#![forbid(unsafe_code)]`).
   - Sync-db container reader: gzip (flate2) and zstd (ruzstd, pure Rust), plain tar. Compression is detected by magic bytes.
   - xz and bzip2 are recognised and refused (`Error::Unsupported`). No pure-Rust xz path was taken.
   - desc v1/v2 parser into a typed `Package`. Unknown keys are counted, not fatal.
   - `vercmp`: port of pacman `version.c` (rpmvercmp, parseEVR). Loop condition and final branch were checked against the pinned source (CachyOS/pacman @663941f).
   - Dependency closure: provides-aware, versioned constraints, earlier repo wins, cycle-safe, optional deps skipped.
   - Tests: 17 pass (`cargo test --workspace`). Includes all 46 pairs of pacman's `vercmptest.sh` checked in both directions.
2. yolo-pm CLI (`distro/alpm/crates/yolo-pm`): `sync`, `search`, `info`, `deps`, `stats`, `--json`, `--cache`.
   - Live sync of core, extra and cachyos succeeded. Receipts: `sync.json`, `stats.txt`, `info-*.txt`, `deps-*.txt`.

## Not done / limits (record, do not present as working)
- Signature verification: not done. `%PGPSIG%` presence is recorded. The `.db.sig` HTTP status is recorded (core and extra: 404, cachyos: 200). No key import, no cryptographic check.
- `.files` databases: parsed by the library, but the CLI only fetches `<repo>.db`.
- No `vercmp` binary on this host: no differential run against vercmp(8). The official vector table is the oracle.
- The `alpm` crate comparison (libalpm bindings) was not run.
- Stable rustup toolchain manifest is missing on this host. Builds use `RUSTUP_TOOLCHAIN=1.99.1-x86_64-unknown-linux-gnu`. Default toolchain was not changed.
- Bun front end (step 3) done: `distro/alpm/bun` (`src/cli.ts`, `src/backends.ts`), no npm deps.
   - pacman backend shells out to `yolo-pm --json search`. cargo backend runs `cargo search`.
   - flatpak, uv and winget are reported unavailable: their binaries are not on PATH on this host. No adapter exists for them either.
   - Tests: 9 pass (`bun test`). Receipts: bun-test.txt, bun-search-sudo-rs.txt.
   - Runtime: the bun on PATH is v1.4.6-canary.1. `bun agent-plugin doctor` reports aphrody:true. The session-start hook warned otherwise. Not reconciled. No runtime was installed.

## Next
- Step 4 receipts: cargo test and vercmp, timings, `info` and `deps` (written in docs/receipts/r2-alpm).
- Optional: signature verification of `cachyos.db.sig` (time-boxed).
