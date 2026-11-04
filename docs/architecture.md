# Architecture

```text
sources.json ──> @yolo-web/web-crawler ──> docs/research/*.json
                      │                          │
                      └── `web crawl`             └──> Markdown extraction

official LFS book ──> wget-list + md5sums ──> download cache
       │
       └──> ALFS/jhalfs generated scripts ──> lfs-bun run --execute

GitHub org/user API ──> ranked repository snapshot
Git HTTPS remotes ──> `git ls-remote` ref/commit snapshot

`cargo lfs-bun` ──> Rust launcher ──> Bun `src/cli.ts`
```

## Components

- `@yolo-web/web-crawler` is the shared Bun implementation. It enforces HTTP(S),
  rejects common local/private IP ranges, follows at most five redirects,
  respects robots rules by default, limits response size and page count, and
  only follows links at allowed origins.
- YOLO-Web's `web crawl` is the general-purpose front door. It emits JSON
  evidence and a deterministic Markdown extraction.
- `yolo-os` owns LFS source selection and orchestration. Dated research captures
  are checked in with timestamps and response hashes; transient runs default
  to the ignored `artifacts/` directory.
- The Rust Cargo launcher only starts Bun and forwards arguments. It is a
  convenient Cargo command surface, not a duplicate implementation.
- GitHub and Git metadata use separate adapters: paginated REST for repository
  metadata and `git ls-remote` for default branch/commit identity.
- LFS book interpretation and step generation are delegated to official ALFS.

## Crawl boundaries

The crawler is intentionally conservative and sequential. Seed origins form the
default allowlist, and a page cap bounds each run. Robots rules are cached per
origin and user agent for the process. An HTTP response body over the configured byte limit is
discarded with an error record. The HTML crawler does not render JavaScript,
use browser cookies, or resolve DNS to block every possible private address.
GitHub API listings are collected by a separate paginated adapter. On GitHub,
HTML link extraction keeps
repository roots and organization repository-list pages to avoid filling the
page budget with global navigation, and seed queues are serviced round-robin
so one popular organization's long listing cannot starve the other seeds.
Prefer API/Git adapters for those sources as a follow-up; a public HTML page may
be incomplete or rate-limited.

## Data flow and integrity

Every successful capture has a content digest and fetch timestamp. Synthesis
links back to the final URL and includes the status and digest. The digest is
for captured response bytes; it is not a signature or proof that the publisher
authored the page. LFS package checksums come only from the selected official
book download directory and are checked independently from crawler hashes.
