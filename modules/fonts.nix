{ pkgs, lib, ... }:

let
  hasManrope = builtins.hasAttr "manrope" pkgs;
  hasGoogleFonts = builtins.hasAttr "google-fonts" pkgs;
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
    ] ++ lib.optionals hasManrope [ pkgs.manrope ]
      ++ lib.optionals (!hasManrope && hasGoogleFonts) [ pkgs.google-fonts ];
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
