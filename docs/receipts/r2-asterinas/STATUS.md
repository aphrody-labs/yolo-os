# r2-asterinas STATUS

Updated: 2026-11-04 (UTC, vps host). Deadline 2026-11-05 01:30 UTC.

## State
- Step 1 (source pin): DONE. asterinas v0.19.0 cloned to ~/src/yolo-work/r2-asterinas-src/asterinas at c4b7ed575b7b32773475b194d2ad41fb6d03765b. Recorded in distro/asterinas/sources.lock.
- Bun 1.4.4 linux-x64.zip downloaded to ~/src/yolo-work/artifacts/r2-asterinas/bun-1.4.4/. SHA-256 bf8ccbfb2b0b80159e0f9a6d15c55317add8077534dbc6130a3edb52fccb395c matches upstream SHASUMS256.txt.
- Docker image asterinas/dev:0.19.0-20261012: pull in progress (log: ~/src/yolo-work/artifacts/r2-asterinas/docker-pull.log).
- Step 2 (kernel boot): NOT STARTED.
- Step 3 (Bun in guest): suite written at distro/asterinas/suites/bun (extra_config.nix + src/main.rs). NOT RUN.
- Step 4 (README matrix): NOT STARTED.

## Next step
Finish docker pull, run `make kernel && make run_kernel` in the dev container, capture serial log to shell prompt.

## Blockers
None yet.
