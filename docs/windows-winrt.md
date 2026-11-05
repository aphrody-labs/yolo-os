# Windows Runtime provider

`packages/windows-winrt` owns the optional Bun WinRT provider backed by the
`windows-rs` Node-API addon in `crates/windows-native-napi`. The package is
separate from yolo-cross's compiler providers and from the Bun `bun:ffi`
Win32/registry/shell layer.

The addon currently exposes Windows.Foundation URI parsing,
Windows.Storage.StorageFolder metadata, and the AnalyticsInfo device family.
Build it on Windows with `bun run windows:winrt:build:napi`; check its TypeScript
surface with `bun run windows:winrt:check`. `cargo check -p yolo-windows-napi`
validates Rust compilation but does not link or run the provider. Runtime
qualification still requires a Windows build and calls through Bun's Node-API
loader.

Do not treat this crate as Windows 7 support evidence. Its current target is the
active Windows host and WinRT APIs; yolo-cross owns MSVC and cross-target
toolchain preparation. The package remains private until the target repository
and registry publication path are available.
