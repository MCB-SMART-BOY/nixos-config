# Fish 环境层：只放环境变量、pager、路径等基础设置。

set -g fish_greeting

if not set -q TERM
    set -gx TERM xterm-256color
end

if not set -q COLORTERM
    set -gx COLORTERM truecolor
end

if not set -q XDG_CONFIG_HOME
    set -gx XDG_CONFIG_HOME $HOME/.config
end
if not set -q XDG_DATA_HOME
    set -gx XDG_DATA_HOME $HOME/.local/share
end
if not set -q XDG_CACHE_HOME
    set -gx XDG_CACHE_HOME $HOME/.cache
end
if not set -q XDG_STATE_HOME
    set -gx XDG_STATE_HOME $HOME/.local/state
end

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

if not set -q PAGER
    set -gx PAGER less
end

if not set -q LESS
    set -gx LESS '--RAW-CONTROL-CHARS --LONG-PROMPT --ignore-case --quit-if-one-screen --tabs=4'
end

if command -q delta
    set -gx GIT_PAGER delta
    set -gx DELTA_PAGER less
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

if not set -q OLDPWD
    set -gx OLDPWD $PWD
end
