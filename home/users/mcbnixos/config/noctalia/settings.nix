# mcbnixos 专用 Noctalia 顶栏布局与按钮定义。
# 该文件只负责 settings attrset；由 ../../noctalia.nix 载入。

{
  scriptBin,
  lib,
  osConfig ? { },
}:

let
  gpuDefaultMode = lib.attrByPath [ "mcb" "hardware" "gpu" "mode" ] "igpu" osConfig;
  gpuIntelBusId = lib.attrByPath [ "mcb" "hardware" "gpu" "prime" "intelBusId" ] null osConfig;
  gpuAmdBusId = lib.attrByPath [ "mcb" "hardware" "gpu" "prime" "amdgpuBusId" ] null osConfig;
  gpuNvidiaBusId = lib.attrByPath [ "mcb" "hardware" "gpu" "prime" "nvidiaBusId" ] null osConfig;
  gpuHostTopology =
    if gpuNvidiaBusId != null && (gpuIntelBusId != null || gpuAmdBusId != null) then
      "multi-gpu"
    else if gpuNvidiaBusId != null || gpuDefaultMode == "dgpu" then
      "dgpu-only"
    else
      "igpu-only";
  gpuButtonEnabled = gpuHostTopology == "multi-gpu";
in

{
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
          leftClickExec = "${scriptBin}/niri-run alacritty -e fish -ic 'cd /etc/nixos 2>/dev/null; or cd \"$HOME/nixos-config\" 2>/dev/null; or cd \"$HOME\"; git status; echo; nix flake check --no-build; or true; echo; exec fish'";
          rightClickExec = "${scriptBin}/niri-run alacritty -e fish -ic 'cd /etc/nixos 2>/dev/null; or cd \"$HOME/nixos-config\" 2>/dev/null; or cd \"$HOME\"; echo \"Hint: run nix flake update && nrs\"; exec fish'";
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
          rightClickExec = "${scriptBin}/niri-run alacritty -e fish -ic 'sensors; or true; echo; exec fish'";
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
          rightClickExec = "${scriptBin}/niri-run alacritty -e fish -ic 'free -h; echo; vmstat -s 2>/dev/null | head -n 20; or true; echo; exec fish'";
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
          leftClickExec = "${scriptBin}/niri-run alacritty -e fish -ic 'watch -n 1 sensors'";
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
          rightClickExec = "${scriptBin}/niri-run alacritty -e fish -ic 'df -h; echo; lsblk; echo; exec fish'";
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
      ] ++ (if gpuButtonEnabled then [
        {
          id = "CustomButton";
          icon = "gpu";
          textCommand = "${scriptBin}/noctalia-gpu-mode";
          leftClickExec = "${scriptBin}/noctalia-gpu-mode --menu";
          rightClickExec = "${scriptBin}/noctalia-gpu-mode --session-note";
          leftClickUpdateText = true;
          parseJson = true;
          textIntervalMs = 5000;
          maxTextLength = {
            horizontal = 10;
            vertical = 10;
          };
        }
      ] else [ ]) ++ [
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
}
