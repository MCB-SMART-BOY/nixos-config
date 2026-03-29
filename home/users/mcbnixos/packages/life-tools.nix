# 日常生活工具。

{ pkgs, ... }:

with pkgs; [
  gnome-calendar # 日历
  gnome-clocks # 时钟
  gnome-calculator # 计算器
  gnome-weather # 天气
  gnome-maps # 地图
  gnome-contacts # 通讯录
  baobab # 磁盘占用可视化
  keepassxc # 密码管理
  simple-scan # 扫描仪前端
]
