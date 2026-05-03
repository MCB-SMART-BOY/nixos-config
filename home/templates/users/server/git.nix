# server 用户模板的 Git 身份示例。

{ ... }:

{
  # 个人 Git 身份（提交时显示）
  programs.git.settings.user = {
    name = "your-name";
    email = "your-email@example.com";
  };
}
