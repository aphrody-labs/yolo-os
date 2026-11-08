# BunOS PID 1 prototype

This package preserves an Apache-2.0 BunOS PID 1 prototype for Linux initramfs experiments. It is userland OS boot code owned by `yolo-os`; it is not part of Bun's runtime and is not a complete LFS init system.

## Provenance

- Source: [aphrody-labs/bun-os](https://github.com/aphrody-labs/bun-os/tree/ae136439ba7d02039d23e2379e8690f9a260b52e/packages/bunos-init)
- Source commit: `ae136439ba7d02039d23e2379e8690f9a260b52e`
- Package at source: `@aphrody/bunos-init@1.4.7`, Apache-2.0
- Copied file retains its Apache-2.0 SPDX header.

The source supports pseudo-filesystem mounts, child reaping, signal handling, shutdown/reboot, an Android binderfs check, and a small HTTP health mode. It does not switch to the LFS root filesystem or start the complete system services. The archived commit message reports a separate QEMU run; its receipt was not found in the local archive, so this copy is not marked qualified. The original file, license and package metadata were snapshotted before migration; the implementation retains its Apache-2.0 SPDX header.
