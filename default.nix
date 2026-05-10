{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "widgets";
  version = "0.1.0";
  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ori-0.1.0" = "sha256-+Q4plQ7RobXtsOlYWmfJxCsx8hcIFvyFz5Xtx9e8Rw4=";
      "ori-native-0.1.0" = "sha256-HTqwZodpmUvBd9FxGTZzinLqgHLb2PTKnDzEY3Elgvo=";
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
