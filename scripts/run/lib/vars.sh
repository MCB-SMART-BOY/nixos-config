# run.sh 共享状态变量。
# 由 run.sh 在加载其他库之前 source，确保所有子脚本共享同一套状态。
# 不要在子脚本中声明同名全局变量；统一在此处管理。

# 脚本版本与仓库信息
RUN_SH_VERSION="v2026.03.20"
REPO_URLS=(
  "https://gitee.com/MCB-SMART-BOY/nixos-config.git"
  "https://github.com/MCB-SMART-BOY/nixos-config.git"
)
BRANCH="master"
SOURCE_REF=""
ALLOW_REMOTE_HEAD=false
SOURCE_COMMIT=""
SOURCE_CHOICE_SET=false
GIT_CLONE_TIMEOUT_SEC="${GIT_CLONE_TIMEOUT_SEC:-90}"

# 运行状态（由向导填充）
TARGET_NAME=""
TARGET_USERS=()
TARGET_ADMIN_USERS=()
DEPLOY_MODE="manage-users"   # manage-users | update-existing
DEPLOY_MODE_SET=false
FORCE_REMOTE_SOURCE=false
OVERWRITE_MODE="ask"
OVERWRITE_MODE_SET=false
HOST_PROFILE_KIND="unknown"

# 每用户 TUN 已移除（代理由 clash-verge-rev GUI 自行管理）。

# 服务器软件/虚拟化临时覆盖
SERVER_OVERRIDES_ENABLED=false
SERVER_ENABLE_NETWORK_CLI=""
SERVER_ENABLE_NETWORK_GUI=""
SERVER_ENABLE_SHELL_TOOLS=""
SERVER_ENABLE_WAYLAND_TOOLS=""
SERVER_ENABLE_SYSTEM_TOOLS=""
SERVER_ENABLE_GEEK_TOOLS=""
SERVER_ENABLE_GAMING=""
SERVER_ENABLE_INSECURE_TOOLS=""
SERVER_ENABLE_DOCKER=""
SERVER_ENABLE_LIBVIRTD=""

# 自动生成的 Home Manager 用户模板
CREATED_HOME_USERS=()

# GPU 临时覆盖
GPU_OVERRIDE=false
GPU_MODE=""
GPU_IGPU_VENDOR=""
GPU_PRIME_MODE=""
GPU_INTEL_BUS=""
GPU_AMD_BUS=""
GPU_NVIDIA_BUS=""
GPU_NVIDIA_OPEN=""
GPU_SPECIALISATIONS_ENABLED=false
GPU_SPECIALISATION_MODES=()
GPU_SPECIALISATIONS_SET=false

# nixos-rebuild 模式（switch/test/build）
MODE="switch"

# 是否在 nixos-rebuild 时升级上游依赖（默认关闭，保证可复现）
REBUILD_UPGRADE=false

# 目标配置目录（默认 /etc/nixos）
ETC_DIR="/etc/nixos"

# 临时 DNS 是否已开启
DNS_ENABLED=false

# 临时仓库目录
TMP_DIR=""

# sudo wrapper（root 模式下为空）
SUDO="sudo"

# rootless 模式标记（无 sudo/无法提权）
ROOTLESS=false

# 运行模式：deploy | release
RUN_ACTION="deploy"

# 进度条控制
PROGRESS_TOTAL=7
PROGRESS_CURRENT=0
WIZARD_ACTION=""

# 颜色定义（由 init_colors() 初始化）
COLOR_RESET=""
COLOR_BOLD=""
COLOR_DIM=""
COLOR_GREEN=""
COLOR_YELLOW=""
COLOR_RED=""
COLOR_CYAN=""

# 初始化颜色支持
init_colors() {
  if command -v tput >/dev/null 2>&1 && [[ -t 1 ]]; then
    COLOR_RESET="$(tput sgr0)"
    COLOR_BOLD="$(tput bold)"
    COLOR_DIM="$(tput dim)"
    COLOR_GREEN="$(tput setaf 2)"
    COLOR_YELLOW="$(tput setaf 3)"
    COLOR_RED="$(tput setaf 1)"
    COLOR_CYAN="$(tput setaf 6)"
  fi
}
