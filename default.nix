{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "widgets";
  version = "0.1.0";
  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ori-0.1.0" = "sha256-oSk4jti0nhCIKtjszUHGF6S0heJBMqeTfw/RnZEkgrI=";
      "ori-native-0.1.0" = "sha256-DH+jCthvVeDd8Biraa0feFN4kG4OJO8TeRoncI707NM=";
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
