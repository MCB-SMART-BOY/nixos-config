# 语言与输入法设置：locale、时区、fcitx5 及其插件。
# 这里的 inputMethod 会影响系统级 IM 启用与插件安装。

{ pkgs, lib, ... }:

let
  fcitx5ChineseAddons =
    if lib.hasAttrByPath [ "fcitx5-chinese-addons" ] pkgs then
      pkgs.fcitx5-chinese-addons
    else if lib.hasAttrByPath [ "qt6Packages" "fcitx5-chinese-addons" ] pkgs then
      pkgs.qt6Packages.fcitx5-chinese-addons
    else
      null;
  fcitx5Configtool =
    if lib.hasAttrByPath [ "qt6Packages" "fcitx5-configtool" ] pkgs then
      pkgs.qt6Packages.fcitx5-configtool
    else if lib.hasAttrByPath [ "fcitx5-configtool" ] pkgs then
      pkgs.fcitx5-configtool
    else
      null;
in
{
  # 时区（影响系统时间显示）
  time.timeZone = "Asia/Shanghai";

  i18n = {
    # 默认语言（GUI/CLI 都会使用）
    defaultLocale = "en_US.UTF-8";
    # 额外启用中文 locale，方便终端/应用显示中文
    supportedLocales = [
      "en_US.UTF-8/UTF-8"
      "zh_CN.UTF-8/UTF-8"
    ];
    inputMethod = {
      enable = true;
      type = "fcitx5";
      fcitx5 = {
        waylandFrontend = true;
        # 输入法插件：Rime/Pinyin/GTK 支持等
        addons =
          (lib.optionals (fcitx5ChineseAddons != null) [ fcitx5ChineseAddons ])
          ++ (with pkgs; [
            fcitx5-rime
            fcitx5-gtk
          ])
          ++ lib.optionals (fcitx5Configtool != null) [ fcitx5Configtool ]
          ++ lib.optionals (lib.hasAttrByPath [ "fcitx5-qt" ] pkgs) [ pkgs.fcitx5-qt ]
          ++ lib.optionals (lib.hasAttrByPath [ "libsForQt5" "fcitx5-qt" ] pkgs) [ pkgs.libsForQt5.fcitx5-qt ]
          ++ lib.optionals (lib.hasAttrByPath [ "qt6Packages" "fcitx5-qt" ] pkgs) [ pkgs.qt6Packages.fcitx5-qt ];
      };
    };
  };
}
