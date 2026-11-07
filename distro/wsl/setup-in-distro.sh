#!/bin/bash
# setup-in-distro.sh: provision the YOLO Arch WSL distro (run as root inside it).
#
# Idempotent. Installs the Rust userland layer (sudo-rs, uutils-coreutils as
# PATH-first shims), Nix from [extra] with nix-daemon under systemd, and the
# YOLO toolchains (rustup, Bun, uv, clang, zig) for a non-root user.
#
# Usage (from Windows):
#   wsl -d yolo-arch -u root -- bash /mnt/c/.../setup-in-distro.sh
# Environment:
#   YOLO_USER   name of the non-root user (default: yolo)
#   YOLO_SHIMS  1 to enable uutils shims (default: 1), 0 to leave GNU first
#
# No secret is created or stored. The user has a locked password and
# passwordless sudo through sudo-rs (WSL already grants root via `wsl -u root`).
set -euo pipefail

YOLO_USER=${YOLO_USER:-yolo}
YOLO_SHIMS=${YOLO_SHIMS:-1}

log() { printf '[yolo-setup] %s\n' "$*"; }

[ "$(id -u)" -eq 0 ] || { echo "must run as root" >&2; exit 1; }

# 1. pacman keyring. The official image runs this in its OOBE script
#    (/usr/lib/wsl/first-setup.sh), which `--no-launch` skips.
if [ ! -s /etc/pacman.d/gnupg/pubring.gpg ] && [ ! -s /etc/pacman.d/gnupg/pubring.kbx ]; then
  log "initialising pacman keyring"
  pacman-key --init
fi
pacman-key --populate archlinux >/dev/null

# 2. Full upgrade plus packages.
log "pacman -Syu"
pacman -Syu --noconfirm --needed \
  sudo-rs uutils-coreutils nix rustup clang zig uv unzip which git

# 3. sudo-rs. Arch ships /usr/bin/sudo-rs and /usr/bin/su-rs, no PAM files
#    and no `provides=sudo`. Add PAM stacks and a sudoers file.
log "configuring sudo-rs"
for svc in sudo sudo-i; do
  if [ ! -e "/etc/pam.d/$svc" ]; then
    cat >"/etc/pam.d/$svc" <<'EOF'
#%PAM-1.0
auth     include system-auth
account  include system-auth
session  include system-auth
EOF
    chmod 0644 "/etc/pam.d/$svc"
  fi
done
if [ ! -e /etc/sudoers ]; then
  cat >/etc/sudoers <<'EOF'
# Managed by YOLO setup-in-distro.sh (sudo-rs reads this file)
Defaults env_reset
Defaults secure_path="/usr/local/sbin:/usr/local/bin:/usr/bin"
root ALL=(ALL:ALL) ALL
@includedir /etc/sudoers.d
EOF
  chmod 0440 /etc/sudoers
fi
install -d -m 0750 /etc/sudoers.d
printf '%%wheel ALL=(ALL:ALL) NOPASSWD: ALL\n' >/etc/sudoers.d/10-yolo-wheel
chmod 0440 /etc/sudoers.d/10-yolo-wheel
ln -sfn /usr/bin/sudo-rs /usr/local/bin/sudo
ln -sfn /usr/bin/su-rs /usr/local/bin/su

# 4. Non-root user.
if ! id "$YOLO_USER" >/dev/null 2>&1; then
  log "creating user $YOLO_USER"
  useradd -m -s /bin/bash -G wheel "$YOLO_USER"
  passwd -l "$YOLO_USER" >/dev/null
fi
usermod -aG wheel "$YOLO_USER"
# Arch nix-daemon.socket is mode 0666 and there is no nix-users group, so no
# extra group membership is needed for daemon access.

# 5. /etc/wsl.conf of THIS distro: systemd on, default user.
log "writing /etc/wsl.conf"
cat >/etc/wsl.conf <<EOF
# Managed by YOLO setup-in-distro.sh
[boot]
systemd=true

[user]
default=$YOLO_USER

# Keep the distro userland self-contained: without this, WSL appends the
# Windows PATH (including any Windows node/npm) to every Linux shell.
[interop]
appendWindowsPath=false
EOF
chmod 0644 /etc/wsl.conf

# 6. Nix daemon (socket-activated) and flakes for the user.
log "enabling nix-daemon"
# Arch nix 2.35.4-1 does not create /nix/store (its tmpfiles.d only makes
# /nix/var/nix/{daemon-socket,builds}); unprivileged clients then fail with
# 'opening file "/nix/store": No such file or directory'. One root call on the
# local store initialises it (root:nixbld 1775) and the db.
[ -d /nix/store ] || nix --extra-experimental-features nix-command store info >/dev/null
# Socket activation is the Arch default; also enable the service so that
# `systemctl is-active nix-daemon` is deterministic after boot.
systemctl enable --now nix-daemon.socket nix-daemon.service
install -d -o "$YOLO_USER" -g "$YOLO_USER" "/home/$YOLO_USER/.config" "/home/$YOLO_USER/.config/nix"
printf 'experimental-features = nix-command flakes\n' >"/home/$YOLO_USER/.config/nix/nix.conf"
chown "$YOLO_USER:$YOLO_USER" "/home/$YOLO_USER/.config/nix/nix.conf"

# 7. uutils shims (reversible: /usr/local/sbin/yolo-uutils-shim disable).
install -m 0755 "$(dirname "$0")/yolo-uutils-shim" /usr/local/sbin/yolo-uutils-shim
if [ "$YOLO_SHIMS" = 1 ]; then
  /usr/local/sbin/yolo-uutils-shim enable
fi

# 8. User-level toolchains.
log "user toolchains"
runuser -u "$YOLO_USER" -- bash -lc '
  set -euo pipefail
  rustup toolchain list | grep -q stable || rustup default stable
  rustup default stable >/dev/null
  if [ ! -x "$HOME/.bun/bin/bun" ]; then
    curl -fsSL https://bun.sh/install | bash
  fi
  grep -q ".bun/bin" "$HOME/.bashrc" 2>/dev/null || {
    printf "export BUN_INSTALL=\"\$HOME/.bun\"\nexport PATH=\"\$BUN_INSTALL/bin:\$PATH\"\n" >>"$HOME/.bashrc"
  }
'
log "done"
