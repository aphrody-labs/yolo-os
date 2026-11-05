# CLI reference

Run with `cargo run -p cargo-lfs -- <command>` or install the binary and use
`cargo lfs <command>`.

## `sources`

Prints `sources.json`, the research seed and Git remote registry.

## `crawl [URL ...]`

Without positional URLs, crawls registry seeds. Options: `--depth N` (default
1), `--max-pages N` (default 60), `--delay-ms N` (default 900 ms), `--output
FILE`, `--markdown FILE`, `--ignore-robots`, and `--include-body`. The crawler
uses bounded sequential HTTP requests, respects robots rules by default and
only follows links at registered origins.

## `repos [--max-pages N] [--output FILE]`

Collects GitHub organization and user sources from `sources.json` through the
REST API, then sorts fetched records by stars. The default is five pages of 100
records per owner; dated JSON preserves API page bodies and a Markdown report
is saved. Set `GITHUB_TOKEN` for an authenticated API quota.

## `git-heads [--output FILE]`

Runs `git ls-remote --symref <url> HEAD` for each Git source. Output records the
symbolic default branch, commit ID, retrieval time, raw response and SHA-256.
It does not clone or fetch repository objects.

## `synthesize <crawl.json> [--output FILE]`

Regenerates deterministic linked Markdown from a captured crawl without network
requests or additional semantic claims.

## `book sync [--track TRACK] [--dir DIR]`

Downloads the official `wget-list` and `md5sums` for `stable-systemd` (default),
`development`, `stable` or `stable-sysv` into `artifacts/lfs-book/<track>`. It
records URLs, retrieval time, byte count and SHA-256 in `book-source.json`.
LFS serves SysV content from the stable download directory.

## `plan --book DIR [--sources DIR]`

Reads `wget-list` and optional `md5sums` from a local official-book download
directory and prints a JSON plan. It performs no package downloads.

## `fetch --book DIR [--sources DIR] [--execute]`

Without `--execute`, prints the plan. With it, retrieves listed package files
and validates the official checksum list using `md5sum`. Requires Linux.

## `alfs setup|check|generate --dir DIR [--execute]`

`setup` plans or clones official jhalfs and records the checkout commit.
`check` reports host command prerequisites; DocBook/XML versions, sudo policy,
book requirements and target mount still need manual verification. `generate`
requires a jhalfs checkout and a configuration with `RUNMAKE="n"`; it rejects
`GETPKG="y"` and `CLEAN="y"`. With `--execute`, it runs the official generator
on Linux and leaves generated scripts and Makefile for review without
downloading sources or starting the package build. Unset gates default to `n`;
enabled or nonstandard values are rejected.

## `run --script FILE [--step-id ID] [--journal FILE] [--execute]`

Without `--execute`, prints the command plan. With it, runs the supplied script
with `bash --noprofile --norc` on Linux. Each execution appends a start event
and completion event (script SHA-256, timestamps, exit code) to the journal,
defaulting to `artifacts/lfs-build-state.json`. Repeating a successful step
with the same ID and digest skips it. A changed digest runs as a new revision.
The flag gates execution; it is not a sandbox.
