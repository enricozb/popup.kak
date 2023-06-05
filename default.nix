{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-popup";
  version = "0.2.7";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
