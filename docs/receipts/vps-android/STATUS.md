# vps-android status (2026-11-04, 22:35 UTC)

## State
- Not started on the boot path. No Cuttlefish image downloaded, no container built, no boot run.
- No `factory/android/README.md` written yet.
- No `adb` getprop, `uname`, or boot-time receipts exist. Nothing is claimed as booting.

## Facts checked on this host
- `/dev/kvm` present (crw-rw---- root:kvm). Whether user `ubuntu` can open it was not tested.
- Docker 29.7.2 present.
- `/home` has 333 GB free (59% used). Sync budget must stay within the 200 GB cap.
- 12 CPUs, 45 GB RAM.
- `~/src/yolo-work/bunos-factory/` is absent, so there is no archive recipe to reuse. Work must start from upstream sources.
- Branch `claude/vps-android` has no remote ref on origin yet.

## Blockers
- Hard deadline of 22:45 UTC left about 10 minutes when this session started. A Cuttlefish fetch, container build and boot are not feasible inside that window.

## Next step
1. Confirm which Android x86_64 build is current on ci.android.com (aosp_cf_x86_64_phone). Do not assume a version number.
2. Run the android-cuttlefish Docker image with `--device /dev/kvm`. Install nothing on the host.
3. Boot headless. Record getprop, uname and boot-complete time under this directory.
4. Write factory/android/README.md with manifest, local_manifests, patch-queue layout and budgets.
