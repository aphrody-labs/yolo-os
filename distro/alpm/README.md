# distro/alpm

Pure-Rust, read-only pacman sync-database reader (`yolo-alpm`) and CLI (`yolo-pm`).

    export RUSTUP_TOOLCHAIN=1.99.1-x86_64-unknown-linux-gnu   # on hosts whose stable manifest is broken
    cargo test --workspace
    cargo build --release -p yolo-pm
    target/release/yolo-pm --cache ~/.cache/yolo-pm sync
    target/release/yolo-pm --cache ~/.cache/yolo-pm deps hyprland

Scope: parse `core`, `extra`, `cachyos` databases; `vercmp`; dependency closure.
Out of scope: installing, transactions, hooks, scriptlets, signature verification (see docs/receipts/r2-alpm/STATUS.md).
