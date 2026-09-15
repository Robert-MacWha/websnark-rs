{
  description = "Dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Rust nightly toolchain required for wasm32-unknown-unknown target.
        # https://github.com/rust-lang/rust/issues/77839
        rustToolchain = pkgs.rust-bin.nightly."2026-09-03".default.override {
          extensions = [
            "rust-src"
            "llvm-tools"
            "rust-analyzer"
          ];
          targets = [
            "wasm32-unknown-unknown"
          ];
        };

        rustfmtNightly = pkgs.rust-bin.nightly."2026-09-03".rustfmt;
      in
      {
        devShells = {
          default = pkgs.mkShell {
            packages = [
              rustToolchain
              rustfmtNightly

              pkgs.bacon
              pkgs.nodejs
              pkgs.wasm-bindgen-cli_0_2_108
              pkgs.wasm-pack
              pkgs.wabt
              pkgs.binaryen
              pkgs.geckodriver

              # CI
              pkgs.cargo-edit
              pkgs.git-cliff
              pkgs.just
            ];
          };

          ci = pkgs.mkShell {
            packages = [
              rustToolchain
              rustfmtNightly

              pkgs.wasm-pack
              pkgs.wasm-bindgen-cli_0_2_108
              pkgs.nodejs
            ];
          };
        };
      }
    );
}
