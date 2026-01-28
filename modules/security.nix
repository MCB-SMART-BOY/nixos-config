# 安全相关配置：polkit/sudo 等系统权限策略。
# 这里允许 wheel 组管理 NetworkManager。

{ ... }:

{
  security.polkit = {
    enable = true;
    extraConfig = ''
      polkit.addRule(function(action, subject) {
        if (action.id.indexOf("org.freedesktop.NetworkManager.") == 0 && subject.isInGroup("wheel")) {
          return polkit.Result.YES;
        }
      });
    '';
  };
}
