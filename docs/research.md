# Research synthesis

Research snapshot: 2026-11-04. Repository star counts below are the observed
GitHub REST API values at collection time; rankings change continuously. The
complete response-derived list, including all collected metadata and repository
links, is saved in [the dated ranking snapshot](research/github-repository-rankings-2026-11-04.md)
and [JSON evidence](research/github-repository-rankings-2026-11-04.json).

## Linux From Scratch and ALFS

LFS is the instruction source for constructing the base system. The project
separates stable books from development books and explicitly warns that package
versions in a book are tested together. The current stable systemd book is
13.1; the download index includes its `wget-list` and `md5sums`. The development
book can temporarily break. Before a build, check the selected book's host
requirements, errata and security advisories. The book is chapter-structured
DocBook XML; its source repository and editor guide are the right inputs for
structured research rather than treating arbitrary rendered prose as shell.
The current rendered book says the book is under a Creative Commons license
and computer instructions may be extracted under MIT; this does not apply to
third-party package manuals or other external documentation.

The official [ALFS project](https://www.linuxfromscratch.org/alfs/) and its
`jhalfs` implementation transforms book instructions into scripts and a
Makefile, can download sources and supports resumption. Its configuration has a
`RUNMAKE` switch; `lfs-bun alfs generate` refuses to run unless the saved
configuration explicitly sets `RUNMAKE="n"`. That prepares generated scripts
and a Makefile without starting the package build. The source checkout commit
is recorded. Host prerequisites include Git, Bash, make, Python, `xmllint`,
`xsltproc`, DocBook XML DTD 4.5 and DocBook XSL; source retrieval and privileged
steps add wget/curl and sudo requirements. The CLI checklist does not replace
jhalfs version checks or the selected book's host requirements. The target
mount must be a dedicated disposable disk because upstream cleanup operations
can remove paths beneath its configured build directory. YOLO OS should
orchestrate that tested extraction flow, retain generated scripts and logs, and
add a review boundary for unusual instructions. It should not invent its own parser for all
shell blocks or assume every Beyond Linux From Scratch (BLFS) chapter is
uniformly automatable.

The live LFS site also lists BLFS, MLFS, GLFS and SLFS alongside the base book,
plus Hints, Patches and the Editor's Guide. LFS homepage describes ALFS as its
automation and build-management subproject. On 2026-11-04, the stable systemd
download directory supplied 99 `wget-list` URL lines and 94 `md5sums` entries;
the retrieved manifests and SHA-256 digests are preserved in the [LFS capture](research/lfs-stable-systemd-2026-11-04.json).

## Kernel and Linux Foundation

`kernel.org` publishes multiple release channels (mainline, stable and
long-term), plus source, patches, signatures, changelogs and documentation.
Store a channel and version together; a single “latest Linux” field loses
important release semantics. Linux kernel development uses Git plus mailing
lists and subsystem workflows. `torvalds/linux` on GitHub is a visible mirror;
the authoritative tree is the Torvalds Git repository at kernel.org.

The captured mainline remote points to `refs/heads/master` at commit
`05441d697f7f9a927264f4058cbbc6a03dfcac9d`; the separately captured LFS book
remote points to `refs/heads/trunk` at `5fc16721c40ecf80cb3b998ed0c179771dbb3b36`.
The full remote snapshot also records Asterinas and uutils refs and output
digests. These are moving branch heads observed at fetch time, not release pins.

The live kernel.org index on 2026-11-04 listed mainline 7.4-rc1 (dated
November 1) and stable 7.3.1 (dated October 30), with separate long-term series
and `linux-next`. These are observed index values, not a recommendation to
build a particular kernel release.
Kernel documentation is maintained alongside the code in Sphinx/reStructuredText;
the official build guide requires checking host tool versions and supports
out-of-tree builds when the same `O=` directory is passed to each make call.

The Torvalds GitHub account returned 12 public repositories in the API
snapshot. `linux` led with 254,384 stars, followed by `AudioNoise` (4,529),
`GuitarPedal` (2,487), `uemacs` (2,192), `test-tlb` (1,081) and `pesconvert`
(593). A high star count alone does not make an unrelated
personal repository a kernel dependency. The Linux Foundation is an ecosystem
host and project umbrella, not the authority for LFS package versions or the
kernel's canonical source history.

## Asterinas and uutils

[Asterinas](https://github.com/asterinas/asterinas) had 61 public repositories
in the API snapshot. `asterinas/asterinas` led with 5,066 stars, followed by
`hyperenclave` (88) and `vostd` (61). It is a Rust-based, Linux-compatible
kernel with a Cargo workspace, `osdk` CLI and mdBook project
book. Its official guide describes architecture-specific support tiers and a
Docker-oriented developer workflow (`make kernel`, `make run_kernel`). The
official guide lists x86-64 as Tier 1, RISC-V and Intel TDX as Tier 2, and ARM64
and LoongArch as Tier 3. The project announced release 0.19.0 on 2026-10-13.
These are useful Rust OS and build-system references; they do
not replace the Linux From Scratch book or kernel sources.

[uutils/coreutils](https://github.com/uutils/coreutils) is a Rust
reimplementation of GNU coreutils and led 31 public uutils repositories with
24,328 stars at capture time. `findutils` (656), `diffutils` (283),
`util-linux` (227), `grep` (138), `platform-info` (131), `procps` (117) and
`sed` (112) followed in the captured ranking. Its
Cargo installation flow, GNU compatibility tests and generated command
documentation offer patterns for a maintainable Rust command-line surface.
Its docs describe a locked Cargo install path (`cargo install coreutils
--features unix --locked`) and exercise GNU compatibility behavior with the
upstream test suite. This CLI follows the same explicit compatibility and
provenance approach without adding uutils as a build dependency.
Related `uutils/findutils`, `diffutils` and `util-linux` projects are relevant
for POSIX/GNU behavior research. Package compatibility must be evaluated
against the selected LFS book and host requirements before substitution.

## LF-Engineering repositories

The requested LF-Engineering organization had 18 public repositories in the
API snapshot. Every returned repository was archived. The highest-star entries
were `dev-analytics-affiliation` (32), `darst` (14),
`dev-analytics-libraries` (10), `kelpy` (9), and the Insights Git and shared
data connectors (7 each). This is a historical analytics/connectors collection
rather than a current LFS implementation base. Their useful design
idea is source adapters feeding a shared normalized evidence model; the
archived code should not be adopted as a maintained runtime dependency.

## GNU documentation and software

GNU maintains online manuals and a package catalog. Documentation and software
are separate indexes; package-level source repositories and licenses can
differ. The GNU project promotes free documentation and uses the GNU Free
Documentation License (GFDL) for many manuals, but that does not grant permission to
repackage every external source's full text. Store short factual extractions,
links and license metadata; do not copy whole manuals into this repository.

## Design conclusions

1. Keep source identity and version/revision explicit (book release, Git commit,
   kernel channel, repository owner/name).
2. Capture retrieval timestamp, canonical URL, HTTP status, content hash,
   license/rights metadata and extractor version.
3. Use a source adapter per HTML, GitHub API and Git repository source. Keep the
   normalized page record independent of any specific upstream host.
4. Generate synthesis from stored evidence, with source links next to claims.
   Mark stale, inaccessible or partial sources rather than filling gaps.
5. Use the official LFS checksum manifest and ALFS-generated build scripts.
   Treat downloads and executing build instructions as separate stages.

## Primary references

- [LFS downloads and version policy](https://www.linuxfromscratch.org/lfs/download.html)
- [LFS stable 13.1 download files](https://www.linuxfromscratch.org/lfs/downloads/stable-systemd/)
- [LFS reading notes, errata and advisories](https://www.linuxfromscratch.org/lfs/read.html)
- [LFS Editor's Guide](https://www.linuxfromscratch.org/lfs/LFS-EDITORS-GUIDE.html)
- [ALFS](https://www.linuxfromscratch.org/alfs/) and [jhalfs Git](https://git.linuxfromscratch.org/jhalfs.git)
- [Linux releases](https://www.kernel.org/) and [kernel documentation](https://docs.kernel.org/)
- [Kernel admin build README](https://docs.kernel.org/admin-guide/README.html) and [license rules](https://docs.kernel.org/process/license-rules.html)
- [kernel.org Git tree](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git)
- [Asterinas](https://github.com/asterinas/asterinas) and [Asterinas book](https://asterinas.github.io/book/)
- [Asterinas repository guide](https://github.com/asterinas/asterinas/blob/main/AGENTS.md) and [Cargo workspace](https://github.com/asterinas/asterinas/blob/main/Cargo.toml)
- [uutils coreutils](https://github.com/uutils/coreutils)
- [uutils installation guide](https://github.com/uutils/coreutils/blob/main/docs/src/installation.md)
- [GitHub REST API: list organization repositories](https://docs.github.com/en/rest/repos/repos#list-organization-repositories)
- [GitHub REST API: list user repositories](https://docs.github.com/en/rest/repos/repos#list-repositories-for-a-user)
- [LF-Engineering organization](https://github.com/orgs/LF-Engineering/repositories?q=sort%3Astars)
- [Linux Foundation](https://www.linuxfoundation.org/)
- [GNU documentation](https://www.gnu.org/doc/doc.html), [GNU software](https://www.gnu.org/software/software.html), and [GNU licenses](https://www.gnu.org/licenses/licenses.en.html)
