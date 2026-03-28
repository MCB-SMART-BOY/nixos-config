# run.sh UI / 交互通用函数

# 统一输出带颜色的日志。
msg() {
  local level="$1"
  local label
  shift
  case "${level}" in
    INFO) label="${COLOR_CYAN}信息${COLOR_RESET}" ;;
    OK) label="${COLOR_GREEN}完成${COLOR_RESET}" ;;
    WARN) label="${COLOR_YELLOW}警告${COLOR_RESET}" ;;
    ERROR) label="${COLOR_RED}错误${COLOR_RESET}" ;;
    *) label="${level}" ;;
  esac
  printf '[%s] %s\n' "${label}" "$*"
}

# 输出普通信息。
log() { msg INFO "$*"; }
# 输出成功信息。
success() { msg OK "$*"; }
# 输出警告信息。
warn() { msg WARN "$*"; }
# 输出错误并退出。
error() {
  msg ERROR "$*"
  exit 1
}

# 以 root 执行命令（root 模式下跳过 sudo）。
as_root() {
  if [[ -n "${SUDO}" ]]; then
    sudo "$@"
  else
    "$@"
  fi
}

# 打印脚本标题。
banner() {
  printf '%s\n' "${COLOR_BOLD}==========================================${COLOR_RESET}"
  printf '%s\n' "${COLOR_BOLD}  NixOS 一键部署（run.sh ${RUN_SH_VERSION}） ${COLOR_RESET}"
  printf '%s\n' "${COLOR_BOLD}==========================================${COLOR_RESET}"
}

# 打印章节标题。
section() {
  printf '\n%s%s%s\n' "${COLOR_BOLD}" "$*" "${COLOR_RESET}"
}

# 打印灰色提示。
note() {
  printf '%s%s%s\n' "${COLOR_DIM}" "$*" "${COLOR_RESET}"
}

# 判断是否交互终端。
is_tty() {
  [[ -t 0 && -t 1 ]]
}

# 读取用户输入，支持默认值。
ask_input() {
  local prompt="$1"
  local default="$2"
  local input=""
  if is_tty; then
    read -r -p "${prompt}（默认 ${default}）： " input
  fi
  if [[ -z "${input}" ]]; then
    printf '%s' "${default}"
  else
    printf '%s' "${input}"
  fi
}

# 更新并显示进度条。
progress_step() {
  local label="$1"
  PROGRESS_CURRENT=$((PROGRESS_CURRENT + 1))
  local width=24
  local filled=$((PROGRESS_CURRENT * width / PROGRESS_TOTAL))
  local empty=$((width - filled))
  local bar
  bar="$(printf "%${filled}s" | tr ' ' '#')"
  bar+=$(printf "%${empty}s" | tr ' ' '-')
  printf '%s进度: [%s] %s/%s %s%s\n' "${COLOR_CYAN}" "${bar}" "${PROGRESS_CURRENT}" "${PROGRESS_TOTAL}" "${label}" "${COLOR_RESET}"
}

# 确认是否继续执行。
confirm_continue() {
  local prompt="$1"
  if ! is_tty; then
    return 0
  fi
  local answer
  read -r -p "${prompt} [Y/n] " answer
  case "${answer}" in
    n|N) error "已退出" ;;
    *) return 0 ;;
  esac
}

# 显示菜单并读取选择。
menu_prompt() {
  local title="$1"
  local default_index="$2"
  shift 2
  local options=("$@")
  local choice=""
  local total=${#options[@]}

  while true; do
    # 打印菜单选项
    printf '\n%s%s%s\n' "${COLOR_BOLD}" "${title}" "${COLOR_RESET}" >&2
    local i=1
    for opt in "${options[@]}"; do
      printf '  %s) %s\n' "${i}" "${opt}" >&2
      i=$((i + 1))
    done
    read -r -p "请选择 [1-${total}]（默认 ${default_index}，输入 q 退出）： " choice
    case "${choice}" in
      q|Q)
        error "已退出"
        ;;
    esac
    if [[ -z "${choice}" ]]; then
      choice="${default_index}"
    fi
    if [[ "${choice}" =~ ^[0-9]+$ ]] && ((choice >= 1 && choice <= total)); then
      printf '%s' "${choice}"
      return 0
    fi
    echo "无效选择，请重试。" >&2
  done
}

# 向导中处理返回/退出。
wizard_back_or_quit() {
  local prompt="$1"
  local answer=""
  # c=继续，b=返回，q=退出
  read -r -p "${prompt} [c继续/b返回/q退出]（默认 c）： " answer
  case "${answer}" in
    b|B) WIZARD_ACTION="back" ;;
    q|Q) error "已退出" ;;
    *) WIZARD_ACTION="continue" ;;
  esac
}
