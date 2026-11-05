# Architecture

```text
sources.json ──> cargo-lfs Rust crawler ──> docs/research/*.json
                         │                         │
                         └── bounded HTTP            └──> Markdown extraction

official LFS book ──> wget-list + md5sums ──> package source cache
       │
       └──> official ALFS/jhalfs scripts ──> cargo lfs run --execute

GitHub REST API ──> ranked repository snapshot
Git HTTPS remotes ──> `git ls-remote` ref/commit snapshot
```

## Components

- `bun-os-native` provides host CPU/memory topology, optional CUDA driver
  discovery, and monotonic clock probes. The API describes the current host;
  it does not infer a cross-compilation target or qualify a GPU runtime.
- `cargo-lfs` is a Rust CLI for source crawling, repository metadata, Git
  snapshots, LFS manifests, checksums, ALFS orchestration and step journaling.
- The crawler is sequential and bounded. It enforces HTTP(S), rejects common
  local/private IP ranges, limits response size and page count, respects robots
  rules by default, and only follows links at allowed origins.
- GitHub API and Git remote metadata use dedicated adapters rather than HTML
  scraping.
- LFS book interpretation and build-step generation stay with official ALFS.
  The CLI preserves generated commands so they can be reviewed and traced to
  the selected book.
- Dated research captures retain timestamps and response hashes; transient
  outputs default to ignored `artifacts/` paths.

## Crawl boundaries

Seed origins form the default allowlist and a page cap bounds each run. Robots
rules are cached per origin for one process. Oversized HTTP response bodies are
discarded with an error record. The crawler does not render JavaScript, use
browser cookies, or resolve DNS to block every possible private address.
GitHub's dynamic pages may be incomplete, so structured repository metadata is
collected through the REST adapter. API listings can be truncated at the page
cap and are reported as observations rather than complete inventories.

## Data flow and integrity

Every successful capture has a content digest and fetch timestamp. Synthesis
links back to the final URL and includes status and digest. A digest identifies
captured bytes; it is not a publisher signature. LFS package checksums come
from the selected official book and are checked independently of crawler
hashes.
