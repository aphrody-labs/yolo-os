# CLI reference

Run with `bun run lfs -- <command>` or `cargo run -p cargo-lfs-bun -- <command>`.
The Cargo alias after installation is `cargo lfs-bun <command>`.

## `sources`

Prints `sources.json`, the research seed and Git remote registry.

## `crawl [URL ...]`

Without positional URLs, crawls the registry seeds. Options:

- `--depth N`: same-origin link depth, 0–8 (default 1).
- `--max-pages N`: response cap (default 60 in this app).
- `--delay-ms N`: minimum request spacing (default 900 ms).
- `--output FILE`: JSON evidence output.
- `--markdown FILE`: extracted Markdown output.
- `--ignore-robots`: explicit robots exclusion override.
- `--include-body`: store raw HTML bodies alongside extracted page records.

The shared YOLO-Web CLI also provides `web crawl <url...>` with the same
bounded crawler and defaults of depth 1, 50 pages and 700 ms spacing.

## `repos [--max-pages N] [--output FILE]`

Collects all GitHub org/user sources from `sources.json` through the REST API,
then sorts the fetched records by stars. The default is five pages of 100
records per owner; the dated JSON preserves raw API page bodies and a Markdown
ranking report is saved. Set `GITHUB_TOKEN` to use an authenticated API quota.
Metadata marks listings as truncated if the page cap is reached.

## `git-heads [--output FILE]`

Runs `git ls-remote --symref <url> HEAD` for each `kind: "git"` entry in
`sources.json`. The output records the symbolic default branch, commit ID,
retrieval time, raw command response and SHA-256. It does not clone or fetch
repository objects.

## `synthesize <crawl.json>`

Regenerates deterministic linked Markdown from a captured crawl. It makes no
network requests and does not add semantic claims beyond the extracted page
text.

## `book sync [--track TRACK] [--dir DIR]`

Downloads the official `wget-list` and `md5sums` for `stable-systemd` (default),
`development`, `stable` or `stable-sysv` into `artifacts/lfs-book/<track>`. It
records each URL, retrieval timestamp, byte count and SHA-256 in
`book-source.json`. LFS currently serves SysV content from the stable download
directory, so `stable-sysv` maps to that same official location.

## `plan --book DIR`

Reads `wget-list` and optional `md5sums` from a local official-book download
directory (for example, the output of `book sync`) and prints a JSON plan. It
performs no package downloads.

## `fetch --book DIR [--sources DIR] [--execute]`

Without `--execute`, prints the same plan. With it, retrieves listed package
files and validates the official checksum list using `md5sum`. Requires Linux.

## `alfs setup|check|generate --dir DIR [--execute]`

`setup` plans or clones official jhalfs from the LFS Git server and records the
checkout commit. `check` reports required host commands (DocBook, XML library
versions, sudo policy, book requirements, and an empty dedicated mount still
need manual verification). `generate` requires an existing jhalfs checkout and
configuration containing exactly `RUNMAKE="n"`; it rejects `GETPKG="y"` and
`CLEAN="y"`. With `--execute` it runs the official generator on Linux and
leaves its generated Makefile available for review without downloading sources
or starting the LFS package build. Unset jhalfs options default to `n`; an
enabled or nonstandard assignment for these three gates is rejected.

## `run --script FILE [--step-id ID] [--journal FILE] [--execute]`

Without `--execute`, prints the command plan. With it, runs the supplied script
with `bash --noprofile --norc` on Linux. This is intended for reviewed
ALFS-generated scripts. Each execution appends a start event and a completion
event (script SHA-256, timestamps and exit code) to the journal, defaulting to
`artifacts/lfs-build-state.json`. Repeating an already successful step with the
same ID and script digest skips it. A changed script digest runs as a new
revision. The flag is an execution switch, not a sandbox.
