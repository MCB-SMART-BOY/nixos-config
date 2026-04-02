# 启动视觉层：只在真正的交互式登录 shell 展示欢迎信息。

if not status is-interactive
    return
end

if not status is-login
    return
end

if test "$TERM" = dumb
    return
end

if set -q CI
    return
end

if command -q fastfetch
    fastfetch
end
