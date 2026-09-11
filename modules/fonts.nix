{ pkgs, ... }:

{
  fonts = {
    packages = with pkgs; [
      maple-mono.NF-CN
      nerd-fonts.jetbrains-mono nerd-fonts.fira-code
      sarasa-gothic
      noto-fonts-cjk-sans noto-fonts-cjk-serif
      source-han-sans source-han-serif
      lxgw-wenkai font-awesome
      noto-fonts-color-emoji
      wqy_zenhei wqy_microhei
      google-fonts
    ];

    fontconfig = {
      defaultFonts = {
        monospace = [ "Sarasa Mono SC" "Maple Mono NF CN" ];
        sansSerif = [ "Noto Sans CJK SC" "LXGW WenKai" ];
        serif = [ "Noto Serif CJK SC" "Source Han Serif SC" ];
        emoji = [ "Noto Color Emoji" ];
      };
      antialias = true;
      hinting.enable = true;
    };
  };
}
