{
  description = "hIDEx: a terminal IDE around Helix, in zellij";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    # the official Rust toolchain with the wasm32-wasip1 target, for the zellij plugin
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      forAll = f: nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ] (system:
        f (import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; }));
    in
    {
      packages = forAll (pkgs: rec {
        plugin = pkgs.callPackage ./plugin.nix { };
        hidex = pkgs.callPackage ./package.nix { inherit plugin; };
        default = hidex;
      });
    };
}
