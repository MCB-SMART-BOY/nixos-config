# 自定义函数：编辑、NixOS 管理、目录跳转和文件操作。

function _mcb_flake_host
    if test -r /etc/hostname
        head -n 1 /etc/hostname
    else
        hostname
    end
end

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
    if test -z "$file"
        echo "extract: 缺少压缩包路径"
        return 1
    end

    if not test -f "$file"
        echo "'$file' 不是有效文件"
        return 1
    end

    if command -q ouch
        ouch decompress $file
        return $status
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
    if not command -q fzf
        echo "fe: 缺少 fzf"
        return 1
    end

    if not command -q fd
        echo "fe: 缺少 fd"
        return 1
    end

    set -l preview_cmd 'sed -n "1,200p" {}'
    if command -q bat
        set preview_cmd 'bat --color=always --paging=never --line-range=:200 {}'
    end
    set -l file (fd --type f --hidden --exclude .git | fzf --preview "$preview_cmd")
    if test -n "$file"
        e $file
    end
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
