# Home Manager Shell 配置入口：只负责聚合 shell 子模块。

{
  imports = [
    ./shell/core.nix
    ./shell/replacements.nix
    ./shell/navigation.nix
    ./shell/prompt.nix
  ];
}
