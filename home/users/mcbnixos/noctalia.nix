# mcbnixos 的 Noctalia 个性化栏配置（高级按钮版）。
# 说明：
# 1) 该文件只对 mcbnixos 生效。
# 2) 通过 mcb.noctalia.barProfile = "none" 关闭模块默认栏，完全使用本文件定义。

{ config, ... }:

let
  scriptBin = "${config.home.homeDirectory}/.local/bin";
in
{
  mcb.noctalia.barProfile = "none";

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
            icon = "git-branch";
            textCommand = "${scriptBin}/noctalia-flake-updates";
            leftClickExec = "${scriptBin}/niri-run alacritty -e bash -lc 'cd /etc/nixos 2>/dev/null || cd \"$HOME/nixos-config\" 2>/dev/null || cd \"$HOME\"; git status; echo; nix flake check --no-build || true; echo; exec bash'";
            rightClickExec = "${scriptBin}/niri-run alacritty -e bash -lc 'cd /etc/nixos 2>/dev/null || cd \"$HOME/nixos-config\" 2>/dev/null || cd \"$HOME\"; echo \"Hint: run nix flake update && nrs\"; exec bash'";
            parseJson = true;
            textIntervalMs = 900000;
            maxTextLength = {
              horizontal = 4;
              vertical = 4;
            };
          }
          {
            id = "CustomButton";
            icon = "wifi";
            textCommand = "${scriptBin}/noctalia-net-status";
            leftClickExec = "${scriptBin}/niri-run alacritty -e nmtui";
            rightClickExec = "${scriptBin}/niri-run clash-nyanpasu";
            parseJson = true;
            textIntervalMs = 5000;
            maxTextLength = {
              horizontal = 8;
              vertical = 8;
            };
          }
          {
            id = "CustomButton";
            icon = "activity";
            textCommand = "${scriptBin}/noctalia-net-speed";
            leftClickExec = "${scriptBin}/niri-run alacritty -e btop";
            rightClickExec = "${scriptBin}/niri-run alacritty -e nmtui";
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
            textCommand = "${scriptBin}/noctalia-bluetooth";
            leftClickExec = "${scriptBin}/niri-run blueman-manager";
            rightClickExec = "${scriptBin}/niri-run blueman-adapters";
            parseJson = true;
            textIntervalMs = 5000;
            maxTextLength = {
              horizontal = 4;
              vertical = 4;
            };
          }
          {
            id = "CustomButton";
            icon = "cpu";
            textCommand = "${scriptBin}/noctalia-cpu";
            leftClickExec = "${scriptBin}/niri-run alacritty -e btop";
            rightClickExec = "${scriptBin}/niri-run alacritty -e bash -lc 'sensors || true; echo; exec bash'";
            parseJson = true;
            textIntervalMs = 2000;
            maxTextLength = {
              horizontal = 6;
              vertical = 6;
            };
          }
          {
            id = "CustomButton";
            icon = "memory-stick";
            textCommand = "${scriptBin}/noctalia-memory";
            leftClickExec = "${scriptBin}/niri-run alacritty -e btop";
            rightClickExec = "${scriptBin}/niri-run alacritty -e bash -lc 'free -h; echo; vmstat -s 2>/dev/null | head -n 20 || true; echo; exec bash'";
            parseJson = true;
            textIntervalMs = 3000;
            maxTextLength = {
              horizontal = 6;
              vertical = 6;
            };
          }
          {
            id = "CustomButton";
            icon = "thermometer";
            textCommand = "${scriptBin}/noctalia-temperature";
            leftClickExec = "${scriptBin}/niri-run alacritty -e bash -lc 'watch -n 1 sensors'";
            rightClickExec = "${scriptBin}/niri-run alacritty -e btop";
            parseJson = true;
            textIntervalMs = 5000;
            maxTextLength = {
              horizontal = 7;
              vertical = 7;
            };
          }
          {
            id = "CustomButton";
            icon = "hard-drive";
            textCommand = "${scriptBin}/noctalia-disk";
            leftClickExec = "${scriptBin}/niri-run baobab";
            rightClickExec = "${scriptBin}/niri-run alacritty -e bash -lc 'df -h; echo; lsblk; echo; exec bash'";
            parseJson = true;
            textIntervalMs = 30000;
            maxTextLength = {
              horizontal = 7;
              vertical = 7;
            };
          }
          { id = "Volume"; }
          { id = "Brightness"; }
          { id = "Battery"; }
          {
            id = "CustomButton";
            icon = "gpu";
            textCommand = "${scriptBin}/noctalia-gpu-mode";
            leftClickExec = "${scriptBin}/noctalia-gpu-mode --menu";
            rightClickExec = "${scriptBin}/niri-run alacritty -e ${scriptBin}/noctalia-gpu-mode --menu-cli";
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
            textCommand = "${scriptBin}/noctalia-proxy-status";
            leftClickExec = "${scriptBin}/niri-run clash-verge";
            rightClickExec = "${scriptBin}/niri-run metacubexd";
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
            textCommand = "${scriptBin}/noctalia-power";
            leftClickExec = "niri msg action quit";
            rightClickExec = "${scriptBin}/lock-screen";
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
