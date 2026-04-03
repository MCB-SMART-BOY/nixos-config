{
  lib,
  ...
}:

{
  options.mcb = {
    desktopEntries = {
      enableZed = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Enable Zed desktop entry override for this user.";
      };

      enableYesPlayMusic = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Enable YesPlayMusic desktop entry override for this user.";
      };
    };

    noctalia = {
      barProfile = lib.mkOption {
        type = lib.types.enum [
          "default"
          "none"
        ];
        default = "default";
        description = "Noctalia bar profile: default (built-in widgets) or none (disable managed bar settings).";
      };
    };
  };
}
