# Shell 基础层：fish、tmux、direnv 这类通用交互底座。

{ ... }:

{
  programs.fish = {
    enable = true;
  };

  programs.direnv = {
    # 自动加载 .envrc
    enable = true;
  };

  programs.tmux = {
    # 终端多路复用
    enable = true;
  };
}
