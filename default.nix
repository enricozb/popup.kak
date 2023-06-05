{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-popup";
  version = "0.2.8";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
