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
- `cargo-lfs-bun` is a launcher for the Bun CLI, not an independent LFS engine.
