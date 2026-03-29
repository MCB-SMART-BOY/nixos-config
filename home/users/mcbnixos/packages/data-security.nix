# 数据处理与密钥工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableShellTools")) (with pkgs; [
  jq # JSON 处理
  yq # YAML/JSON 处理
  age # 现代加密工具
  sops # 密文配置管理
])
