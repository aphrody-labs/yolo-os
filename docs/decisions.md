# Decisions and known limitations

## Decisions

- **Rust owns the application CLI.** Research, crawling, provenance, manifest
  handling and workflow state are implemented in one native Rust binary.
- **The book owns package recipes.** Build commands stay aligned with the exact
  selected LFS edition; this CLI orchestrates rather than translates them.
- **ALFS owns instruction extraction.** The established project-specific
  tooling creates scripts and Makefiles from the book's structured source.
- **Plans precede side effects.** Package downloads and script execution each
  require an explicit `--execute` option.
- **Evidence stays linked.** Crawl output is a hashed extraction with source
  URLs, not a claim that every page or dynamic state was reviewed.
- **The Awesome Rust catalog is a reference.** The upstream list is
  community-maintained, not official Rust Foundation documentation. The local
  LFS curation is not represented as an upstream-approved contribution.

## Gaps and limits

- GitHub rankings and default Git remote heads are observations, not immutable
  pins. Release artifact signature checks are not implemented.
- The crawler does not render dynamic pages or deeply parse structured tables.
- LFS execution journals successful script-level steps, but does not snapshot
  environments, disk state, VM lifecycle or progress within a script.
- There is no evidence of a successful end-to-end LFS system build yet. The
  dated ALFS generation record documents the earlier Bun tool generation and
  remains historical evidence; it does not validate the Rust implementation.
- Research rankings, source pages and Git heads change over time and should be
  recaptured before relying on them.
