# 用户最小 profile：适合服务器/SSH 环境。

{ ... }:

{
  # 轻量 profile：仅保留基础 shell 与 git
  imports = [
    ../modules/base.nix
    ../modules/shell.nix
    ../modules/git.nix
  ];

  home.sessionVariables = {
    # 服务器环境使用基础编辑器
    EDITOR = "vi";
    VISUAL = "vi";
    TERMINAL = "bash";
  };
}
