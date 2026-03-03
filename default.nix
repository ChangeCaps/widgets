{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage {
  pname = "widgets";
  version = "0.1.0";
  src = pkgs.lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "ori-0.1.0" = "sha256-VqQ2SSJiqnjvG7GeeXJJrg1p4ZYdSo4GBM+BAkcaUjg=";
      "ori-native-0.1.0" = "sha256-WhTgBb42z1isVhA7tSP33bGNNt2aM9V0tI3lGv+PZVA=";
    };
  };

  nativeBuildInputs = [
    pkgs.pkg-config
  ];

  buildInputs = [
    pkgs.gtk4
    pkgs.gtk4-layer-shell
    pkgs.librsvg
    pkgs.pulseaudio
    pkgs.dbus
  ];
}
