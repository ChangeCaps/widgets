{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "widgets";
  version = "0.1.0";
  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ori-0.1.0" = "sha256-1qrW9PUIoDv5VB22Qci6b8o0aIgleqtGeZ/eIkhjcJQ=";
      "ori-native-0.1.0" = "sha256-hBnoKPXNRg9QHp5wYPIwxEfgAQPIciD9v0UlaKHSLk8=";
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
