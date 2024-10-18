{
  description = "A flake for building a Rust project with cargo2nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cargo2nix.url = "github:cargo2nix/cargo2nix/main";
    flake-utils.follows = "cargo2nix/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    cargo2nix,
    rust-overlay,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [cargo2nix.overlays.default rust-overlay.overlays.default];
      };
      rustPkgs = pkgs.rustBuilder.makePackageSet {
        packageFun = import ./Cargo.nix;
        rustVersion = "1.81.0";
        extraRustComponents = ["rustfmt" "clippy"];
      };
      workspaceShell = rustPkgs.workspaceShell {
        # packages = [ pkgs.somethingExtra ];
        # shellHook = ''
        #   export PS1="\033[0;31m☠dev-shell☠ $ \033[0m"
        # '';
      }; # supports override & overrideAttrs

      bootstrapShell = pkgs.mkShell {
        packages = [cargo2nix];
        # inputsFrom = [ cargo2nix ];
        nativeBuildInputs = cargo2nix.nativeBuildInputs;
      };
    in rec {
      packages = {
        pocbot = rustPkgs.workspace.jank_rs {};
        default = packages.jank_rs.bin;
      };
      devShells = {
        default = workspaceShell;
        # ootstrap = bootstrapShell;
      };
    });

  nixConfig = {
    extra-substituters = ["https://pocbot.cachix.org"];
    extra-trusted-public-keys = ["pocbot.cachix.org-1:CQf58F6rUcUA/mHTJN0YJRyK1AfIOUe8bu7lP45hhjo="];
  };
}
