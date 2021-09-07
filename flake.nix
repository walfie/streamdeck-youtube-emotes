# Originally generated via template:
# `nix flake init -t github:serokell/templates#rust-crate2nix`
# https://serokell.io/blog/practical-nix-flakes#rust-(cargo)
#
# With additional changes inspired by:
# https://www.srid.ca/rust-nix
{
  inputs = {
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs";
  };

  outputs = { self, nixpkgs, crate2nix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        crateName = "streamdeck-youtube-emotes";

        inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
          generatedCargoNix;

        project = import
          (generatedCargoNix {
            name = crateName;
            src = ./.;
          })
          {
            inherit pkgs;
            defaultCrateOverrides = pkgs.defaultCrateOverrides // {
              ${crateName} = _: {
                buildInputs = pkgs.lib.optionals (system == "x86_64-darwin") [
                  # Needed to resolve the following error:
                  # `ld: framework not found Security`
                  pkgs.darwin.apple_sdk.frameworks.Security
                ];
              };
            };
          };
      in
      rec {
        # `nix build`
        packages.${crateName} = project.rootCrate.build;
        defaultPackage = self.packages.${system}.${crateName};

        # `nix run`
        apps.${crateName} = flake-utils.lib.mkApp {
          name = crateName;
          drv = packages.${crateName};
        };
        defaultApp = apps.${crateName};

        # `nix develop`
        devShell = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.packages.${system};
          buildInputs = [ pkgs.cargo pkgs.cargo-watch ];
        };
      });
}
