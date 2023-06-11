{
  description = "Discord bot for interacting with subsonic music libraries";

  nixConfig = {
    extra-substituters = [ "https://cache.m7.rs" ];
    extra-trusted-public-keys =
      [ "cache.m7.rs:kszZ/NSwE/TjhOcPPQ16IuUiuRSisdiIwhKZCxguaWg=" ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
      forAllPkgs = f: forAllSystems (sys: f pkgsFor.${sys});
      pkgsFor = forAllSystems (system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      });
      mkPackage = pkgs: pkgs.callPackage ./default.nix {
        rustPlatform = pkgs.makeRustPlatform rec {
          rustc = pkgs.rust-bin.stable.latest.default;
          cargo = rustc;
        };
      };
    in {
      nixosModules.default = import ./module.nix;

      overlays.default = final: _prev: {
        disconic = mkPackage final;
      };

      packages = forAllPkgs (pkgs: {
        default = mkPackage pkgs;
      });

      devShells = forAllPkgs (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [(mkPackage pkgs)];
          buildInputs = with pkgs; [ clippy rust-analyzer rustc rustfmt ];
        };
      });

      hydraJobs = self.outputs.packages;
    };
}
