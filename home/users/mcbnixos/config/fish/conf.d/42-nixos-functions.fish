# NixOS 管理函数：围绕当前 flake 主机的常用构建与切换入口。

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
