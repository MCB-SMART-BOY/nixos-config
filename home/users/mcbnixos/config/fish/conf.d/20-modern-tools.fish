# 现代命令替代：只在交互式环境替换旧命令，排障仍可走 old*。

if not status is-interactive
    return
end

if command -q eza
    alias tree='eza --tree --icons'
end

if command -q bat
    alias cat='bat --paging=never --style=plain'
    alias bcat='bat --paging=never --style=plain'
    alias catt='bat --paging=always'
end

if command -q batman
    alias man='batman'
end

if command -q batdiff
    alias diff='batdiff'
end

if command -q batwatch
    alias watch='batwatch'
end

if command -q tldr
    alias tl='tldr'
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

if command -q xh
    alias http='xh'
end

if command -q zellij
    alias zj='zellij'
end

if command -q ouch
    alias archive='ouch'
end

alias grep='grep --color=auto'
alias oldls='command ls'
alias oldcat='command cat'
alias oldgrep='command grep'
alias oldman='command man'
alias olddiff='command diff'
alias oldwatch='command watch'
