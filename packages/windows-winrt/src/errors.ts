// SPDX-License-Identifier: Apache-2.0
export type WindowsWinrtErrorCode =
  "ERR_WINDOWS_UNSUPPORTED" | "ERR_WINDOWS_PROVIDER_MISSING";

export class WindowsWinrtError extends Error {
  readonly code: WindowsWinrtErrorCode;
  constructor(code: WindowsWinrtErrorCode, message: string) {
    super(message);
    this.name = "WindowsWinrtError";
    this.code = code;
  }
}

export function assertWindows(): void {
  if (process.platform !== "win32")
    throw new WindowsWinrtError(
      "ERR_WINDOWS_UNSUPPORTED",
      "@yolo-rs/windows-winrt needs Windows",
    );
}
