{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = nixpkgs.legacyPackages;
    in {
      packages = forAllSystems (system: {
        default = pkgsFor.${system}.rustPlatform.buildRustPackage rec {
              pname = "cooklang-chef";
              version = "0.4.2";
              cargoLock.lockFile = ./Cargo.lock;
              src = pkgsFor.${system}.lib.cleanSource ./.;
            };
      });
    };
}

