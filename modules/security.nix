# 安全相关配置：polkit/sudo 等系统权限策略。
# wheel 组可无密码管理 NetworkManager；sudo 需要密码。

{ ... }:

{
  security = {
    polkit = {
      enable = true;
      extraConfig = ''
        polkit.addRule(function(action, subject) {
          if (action.id.indexOf("org.freedesktop.NetworkManager.") == 0 && subject.isInGroup("wheel")) {
            return polkit.Result.YES;
          }
        });
      '';
    };

    sudo = {
      enable = true;
      wheelNeedsPassword = true;
    };
  };
}
