# Source registry

The registry is [`../sources.json`](../sources.json), containing the owner's
requested URLs plus companion upstream Git remotes for revision evidence. It
identifies where to collect data and which source is authoritative for a claim.

| Area | Primary source | What it can establish | Collection notes |
| --- | --- | --- | --- |
| Asterinas | [GitHub org](https://github.com/orgs/asterinas/repositories?q=sort%3Astars), [kernel](https://github.com/asterinas/asterinas) | Project repositories, code, releases, Cargo manifests | Use GitHub API for stable structured listing; preserve repo commit and API observation time. |
| uutils | [GitHub org](https://github.com/orgs/uutils/repositories?q=sort%3Astars), [coreutils](https://github.com/uutils/coreutils) | Rust utility projects, Cargo/build and compatibility docs | Read per-repository README/docs; do not infer behavior from stars. |
| LFS | [Project](https://www.linuxfromscratch.org/), [download page](https://www.linuxfromscratch.org/lfs/download.html), [book Git](https://git.linuxfromscratch.org/lfs.git) | Book versions, package URL list, checksums, instructions, errata | Pin stable/development and systemd/SysV explicitly. Fetch matching `wget-list` and `md5sums`. |
| ALFS | [Project](https://www.linuxfromscratch.org/alfs/), [jhalfs](https://git.linuxfromscratch.org/jhalfs.git) | Existing book-to-script automation | Treat generated scripts as executable upstream-derived input and inspect before running. |
| LF Engineering | [Org listing](https://github.com/orgs/LF-Engineering/repositories?q=sort%3Astars) | Historical analytics/connectors source | Check archived status and latest commit; not a production dependency assumption. |
| Linux Foundation | [Foundation](https://www.linuxfoundation.org/) | Foundation and hosted-project context | Not an authority for kernel release state or LFS instructions. |
| Linux releases | [kernel.org](https://www.kernel.org/), [mainline Git](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git) | Release channel/version and canonical mainline source | Record channel, tag/commit, signature and observation time. |
| Torvalds repositories | [GitHub listing](https://github.com/torvalds?tab=repositories&q=&type=&language=&sort=stargazers), [Linux mirror](https://github.com/torvalds/linux) | Account projects and mirror metadata | Stars are mutable; kernel Git on kernel.org remains the source-of-truth tree. |
| GNU | [Documentation](https://www.gnu.org/doc/doc.html), [software index](https://www.gnu.org/software/software.html) | GNU manual and software catalogs | Resolve individual project sources and licenses; do not bulk-copy manuals. |

GitHub org/user listings are collected with the official REST endpoints at 100
records per page, then sorted locally by star count. Collection reads five
pages per owner by default (configurable, hard cap 10). The saved snapshot reports page count and whether
the configured page cap truncated the listing; each API page records retrieval
time, ETag, item count and SHA-256. A truncated ranking may omit
repositories from later pages. A GitHub token can be supplied as
`GITHUB_TOKEN` to use the higher authenticated API quota.

## Evidence schema

Each crawler record stores requested and final URL, HTTP status, retrieval
timestamp, content type, extracted title/description/text, discovered links,
SHA-256 and any fetch error. Use `--include-body` when preserving a source
capture; this stores the raw response body and increases snapshot size. GitHub
listing records include raw API page bodies, pagination state and repository
metadata. Git remote records retain raw `ls-remote` output as well as commit and
branch metadata. License is not reliably inferable from HTML; record it only
when an authoritative license statement is available.
