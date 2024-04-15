{
  description = "popup.kak";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      kak-popup = pkgs.callPackage ./default.nix {};
    in {
      packages = {
        default = kak-popup;
        kak-popup = kak-popup;
      };

      devShells.default =
        pkgs.mkShell {packages = with pkgs; [cargo clippy rustc tmux];};
    })
    // {
      overlays.default = import ./overlay.nix;
    };
}
