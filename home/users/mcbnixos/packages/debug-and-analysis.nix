# 调试、诊断与分析工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableGeekTools")) (with pkgs; [
  strace # 系统调用跟踪
  ltrace # 动态库调用跟踪
  gdb # GNU 调试器
  lldb # LLVM 调试器
  patchelf # ELF 修改工具
  file # 文件类型识别
  htop # 进程查看器
  iotop # IO 占用监控
  iftop # 网络流量监控
  sysstat # 系统性能统计（sar/iostat）
  lsof # 文件句柄/端口占用查询
  mtr # 路由诊断
  nmap # 端口扫描
  tcpdump # 抓包 CLI
  traceroute # 路由追踪
  socat # 套接字转发/桥接
  iperf3 # 网络带宽测试
  ethtool # 网卡参数查询/调优
  hyperfine # 命令基准测试
  tokei # 代码行数统计
  tree # 目录树查看
  unzip # 解压 zip
  zip # 打包 zip
  p7zip # 7z 压缩工具
  rsync # 增量同步
  rclone # 多云存储同步
  just # 任务运行器
  entr # 文件变化触发命令
  ncdu # 磁盘占用 TUI
  binwalk # 固件分析
  radare2 # 逆向分析框架
  wireshark # 图形化抓包工具
  vulkan-tools # Vulkan 诊断工具
  gh # GitHub CLI
  hexyl # 十六进制查看器
])
