# vps-boot STATUS

Date: 2026-11-04 (host clock 22:34 UTC at start; deadline 22:45 UTC)

## State
- Step 1 (bun build --compile of packages/bunos-init/src/init.ts, ldd closure, initramfs): NOT DONE.
- Step 2 (QEMU/KVM boot): NOT DONE. No kernel found at ~/src/yolo-work/artifacts/kernel/ (directory absent).
- Steps 3-5: NOT DONE.

## Evidence present
- /dev/kvm: crw-rw---- root:kvm; current user is in group kvm (KVM usable in principle, not yet tested).
- Tools on host: bun, qemu-system-x86_64, ldd, cpio, gcc.
- factory/linux/build-initramfs.sh exists (32 lines), not yet run.
- No boot log exists, so no boot claim is made.

## Blockers
- Session deadline reached in ~10 minutes; the build-and-boot chain was not started.
- Required kernel artifact (prod-kernel Linux 7.4 image) is absent from artifacts/kernel/.

## Next step
1. Build init: `bun build --compile packages/bunos-init/src/init.ts --outfile <artifacts>/init`.
2. ldd closure, then run factory/linux/build-initramfs.sh (or a fixed copy) to a file under ~/src/yolo-work/artifacts/.
3. Obtain a readable kernel (Ubuntu/Arch extracted without touching host), boot with -enable-kvm, capture serial log to docs/receipts/vps-boot/.
