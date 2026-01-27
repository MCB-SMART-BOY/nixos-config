# ZSH Configuration
# ~/.zshrc
# 作者: mcbnixos (NixOS 25.11 Optimized)
# 主题: Catppuccin Mocha

if [ -e /etc/zshrc ]; then
    source /etc/zshrc
fi

# ══════════════════════════════════════════════════════════════════
# 1. 🎨 终端环境与色彩
# ══════════════════════════════════════════════════════════════════
export TERM="xterm-256color"

# 解决某些终端下 Delete/Home/End 键映射异常的问题
bindkey "^[[3~" delete-char
bindkey "^[3;5~" delete-char
bindkey "^[[H" beginning-of-line
bindkey "^[[F" end-of-line

# ══════════════════════════════════════════════════════════════════
# 2. 📁 环境变量 (Environment Variables)
# ══════════════════════════════════════════════════════════════════
# Prefer NixOS setuid wrappers (sudo lives here).
export PATH="/run/wrappers/bin:$PATH"

# 默认编辑器（优先沿用已有环境变量）
if [ -z "${EDITOR:-}" ]; then
    if command -v hx &> /dev/null; then
        export EDITOR="hx"
        export VISUAL="hx"
    else
        export EDITOR="nvim"
        export VISUAL="nvim"
    fi
fi
export BROWSER="firefox"

# 让 man 手册页使用 bat（若不可用则退回 less）
if [ -z "${MANPAGER:-}" ]; then
    if command -v bat &> /dev/null; then
        export MANPAGER="sh -c 'col -bx | bat -l man -p'"
    else
        export MANPAGER="less -R"
    fi
fi

# XDG 标准目录 (规范化配置存储位置)
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_CACHE_HOME="$HOME/.cache"
export XDG_STATE_HOME="$HOME/.local/state"

# Rust 开发环境
export RUSTUP_HOME="$HOME/.rustup"
export CARGO_HOME="$HOME/.cargo"
export PATH="$CARGO_HOME/bin:$PATH"

# Go 开发环境
export GOPATH="$HOME/go"
export PATH="$GOPATH/bin:$PATH"

# 用户私有二进制目录
export PATH="$HOME/.local/bin:$PATH"

# ══════════════════════════════════════════════════════════════════
# 3. 📚 历史记录 (History)
# ══════════════════════════════════════════════════════════════════
HISTFILE="$HOME/.zsh_history"
HISTSIZE=50000
SAVEHIST=50000

setopt EXTENDED_HISTORY          # 记录命令执行的时间戳
setopt HIST_EXPIRE_DUPS_FIRST    # 当历史记录满时，优先删除重复的
setopt HIST_IGNORE_DUPS          # 忽略连续重复的命令 (ls; ls -> 只记一次)
setopt HIST_IGNORE_ALL_DUPS      # 删除历史中所有旧的重复命令
setopt HIST_IGNORE_SPACE         # 忽略以空格开头的命令 (防止密码记录)
setopt HIST_FIND_NO_DUPS         # 搜索历史时不显示重复项
setopt HIST_REDUCE_BLANKS        # 删除命令中的多余空白
setopt HIST_VERIFY               # 执行历史命令前先显示出来确认
setopt SHARE_HISTORY             # 在多个终端窗口间共享历史
setopt INC_APPEND_HISTORY        # 执行完命令立即写入历史文件

# ══════════════════════════════════════════════════════════════════
# 4. ⚙️ Zsh 选项 (Options)
# ══════════════════════════════════════════════════════════════════
setopt AUTO_CD                   # 输入目录名直接进入 (无需 cd)
setopt AUTO_PUSHD                # cd 时自动推入目录栈 (方便 popd 回去)
setopt PUSHD_IGNORE_DUPS         # 目录栈忽略重复
setopt PUSHD_SILENT              # 推入栈时不打印信息
setopt CORRECT                   # 简单的命令拼写纠错
setopt INTERACTIVE_COMMENTS      # 允许在交互式命令行输入注释 (# 后面的内容)
setopt NO_BEEP                   # 关闭烦人的蜂鸣声
setopt EXTENDED_GLOB             # 启用扩展通配符 (如 ^git 排除文件)
setopt GLOB_DOTS                 # 通配符包含隐藏文件
setopt COMPLETE_IN_WORD          # 在单词中间补全
setopt AUTO_MENU                 # 多结果时自动进入菜单
setopt NO_FLOW_CONTROL           # 禁用 Ctrl-S/Ctrl-Q 卡住终端

# 补全样式 (Completion Styles)
zstyle ':completion:*' menu select
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}'
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}

# ══════════════════════════════════════════════════════════════════
# 5. 🚀 现代工具无感替换 (Modern Replacements)
# ══════════════════════════════════════════════════════════════════
# 这里将 ls, cat 等命令替换为 eza, bat 等现代化工具

# ls -> eza
if command -v eza &> /dev/null; then
    alias ls='eza --icons --group-directories-first --git'
    alias ll='eza -l --icons --group-directories-first --git --time-style=long-iso'
    alias la='eza -la --icons --group-directories-first --git'
    alias tree='eza --tree --icons'
fi

# cat -> bat
if command -v bat &> /dev/null; then
    alias cat='bat --paging=never --style=plain' # 像 cat 一样直接输出
    alias catt='bat --paging=always'             # 带行号和分页
fi

# grep -> ripgrep
if command -v rg &> /dev/null; then
    alias grep='rg --color=auto'
fi

# find -> fd
if command -v fd &> /dev/null; then
    alias find='fd'
fi

# df -> duf (更好看的磁盘空间)
if command -v duf &> /dev/null; then
    alias df='duf'
fi

# du -> dust (直观的磁盘占用饼图)
if command -v dust &> /dev/null; then
    alias du='dust'
fi

# ps -> procs (支持高亮的进程管理)
if command -v procs &> /dev/null; then
    alias ps='procs'
fi

# top -> btop
if command -v btop &> /dev/null; then
    alias top='btop'
fi

# cd -> zoxide (智能跳转)
# 输入 z <部分目录名> 即可跳转
if command -v zoxide &> /dev/null; then
    eval "$(zoxide init zsh)"
    alias cd='z'
    alias cdi='zi'
fi

# 🛡️ 后悔药：保留原生命令的访问方式
alias oldls='command ls'
alias oldcat='command cat'
alias oldgrep='command grep'

# ══════════════════════════════════════════════════════════════════
# 6. 🛠️ 常用别名 (Aliases)
# ══════════════════════════════════════════════════════════════════

# --- NixOS 管理 ---
alias nrs='sudo nixos-rebuild switch --flake "/etc/nixos#nixos" --show-trace --upgrade' # 一键更新并重建
alias nrt='sudo nixos-rebuild test'        # 测试新配置但不设为默认
alias nrb='sudo nixos-rebuild boot'        # 下次启动时应用
alias nfu='nix flake update'               # 更新 flake.lock
alias nsp='nix search nixpkgs'             # 搜索软件包
alias nsh='nix-shell'                      # 进入临时 Shell
alias ngc='sudo nix-collect-garbage -d'    # 清理旧系统版本 (慎用)
# 快速查看将要构建/下载的 derivations（判断是否会源码编译）
nrc() {
    local flake="${1:-/etc/nixos#nixosConfigurations.nixos.config.system.build.toplevel}"
    nix build "$flake" --dry-run --accept-flake-config
}

# --- Git ---
alias g='git'
alias ga='git add'
alias gc='git commit'
alias gp='git push'
alias gl='git pull'
alias gs='git status'
alias gd='git diff'
alias gco='git checkout'
alias gb='git branch'
alias glg='git log --oneline --graph --decorate'
alias lg='lazygit'  # 终端 Git GUI 神器

# --- Cargo / Rust ---
alias c='cargo'
alias cb='cargo build'
alias cr='cargo run'
alias ct='cargo test'
alias cc='cargo check'
alias cw='cargo watch -x check' # 自动监控代码变动并检查
alias cf='cargo fmt'
alias ccl='cargo clippy'

# --- 快捷导航 ---
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias ~='cd ~'
alias -- -='cd -'
alias md='mkdir -p'
alias rd='rmdir'
alias cp='cp -i'
alias mv='mv -i'
alias rm='rm -i'

# --- 编辑器快捷键 ---
e() { "${EDITOR:-nvim}" "$@"; }
v() { "${EDITOR:-nvim}" "$@"; }
if command -v zed &> /dev/null; then
    alias z='zed'
fi

# --- 网络 ---
alias ip='ip -color=auto'
alias myip='curl -s https://ipinfo.io/ip'
alias ports='ss -tulanp'

# ══════════════════════════════════════════════════════════════════
# 7. 🔨 实用函数 (Functions)
# ══════════════════════════════════════════════════════════════════

# 创建目录并立即进入
mkcd() {
    mkdir -p "$@" && cd "$@"
}

# 万能解压函数 (自动识别格式)
extract() {
    if [ -f "$1" ]; then
        case $1 in
            *.tar.bz2)   tar xjf $1    ;;
            *.tar.gz)    tar xzf $1    ;;
            *.tar.xz)    tar xJf $1    ;;
            *.bz2)       bunzip2 $1    ;;
            *.gz)        gunzip $1     ;;
            *.tar)       tar xf $1     ;;
            *.tbz2)      tar xjf $1    ;;
            *.tgz)       tar xzf $1    ;;
            *.zip)       unzip $1      ;;
            *.Z)         uncompress $1 ;;
            *.7z)        7z x $1       ;;
            *.rar)       unrar x $1    ;;
            *)           echo "'$1' 无法识别的压缩格式" ;;
        esac
    else
        echo "'$1' 不是有效文件"
    fi
}

# fzf: 模糊搜索文件并用 Helix 编辑
fe() {
    local file
    local preview_cmd
    if command -v bat &> /dev/null; then
        preview_cmd='bat --color=always {}'
    else
        preview_cmd='sed -n "1,200p" {}'
    fi
    file=$(fd --type f --hidden --exclude .git | fzf --preview "${preview_cmd}")
    [ -n "$file" ] && "${EDITOR:-nvim}" "$file"
}

# fzf: 模糊搜索目录并进入
fcd() {
    local dir
    dir=$(fd --type d --hidden --exclude .git | fzf --preview 'eza -la --icons {}')
    [ -n "$dir" ] && cd "$dir"
}

# ══════════════════════════════════════════════════════════════════
# 8. 🔧 补全系统 (Completion)
# ══════════════════════════════════════════════════════════════════
autoload -Uz compinit
compinit

# 补全菜单配置
zstyle ':completion:*' menu select
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}'  # 大小写不敏感
zstyle ':completion:*' list-colors "${(s.:.)LS_COLORS}"     # 使用 ls 的颜色
zstyle ':completion:*' group-name ''
zstyle ':completion:*:descriptions' format '%F{magenta}── %d ──%f'
zstyle ':completion:*:messages' format '%F{yellow}%d%f'
zstyle ':completion:*:warnings' format '%F{red}没有找到匹配项%f'

# ══════════════════════════════════════════════════════════════════
# 9. 🔌 第三方工具集成
# ══════════════════════════════════════════════════════════════════

# Fzf 配置 (Catppuccin Mocha 配色)
if command -v fzf &> /dev/null; then
    export FZF_DEFAULT_COMMAND='fd --type f --hidden --follow --exclude .git'
    export FZF_DEFAULT_OPTS='
        --height 40%
        --layout=reverse
        --border=rounded
        --preview-window=right:60%
        --color=bg+:#313244,bg:#1e1e2e,spinner:#f5e0dc,hl:#f38ba8
        --color=fg:#cdd6f4,header:#f38ba8,info:#cba6f7,pointer:#f5e0dc
        --color=marker:#f5e0dc,fg+:#cdd6f4,prompt:#cba6f7,hl+:#f38ba8
    '
    export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"
    export FZF_ALT_C_COMMAND='fd --type d --hidden --follow --exclude .git'
fi

# Direnv (Nix 开发环境神器 - 自动加载 flake.nix)
if command -v direnv &> /dev/null; then
    eval "$(direnv hook zsh)"
fi

# ══════════════════════════════════════════════════════════════════
# 🌟 启动 Starship Prompt (必须在最后)
# ══════════════════════════════════════════════════════════════════
eval "$(starship init zsh)"

# ══════════════════════════════════════════════════════════════════
# 🎉 欢迎语
# ══════════════════════════════════════════════════════════════════
# 如果是 Emacs 环境则不显示，防止卡死
if [[ "$TERM" != "dumb" ]]; then
    if command -v fastfetch &> /dev/null; then
        # fastfetch --logo small
        fastfetch
    fi
fi
