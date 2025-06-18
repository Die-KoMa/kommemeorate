# SPDX-FileCopyrightText: 2025 Maximilian Marx
#
# SPDX-License-Identifier: EUPL-1.2

{ std, ... }:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  inherit (lib)
    mkEnableOption
    mkIf
    mkOption
    types
    ;
in
{
  options.die-koma.kommemeorate = {
    enable = mkEnableOption "collects and structures your memes";

    telegram = mkOption {
      type = types.submodule {
        options = {
          apiIdFile = mkOption {
            description = "File containing the kommemeomorate Telegram API id";
            type = types.path;
          };

          apiHashFile = mkOption {
            description = "File containing the kommemeomorate Telegram API hash";
            type = types.path;
          };

          passwordFile = mkOption {
            description = "File containing the kommemeomorate Telegram bot password";
            type = types.path;
          };

          groups = mkOption {
            description = "groups to collect memes from";
            type = types.listOf (
              types.submodule {
                options = {
                  name = mkOption {
                    description = "name to identify group";
                    type = types.str;
                  };

                  id = mkOption {
                    description = "Telegram group id";
                    type = types.number;
                  };
                };
              }
            );
          };
        };
      };
    };

    matrix = mkOption {
      type = types.submodule {
        options = {
          homeserver = mkOption {
            description = "Matrix homeserver";
            default = "die-koma.org";
            type = types.str;
          };

          username = mkOption {
            description = "Matrix bot username";
            type = types.str;
          };

          passwordFile = mkOption {
            description = "File containing the kommemeomorate Matrix bot password";
            type = types.path;
          };

          rooms = mkOption {
            description = "Rooms to collect memes from";
            type = types.listOf (
              types.submodule {
                name = mkOption {
                  description = "name to identify Room";
                  type = types.str;
                };

                address = mkOption {
                  description = "Matrix room address";
                  type = types.str;
                };
              }
            );
          };
        };
      };
    };

    database = mkOption {
      type = types.submodule {
        options = {
          url = mkOption {
            description = "PostgreSQL database connection URL";
            type = types.str;
          };
        };
      };
    };

    storage = mkOption {
      type = types.submodule {
        options = {
          path = mkOption {
            description = "where to store the memes";
            type = types.path;
          };
        };
      };
    };

    user = mkOption {
      description = "user to run as";
      type = types.str;
    };

    group = mkOption {
      description = "group to run as";
      type = types.str;
    };
  };

  config =
    let
      cfg = config.die-koma.kommemeorate;

      configFile = pkgs.writeText "kommemeorate-config.toml" (
        std.serde.toTOML {
          inherit (cfg)
            storage
            database
            telegram
            matrix
            ;
        }
      );
    in
    mkIf cfg.enable {

      systemd = {
        services.kommemeorate = {
          after = [
            "network.target"
            "matrix-synapse.service"
          ];
          reloadTriggers = [ ];
          serviceConfig = {
            User = cfg.user;
            Group = cfg.group;
            ExecStart = "${lib.getExe pkgs.kommemeorate} --config ${configFile}";
            Type = "simple";
            Environment = "RUST_LOG=debug";
          };
        };

        tmpfiles.rules = [
          "d ${cfg.storage.path} 0755 ${cfg.user} ${cfg.group} - -"
        ];
      };
    };
}
