{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "mcb-scripts-rs";
  version = "0.1.0";

  src = lib.cleanSourceWith {
    src = ../../scripts-rs;
    filter =
      path: type:
      let
        base = baseNameOf path;
      in
      !(base == "target" || base == ".gitignore");
  };

  cargoLock.lockFile = ../../scripts-rs/Cargo.lock;

  doCheck = false;

  postInstall = ''
    ln -s "$out/bin/lock-screen-rs" "$out/bin/lock-screen"
    ln -s "$out/bin/niri-run-rs" "$out/bin/niri-run"
    ln -s "$out/bin/noctalia-bluetooth-rs" "$out/bin/noctalia-bluetooth"
    ln -s "$out/bin/noctalia-cpu-rs" "$out/bin/noctalia-cpu"
    ln -s "$out/bin/noctalia-disk-rs" "$out/bin/noctalia-disk"
    ln -s "$out/bin/noctalia-flake-updates-rs" "$out/bin/noctalia-flake-updates"
    ln -s "$out/bin/noctalia-gpu-mode-rs" "$out/bin/noctalia-gpu-mode"
    ln -s "$out/bin/noctalia-gpu-current-rs" "$out/bin/noctalia-gpu-current"
    ln -s "$out/bin/noctalia-memory-rs" "$out/bin/noctalia-memory"
    ln -s "$out/bin/noctalia-net-speed-rs" "$out/bin/noctalia-net-speed"
    ln -s "$out/bin/noctalia-net-status-rs" "$out/bin/noctalia-net-status"
    ln -s "$out/bin/noctalia-power-rs" "$out/bin/noctalia-power"
    ln -s "$out/bin/noctalia-proxy-status-rs" "$out/bin/noctalia-proxy-status"
    ln -s "$out/bin/noctalia-temperature-rs" "$out/bin/noctalia-temperature"
    ln -s "$out/bin/wallpaper-random-rs" "$out/bin/wallpaper-random"

    ln -s "$out/bin/clash-verge-prestart-rs" "$out/bin/clash-verge-prestart"
    ln -s "$out/bin/flatpak-setup-rs" "$out/bin/flatpak-setup"
    ln -s "$out/bin/mcb-tun-route-rs" "$out/bin/mcb-tun-route"
    ln -s "$out/bin/musicfox-wrapper-rs" "$out/bin/musicfox-wrapper"
    ln -s "$out/bin/zed-auto-gpu-rs" "$out/bin/zed-auto-gpu"
    ln -s "$out/bin/electron-auto-gpu-rs" "$out/bin/electron-auto-gpu"
    ln -s "$out/bin/steam-gamescope-rs" "$out/bin/steam-gamescope"

    ln -s "$out/bin/run-rs" "$out/bin/run"
    ln -s "$out/bin/update-zed-source-rs" "$out/bin/update-zed-source"
    ln -s "$out/bin/update-yesplaymusic-source-rs" "$out/bin/update-yesplaymusic-source"
    ln -s "$out/bin/update-upstream-apps-rs" "$out/bin/update-upstream-apps"
  '';

  meta = {
    description = "Rust-based script suite for this NixOS configuration";
    mainProgram = "run-rs";
    platforms = lib.platforms.linux;
  };
}
