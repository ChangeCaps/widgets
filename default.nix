{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "widgets";
  version = "0.1.0";
  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ori-0.1.0" = "sha256-QLYDHMgrq53B/cGGoaDWDd+9AC2kv6d2L7BKGvS0PV8=";
      "ori-native-0.1.0" = "sha256-6wF81bouWcFzbFWscWq50aBOGN4X9FmQkwPXOvhNzBE=";
    };
  };

  nativeBuildInputs = [
    pkgs.pkg-config
    pkgs.rust-analyzer
  ];

  buildInputs = [
    pkgs.gtk4
    pkgs.gtk4-layer-shell
    pkgs.librsvg
    pkgs.pulseaudio
    pkgs.dbus
  ];
}
