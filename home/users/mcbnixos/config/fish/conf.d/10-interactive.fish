# Fish 交互层：键位、历史、交互增强。

if not status is-interactive
    return
end

if test "$TERM" = dumb
    fish_default_key_bindings
else
    fish_vi_key_bindings
    set -g fish_cursor_default block
    set -g fish_cursor_insert line
    set -g fish_cursor_replace_one underscore
    set -g fish_cursor_visual block
end

if command -q atuin
    atuin init fish | source
end
