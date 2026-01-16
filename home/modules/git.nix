{ config, lib, ... }:

let
  heavy = config.mcb.packages.enableHeavyBuilds;
  editor = if heavy then "hx" else "nvim";
in
{
  programs.git = {
    enable = true;
    lfs.enable = true;
    settings = lib.mkMerge [
      {
        user = {
          name = "MCB-SMART-BOY";
          email = "mcb2720838051@gmail.com";
        };
        core = {
          editor = editor;
        };
      }
      (lib.mkIf heavy {
        core.pager = "delta";
        interactive.diffFilter = "delta --color-only";
        delta = {
          navigate = true;
          "side-by-side" = true;
        };
      })
    ];
  };
}
