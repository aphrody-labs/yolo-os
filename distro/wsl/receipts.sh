#!/bin/bash
# receipts.sh: print the verification receipts for the YOLO Arch WSL distro.
# Run as the default (non-root) user: wsl -d yolo-arch -- bash -l <path>/receipts.sh
set -uo pipefail
cd "$HOME"
export PATH="$HOME/.bun/bin:$HOME/.cargo/bin:$PATH"

run() {
  printf '\n$ %s\n' "$*"
  bash -c "$*" 2>&1
  printf '[exit %s]\n' "$?"
}

printf '# receipts generated %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
run 'id'
run 'uname -a'
run 'cat /etc/os-release'
run 'cat /etc/wsl.conf'
run 'ps -p 1 -o comm=; systemctl is-system-running'
run 'command -v ls; readlink -f "$(command -v ls)"; ls --version'
run '/usr/local/sbin/yolo-uutils-shim status'
run 'command -v sudo; readlink -f "$(command -v sudo)"; sudo --version'
run 'sudo -n true && echo sudo-noninteractive-ok'
run 'sudo -n id -u'
run 'nix --version'
run 'systemctl is-active nix-daemon; systemctl is-active nix-daemon.socket'
run 'nix store info'
run 'nix config show experimental-features'
run 'rustup --version 2>/dev/null; rustup show active-toolchain'
run 'rustc -V; cargo -V'
run 'command -v bun; bun --version; bun --revision'
run 'uv --version'
run 'clang --version | head -1'
run 'zig version'
run 'command -v node npm || echo "node/npm: absent"'
run 'nix run nixpkgs#hello'
run 'nix eval --raw nixpkgs#hello.outPath'
run 'nix flake metadata nixpkgs --json | bun -e "const m=JSON.parse(await Bun.stdin.text()); console.log(m.resolvedUrl ?? m.url, m.locked?.rev, m.locked?.narHash)"'
run 'pacman -Q sudo-rs uutils-coreutils nix rustup clang zig uv linux-api-headers glibc systemd pacman'
run 'df -h /'
