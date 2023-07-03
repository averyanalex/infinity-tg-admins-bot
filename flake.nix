{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    flake-utils.url = "github:numtide/flake-utils";
    import-cargo.url = "github:edolstra/import-cargo";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    import-cargo,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
        inherit (import-cargo.builders) importCargo;

        rustDev = pkgs.rust-bin.stable.latest.default;
        rustBuild = pkgs.rust-bin.stable.latest.minimal;

		devInputs = [rustDev] ++ (with pkgs; [alejandra]);
        buildInputs = with pkgs; [openssl];
        nativeBuildInputs = with pkgs; [pkg-config];

        package = pkgs.stdenv.mkDerivation {
          name = "template";
          src = self;

          buildInputs = buildInputs;

          nativeBuildInputs =
            nativeBuildInputs
            ++ [
              (importCargo {
                lockFile = ./Cargo.lock;
                inherit pkgs;
              })
              .cargoHome
            ]
            ++ [rustBuild];

          buildPhase = ''
            cargo build --release --offline
          '';

          installPhase = ''
            install -Dm775 ./target/release/template $out/bin/template
          '';
        };
      in {
        packages = {
          default = package;
          template = package;
        };
        devShells = {
          default = pkgs.mkShell {
            buildInputs = devInputs ++ buildInputs ++ nativeBuildInputs;
          };
        };
      }
    );
}
