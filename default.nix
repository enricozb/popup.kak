{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "kak-popup";
  version = "0.6.3-beta";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };

  postInstall = ''
    mkdir -p $out/share/kak/autoload/plugins/
    cp -r ${./rc} $out/share/kak/autoload/plugins/popup.kak/
  '';
}
