{
  description = "Shell script collection to download ELF binaries and use them right away!";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: let
      l = nixpkgs.lib // builtins;

      scriptNames =
        l.filter
        (script: l.hasPrefix "nix-autobahn" script)
        (l.attrNames (l.readDir ./.));
  in
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        packageName = "nix-autobahn";

        scripts =
          l.map
            (scriptName:
              pkgs.writeShellApplication {
                name = scriptName;
                text = l.readFile ("${./.}/${scriptName}");

                runtimeInputs = with pkgs; [
                  findutils
                  fzf
                  nix-index
                  nix-ld
                ];
              })
            scriptNames;
      in {

        packages.${packageName} = pkgs.runCommand "nix-autobahn" {} ''
          mkdir -p $out/bin
          for script in ${l.toString scripts}; do
            cp $script/bin/* $out/bin/
          done
        '';

        defaultPackage = self.packages.${system}.${packageName};
      });
}
