# Fish 环境层：只放环境变量、pager、路径等基础设置。

set -g fish_greeting

if not set -q TERM
    set -gx TERM xterm-256color
end

if not set -q OLDPWD
    set -gx OLDPWD $PWD
end
