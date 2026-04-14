{
  lib,
  rustPlatform,
  doCheck ? true,
}:

rustPlatform.buildRustPackage {
  pname = "mcbctl";
  version = "3.0.0";

  src = lib.cleanSourceWith {
    src = ../../mcbctl;
    filter =
      path: type:
      let
        base = baseNameOf path;
      in
      !(base == "target" || base == ".gitignore");
  };

  cargoLock.lockFile = ../../mcbctl/Cargo.lock;

  CARGO_TARGET_DIR = "target";

  inherit doCheck;

  postInstall = ''
    ln -s "$out/bin/mcb-deploy" "$out/bin/deploy"
  '';

  meta = {
    description = "Rust-based control suite for this NixOS configuration";
    mainProgram = "mcbctl";
    platforms = lib.platforms.linux;
  };
}
