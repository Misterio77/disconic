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

    environmentFile = mkOption {
      type = types.path;
      default = null;
      description = "File path containing environment variables.";
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
        EnvironmentFile = lib.mkIf (cfg.environmentFile != null) cfg.environmentFile;
      };
      script = (lib.getExe cfg.package) + (lib.escapeShellArg cfg.extraArgs);
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
