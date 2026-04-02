# Nix 缓存策略相关选项：镜像与自定义 substituter 入口。

{ lib, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb.nix = {
    cacheProfile = mkOption {
      type = types.enum [
        "cn"
        "global"
        "official-only"
        "custom"
      ];
      default = "cn";
      description = "Binary cache profile: cn/global/official-only/custom.";
    };

    customSubstituters = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Custom substituters used when mcb.nix.cacheProfile = \"custom\".";
    };

    customTrustedPublicKeys = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Custom trusted-public-keys used when mcb.nix.cacheProfile = \"custom\".";
    };
  };
}
