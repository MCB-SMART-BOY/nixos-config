{ pkgs, lib, ... }:

let
  manropeEval =
    if pkgs ? manrope then
      builtins.tryEval pkgs.manrope
    else
      { success = false; value = null; };
  googleFontsEval =
    if pkgs ? "google-fonts" then
      builtins.tryEval pkgs."google-fonts"
    else
      { success = false; value = null; };
in {
  fonts = {
    packages = with pkgs; [
      nerd-fonts.jetbrains-mono
      nerd-fonts.fira-code
      noto-fonts-cjk-sans
      noto-fonts-cjk-serif
      source-han-sans
      source-han-serif
      lxgw-wenkai
      font-awesome
      wqy_zenhei
      wqy_microhei
    ] ++ lib.optionals manropeEval.success [ manropeEval.value ]
      ++ lib.optionals (!manropeEval.success && googleFontsEval.success) [ googleFontsEval.value ];
    fontconfig = {
      defaultFonts = {
        monospace = [
          "JetBrainsMono Nerd Font"
          "LXGW WenKai Mono"
        ];
        sansSerif = [
          "Manrope"
          "Noto Sans CJK SC"
          "LXGW WenKai"
        ];
        serif = [
          "Noto Serif CJK SC"
          "Source Han Serif SC"
        ];
        emoji = [ "Noto Color Emoji" ];
      };
      antialias = true;
      hinting.enable = true;
    };
  };
}
