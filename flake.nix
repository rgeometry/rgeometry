{
  description = "rgeometry flake with 'earclip' demo packaged as a single HTML output";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    alejandra.url = "github:kamadorueda/alejandra";
    statix.url = "github:oppiliappan/statix";
    deadnix.url = "github:astro/deadnix";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    alejandra,
    statix,
    deadnix,
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
            # Root level Rust files and workspace
            ./Cargo.toml
            ./Cargo.lock
            ./src
            ./benches
            ./tests
            # Configuration files
            ./rust-toolchain.toml
            ./rustfmt.toml
            ./taplo.toml
            # Metadata
            ./README.md
            ./LICENSE
            # rgeometry-wasm workspace member
            ./rgeometry-wasm/Cargo.toml
            ./rgeometry-wasm/src
            ./rgeometry-wasm/rustfmt.toml
            # Demo workspace members
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

        # Build dependencies only (for caching) - native target for tests/clippy
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build entire workspace for wasm32-unknown-unknown target
        # Create a separate build without --all-features which is incompatible with wasm32
        wasmBuild = craneLib.buildPackage {
          inherit src;
          pname = "rgeometry-wasm32";
          # Don't use cargoArtifacts since wasm32 has different feature requirements
          # cargoArtifacts = null; # This would skip the deps build
          cargoExtraArgs = "--workspace --target wasm32-unknown-unknown";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          # Try to be lenient with dependencies for wasm32
          strictDeps = true;
          # Preserve dependencies but don't install binaries
          doNotPostBuildInstallCargoBinaries = true;
          installPhaseCommand = "mkdir -p $out && cp -r target/wasm32-unknown-unknown/release $out/";
          nativeBuildInputs = with pkgs; [
            pkg-config
            m4
          ];
          buildInputs = with pkgs; [
            gmp
            mpfr
          ];
          GMP_MPFR_SYS_CACHE = "no-test";
        };

        # Demo builder - extracts WASM files from the wasm32 build
        mkDemo = demoName:
          pkgs.runCommand "rgeometry-demo-${demoName}" {
            nativeBuildInputs = with pkgs; [
              pkgs.wasm-bindgen-cli
              binaryen
            ];
          } ''
            mkdir -p $out
            cd ${wasmBuild}/release
            
            # Find and process the wasm binary for this demo
            if [ -f "${demoName}.wasm" ]; then
              mkdir -p pkg
              wasm-bindgen --target no-modules --out-dir pkg --out-name ${demoName} \
                ${demoName}.wasm
              wasm-opt -Oz -o pkg/${demoName}_bg.wasm pkg/${demoName}_bg.wasm
              
              # Generate HTML with the merge script from source
              bash ${src}/utils/merge.sh -o $out/${demoName}.html \
                pkg/${demoName}_bg.wasm pkg/${demoName}.js
            else
              echo "Error: ${demoName}.wasm not found in wasm build"
              echo "Available WASM files:"
              ls -la *.wasm 2>/dev/null || echo "No WASM files found"
              exit 1
            fi
          '';

        # Reference for wasm-bindgen-cli version check (kept for documentation)
        # wasmBindgenCliVersion = "0.2.100";
        inherit (pkgs) lib;
        demosDir = builtins.readDir ./demos;
        # Get all demo directories
        allDemoDirs = lib.attrNames (lib.filterAttrs (_name: kind: kind == "directory") demosDir);
        # Check that all demos have Cargo.lock files and fail if any are missing
        demoNames =
          let
            demosWithoutLock = builtins.filter (name: !builtins.pathExists (./. + "/demos/${name}/Cargo.lock")) allDemoDirs;
          in
            assert demosWithoutLock == [] || builtins.throw "The following demos are missing Cargo.lock files: ${builtins.toString demosWithoutLock}";
            allDemoDirs;
        allDemos = pkgs.symlinkJoin {
          name = "rgeometry-demos";
          paths = builtins.map mkDemo demoNames;
        };

        # Generate code coverage report with grcov
        coverage = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            pname = "rgeometry-coverage";
            cargoExtraArgs = "--all-features --workspace";
            nativeBuildInputs = commonArgs.nativeBuildInputs ++ [pkgs.grcov];
            CARGO_INCREMENTAL = "0";
            RUSTFLAGS = "-Cinstrument-coverage";
            LLVM_PROFILE_FILE = "rgeometry-%p-%m.profraw";
            buildPhaseCargoCommand = ''
              cargo test --all-features --workspace
              cargo test --all-features --workspace --doc
              mkdir -p $out/html
              grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o $out/html
              grcov . --binary-path ./target/debug/ -s . -t lcov --branch --ignore-not-existing -o $out/lcov.info
            '';
            doCheck = false;
            doNotPostBuildInstallCargoBinaries = true;
            installPhaseCommand = "echo 'Coverage report generated'";
          });

        # Extract uncovered code snippets from coverage report
        uncoveredSnippets = pkgs.runCommand "rgeometry-uncovered-snippets" {
          nativeBuildInputs = [pkgs.python3];
          preferLocalBuild = true;
        } ''
          mkdir -p $out
          ${pkgs.python3}/bin/python3 ${./nix/extract-uncovered-snippets.py} \
            ${coverage}/lcov.info \
            $out/uncovered-snippets.md \
            ${src}
          echo "✓ Uncovered snippets report generated"
        '';

        # Build documentation with rustdoc
        documentation = (craneLib.cargoDoc (commonArgs
          // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "--html-in-header ${./doc-header.html}";
          })).overrideAttrs (_: {
          # After building docs, include demo HTML files and compute checksum
          postInstall = ''
            # Copy demo HTML files if available
            ${pkgs.bash}/bin/bash -c 'cp -v ${allDemos}/*.html $out/ 2>/dev/null || true'
            # Create redirect index.html at root
            cat > $out/index.html <<'EOF'
             <!DOCTYPE html>
            <html>
            <head>
              <meta charset="utf-8">
              <meta http-equiv="refresh" content="0; url=./share/doc/rgeometry/">
              <title>RGeometry Documentation</title>
            </head>
            <body>
              <p>Redirecting to <a href="./share/doc/rgeometry/">RGeometry API documentation</a>...</p>
            </body>
            </html>
            EOF
            # Compute checksum of all documentation files
            CHECKSUM=$(find $out -type f -exec md5sum {} \; | sort | md5sum | cut -d' ' -f1)
            echo "$CHECKSUM" > $out/rgeometry.checksum
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
            inherit coverage documentation;
            uncovered-snippets = uncoveredSnippets;
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

          # Check Nix code with statix
          statix-check = pkgs.runCommand "statix-check" {} ''
            ${statix.packages.${system}.default}/bin/statix check ${./.}
            touch $out
          '';

          # Check for dead Nix code with deadnix
          deadnix-check = pkgs.runCommand "deadnix-check" {} ''
            ${deadnix.packages.${system}.default}/bin/deadnix --fail ${./.}
            touch $out
          '';

          # Check Python code with Ruff
          ruff-check = pkgs.runCommand "ruff-check"
            {
              # Disable cache to avoid permission issues in Nix store
              RUFF_CACHE_DIR = "/tmp/ruff-cache";
            } ''
            ${pkgs.ruff}/bin/ruff check --no-cache ${./.}
            ${pkgs.ruff}/bin/ruff format --no-cache --check ${./.}
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

          # Verify code coverage can be generated
          rgeometry-coverage = craneLib.buildPackage (commonArgs
            // {
              inherit cargoArtifacts;
              pname = "rgeometry-coverage-check";
              cargoExtraArgs = "--all-features --workspace";
              nativeBuildInputs = commonArgs.nativeBuildInputs ++ [pkgs.grcov];
              CARGO_INCREMENTAL = "0";
              RUSTFLAGS = "-Cinstrument-coverage";
              LLVM_PROFILE_FILE = "rgeometry-%p-%m.profraw";
               buildPhaseCargoCommand = ''
                 cargo test --all-features --workspace
                 cargo test --all-features --workspace --doc
                 grcov . --binary-path ./target/debug/ -s . -t lcov --branch --ignore-not-existing -o coverage.lcov
                 echo "Coverage report generated successfully"
               '';
              doCheck = false;
              doNotPostBuildInstallCargoBinaries = true;
              installPhaseCommand = ''
                mkdir -p $out
                cp coverage.lcov $out/ 2>/dev/null || echo "Coverage file generated"
                touch $out/coverage-check-passed
              '';
            });

          # Build documentation
          documentation-check = documentation;
        };

        apps = {
          pre-commit = {
            type = "app";
            program = toString (pkgs.writeShellScript "pre-commit" ''
              set -e
              echo "Running pre-commit formatting checks..."
              echo ""
              echo "→ Nix formatting: ${self.checks.${system}.alejandra-check}"
              echo "→ TOML formatting: ${self.checks.${system}.taplo-fmt-check}"
              echo "→ Rust formatting: ${self.checks.${system}.cargo-fmt-check}"
              echo "→ Python linting: ${self.checks.${system}.ruff-check}"
              echo ""
              echo "✓ All checks passed!"
            '');
            meta = {
              description = "Run pre-commit formatting checks";
            };
          };

          serve-docs = {
            type = "app";
            program = toString (pkgs.writeShellScript "serve-docs" ''
              set -e

              DOC_PATH="${documentation}"
              PORT="''${1:-8000}"

              echo ""
              echo "📚 Serving rgeometry documentation"
              echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
              echo "URL:  http://localhost:$PORT"
              echo "Docs: $DOC_PATH"
              echo ""

              ${pkgs.python3}/bin/python3 -m http.server --directory "$DOC_PATH" "$PORT"
            '');
            meta = {
              description = "Serve rgeometry documentation on a local web server";
            };
          };

          serve-coverage = {
            type = "app";
            program = toString (pkgs.writeShellScript "serve-coverage" ''
              set -e

              COVERAGE_PATH="${coverage}/html"
              PORT="''${1:-8080}"

              echo ""
              echo "📊 Serving rgeometry code coverage report"
              echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
              echo "URL:      http://localhost:$PORT"
              echo "Coverage: $COVERAGE_PATH"
              echo ""

              ${pkgs.python3}/bin/python3 -m http.server --directory "$COVERAGE_PATH" "$PORT"
            '');
            meta = {
              description = "Serve rgeometry code coverage report on a local web server";
            };
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [
            (craneLib.buildPackage commonArgs)
          ];
          packages = with pkgs; [
            rustToolchain
            pkg-config
            m4
            gmp
            mpfr
            # Additional tools for development
            wasm-pack
            wasm-bindgen-cli
            binaryen
          ];
          shellHook = ''
            echo "$(cargo --version)"
          '';
          GMP_MPFR_SYS_CACHE = "no-test";
        };
      }
    );
}
