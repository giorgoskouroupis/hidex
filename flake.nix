{
  description = "tIDEx development: the tools to build its release (scripts/release) and test it";

  # tIDEx itself is installed with install.sh, the same on every Linux; this flake only gives a shell to work
  # on it here: nix develop, then scripts/release, tests/unit, tests/run
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    # the official Rust toolchain with the wasm32-wasip1 target, for the zellij plugin
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
      # the compiler pinned in plugin/rust-toolchain.toml, as the release build uses it
      rust = pkgs.rust-bin.fromRustupToolchainFile ./plugin/rust-toolchain.toml;
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        # pkg-config and openssl: the plugin's cargo test, natively (zellij-tile's own dependencies want them)
        packages = with pkgs; [ rust curl gnutar xz unzip gzip pkg-config openssl ];
      };
    };
}
