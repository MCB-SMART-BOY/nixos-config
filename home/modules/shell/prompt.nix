# Shell 提示与历史层：提示符、历史同步和搜索体验。

{ ... }:

{
  programs.atuin = {
    enable = true;
    enableFishIntegration = true;
    settings = {
      auto_sync = true;
      update_check = false;
      search_mode = "fuzzy";
      sync_frequency = "5m";
    };
  };

  programs.starship = {
    # 统一提示符主题（配置在 home/users/*/config/starship.toml）
    enable = true;
    enableFishIntegration = true;
  };
}
