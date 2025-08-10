{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        rust_fenix = with fenix.packages.${system};
          combine [
            (stable.withComponents [
              "cargo"
              "rustc"
            ])
          ];
        rust_fenix_dev = with fenix.packages.${system};
          combine [
            (stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
          ];
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust_fenix;
          rustc = rust_fenix;
        };
        rust = pkgs.rust-bin.stable.latest.default;

        native_libs = with pkgs; [
          pkg-config
          openssl
        ];
        native_libs_path = pkgs.lib.makeLibraryPath native_libs;
        libs = with pkgs; [
          pkg-config
          openssl
          onnxruntime
        ];
        libs_path = pkgs.lib.makeLibraryPath libs;

        kitten-cli = rustPlatform.buildRustPackage {
          nativeBuildInputs = native_libs;
          buildInputs = libs;
          pname = "kitten-cli";
          version = "0.1.0";
          src = ./.;

          LD_LIBRARY_PATH = libs_path;
          ORT_LIB_LOCATION = "${pkgs.onnxruntime}/lib";

          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [
            "--package"
            "kittentts-cli"
          ];
        };
      in {
        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = native_libs;

            packages = [
              rust_fenix_dev
              rust-analyzer
            ];

            LD_LIBRARY_PATH = libs_path;
            ORT_LIB_LOCATION = "${pkgs.onnxruntime}/lib";
          };

        packages = {
          kitten-cli = kitten-cli;
          default = kitten-cli;
        };
      }
    );
}
