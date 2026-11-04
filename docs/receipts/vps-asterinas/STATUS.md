# vps-asterinas status

Updated: 2026-11-04 22:35 UTC (session deadline 22:45 UTC)

## State

Preflight only. No Asterinas build, boot, or Bun run has been executed. No result below is a boot or runtime claim.

## Verified facts (2026-11-04)

- Host: Linux 7.3.0-9-generic, 12 vCPU, 45 GiB RAM, 333 GiB free on /home.
- `/dev/kvm` exists (group `kvm`, mode `crw-rw----`). The `ubuntu` user is in `kvm` and `docker`, so KVM and Docker are usable from this shell.
- Docker daemon responds (`docker info` ok). QEMU at `/usr/bin/qemu-system-x86_64`.
- Upstream reachable: `git ls-remote https://github.com/asterinas/asterinas HEAD` returned `925f829a3e64bd91bd9b48435b81173dfbc2860c`. This is the upstream HEAD at probe time, not yet a pinned build commit.
- `origin/claude/vps-asterinas` did not exist before this push.
- `~/src/yolo-work/bunos-factory/` is absent. Work proceeds from upstream sources directly.
- `~/src/yolo-work/third_party/` and `~/src/yolo-work/artifacts/` do not exist yet. They were not created in this session.

## Blockers

- Time: about 10 minutes remained at the start of this session, not enough for an Asterinas Docker image pull, kernel build, and QEMU boot.
- Not yet done: clone at a pinned commit, read the book and distro directory, run the documented Docker dev image, boot, static/dynamic hello, `bun --version`, Bun script under musl and glibc, NixOS path.

## Evidence paths

- This file only. No receipts, logs, or images exist yet.

## Next step

1. Clone `asterinas/asterinas` into `~/src/yolo-work/third_party/asterinas` and pin the commit (record hash).
2. Read its `book/` and the docker dev-image instructions. Pull the image, run `make build` under Docker, then `make run` with KVM. Save the trimmed serial log to `docs/receipts/vps-asterinas/`.
3. Only then start the hello/Bun ladder. Record syscall failures as they occur.
4. Create `factory/asterinas/README.md` with commands and a works/fails/syscall table once any results exist.

Write scope held: `docs/receipts/vps-asterinas/` only. `factory/asterinas/` not yet created.
