# LFS automation workflow

## Authority and inputs

Choose one exact upstream LFS book (stable or development, systemd or SysV),
then use its own host requirements, package list, checksums, errata and security
advisories. LFS warns that listed package versions are tested together. Do not
mix the current package list with an older book's checksum file. Development
snapshots can break temporarily.

The project's [ALFS/jhalfs](https://www.linuxfromscratch.org/alfs/) path
extracts book commands into scripts and a Makefile. Use it for build-step
generation rather than converting every code block from HTML independently.
Check its prerequisites and the generated scripts for the selected book version.

## Commands in this repository

1. `lfs-bun book sync --track stable-systemd` retrieves the official `wget-list`
   and `md5sums` for the chosen track.
2. `lfs-bun plan --book <directory>` prints the discovered input counts and
   download plan without network writes.
3. `lfs-bun fetch --book <directory> --execute` retrieves missing files to the
   source cache and verifies the official MD5 list.
4. Clone ALFS with `lfs-bun alfs setup --dir <path> --execute`, configure jhalfs
   with `make menuconfig`, select the matching book revision and set
   `RUNMAKE="n"`, `GETPKG="n"` and `CLEAN="n"`. Review the target mount and config, then use
   `lfs-bun alfs check --dir <path>` and
   `lfs-bun alfs generate --dir <path> --execute` to generate scripts and its
   Makefile without running the package build.
5. Read the host, mount, target disk, privilege and destructive-operation
   assumptions in the generated scripts and Makefile.
6. `lfs-bun run --script <generated-script>` previews. Add `--execute` to run
   the script under Bash on Linux.

## Execution boundary

LFS instructions can create filesystems, chroot, change ownership and install a
kernel/bootloader. Run builds inside a disposable Linux VM with a dedicated
virtual disk. This CLI does not provision a VM, pick a disk, mount partitions or
claim that the build is reproducible across arbitrary hosts. Each executed step
is journaled with its ID, absolute script path, script digest, start/end time and
exit status. A successful step with the same ID and digest is skipped on rerun,
which supports continuation after interruption between scripts. The journal
does not snapshot the host environment, disk state or external side effects
within a partially completed script; inspect a failed step before retrying it.

The Cargo launcher works wherever Bun is installed, but source acquisition with
verification and build execution are supported only on Linux. On Windows use
the research/crawl functions and run LFS inside WSL2 or a Linux VM.

## Why this delegates to ALFS

ALFS already understands the book's structured instructions and has years of
LFS-specific build behavior. The Bun tool adds source inventory, crawl
provenance, planning, checksum-verified downloads and an explicit execution
gate. Reimplementing XML/XSLT extraction without parity would create a second,
unverified build engine.
