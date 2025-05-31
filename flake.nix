{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nativeBuildInputs = [
          (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
        ];
      in
      with pkgs;
      {
        devShells.default = mkShell {
          inherit nativeBuildInputs;
          # This is needed or rust-analyzer will not work correctly.
          # Source: https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570
          RUST_SRC_PATH = "${
            pkgs.rust-bin.stable.latest.default.override { extensions = [ "rust-src" ]; }
          }/lib/rustlib/src/rust/library";
        };
        packages.default = callPackage ./. { };
      }
    );
}
