{
  description = "Discord bot for interacting with subsonic music libraries";

  nixConfig = {
    extra-substituters = [ "https://cache.m7.rs" ];
    extra-trusted-public-keys =
      [ "cache.m7.rs:kszZ/NSwE/TjhOcPPQ16IuUiuRSisdiIwhKZCxguaWg=" ];
  };

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";

  outputs = { self, nixpkgs }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
      forAllPkgs = f: forAllSystems (sys: f pkgsFor.${sys});
      pkgsFor = nixpkgs.legacyPackages;
    in {
      nixosModules.default = import ./module.nix;

      overlays.default = final: _prev: {
        disconic = final.callPackage ./default.nix { };
      };

      packages =
        forAllPkgs (pkgs: { default = pkgs.callPackage ./default.nix { }; });

      devShells = forAllPkgs (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ self.outputs.packages.${pkgs.system}.default ];
          buildInputs = with pkgs; [ clippy rust-analyzer rustc rustfmt ];
        };
      });

      hydraJobs = self.outputs.packages;
    };
}
