{
  description = "lazynixos - A Rust TUI for NixOS deployments";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            rustPackages.clippy
            rust-analyzer
          ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "lazynixos";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      }
    );
}
