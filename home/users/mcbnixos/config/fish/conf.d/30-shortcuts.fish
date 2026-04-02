# 高频快捷入口：nix、git、cargo 和日常命令。

if command -q zoxide
    alias j='z'
    alias ji='zi'
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

if command -q zed
    alias ze='zed'
end

alias ip='ip -color=auto'
alias myip='curl -s https://ipinfo.io/ip'
alias ports='ss -tulanp'
