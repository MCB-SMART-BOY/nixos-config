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

  # 覆盖桌面入口：让 wemeet 启动 XWayland 版本
  xdg.desktopEntries.wemeet = {
    name = "Tencent Meeting";
    genericName = "Video Conference";
    comment = "Tencent Meeting (XWayland)";
    exec = "wemeet-xwayland %U";
    icon = "wemeet";
    terminal = false;
    categories = [
      "Network"
      "VideoConference"
      "AudioVideo"
    ];
  };

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
          {
            id = "CustomButton";
            icon = "git-branch";
            textCommand = "${homeDir}/.local/bin/noctalia-flake-updates";
            parseJson = true;
            textIntervalMs = 900000;
            maxTextLength = {
              horizontal = 6;
              vertical = 6;
            };
          }
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
            icon = "transfer";
            textCommand = "${homeDir}/.local/bin/noctalia-net-speed";
            parseJson = true;
            textIntervalMs = 2000;
            maxTextLength = {
              horizontal = 20;
              vertical = 20;
            };
          }
          {
            id = "CustomButton";
            icon = "bluetooth";
            textCommand = "${homeDir}/.local/bin/noctalia-bluetooth";
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
            icon = "cpu";
            textCommand = "${homeDir}/.local/bin/noctalia-cpu";
            parseJson = true;
            textIntervalMs = 2000;
            maxTextLength = {
              horizontal = 6;
              vertical = 6;
            };
          }
          {
            id = "CustomButton";
            icon = "database";
            textCommand = "${homeDir}/.local/bin/noctalia-memory";
            parseJson = true;
            textIntervalMs = 2000;
            maxTextLength = {
              horizontal = 6;
              vertical = 6;
            };
          }
          {
            id = "CustomButton";
            icon = "thermometer";
            textCommand = "${homeDir}/.local/bin/noctalia-temperature";
            parseJson = true;
            textIntervalMs = 5000;
            maxTextLength = {
              horizontal = 6;
              vertical = 6;
            };
          }
          {
            id = "CustomButton";
            icon = "hard-drive";
            textCommand = "${homeDir}/.local/bin/noctalia-disk";
            parseJson = true;
            textIntervalMs = 60000;
            maxTextLength = {
              horizontal = 6;
              vertical = 6;
            };
          }
          {
            id = "CustomButton";
            icon = "cpu";
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
