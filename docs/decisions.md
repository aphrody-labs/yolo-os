# Decisions and known limitations

## Decisions

- **Bun owns the TypeScript CLI.** This matches YOLO-Web's repository policy and
  keeps the crawler shared between `web crawl` and the LFS tool.
- **Cargo is a thin launcher.** It offers a Rust/Cargo installation surface
  without duplicating networking, parsing or synthesis logic.
- **ALFS owns build-step extraction.** LFS book commands are structured and
  ALFS is the project-specific established automation path.
- **Plans precede side effects.** Downloads and shell execution each require an
  explicit `--execute` option.
- **Evidence stays linked.** Crawl output is an extraction with hashes and
  citations, not a claim that every page or dynamic state was reviewed.

## Gaps to close

- GitHub rankings and default Git remote heads are captured; the HTML crawler
  itself still cannot render dynamic pages or replace API/Git adapters for
  structured content.
- Git remote heads are observed but not pinned for later cloning, and release
  artifact signature checks are not implemented.
- The crawler does not render dynamic pages or parse structured tables deeply.
- LFS execution journals successful script-level steps, but does not snapshot
  environments, disk state, VM lifecycle or resume midway through a script.
- There is no evidence of a successful end-to-end LFS image build yet.
- The app imports the local sibling YOLO-Web checkout with a `file:` dependency;
  publication requires a versioned registry/Git dependency policy.
