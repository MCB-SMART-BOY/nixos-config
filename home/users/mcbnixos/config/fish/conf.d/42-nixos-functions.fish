# NixOS 管理函数：围绕当前 flake 主机的常用构建与切换入口。
# 当前主线改为调用 Rust 入口，而不是直接在 fish 中拼装系统级流程。

function nrs
    set -l host (_mcb_flake_host)
    if test (count $argv) -ge 1
        set host $argv[1]
    end
    mcbctl rebuild switch $host --upgrade
end

function nrt
    set -l host (_mcb_flake_host)
    if test (count $argv) -ge 1
        set host $argv[1]
    end
    mcbctl rebuild test $host
end

function nrb
    set -l host (_mcb_flake_host)
    if test (count $argv) -ge 1
        set host $argv[1]
    end
    mcbctl rebuild boot $host
end

function nrc
    set -l flake
    if test (count $argv) -ge 1
        set flake $argv[1]
        mcbctl build-host --target $flake --dry-run
    else
        mcbctl build-host --dry-run
    end
end
