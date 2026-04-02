# 系统包辅助套件：把一组常见命令以包集合形式统一暴露。

{
  pkgs,
  mcbctlPkg,
}:

let
  batExtrasSuite = pkgs.symlinkJoin {
    name = "bat-extras-suite";
    paths = with pkgs.bat-extras; [
      batdiff
      batgrep
      batman
      batwatch
      prettybat
    ];
  };

  schedulerCliSuite = pkgs.symlinkJoin {
    name = "scheduler-cli-suite";
    paths = with pkgs; [
      cronie
      at
    ];
  };

  classicAdminSuite = pkgs.symlinkJoin {
    name = "classic-admin-suite";
    paths = with pkgs; [
      acl
      attr
      bc
      time
      patch
      dos2unix
    ];
  };

  mailCliSuite = pkgs.symlinkJoin {
    name = "mail-cli-suite";
    paths = with pkgs; [
      mailutils
      procmail
    ];
  };

  # Wrap musicfox startup and harden playback-related defaults.
  goMusicfoxCompat = pkgs.runCommand "go-musicfox-compat" { } ''
    mkdir -p "$out/bin"
    ln -s ${pkgs.go-musicfox}/bin/musicfox "$out/bin/musicfox-real"
    ln -s ${pkgs.go-musicfox}/bin/musicfox "$out/bin/go-musicfox-real"
    ln -s ${mcbctlPkg}/bin/musicfox-wrapper "$out/bin/musicfox"
    ln -s ${mcbctlPkg}/bin/musicfox-wrapper "$out/bin/go-musicfox"
  '';
in
{
  inherit
    batExtrasSuite
    schedulerCliSuite
    classicAdminSuite
    mailCliSuite
    goMusicfoxCompat
    ;
}
