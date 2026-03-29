# 科研阅读与文档写作。

{ pkgs, ... }:

with pkgs; [
  sioyek # 学术 PDF 阅读器
  zotero # 文献管理
  pandoc # 文档格式转换
  typst # 新一代排版引擎
  tinymist # Typst LSP
  texstudio # LaTeX IDE
  texlab # LaTeX LSP
  texlive.combined.scheme-medium # TeX Live 中型套装
  biber # BibLaTeX 参考文献工具
  qpdf # PDF 结构处理
  poppler-utils # PDF 命令行工具（pdftotext 等）
]
