# Home Manager 的 Git 通用配置。

{ ... }:

let
  # 统一 Git 默认编辑器
  editor = "hx";
in
{
  programs.git = {
    enable = true;
    lfs.enable = true;
    settings = {
      core = {
        editor = editor;
        pager = "delta";
      };
      # 让交互式 diff 使用 delta 作为渲染器
      interactive.diffFilter = "delta --color-only";
      delta = {
        navigate = true;
        "side-by-side" = true;
      };
    };
  };
}
