# 终端与浏览器。

{ pkgs, ... }:

with pkgs; [
  foot # 轻量 Wayland 终端
  alacritty # GPU 加速终端
  firefox # 浏览器（开源）
  google-chrome # 浏览器（Chromium 系）
]
