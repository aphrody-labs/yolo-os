# SPDX-License-Identifier: MPL-2.0
# Adds the official Bun 1.4.4 linux-x64 (glibc) binary to the guest.
# Source and digest: https://github.com/oven-sh/bun/releases/tag/bun-v1.4.4
# sha256 bf8ccbfb2b0b80159e0f9a6d15c55317add8077534dbc6130a3edb52fccb395c
{ pkgs, ... }:
let
  bun-1-4-4 = pkgs.stdenvNoCC.mkDerivation {
    pname = "bun";
    version = "1.4.4";
    src = pkgs.fetchurl {
      url = "https://github.com/oven-sh/bun/releases/download/bun-v1.4.4/bun-linux-x64.zip";
      sha256 = "bf8ccbfb2b0b80159e0f9a6d15c55317add8077534dbc6130a3edb52fccb395c";
    };
    nativeBuildInputs = [
      pkgs.unzip
      pkgs.autoPatchelfHook
    ];
    buildInputs = [ pkgs.stdenv.cc.cc.lib ];
    dontConfigure = true;
    dontBuild = true;
    unpackPhase = ''
      unzip -q $src
    '';
    installPhase = ''
      install -Dm755 bun-linux-x64/bun $out/bin/bun
    '';
  };
in
{
  environment.systemPackages = [ bun-1-4-4 ];
}
