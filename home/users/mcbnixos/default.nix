# 用户入口（mcbnixos）：选择 profile + 用户级文件。

{ ... }:

let
  user = "mcbnixos";
  homeDir = "/home/${user}";
in
{
  # 该用户启用完整桌面 profile
  imports = [
    ../../profiles/full.nix
    ./git.nix
    ./files.nix
    ./scripts.nix
  ];

  # Home Manager 基本信息
  home.username = user;
  home.homeDirectory = homeDir;
  home.stateVersion = "25.11";

  # 启用 Home Manager 管理自身
  programs.home-manager.enable = true;

  # 启用 XDG 规范目录结构
  xdg.enable = true;

  programs.noctalia-shell.settings = {
    bar = {
      widgets = {
        left = [
          { id = "Launcher"; }
          { id = "Workspace"; }
        ];
        center = [
          { id = "Clock"; }
        ];
        right = [
          { id = "Tray"; }
          {
            id = "CustomButton";
            icon = "wifi";
            textCommand = "${homeDir}/.local/bin/noctalia-net-status";
            parseJson = true;
            textIntervalMs = 5000;
            maxTextLength = {
              horizontal = 8;
              vertical = 8;
            };
          }
          {
            id = "CustomButton";
            icon = "bluetooth";
            textCommand = "${homeDir}/.local/bin/noctalia-bluetooth";
            leftClickExec = "${homeDir}/.local/bin/niri-run blueman-manager";
            parseJson = true;
            textIntervalMs = 5000;
            maxTextLength = {
              horizontal = 4;
              vertical = 4;
            };
          }
          { id = "Volume"; }
          { id = "Brightness"; }
          { id = "Battery"; }
          {
            id = "CustomButton";
            icon = "gpu";
            textCommand = "${homeDir}/.local/bin/noctalia-gpu-mode";
            leftClickExec = "${homeDir}/.local/bin/noctalia-gpu-mode --menu";
            leftClickUpdateText = true;
            parseJson = true;
            textIntervalMs = 5000;
            maxTextLength = {
              horizontal = 10;
              vertical = 10;
            };
          }
          {
            id = "CustomButton";
            icon = "shield";
            textCommand = "${homeDir}/.local/bin/noctalia-proxy-status";
            parseJson = true;
            textIntervalMs = 5000;
            maxTextLength = {
              horizontal = 6;
              vertical = 6;
            };
          }
          {
            id = "CustomButton";
            icon = "power";
            textCommand = "${homeDir}/.local/bin/noctalia-power";
            leftClickExec = "niri msg action quit";
            parseJson = true;
            textIntervalMs = 60000;
          }
          { id = "NotificationHistory"; }
          { id = "ControlCenter"; }
        ];
      };
    };
  };
}
