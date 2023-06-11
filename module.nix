{ config, lib, pkgs, ... }:

with lib;
let cfg = config.services.disconic;

in {
  options.services.disconic = {
    enable = mkEnableOption "Disconic, a discord bot for subsonic music libraries";
    package = mkOption {
      type = types.package;
      default = pkgs.disconic;
      defaultText = "pkgs.disconic";
      description = ''
        The package implementing disconic
      '';
    };
    user = mkOption {
      type = types.str;
      default = "yrmos";
      description = "Service user that will run the daemon.";
    };

    subsonicUrl = mkOption {
      type = types.str;
      description = "Subsonic library base API URL";
    };
    subsonicUser = mkOption {
      type = types.str;
      description = "Subsonic user login";
    };
    subsonicPasswordFile = mkOption {
      type = types.path;
      description = "File path containing subsonic user password";
    };
    discordTokenFile = mkOption {
      type = types.path;
      description = "File path containing discord token";
    };
    discordGuild = mkOption {
      type = types.str;
      description = "Your server's guild ID, to auto-register commands";
    };
    extraArgs = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Extra arguments to pass to disconic.";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.disconic = {
      description = "Disconic, a Discord Subsonic Bot";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Restart = "always";
        User = cfg.user;
        ExecStart = lib.escapeShellArgs ([
          (lib.getExe cfg.package)
          "--subsonic-url=${cfg.subsonicUrl}"
          "--subsonic-user=${cfg.subsonicUser}"
          "--subsonic-password=$(cat ${cfg.subsonicPasswordFile})"
          "--discord-token=$(cat ${cfg.discordTokenFile})"
          "--discord-guild=${cfg.discordGuild}"
        ] ++ cfg.extraArgs);
      };
    };

    users = {
      users.${cfg.user} = {
        description = "disconic service user";
        isSystemUser = true;
        group = cfg.user;
      };
      groups.${cfg.user} = { };
    };
  };
}
