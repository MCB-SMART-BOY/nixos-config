# 核心服务：SSH、nix-ld 兼容运行时。

{
  config,
  pkgs,
  lib,
  ...
}:

let
  resolvePkg =
    path:
    let
      eval =
        if lib.hasAttrByPath path pkgs then
          builtins.tryEval (lib.getAttrFromPath path pkgs)
        else
          {
            success = false;
            value = null;
          };
    in
    if eval.success then eval.value else null;

  graphicsRuntimeLibs = lib.unique (
    lib.filter (x: x != null) [
      (resolvePkg [ "libglvnd" ])
      (resolvePkg [ "vulkan-loader" ])
      (resolvePkg [ "mesa" ])
      (resolvePkg [ "libdrm" ])
      (resolvePkg [
        "glib"
        "out"
      ])
      (resolvePkg [
        "fontconfig"
        "lib"
      ])
      (resolvePkg [ "freetype" ])
      (resolvePkg [
        "dbus"
        "lib"
      ])
      (resolvePkg [ "wayland" ])
      (resolvePkg [ "libxkbcommon" ])
      (resolvePkg [
        "xorg"
        "libX11"
      ])
      (resolvePkg [
        "xorg"
        "libXext"
      ])
      (resolvePkg [
        "xorg"
        "libXrender"
      ])
      (resolvePkg [
        "xorg"
        "libXrandr"
      ])
      (resolvePkg [
        "xorg"
        "libXi"
      ])
      (resolvePkg [
        "xorg"
        "libXcursor"
      ])
      (resolvePkg [
        "xorg"
        "libXinerama"
      ])
      (resolvePkg [
        "xorg"
        "libXfixes"
      ])
      (resolvePkg [ "libxcb" ])
    ]
  );
in
{
  services.openssh.enable = true;

  # nix-ld：让外部二进制（AppImage/upstream tarball）找到图形运行时依赖
  programs.nix-ld = lib.mkIf (config.mcb.hostRole == "desktop") {
    enable = true;
    libraries = lib.mkAfter graphicsRuntimeLibs;
  };
}
