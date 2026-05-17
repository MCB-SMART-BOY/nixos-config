# 全局覆盖层：集中管理 nixpkgs 包补丁与版本固定。
# 此前这些覆盖层内联在 flake.nix 中；现移出以保持 flake.nix 精简。
# 新增覆盖层时，在此文件中追加；flake.nix 会自动导入。

final: prev: {
  valkey = prev.valkey.overrideAttrs (old: {
    doCheck = false;
  });
  openldap = prev.openldap.overrideAttrs (old: {
    doCheck = false;
  });
  wireshark = prev.wireshark.overrideAttrs (old: {
    src = final.fetchzip {
      url = "https://www.wireshark.org/download/src/wireshark-${old.version}.tar.xz";
      hash = "sha256-CMybqzDHi+HiC7zCcVTz0aGMY93K4BbtMLg2sDHypc8=";
    };
  });
}
