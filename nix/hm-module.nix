# Home-manager module for FreeFlow speech-to-text
#
# Provides a systemd user service for autostart.
# Usage: imports = [ freeflow.homeManagerModules.default ];
#        services.freeflow.enable = true;
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.freeflow;
in
{
  options.services.freeflow = {
    enable = lib.mkEnableOption "FreeFlow speech-to-text user service";

    package = lib.mkOption {
      type = lib.types.package;
      defaultText = lib.literalExpression "freeflow.packages.\${system}.freeflow";
      description = "The FreeFlow package to use.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.freeflow = {
      Unit = {
        Description = "FreeFlow speech-to-text";
        After = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
      };
      Service = {
        ExecStart = "${cfg.package}/bin/freeflow";
        Restart = "on-failure";
        RestartSec = 5;
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };
  };
}
