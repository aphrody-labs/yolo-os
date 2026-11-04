# Documentation

- [Research synthesis](research.md): findings across the requested Linux,
  Asterinas, GNU, LFS, uutils and Linux Foundation sources.
- [Dated GitHub ranking capture](research/github-repository-rankings-2026-11-04.md)
  and [JSON evidence](research/github-repository-rankings-2026-11-04.json): all
  public repositories returned for the requested orgs/user, sorted by stars.
- [Web source captures](research/web-source-captures-2026-11-04.md) and
  [JSON evidence](research/web-source-captures-2026-11-04.json): 60 crawl
  attempts, 59 raw response bodies, and one recorded out-of-origin redirect
  block, with timestamps and SHA-256 digests.
- [LFS package URL list](research/lfs-stable-systemd-2026-11-04/wget-list) and
  [matching checksum list](research/lfs-stable-systemd-2026-11-04/md5sums):
  original official inputs used to produce the manifest capture.
- [LFS 13.1-systemd manifest capture](research/lfs-stable-systemd-2026-11-04.json):
  source URLs, observed counts and checksums for the official `wget-list` and
  `md5sums` files.
- [Git remote heads](research/git-remote-heads-2026-11-04.md) and
  [JSON evidence](research/git-remote-heads-2026-11-04.json): default ref,
  commit ID and response digest for kernel.org, LFS, Asterinas and uutils.
- [Source registry](sources.md): canonical URLs, authority boundaries and
  metadata requirements for each source family.
- [System design](architecture.md): crawler, evidence model, CLI and Cargo
  launcher boundaries.
- [LFS automation](lfs-automation.md): supported acquisition workflow, ALFS
  integration and execution controls.
- [CLI reference](cli.md): commands, flags and output format.
- [Decisions and limitations](decisions.md): rationale and known gaps.

Research state in this initial documentation was checked on 2026-11-04. Crawl
artifacts should be regenerated to check mutable pages, stars, branches and
release channels.
