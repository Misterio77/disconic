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
      serviceConfig.ExecStart = lib.escapeShellArgs ([
        (lib.getExe cfg.package)
        "--subsonic-url=${cfg.subsonicUrl}"
        "--subsonic-user=${cfg.subsonicUser}"
        "--subsonic-password=$(cat ${cfg.subsonicPasswordFile})"
        "--discord-token=$(cat ${cfg.discordTokenFile}"
      ] ++ cfg.extraArgs);
    };
  };
}
