# GPU topology abstraction: iGPU-only, hybrid PRIME, or dGPU-only.

{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.mcb.hardware.gpu;
  legacyNvidia = config.mcb.hardware.nvidia.enable;
  nvidiaEnabled = legacyNvidia || cfg.mode != "igpu";
  hybrid = cfg.mode == "hybrid";
  igpuVendor = cfg.igpuVendor;
  resolvePkg =
    path:
    let
      eval =
        if lib.hasAttrByPath path pkgs then
          builtins.tryEval (lib.getAttrFromPath path pkgs)
        else
          { success = false; value = null; };
    in
    if eval.success then eval.value else null;
  pkgs32 = if pkgs ? pkgsi686Linux then pkgs.pkgsi686Linux else null;
  resolvePkg32 =
    path:
    if pkgs32 == null then
      null
    else
      let
        eval =
          if lib.hasAttrByPath path pkgs32 then
            builtins.tryEval (lib.getAttrFromPath path pkgs32)
          else
            { success = false; value = null; };
      in
      if eval.success then eval.value else null;
  intelExtraPackages = lib.unique (lib.filter (x: x != null) [
    (resolvePkg [ "intel-media-driver" ])
    (resolvePkg [ "vaapiIntel" ])
    (resolvePkg [ "intel-vaapi-driver" ])
    (resolvePkg [ "libvdpau-va-gl" ])
  ]);
  intelExtraPackages32 = lib.unique (lib.filter (x: x != null) [
    (resolvePkg32 [ "vaapiIntel" ])
    (resolvePkg32 [ "intel-vaapi-driver" ])
    (resolvePkg32 [ "libvdpau-va-gl" ])
  ]);
  amdExtraPackages = lib.unique (lib.filter (x: x != null) [
    (resolvePkg [ "vaapiVdpau" ])
    (resolvePkg [ "libvdpau-va-gl" ])
  ]);
  hasIgpBus =
    (igpuVendor == "intel" && cfg.prime.intelBusId != null)
    || (igpuVendor == "amd" && cfg.prime.amdgpuBusId != null);
  hasNvidiaBus = cfg.prime.nvidiaBusId != null;
  primeBusIds =
    if igpuVendor == "intel" then
      { intelBusId = cfg.prime.intelBusId; }
    else
      { amdgpuBusId = cfg.prime.amdgpuBusId; };
  primeModeConfig =
    if cfg.prime.mode == "offload" then
      {
        offload = {
          enable = true;
          enableOffloadCmd = true;
        };
      }
    else if cfg.prime.mode == "sync" then
      {
        sync.enable = true;
      }
    else
      {
        reverseSync.enable = true;
      };
  nvidiaVideoDrivers =
    if hybrid then
      if igpuVendor == "amd" then
        [
          "nvidia"
          "amdgpu"
        ]
      else
        [
          "nvidia"
          "modesetting"
        ]
    else
      [ "nvidia" ];
in
{
  config = lib.mkMerge [
    {
      assertions = [
        {
          assertion = (!hybrid) || (hasNvidiaBus && hasIgpBus);
          message = "mcb.hardware.gpu.mode = \"hybrid\" requires mcb.hardware.gpu.prime.nvidiaBusId and the matching iGPU busId (intelBusId or amdgpuBusId).";
        }
        {
          assertion =
            !(cfg.specialisations.enable && lib.elem "hybrid" cfg.specialisations.modes)
            || (cfg.prime.nvidiaBusId != null && hasIgpBus);
          message = "mcb.hardware.gpu.specialisations includes \"hybrid\"; please set prime.nvidiaBusId and the matching iGPU busId.";
        }
      ];
      hardware.graphics = {
        enable = true;
        enable32Bit = true;
        extraPackages =
          (lib.optionals (igpuVendor == "intel") intelExtraPackages)
          ++ (lib.optionals (igpuVendor == "amd") amdExtraPackages);
        extraPackages32 = lib.optionals (igpuVendor == "intel") intelExtraPackages32;
      };
    }

    (lib.mkIf nvidiaEnabled {
      services.xserver.videoDrivers = nvidiaVideoDrivers;

      hardware.nvidia = {
        open = cfg.nvidia.open;
        modesetting.enable = true;
      };
    })

    (lib.mkIf hybrid {
      hardware.nvidia.prime = {
        nvidiaBusId = cfg.prime.nvidiaBusId;
      }
      // primeBusIds
      // primeModeConfig;
    })

    (lib.mkIf cfg.specialisations.enable {
      specialisation =
        let
          mkSpec = mode: {
            name = "gpu-${mode}";
            value = {
              configuration = {
                mcb.hardware.gpu.mode = mode;
              };
            };
          };
          modes = cfg.specialisations.modes;
        in
        builtins.listToAttrs (map mkSpec modes);
    })
  ];
}
