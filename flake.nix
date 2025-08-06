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
            stable.toolchain
          ];
        libs = with pkgs; [
          pkg-config
          openssl
        ];
      in {
        nixpkgs.overlays = [fenix.overlays.default];

        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = libs;

            packages = [
              clang
              rust_fenix
              rust-analyzer
            ];

            LD_LIBRARY_PATH = lib.makeLibraryPath libs;
          };
      }
    );
}
