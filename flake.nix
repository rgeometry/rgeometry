{
  description = "rgeometry flake with 'earclip' demo packaged as a single HTML output";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    alejandra.url = "github:kamadorueda/alejandra";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    alejandra,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = ./.;

        mkDemo = demoName:
          craneLib.buildPackage {
            inherit src;
            preBuild = "cd demos/${demoName}";
            buildPhaseCargoCommand = "HOME=$PWD/tmp wasm-pack build --release --target no-modules --out-dir pkg --mode no-install";
            doNotPostBuildInstallCargoBinaries = true;
            cargoLock = ./. + "/demos/${demoName}/Cargo.lock";
            installPhaseCommand = ''
              mkdir -p $out
              ${pkgs.bash}/bin/bash "$src/utils/merge.sh" -o "$out/${demoName}.html" \
                  "pkg/${demoName}_bg.wasm" "pkg/${demoName}.js"
            '';
            doCheck = false;
            nativeBuildInputs = with pkgs; [
              wasm-pack
              wasm-bindgen-cli
              binaryen
            ];
          };
        lib = pkgs.lib;
        demosDir = builtins.readDir ./demos;
        demoNames = lib.attrNames (lib.filterAttrs (name: kind: kind == "directory" && builtins.pathExists (./. + "/demos/${name}/Cargo.lock")) demosDir);
        allDemos = pkgs.symlinkJoin {
          name = "rgeometry-demos";
          paths = builtins.map (n: mkDemo n) demoNames;
        };
      in {
        packages = let
          demoPkgs = builtins.listToAttrs (map (name: {
              inherit name;
              value = mkDemo name;
            })
            demoNames);
        in
          demoPkgs
          // {
            all-demos = allDemos;
            default = self.packages.${system}.all-demos;
          };

        formatter = alejandra.defaultPackage.${system};

        apps.pre-commit = {
          type = "app";
          program = toString (pkgs.writeShellScript "pre-commit" ''
            set -e
            echo "Running pre-commit checks..."

            echo "→ Checking Nix formatting..."
            ${alejandra.defaultPackage.${system}}/bin/alejandra --check .

            echo "→ Checking TOML formatting..."
            ${pkgs.taplo}/bin/taplo fmt --check

            echo "→ Checking Rust formatting..."
            ${rustToolchain}/bin/cargo fmt --all --check

            echo "→ Running clippy..."
            ${rustToolchain}/bin/cargo clippy --all-targets --all-features -- -D warnings

            echo "→ Running tests..."
            ${rustToolchain}/bin/cargo test --all-features

            echo "→ Checking demos..."
            for demo in ${lib.concatStringsSep " " demoNames}; do
              echo "  → Checking demo: $demo"
              (cd demos/$demo && ${rustToolchain}/bin/cargo check --target wasm32-unknown-unknown)
              (cd demos/$demo && ${rustToolchain}/bin/cargo clippy --target wasm32-unknown-unknown -- -D warnings)
            done

            echo "✓ All pre-commit checks passed!"
          '');
        };
      }
    );
}
