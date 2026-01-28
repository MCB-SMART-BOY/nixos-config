# 语言与输入法设置：locale、时区、fcitx5 及其插件。
# 这里的 inputMethod 会影响系统级 IM 启用与插件安装。

{ pkgs, lib, ... }:

let
  # nixpkgs 新版本将 fcitx5-* 顶层包改为 qt6Packages.*，旧名会 throw。
  # 使用 tryEval + getAttrFromPath 避免访问到 throw 别名导致构建失败。
  resolvePkg = path:
    let
      eval =
        if lib.hasAttrByPath path pkgs then
          builtins.tryEval (lib.getAttrFromPath path pkgs)
        else
          { success = false; value = null; };
    in
    if eval.success then eval.value else null;

  pickFirst = list: lib.findFirst (x: x != null) null list;

  fcitx5ChineseAddons = pickFirst [
    (resolvePkg [ "qt6Packages" "fcitx5-chinese-addons" ])
    (resolvePkg [ "fcitx5-chinese-addons" ])
  ];

  fcitx5Configtool = pickFirst [
    (resolvePkg [ "qt6Packages" "fcitx5-configtool" ])
    (resolvePkg [ "fcitx5-configtool" ])
  ];

  fcitx5Rime = pickFirst [
    (resolvePkg [ "qt6Packages" "fcitx5-rime" ])
    (resolvePkg [ "fcitx5-rime" ])
  ];

  fcitx5Gtk = pickFirst [
    (resolvePkg [ "qt6Packages" "fcitx5-gtk" ])
    (resolvePkg [ "fcitx5-gtk" ])
  ];

  fcitx5Qt = pickFirst [
    (resolvePkg [ "qt6Packages" "fcitx5-qt" ])
    (resolvePkg [ "libsForQt5" "fcitx5-qt" ])
    (resolvePkg [ "fcitx5-qt" ])
  ];
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
          ++ lib.optionals (fcitx5Rime != null) [ fcitx5Rime ]
          ++ lib.optionals (fcitx5Gtk != null) [ fcitx5Gtk ]
          ++ lib.optionals (fcitx5Configtool != null) [ fcitx5Configtool ]
          ++ lib.optionals (fcitx5Qt != null) [ fcitx5Qt ];
      };
    };
  };
}
