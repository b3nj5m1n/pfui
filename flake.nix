{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        libPath = with pkgs; lib.makeLibraryPath [
          libpulseaudio
        ];

        naersk' = pkgs.callPackage naersk {};

        cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;
          name = cargoToml.package.name;
          version = cargoToml.package.version;
          nativeBuildInputs = [ pkgs.makeWrapper ];
          buildInputs = with pkgs; [
              libpulseaudio
          ];
          postInstall = ''
              wrapProgram "$out/bin/pfui" --prefix LD_LIBRARY_PATH : "${libPath}"
              '';

        };
        LD_LIBRARY_PATH = libPath;
      }
    );
}
