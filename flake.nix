# SPDX-FileCopyrightText: 2025 Maximilian Marx
#
# SPDX-License-Identifier: EUPL-1.2

{
  description = "collects and structures your memes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane.url = "github:ipetkov/crane";

    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pre-commit-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        gitignore.follows = "gitignore";
      };
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    inputs@{
      self,
      utils,
      rust-overlay,
      ...
    }:
    utils.lib.mkFlake {
      inherit self inputs;

      channels.nixpkgs.overlaysBuilder = _channels: [
        rust-overlay.overlays.default
      ];

      overlays = {
        rust-overlay = rust-overlay.overlays.default;

        default =
          final: _prev:
          let
            pkgs = self.packages."${final.system}";
          in
          {
            inherit (pkgs) kommemeorate;
          };
      };

      nixosModules = {
        kommemeorate = import ./nix/module.nix;
      };

      outputsBuilder =
        channels:
        let
          pkgs = channels.nixpkgs;
          inherit (pkgs) lib system;

          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

          crane = (inputs.crane.mkLib pkgs).overrideToolchain toolchain;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (crane.fileset.commonCargoSources ./.)
              (lib.fileset.fileFilter (file: file.hasExt "sql") ./.)
            ];
          };

          commonArgs = {
            inherit src;
            strictDeps = true;

            nativeBuildInputs = [ pkgs.pkg-config ];

            buildInputs =
              [
                pkgs.openssl
                pkgs.sqlite.dev
                pkgs.postgresql.dev
                pkgs.installShellFiles
              ]
              ++ lib.optionals pkgs.stdenv.isDarwin [
                pkgs.libiconv
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
          };

          cargoArtifacts = crane.buildDepsOnly commonArgs;

          individualCrateArgs = commonArgs // {
            inherit cargoArtifacts;
            inherit (crane.crateNameFromCargoToml { inherit src; }) version;
            doCheck = false;
          };

          fileSetForCrate =
            crate:
            lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./build.rs
                ./migrations
                crate
              ];
            };

          cargoMeta = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;

          kommemeorate = crane.buildPackage (
            individualCrateArgs
            // {
              pname = "kommemeorate";
              cargoExtraArgs = "-p kommemeorate";
              src = fileSetForCrate ./src;

              MAN_OUT = "/build";

              preInstall = ''
                installManPage $MAN_OUT/kommemeorate.1
                installShellCompletion \
                  --fish $MAN_OUT/kommemeorate.fish \
                  --bash $MAN_OUT/kommemeorate.bash \
                  --zsh  $MAN_OUT/_kommemeorate
                mkdir -p $out
              '';

              meta = {
                inherit (cargoMeta) description homepage;
                license = lib.licenses.eupl12;
                mainProgram = "kommemeorate";
              };
            }
          );

          treefmtConfig = {
            projectRootFile = "flake.nix";

            programs = {
              # nix
              nixfmt.enable = true;
              statix.enable = true;
              deadnix.enable = true;

              # rust
              rustfmt = {
                enable = true;
                package = toolchain;
              };
              taplo.enable = true;

              shellcheck.enable = true;
            };

            settings = {
              formatter = {
                shellcheck.excludes = [ ".envrc" ];
              };
            };
          };

          treefmt = inputs.treefmt-nix.lib.evalModule pkgs treefmtConfig;
        in
        {

          packages = {
            inherit kommemeorate;
            default = kommemeorate;
          };

          checks = {
            inherit kommemeorate;

            cargo-workspace-clippy = crane.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );

            cargo-workspace-doc = crane.cargoDoc (commonArgs // { inherit cargoArtifacts; });
            cargo-workspace-fmt = crane.cargoFmt { inherit src; };
            cargo-workspace-audit = crane.cargoAudit {
              inherit src;
              inherit (inputs) advisory-db;
            };
            cargo-workspace-deny = crane.cargoDeny { inherit src; };
            cargo-workspace-nextest = crane.cargoNextest (
              commonArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
              }
            );

            pre-commit-check =
              let
                replaceFormatters = {
                  nixfmt = "nixfmt-rfc-style";
                };
                treefmtFormatters = lib.mapAttrs' (
                  key: value: lib.nameValuePair (replaceFormatters.${key} or key) value
                ) treefmtConfig.programs;
              in
              inputs.pre-commit-hooks.lib.${system}.run {
                src = ./.;
                hooks = treefmtFormatters // {
                  convco.enable = true;
                  reuse = {
                    enable = true;
                    name = "reuse";
                    entry = with pkgs; "${reuse}/bin/reuse lint";
                    pass_filenames = false;
                  };
                  rustfmt = {
                    enable = true;
                    packageOverrides = {
                      cargo = toolchain;
                      rustfmt = toolchain;
                    };
                  };
                  check-merge-conflicts.enable = true;
                  end-of-file-fixer.enable = true;
                  fix-byte-order-marker.enable = true;
                  editorconfig-checker = {
                    enable = true;
                    excludes = [ ''^LICENSES/.*\.txt$'' ];
                  };
                  shellcheck = {
                    enable = true;
                    excludes = [ "\\.envrc" ];
                  };
                };
              };

            formatting = treefmt.config.build.check self;
          };

          apps = {
            changelog = utils.lib.mkApp {
              drv = pkgs.writeShellApplication {
                name = "changelog";

                runtimeInputs = lib.attrValues {
                  inherit (pkgs)
                    git
                    git-cliff
                    ;
                };

                text = ''
                  git cliff -c ./cliff.toml;
                '';
              };
            };
          };

          devShells.default = crane.devShell {
            checks = self.checks.${system};

            RUST_LOG = "warn,kommemeorate=debug";
            RUST_BACKTRACE = 1;
            DATABASE_URL = "postgres://kommemeorate@/kommemeorate";

            packages = lib.attrValues {
              inherit toolchain;
              inherit (pkgs)
                cargo-license
                cargo-audit
                cargo-update
                reuse
                commitizen
                rust-analyzer
                diesel-cli
                ;
            };

            inherit (self.checks.${system}.pre-commit-check) shellHook;
          };

          formatter = treefmt.config.build.wrapper;
        };
    };
}
