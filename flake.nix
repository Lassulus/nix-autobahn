{
  description = "My haskell application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        packageName = "nix-autobahn";
      in {
        packages.${packageName} = pkgs.stdenv.mkDerivation {
          name = packageName;
          src = ./.;
          installPhase = ''
            echo nix-autobahn*
            mkdir -p $out/bin
            mv nix-autobahn* $out/bin
          '';
        };

        defaultPackage = self.packages.${system}.${packageName};
      });
}
