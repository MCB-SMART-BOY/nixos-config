# 日常 CLI 基础工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableShellTools")) (with pkgs; [
  git # 版本控制
  lazygit # Git TUI
  wget # 文件下载（非交互）
  curl # HTTP 调试/下载
  openssh # ssh/scp/sftp
  man-db # man 命令本体
  man-pages # 常见 Linux 手册页
  bind # dig/nslookup
  netcat-openbsd # nc
  pciutils # lspci
  file # 文件类型识别
  tree # 目录树查看
  unzip # 解压 zip
  zip # 打包 zip
  p7zip # 7z 压缩工具
  rsync # 增量同步
  eza # ls 增强
  fd # find 替代，速度快
  fzf # 模糊搜索
  ripgrep # 全文搜索
  bat # cat 高亮版
  bat-extras.batdiff # diff 高亮查看
  bat-extras.batgrep # grep 结果高亮查看
  bat-extras.batman # man 彩色渲染
  bat-extras.batwatch # watch + bat
  bat-extras.prettybat # bat 美化输出
  delta # git diff 美化
  xh # HTTPie 风格的现代 HTTP 客户端
  doggo # dig 的现代替代
])
