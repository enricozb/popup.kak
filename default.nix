{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-popup";
  version = "0.2.1";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
