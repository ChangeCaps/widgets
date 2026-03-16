{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "widgets";
  version = "0.1.0";
  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ori-0.1.0" = "sha256-jaeJDIt+RQbTF3+A6BZhv3q/4qYLxC6CBMGYnH7iopk=";
      "ori-native-0.1.0" = "sha256-EddC2zW1pKQ3wh9mHtGWrZMttaYdnJSZVwF/Gsv/bfk=";
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
