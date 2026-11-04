# factory/kernel (vps-kernel slice)

Status: NOT BUILT. Build script and config are not yet written. No bzImage exists.

Observed upstream (queried 2026-11-04 ~22:35 UTC via `git ls-remote` on torvalds/linux):
- Newest 7.4 tag seen: `v7.4-rc1` (bc71431221f5ba8bb33918399552636e059c2c45). No final `v7.4` tag was visible in the query.
- `v7.3-rc1..rc7` tags are present; a final `v7.3` tag was not confirmed by this query.
- CachyOS/kernel-patches: not pinned. The query returned branches only; no tag or commit was selected.

Not done: CachyOS patch pin, reproducible build script, Docker build, config, bzImage, SHA-256, scp delivery, 7.4 patch-apply check.
Next step: pin the final tag, pin the CachyOS patch commit, write `build.sh` with `--execute`, and build in a container with `--cpus=8 --memory=24g`.
