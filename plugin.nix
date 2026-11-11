{ makeRustPlatform, rust-bin }:

# hIDEx's zellij plugin (plugin/), built for wasm32-wasip1 with the official toolchain from rust-overlay.
let
  toolchain = rust-bin.stable.latest.minimal.override { targets = [ "wasm32-wasip1" ]; };
  rustPlatform = makeRustPlatform { cargo = toolchain; rustc = toolchain; };
in
rustPlatform.buildRustPackage {
  pname = "hidex-panes";
  version = "0.1.0";
  src = ./plugin;
  cargoLock.lockFile = ./plugin/Cargo.lock;
  buildPhase = ''
    runHook preBuild
    cargo build --release --frozen --target wasm32-wasip1
    runHook postBuild
  '';
  installPhase = ''
    install -Dm644 target/wasm32-wasip1/release/hidex-panes.wasm $out/hidex-panes.wasm
  '';
  doCheck = false;
}
