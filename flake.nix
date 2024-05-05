{
  description = "Discord bot for interacting with subsonic music libraries";

  nixConfig = {
    extra-substituters = [ "https://cache.m7.rs" ];
    extra-trusted-public-keys =
      [ "cache.m7.rs:kszZ/NSwE/TjhOcPPQ16IuUiuRSisdiIwhKZCxguaWg=" ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    systems.url = "github:nix-systems/default";
  };

  outputs = { self, nixpkgs, systems }:
    let
      forAllSystems = nixpkgs.lib.genAttrs (import systems);
      forAllPkgs = f: forAllSystems (sys: f pkgsFor.${sys});
      pkgsFor = nixpkgs.legacyPackages;

      mkPackage = pkgs: pkgs.callPackage ./default.nix { };
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
