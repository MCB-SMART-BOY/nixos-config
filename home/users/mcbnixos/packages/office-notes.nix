# 办公与知识管理。

{ pkgs, ... }:

with pkgs; [
  obsidian # Markdown 知识库
  libreoffice-still # 办公套件（稳定分支）
  xournalpp # 手写笔记/PDF 标注
  goldendict-ng # 词典工具
]
