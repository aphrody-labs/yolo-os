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
