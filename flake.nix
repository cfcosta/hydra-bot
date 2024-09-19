{
  description = "Hydra control plane";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";

  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        inherit (pkgs) makeRustPlatform mkShell rust-bin;
        inherit (pkgs.lib) optionals;
        inherit (pkgs.stdenv) isDarwin;

        rust = rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        platform = makeRustPlatform {
          rustc = rust;
          cargo = rust;
        };
      in
      {
        packages.default = platform.buildRustPackage {
          name = "hydra-control-plane";
          src = ./.;
          buildInputs =
            with pkgs;
            (
              [
                pkg-config
                openssl
              ]
              ++ optionals isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
              ]
            );
          cargoLock = {
            lockFile = ./Cargo.lock;

            outputHashes = {
              "pallas-0.29.0" = "sha256-P//R/17kMaqN4JGHFFTMy2gbo7k+xWUaqkF0LFVUxWQ=";
            };
          };
        };

        devShells.default = mkShell {
          buildInputs =
            [
              rust
              pkgs.pkg-config
              pkgs.openssl
              pkgs.python312Packages.virtualenvwrapper
            ]
            ++ optionals isDarwin [
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

          shellHook =
            let
              lib-path = pkgs.lib.makeLibraryPath [
                pkgs.libffi
                pkgs.openssl
                pkgs.stdenv.cc.cc
              ];
            in
            ''
              # Augment the dynamic linker path
              export LD_LIBRARY_PATH="${lib-path}"
              SOURCE_DATE_EPOCH=$(date +%s)

              if test ! -d .venv; then
                virtualenv .venv
              fi

              source ./.venv/bin/activate

              export PYTHONPATH=`pwd`/.venv/${pkgs.python312.sitePackages}/:$PYTHONPATH

              [ -e .venv/bin/aider ] || pip install git+https://github.com/paul-gauthier/aider.git
            '';
        };
      }
    );
}
