{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-popup";
  version = "0.6.1-beta";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
