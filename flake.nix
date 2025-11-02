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

        # Filter source to only include Rust-related files to minimize rebuilds
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            # Root level Rust files
            ./Cargo.toml
            ./Cargo.lock
            ./src
            ./benches
            ./examples
            ./tests
            # Configuration files
            ./rust-toolchain.toml
            ./rustfmt.toml
            ./taplo.toml
            # Metadata
            ./README.md
            ./LICENSE
            # rgeometry-wasm crate
            ./rgeometry-wasm/Cargo.toml
            ./rgeometry-wasm/src
            ./rgeometry-wasm/rustfmt.toml
            # Demo crates
            (lib.fileset.fileFilter
              (file: file.hasExt "toml" || file.hasExt "lock" || file.hasExt "rs")
              ./demos)
            # Utils needed for demo builds
            ./utils
          ];
        };

        # Common arguments for building the library
        commonArgs = {
          inherit src;
          strictDeps = true;
          cargoExtraArgs = "--all-features";
          nativeBuildInputs = with pkgs; [
            pkg-config
            m4
          ];
          buildInputs = with pkgs; [
            gmp
            mpfr
          ];
          # Use system GMP and MPFR instead of building from source
          GMP_MPFR_SYS_CACHE = "no-test";
        };

        # Build dependencies only (for caching)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

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

        # Build documentation with rustdoc and include demo HTML files
        documentation = (craneLib.cargoDoc (commonArgs
          // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "--html-in-header ${./doc-header.html}";
          })).overrideAttrs (oldAttrs: {
          # After building docs, include demo HTML files
          postInstall = ''
            ${pkgs.bash}/bin/bash -c 'cp -v ${allDemos}/*.html $out/ 2>/dev/null || true'
          '';
        });
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
            documentation = documentation;
            default = self.packages.${system}.all-demos;
          };

        formatter = alejandra.defaultPackage.${system};

        checks = {
          # Run the library tests
          rgeometry-test = craneLib.cargoTest (commonArgs
            // {
              inherit cargoArtifacts;
            });

          # Run the doc tests
          rgeometry-doc-test = craneLib.cargoDocTest (commonArgs
            // {
              inherit cargoArtifacts;
            });

          # Run clippy
          rgeometry-clippy = craneLib.cargoClippy (commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- -D warnings";
            });

          # Check Nix formatting
          alejandra-check = pkgs.runCommand "alejandra-check" {} ''
            ${alejandra.defaultPackage.${system}}/bin/alejandra --check ${src}
            touch $out
          '';

          # Check TOML formatting with crane
          taplo-fmt-check = craneLib.taploFmt (commonArgs
            // {
              inherit cargoArtifacts;
              cargoExtraArgs = "";
            });

          # Check Rust formatting with crane
          cargo-fmt-check = craneLib.cargoFmt (commonArgs
            // {
              inherit cargoArtifacts;
              cargoExtraArgs = "";
            });

          # Build all demos
          all-demos-check = allDemos;
        };

        apps.pre-commit = {
          type = "app";
          program = toString (pkgs.writeShellScript "pre-commit" ''
            set -e
            echo "Running pre-commit formatting checks..."
            echo ""
            echo "→ Nix formatting: ${self.checks.${system}.alejandra-check}"
            echo "→ TOML formatting: ${self.checks.${system}.taplo-fmt-check}"
            echo "→ Rust formatting: ${self.checks.${system}.cargo-fmt-check}"
            echo ""
            echo "✓ All formatting checks passed!"
          '');
        };

        apps.serve-docs = {
          type = "app";
          program = toString (pkgs.writeShellScript "serve-docs" ''
            set -e
            
            # Build documentation if not already built
            if [ ! -d "result" ]; then
              echo "Building documentation..."
              nix build .#documentation
            fi
            
            DOC_PATH="$(cd result && pwd)"
            PORT="''${1:-8000}"
            
            echo ""
            echo "📚 Serving rgeometry documentation"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "URL:  http://localhost:$PORT"
            echo "Docs: $DOC_PATH"
            echo ""
            
            ${pkgs.python3}/bin/python3 -m http.server --directory "$DOC_PATH" "$PORT"
          '');
        };
      }
    );
}
