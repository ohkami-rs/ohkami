{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs@{
    flake-parts,
    systems,
    ...
  }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = import systems;
    perSystem = { pkgs, system, ... }: {
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ (inputs.rust-overlay.overlays.default) ];
      };

      devShells.default =
        let
          rustToolchain = pkgs.rust-bin.stable."1.100.0".default.override {
            extensions = [ "rust-src" ];
            targets = [ "wasm32-unknown-unknown" ];
          };
        in
        pkgs.mkShell {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
          packages = with pkgs; [
            rustToolchain
            go-task
            cargo-expand
            sqlx-cli
            sccache
            nodejs
            wrangler
            wasm-pack
            (worker-build.overrideAttrs (old: {
              buildInputs = (old.buildInputs or []) ++ [ zstd ];
            }))
          ];
        };
    };
  };
}

