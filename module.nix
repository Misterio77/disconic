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
      type = types.string;
      description = "Subsonic library base API URL";
    };
    subsonicUser = mkOption {
      type = types.string;
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
  };

  config = mkIf cfg.enable {
    systemd.services.disconic = {
      description = "Disconic, a Discord Subsonic Bot";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/disconic";
        Restart = "on-failure";
        Environment = [
          "SUBSONIC_URL=${cfg.subsonicUrl}"
          "SUBSONIC_USER=${cfg.subsonicUser}"
          "SUBSONIC_PASSWORD_FILE=${cfg.subsonicPasswordFile}"
          "DISCORD_TOKEN_FILE=${cfg.discordTokenFile}"
        ];
      };
    };
  };
}
