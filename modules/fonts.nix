# 字体包与 fontconfig 默认字体设置。
# 统一中英文/等宽字体，避免应用字体不一致。

{ pkgs, lib, ... }:

let
  # 有些 nixpkgs 版本没有 manrope/google-fonts，这里做兼容
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
  emojiFontsEval =
    if pkgs ? "noto-fonts-color-emoji" then
      builtins.tryEval pkgs."noto-fonts-color-emoji"
    else if pkgs ? "noto-fonts-emoji" then
      builtins.tryEval pkgs."noto-fonts-emoji"
    else
      { success = false; value = null; };
in {
  fonts = {
    # 系统字体包（中英日韩 + 等宽 + 图标字体）
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
    ] ++ lib.optionals emojiFontsEval.success [ emojiFontsEval.value ]
      ++ lib.optionals manropeEval.success [ manropeEval.value ]
      ++ lib.optionals (!manropeEval.success && googleFontsEval.success) [ googleFontsEval.value ];
    fontconfig = {
      # 默认字体映射（应用层会自动使用）
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
