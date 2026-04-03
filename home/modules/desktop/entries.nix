{
  config,
  lib,
  ...
}:

{
  config = {
    xdg.desktopEntries."sioyek" = {
      name = "Sioyek";
      genericName = "PDF Viewer";
      comment = "PDF viewer optimized for research papers";
      exec = "sioyek %U";
      icon = "sioyek";
      categories = [
        "Office"
        "Viewer"
      ];
      mimeType = [
        "application/pdf"
        "application/postscript"
      ];
      startupNotify = true;
      terminal = false;
    };

    xdg.desktopEntries."zotero" = {
      name = "Zotero";
      genericName = "Reference Manager";
      comment = "Collect, organize and cite research";
      exec = "zotero %U";
      icon = "zotero";
      categories = [
        "Office"
        "Education"
        "Science"
      ];
      mimeType = [
        "x-scheme-handler/zotero"
        "text/x-bibtex"
        "application/x-research-info-systems"
      ];
      startupNotify = true;
      terminal = false;
    };

    xdg.desktopEntries."dev.zed.Zed" = lib.mkIf config.mcb.desktopEntries.enableZed {
      name = "Zed";
      genericName = "Text Editor";
      comment = "A high-performance, multiplayer code editor.";
      exec = "zed-auto-gpu %U";
      icon = "zed";
      categories = [
        "Utility"
        "TextEditor"
        "Development"
        "IDE"
      ];
      mimeType = [
        "text/plain"
        "application/x-zerosize"
        "x-scheme-handler/zed"
      ];
      startupNotify = true;
      terminal = false;
    };

    xdg.desktopEntries."io.github.msojocs.bilibili" = {
      name = "Bilibili";
      comment = "Bilibili Desktop";
      exec = "electron-auto-gpu bilibili %U";
      icon = "io.github.msojocs.bilibili";
      categories = [
        "AudioVideo"
        "Video"
        "TV"
      ];
      startupNotify = true;
      terminal = false;
    };

    xdg.desktopEntries."discord" = {
      name = "Discord";
      genericName = "All-in-one cross-platform voice and text chat for gamers";
      exec = "electron-auto-gpu Discord %U";
      icon = "discord";
      categories = [
        "Network"
        "InstantMessaging"
      ];
      mimeType = [ "x-scheme-handler/discord" ];
      startupNotify = true;
      terminal = false;
    };

    xdg.desktopEntries."obsidian" = {
      name = "Obsidian";
      comment = "Knowledge base";
      exec = "electron-auto-gpu obsidian %U";
      icon = "obsidian";
      categories = [ "Office" ];
      mimeType = [ "x-scheme-handler/obsidian" ];
      startupNotify = true;
      terminal = false;
    };

    xdg.desktopEntries."clash-verge" = {
      name = "Clash Verge";
      comment = "Clash Verge Rev";
      exec = "electron-auto-gpu clash-verge %U";
      icon = "clash-verge";
      categories = [ "Development" ];
      mimeType = [ "x-scheme-handler/clash" ];
      startupNotify = true;
      terminal = false;
    };

    xdg.desktopEntries."clash-nyanpasu" = {
      name = "Clash Nyanpasu";
      comment = "Clash Nyanpasu! (∠・ω< )⌒☆";
      exec = "electron-auto-gpu clash-nyanpasu";
      icon = "clash-nyanpasu";
      categories = [ "Development" ];
      startupNotify = true;
      terminal = false;
    };

    xdg.desktopEntries."yesplaymusic" = lib.mkIf config.mcb.desktopEntries.enableYesPlayMusic {
      name = "YesPlayMusic";
      comment = "A third-party music player for Netease Music";
      exec = "electron-auto-gpu yesplaymusic --no-sandbox %U";
      icon = "yesplaymusic";
      categories = [
        "AudioVideo"
        "Audio"
        "Player"
        "Music"
      ];
      startupNotify = true;
      terminal = false;
    };
  };
}
