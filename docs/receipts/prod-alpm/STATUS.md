# prod-alpm STATUS (2026-11-04T22:39:15Z)

State: steps 1 and 2 done. Steps 3 and 4 blocked (not started).

## Evidence
- tests/alpm-oracle/vercmp.json: 2400 unique pairs, results from /usr/bin/vercmp (-1: 1264, 1: 1134, 0: 2). Pairs come from installed package versions (pacman -Q) plus ~36 edge cases. Generator: tests/alpm-oracle/gen_vercmp.py (seeded).
- tests/alpm-oracle/installed-sample.json: 60 random packages from pacman -Qi (host has 649 local packages).
- tests/alpm-oracle/desc/: 5 real /var/lib/pacman/local desc files.
- Fixtures total about 180 KB.
- Dependency data is in the pacman -Qi fields (Depends On, Optional Deps, Provides, Conflicts With). There is no separate dependency fixture yet.

## Sync databases
Fetched with curl into ~/src/yolo-work/prod-alpm-dbs/, not /var/lib/pacman.
| repo | bytes | packages | source |
|---|---|---|---|
| core | 131681 | 299 | fastly.mirror.pkgbuild.com |
| extra | 9005496 | 15208 | fastly.mirror.pkgbuild.com |
| omarchy | 54714 | 251 (zstd) | pkgs.omarchy.org/stable/x86_64 |
| cachyos | 523100 | 827 | mirror.cachyos.org/repo/x86_64/cachyos |
The packages column is the count of */desc entries in the archive. The core and extra databases are gzip, omarchy is zstd, and the cachyos compression was not checked. The cachyos count came from plain `tar -tf`.

## Blockers
- Push to claude/prod-alpm is rejected because ref refs/heads/claude exists (git ref namespace clash). Pushed to claude-prod-alpm instead, matching peers such as claude-bunos-arch.
- Steps 3 and 4: origin/claude/alpm does not exist yet (git fetch at 22:39:15Z). No mismatches to report.
- Bus: no .coordkit-bus/ in the checkout, so there is no inbox/alpm/ to write to.

## Next
Re-fetch. When origin/claude/alpm appears, build crates/yolo-alpm with nice -n 19 and -j2, run the vercmp fixtures and the read-only /var/lib/pacman database, and list any mismatches here.
