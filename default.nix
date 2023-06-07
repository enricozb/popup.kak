{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-popup";
  version = "0.4.0-alpha.1";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
