# 核心函数：共享小工具与编辑器入口。

function _mcb_flake_host
    if test -r /etc/hostname
        head -n 1 /etc/hostname
    else
        hostname
    end
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
