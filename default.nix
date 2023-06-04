{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-popup";
  version = "0.2.6";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
