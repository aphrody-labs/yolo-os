# @yolo-rs/windows-winrt

Optional Windows Runtime APIs for Bun, implemented with `windows-rs` and a
Node-API addon. The package builds and loads the addon on Windows; it does not
provide the Windows OS package manager, the Win32 core API layer, or WinUI.

```ts
import { deviceFamily, folderInfo, parseUri } from "@yolo-rs/windows-winrt";
```

Build the native provider with `bun run build:napi`. The provider exposes URI
parsing, folder metadata, and device-family detection. Windows-only runtime
behavior still requires execution on a Windows host; Cargo metadata alone does
not qualify it.

License: Apache-2.0.
