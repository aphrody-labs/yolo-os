# bunos-rust status (2026-11-04 22:35 UTC)

State: partial. Only the matrix draft is written.

Done:
- profiles/rust-userland/matrix.json: 19 GNU/classic tools, candidate replacements. Versions, Arch/nixpkgs package names and maturity are NOT verified.

Not done:
- Step 2 (Docker conformance run, GNU vs Rust output, upstream test suites): not started. No pass counts exist.
- Step 3 (profiles/rust-userland/README.md): not written.
- Step 4 (pacman list, Nix overlay fragment): not written.

Blocker: session deadline 22:45 UTC reached in ~10 minutes at start; Docker access was not confirmed (`docker info` failed for this user).

Next step: verify versions and package names from upstream and the Arch/nixpkgs indexes, then run the conformance corpus in Docker with low parallelism (nice), recording pass counts per tool under docs/receipts/bunos-rust/.
