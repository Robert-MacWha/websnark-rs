{
  description = "Dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    oldNixPkgs.url = "github:NixOS/nixpkgs/ae5fe741ba9acade281a9185139e3922811c9696";
    unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
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
      oldNixPkgs,
      unstable,
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

        oldPkgs = import oldNixPkgs {
          inherit system;
        };

        nodejs14 = oldPkgs.nodejs_14.override {
          python3 = oldPkgs.python39;
        };

        unstablePkgs = import unstable {
          inherit system;
        };

        rustToolchain = pkgs.rust-bin.nightly."2025-11-15".default.override {
          extensions = [
            "rust-src"
            "llvm-tools"
            "rust-analyzer"
          ];
          targets = [
            "wasm32-unknown-unknown"
          ];
        };

        rustfmtNightly = pkgs.rust-bin.nightly."2025-11-15".rustfmt;
      in
      {
        devShells = {
          default = pkgs.mkShell {
            packages = [
              rustToolchain
              rustfmtNightly

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

          # The CLI expects nodejs 14. It'll probably work with newer versions, but safer to use the correct version.
          # Unfortunately it does takea a hot second to build
          tc = pkgs.mkShell {
            packages = [
              nodejs14
            ];
          };
        };
      }
    );
}
