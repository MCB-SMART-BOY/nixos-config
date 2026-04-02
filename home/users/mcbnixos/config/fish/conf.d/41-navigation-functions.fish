# 目录与导航函数：cd 包装、快速建目录与模糊跳转。

function cd --description 'cd with zoxide fallback' --wraps cd
    set -l current_dir $PWD

    if not status is-interactive
        builtin cd $argv
        and set -gx OLDPWD $current_dir
        return $status
    end

    if test (count $argv) -eq 0
        builtin cd ~
        and set -gx OLDPWD $current_dir
        return
    end

    if test "$argv[1]" = "-"
        if not set -q OLDPWD
            echo "cd: OLDPWD 未设置"
            return 1
        end

        set -l target $OLDPWD
        builtin cd $target
        and set -gx OLDPWD $current_dir
        and printf '%s\n' $PWD
        return
    end

    if test -d "$argv[1]"
        builtin cd $argv
        and set -gx OLDPWD $current_dir
        return
    end

    if string match -qr '^~' -- "$argv[1]"
        set -l expanded_target (string replace -r '^~' $HOME -- "$argv[1]")
        builtin cd $expanded_target
        and set -gx OLDPWD $current_dir
        return
    end

    if string match -qr '^(\.|\.\.|/|~)' -- "$argv[1]"
        builtin cd $argv
        and set -gx OLDPWD $current_dir
        return
    end

    if command -q z
        z $argv
        and set -gx OLDPWD $current_dir
    else
        builtin cd $argv
        and set -gx OLDPWD $current_dir
    end
end

function mkcd
    set -l target $argv[1]
    if test -z "$target"
        echo "mkcd: 缺少目录名"
        return 1
    end
    mkdir -p $argv; and cd $target
end

function fcd
    if not command -q fzf
        echo "fcd: 缺少 fzf"
        return 1
    end

    if not command -q fd
        echo "fcd: 缺少 fd"
        return 1
    end

    set -l preview_cmd 'ls -la {}'
    if command -q eza
        set preview_cmd 'eza -la --icons {}'
    end

    set -l dir (fd --type d --hidden --exclude .git | fzf --preview "$preview_cmd")
    if test -n "$dir"
        cd $dir
    end
end
