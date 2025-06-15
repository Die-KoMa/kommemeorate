# SPDX-FileCopyrightText: 2025 Maximilian Marx
#
# SPDX-License-Identifier: EUPL-1.2

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
    toToml
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
      type = types.user;
    };

    group = mkOption {
      description = "group to run as";
      type = types.group;
    };
  };

  config =
    let
      cfg = config.die-koma.kommemeorate;

      configFile = pkgs.writeText "kommemeorate-config.toml" (toToml {
        inherit (cfg)
          storage
          database
          telegram
          matrix
          ;
      });
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
          };
        };

        tmpfiles.rules = [
          "d ${cfg.storage} 0755 ${cfg.user} ${cfg.group} - -"
        ];
      };
    };
}
