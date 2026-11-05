# Slice bunos-arch: status

Updated: 2026-11-04 22:35 UTC (deadline 22:45 UTC)

## State

Not started on the Arch rootfs. No profile, package list, pacman.conf,
rootfs, WSL tarball or disk image exists yet. Nothing is built or verified.

## Checks run (evidence)

- `date -u`: 22:34 UTC at start. The session had about 10 minutes left.
- Docker: `docker info` fails with "Cannot connect to the Docker daemon at
  unix:///var/run/docker.sock". Step 1 (pacstrap in an archlinux container)
  is blocked until the daemon is available.
- `/dev/kvm`: present (`crw-rw---- root:kvm`). The `ubuntu` user may not be
  in the `kvm` group; not checked.
- Disk: 75 GB free on `/`. Memory: 11 GB total, 10 GB available.
- `profiles/` and `docs/receipts/` did not exist before this commit.
- `git ls-remote origin claude/bunos-arch` returned no output in the time
  available; remote branch state not confirmed.

## Steps

1. Package list from Docker archlinux container: BLOCKED (no Docker daemon).
2. pacman.conf with official, CachyOS, Omarchy repos and key handling: NOT STARTED.
3. Smoke tests (bun, nix, uutils, sudo-rs, Hyprland, zeditor, cosmic-*): NOT STARTED.
4. yolo-arch-rootfs.tar.zst and WSL tarball: NOT STARTED.
5. Bootable image under QEMU in container with /dev/kvm: NOT STARTED.
6. Bus message to vps-boot with fetch location: NOT SENT (no rootfs exists).

## Next step

1. Start the Docker daemon, or confirm a user-level rootless alternative.
   Do not change system services without the slice allowing it.
2. Verify `archlinux` image pull and sync-db download, then fill
   `profiles/arch/packages.txt` by checking each name against the sync DBs.

## Blockers

- Docker daemon not running on bunos; no evidence that `ubuntu` can start it.
- Time budget: the 22:45 UTC deadline left about 10 minutes at session start.
