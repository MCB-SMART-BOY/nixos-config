{ lib, pkgs, ... }:

let
  inherit(import ./lib.nix { inherit lib; }) resolvePkg;

  graphicsRuntimeLibs = lib.unique ( lib.filter (x: x != null) [
    (resolvePkg [ "libglvnd" ] pkgs)
    (resolvePkg [ "vulkan-loader" ] pkgs)
    (resolvePkg [ "mesa" ] pkgs)
    (resolvePkg [ "libdrm" ] pkgs)
    (resolvePkg [ "glib" "out" ] pkgs)
    (resolvePkg [ "fontconfig" "lib" ] pkgs)
    (resolvePkg [ "freetype" ] pkgs)
    (resolvePkg [ "dbus" "lib" ] pkgs)
    (resolvePkg [ "wayland" ] pkgs)
    (resolvePkg [ "libxkbcommon" ] pkgs)
    (resolvePkg [ "libx11"] pkgs)
    (resolvePkg [ "libxext" ] pkgs)
    (resolvePkg [ "libxrender" ] pkgs)
    (resolvePkg [ "libxrandr" ] pkgs)
    (resolvePkg [ "libxi" ] pkgs)
    (resolvePkg [ "libxcursor" ] pkgs)
    (resolvePkg [ "libxinerama" ] pkgs)
    (resolvePkg [ "libfixes" ] pkgs)
    (resolvePkg [ "libxcb" ] pkgs)
  ]);

in {
  programs.nix-ld = {
    enable = true;
    libraries = graphicsRuntimeLibs;
  };
}
