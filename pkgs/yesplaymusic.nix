{
  appimageTools,
  fetchurl,
  lib,
}:

let
  pname = "yesplaymusic";
  version = "0.4.10";

  src = fetchurl {
    url = "https://github.com/qier222/YesPlayMusic/releases/download/v${version}/YesPlayMusic-${version}.AppImage";
    hash = "sha256-Qj9ZQbHqzKX2QBlXWtey/j/4PqrCJCObdvOans79KW4=";
  };

  appimageContents = appimageTools.extractType2 {
    inherit pname version src;
  };
in
appimageTools.wrapType2 {
  inherit pname version src;

  # Runtime libs required by upstream Electron bundle on NixOS.
  extraPkgs =
    pkgs: with pkgs; [
      alsa-lib
      atk
      at-spi2-atk
      cairo
      cups
      dbus
      expat
      gdk-pixbuf
      glib
      gtk3
      libdrm
      libxkbcommon
      libxshmfence
      mesa
      nspr
      nss
      pango
      xorg.libX11
      xorg.libXcomposite
      xorg.libXdamage
      xorg.libXext
      xorg.libXfixes
      xorg.libXrandr
      xorg.libxcb
    ];

  extraInstallCommands = ''
        # Upstream desktop file points to AppRun; repoint to wrapped binary.
        install -Dm444 ${appimageContents}/yesplaymusic.desktop $out/share/applications/yesplaymusic.desktop
        substituteInPlace $out/share/applications/yesplaymusic.desktop \
          --replace-fail "Exec=AppRun --no-sandbox %U" "Exec=yesplaymusic --no-sandbox %U"

        # Preserve upstream icon set for desktop environments.
        cp -r ${appimageContents}/usr/share/icons $out/share/

        # Keep CLI behavior aligned with desktop entry.
        mv $out/bin/yesplaymusic $out/bin/.yesplaymusic-wrapped
        cat > $out/bin/yesplaymusic <<EOF
    #!/usr/bin/env bash
    exec $out/bin/.yesplaymusic-wrapped --no-sandbox "\$@"
    EOF
        chmod 0755 $out/bin/yesplaymusic
  '';

  meta = {
    description = "A third-party Netease Cloud Music player";
    homepage = "https://github.com/qier222/YesPlayMusic";
    license = lib.licenses.mit;
    mainProgram = "yesplaymusic";
    platforms = [ "x86_64-linux" ];
  };
}
