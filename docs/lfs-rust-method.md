# Rust tooling, LFS method

## The book remains the build specification

LFS is a guided construction of a small Linux system from documented source
code and commands. Choose one book revision and keep its package versions,
checksums, patches, host requirements, errata and advisories together. Each
package keeps its own upstream build system and commands. This project does not
translate book recipes into Rust or silently replace them with Rust crates.

## Rust makes the surrounding work inspectable

The Rust CLI handles acquisition, source records, crawler policy, manifests,
checksums, planning, ALFS invocation and resumable step journals. Keep those
operations explicit:

1. Record which book track and upstream revision is selected.
2. Show the plan and inputs before network writes or shell execution.
3. Preserve upstream artifacts and command output for review.
4. Check downloads against the checksum manifest from that same book.
5. Run generated instructions one reviewed step at a time in a disposable
   Linux environment.

An execution journal records what was invoked and its result; it cannot prove
the state of a host or reverse external side effects. A failed step must be
inspected before retry.

## Keep the result minimal and understandable

LFS deliberately teaches how a base system fits together; it is a starting
point, not a turnkey general-purpose distribution. Avoid adding runtime layers
that conceal package order, host dependencies, privilege boundaries or target
disk effects. Rust improves the safety and clarity of the research tools around
that process; it does not make arbitrary upstream shell instructions safe.

## Where Rust substitutions belong

uutils and other Rust system tools can be studied as optional replacements
after the base system exists. A replacement is a separate experiment: record
the package, book revision, command compatibility evidence, dependencies,
license and recovery path. Do not change the canonical LFS recipe or checksum
manifest to accommodate it.
