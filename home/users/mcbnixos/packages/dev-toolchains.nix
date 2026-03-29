# 开发工具链。

{ pkgs, ... }:

with pkgs; [
  rustup # Rust 工具链管理
  opam # OCaml 包管理
  elan # Lean 工具链管理
  gnumake # make 构建
  cmake # C/C++ 构建系统
  pkg-config # 库探测
  openssl # TLS 库与工具
  gcc # GNU C/C++ 编译器
  binutils # 链接器/二进制工具
  clang-tools # Clang 工具集
  uv # Python 包与虚拟环境工具
  conda # Python/数据科学环境管理
]
