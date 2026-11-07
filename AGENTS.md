# Repository guidance

- Use Bun for repository-owned TypeScript and JavaScript. Do not add Node.js,
  npm, pnpm or Yarn runtime dependencies.
- Preserve upstream attribution, URL, retrieval time, version/revision, license
  and content digest for gathered material.
- Keep research extraction deterministic and cite source URLs in generated
  documents. Do not present extracted text as a verified build instruction.
- Never run LFS build scripts implicitly. Require an explicit `--execute` flag.
- LFS execution is Linux-only. Keep source fetching, host checks, planning and
  build execution as distinct operations.
- Treat ALFS-generated commands and package checksums from the selected book as
  upstream-controlled inputs. Do not invent package versions or checksums.
- `yolo-bun` owns the `yolo pkg` workflow and deterministic package plans;
  `yolo-os pkg` forwards to that executable. Each native backend owns dependency
  resolution, signatures, installation, and rollback. `yolo-os` is not an
  independent package resolver or LFS build engine.

---

## Autonomous NHITL Agent & Context / Token Optimization Best Practices (24/7 Operations)

### 1. Context Window & Token Optimization
- **High-Precision Targeting**: Never dump large raw files into the model context. Use exact byte/line range reads or AST/symbol lookups.
- **Zero-Polling Reactive Wakeup**: When launching background tasks or subagents, do not poll in loops. Yield execution immediately and wait for reactive system events.
- **Compact & High Information Density**: Output concise, fact-based reports with clickable markdown file links (`[file.ext](file:///path/to/file)`). Do not narrate obvious actions.
- **Strict Zero-Emoji Policy**: Zero emojis in all code, comments, commit messages, PRs, logs, and agent communications.

### 2. Autonomous Execution (NHITL - No Human In The Loop)
- **Continuous Grind & Auto-Goal**: Independently identify objectives, resolve compile errors, apply fixes, format (`bun fmt`), lint (`bun lint`), typecheck (`bun check`), test (`bun test` / `cargo test`), commit, and push.
- **Pure Tech Stack**: 100% Upstream Bun v1.4.4+, Rust 2024, Python uv v0.13.7+, Go 1.27+. Zero Node.js, npm, yarn, or pnpm.
- **Strict Isolation & Safe Execution**: Heavy Rust builds run on vps (12 cores) with sccache under shared lock; web and fast tests run on bunos or local Bun.
