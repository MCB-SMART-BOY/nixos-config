# admin 的 Git 配置（选项定义 + 程序配置 + 身份覆盖，自包含）。

{ config, lib, ... }:

let
  cfg = config.mcb.git;
  editor = "hx";
in
{
  options.mcb.git = {
    userName = lib.mkOption {
      type = lib.types.str;
      default = "MCB-SMART-BOY";
      description = "Git user.name for commits.";
    };
    userEmail = lib.mkOption {
      type = lib.types.str;
      default = "mcb2720838051@gmail.com";
      description = "Git user.email for commits.";
    };
  };

  config = {
    mcb.git.userName = lib.mkDefault "MCB-SMART-BOY";
    mcb.git.userEmail = lib.mkDefault "mcb2720838051@gmail.com";

    programs.git = {
      enable = true;
      lfs.enable = true;
      settings = {
        user = {
          name = cfg.userName;
          email = cfg.userEmail;
        };
        core = {
          editor = editor;
          pager = "delta";
        };
        interactive.diffFilter = "delta --color-only";
        delta = {
          navigate = true;
          "side-by-side" = true;
        };
      };
    };
  };
}
