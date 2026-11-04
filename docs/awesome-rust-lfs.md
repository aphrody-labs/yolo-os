# Awesome Rust references for LFS tooling

The upstream [`rust-unofficial/awesome-rust`](https://github.com/rust-unofficial/awesome-rust)
repository is a community-maintained, unofficial curated list. It is not an
official Rust project or Rust Foundation catalog. The LFS adaptation is on the
public fork [`aphrody-dev/awesome-rust`](https://github.com/aphrody-dev/awesome-rust),
branch `yolo-lfs-2026`. It adds `YOLO-LFS.md` and a README link while leaving
the upstream catalog entries intact. The working clone is at
`work/awesome-rust-lfs` and records the upstream base commit in Git history.

## Selected Rust building blocks

| Crate | Use here | LFS-oriented reason |
|---|---|---|
| [clap](https://crates.io/crates/clap) | Commands and argument parsing | Keep plan, fetch, generation and execution gates explicit. |
| [reqwest](https://crates.io/crates/reqwest) | Blocking HTTP client | Simple sequential retrieval suits bounded research and manifest sync. |
| [scraper](https://crates.io/crates/scraper) | HTML document parsing | Extract links and text without pretending to execute or render pages. |
| [serde](https://crates.io/crates/serde) and [serde_json](https://crates.io/crates/serde_json) | Evidence, manifests and journals | Persist inspectable records with stable machine-readable structure. |
| [sha2](https://crates.io/crates/sha2) | Response and script digests | Identify the exact captured bytes and reviewed script revision. |
| [url](https://crates.io/crates/url) | URL validation and resolution | Constrain crawler scope and preserve canonical source URLs. |
| [chrono](https://crates.io/crates/chrono) | UTC event times | Make capture and execution chronology explicit. |
| [anyhow](https://crates.io/crates/anyhow) | CLI error context | Report which source or step failed while preserving the underlying cause. |

## Bun's upstream Rust workspace

[Bun 1.4](https://bun.com/blog/bun-v1.4) rewrote Bun from Zig to Rust. Its
[upstream Cargo workspace](https://github.com/oven-sh/bun/blob/main/Cargo.toml)
contains Rust crates such as `bun_http`, `bun_install`, `bun_resolver`, and
`bun_transpiler`, alongside many more internal crates. These are Bun's own
workspace crates and path dependencies, not generally published standalone
crates. Treat Bun as a Rust project in the 2026 Rust ecosystem; reuse its
internal crates only by depending on a deliberately pinned upstream checkout
and respecting their workspace/build requirements. `cargo-lfs` does not need
those internals, so it depends on focused crates from crates.io instead.

## Selection rules

- Prefer the standard library when a dependency does not materially simplify a
  visible task.
- Keep networking synchronous and sequential while the crawler has a bounded
  page count and delay policy.
- Keep the tool headless and CLI-first; LFS work is driven through shell tools
  and documented build steps.
- Treat a Rust rewrite of an operating-system package as an optional later
  substitution experiment, never as an invisible change to the selected book.
- Review licenses and dependency updates when changing `Cargo.toml`; the lock
  file pins the resolved application graph for reproducible tool builds.

The fork's companion page links to this reference. Its branch is pushed to the
user's fork; no pull request has been opened against upstream.
