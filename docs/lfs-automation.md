# LFS automation workflow

## Authority and inputs

Choose one exact upstream LFS book (stable or development, systemd or SysV),
then use its own host requirements, package list, checksums, errata and security
advisories. Package versions in each book are tested together. Development
snapshots can break temporarily.

The [ALFS/jhalfs project](https://www.linuxfromscratch.org/alfs/) extracts book
commands into scripts and a Makefile. Use it for generation rather than
converting rendered HTML code blocks independently. Check its prerequisites
and generated scripts against the selected book version.

## Commands in this repository

1. `cargo lfs book sync --track stable-systemd` retrieves the official
   `wget-list` and `md5sums` for the chosen track.
2. `cargo lfs plan --book <directory>` prints the inputs and download plan
   without package network writes.
3. `cargo lfs fetch --book <directory> --execute` retrieves missing files and
   verifies the official MD5 list.
4. Clone ALFS with `cargo lfs alfs setup --dir <path> --execute`, configure
   jhalfs with `make menuconfig`, select the matching book revision and leave
   `RUNMAKE`, `GETPKG` and `CLEAN` disabled. Review the target mount and
   configuration, then use `cargo lfs alfs check --dir <path>` and
   `cargo lfs alfs generate --dir <path> --execute` to generate scripts and
   Makefile without starting the package build.
5. Review host, mount, target disk, privilege and destructive-operation
   assumptions in the generated scripts and Makefile.
6. `cargo lfs run --script <generated-script>` previews. Add `--execute` to run
   the script under Bash on Linux.

## Execution boundary

LFS instructions can create filesystems, chroot, change ownership and install a
kernel or bootloader. Use a disposable Linux VM with a dedicated virtual disk.
This CLI does not provision a VM, pick a disk, mount partitions or claim a
build is reproducible across arbitrary hosts. Each executed step is journaled
with its ID, absolute script path, digest, start/end time and exit status. A
successful step with the same ID and digest is skipped on rerun. The journal
does not snapshot host state or external side effects within a partially
completed script; inspect a failed step before retrying it.

Windows supports research and planning; package retrieval with checksum
verification and build execution are intended for Linux. Use WSL2 or a Linux
VM for LFS construction.

## Why this delegates to ALFS

LFS is a learning process: the builder sees how the base system is assembled
from its documented sources and instructions. Rust makes the surrounding
workflow explicit and inspectable, while each package continues to use its own
upstream build system. ALFS already understands the book's structured
instructions and established build behavior. A second XML/XSLT extraction
engine without parity would hide steps and create an unverified build path.
