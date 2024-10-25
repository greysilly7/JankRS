{
  # ...

  inputs = {
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    crate2nix.url = "github:nix-community/crate2nix";
    # ...
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    rust-overlay,
    crate2nix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
      ];

      perSystem = {
        system,
        pkgs,
        lib,
        inputs',
        ...
      }: let
        cargoNix = import ./Cargo.nix {inherit pkgs;};
      in rec {
        checks = {
          jank_rs = cargoNix.rootCrate.build.override {
            runTests = true;
          };
        };

        packages = {
          jank_rs = cargoNix.rootCrate.build;
          default = packages.jank_rs;
        };
      };
    };
}
