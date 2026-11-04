# Linux userland boot profile

Target source: `torvalds/linux` tag `v7.4-rc1` as observed 2026-11-04. The attempted version string `v7.4.1-rc` was not a Git ref in the checked mainline repository.

Use `bun run pid1:build-initramfs -- <output.cpio.gz>` to compile the first-party PID 1 entry to `/init`, include the minimal pseudo-filesystem layout, and create a test initramfs. The script uses AOT plus trimming when the selected Bun exposes those flags; stable Bun 1.4.4 falls back to its standalone compiler, while the pinned upstream canary lane enables AOT. This produces a minimal test initramfs only: runtime library closure analysis, a pinned kernel boot, root-switch/service handoff and boot receipts remain required before a production image can use it. The production LFS image remains owned by `yolo-os`.
