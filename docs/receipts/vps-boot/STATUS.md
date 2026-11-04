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

## Update 22:34 UTC
- Step 1 partial: `bun build --compile packages/bunos-init/src/init.ts` succeeded. Output (not committed, 140 MB):
  ~/src/yolo-work/artifacts/vps-boot/init. The binary is dynamically linked to glibc
  (libc.so.6, libpthread, libdl, libm, ld-linux-x86-64). Initramfs needs those libs copied in or a static build.
- PUSH BLOCKED: `git push origin HEAD:claude/vps-boot` rejected by remote:
  "cannot create 'refs/heads/claude/vps-boot': 'refs/heads/claude' exists". The coordinator must rename or
  remove the remote `claude` ref; this session will not delete it or force-push. Commit e9cf629 is local only.
