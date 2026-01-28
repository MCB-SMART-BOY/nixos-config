# Home Manager 程序功能开关与默认配置。

{ ... }:

{
  # 终端模拟器（配置在 home/users/*/config/alacritty/）
  programs.alacritty = {
    enable = true;
  };

  # Helix 编辑器（配置在 home/users/*/config/helix/）
  programs.helix = {
    enable = true;
  };
}
