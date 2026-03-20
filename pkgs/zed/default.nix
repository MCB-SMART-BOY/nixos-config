{
  fetchurl,
  lib,
  makeWrapper,
  stdenvNoCC,
}:

let
  sources = import ./source.nix;
  sourceInfo =
    if builtins.hasAttr stdenvNoCC.hostPlatform.system sources then
      sources.${stdenvNoCC.hostPlatform.system}
    else
      throw "zed-official is not supported on ${stdenvNoCC.hostPlatform.system}";
in
stdenvNoCC.mkDerivation {
  pname = "zed-official";
  inherit (sourceInfo) version;

  src = fetchurl {
    inherit (sourceInfo) url hash;
  };

  nativeBuildInputs = [ makeWrapper ];

  sourceRoot = ".";

  installPhase = ''
    runHook preInstall

    mkdir -p "$out/lib" "$out/bin" "$out/share"
    cp -a zed.app "$out/lib/zed.app"
    cp -a "$out/lib/zed.app/share/applications" "$out/share/"
    cp -a "$out/lib/zed.app/share/icons" "$out/share/"

    # Keep desktop launcher and wrappers aligned with current workflow (zeditor).
    substituteInPlace "$out/share/applications/dev.zed.Zed.desktop" \
      --replace-fail "Exec=zed %U" "Exec=zeditor %U"

    makeWrapper "$out/lib/zed.app/bin/zed" "$out/bin/zed"
    makeWrapper "$out/bin/zed" "$out/bin/zeditor"

    runHook postInstall
  '';

  meta = {
    description = "Zed editor from upstream official stable binaries";
    homepage = "https://zed.dev";
    license = lib.licenses.gpl3Only;
    mainProgram = "zeditor";
    platforms = builtins.attrNames sources;
  };
}
