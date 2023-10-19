{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-popup";
  version = "0.4.3";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
