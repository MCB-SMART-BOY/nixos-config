{
  fetchFromGitHub,
  lib,
  makeWrapper,
  unstablePkgs,
  pkg-config,
  stdenv,
  openssl,
  gtk3,
  xdotool,
  wayland,
  libxkbcommon,
  libglvnd,
  mesa,
  copyDesktopItems,
  makeDesktopItem,
}:

let
  sourceInfo = import ./source.nix;
  rustPlatform = unstablePkgs.makeRustPlatform {
    cargo = unstablePkgs.cargo;
    rustc = unstablePkgs.rustc;
  };
  runtimeLibs = [
    gtk3
    xdotool
    wayland
    libxkbcommon
    libglvnd
    mesa
  ];
  runtimeLibraryPath = lib.makeLibraryPath runtimeLibs;
  desktopItem = makeDesktopItem {
    name = "gridix";
    desktopName = "Gridix";
    comment = "A simple database manager written in Rust";
    exec = "gridix";
    icon = "gridix";
    categories = [
      "Development"
      "Utility"
    ];
  };
in
rustPlatform.buildRustPackage {
  pname = "gridix";
  inherit (sourceInfo) version;

  src = fetchFromGitHub {
    owner = "MCB-SMART-BOY";
    repo = "Gridix";
    inherit (sourceInfo) rev hash;
  };

  inherit (sourceInfo) cargoHash;

  nativeBuildInputs = [
    copyDesktopItems
    makeWrapper
    pkg-config
  ];

  buildInputs = [ openssl ] ++ lib.optionals stdenv.hostPlatform.isLinux runtimeLibs;

  doCheck = false;

  postInstall = lib.optionalString stdenv.hostPlatform.isLinux ''
    install -Dm444 gridix.png "$out/share/icons/hicolor/256x256/apps/gridix.png"

    wrapProgram "$out/bin/gridix" \
      --prefix LD_LIBRARY_PATH : "${runtimeLibraryPath}" \
      --set-default __EGL_VENDOR_LIBRARY_DIRS "${mesa}/share/glvnd/egl_vendor.d" \
      --set-default LIBGL_DRIVERS_PATH "${mesa}/lib/dri"
  '';

  desktopItems = lib.optionals stdenv.hostPlatform.isLinux [ desktopItem ];

  meta = {
    description = "Fast, secure, cross-platform database management tool with Helix/Vim keybindings";
    homepage = "https://github.com/MCB-SMART-BOY/Gridix";
    license = lib.licenses.asl20;
    mainProgram = "gridix";
    platforms = lib.platforms.linux;
  };
}
