{ pkgs, lib, ... }:

{
  time.timeZone = "Asia/Shanghai";

  i18n = {
    defaultLocale = "en_US.UTF-8";
    supportedLocales = [
      "en_US.UTF-8/UTF-8"
      "zh_CN.UTF-8/UTF-8"
    ];
    inputMethod = {
      enable = true;
      type = "fcitx5";
      fcitx5 = {
        waylandFrontend = true;
        addons =
          (with pkgs; [
            qt6Packages.fcitx5-chinese-addons
            fcitx5-rime
            fcitx5-gtk
          ])
          ++ lib.optionals (lib.hasAttrByPath [ "qt6Packages" "fcitx5-configtool" ] pkgs) [
            pkgs.qt6Packages.fcitx5-configtool
          ]
          ++ lib.optionals (lib.hasAttrByPath [ "fcitx5-configtool" ] pkgs) [
            pkgs.fcitx5-configtool
          ]
          ++ lib.optionals (lib.hasAttrByPath [ "fcitx5-qt" ] pkgs) [ pkgs.fcitx5-qt ]
          ++ lib.optionals (lib.hasAttrByPath [ "qt6Packages" "fcitx5-qt" ] pkgs) [ pkgs.qt6Packages.fcitx5-qt ];
      };
    };
  };
}
