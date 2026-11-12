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

      # nix flake check: the unit tests, in seconds (tests/run, the end-to-end one, needs a real session)
      checks = forAll (pkgs:
        let p = self.packages.${pkgs.stdenv.hostPlatform.system}; in {
          # the plugin's logic, tested natively (zellij-tile stands in for zellij there)
          # (natively, zellij-tile's own dependencies want OpenSSL, left out of the wasm build)
          plugin = p.plugin.overrideAttrs (o: {
            doCheck = true;
            checkPhase = "cargo test --frozen";
            nativeBuildInputs = (o.nativeBuildInputs or [ ]) ++ [ pkgs.pkg-config ];
            buildInputs = (o.buildInputs or [ ]) ++ [ pkgs.openssl ];
          });
          # the scripts' rules, with fake fzf, zellij and delta where they would need a session
          scripts = pkgs.runCommand "hidex-unit-tests" { } ''
            ${pkgs.bash}/bin/bash ${./tests/unit} ${p.hidex} && touch $out
          '';
        });
    };
}
