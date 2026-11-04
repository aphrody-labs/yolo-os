# ALFS generation check

On 2026-11-04, the Bun CLI cloned official jhalfs commit
`a13fd9828b8c28d02dfde824a441cd7dd3a1643d` in an isolated Debian 13 container
and generated LFS systemd 13.1 scripts from book commit
`2e45031d5a2c1fe9e77239667f70631580eebfc3`.

The retained Makefile SHA-256 is
`7ea406cb7e220cae34f51f457cb562a62776bc37ec56986957c038e15e7b06d1`. The
tree contained 538 files including the book checkout, with 131 package-step
scripts plus 3 kernfs/chroot helpers. `RUNMAKE`, `GETPKG` and
`CLEAN` were disabled. No package files were downloaded, and no LFS package
build was started. jhalfs reported its host tool and selected-book prerequisite
checks as passing.

The output was generated in a dedicated Docker named volume. Only the dated
metadata is retained here; scripts and Makefile contents remain upstream-derived
executable instructions and require review before use.
