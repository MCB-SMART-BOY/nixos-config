# Shell 工具替换层：现代 CLI 与旧工具兼容设置。

{ ... }:

{
  programs.eza = {
    enable = true;
    enableFishIntegration = true;
    git = true;
    icons = "auto";
    colors = "auto";
    extraOptions = [
      "--group-directories-first"
      "--time-style=long-iso"
    ];
  };

  programs.bat = {
    enable = true;
    config = {
      paging = "never";
      style = "plain";
    };
  };

  programs.broot = {
    enable = true;
    enableFishIntegration = true;
    settings = {
      modal = true;
    };
  };
}
