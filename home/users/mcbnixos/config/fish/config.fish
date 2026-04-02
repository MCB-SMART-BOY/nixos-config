# Fish 配置
# ~/.config/fish/config.fish

set -g fish_greeting
set -gx TERM xterm-256color

if not set -q EDITOR
    if command -q hx
        set -gx EDITOR hx
        set -gx VISUAL hx
    else
        set -gx EDITOR nvim
        set -gx VISUAL nvim
    end
end

if not set -q BROWSER
    set -gx BROWSER firefox
end

if not set -q MANPAGER
    if command -q bat
        set -gx MANPAGER "sh -c 'col -bx | bat -l man -p'"
    else
        set -gx MANPAGER "less -R"
    end
end

set -gx RUSTUP_HOME $HOME/.rustup
set -gx CARGO_HOME $HOME/.cargo
set -gx GOPATH $HOME/go

if command -q fish_add_path
    fish_add_path --move --prepend /run/wrappers/bin
    fish_add_path --move --prepend $CARGO_HOME/bin
    fish_add_path --move --prepend $GOPATH/bin
    fish_add_path --move --prepend $HOME/.local/bin
end

if status is-interactive
    if command -q atuin
        atuin init fish | source
    end
end

if command -q eza
    alias ls='eza --icons --group-directories-first --git'
    alias ll='eza -l --icons --group-directories-first --git --time-style=long-iso'
    alias la='eza -la --icons --group-directories-first --git'
    alias tree='eza --tree --icons'
end

if command -q bat
    alias bcat='bat --paging=never --style=plain'
    alias catt='bat --paging=always'
end

if command -q broot
    alias br='broot'
end

if command -q tldr
    alias tl='tldr'
end

alias grep='grep --color=auto'

if command -q fd
    alias fdf='fd'
end

if command -q duf
    alias df='duf'
end

if command -q dust
    alias du='dust'
end

if command -q procs
    alias ps='procs'
end

if command -q btop
    alias top='btop'
end

if command -q zoxide
    alias j='z'
    alias ji='zi'
end

alias oldls='command ls'
alias oldcat='command cat'
alias oldgrep='command grep'

function _mcb_flake_host
    if test -r /etc/hostname
        head -n 1 /etc/hostname
    else
        hostname
    end
end

function nrs
    set -l host (_mcb_flake_host)
    if test (count $argv) -ge 1
        set host $argv[1]
    end
    sudo nixos-rebuild switch --flake /etc/nixos#$host --show-trace --upgrade-all
end

function nrt
    set -l host (_mcb_flake_host)
    if test (count $argv) -ge 1
        set host $argv[1]
    end
    sudo nixos-rebuild test --flake /etc/nixos#$host --show-trace
end

function nrb
    set -l host (_mcb_flake_host)
    if test (count $argv) -ge 1
        set host $argv[1]
    end
    sudo nixos-rebuild boot --flake /etc/nixos#$host --show-trace
end

function nrc
    set -l flake
    if test (count $argv) -ge 1
        set flake $argv[1]
    else
        set -l host (_mcb_flake_host)
        set flake /etc/nixos#nixosConfigurations.$host.config.system.build.toplevel
    end
    nix build $flake --dry-run --accept-flake-config
end

alias nfu='nix flake update'
alias nsp='nix search nixpkgs'
alias nsh='nix-shell'
alias ngc='sudo nix-collect-garbage -d'
alias timers='systemctl list-timers --all'
alias utimers='systemctl --user list-timers --all'

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
alias lg='lazygit'

alias c='cargo'
alias cb='cargo build'
alias cr='cargo run'
alias ct='cargo test'
alias cc='cargo check'
alias cw='cargo watch -x check'
alias cf='cargo fmt'
alias ccl='cargo clippy'

alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias md='mkdir -p'
alias rd='rmdir'
alias cp='cp -i'
alias mv='mv -i'
alias rm='rm -i'

function e
    set -l editor $EDITOR
    if test -z "$editor"
        set editor nvim
    end
    $editor $argv
end

function v
    e $argv
end

if command -q zed
    alias ze='zed'
end

alias ip='ip -color=auto'
alias myip='curl -s https://ipinfo.io/ip'
alias ports='ss -tulanp'

function mkcd
    set -l target $argv[1]
    if test -z "$target"
        echo "mkcd: 缺少目录名"
        return 1
    end
    mkdir -p $argv; and cd $target
end

function extract
    set -l file $argv[1]
    if not test -f "$file"
        echo "'$file' 不是有效文件"
        return 1
    end

    switch $file
        case '*.tar.bz2'
            tar xjf $file
        case '*.tar.gz' '*.tgz'
            tar xzf $file
        case '*.tar.xz'
            tar xJf $file
        case '*.bz2'
            bunzip2 $file
        case '*.gz'
            gunzip $file
        case '*.tar'
            tar xf $file
        case '*.zip'
            unzip $file
        case '*.Z'
            uncompress $file
        case '*.7z'
            7z x $file
        case '*.rar'
            unrar x $file
        case '*'
            echo "'$file' 无法识别的压缩格式"
            return 1
    end
end

function fe
    set -l preview_cmd 'sed -n "1,200p" {}'
    if command -q bat
        set preview_cmd 'bat --color=always {}'
    end
    set -l file (fd --type f --hidden --exclude .git | fzf --preview "$preview_cmd")
    if test -n "$file"
        e $file
    end
end

function fcd
    set -l dir (fd --type d --hidden --exclude .git | fzf --preview 'eza -la --icons {}')
    if test -n "$dir"
        cd $dir
    end
end

if command -q fzf
    set -gx FZF_DEFAULT_COMMAND 'fd --type f --hidden --follow --exclude .git'
    set -gx FZF_DEFAULT_OPTS '--height 40% --layout=reverse --border=rounded --preview-window=right:60% --color=bg+:#313244,bg:#1e1e2e,spinner:#f5e0dc,hl:#f38ba8 --color=fg:#cdd6f4,header:#f38ba8,info:#cba6f7,pointer:#f5e0dc --color=marker:#f5e0dc,fg+:#cdd6f4,prompt:#cba6f7,hl+:#f38ba8'
    set -gx FZF_CTRL_T_COMMAND $FZF_DEFAULT_COMMAND
    set -gx FZF_ALT_C_COMMAND 'fd --type d --hidden --follow --exclude .git'
end

if status is-interactive
    if test "$TERM" != dumb
        if command -q fastfetch
            fastfetch
        end
    end
end
